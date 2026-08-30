#!/usr/bin/env python3
"""Compare normalized disassembly for explicitly configured codegen roots."""

from __future__ import annotations

import argparse
import difflib
import hashlib
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

from bench_config import CODEGEN_CONFIGS

FUNCTION = re.compile(r"^\s*[0-9a-f]+ <(.+)>:\s*$")
X86_INSTRUCTION = re.compile(r"^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}(?:\s+|$))+)(.*?)\s*$")
AARCH64_INSTRUCTION = re.compile(r"^\s*([0-9a-f]+):\s+([0-9a-f]{8})\s+(.*?)\s*$")
RELOCATION = re.compile(r"^\s*([0-9a-f]+):\s+(R_[A-Z0-9_]+)\s+(.+?)\s*$")
INLINE_RELOCATION = re.compile(r"\s+([0-9a-f]+):\s+(R_[A-Z0-9_]+)\s+(.+?)\s*$")
SYMBOL_TARGET = re.compile(r"(?:0x)?([0-9a-f]+)\s+<([^>]+)>")
PC_RELATIVE_X86_RELOCATIONS = {"R_X86_64_PC32", "R_X86_64_PLT32"}
X86_PC_BIAS = re.compile(r"-0x4$")
ADDEND = re.compile(r"^(?P<base>.*?)(?P<addend>[+-]0x[0-9a-f]+)?$")
ANONYMOUS_SYMBOL_INSTANCE = re.compile(
    r"(?P<stem>(?:\.Lanon|anon)\.[^.\s]+)\.(?P<instance>\d+)"
    r"(?:\.llvm\.[^+\-\s]+)?(?P<addend>[+-]0x[0-9a-f]+)?$"
)
OBJDUMP_COMMENT = re.compile(r"\s+#\s+(?:0x)?[0-9a-f]+\s+<[^>]+>.*$")
BACKEND_MODULE = re.compile(r"::(?:scalar|sse2|ssse3|sse41|avx2|neon)::")


@dataclass
class Instruction:
    address: int
    encoded: bytes
    assembly: str
    relocations: list[tuple[str, str]] = field(default_factory=list)
    member: str = ""


@dataclass(frozen=True)
class Relocation:
    kind: str
    raw_target: str
    token: str
    anonymous: bool
    resolved: bool
    instruction_index: int


@dataclass(frozen=True)
class SectionRelocation:
    offset: int
    kind: str
    target: str


@dataclass(frozen=True)
class SectionData:
    member: str
    name: str
    data: bytes
    relocations: tuple[SectionRelocation, ...]


@dataclass(frozen=True)
class ResolvedTarget:
    token: str
    anonymous: bool
    resolved: bool


class ArtifactSections:
    """Small, conservative index of sections needed by anonymous relocations."""

    SECTION_HEADER = re.compile(r"^Contents of section (.+):$")
    FILE_FORMAT = re.compile(r"^(?P<member>.+):\s+file format .+$")
    RELOCATION_HEADER = re.compile(r"^RELOCATION RECORDS FOR \[(.+)\]:$")
    RELOCATION_ENTRY = re.compile(r"^\s*([0-9a-f]+)\s+(R_[A-Z0-9_]+)\s+(.+?)\s*$")

    def __init__(self, sections: dict[tuple[str, str], SectionData], arch: str) -> None:
        self.sections = sections
        self.arch = arch

    @classmethod
    def from_objdump(cls, output: str, arch: str) -> ArtifactSections:
        member = ""
        section_name: str | None = None
        section_bytes: dict[tuple[str, str], bytearray] = {}
        section_relocations: dict[tuple[str, str], list[SectionRelocation]] = {}
        relocation_section: str | None = None
        for line in output.splitlines():
            file_format = cls.FILE_FORMAT.match(line)
            if file_format:
                member = file_format.group("member")
                section_name = None
                relocation_section = None
                continue
            section = cls.SECTION_HEADER.match(line)
            if section:
                section_name = section.group(1)
                section_bytes.setdefault((member, section_name), bytearray())
                relocation_section = None
                continue
            relocation_header = cls.RELOCATION_HEADER.match(line)
            if relocation_header:
                relocation_section = relocation_header.group(1)
                section_relocations.setdefault((member, relocation_section), [])
                section_name = None
                continue
            relocation = cls.RELOCATION_ENTRY.match(line)
            if relocation and relocation_section is not None:
                section_relocations[(member, relocation_section)].append(
                    SectionRelocation(int(relocation.group(1), 16), relocation.group(2), relocation.group(3))
                )
                continue
            if section_name is None:
                continue
            fields = line.split()
            if len(fields) < 2 or not re.fullmatch(r"[0-9a-f]+", fields[0]):
                continue
            chunks: list[str] = []
            for field in fields[1:]:
                if len(field) in {2, 4, 6, 8} and len(field) % 2 == 0 and re.fullmatch(r"[0-9a-fA-F]+", field):
                    chunks.append(field)
                else:
                    break
            if not chunks:
                continue
            data = section_bytes[(member, section_name)]
            offset = int(fields[0], 16)
            encoded = bytes.fromhex("".join(chunks))
            end = offset + len(encoded)
            if len(data) < end:
                data.extend(b"\0" * (end - len(data)))
            data[offset:end] = encoded
        sections = {
            key: SectionData(key[0], key[1], bytes(data), tuple(section_relocations.get(key, ())))
            for key, data in section_bytes.items()
        }
        return cls(sections, arch)

    def _find(self, member: str, name: str) -> SectionData | None:
        exact = self.sections.get((member, name))
        if exact is not None:
            return exact
        candidates = [
            section
            for (candidate_member, candidate_name), section in self.sections.items()
            if candidate_member == member
            and (candidate_name == name or candidate_name.endswith(f".{name}"))
        ]
        if not candidates:
            candidates = [
                section
                for (candidate_member, candidate_name), section in self.sections.items()
                if candidate_name == name or candidate_name.endswith(f".{name}")
            ]
        return candidates[0] if len(candidates) == 1 else None

    @staticmethod
    def _split_target(value: str) -> tuple[str, str]:
        match = ADDEND.match(value.strip())
        assert match is not None
        return match.group("base"), match.group("addend") or ""

    @staticmethod
    def _addend(value: str) -> int:
        return int(value, 16) if value.startswith("+0x") else -int(value[3:], 16)

    def _string_target(self, member: str, name: str, addend: str) -> str | None:
        if not name.startswith(".rodata.str"):
            return None
        section = self._find(member, name)
        if section is None:
            return None
        offset = self._addend(addend) if addend else 0
        if offset < 0 or offset >= len(section.data):
            return None
        value = section.data[offset:]
        return value.split(b"\0", 1)[0].hex()

    def _fingerprint(self, section: SectionData, member: str) -> str | None:
        data = bytearray(section.data)
        descriptors: list[str] = []
        widths = {
            "R_X86_64_64": 8,
            "R_X86_64_32": 4,
            "R_X86_64_PC32": 4,
            "R_X86_64_PLT32": 4,
            "R_X86_64_GOTPCREL": 4,
            "R_AARCH64_ABS64": 8,
            "R_AARCH64_ADR_PREL_PG_HI21": 4,
            "R_AARCH64_ADD_ABS_LO12_NC": 4,
            "R_AARCH64_LDST64_ABS_LO12_NC": 4,
        }
        for relocation in section.relocations:
            width = widths.get(relocation.kind)
            if width is None or relocation.offset < 0 or relocation.offset + width > len(data):
                return None
            data[relocation.offset : relocation.offset + width] = b"\0" * width
            target = relocation.target
            if target in {"", ".text"}:
                descriptor_target = target
            else:
                target_name, addend = self._split_target(target)
                if _anonymous_instance(target_name) is not None:
                    return None
                string = self._string_target(member, target_name, addend)
                descriptor_target = f"string:{string}" if string is not None else f"symbol:{target_name}{addend}"
            descriptors.append(f"{relocation.offset}:{relocation.kind}:{descriptor_target}")
        payload = bytes(data) + b"\0" + "\n".join(descriptors).encode()
        return hashlib.sha256(payload).hexdigest()

    def resolve(self, member: str, kind: str, value: str) -> ResolvedTarget:
        target = value.strip()
        if kind in PC_RELATIVE_X86_RELOCATIONS:
            target = X86_PC_BIAS.sub("", target)
        anonymous = _anonymous_instance(target)
        if anonymous is None:
            name, addend = self._split_target(target)
            string = self._string_target(member, name, addend)
            token = f"string:{string}" if string is not None else f"symbol:{name}{addend}"
            return ResolvedTarget(token, False, True)
        name, addend = self._split_target(target)
        section = self._find(member, name)
        if section is None:
            return ResolvedTarget(f"anonymous-unresolved:{anonymous[0]}", True, False)
        fingerprint = self._fingerprint(section, member)
        if fingerprint is None:
            return ResolvedTarget(f"anonymous-unresolved:{anonymous[0]}", True, False)
        return ResolvedTarget(f"anonymous:{fingerprint}:{addend}", True, True)


@dataclass(frozen=True)
class Function:
    instructions: tuple[str, ...]
    encoded: bytes
    calls: int
    branches: int
    executable: tuple[str, ...] = ()
    relocations: tuple[Relocation, ...] = ()
    alias_partition: tuple[int, ...] = ()

    @property
    def digest(self) -> str:
        return hashlib.sha256("\n".join(self.executable).encode()).hexdigest()[:16]


@dataclass(frozen=True)
class Root:
    symbol: str
    artifact: str
    targets: frozenset[str]


def classify(old: Function, new: Function) -> str:
    if old.executable != new.executable:
        return "CODEGEN CHANGED"
    if old.encoded != new.encoded:
        return "CODEGEN ENCODING CHANGED"
    if _same_relocations(old.relocations, new.relocations) and old.alias_partition == new.alias_partition:
        return "CODEGEN IDENTICAL"
    if _metadata_only_relocations(old, new):
        return "CODEGEN METADATA CHANGED"
    return "CODEGEN CHANGED"


def _same_relocations(old: tuple[Relocation, ...], new: tuple[Relocation, ...]) -> bool:
    return all(
        before.kind == after.kind
        and before.token == after.token
        and before.anonymous == after.anonymous
        and before.resolved == after.resolved
        and (not before.anonymous or before.resolved)
        for before, after in zip(old, new)
    ) and len(old) == len(new)


def _metadata_only_relocations(old: Function, new: Function) -> bool:
    if old.alias_partition != new.alias_partition or len(old.relocations) != len(new.relocations):
        return False
    changed = False
    for before, after in zip(old.relocations, new.relocations):
        if before.kind != after.kind or before.anonymous != after.anonymous:
            return False
        if before.token != after.token:
            if not before.anonymous or not before.resolved or not after.resolved:
                return False
            changed = True
    return changed


def _anonymous_instance(value: str) -> tuple[str, int, str] | None:
    match = ANONYMOUS_SYMBOL_INSTANCE.search(value)
    if match is None:
        return None
    key = value[: match.start("stem")] + match.group("stem")
    return key, int(match.group("instance")), match.group("addend") or ""


def _relocation_target(kind: str, value: str, anonymous_bases: dict[str, int]) -> str:
    value = value.strip()
    if kind in PC_RELATIVE_X86_RELOCATIONS:
        value = X86_PC_BIAS.sub("", value)
    anonymous = _anonymous_instance(value)
    if anonymous is None:
        return value
    key, instance, addend = anonymous
    base = anonymous_bases[key]
    return f"{key}.@{instance - base}{addend}"


def _is_call(mnemonic: str, arch: str) -> bool:
    return mnemonic.startswith("call") if arch == "x86_64" else mnemonic in {"bl", "blr"}


def _is_conditional_branch(mnemonic: str, arch: str) -> bool:
    if arch == "x86_64":
        return mnemonic.startswith("j") and mnemonic not in {"jmp", "jmpq"}
    return mnemonic.startswith("b.") or mnemonic in {"cbz", "cbnz", "tbz", "tbnz"}


def _has_control_target(mnemonic: str, arch: str) -> bool:
    if arch == "x86_64":
        return mnemonic.startswith(("j", "call", "loop"))
    return mnemonic in {"b", "bl", "blr", "br", "cbz", "cbnz", "tbz", "tbnz"} or mnemonic.startswith("b.")


def _normalize(instructions: list[Instruction], arch: str, sections: ArtifactSections | None = None) -> Function:
    indexes = {instruction.address: index for index, instruction in enumerate(instructions)}
    anonymous_bases: dict[str, int] = {}
    for instruction in instructions:
        for _, target in instruction.relocations:
            anonymous = _anonymous_instance(target.strip())
            if anonymous is None:
                continue
            key, instance, _ = anonymous
            anonymous_bases[key] = min(instance, anonymous_bases.get(key, instance))
    normalized: list[str] = []
    executable: list[str] = []
    relocations: list[Relocation] = []
    aliases: dict[str, int] = {}
    alias_partition: list[int] = []
    calls = branches = 0
    for instruction_index, instruction in enumerate(instructions):
        assembly = OBJDUMP_COMMENT.sub("", instruction.assembly).strip()
        if not assembly:
            continue
        parts = assembly.split(None, 1)
        mnemonic = parts[0]
        operands = parts[1] if len(parts) == 2 else ""
        calls += _is_call(mnemonic, arch)
        branches += _is_conditional_branch(mnemonic, arch)
        if _has_control_target(mnemonic, arch):
            match = SYMBOL_TARGET.search(operands)
            if match:
                address = int(match.group(1), 16)
                symbol = match.group(2)
                target = f"@instruction_{indexes[address]}" if address in indexes and "+0x" in symbol else f"<{symbol}>"
                operands = SYMBOL_TARGET.sub(target, operands)
        relocation_text: list[str] = []
        for kind, target in instruction.relocations:
            if sections is None:
                token = _relocation_target(kind, target, anonymous_bases)
                anonymous = _anonymous_instance(target) is not None
                resolved = True
            else:
                resolved_target = sections.resolve(instruction.member, kind, target)
                token = resolved_target.token
                anonymous = resolved_target.anonymous
                resolved = resolved_target.resolved
            relocation_text.append(f"[{kind}:{token}]")
            relocations.append(Relocation(kind, target, token, anonymous, resolved, instruction_index))
            if anonymous:
                alias_key = target
                if kind in PC_RELATIVE_X86_RELOCATIONS:
                    alias_key = X86_PC_BIAS.sub("", alias_key)
                alias = aliases.setdefault(alias_key, len(aliases))
                alias_partition.append(alias)
        executable.append(" ".join(f"{mnemonic} {operands}".split()))
        normalized.append(" ".join(f"{mnemonic} {operands} {' '.join(relocation_text)}".split()))
    return Function(
        tuple(normalized),
        b"".join(item.encoded for item in instructions),
        calls,
        branches,
        tuple(executable),
        tuple(relocations),
        tuple(alias_partition),
    )


def parse_objdump(output: str, arch: str, sections: ArtifactSections | None = None) -> dict[str, Function]:
    instruction_pattern = X86_INSTRUCTION if arch == "x86_64" else AARCH64_INSTRUCTION
    raw: dict[str, list[Instruction]] = {}
    current: str | None = None
    current_member = ""
    last_instruction: Instruction | None = None
    for line in output.splitlines():
        file_format = ArtifactSections.FILE_FORMAT.match(line)
        if file_format:
            current_member = file_format.group("member")
            continue
        label = FUNCTION.match(line)
        if label:
            current = label.group(1)
            if current in raw:
                raise ValueError(f"duplicate disassembly symbol: {current}")
            raw[current] = []
            last_instruction = None
            continue
        if current is None:
            continue
        relocation = RELOCATION.match(line)
        if relocation:
            if last_instruction is not None:
                last_instruction.relocations.append((relocation.group(2), relocation.group(3)))
            continue
        match = instruction_pattern.match(line)
        if not match:
            continue
        encoded = bytes.fromhex(match.group(2))
        assembly = match.group(3).strip()
        relocations: list[tuple[str, str]] = []
        inline_relocation = INLINE_RELOCATION.search(assembly)
        if inline_relocation is not None:
            assembly = assembly[: inline_relocation.start()].rstrip()
            relocations.append((inline_relocation.group(2), inline_relocation.group(3)))
        if not assembly:
            continue
        last_instruction = Instruction(int(match.group(1), 16), encoded, assembly, relocations, current_member)
        raw[current].append(last_instruction)
    return {name: _normalize(items, arch, sections) for name, items in raw.items() if items}


def load_functions(artifact: Path, objdump: str, arch: str) -> dict[str, Function]:
    output = subprocess.check_output([objdump, "-drwC", str(artifact)], text=True, errors="replace")
    section_output = subprocess.check_output([objdump, "-sr", str(artifact)], text=True, errors="replace")
    sections = ArtifactSections.from_objdump(section_output, arch)
    return parse_objdump(output, arch, sections)


def load_roots(path: Path) -> list[Root]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    roots = [Root(item["symbol"], item["artifact"], frozenset(item["targets"])) for item in data.get("root", [])]
    identities = [(root.artifact, root.symbol) for root in roots]
    if len(identities) != len(set(identities)):
        raise ValueError(f"{path}: duplicate codegen root")
    unknown_targets = set().union(*(root.targets for root in roots)) - CODEGEN_CONFIGS.keys()
    if unknown_targets:
        raise ValueError(f"{path}: unknown target configurations: {sorted(unknown_targets)}")
    return roots


def load_kernel_modules(path: Path) -> tuple[str, ...]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    modules = []
    for kernel in data.get("kernel", []):
        source = Path(kernel["path"])
        if source.parts[0] != "src" or source.suffix != ".rs":
            raise ValueError(f"{path}: invalid kernel source path: {source}")
        modules.append("bit_string::" + "::".join(source.with_suffix("").parts[1:]))
    return tuple(modules)


def independent_backend_symbols(functions: dict[str, Function], modules: tuple[str, ...]) -> set[str]:
    return {
        symbol
        for symbol in functions
        if BACKEND_MODULE.search(symbol) and any(symbol.startswith(f"{module}::") for module in modules)
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--roots", type=Path, required=True)
    parser.add_argument("--artifact", required=True)
    parser.add_argument("--inventory", type=Path)
    parser.add_argument("--target", required=True)
    parser.add_argument("--arch", choices=("x86_64", "aarch64"), required=True)
    parser.add_argument("--objdump", default="objdump")
    args = parser.parse_args()
    before = load_functions(args.baseline, args.objdump, args.arch)
    after = load_functions(args.current, args.objdump, args.arch)
    roots = [root for root in load_roots(args.roots) if root.artifact == args.artifact]
    identical = metadata_changed = encoding_changed = changed = errors = skipped = 0
    if args.artifact == "library":
        if args.inventory is None:
            parser.error("--inventory is required for library artifacts")
        modules = load_kernel_modules(args.inventory)
        registered = {root.symbol for root in roots if args.target in root.targets}
        discovered = independent_backend_symbols(before, modules) | independent_backend_symbols(after, modules)
        for symbol in sorted(discovered - registered):
            errors += 1
            print(f"ERROR: independent backend symbol is not a configured root: {symbol}", file=sys.stderr)
    for root in roots:
        if args.target not in root.targets:
            skipped += 1
            continue
        old, new = before.get(root.symbol), after.get(root.symbol)
        if old is None or new is None:
            errors += 1
            where = "both" if old is None and new is None else "baseline" if old is None else "current"
            print(f"ERROR: expected symbol missing in {where}: {root.symbol}", file=sys.stderr)
            continue
        same_instructions = old.instructions == new.instructions
        status = classify(old, new)
        if status == "CODEGEN IDENTICAL":
            identical += 1
        elif status == "CODEGEN METADATA CHANGED":
            metadata_changed += 1
        elif status == "CODEGEN ENCODING CHANGED":
            encoding_changed += 1
        else:
            changed += 1
        print(f"{root.symbol}: {status}")
        print(f"  text bytes    {len(old.encoded)} -> {len(new.encoded)}")
        print(f"  instructions  {len(old.instructions)} -> {len(new.instructions)}")
        print(f"  calls         {old.calls} -> {new.calls}")
        print(f"  branches      {old.branches} -> {new.branches}")
        print(f"  asm hash      {old.digest} -> {new.digest}")
        if status == "CODEGEN METADATA CHANGED":
            for old_relocation, new_relocation in zip(old.relocations, new.relocations):
                if old_relocation.token != new_relocation.token:
                    print(f"  metadata      {old_relocation.raw_target} -> {new_relocation.raw_target}")
                    print(f"  semantic      {old_relocation.token} -> {new_relocation.token}")
        if new.calls > old.calls:
            print(f"WARNING: {root.symbol} call count increased from {old.calls} to {new.calls}", file=sys.stderr)
        if not same_instructions:
            print(
                "\n".join(
                    difflib.unified_diff(old.instructions, new.instructions, "baseline", "current", lineterm="")
                )
            )
    print("\nCODEGEN SUMMARY")
    print(f"compared         : {identical + metadata_changed + encoding_changed + changed}")
    print(f"identical        : {identical}")
    print(f"metadata changed : {metadata_changed}")
    print(f"encoding changed : {encoding_changed}")
    print(f"changed          : {changed}")
    print(f"errors           : {errors}")
    print(f"skipped          : {skipped}")
    return 0 if not (metadata_changed or encoding_changed or changed or errors) else 1


if __name__ == "__main__":
    raise SystemExit(main())

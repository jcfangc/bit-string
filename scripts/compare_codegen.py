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
INSTRUCTION = re.compile(r"^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}(?:\s+|$))+)(.*?)\s*$")
RELOCATION = re.compile(r"^\s*([0-9a-f]+):\s+(R_[A-Z0-9_]+)\s+(.+?)\s*$")
SYMBOL_TARGET = re.compile(r"(?:0x)?([0-9a-f]+)\s+<([^>]+)>")
RELOCATION_ADDEND = re.compile(r"[+-]0x[0-9a-f]+$")
OBJDUMP_COMMENT = re.compile(r"\s+#\s+(?:0x)?[0-9a-f]+\s+<[^>]+>.*$")


@dataclass
class Instruction:
    address: int
    encoded: bytes
    assembly: str
    relocations: list[tuple[str, str]] = field(default_factory=list)


@dataclass(frozen=True)
class Function:
    instructions: tuple[str, ...]
    encoded: bytes
    calls: int
    branches: int

    @property
    def digest(self) -> str:
        return hashlib.sha256("\n".join(self.instructions).encode()).hexdigest()[:16]


@dataclass(frozen=True)
class Root:
    symbol: str
    artifact: str
    targets: frozenset[str]


def classify(old: Function, new: Function) -> str:
    if old.instructions != new.instructions:
        return "CODEGEN CHANGED"
    if len(old.encoded) != len(new.encoded):
        return "CODEGEN ENCODING CHANGED"
    return "CODEGEN IDENTICAL"


def _relocation_target(value: str) -> str:
    return RELOCATION_ADDEND.sub("", value.strip())


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


def _normalize(instructions: list[Instruction], arch: str) -> Function:
    indexes = {instruction.address: index for index, instruction in enumerate(instructions)}
    normalized: list[str] = []
    calls = branches = 0
    for instruction in instructions:
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
        relocations = " ".join(f"[{kind}:{_relocation_target(target)}]" for kind, target in instruction.relocations)
        normalized.append(" ".join(f"{mnemonic} {operands} {relocations}".split()))
    return Function(tuple(normalized), b"".join(item.encoded for item in instructions), calls, branches)


def parse_objdump(output: str, arch: str) -> dict[str, Function]:
    raw: dict[str, list[Instruction]] = {}
    current: str | None = None
    last_instruction: Instruction | None = None
    for line in output.splitlines():
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
        match = INSTRUCTION.match(line)
        if not match:
            continue
        encoded = bytes.fromhex(match.group(2))
        assembly = match.group(3).strip()
        if not assembly:
            continue
        last_instruction = Instruction(int(match.group(1), 16), encoded, assembly)
        raw[current].append(last_instruction)
    return {name: _normalize(items, arch) for name, items in raw.items() if items}


def load_functions(artifact: Path, objdump: str, arch: str) -> dict[str, Function]:
    output = subprocess.check_output([objdump, "-drwC", str(artifact)], text=True, errors="replace")
    return parse_objdump(output, arch)


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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--roots", type=Path, required=True)
    parser.add_argument("--artifact", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--arch", choices=("x86_64", "aarch64"), required=True)
    parser.add_argument("--objdump", default="objdump")
    args = parser.parse_args()
    before = load_functions(args.baseline, args.objdump, args.arch)
    after = load_functions(args.current, args.objdump, args.arch)
    roots = [root for root in load_roots(args.roots) if root.artifact == args.artifact]
    identical = encoding_changed = changed = errors = skipped = 0
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
        if new.calls > old.calls:
            print(f"WARNING: {root.symbol} call count increased from {old.calls} to {new.calls}", file=sys.stderr)
        if not same_instructions:
            print("\n".join(difflib.unified_diff(old.instructions, new.instructions, "baseline", "current", lineterm="")))
    print("\nCODEGEN SUMMARY")
    print(f"compared         : {identical + encoding_changed + changed}")
    print(f"identical        : {identical}")
    print(f"encoding changed : {encoding_changed}")
    print(f"changed          : {changed}")
    print(f"errors           : {errors}")
    print(f"skipped          : {skipped}")
    return 0 if not (encoding_changed or changed or errors) else 1


if __name__ == "__main__":
    raise SystemExit(main())

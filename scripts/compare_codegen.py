#!/usr/bin/env python3
"""Compare normalized release-rlib disassembly for selected codegen roots."""

from __future__ import annotations

import argparse
import hashlib
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

FUNCTION = re.compile(r"^\s*[0-9a-f]+ <(.+)>:\s*$")
INSTRUCTION = re.compile(r"^\s*[0-9a-f]+:\s+(?:[0-9a-f]{2}(?:\s+[0-9a-f]{2})*\s+)?(.+?)\s*$")
TARGET_INSTRUCTION = re.compile(r"^(?:j[a-z]+|call|loop[a-z]*)(?:\s|$)")
CONDITIONAL_BRANCH = re.compile(r"^(?:ja|jae|jb|jbe|jc|je|jg|jge|jl|jle|jna|jnae|jnb|jnbe|jnc|jne|jng|jnge|jnl|jnle|jno|jnp|jns|jnz|jo|jp|jpe|jpo|js|jz)(?:\s|$)")
TARGET = re.compile(r"(?:0x[0-9a-f]+|[0-9a-f]+)\s+<[^>]+>")


@dataclass(frozen=True)
class Function:
    instructions: tuple[str, ...]
    bytes: int
    calls: int
    branches: int

    @property
    def digest(self) -> str:
        return hashlib.sha256("\n".join(self.instructions).encode()).hexdigest()[:16]


def normalized_instruction(text: str) -> str:
    text = text.split("#", 1)[0].strip()
    if not text:
        return ""
    if TARGET_INSTRUCTION.match(text):
        text = TARGET.sub("<target>", text)
    return " ".join(text.split())


def load_functions(artifact: Path, objdump: str) -> dict[str, Function]:
    output = subprocess.check_output([objdump, "-drwC", str(artifact)], text=True, errors="replace")
    found: dict[str, list[str]] = {}
    sizes: dict[str, int] = {}
    current: str | None = None
    for line in output.splitlines():
        label = FUNCTION.match(line)
        if label:
            current = label.group(1)
            found.setdefault(current, [])
            sizes.setdefault(current, 0)
            continue
        if current is None:
            continue
        match = INSTRUCTION.match(line)
        if not match:
            continue
        raw = match.group(1)
        normalized = normalized_instruction(raw)
        if not normalized:
            continue
        found[current].append(normalized)
        # The byte column is the sequence before the mnemonic. Count bytes from
        # the source line rather than estimating from the mnemonic.
        prefix = line.split(":", 1)[1].split(raw, 1)[0]
        sizes[current] += len(re.findall(r"\b[0-9a-f]{2}\b", prefix))
    result: dict[str, Function] = {}
    for name, instructions in found.items():
        if not instructions:
            continue
        calls = sum(1 for item in instructions if item.startswith("call "))
        branches = sum(1 for item in instructions if CONDITIONAL_BRANCH.match(item))
        result[name] = Function(tuple(instructions), sizes[name], calls, branches)
    return result


def symbols(path: Path) -> list[str]:
    return [line.strip() for line in path.read_text().splitlines() if line.strip() and not line.lstrip().startswith("#")]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path)
    parser.add_argument("current", type=Path)
    parser.add_argument("--symbols", type=Path, required=True)
    parser.add_argument("--objdump", default="objdump")
    args = parser.parse_args()
    before, after = load_functions(args.baseline, args.objdump), load_functions(args.current, args.objdump)
    changed = missing = unavailable = 0
    names = symbols(args.symbols)
    print(f"{'symbol':<78} {'bytes':>13} {'insns':>13} {'calls':>11} {'branches':>14}  status")
    print("-" * 140)
    for name in names:
        old, new = before.get(name), after.get(name)
        if old is None or new is None:
            if old is None and new is None:
                unavailable += 1
                status = "UNAVAILABLE"
            else:
                missing += 1
                status = "MISSING"
            print(f"{name:<78} {'-':>6} -> {'-':<6} {'-':>6} -> {'-':<6} {'-':>5} -> {'-':<5} {'-':>7} -> {'-':<7}  {status}")
            continue
        identical = old.instructions == new.instructions
        if not identical:
            changed += 1
        status = "CODEGEN IDENTICAL" if identical else "CODEGEN CHANGED"
        print(f"{name:<78} {old.bytes:>6} -> {new.bytes:<6} {len(old.instructions):>6} -> {len(new.instructions):<6} {old.calls:>5} -> {new.calls:<5} {old.branches:>7} -> {new.branches:<7} {old.digest} -> {new.digest}  {status}")
        if new.calls > old.calls:
            print(f"WARNING: {name} call count increased from {old.calls} to {new.calls}", file=sys.stderr)
        if not identical:
            print(f"--- baseline {name}\n+++ current {name}")
            for left, right in zip(old.instructions, new.instructions):
                if left != right:
                    print(f"- {left}\n+ {right}")
                    break
            if len(old.instructions) != len(new.instructions):
                print(f"instruction count: {len(old.instructions)} -> {len(new.instructions)}")
    print("\nCODEGEN SUMMARY")
    print(f"compared       : {len(names) - missing - unavailable}")
    print(f"identical      : {len(names) - missing - unavailable - changed}")
    print(f"changed        : {changed}")
    print(f"missing        : {missing}")
    print(f"unavailable    : {unavailable}")
    return 0 if not changed and not missing else 1


if __name__ == "__main__":
    raise SystemExit(main())

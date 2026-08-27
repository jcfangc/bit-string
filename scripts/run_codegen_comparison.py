#!/usr/bin/env python3
"""Build the fixed baseline and current worktree, then compare release codegen."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

from bench_config import BASELINE_SHA, RUSTFLAGS


def run(command: list[str], cwd: Path, env: dict[str, str]) -> None:
    print(f"[{cwd}] {' '.join(command)}", flush=True)
    completed = subprocess.run(command, cwd=cwd, env=env, check=False)
    if completed.returncode:
        raise SystemExit(completed.returncode)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", default=BASELINE_SHA)
    parser.add_argument("--symbols", type=Path, default=None)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = root / "target" / "codegen-comparison"
    baseline = output / "baseline-worktree"
    output.mkdir(parents=True, exist_ok=True)
    if not baseline.exists():
        run(["git", "worktree", "add", "--detach", str(baseline), args.baseline], root, os.environ.copy())
    else:
        run(["git", "-C", str(baseline), "checkout", "--detach", args.baseline], root, os.environ.copy())
    env = os.environ.copy()
    env["RUSTFLAGS"] = RUSTFLAGS
    command = ["cargo", "rustc", "--release", "--lib", "--", "--emit=obj"]
    run(command, baseline, env)
    run(command, root, env)
    symbols = args.symbols or (root / "scripts" / "codegen_roots.txt")
    compare = [sys.executable, str(root / "scripts" / "compare_codegen.py"), str(baseline / "target" / "release" / "libbit_string.rlib"), str(root / "target" / "release" / "libbit_string.rlib"), "--symbols", str(symbols)]
    return subprocess.run(compare, cwd=root, env=env, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())

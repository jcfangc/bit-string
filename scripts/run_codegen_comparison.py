#!/usr/bin/env python3
"""Build fixed-feature baseline/current artifacts and compare their codegen."""

from __future__ import annotations

import argparse
import os
import platform
import shlex
import subprocess
import sys
from pathlib import Path

from bench_config import BASELINE_SHA, CODEGEN_CONFIGS


def run(command: list[str], cwd: Path, env: dict[str, str]) -> None:
    print(f"[{cwd}] {' '.join(command)}", flush=True)
    completed = subprocess.run(command, cwd=cwd, env=env, check=False)
    if completed.returncode:
        raise SystemExit(completed.returncode)


def build_library(worktree: Path, target_dir: Path, rustflags: str) -> Path:
    env = os.environ.copy()
    env["RUSTFLAGS"] = rustflags
    env["CARGO_TARGET_DIR"] = str(target_dir)
    run(["cargo", "build", "--release", "--lib"], worktree, env)
    return target_dir / "release" / "libbit_string.rlib"


def build_harness(root: Path, library: Path, output: Path, rustflags: str) -> None:
    dependency_dir = library.parent / "deps"
    command = [
        "rustc",
        str(root / "scripts" / "codegen_harness.rs"),
        "--crate-name",
        "bit_string_codegen_harness",
        "--crate-type=lib",
        "--edition=2024",
        "-O",
        "--emit=obj",
        "-o",
        str(output),
        "--extern",
        f"bit_string={library}",
        "-L",
        f"dependency={dependency_dir}",
        *shlex.split(rustflags),
    ]
    run(command, root, os.environ.copy())


def compare(root: Path, baseline: Path, current: Path, artifact: str, target: str, arch: str) -> int:
    command = [
        sys.executable,
        str(root / "scripts" / "compare_codegen.py"),
        str(baseline),
        str(current),
        "--roots",
        str(root / "scripts" / "codegen_roots.toml"),
        "--artifact",
        artifact,
        "--target",
        target,
        "--arch",
        arch,
    ]
    return subprocess.run(command, cwd=root, check=False).returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", default=BASELINE_SHA)
    parser.add_argument("--config", choices=sorted(CODEGEN_CONFIGS), action="append")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    output = root / "target" / "codegen-comparison"
    baseline_worktree = output / "baseline-worktree"
    output.mkdir(parents=True, exist_ok=True)
    if not baseline_worktree.exists():
        run(["git", "worktree", "add", "--detach", str(baseline_worktree), args.baseline], root, os.environ.copy())
    else:
        run(["git", "-C", str(baseline_worktree), "checkout", "--detach", args.baseline], root, os.environ.copy())
    host_arch = "aarch64" if platform.machine() in {"aarch64", "arm64"} else "x86_64"
    configs = args.config or [name for name, (arch, _) in CODEGEN_CONFIGS.items() if arch == host_arch]
    failed = False
    for config in configs:
        arch, rustflags = CODEGEN_CONFIGS[config]
        if arch != host_arch:
            print(f"ERROR: {config} requires {arch}, but host architecture is {host_arch}", file=sys.stderr)
            return 2
        config_dir = output / config
        baseline_library = build_library(baseline_worktree, config_dir / "baseline-target", rustflags)
        current_library = build_library(root, config_dir / "current-target", rustflags)
        baseline_harness = config_dir / "baseline-harness.o"
        current_harness = config_dir / "current-harness.o"
        build_harness(root, baseline_library, baseline_harness, rustflags)
        build_harness(root, current_library, current_harness, rustflags)
        failed |= compare(root, baseline_harness, current_harness, "harness", config, arch) != 0
        failed |= compare(root, baseline_library, current_library, "library", config, arch) != 0
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())

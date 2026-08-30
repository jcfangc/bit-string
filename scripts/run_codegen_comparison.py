#!/usr/bin/env python3
"""Build fixed-feature baseline/current artifacts and compare their codegen."""

from __future__ import annotations

import argparse
import atexit
import os
import platform
import shlex
import subprocess
import sys
import tempfile
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


def materialize_harness(root: Path, output: Path, revision: str) -> Path:
    source = output.with_suffix(".rs")
    source.write_bytes(subprocess.check_output(["git", "show", f"{revision}:scripts/codegen_harness.rs"], cwd=root))
    return source


def build_harness(root: Path, library: Path, output: Path, rustflags: str, revision: str) -> None:
    dependency_dir = library.parent / "deps"
    source = materialize_harness(root, output, revision)
    command = [
        "rustc",
        str(source),
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


def compare(
    root: Path,
    baseline: Path,
    current: Path,
    artifact: str,
    target: str,
    arch: str,
    strict_metadata: bool,
) -> int:
    command = [
        sys.executable,
        str(root / "scripts" / "compare_codegen.py"),
        str(baseline),
        str(current),
        "--roots",
        str(root / "scripts" / "codegen_roots.toml"),
        "--artifact",
        artifact,
        "--inventory",
        str(root / "scripts" / "kernel_inventory.toml"),
        "--target",
        target,
        "--arch",
        arch,
    ]
    if strict_metadata:
        command.append("--strict-metadata")
    return subprocess.run(command, cwd=root, check=False).returncode


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline", default=BASELINE_SHA)
    parser.add_argument("--config", choices=sorted(CODEGEN_CONFIGS), action="append")
    parser.add_argument("--strict-metadata", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    current_revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    output = root / "target" / "codegen-comparison"
    output.mkdir(parents=True, exist_ok=True)
    baseline_worktree = Path(tempfile.mkdtemp(prefix="bit-string-codegen-baseline-"))
    baseline_worktree.rmdir()
    run(["git", "worktree", "add", "--detach", str(baseline_worktree), args.baseline], root, os.environ.copy())

    def cleanup_worktree() -> None:
        subprocess.run(
            ["git", "worktree", "remove", "--force", str(baseline_worktree)],
            cwd=root,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

    atexit.register(cleanup_worktree)
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
        build_harness(root, baseline_library, baseline_harness, rustflags, current_revision)
        build_harness(root, current_library, current_harness, rustflags, current_revision)
        failed |= compare(
            root, baseline_harness, current_harness, "harness", config, arch, args.strict_metadata
        ) != 0
        failed |= compare(
            root, baseline_library, current_library, "library", config, arch, args.strict_metadata
        ) != 0
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Run interleaved Divan benchmarks for a fixed baseline and this worktree."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

from bench_config import BASELINE_SHA, BENCH_RUSTFLAGS

ORDER = ("baseline", "current", "current", "baseline", "baseline", "current", "current", "baseline", "baseline", "current")


def run(command: list[str], *, cwd: Path, env: dict[str, str] | None = None, output: Path | None = None) -> None:
    completed = subprocess.run(command, cwd=cwd, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
    if output is not None:
        output.write_text(completed.stdout, encoding="utf-8")
    if completed.returncode:
        sys.stdout.write(completed.stdout)
        raise SystemExit(f"command failed ({completed.returncode}): {' '.join(command)}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runs", type=int, default=5, help="runs per revision; default is 5")
    parser.add_argument("--warning-threshold", type=float, default=5.0)
    parser.add_argument("--baseline", default=BASELINE_SHA)
    args = parser.parse_args()
    if args.runs != 5:
        raise SystemExit("this runner currently fixes --runs at 5 to preserve the documented interleaving")

    root = Path(__file__).resolve().parents[1]
    output_dir = root / "target" / "bench-comparison"
    baseline_dir = output_dir / "baseline-worktree"
    output_dir.mkdir(parents=True, exist_ok=True)
    if not baseline_dir.exists():
        run(["git", "worktree", "add", "--detach", str(baseline_dir), args.baseline], cwd=root)
    else:
        run(["git", "-C", str(baseline_dir), "checkout", "--detach", args.baseline], cwd=root)

    env = os.environ.copy()
    env["RUSTFLAGS"] = BENCH_RUSTFLAGS
    command = ["cargo", "bench", "--", "--color", "never"]
    counts = {"baseline": 0, "current": 0}
    for revision in ORDER:
        counts[revision] += 1
        report = output_dir / f"{revision}-{counts[revision]}.txt"
        print(f"[{revision} {counts[revision]}/5] {' '.join(command)}")
        run(command, cwd=baseline_dir if revision == "baseline" else root, env=env, output=report)

    compare = [sys.executable, str(root / "scripts" / "compare_bench.py")]
    compare += [str(output_dir / f"baseline-{i}.txt") for i in range(1, 6)]
    compare += ["--current"] + [str(output_dir / f"current-{i}.txt") for i in range(1, 6)]
    compare += ["--expected-runs", str(args.runs)]
    compare += ["--warning-threshold", str(args.warning_threshold)]
    return subprocess.run(compare, cwd=root, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())

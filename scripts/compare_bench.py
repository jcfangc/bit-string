#!/usr/bin/env python3
"""Compare repeated Divan reports using the median of per-run medians."""

from __future__ import annotations

import argparse
import statistics
import sys
from pathlib import Path

from divan_parser import parse_report


def aggregate(paths: list[Path], expected_runs: int) -> dict[str, float]:
    if len(paths) != expected_runs:
        raise ValueError(f"expected {expected_runs} reports, got {len(paths)}")
    reports = [parse_report(path) for path in paths]
    expected_names = set(reports[0])
    for path, report in zip(paths[1:], reports[1:], strict=True):
        if set(report) != expected_names:
            missing = sorted(expected_names - set(report))
            extra = sorted(set(report) - expected_names)
            raise ValueError(f"{path}: inconsistent benchmark set; missing={missing}, extra={extra}")
    return {name: statistics.median(report[name].median_ns for report in reports) for name in expected_names}


def fmt_ns(value: float) -> str:
    if value >= 1_000_000_000:
        return f"{value / 1_000_000_000:.3f} s"
    if value >= 1_000_000:
        return f"{value / 1_000_000:.3f} ms"
    if value >= 1_000:
        return f"{value / 1_000:.3f} µs"
    if value < 1:
        return f"{value * 1_000:.3f} ps"
    return f"{value:.3f} ns"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path, nargs="+", help="baseline Divan text reports")
    parser.add_argument("--current", type=Path, nargs="+", required=True, help="current Divan text reports")
    parser.add_argument("--expected-runs", type=int, default=5)
    parser.add_argument("--warning-threshold", type=float, default=5.0)
    args = parser.parse_args()
    try:
        baseline = aggregate(args.baseline, args.expected_runs)
        current = aggregate(args.current, args.expected_runs)
    except ValueError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 2
    only_baseline = sorted(baseline.keys() - current.keys())
    only_current = sorted(current.keys() - baseline.keys())
    if only_baseline or only_current:
        for name in only_baseline:
            print(f"ERROR: missing in current: {name}", file=sys.stderr)
        for name in only_current:
            print(f"ERROR: missing in baseline: {name}", file=sys.stderr)
        return 2
    improved = unchanged = regressed = warnings = 0
    print(f"{'benchmark':<50} {'baseline':>12} {'current':>12} {'delta':>10}  status")
    print("-" * 96)
    for name in sorted(baseline):
        before, after = baseline[name], current[name]
        delta = (after - before) / before * 100
        improved += delta < 0
        unchanged += delta == 0
        regressed += delta > 0
        status = "WARNING" if delta > args.warning_threshold else "OK"
        if status == "WARNING":
            warnings += 1
            print(f"WARNING: {name} regressed by {delta:+.2f}%", file=sys.stderr)
        print(f"{name:<50.50} {fmt_ns(before):>12} {fmt_ns(after):>12} {delta:>+9.2f}%  {status}")
    print("\nSUMMARY")
    print(f"matched benchmarks : {len(baseline)}")
    print(f"improved           : {improved}")
    print(f"unchanged          : {unchanged}")
    print(f"regressed          : {regressed}")
    print(f"warnings           : {warnings}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

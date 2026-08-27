#!/usr/bin/env python3
"""Compare Divan text reports using each benchmark's median time."""

from __future__ import annotations

import argparse
import re
import statistics
import sys
from pathlib import Path

TIME = re.compile(r"(?P<value>[0-9]+(?:\.[0-9]+)?)\s*(?P<unit>ns|µs|us|ms|s)")
NAME = re.compile(r"^[├╰]─\s*(?P<name>.+?)(?:\s{2,}|\s+Timer precision:)")
UNITS_TO_NS = {"ns": 1.0, "µs": 1_000.0, "us": 1_000.0, "ms": 1_000_000.0, "s": 1_000_000_000.0}


def parse_report(path: Path) -> dict[str, float]:
    """Parse Divan's columnar text report, returning median values in ns."""
    result: dict[str, float] = {}
    pending: str | None = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        match = NAME.match(raw)
        if match:
            pending = match.group("name").strip()
        if pending is None:
            continue
        values = list(TIME.finditer(raw))
        # Data rows contain fastest, slowest, median, and mean in that order.
        if len(values) >= 4:
            median = values[2]
            result[pending] = float(median.group("value")) * UNITS_TO_NS[median.group("unit")]
            pending = None
        elif raw.strip() and not raw.startswith((" ", "\t")):
            pending = None
    if not result:
        raise ValueError(f"no Divan benchmark medians found in {path}")
    return result


def fmt_ns(value: float) -> str:
    if value >= 1_000_000_000:
        return f"{value / 1_000_000_000:.3f} s"
    if value >= 1_000_000:
        return f"{value / 1_000_000:.3f} ms"
    if value >= 1_000:
        return f"{value / 1_000:.3f} µs"
    return f"{value:.3f} ns"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline", type=Path, nargs="+", help="baseline Divan text reports")
    parser.add_argument("--current", type=Path, nargs="+", required=True, help="current Divan text reports")
    parser.add_argument("--warning-threshold", type=float, default=5.0)
    args = parser.parse_args()

    def aggregate(paths: list[Path]) -> dict[str, float]:
        samples: dict[str, list[float]] = {}
        for path in paths:
            for name, value in parse_report(path).items():
                samples.setdefault(name, []).append(value)
        return {name: statistics.median(values) for name, values in samples.items()}

    baseline, current = aggregate(args.baseline), aggregate(args.current)
    common = sorted(baseline.keys() & current.keys())
    only_baseline = sorted(baseline.keys() - current.keys())
    only_current = sorted(current.keys() - baseline.keys())
    improved = unchanged = regressed = warnings = 0
    print(f"{'benchmark':<50} {'baseline':>12} {'current':>12} {'delta':>10}  status")
    print("-" * 96)
    for name in common:
        before, after = baseline[name], current[name]
        delta = (after - before) / before * 100
        if delta < 0:
            improved += 1
        elif delta > 0:
            regressed += 1
        else:
            unchanged += 1
        status = "WARNING" if delta > args.warning_threshold else "OK"
        if status == "WARNING":
            warnings += 1
            print(f"WARNING: {name} regressed by {delta:+.2f}%", file=sys.stderr)
        print(f"{name:<50.50} {fmt_ns(before):>12} {fmt_ns(after):>12} {delta:>+9.2f}%  {status}")
    for name in only_baseline:
        print(f"missing in current: {name}", file=sys.stderr)
    for name in only_current:
        print(f"missing in baseline: {name}", file=sys.stderr)
    print("\nSUMMARY")
    print(f"matched benchmarks : {len(common)}")
    print(f"improved           : {improved}")
    print(f"unchanged          : {unchanged}")
    print(f"regressed          : {regressed}")
    print(f"warnings           : {warnings}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

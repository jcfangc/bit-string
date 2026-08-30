"""Strict parser for Divan's text table format."""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

TIME = re.compile(r"(?P<value>[0-9]+(?:\.[0-9]+)?)\s*(?P<unit>ps|ns|µs|us|ms|s)\b")
ROW = re.compile(r"^[│\s]*[├╰]─\s*(?P<body>.+)$")
COUNTS = re.compile(r"│\s*(?P<samples>[0-9]+)\s*│\s*(?P<iters>[0-9]+)\s*$")
UNITS_TO_NS = {"ps": 1e-3, "ns": 1.0, "µs": 1_000.0, "us": 1_000.0, "ms": 1_000_000.0, "s": 1_000_000_000.0}


@dataclass(frozen=True)
class DivanResult:
    fastest_ns: float
    slowest_ns: float
    median_ns: float
    mean_ns: float
    samples: int
    iters: int


def _time_ns(match: re.Match[str]) -> float:
    return float(match.group("value")) * UNITS_TO_NS[match.group("unit")]


def parse_report(path: Path) -> dict[str, DivanResult]:
    result: dict[str, DivanResult] = {}
    pending: str | None = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        row = ROW.match(raw)
        if row:
            body = row.group("body")
            first_time = TIME.search(body)
            end = first_time.start() if first_time else body.find("Timer precision:")
            pending = body[: end if end >= 0 else None].rstrip()
        if pending is None:
            continue
        times = list(TIME.finditer(raw))
        if len(times) < 4:
            continue
        counts = COUNTS.search(raw)
        if counts is None:
            raise ValueError(f"{path}: benchmark row for {pending!r} has no samples/iters")
        if pending in result:
            raise ValueError(f"{path}: duplicate benchmark name {pending!r}")
        values = [_time_ns(match) for match in times[:4]]
        result[pending] = DivanResult(*values, int(counts.group("samples")), int(counts.group("iters")))
        pending = None
    if not result:
        raise ValueError(f"{path}: no Divan benchmark rows found")
    return result

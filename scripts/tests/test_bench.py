from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import compare_bench
from divan_parser import parse_report

ROW = "├─ bench  500 ps │ 1 ns │ 750 ps │ 800 ps │ 100 │ 1000\n"


class DivanParserTests(unittest.TestCase):
    def write(self, directory: str, name: str, content: str = ROW) -> Path:
        path = Path(directory) / name
        path.write_text(content)
        return path

    def test_picoseconds_are_converted_to_ns(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            result = parse_report(self.write(directory, "report.txt"))["bench"]
            self.assertEqual(result.median_ns, 0.75)
            self.assertEqual(result.samples, 100)
            self.assertEqual(result.iters, 1000)

    def test_duplicate_benchmark_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory, self.assertRaisesRegex(ValueError, "duplicate"):
            parse_report(self.write(directory, "report.txt", ROW + ROW))

    def test_expected_run_count_is_enforced(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = [self.write(directory, f"report-{index}.txt") for index in range(4)]
            with self.assertRaisesRegex(ValueError, "expected 5 reports"):
                compare_bench.aggregate(paths, 5)

    def test_missing_benchmark_in_one_run_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = [self.write(directory, f"report-{index}.txt") for index in range(5)]
            paths[-1].write_text(ROW.replace("bench", "different"))
            with self.assertRaisesRegex(ValueError, "inconsistent benchmark set"):
                compare_bench.aggregate(paths, 5)

    def test_missing_between_revisions_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            baseline = [self.write(directory, f"baseline-{index}.txt") for index in range(5)]
            current = [self.write(directory, f"current-{index}.txt", ROW.replace("bench", "other")) for index in range(5)]
            argv = [
                "compare_bench.py",
                *(str(path) for path in baseline),
                "--current",
                *(str(path) for path in current),
            ]
            with (
                mock.patch.object(sys, "argv", argv),
                contextlib.redirect_stdout(io.StringIO()),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertNotEqual(compare_bench.main(), 0)


if __name__ == "__main__":
    unittest.main()

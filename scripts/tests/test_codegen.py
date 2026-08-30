from __future__ import annotations

import contextlib
import io
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import compare_codegen


def parse(body: str, arch: str = "x86_64") -> compare_codegen.Function:
    return compare_codegen.parse_objdump(body, arch)["root"]


class CodegenParserTests(unittest.TestCase):
    def test_call_target_identity_is_preserved(self) -> None:
        foo = parse("0000 <root>:\n 0: e8 00 00 00 00 call 5 <foo>\n 1: R_X86_64_PLT32 foo-0x4")
        bar = parse("0000 <root>:\n 0: e8 00 00 00 00 call 5 <bar>\n 1: R_X86_64_PLT32 bar-0x4")
        self.assertNotEqual(foo.instructions, bar.instructions)

    def test_local_branch_uses_target_instruction_index(self) -> None:
        old = parse("0000 <root>:\n 0: 74 02 je 4 <root+0x4>\n 2: 90 nop\n 4: c3 ret")
        relocated = parse("0000 <root>:\n 0: 74 08 je a <root+0xa>\n 2: 66 90 xchg %ax,%ax\n a: c3 ret")
        different = parse("0000 <root>:\n 0: 74 00 je 2 <root+0x2>\n 2: 90 nop\n 4: c3 ret")
        self.assertEqual(old.instructions[0], relocated.instructions[0])
        self.assertNotEqual(old.instructions[0], different.instructions[0])

    def test_relocation_is_not_an_instruction(self) -> None:
        function = parse("0000 <root>:\n 0: e8 00 00 00 00 call 5 <foo>\n 1: R_X86_64_PLT32 foo-0x4\n 5: c3 ret")
        self.assertEqual(len(function.instructions), 2)
        self.assertEqual(function.instructions[0], "call <foo> [R_X86_64_PLT32:foo]")

    def test_semantic_relocation_addend_is_preserved(self) -> None:
        first = parse("0000 <root>:\n 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n 3: R_X86_64_PC32 .rodata+0x20")
        second = parse("0000 <root>:\n 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n 3: R_X86_64_PC32 .rodata+0x40")
        self.assertNotEqual(first.instructions, second.instructions)

    def test_duplicate_disassembly_symbol_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "duplicate disassembly symbol"):
            compare_codegen.parse_objdump("0000 <root>:\n 0: c3 ret\n0000 <root>:\n 0: c3 ret", "x86_64")

    def test_encoding_size_change_is_not_identical(self) -> None:
        old = parse("0000 <root>:\n 0: 90 nop\n 1: c3 ret")
        new = parse("0000 <root>:\n 0: 66 90 nop\n 2: c3 ret")
        self.assertEqual(old.instructions, new.instructions)
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN ENCODING CHANGED")

    def test_aarch64_calls_and_branches(self) -> None:
        function = parse(
            "0000000000000000 <root>:\n"
            "   0:   94000000    bl 0 <callee>\n"
            "            0: R_AARCH64_CALL26 callee\n"
            "   4:   54000040    b.eq c <root+0xc>\n"
            "   8:   91000400    add x0, x0, #0x1\n"
            "   c:   d65f03c0    ret",
            "aarch64",
        )
        self.assertEqual(function.calls, 1)
        self.assertEqual(function.branches, 1)
        self.assertIn("@instruction_3", function.instructions[1])
        self.assertIn("#0x1", function.instructions[2])
        self.assertIn("[R_AARCH64_CALL26:callee]", function.instructions[0])

    def test_expected_missing_symbol_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            roots = Path(directory) / "roots.toml"
            roots.write_text('[[root]]\nsymbol="expected"\nartifact="harness"\ntargets=["x86_64-avx2"]\n')
            argv = [
                "compare_codegen.py",
                "baseline.o",
                "current.o",
                "--roots",
                str(roots),
                "--artifact",
                "harness",
                "--target",
                "x86_64-avx2",
                "--arch",
                "x86_64",
            ]
            with (
                mock.patch.object(sys, "argv", argv),
                mock.patch.object(compare_codegen, "load_functions", return_value={}),
                contextlib.redirect_stdout(io.StringIO()),
                contextlib.redirect_stderr(io.StringIO()),
            ):
                self.assertNotEqual(compare_codegen.main(), 0)

    def test_inventory_has_root_for_each_kernel(self) -> None:
        scripts = Path(__file__).resolve().parents[1]
        roots = {root.symbol for root in compare_codegen.load_roots(scripts / "codegen_roots.toml")}
        inventory = tomllib.loads((scripts / "kernel_inventory.toml").read_text())["kernel"]
        self.assertEqual(len(inventory), 17)
        for kernel in inventory:
            self.assertEqual(kernel["coverage"], "caller")
            self.assertTrue(kernel["roots"])
            self.assertLessEqual(set(kernel["roots"]), roots)

        binary = next(
            kernel
            for kernel in inventory
            if kernel["path"] == "src/traits/words_arith/funcs_for_binary_core.rs"
        )
        self.assertEqual(
            set(binary["roots"]),
            {"codegen_bit_string_and", "codegen_bit_string_or", "codegen_bit_string_xor"},
        )

    def test_independent_backend_symbols_are_discovered_from_inventory_modules(self) -> None:
        function = parse("0000 <root>:\n 0: c3 ret")
        functions = {
            "bit_string::traits::words_scan::funcs_for_count_ones::avx2::count_words": function,
            "bit_string::traits::words_scan::unrelated::avx2::count_words": function,
            "bit_string::traits::words_scan::funcs_for_count_ones::entry": function,
        }
        discovered = compare_codegen.independent_backend_symbols(
            functions,
            ("bit_string::traits::words_scan::funcs_for_count_ones",),
        )
        self.assertEqual(
            discovered,
            {"bit_string::traits::words_scan::funcs_for_count_ones::avx2::count_words"},
        )


if __name__ == "__main__":
    unittest.main()

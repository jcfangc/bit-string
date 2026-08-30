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


def parse_with_sections(body: str, sections: str, arch: str = "x86_64") -> compare_codegen.Function:
    index = compare_codegen.ArtifactSections.from_objdump(sections, arch)
    return compare_codegen.parse_objdump(body, arch, index)["root"]


class CodegenParserTests(unittest.TestCase):
    SECTION_PREFIX = (
        "fixture.o: file format elf64-x86-64\n"
        "Contents of section .rodata.str1.1:\n"
        " 0000 66696c65 2e727300                    file.rs.\n"
    )

    @classmethod
    def anonymous_section(cls, ordinal: int, line: int = 1, column: int = 2) -> str:
        line_bytes = line.to_bytes(4, "little").hex()
        column_bytes = column.to_bytes(4, "little").hex()
        return (
            f"Contents of section .data.rel.ro..Lanon.hash.{ordinal}:\n"
            f" 0000 00000000 00000000 {line_bytes} {column_bytes}  ........\n"
            "RELOCATION RECORDS FOR [.data.rel.ro..Lanon.hash."
            f"{ordinal}]:\n"
            "OFFSET           TYPE              VALUE\n"
            "0000000000000000 R_X86_64_64     .rodata.str1.1+0x0000000000000000\n"
        )

    def test_anonymous_section_content_ignores_ordinal(self) -> None:
        old_sections = self.SECTION_PREFIX + self.anonymous_section(1)
        new_sections = self.SECTION_PREFIX + self.anonymous_section(9)
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4\n",
            old_sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.9-0x4\n",
            new_sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN IDENTICAL")

    @staticmethod
    def nested_sections(
        outer_ordinal: int,
        nested_ordinal: int,
        nested_value: int = 1,
        nested_addend: str = "+0x0",
    ) -> str:
        nested_name = f".data.rel.ro..Lanon.nested.{nested_ordinal}"
        outer_name = f".data.rel.ro..Lanon.outer.{outer_ordinal}"
        result = (
            "fixture.o: file format elf64-x86-64\n"
            f"Contents of section {nested_name}:\n"
            f" 0000 {nested_value & 0xffffffff:08x} 00000000  ........\n"
            f"Contents of section {outer_name}:\n"
            " 0000 00000000 00000000  ........\n"
            f"RELOCATION RECORDS FOR [{outer_name}]:\n"
            "OFFSET           TYPE              VALUE\n"
            f"0000000000000000 R_X86_64_64     {nested_name}{nested_addend}\n"
        )
        return result

    def nested_root(self, outer_ordinal: int, addend: str = "+0x0") -> str:
        return (
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            f" 3: R_X86_64_PC32 .data.rel.ro..Lanon.outer.{outer_ordinal}{addend}\n"
        )

    def test_one_level_nested_ordinal_change_is_identical(self) -> None:
        old = parse_with_sections(self.nested_root(1), self.nested_sections(1, 2))
        new = parse_with_sections(self.nested_root(9), self.nested_sections(9, 8))
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN IDENTICAL")

    def test_nested_semantic_content_change_is_metadata_change(self) -> None:
        old = parse_with_sections(self.nested_root(1), self.nested_sections(1, 2, nested_value=1))
        new = parse_with_sections(self.nested_root(9), self.nested_sections(9, 8, nested_value=2))
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN METADATA CHANGED")

    def test_nested_addend_change_is_not_metadata(self) -> None:
        old = parse_with_sections(self.nested_root(1), self.nested_sections(1, 2, nested_addend="+0x0"))
        new = parse_with_sections(self.nested_root(9), self.nested_sections(9, 8, nested_addend="+0x8"))
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_nested_alias_partition_is_preserved(self) -> None:
        def sections(outer_ordinal: int, first: int, second: int) -> str:
            outer = f".data.rel.ro..Lanon.outer.{outer_ordinal}"
            first_name = f".data.rel.ro..Lanon.nested.{first}"
            second_name = f".data.rel.ro..Lanon.nested.{second}"
            return (
                "fixture.o: file format elf64-x86-64\n"
                f"Contents of section {first_name}:\n"
                " 0000 01000000 00000000  ........\n"
                + (
                    f"Contents of section {second_name}:\n"
                    " 0000 01000000 00000000  ........\n"
                    if second != first
                    else ""
                )
                + f"Contents of section {outer}:\n"
                " 0000 00000000 00000000 00000000 00000000  ................\n"
                f"RELOCATION RECORDS FOR [{outer}]:\n"
                "OFFSET           TYPE              VALUE\n"
                f"0000000000000000 R_X86_64_64     {first_name}+0x0\n"
                f"0000000000000008 R_X86_64_64     {second_name}+0x0\n"
            )

        old = parse_with_sections(self.nested_root(1), sections(1, 2, 2))
        new = parse_with_sections(self.nested_root(9), sections(9, 8, 9))
        self.assertTrue(old.relocations[0].resolved)
        self.assertTrue(new.relocations[0].resolved)
        self.assertEqual(old.relocations[0].nested_alias_partition, (0, 0))
        self.assertEqual(new.relocations[0].nested_alias_partition, (0, 1))
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

        same_alias = parse_with_sections(self.nested_root(9), sections(9, 8, 8))
        self.assertTrue(same_alias.relocations[0].resolved)
        self.assertEqual(same_alias.relocations[0].nested_alias_partition, (0, 0))
        self.assertEqual(compare_codegen.classify(old, same_alias), "CODEGEN IDENTICAL")

    def test_second_level_nested_anonymous_is_conservative(self) -> None:
        inner = ".data.rel.ro..Lanon.inner.3"
        nested = ".data.rel.ro..Lanon.nested.2"
        outer = ".data.rel.ro..Lanon.outer.1"
        sections = (
            "fixture.o: file format elf64-x86-64\n"
            f"Contents of section {inner}:\n"
            " 0000 00000000 00000000  ........\n"
            f"Contents of section {nested}:\n"
            " 0000 00000000 00000000  ........\n"
            f"RELOCATION RECORDS FOR [{nested}]:\n"
            "OFFSET           TYPE              VALUE\n"
            f"0000000000000000 R_X86_64_64     {inner}+0x0\n"
            f"Contents of section {outer}:\n"
            " 0000 00000000 00000000  ........\n"
            f"RELOCATION RECORDS FOR [{outer}]:\n"
            "OFFSET           TYPE              VALUE\n"
            f"0000000000000000 R_X86_64_64     {nested}+0x0\n"
        )
        old = parse_with_sections(self.nested_root(1), sections)
        self.assertFalse(old.relocations[0].resolved)
        self.assertEqual(compare_codegen.classify(old, old), "CODEGEN CHANGED")

    def test_nested_target_does_not_cross_archive_members(self) -> None:
        outer = ".data.rel.ro..Lanon.outer.1"
        nested = ".data.rel.ro..Lanon.nested.2"
        sections = (
            "member-a.o: file format elf64-x86-64\n"
            f"Contents of section {outer}:\n"
            " 0000 00000000 00000000  ........\n"
            f"RELOCATION RECORDS FOR [{outer}]:\n"
            "OFFSET           TYPE              VALUE\n"
            f"0000000000000000 R_X86_64_64     {nested}+0x0\n"
            "member-b.o: file format elf64-x86-64\n"
            f"Contents of section {nested}:\n"
            " 0000 01000000 00000000  ........\n"
        )
        index = compare_codegen.ArtifactSections.from_objdump(sections, "x86_64")
        resolved = index.resolve("member-a.o", "R_X86_64_PC32", outer)
        self.assertFalse(resolved.resolved)

    def test_relocation_site_change_is_not_ignored(self) -> None:
        old = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 foo-0x4\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax"
        )
        new = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " a: R_X86_64_PC32 foo-0x4"
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_relocation_offset_change_is_not_ignored(self) -> None:
        old = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 foo-0x4"
        )
        new = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 4: R_X86_64_PC32 foo-0x4"
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_anonymous_section_content_change_is_metadata_change(self) -> None:
        old_sections = self.SECTION_PREFIX + self.anonymous_section(1, line=1)
        new_sections = self.SECTION_PREFIX + self.anonymous_section(9, line=3)
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4\n",
            old_sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.9-0x4\n",
            new_sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN METADATA CHANGED")

    def test_anonymous_section_alias_partition_is_preserved(self) -> None:
        sections = self.SECTION_PREFIX + self.anonymous_section(1) + self.anonymous_section(2)
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " a: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4\n",
            sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " a: R_X86_64_PC32 .data.rel.ro..Lanon.hash.2-0x4\n",
            sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_unresolved_anonymous_section_is_conservative(self) -> None:
        sections = "fixture.o: file format elf64-x86-64\n"
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4\n",
            sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.9-0x4\n",
            sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_named_relocation_change_is_not_metadata(self) -> None:
        old = parse(
            "0000 <root>:\n"
            " 0: e8 00 00 00 00 call 5 <foo>\n"
            " 1: R_X86_64_PLT32 foo-0x4"
        )
        new = parse(
            "0000 <root>:\n"
            " 0: e8 00 00 00 00 call 5 <bar>\n"
            " 1: R_X86_64_PLT32 bar-0x4"
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_relocation_addend_change_is_not_metadata(self) -> None:
        old = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .rodata+0x20"
        )
        new = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .rodata+0x40"
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_aarch64_anonymous_section_identity_ignores_ordinal(self) -> None:
        def sections(ordinal: int) -> str:
            return (
                "fixture.o: file format elf64-littleaarch64\n"
                "Contents of section .rodata.str1.1:\n"
                " 0000 66696c65 2e727300                    file.rs.\n"
                f"Contents of section .data.rel.ro..Lanon.hash.{ordinal}:\n"
                " 0000 00000000 00000000 01000000 02000000  ........\n"
                f"RELOCATION RECORDS FOR [.data.rel.ro..Lanon.hash.{ordinal}]:\n"
                "OFFSET           TYPE              VALUE\n"
                "0000000000000000 R_AARCH64_ABS64 .rodata.str1.1+0x0000000000000000\n"
            )

        old = parse_with_sections(
            "fixture.o: file format elf64-littleaarch64\n"
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.1\n",
            sections(1),
            "aarch64",
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-littleaarch64\n"
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.9\n",
            sections(9),
            "aarch64",
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN IDENTICAL")

    def test_anonymous_addend_change_is_not_metadata(self) -> None:
        sections = self.SECTION_PREFIX + self.anonymous_section(1)
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1+0x0",
            sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1+0x8",
            sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_alias_identity_excludes_addend(self) -> None:
        sections = (
            self.SECTION_PREFIX
            + self.anonymous_section(1)
            + self.anonymous_section(2)
            + self.anonymous_section(3)
        )
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1+0x0\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " a: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1+0x8",
            sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.2+0x0\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " a: R_X86_64_PC32 .data.rel.ro..Lanon.hash.3+0x8",
            sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN CHANGED")

    def test_section_class_is_part_of_anonymous_identity(self) -> None:
        old_sections = self.SECTION_PREFIX + self.anonymous_section(1)
        new_sections = self.SECTION_PREFIX + self.anonymous_section(9).replace(
            ".data.rel.ro..Lanon", ".rodata..Lanon"
        )
        old = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .data.rel.ro..Lanon.hash.1-0x4",
            old_sections,
        )
        new = parse_with_sections(
            "fixture.o: file format elf64-x86-64\n"
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax\n"
            " 3: R_X86_64_PC32 .rodata..Lanon.hash.9-0x4",
            new_sections,
        )
        self.assertEqual(compare_codegen.classify(old, new), "CODEGEN METADATA CHANGED")

    def test_section_lookup_does_not_cross_archive_members(self) -> None:
        sections = (
            "member-a.o: file format elf64-x86-64\n"
            "member-b.o: file format elf64-x86-64\n"
            "Contents of section .data.rel.ro..Lanon.hash.1:\n"
            " 0000 00000000 00000000 01000000 02000000  ........\n"
        )
        index = compare_codegen.ArtifactSections.from_objdump(sections, "x86_64")
        resolved = index.resolve("member-a.o", "R_X86_64_PC32", ".data.rel.ro..Lanon.hash.1-0x4")
        self.assertFalse(resolved.resolved)

    def test_section_bytes_ignore_ascii_column_after_four_groups(self) -> None:
        output = (
            "fixture.o: file format elf64-x86-64\n"
            "Contents of section .rodata:\n"
            " 0000 00000000 00000000 00000000 00000000  deadbeef\n"
        )
        index = compare_codegen.ArtifactSections.from_objdump(output, "x86_64")
        section = index.sections[("fixture.o", ".rodata")]
        self.assertEqual(section.data, b"\0" * 16)
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

    def test_anonymous_relocation_instance_is_ignored(self) -> None:
        first = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.abcdef0123456789.67",
            "aarch64",
        )
        second = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.abcdef0123456789.68",
            "aarch64",
        )
        self.assertEqual(first.instructions, second.instructions)

    def test_anonymous_relocation_instance_with_addend_is_ignored(self) -> None:
        first = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash-with-extra.67+0x20",
            "aarch64",
        )
        second = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash-with-extra.68+0x20",
            "aarch64",
        )
        self.assertEqual(first.instructions, second.instructions)
        self.assertIn("+0x20", first.instructions[0])

        different_addend = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root>\n"
            "            0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash-with-extra.68+0x40",
            "aarch64",
        )
        self.assertNotEqual(first.instructions, different_addend.instructions)

    def test_inline_anonymous_relocations_allow_uniform_ordinal_shift(self) -> None:
        first = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root> 0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.67\n"
            "   4:   90000000    adrp x1, 0 <root> 4: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.69\n"
            "   8:   90000000    adrp x2, 0 <root> 8: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.68\n",
            "aarch64",
        )
        shifted = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root> 0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.68\n"
            "   4:   90000000    adrp x1, 0 <root> 4: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.70\n"
            "   8:   90000000    adrp x2, 0 <root> 8: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.69\n",
            "aarch64",
        )
        self.assertEqual(first.instructions, shifted.instructions)

    def test_inline_anonymous_relocations_preserve_alias_relationships(self) -> None:
        aliased = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root> 0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.67\n"
            "   4:   90000000    adrp x1, 0 <root> 4: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.67\n"
            "   8:   90000000    adrp x2, 0 <root> 8: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.69\n",
            "aarch64",
        )
        no_longer_aliased = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root> 0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.68\n"
            "   4:   90000000    adrp x1, 0 <root> 4: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.69\n"
            "   8:   90000000    adrp x2, 0 <root> 8: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.70\n",
            "aarch64",
        )
        self.assertNotEqual(aliased.instructions, no_longer_aliased.instructions)

    def test_inline_anonymous_relocations_preserve_relative_gaps(self) -> None:
        first = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root> 0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.67\n"
            "   4:   90000000    adrp x1, 0 <root> 4: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.69\n",
            "aarch64",
        )
        different_gap = parse(
            "0000000000000000 <root>:\n"
            "   0:   90000000    adrp x0, 0 <root> 0: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.68\n"
            "   4:   90000000    adrp x1, 0 <root> 4: R_AARCH64_ADR_PREL_PG_HI21 .data.rel.ro..Lanon.hash.71\n",
            "aarch64",
        )
        self.assertNotEqual(first.instructions, different_gap.instructions)

    def test_x86_anonymous_relocations_allow_llvm_suffix_changes(self) -> None:
        first = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax 0: R_X86_64_PC32 anon.hash.67.llvm.123\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax 7: R_X86_64_PC32 anon.hash.69.llvm.123\n"
        )
        shifted = parse(
            "0000 <root>:\n"
            " 0: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax 0: R_X86_64_PC32 anon.hash.68.llvm.456\n"
            " 7: 48 8d 05 00 00 00 00 lea 0x0(%rip),%rax 7: R_X86_64_PC32 anon.hash.70.llvm.456\n"
        )
        self.assertEqual(first.instructions, shifted.instructions)

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

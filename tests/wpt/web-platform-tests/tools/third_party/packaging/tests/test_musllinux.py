import collections
import io
import pathlib
import struct
import subprocess

import pretend
import pytest

from packaging import _musllinux
from packaging._musllinux import (
    _get_musl_version,
    _MuslVersion,
    _parse_ld_musl_from_elf,
    _parse_musl_version,
)

MUSL_AMD64 = "musl libc (x86_64)\nVersion 1.2.2\n"
MUSL_I386 = "musl libc (i386)\nVersion 1.2.1\n"
MUSL_AARCH64 = "musl libc (aarch64)\nVersion 1.1.24\n"
MUSL_INVALID = "musl libc (invalid)\n"
MUSL_UNKNOWN = "musl libc (unknown)\nVersion unknown\n"

MUSL_DIR = pathlib.Path(__file__).with_name("musllinux").resolve()

BIN_GLIBC_X86_64 = MUSL_DIR.joinpath("glibc-x86_64")
BIN_MUSL_X86_64 = MUSL_DIR.joinpath("musl-x86_64")
BIN_MUSL_I386 = MUSL_DIR.joinpath("musl-i386")
BIN_MUSL_AARCH64 = MUSL_DIR.joinpath("musl-aarch64")

LD_MUSL_X86_64 = "/lib/ld-musl-x86_64.so.1"
LD_MUSL_I386 = "/lib/ld-musl-i386.so.1"
LD_MUSL_AARCH64 = "/lib/ld-musl-aarch64.so.1"


@pytest.fixture(autouse=True)
def clear_lru_cache():
    yield
    _get_musl_version.cache_clear()


@pytest.mark.parametrize(
    "output, version",
    [
        (MUSL_AMD64, _MuslVersion(1, 2)),
        (MUSL_I386, _MuslVersion(1, 2)),
        (MUSL_AARCH64, _MuslVersion(1, 1)),
        (MUSL_INVALID, None),
        (MUSL_UNKNOWN, None),
    ],
    ids=["amd64-1.2.2", "i386-1.2.1", "aarch64-1.1.24", "invalid", "unknown"],
)
def test_parse_musl_version(output, version):
    assert _parse_musl_version(output) == version


@pytest.mark.parametrize(
    "executable, location",
    [
        (BIN_GLIBC_X86_64, None),
        (BIN_MUSL_X86_64, LD_MUSL_X86_64),
        (BIN_MUSL_I386, LD_MUSL_I386),
        (BIN_MUSL_AARCH64, LD_MUSL_AARCH64),
    ],
    ids=["glibc", "x86_64", "i386", "aarch64"],
)
def test_parse_ld_musl_from_elf(executable, location):
    with executable.open("rb") as f:
        assert _parse_ld_musl_from_elf(f) == location


@pytest.mark.parametrize(
    "data",
    [
        # Too short for magic.
        b"\0",
        # Enough for magic, but not ELF.
        b"#!/bin/bash" + b"\0" * 16,
        # ELF, but unknown byte declaration.
        b"\x7fELF\3" + b"\0" * 16,
    ],
    ids=["no-magic", "wrong-magic", "unknown-format"],
)
def test_parse_ld_musl_from_elf_invalid(data):
    assert _parse_ld_musl_from_elf(io.BytesIO(data)) is None


@pytest.mark.parametrize(
    "head",
    [
        25,  # Enough for magic, but not the section definitions.
        58,  # Enough for section definitions, but not the actual sections.
    ],
)
def test_parse_ld_musl_from_elf_invalid_section(head):
    data = BIN_MUSL_X86_64.read_bytes()[:head]
    assert _parse_ld_musl_from_elf(io.BytesIO(data)) is None


def test_parse_ld_musl_from_elf_no_interpreter_section():
    with BIN_MUSL_X86_64.open("rb") as f:
        data = f.read()

    # Change all sections to *not* PT_INTERP.
    unpacked = struct.unpack("16BHHIQQQIHHH", data[:58])
    *_, e_phoff, _, _, _, e_phentsize, e_phnum = unpacked
    for i in range(e_phnum + 1):
        sb = e_phoff + e_phentsize * i
        se = sb + 56
        section = struct.unpack("IIQQQQQQ", data[sb:se])
        data = data[:sb] + struct.pack("IIQQQQQQ", 0, *section[1:]) + data[se:]

    assert _parse_ld_musl_from_elf(io.BytesIO(data)) is None


@pytest.mark.parametrize(
    "executable, output, version, ld_musl",
    [
        (MUSL_DIR.joinpath("does-not-exist"), "error", None, None),
        (BIN_GLIBC_X86_64, "error", None, None),
        (BIN_MUSL_X86_64, MUSL_AMD64, _MuslVersion(1, 2), LD_MUSL_X86_64),
        (BIN_MUSL_I386, MUSL_I386, _MuslVersion(1, 2), LD_MUSL_I386),
        (BIN_MUSL_AARCH64, MUSL_AARCH64, _MuslVersion(1, 1), LD_MUSL_AARCH64),
    ],
    ids=["does-not-exist", "glibc", "x86_64", "i386", "aarch64"],
)
def test_get_musl_version(monkeypatch, executable, output, version, ld_musl):
    def mock_run(*args, **kwargs):
        return collections.namedtuple("Proc", "stderr")(output)

    run_recorder = pretend.call_recorder(mock_run)
    monkeypatch.setattr(_musllinux.subprocess, "run", run_recorder)

    assert _get_musl_version(str(executable)) == version

    if ld_musl is not None:
        expected_calls = [
            pretend.call(
                [ld_musl],
                stderr=subprocess.PIPE,
                universal_newlines=True,
            )
        ]
    else:
        expected_calls = []
    assert run_recorder.calls == expected_calls

try:
    import ctypes
except ImportError:
    ctypes = None
import os
import platform
import sys
import types
import warnings

import pretend
import pytest

from packaging import _manylinux
from packaging._manylinux import (
    _ELFFileHeader,
    _get_elf_header,
    _get_glibc_version,
    _glibc_version_string,
    _glibc_version_string_confstr,
    _glibc_version_string_ctypes,
    _is_compatible,
    _is_linux_armhf,
    _is_linux_i686,
    _parse_glibc_version,
)


@pytest.fixture(autouse=True)
def clear_lru_cache():
    yield
    _get_glibc_version.cache_clear()


@pytest.fixture
def manylinux_module(monkeypatch):
    monkeypatch.setattr(_manylinux, "_get_glibc_version", lambda *args: (2, 20))
    module_name = "_manylinux"
    module = types.ModuleType(module_name)
    monkeypatch.setitem(sys.modules, module_name, module)
    return module


@pytest.mark.parametrize("tf", (True, False))
@pytest.mark.parametrize(
    "attribute,glibc", (("1", (2, 5)), ("2010", (2, 12)), ("2014", (2, 17)))
)
def test_module_declaration(monkeypatch, manylinux_module, attribute, glibc, tf):
    manylinux = f"manylinux{attribute}_compatible"
    monkeypatch.setattr(manylinux_module, manylinux, tf, raising=False)
    res = _is_compatible(manylinux, "x86_64", glibc)
    assert tf is res


@pytest.mark.parametrize(
    "attribute,glibc", (("1", (2, 5)), ("2010", (2, 12)), ("2014", (2, 17)))
)
def test_module_declaration_missing_attribute(
    monkeypatch, manylinux_module, attribute, glibc
):
    manylinux = f"manylinux{attribute}_compatible"
    monkeypatch.delattr(manylinux_module, manylinux, raising=False)
    assert _is_compatible(manylinux, "x86_64", glibc)


@pytest.mark.parametrize(
    "version,compatible", (((2, 0), True), ((2, 5), True), ((2, 10), False))
)
def test_is_manylinux_compatible_glibc_support(version, compatible, monkeypatch):
    monkeypatch.setitem(sys.modules, "_manylinux", None)
    monkeypatch.setattr(_manylinux, "_get_glibc_version", lambda: (2, 5))
    assert bool(_is_compatible("manylinux1", "any", version)) == compatible


@pytest.mark.parametrize("version_str", ["glibc-2.4.5", "2"])
def test_check_glibc_version_warning(version_str):
    with warnings.catch_warnings(record=True) as w:
        _parse_glibc_version(version_str)
        assert len(w) == 1
        assert issubclass(w[0].category, RuntimeWarning)


@pytest.mark.skipif(not ctypes, reason="requires ctypes")
@pytest.mark.parametrize(
    "version_str,expected",
    [
        # Be very explicit about bytes and Unicode for Python 2 testing.
        (b"2.4", "2.4"),
        ("2.4", "2.4"),
    ],
)
def test_glibc_version_string(version_str, expected, monkeypatch):
    class LibcVersion:
        def __init__(self, version_str):
            self.version_str = version_str

        def __call__(self):
            return version_str

    class ProcessNamespace:
        def __init__(self, libc_version):
            self.gnu_get_libc_version = libc_version

    process_namespace = ProcessNamespace(LibcVersion(version_str))
    monkeypatch.setattr(ctypes, "CDLL", lambda _: process_namespace)
    monkeypatch.setattr(_manylinux, "_glibc_version_string_confstr", lambda: False)

    assert _glibc_version_string() == expected

    del process_namespace.gnu_get_libc_version
    assert _glibc_version_string() is None


def test_glibc_version_string_confstr(monkeypatch):
    monkeypatch.setattr(os, "confstr", lambda x: "glibc 2.20", raising=False)
    assert _glibc_version_string_confstr() == "2.20"


def test_glibc_version_string_fail(monkeypatch):
    monkeypatch.setattr(os, "confstr", lambda x: None, raising=False)
    monkeypatch.setitem(sys.modules, "ctypes", None)
    assert _glibc_version_string() is None
    assert _get_glibc_version() == (-1, -1)


@pytest.mark.parametrize(
    "failure",
    [pretend.raiser(ValueError), pretend.raiser(OSError), lambda x: "XXX"],
)
def test_glibc_version_string_confstr_fail(monkeypatch, failure):
    monkeypatch.setattr(os, "confstr", failure, raising=False)
    assert _glibc_version_string_confstr() is None


def test_glibc_version_string_confstr_missing(monkeypatch):
    monkeypatch.delattr(os, "confstr", raising=False)
    assert _glibc_version_string_confstr() is None


def test_glibc_version_string_ctypes_missing(monkeypatch):
    monkeypatch.setitem(sys.modules, "ctypes", None)
    assert _glibc_version_string_ctypes() is None


def test_glibc_version_string_ctypes_raise_oserror(monkeypatch):
    def patched_cdll(name):
        raise OSError("Dynamic loading not supported")

    monkeypatch.setattr(ctypes, "CDLL", patched_cdll)
    assert _glibc_version_string_ctypes() is None


@pytest.mark.skipif(platform.system() != "Linux", reason="requires Linux")
def test_is_manylinux_compatible_old():
    # Assuming no one is running this test with a version of glibc released in
    # 1997.
    assert _is_compatible("any", "any", (2, 0))


def test_is_manylinux_compatible(monkeypatch):
    monkeypatch.setattr(_manylinux, "_glibc_version_string", lambda: "2.4")
    assert _is_compatible("", "any", (2, 4))


def test_glibc_version_string_none(monkeypatch):
    monkeypatch.setattr(_manylinux, "_glibc_version_string", lambda: None)
    assert not _is_compatible("any", "any", (2, 4))


def test_is_linux_armhf_not_elf(monkeypatch):
    monkeypatch.setattr(_manylinux, "_get_elf_header", lambda: None)
    assert not _is_linux_armhf()


def test_is_linux_i686_not_elf(monkeypatch):
    monkeypatch.setattr(_manylinux, "_get_elf_header", lambda: None)
    assert not _is_linux_i686()


@pytest.mark.parametrize(
    "machine, abi, elf_class, elf_data, elf_machine",
    [
        (
            "x86_64",
            "x32",
            _ELFFileHeader.ELFCLASS32,
            _ELFFileHeader.ELFDATA2LSB,
            _ELFFileHeader.EM_X86_64,
        ),
        (
            "x86_64",
            "i386",
            _ELFFileHeader.ELFCLASS32,
            _ELFFileHeader.ELFDATA2LSB,
            _ELFFileHeader.EM_386,
        ),
        (
            "x86_64",
            "amd64",
            _ELFFileHeader.ELFCLASS64,
            _ELFFileHeader.ELFDATA2LSB,
            _ELFFileHeader.EM_X86_64,
        ),
        (
            "armv7l",
            "armel",
            _ELFFileHeader.ELFCLASS32,
            _ELFFileHeader.ELFDATA2LSB,
            _ELFFileHeader.EM_ARM,
        ),
        (
            "armv7l",
            "armhf",
            _ELFFileHeader.ELFCLASS32,
            _ELFFileHeader.ELFDATA2LSB,
            _ELFFileHeader.EM_ARM,
        ),
        (
            "s390x",
            "s390x",
            _ELFFileHeader.ELFCLASS64,
            _ELFFileHeader.ELFDATA2MSB,
            _ELFFileHeader.EM_S390,
        ),
    ],
)
def test_get_elf_header(monkeypatch, machine, abi, elf_class, elf_data, elf_machine):
    path = os.path.join(
        os.path.dirname(__file__),
        "manylinux",
        f"hello-world-{machine}-{abi}",
    )
    monkeypatch.setattr(sys, "executable", path)
    elf_header = _get_elf_header()
    assert elf_header.e_ident_class == elf_class
    assert elf_header.e_ident_data == elf_data
    assert elf_header.e_machine == elf_machine


@pytest.mark.parametrize(
    "content", [None, "invalid-magic", "invalid-class", "invalid-data", "too-short"]
)
def test_get_elf_header_bad_executable(monkeypatch, content):
    if content:
        path = os.path.join(
            os.path.dirname(__file__),
            "manylinux",
            f"hello-world-{content}",
        )
    else:
        path = None
    monkeypatch.setattr(sys, "executable", path)
    assert _get_elf_header() is None

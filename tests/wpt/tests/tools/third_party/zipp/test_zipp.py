# coding: utf-8

from __future__ import division, unicode_literals

import io
import zipfile
import contextlib
import tempfile
import shutil
import string

try:
    import pathlib
except ImportError:
    import pathlib2 as pathlib

if not hasattr(contextlib, 'ExitStack'):
    import contextlib2
    contextlib.ExitStack = contextlib2.ExitStack

try:
    import unittest

    unittest.TestCase.subTest
except AttributeError:
    import unittest2 as unittest

import jaraco.itertools
import func_timeout

import zipp

__metaclass__ = type
consume = tuple


def add_dirs(zf):
    """
    Given a writable zip file zf, inject directory entries for
    any directories implied by the presence of children.
    """
    for name in zipp.CompleteDirs._implied_dirs(zf.namelist()):
        zf.writestr(name, b"")
    return zf


def build_alpharep_fixture():
    """
    Create a zip file with this structure:

    .
    ├── a.txt
    ├── b
    │   ├── c.txt
    │   ├── d
    │   │   └── e.txt
    │   └── f.txt
    └── g
        └── h
            └── i.txt

    This fixture has the following key characteristics:

    - a file at the root (a)
    - a file two levels deep (b/d/e)
    - multiple files in a directory (b/c, b/f)
    - a directory containing only a directory (g/h)

    "alpha" because it uses alphabet
    "rep" because it's a representative example
    """
    data = io.BytesIO()
    zf = zipfile.ZipFile(data, "w")
    zf.writestr("a.txt", b"content of a")
    zf.writestr("b/c.txt", b"content of c")
    zf.writestr("b/d/e.txt", b"content of e")
    zf.writestr("b/f.txt", b"content of f")
    zf.writestr("g/h/i.txt", b"content of i")
    zf.filename = "alpharep.zip"
    return zf


@contextlib.contextmanager
def temp_dir():
    tmpdir = tempfile.mkdtemp()
    try:
        yield pathlib.Path(tmpdir)
    finally:
        shutil.rmtree(tmpdir)


class TestPath(unittest.TestCase):
    def setUp(self):
        self.fixtures = contextlib.ExitStack()
        self.addCleanup(self.fixtures.close)

    def zipfile_alpharep(self):
        with self.subTest():
            yield build_alpharep_fixture()
        with self.subTest():
            yield add_dirs(build_alpharep_fixture())

    def zipfile_ondisk(self):
        tmpdir = pathlib.Path(self.fixtures.enter_context(temp_dir()))
        for alpharep in self.zipfile_alpharep():
            buffer = alpharep.fp
            alpharep.close()
            path = tmpdir / alpharep.filename
            with path.open("wb") as strm:
                strm.write(buffer.getvalue())
            yield path

    def test_iterdir_and_types(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            assert root.is_dir()
            a, b, g = root.iterdir()
            assert a.is_file()
            assert b.is_dir()
            assert g.is_dir()
            c, f, d = b.iterdir()
            assert c.is_file() and f.is_file()
            e, = d.iterdir()
            assert e.is_file()
            h, = g.iterdir()
            i, = h.iterdir()
            assert i.is_file()

    def test_open(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            a, b, g = root.iterdir()
            with a.open() as strm:
                data = strm.read()
            assert data == "content of a"

    def test_read(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            a, b, g = root.iterdir()
            assert a.read_text() == "content of a"
            assert a.read_bytes() == b"content of a"

    def test_joinpath(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            a = root.joinpath("a")
            assert a.is_file()
            e = root.joinpath("b").joinpath("d").joinpath("e.txt")
            assert e.read_text() == "content of e"

    def test_traverse_truediv(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            a = root / "a"
            assert a.is_file()
            e = root / "b" / "d" / "e.txt"
            assert e.read_text() == "content of e"

    def test_traverse_simplediv(self):
        """
        Disable the __future__.division when testing traversal.
        """
        for alpharep in self.zipfile_alpharep():
            code = compile(
                source="zipp.Path(alpharep) / 'a'",
                filename="(test)",
                mode="eval",
                dont_inherit=True,
            )
            eval(code)

    def test_pathlike_construction(self):
        """
        zipp.Path should be constructable from a path-like object
        """
        for zipfile_ondisk in self.zipfile_ondisk():
            pathlike = pathlib.Path(str(zipfile_ondisk))
            zipp.Path(pathlike)

    def test_traverse_pathlike(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            root / pathlib.Path("a")

    def test_parent(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            assert (root / 'a').parent.at == ''
            assert (root / 'a' / 'b').parent.at == 'a/'

    def test_dir_parent(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            assert (root / 'b').parent.at == ''
            assert (root / 'b/').parent.at == ''

    def test_missing_dir_parent(self):
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            assert (root / 'missing dir/').parent.at == ''

    def test_mutability(self):
        """
        If the underlying zipfile is changed, the Path object should
        reflect that change.
        """
        for alpharep in self.zipfile_alpharep():
            root = zipp.Path(alpharep)
            a, b, g = root.iterdir()
            alpharep.writestr('foo.txt', b'foo')
            alpharep.writestr('bar/baz.txt', b'baz')
            assert any(
                child.name == 'foo.txt'
                for child in root.iterdir())
            assert (root / 'foo.txt').read_text() == 'foo'
            baz, = (root / 'bar').iterdir()
            assert baz.read_text() == 'baz'

    HUGE_ZIPFILE_NUM_ENTRIES = 2 ** 13

    def huge_zipfile(self):
        """Create a read-only zipfile with a huge number of entries entries."""
        strm = io.BytesIO()
        zf = zipfile.ZipFile(strm, "w")
        for entry in map(str, range(self.HUGE_ZIPFILE_NUM_ENTRIES)):
            zf.writestr(entry, entry)
        zf.mode = 'r'
        return zf

    def test_joinpath_constant_time(self):
        """
        Ensure joinpath on items in zipfile is linear time.
        """
        root = zipp.Path(self.huge_zipfile())
        entries = jaraco.itertools.Counter(root.iterdir())
        for entry in entries:
            entry.joinpath('suffix')
        # Check the file iterated all items
        assert entries.count == self.HUGE_ZIPFILE_NUM_ENTRIES

    @func_timeout.func_set_timeout(3)
    def test_implied_dirs_performance(self):
        data = ['/'.join(string.ascii_lowercase + str(n)) for n in range(10000)]
        zipp.CompleteDirs._implied_dirs(data)

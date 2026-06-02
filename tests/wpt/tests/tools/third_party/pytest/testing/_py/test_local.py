# mypy: allow-untyped-defs
import contextlib
import multiprocessing
import os
import sys
import time
from unittest import mock
import warnings

from py import error
from py.path import local

import pytest


@contextlib.contextmanager
def ignore_encoding_warning():
    with warnings.catch_warnings():
        if sys.version_info > (3, 10):
            warnings.simplefilter("ignore", EncodingWarning)
        yield


class CommonFSTests:
    def test_constructor_equality(self, path1):
        p = path1.__class__(path1)
        assert p == path1

    def test_eq_nonstring(self, path1):
        p1 = path1.join("sampledir")
        p2 = path1.join("sampledir")
        assert p1 == p2

    def test_new_identical(self, path1):
        assert path1 == path1.new()

    def test_join(self, path1):
        p = path1.join("sampledir")
        strp = str(p)
        assert strp.endswith("sampledir")
        assert strp.startswith(str(path1))

    def test_join_normalized(self, path1):
        newpath = path1.join(path1.sep + "sampledir")
        strp = str(newpath)
        assert strp.endswith("sampledir")
        assert strp.startswith(str(path1))
        newpath = path1.join((path1.sep * 2) + "sampledir")
        strp = str(newpath)
        assert strp.endswith("sampledir")
        assert strp.startswith(str(path1))

    def test_join_noargs(self, path1):
        newpath = path1.join()
        assert path1 == newpath

    def test_add_something(self, path1):
        p = path1.join("sample")
        p = p + "dir"
        assert p.check()
        assert p.exists()
        assert p.isdir()
        assert not p.isfile()

    def test_parts(self, path1):
        newpath = path1.join("sampledir", "otherfile")
        par = newpath.parts()[-3:]
        assert par == [path1, path1.join("sampledir"), newpath]

        revpar = newpath.parts(reverse=True)[:3]
        assert revpar == [newpath, path1.join("sampledir"), path1]

    def test_common(self, path1):
        other = path1.join("sampledir")
        x = other.common(path1)
        assert x == path1

    # def test_parents_nonexisting_file(self, path1):
    #    newpath = path1 / 'dirnoexist' / 'nonexisting file'
    #    par = list(newpath.parents())
    #    assert par[:2] == [path1 / 'dirnoexist', path1]

    def test_basename_checks(self, path1):
        newpath = path1.join("sampledir")
        assert newpath.check(basename="sampledir")
        assert newpath.check(notbasename="xyz")
        assert newpath.basename == "sampledir"

    def test_basename(self, path1):
        newpath = path1.join("sampledir")
        assert newpath.check(basename="sampledir")
        assert newpath.basename, "sampledir"

    def test_dirname(self, path1):
        newpath = path1.join("sampledir")
        assert newpath.dirname == str(path1)

    def test_dirpath(self, path1):
        newpath = path1.join("sampledir")
        assert newpath.dirpath() == path1

    def test_dirpath_with_args(self, path1):
        newpath = path1.join("sampledir")
        assert newpath.dirpath("x") == path1.join("x")

    def test_newbasename(self, path1):
        newpath = path1.join("samplefile")
        newbase = newpath.new(basename="samplefile2")
        assert newbase.basename == "samplefile2"
        assert newbase.dirpath() == newpath.dirpath()

    def test_not_exists(self, path1):
        assert not path1.join("does_not_exist").check()
        assert path1.join("does_not_exist").check(exists=0)

    def test_exists(self, path1):
        assert path1.join("samplefile").check()
        assert path1.join("samplefile").check(exists=1)
        assert path1.join("samplefile").exists()
        assert path1.join("samplefile").isfile()
        assert not path1.join("samplefile").isdir()

    def test_dir(self, path1):
        # print repr(path1.join("sampledir"))
        assert path1.join("sampledir").check(dir=1)
        assert path1.join("samplefile").check(notdir=1)
        assert not path1.join("samplefile").check(dir=1)
        assert path1.join("samplefile").exists()
        assert not path1.join("samplefile").isdir()
        assert path1.join("samplefile").isfile()

    def test_fnmatch_file(self, path1):
        assert path1.join("samplefile").check(fnmatch="s*e")
        assert path1.join("samplefile").fnmatch("s*e")
        assert not path1.join("samplefile").fnmatch("s*x")
        assert not path1.join("samplefile").check(fnmatch="s*x")

    # def test_fnmatch_dir(self, path1):

    #    pattern = path1.sep.join(['s*file'])
    #    sfile = path1.join("samplefile")
    #    assert sfile.check(fnmatch=pattern)

    def test_relto(self, path1):
        p = path1.join("sampledir", "otherfile")
        assert p.relto(path1) == p.sep.join(["sampledir", "otherfile"])
        assert p.check(relto=path1)
        assert path1.check(notrelto=p)
        assert not path1.check(relto=p)

    def test_bestrelpath(self, path1):
        curdir = path1
        sep = curdir.sep
        s = curdir.bestrelpath(curdir)
        assert s == "."
        s = curdir.bestrelpath(curdir.join("hello", "world"))
        assert s == "hello" + sep + "world"

        s = curdir.bestrelpath(curdir.dirpath().join("sister"))
        assert s == ".." + sep + "sister"
        assert curdir.bestrelpath(curdir.dirpath()) == ".."

        assert curdir.bestrelpath("hello") == "hello"

    def test_relto_not_relative(self, path1):
        l1 = path1.join("bcde")
        l2 = path1.join("b")
        assert not l1.relto(l2)
        assert not l2.relto(l1)

    def test_listdir(self, path1):
        p = path1.listdir()
        assert path1.join("sampledir") in p
        assert path1.join("samplefile") in p
        with pytest.raises(error.ENOTDIR):
            path1.join("samplefile").listdir()

    def test_listdir_fnmatchstring(self, path1):
        p = path1.listdir("s*dir")
        assert len(p)
        assert p[0], path1.join("sampledir")

    def test_listdir_filter(self, path1):
        p = path1.listdir(lambda x: x.check(dir=1))
        assert path1.join("sampledir") in p
        assert path1.join("samplefile") not in p

    def test_listdir_sorted(self, path1):
        p = path1.listdir(lambda x: x.check(basestarts="sample"), sort=True)
        assert path1.join("sampledir") == p[0]
        assert path1.join("samplefile") == p[1]
        assert path1.join("samplepickle") == p[2]

    def test_visit_nofilter(self, path1):
        lst = []
        for i in path1.visit():
            lst.append(i.relto(path1))
        assert "sampledir" in lst
        assert path1.sep.join(["sampledir", "otherfile"]) in lst

    def test_visit_norecurse(self, path1):
        lst = []
        for i in path1.visit(None, lambda x: x.basename != "sampledir"):
            lst.append(i.relto(path1))
        assert "sampledir" in lst
        assert path1.sep.join(["sampledir", "otherfile"]) not in lst

    @pytest.mark.parametrize(
        "fil",
        ["*dir", "*dir", pytest.mark.skip("sys.version_info <" " (3,6)")(b"*dir")],
    )
    def test_visit_filterfunc_is_string(self, path1, fil):
        lst = []
        for i in path1.visit(fil):
            lst.append(i.relto(path1))
        assert len(lst), 2
        assert "sampledir" in lst
        assert "otherdir" in lst

    def test_visit_ignore(self, path1):
        p = path1.join("nonexisting")
        assert list(p.visit(ignore=error.ENOENT)) == []

    def test_visit_endswith(self, path1):
        p = []
        for i in path1.visit(lambda x: x.check(endswith="file")):
            p.append(i.relto(path1))
        assert path1.sep.join(["sampledir", "otherfile"]) in p
        assert "samplefile" in p

    def test_cmp(self, path1):
        path1 = path1.join("samplefile")
        path2 = path1.join("samplefile2")
        assert (path1 < path2) == ("samplefile" < "samplefile2")
        assert not (path1 < path1)

    def test_simple_read(self, path1):
        with ignore_encoding_warning():
            x = path1.join("samplefile").read("r")
        assert x == "samplefile\n"

    def test_join_div_operator(self, path1):
        newpath = path1 / "/sampledir" / "/test//"
        newpath2 = path1.join("sampledir", "test")
        assert newpath == newpath2

    def test_ext(self, path1):
        newpath = path1.join("sampledir.ext")
        assert newpath.ext == ".ext"
        newpath = path1.join("sampledir")
        assert not newpath.ext

    def test_purebasename(self, path1):
        newpath = path1.join("samplefile.py")
        assert newpath.purebasename == "samplefile"

    def test_multiple_parts(self, path1):
        newpath = path1.join("samplefile.py")
        dirname, purebasename, basename, ext = newpath._getbyspec(
            "dirname,purebasename,basename,ext"
        )
        assert str(path1).endswith(dirname)  # be careful with win32 'drive'
        assert purebasename == "samplefile"
        assert basename == "samplefile.py"
        assert ext == ".py"

    def test_dotted_name_ext(self, path1):
        newpath = path1.join("a.b.c")
        ext = newpath.ext
        assert ext == ".c"
        assert newpath.ext == ".c"

    def test_newext(self, path1):
        newpath = path1.join("samplefile.py")
        newext = newpath.new(ext=".txt")
        assert newext.basename == "samplefile.txt"
        assert newext.purebasename == "samplefile"

    def test_readlines(self, path1):
        fn = path1.join("samplefile")
        with ignore_encoding_warning():
            contents = fn.readlines()
        assert contents == ["samplefile\n"]

    def test_readlines_nocr(self, path1):
        fn = path1.join("samplefile")
        with ignore_encoding_warning():
            contents = fn.readlines(cr=0)
        assert contents == ["samplefile", ""]

    def test_file(self, path1):
        assert path1.join("samplefile").check(file=1)

    def test_not_file(self, path1):
        assert not path1.join("sampledir").check(file=1)
        assert path1.join("sampledir").check(file=0)

    def test_non_existent(self, path1):
        assert path1.join("sampledir.nothere").check(dir=0)
        assert path1.join("sampledir.nothere").check(file=0)
        assert path1.join("sampledir.nothere").check(notfile=1)
        assert path1.join("sampledir.nothere").check(notdir=1)
        assert path1.join("sampledir.nothere").check(notexists=1)
        assert not path1.join("sampledir.nothere").check(notfile=0)

    #    pattern = path1.sep.join(['s*file'])
    #    sfile = path1.join("samplefile")
    #    assert sfile.check(fnmatch=pattern)

    def test_size(self, path1):
        url = path1.join("samplefile")
        assert url.size() > len("samplefile")

    def test_mtime(self, path1):
        url = path1.join("samplefile")
        assert url.mtime() > 0

    def test_relto_wrong_type(self, path1):
        with pytest.raises(TypeError):
            path1.relto(42)

    def test_load(self, path1):
        p = path1.join("samplepickle")
        obj = p.load()
        assert type(obj) is dict
        assert obj.get("answer", None) == 42

    def test_visit_filesonly(self, path1):
        p = []
        for i in path1.visit(lambda x: x.check(file=1)):
            p.append(i.relto(path1))
        assert "sampledir" not in p
        assert path1.sep.join(["sampledir", "otherfile"]) in p

    def test_visit_nodotfiles(self, path1):
        p = []
        for i in path1.visit(lambda x: x.check(dotfile=0)):
            p.append(i.relto(path1))
        assert "sampledir" in p
        assert path1.sep.join(["sampledir", "otherfile"]) in p
        assert ".dotfile" not in p

    def test_visit_breadthfirst(self, path1):
        lst = []
        for i in path1.visit(bf=True):
            lst.append(i.relto(path1))
        for i, p in enumerate(lst):
            if path1.sep in p:
                for j in range(i, len(lst)):
                    assert path1.sep in lst[j]
                break
        else:
            pytest.fail("huh")

    def test_visit_sort(self, path1):
        lst = []
        for i in path1.visit(bf=True, sort=True):
            lst.append(i.relto(path1))
        for i, p in enumerate(lst):
            if path1.sep in p:
                break
        assert lst[:i] == sorted(lst[:i])
        assert lst[i:] == sorted(lst[i:])

    def test_endswith(self, path1):
        def chk(p):
            return p.check(endswith="pickle")

        assert not chk(path1)
        assert not chk(path1.join("samplefile"))
        assert chk(path1.join("somepickle"))

    def test_copy_file(self, path1):
        otherdir = path1.join("otherdir")
        initpy = otherdir.join("__init__.py")
        copied = otherdir.join("copied")
        initpy.copy(copied)
        try:
            assert copied.check()
            s1 = initpy.read_text(encoding="utf-8")
            s2 = copied.read_text(encoding="utf-8")
            assert s1 == s2
        finally:
            if copied.check():
                copied.remove()

    def test_copy_dir(self, path1):
        otherdir = path1.join("otherdir")
        copied = path1.join("newdir")
        try:
            otherdir.copy(copied)
            assert copied.check(dir=1)
            assert copied.join("__init__.py").check(file=1)
            s1 = otherdir.join("__init__.py").read_text(encoding="utf-8")
            s2 = copied.join("__init__.py").read_text(encoding="utf-8")
            assert s1 == s2
        finally:
            if copied.check(dir=1):
                copied.remove(rec=1)

    def test_remove_file(self, path1):
        d = path1.ensure("todeleted")
        assert d.check()
        d.remove()
        assert not d.check()

    def test_remove_dir_recursive_by_default(self, path1):
        d = path1.ensure("to", "be", "deleted")
        assert d.check()
        p = path1.join("to")
        p.remove()
        assert not p.check()

    def test_ensure_dir(self, path1):
        b = path1.ensure_dir("001", "002")
        assert b.basename == "002"
        assert b.isdir()

    def test_mkdir_and_remove(self, path1):
        tmpdir = path1
        with pytest.raises(error.EEXIST):
            tmpdir.mkdir("sampledir")
        new = tmpdir.join("mktest1")
        new.mkdir()
        assert new.check(dir=1)
        new.remove()

        new = tmpdir.mkdir("mktest")
        assert new.check(dir=1)
        new.remove()
        assert tmpdir.join("mktest") == new

    def test_move_file(self, path1):
        p = path1.join("samplefile")
        newp = p.dirpath("moved_samplefile")
        p.move(newp)
        try:
            assert newp.check(file=1)
            assert not p.check()
        finally:
            dp = newp.dirpath()
            if hasattr(dp, "revert"):
                dp.revert()
            else:
                newp.move(p)
                assert p.check()

    def test_move_dir(self, path1):
        source = path1.join("sampledir")
        dest = path1.join("moveddir")
        source.move(dest)
        assert dest.check(dir=1)
        assert dest.join("otherfile").check(file=1)
        assert not source.join("sampledir").check()

    def test_fspath_protocol_match_strpath(self, path1):
        assert path1.__fspath__() == path1.strpath

    def test_fspath_func_match_strpath(self, path1):
        from os import fspath

        assert fspath(path1) == path1.strpath

    @pytest.mark.skip("sys.version_info < (3,6)")
    def test_fspath_open(self, path1):
        f = path1.join("opentestfile")
        open(f)

    @pytest.mark.skip("sys.version_info < (3,6)")
    def test_fspath_fsencode(self, path1):
        from os import fsencode

        assert fsencode(path1) == fsencode(path1.strpath)


def setuptestfs(path):
    if path.join("samplefile").check():
        return
    # print "setting up test fs for", repr(path)
    samplefile = path.ensure("samplefile")
    samplefile.write_text("samplefile\n", encoding="utf-8")

    execfile = path.ensure("execfile")
    execfile.write_text("x=42", encoding="utf-8")

    execfilepy = path.ensure("execfile.py")
    execfilepy.write_text("x=42", encoding="utf-8")

    d = {1: 2, "hello": "world", "answer": 42}
    path.ensure("samplepickle").dump(d)

    sampledir = path.ensure("sampledir", dir=1)
    sampledir.ensure("otherfile")

    otherdir = path.ensure("otherdir", dir=1)
    otherdir.ensure("__init__.py")

    module_a = otherdir.ensure("a.py")
    module_a.write_text("from .b import stuff as result\n", encoding="utf-8")
    module_b = otherdir.ensure("b.py")
    module_b.write_text('stuff="got it"\n', encoding="utf-8")
    module_c = otherdir.ensure("c.py")
    module_c.write_text(
        """import py;
import otherdir.a
value = otherdir.a.result
""",
        encoding="utf-8",
    )
    module_d = otherdir.ensure("d.py")
    module_d.write_text(
        """import py;
from otherdir import a
value2 = a.result
""",
        encoding="utf-8",
    )


win32only = pytest.mark.skipif(
    "not (sys.platform == 'win32' or getattr(os, '_name', None) == 'nt')"
)
skiponwin32 = pytest.mark.skipif(
    "sys.platform == 'win32' or getattr(os, '_name', None) == 'nt'"
)

ATIME_RESOLUTION = 0.01


@pytest.fixture(scope="session")
def path1(tmpdir_factory):
    path = tmpdir_factory.mktemp("path")
    setuptestfs(path)
    yield path
    assert path.join("samplefile").check()


@pytest.fixture
def fake_fspath_obj(request):
    class FakeFSPathClass:
        def __init__(self, path):
            self._path = path

        def __fspath__(self):
            return self._path

    return FakeFSPathClass(os.path.join("this", "is", "a", "fake", "path"))


def batch_make_numbered_dirs(rootdir, repeats):
    for i in range(repeats):
        dir_ = local.make_numbered_dir(prefix="repro-", rootdir=rootdir)
        file_ = dir_.join("foo")
        file_.write_text("%s" % i, encoding="utf-8")
        actual = int(file_.read_text(encoding="utf-8"))
        assert (
            actual == i
        ), f"int(file_.read_text(encoding='utf-8')) is {actual} instead of {i}"
        dir_.join(".lock").remove(ignore_errors=True)
    return True


class TestLocalPath(CommonFSTests):
    def test_join_normpath(self, tmpdir):
        assert tmpdir.join(".") == tmpdir
        p = tmpdir.join("../%s" % tmpdir.basename)
        assert p == tmpdir
        p = tmpdir.join("..//%s/" % tmpdir.basename)
        assert p == tmpdir

    @skiponwin32
    def test_dirpath_abs_no_abs(self, tmpdir):
        p = tmpdir.join("foo")
        assert p.dirpath("/bar") == tmpdir.join("bar")
        assert tmpdir.dirpath("/bar", abs=True) == local("/bar")

    def test_gethash(self, tmpdir):
        from hashlib import md5
        from hashlib import sha1 as sha

        fn = tmpdir.join("testhashfile")
        data = b"hello"
        fn.write(data, mode="wb")
        assert fn.computehash("md5") == md5(data).hexdigest()
        assert fn.computehash("sha1") == sha(data).hexdigest()
        with pytest.raises(ValueError):
            fn.computehash("asdasd")

    def test_remove_removes_readonly_file(self, tmpdir):
        readonly_file = tmpdir.join("readonly").ensure()
        readonly_file.chmod(0)
        readonly_file.remove()
        assert not readonly_file.check(exists=1)

    def test_remove_removes_readonly_dir(self, tmpdir):
        readonly_dir = tmpdir.join("readonlydir").ensure(dir=1)
        readonly_dir.chmod(int("500", 8))
        readonly_dir.remove()
        assert not readonly_dir.check(exists=1)

    def test_remove_removes_dir_and_readonly_file(self, tmpdir):
        readonly_dir = tmpdir.join("readonlydir").ensure(dir=1)
        readonly_file = readonly_dir.join("readonlyfile").ensure()
        readonly_file.chmod(0)
        readonly_dir.remove()
        assert not readonly_dir.check(exists=1)

    def test_remove_routes_ignore_errors(self, tmpdir, monkeypatch):
        lst = []
        monkeypatch.setattr("shutil.rmtree", lambda *args, **kwargs: lst.append(kwargs))
        tmpdir.remove()
        assert not lst[0]["ignore_errors"]
        for val in (True, False):
            lst[:] = []
            tmpdir.remove(ignore_errors=val)
            assert lst[0]["ignore_errors"] == val

    def test_initialize_curdir(self):
        assert str(local()) == os.getcwd()

    @skiponwin32
    def test_chdir_gone(self, path1):
        p = path1.ensure("dir_to_be_removed", dir=1)
        p.chdir()
        p.remove()
        pytest.raises(error.ENOENT, local)
        assert path1.chdir() is None
        assert os.getcwd() == str(path1)

        with pytest.raises(error.ENOENT):
            with p.as_cwd():
                raise NotImplementedError

    @skiponwin32
    def test_chdir_gone_in_as_cwd(self, path1):
        p = path1.ensure("dir_to_be_removed", dir=1)
        p.chdir()
        p.remove()

        with path1.as_cwd() as old:
            assert old is None

    def test_as_cwd(self, path1):
        dir = path1.ensure("subdir", dir=1)
        old = local()
        with dir.as_cwd() as x:
            assert x == old
            assert local() == dir
        assert os.getcwd() == str(old)

    def test_as_cwd_exception(self, path1):
        old = local()
        dir = path1.ensure("subdir", dir=1)
        with pytest.raises(ValueError):
            with dir.as_cwd():
                raise ValueError()
        assert old == local()

    def test_initialize_reldir(self, path1):
        with path1.as_cwd():
            p = local("samplefile")
            assert p.check()

    def test_tilde_expansion(self, monkeypatch, tmpdir):
        monkeypatch.setenv("HOME", str(tmpdir))
        p = local("~", expanduser=True)
        assert p == os.path.expanduser("~")

    @pytest.mark.skipif(
        not sys.platform.startswith("win32"), reason="case-insensitive only on windows"
    )
    def test_eq_hash_are_case_insensitive_on_windows(self):
        a = local("/some/path")
        b = local("/some/PATH")
        assert a == b
        assert hash(a) == hash(b)
        assert a in {b}
        assert a in {b: "b"}

    def test_eq_with_strings(self, path1):
        path1 = path1.join("sampledir")
        path2 = str(path1)
        assert path1 == path2
        assert path2 == path1
        path3 = path1.join("samplefile")
        assert path3 != path2
        assert path2 != path3

    def test_eq_with_none(self, path1):
        assert path1 != None  # noqa: E711

    def test_eq_non_ascii_unicode(self, path1):
        path2 = path1.join("temp")
        path3 = path1.join("ação")
        path4 = path1.join("ディレクトリ")

        assert path2 != path3
        assert path2 != path4
        assert path4 != path3

    def test_gt_with_strings(self, path1):
        path2 = path1.join("sampledir")
        path3 = str(path1.join("ttt"))
        assert path3 > path2
        assert path2 < path3
        assert path2 < "ttt"
        assert "ttt" > path2
        path4 = path1.join("aaa")
        lst = [path2, path4, path3]
        assert sorted(lst) == [path4, path2, path3]

    def test_open_and_ensure(self, path1):
        p = path1.join("sub1", "sub2", "file")
        with p.open("w", ensure=1, encoding="utf-8") as f:
            f.write("hello")
        assert p.read_text(encoding="utf-8") == "hello"

    def test_write_and_ensure(self, path1):
        p = path1.join("sub1", "sub2", "file")
        p.write_text("hello", ensure=1, encoding="utf-8")
        assert p.read_text(encoding="utf-8") == "hello"

    @pytest.mark.parametrize("bin", (False, True))
    def test_dump(self, tmpdir, bin):
        path = tmpdir.join("dumpfile%s" % int(bin))
        try:
            d = {"answer": 42}
            path.dump(d, bin=bin)
            f = path.open("rb+")
            import pickle

            dnew = pickle.load(f)
            assert d == dnew
        finally:
            f.close()

    def test_setmtime(self):
        import tempfile
        import time

        try:
            fd, name = tempfile.mkstemp()
            os.close(fd)
        except AttributeError:
            name = tempfile.mktemp()
            open(name, "w").close()
        try:
            mtime = int(time.time()) - 100
            path = local(name)
            assert path.mtime() != mtime
            path.setmtime(mtime)
            assert path.mtime() == mtime
            path.setmtime()
            assert path.mtime() != mtime
        finally:
            os.remove(name)

    def test_normpath(self, path1):
        new1 = path1.join("/otherdir")
        new2 = path1.join("otherdir")
        assert str(new1) == str(new2)

    def test_mkdtemp_creation(self):
        d = local.mkdtemp()
        try:
            assert d.check(dir=1)
        finally:
            d.remove(rec=1)

    def test_tmproot(self):
        d = local.mkdtemp()
        tmproot = local.get_temproot()
        try:
            assert d.check(dir=1)
            assert d.dirpath() == tmproot
        finally:
            d.remove(rec=1)

    def test_chdir(self, tmpdir):
        old = local()
        try:
            res = tmpdir.chdir()
            assert str(res) == str(old)
            assert os.getcwd() == str(tmpdir)
        finally:
            old.chdir()

    def test_ensure_filepath_withdir(self, tmpdir):
        newfile = tmpdir.join("test1", "test")
        newfile.ensure()
        assert newfile.check(file=1)
        newfile.write_text("42", encoding="utf-8")
        newfile.ensure()
        s = newfile.read_text(encoding="utf-8")
        assert s == "42"

    def test_ensure_filepath_withoutdir(self, tmpdir):
        newfile = tmpdir.join("test1file")
        t = newfile.ensure()
        assert t == newfile
        assert newfile.check(file=1)

    def test_ensure_dirpath(self, tmpdir):
        newfile = tmpdir.join("test1", "testfile")
        t = newfile.ensure(dir=1)
        assert t == newfile
        assert newfile.check(dir=1)

    def test_ensure_non_ascii_unicode(self, tmpdir):
        newfile = tmpdir.join("ação", "ディレクトリ")
        t = newfile.ensure(dir=1)
        assert t == newfile
        assert newfile.check(dir=1)

    @pytest.mark.xfail(run=False, reason="unreliable est for long filenames")
    def test_long_filenames(self, tmpdir):
        if sys.platform == "win32":
            pytest.skip("win32: work around needed for path length limit")
        # see http://codespeak.net/pipermail/py-dev/2008q2/000922.html

        # testing paths > 260 chars (which is Windows' limitation, but
        # depending on how the paths are used), but > 4096 (which is the
        # Linux' limitation) - the behaviour of paths with names > 4096 chars
        # is undetermined
        newfilename = "/test" * 60  # type:ignore[unreachable,unused-ignore]
        l1 = tmpdir.join(newfilename)
        l1.ensure(file=True)
        l1.write_text("foo", encoding="utf-8")
        l2 = tmpdir.join(newfilename)
        assert l2.read_text(encoding="utf-8") == "foo"

    def test_visit_depth_first(self, tmpdir):
        tmpdir.ensure("a", "1")
        tmpdir.ensure("b", "2")
        p3 = tmpdir.ensure("breadth")
        lst = list(tmpdir.visit(lambda x: x.check(file=1)))
        assert len(lst) == 3
        # check that breadth comes last
        assert lst[2] == p3

    def test_visit_rec_fnmatch(self, tmpdir):
        p1 = tmpdir.ensure("a", "123")
        tmpdir.ensure(".b", "345")
        lst = list(tmpdir.visit("???", rec="[!.]*"))
        assert len(lst) == 1
        # check that breadth comes last
        assert lst[0] == p1

    def test_fnmatch_file_abspath(self, tmpdir):
        b = tmpdir.join("a", "b")
        assert b.fnmatch(os.sep.join("ab"))
        pattern = os.sep.join([str(tmpdir), "*", "b"])
        assert b.fnmatch(pattern)

    def test_sysfind(self):
        name = sys.platform == "win32" and "cmd" or "test"
        x = local.sysfind(name)
        assert x.check(file=1)
        assert local.sysfind("jaksdkasldqwe") is None
        assert local.sysfind(name, paths=[]) is None
        x2 = local.sysfind(name, paths=[x.dirpath()])
        assert x2 == x

    def test_fspath_protocol_other_class(self, fake_fspath_obj):
        # py.path is always absolute
        py_path = local(fake_fspath_obj)
        str_path = fake_fspath_obj.__fspath__()
        assert py_path.check(endswith=str_path)
        assert py_path.join(fake_fspath_obj).strpath == os.path.join(
            py_path.strpath, str_path
        )

    @pytest.mark.xfail(
        reason="#11603", raises=(error.EEXIST, error.ENOENT), strict=False
    )
    def test_make_numbered_dir_multiprocess_safe(self, tmpdir):
        # https://github.com/pytest-dev/py/issues/30
        with multiprocessing.Pool() as pool:
            results = [
                pool.apply_async(batch_make_numbered_dirs, [tmpdir, 100])
                for _ in range(20)
            ]
            for r in results:
                assert r.get()


class TestExecutionOnWindows:
    pytestmark = win32only

    def test_sysfind_bat_exe_before(self, tmpdir, monkeypatch):
        monkeypatch.setenv("PATH", str(tmpdir), prepend=os.pathsep)
        tmpdir.ensure("hello")
        h = tmpdir.ensure("hello.bat")
        x = local.sysfind("hello")
        assert x == h


class TestExecution:
    pytestmark = skiponwin32

    def test_sysfind_no_permission_ignored(self, monkeypatch, tmpdir):
        noperm = tmpdir.ensure("noperm", dir=True)
        monkeypatch.setenv("PATH", str(noperm), prepend=":")
        noperm.chmod(0)
        try:
            assert local.sysfind("jaksdkasldqwe") is None
        finally:
            noperm.chmod(0o644)

    def test_sysfind_absolute(self):
        x = local.sysfind("test")
        assert x.check(file=1)
        y = local.sysfind(str(x))
        assert y.check(file=1)
        assert y == x

    def test_sysfind_multiple(self, tmpdir, monkeypatch):
        monkeypatch.setenv(
            "PATH", "{}:{}".format(tmpdir.ensure("a"), tmpdir.join("b")), prepend=":"
        )
        tmpdir.ensure("b", "a")
        x = local.sysfind("a", checker=lambda x: x.dirpath().basename == "b")
        assert x.basename == "a"
        assert x.dirpath().basename == "b"
        assert local.sysfind("a", checker=lambda x: None) is None

    def test_sysexec(self):
        x = local.sysfind("ls")
        out = x.sysexec("-a")
        for x in local().listdir():
            assert out.find(x.basename) != -1

    def test_sysexec_failing(self):
        try:
            from py._process.cmdexec import ExecutionFailed  # py library
        except ImportError:
            ExecutionFailed = RuntimeError  # py vendored
        x = local.sysfind("false")
        with pytest.raises(ExecutionFailed):
            x.sysexec("aksjdkasjd")

    def test_make_numbered_dir(self, tmpdir):
        tmpdir.ensure("base.not_an_int", dir=1)
        for i in range(10):
            numdir = local.make_numbered_dir(
                prefix="base.", rootdir=tmpdir, keep=2, lock_timeout=0
            )
            assert numdir.check()
            assert numdir.basename == "base.%d" % i
            if i >= 1:
                assert numdir.new(ext=str(i - 1)).check()
            if i >= 2:
                assert numdir.new(ext=str(i - 2)).check()
            if i >= 3:
                assert not numdir.new(ext=str(i - 3)).check()

    def test_make_numbered_dir_case(self, tmpdir):
        """make_numbered_dir does not make assumptions on the underlying
        filesystem based on the platform and will assume it _could_ be case
        insensitive.

        See issues:
        - https://github.com/pytest-dev/pytest/issues/708
        - https://github.com/pytest-dev/pytest/issues/3451
        """
        d1 = local.make_numbered_dir(
            prefix="CAse.",
            rootdir=tmpdir,
            keep=2,
            lock_timeout=0,
        )
        d2 = local.make_numbered_dir(
            prefix="caSE.",
            rootdir=tmpdir,
            keep=2,
            lock_timeout=0,
        )
        assert str(d1).lower() != str(d2).lower()
        assert str(d2).endswith(".1")

    def test_make_numbered_dir_NotImplemented_Error(self, tmpdir, monkeypatch):
        def notimpl(x, y):
            raise NotImplementedError(42)

        monkeypatch.setattr(os, "symlink", notimpl)
        x = tmpdir.make_numbered_dir(rootdir=tmpdir, lock_timeout=0)
        assert x.relto(tmpdir)
        assert x.check()

    def test_locked_make_numbered_dir(self, tmpdir):
        for i in range(10):
            numdir = local.make_numbered_dir(prefix="base2.", rootdir=tmpdir, keep=2)
            assert numdir.check()
            assert numdir.basename == "base2.%d" % i
            for j in range(i):
                assert numdir.new(ext=str(j)).check()

    def test_error_preservation(self, path1):
        pytest.raises(EnvironmentError, path1.join("qwoeqiwe").mtime)
        pytest.raises(EnvironmentError, path1.join("qwoeqiwe").read)

    # def test_parentdirmatch(self):
    #    local.parentdirmatch('std', startmodule=__name__)
    #


class TestImport:
    @pytest.fixture(autouse=True)
    def preserve_sys(self):
        with mock.patch.dict(sys.modules):
            with mock.patch.object(sys, "path", list(sys.path)):
                yield

    def test_pyimport(self, path1):
        obj = path1.join("execfile.py").pyimport()
        assert obj.x == 42
        assert obj.__name__ == "execfile"

    def test_pyimport_renamed_dir_creates_mismatch(self, tmpdir, monkeypatch):
        p = tmpdir.ensure("a", "test_x123.py")
        p.pyimport()
        tmpdir.join("a").move(tmpdir.join("b"))
        with pytest.raises(tmpdir.ImportMismatchError):
            tmpdir.join("b", "test_x123.py").pyimport()

        # Errors can be ignored.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "1")
        tmpdir.join("b", "test_x123.py").pyimport()

        # PY_IGNORE_IMPORTMISMATCH=0 does not ignore error.
        monkeypatch.setenv("PY_IGNORE_IMPORTMISMATCH", "0")
        with pytest.raises(tmpdir.ImportMismatchError):
            tmpdir.join("b", "test_x123.py").pyimport()

    def test_pyimport_messy_name(self, tmpdir):
        # http://bitbucket.org/hpk42/py-trunk/issue/129
        path = tmpdir.ensure("foo__init__.py")
        path.pyimport()

    def test_pyimport_dir(self, tmpdir):
        p = tmpdir.join("hello_123")
        p_init = p.ensure("__init__.py")
        m = p.pyimport()
        assert m.__name__ == "hello_123"
        m = p_init.pyimport()
        assert m.__name__ == "hello_123"

    def test_pyimport_execfile_different_name(self, path1):
        obj = path1.join("execfile.py").pyimport(modname="0x.y.z")
        assert obj.x == 42
        assert obj.__name__ == "0x.y.z"

    def test_pyimport_a(self, path1):
        otherdir = path1.join("otherdir")
        mod = otherdir.join("a.py").pyimport()
        assert mod.result == "got it"
        assert mod.__name__ == "otherdir.a"

    def test_pyimport_b(self, path1):
        otherdir = path1.join("otherdir")
        mod = otherdir.join("b.py").pyimport()
        assert mod.stuff == "got it"
        assert mod.__name__ == "otherdir.b"

    def test_pyimport_c(self, path1):
        otherdir = path1.join("otherdir")
        mod = otherdir.join("c.py").pyimport()
        assert mod.value == "got it"

    def test_pyimport_d(self, path1):
        otherdir = path1.join("otherdir")
        mod = otherdir.join("d.py").pyimport()
        assert mod.value2 == "got it"

    def test_pyimport_and_import(self, tmpdir):
        tmpdir.ensure("xxxpackage", "__init__.py")
        mod1path = tmpdir.ensure("xxxpackage", "module1.py")
        mod1 = mod1path.pyimport()
        assert mod1.__name__ == "xxxpackage.module1"
        from xxxpackage import module1

        assert module1 is mod1

    def test_pyimport_check_filepath_consistency(self, monkeypatch, tmpdir):
        name = "pointsback123"
        ModuleType = type(os)
        p = tmpdir.ensure(name + ".py")
        with monkeypatch.context() as mp:
            for ending in (".pyc", "$py.class", ".pyo"):
                mod = ModuleType(name)
                pseudopath = tmpdir.ensure(name + ending)
                mod.__file__ = str(pseudopath)
                mp.setitem(sys.modules, name, mod)
                newmod = p.pyimport()
                assert mod == newmod
        mod = ModuleType(name)
        pseudopath = tmpdir.ensure(name + "123.py")
        mod.__file__ = str(pseudopath)
        monkeypatch.setitem(sys.modules, name, mod)
        excinfo = pytest.raises(pseudopath.ImportMismatchError, p.pyimport)
        modname, modfile, orig = excinfo.value.args
        assert modname == name
        assert modfile == pseudopath
        assert orig == p
        assert issubclass(pseudopath.ImportMismatchError, ImportError)

    def test_issue131_pyimport_on__init__(self, tmpdir):
        # __init__.py files may be namespace packages, and thus the
        # __file__ of an imported module may not be ourselves
        # see issue
        p1 = tmpdir.ensure("proja", "__init__.py")
        p2 = tmpdir.ensure("sub", "proja", "__init__.py")
        m1 = p1.pyimport()
        m2 = p2.pyimport()
        assert m1 == m2

    def test_ensuresyspath_append(self, tmpdir):
        root1 = tmpdir.mkdir("root1")
        file1 = root1.ensure("x123.py")
        assert str(root1) not in sys.path
        file1.pyimport(ensuresyspath="append")
        assert str(root1) == sys.path[-1]
        assert str(root1) not in sys.path[:-1]


class TestImportlibImport:
    OPTS = {"ensuresyspath": "importlib"}

    def test_pyimport(self, path1):
        obj = path1.join("execfile.py").pyimport(**self.OPTS)
        assert obj.x == 42
        assert obj.__name__ == "execfile"

    def test_pyimport_dir_fails(self, tmpdir):
        p = tmpdir.join("hello_123")
        p.ensure("__init__.py")
        with pytest.raises(ImportError):
            p.pyimport(**self.OPTS)

    def test_pyimport_execfile_different_name(self, path1):
        obj = path1.join("execfile.py").pyimport(modname="0x.y.z", **self.OPTS)
        assert obj.x == 42
        assert obj.__name__ == "0x.y.z"

    def test_pyimport_relative_import_fails(self, path1):
        otherdir = path1.join("otherdir")
        with pytest.raises(ImportError):
            otherdir.join("a.py").pyimport(**self.OPTS)

    def test_pyimport_doesnt_use_sys_modules(self, tmpdir):
        p = tmpdir.ensure("file738jsk.py")
        mod = p.pyimport(**self.OPTS)
        assert mod.__name__ == "file738jsk"
        assert "file738jsk" not in sys.modules


def test_pypkgdir(tmpdir):
    pkg = tmpdir.ensure("pkg1", dir=1)
    pkg.ensure("__init__.py")
    pkg.ensure("subdir/__init__.py")
    assert pkg.pypkgpath() == pkg
    assert pkg.join("subdir", "__init__.py").pypkgpath() == pkg


def test_pypkgdir_unimportable(tmpdir):
    pkg = tmpdir.ensure("pkg1-1", dir=1)  # unimportable
    pkg.ensure("__init__.py")
    subdir = pkg.ensure("subdir/__init__.py").dirpath()
    assert subdir.pypkgpath() == subdir
    assert subdir.ensure("xyz.py").pypkgpath() == subdir
    assert not pkg.pypkgpath()


def test_isimportable():
    try:
        from py.path import isimportable  # py vendored version
    except ImportError:
        from py._path.local import isimportable  # py library

    assert not isimportable("")
    assert isimportable("x")
    assert isimportable("x1")
    assert isimportable("x_1")
    assert isimportable("_")
    assert isimportable("_1")
    assert not isimportable("x-1")
    assert not isimportable("x:1")


def test_homedir_from_HOME(monkeypatch):
    path = os.getcwd()
    monkeypatch.setenv("HOME", path)
    assert local._gethomedir() == local(path)


def test_homedir_not_exists(monkeypatch):
    monkeypatch.delenv("HOME", raising=False)
    monkeypatch.delenv("HOMEDRIVE", raising=False)
    homedir = local._gethomedir()
    assert homedir is None


def test_samefile(tmpdir):
    assert tmpdir.samefile(tmpdir)
    p = tmpdir.ensure("hello")
    assert p.samefile(p)
    with p.dirpath().as_cwd():
        assert p.samefile(p.basename)
    if sys.platform == "win32":
        p1 = p.__class__(str(p).lower())
        p2 = p.__class__(str(p).upper())
        assert p1.samefile(p2)


@pytest.mark.skipif(not hasattr(os, "symlink"), reason="os.symlink not available")
def test_samefile_symlink(tmpdir):
    p1 = tmpdir.ensure("foo.txt")
    p2 = tmpdir.join("linked.txt")
    try:
        os.symlink(str(p1), str(p2))
    except (OSError, NotImplementedError) as e:
        # on Windows this might fail if the user doesn't have special symlink permissions
        # pypy3 on Windows doesn't implement os.symlink and raises NotImplementedError
        pytest.skip(str(e.args[0]))

    assert p1.samefile(p2)


def test_listdir_single_arg(tmpdir):
    tmpdir.ensure("hello")
    assert tmpdir.listdir("hello")[0].basename == "hello"


def test_mkdtemp_rootdir(tmpdir):
    dtmp = local.mkdtemp(rootdir=tmpdir)
    assert tmpdir.listdir() == [dtmp]


class TestWINLocalPath:
    pytestmark = win32only

    def test_owner_group_not_implemented(self, path1):
        with pytest.raises(NotImplementedError):
            _ = path1.stat().owner
        with pytest.raises(NotImplementedError):
            _ = path1.stat().group

    def test_chmod_simple_int(self, path1):
        mode = path1.stat().mode
        # Ensure that we actually change the mode to something different.
        path1.chmod(mode == 0 and 1 or 0)
        try:
            print(path1.stat().mode)
            print(mode)
            assert path1.stat().mode != mode
        finally:
            path1.chmod(mode)
            assert path1.stat().mode == mode

    def test_path_comparison_lowercase_mixed(self, path1):
        t1 = path1.join("a_path")
        t2 = path1.join("A_path")
        assert t1 == t1
        assert t1 == t2

    def test_relto_with_mixed_case(self, path1):
        t1 = path1.join("a_path", "fiLe")
        t2 = path1.join("A_path")
        assert t1.relto(t2) == "fiLe"

    def test_allow_unix_style_paths(self, path1):
        t1 = path1.join("a_path")
        assert t1 == str(path1) + "\\a_path"
        t1 = path1.join("a_path/")
        assert t1 == str(path1) + "\\a_path"
        t1 = path1.join("dir/a_path")
        assert t1 == str(path1) + "\\dir\\a_path"

    def test_sysfind_in_currentdir(self, path1):
        cmd = local.sysfind("cmd")
        root = cmd.new(dirname="", basename="")  # c:\ in most installations
        with root.as_cwd():
            x = local.sysfind(cmd.relto(root))
            assert x.check(file=1)

    def test_fnmatch_file_abspath_posix_pattern_on_win32(self, tmpdir):
        # path-matching patterns might contain a posix path separator '/'
        # Test that we can match that pattern on windows.
        import posixpath

        b = tmpdir.join("a", "b")
        assert b.fnmatch(posixpath.sep.join("ab"))
        pattern = posixpath.sep.join([str(tmpdir), "*", "b"])
        assert b.fnmatch(pattern)


class TestPOSIXLocalPath:
    pytestmark = skiponwin32

    def test_hardlink(self, tmpdir):
        linkpath = tmpdir.join("test")
        filepath = tmpdir.join("file")
        filepath.write_text("Hello", encoding="utf-8")
        nlink = filepath.stat().nlink
        linkpath.mklinkto(filepath)
        assert filepath.stat().nlink == nlink + 1

    def test_symlink_are_identical(self, tmpdir):
        filepath = tmpdir.join("file")
        filepath.write_text("Hello", encoding="utf-8")
        linkpath = tmpdir.join("test")
        linkpath.mksymlinkto(filepath)
        assert linkpath.readlink() == str(filepath)

    def test_symlink_isfile(self, tmpdir):
        linkpath = tmpdir.join("test")
        filepath = tmpdir.join("file")
        filepath.write_text("", encoding="utf-8")
        linkpath.mksymlinkto(filepath)
        assert linkpath.check(file=1)
        assert not linkpath.check(link=0, file=1)
        assert linkpath.islink()

    def test_symlink_relative(self, tmpdir):
        linkpath = tmpdir.join("test")
        filepath = tmpdir.join("file")
        filepath.write_text("Hello", encoding="utf-8")
        linkpath.mksymlinkto(filepath, absolute=False)
        assert linkpath.readlink() == "file"
        assert filepath.read_text(encoding="utf-8") == linkpath.read_text(
            encoding="utf-8"
        )

    def test_symlink_not_existing(self, tmpdir):
        linkpath = tmpdir.join("testnotexisting")
        assert not linkpath.check(link=1)
        assert linkpath.check(link=0)

    def test_relto_with_root(self, path1, tmpdir):
        y = path1.join("x").relto(local("/"))
        assert y[0] == str(path1)[1]

    def test_visit_recursive_symlink(self, tmpdir):
        linkpath = tmpdir.join("test")
        linkpath.mksymlinkto(tmpdir)
        visitor = tmpdir.visit(None, lambda x: x.check(link=0))
        assert list(visitor) == [linkpath]

    def test_symlink_isdir(self, tmpdir):
        linkpath = tmpdir.join("test")
        linkpath.mksymlinkto(tmpdir)
        assert linkpath.check(dir=1)
        assert not linkpath.check(link=0, dir=1)

    def test_symlink_remove(self, tmpdir):
        linkpath = tmpdir.join("test")
        linkpath.mksymlinkto(linkpath)  # point to itself
        assert linkpath.check(link=1)
        linkpath.remove()
        assert not linkpath.check()

    def test_realpath_file(self, tmpdir):
        linkpath = tmpdir.join("test")
        filepath = tmpdir.join("file")
        filepath.write_text("", encoding="utf-8")
        linkpath.mksymlinkto(filepath)
        realpath = linkpath.realpath()
        assert realpath.basename == "file"

    def test_owner(self, path1, tmpdir):
        from grp import getgrgid  # type:ignore[attr-defined,unused-ignore]
        from pwd import getpwuid  # type:ignore[attr-defined,unused-ignore]

        stat = path1.stat()
        assert stat.path == path1

        uid = stat.uid
        gid = stat.gid
        owner = getpwuid(uid)[0]
        group = getgrgid(gid)[0]

        assert uid == stat.uid
        assert owner == stat.owner
        assert gid == stat.gid
        assert group == stat.group

    def test_stat_helpers(self, tmpdir, monkeypatch):
        path1 = tmpdir.ensure("file")
        stat1 = path1.stat()
        stat2 = tmpdir.stat()
        assert stat1.isfile()
        assert stat2.isdir()
        assert not stat1.islink()
        assert not stat2.islink()

    def test_stat_non_raising(self, tmpdir):
        path1 = tmpdir.join("file")
        pytest.raises(error.ENOENT, lambda: path1.stat())
        res = path1.stat(raising=False)
        assert res is None

    def test_atime(self, tmpdir):
        import time

        path = tmpdir.ensure("samplefile")
        now = time.time()
        atime1 = path.atime()
        # we could wait here but timer resolution is very
        # system dependent
        path.read_binary()
        time.sleep(ATIME_RESOLUTION)
        atime2 = path.atime()
        time.sleep(ATIME_RESOLUTION)
        duration = time.time() - now
        assert (atime2 - atime1) <= duration

    def test_commondir(self, path1):
        # XXX This is here in local until we find a way to implement this
        #     using the subversion command line api.
        p1 = path1.join("something")
        p2 = path1.join("otherthing")
        assert p1.common(p2) == path1
        assert p2.common(p1) == path1

    def test_commondir_nocommon(self, path1):
        # XXX This is here in local until we find a way to implement this
        #     using the subversion command line api.
        p1 = path1.join("something")
        p2 = local(path1.sep + "blabla")
        assert p1.common(p2) == "/"

    def test_join_to_root(self, path1):
        root = path1.parts()[0]
        assert len(str(root)) == 1
        assert str(root.join("a")) == "/a"

    def test_join_root_to_root_with_no_abs(self, path1):
        nroot = path1.join("/")
        assert str(path1) == str(nroot)
        assert path1 == nroot

    def test_chmod_simple_int(self, path1):
        mode = path1.stat().mode
        path1.chmod(int(mode / 2))
        try:
            assert path1.stat().mode != mode
        finally:
            path1.chmod(mode)
            assert path1.stat().mode == mode

    def test_chmod_rec_int(self, path1):
        # XXX fragile test
        def recfilter(x):
            return x.check(dotfile=0, link=0)

        oldmodes = {}
        for x in path1.visit(rec=recfilter):
            oldmodes[x] = x.stat().mode
        path1.chmod(int("772", 8), rec=recfilter)
        try:
            for x in path1.visit(rec=recfilter):
                assert x.stat().mode & int("777", 8) == int("772", 8)
        finally:
            for x, y in oldmodes.items():
                x.chmod(y)

    def test_copy_archiving(self, tmpdir):
        unicode_fn = "something-\342\200\223.txt"
        f = tmpdir.ensure("a", unicode_fn)
        a = f.dirpath()
        oldmode = f.stat().mode
        newmode = oldmode ^ 1
        f.chmod(newmode)
        b = tmpdir.join("b")
        a.copy(b, mode=True)
        assert b.join(f.basename).stat().mode == newmode

    def test_copy_stat_file(self, tmpdir):
        src = tmpdir.ensure("src")
        dst = tmpdir.join("dst")
        # a small delay before the copy
        time.sleep(ATIME_RESOLUTION)
        src.copy(dst, stat=True)
        oldstat = src.stat()
        newstat = dst.stat()
        assert oldstat.mode == newstat.mode
        assert (dst.atime() - src.atime()) < ATIME_RESOLUTION
        assert (dst.mtime() - src.mtime()) < ATIME_RESOLUTION

    def test_copy_stat_dir(self, tmpdir):
        test_files = ["a", "b", "c"]
        src = tmpdir.join("src")
        for f in test_files:
            src.join(f).write_text(f, ensure=True, encoding="utf-8")
        dst = tmpdir.join("dst")
        # a small delay before the copy
        time.sleep(ATIME_RESOLUTION)
        src.copy(dst, stat=True)
        for f in test_files:
            oldstat = src.join(f).stat()
            newstat = dst.join(f).stat()
            assert (newstat.atime - oldstat.atime) < ATIME_RESOLUTION
            assert (newstat.mtime - oldstat.mtime) < ATIME_RESOLUTION
            assert oldstat.mode == newstat.mode

    def test_chown_identity(self, path1):
        owner = path1.stat().owner
        group = path1.stat().group
        path1.chown(owner, group)

    def test_chown_dangling_link(self, path1):
        owner = path1.stat().owner
        group = path1.stat().group
        x = path1.join("hello")
        x.mksymlinkto("qlwkejqwlek")
        try:
            path1.chown(owner, group, rec=1)
        finally:
            x.remove(rec=0)

    def test_chown_identity_rec_mayfail(self, path1):
        owner = path1.stat().owner
        group = path1.stat().group
        path1.chown(owner, group)


class TestUnicode:
    def test_join_ensure(self, tmpdir, monkeypatch):
        if "LANG" not in os.environ:
            pytest.skip("cannot run test without locale")
        x = local(tmpdir.strpath)
        part = "hällo"
        y = x.ensure(part)
        assert x.join(part) == y

    def test_listdir(self, tmpdir):
        if "LANG" not in os.environ:
            pytest.skip("cannot run test without locale")
        x = local(tmpdir.strpath)
        part = "hällo"
        y = x.ensure(part)
        assert x.listdir(part)[0] == y

    @pytest.mark.xfail(reason="changing read/write might break existing usages")
    def test_read_write(self, tmpdir):
        x = tmpdir.join("hello")
        part = "hällo"
        with ignore_encoding_warning():
            x.write(part)
            assert x.read() == part
            x.write(part.encode(sys.getdefaultencoding()))
            assert x.read() == part.encode(sys.getdefaultencoding())


class TestBinaryAndTextMethods:
    def test_read_binwrite(self, tmpdir):
        x = tmpdir.join("hello")
        part = "hällo"
        part_utf8 = part.encode("utf8")
        x.write_binary(part_utf8)
        assert x.read_binary() == part_utf8
        s = x.read_text(encoding="utf8")
        assert s == part
        assert isinstance(s, str)

    def test_read_textwrite(self, tmpdir):
        x = tmpdir.join("hello")
        part = "hällo"
        part_utf8 = part.encode("utf8")
        x.write_text(part, encoding="utf8")
        assert x.read_binary() == part_utf8
        assert x.read_text(encoding="utf8") == part

    def test_default_encoding(self, tmpdir):
        x = tmpdir.join("hello")
        # Can't use UTF8 as the default encoding (ASCII) doesn't support it
        part = "hello"
        x.write_text(part, "ascii")
        s = x.read_text("ascii")
        assert s == part
        assert type(s) is type(part)

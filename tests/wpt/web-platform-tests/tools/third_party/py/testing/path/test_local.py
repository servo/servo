# -*- coding: utf-8 -*-

from __future__ import with_statement
import time
import py
import pytest
import os
import sys
import multiprocessing
from py.path import local
import common

failsonjython = py.test.mark.xfail("sys.platform.startswith('java')")
failsonjywin32 = py.test.mark.xfail(
    "sys.platform.startswith('java') "
    "and getattr(os, '_name', None) == 'nt'")
win32only = py.test.mark.skipif(
        "not (sys.platform == 'win32' or getattr(os, '_name', None) == 'nt')")
skiponwin32 = py.test.mark.skipif(
        "sys.platform == 'win32' or getattr(os, '_name', None) == 'nt'")

ATIME_RESOLUTION = 0.01


@pytest.yield_fixture(scope="session")
def path1(tmpdir_factory):
    path = tmpdir_factory.mktemp('path')
    common.setuptestfs(path)
    yield path
    assert path.join("samplefile").check()


@pytest.fixture
def fake_fspath_obj(request):
    class FakeFSPathClass(object):
        def __init__(self, path):
            self._path = path

        def __fspath__(self):
            return self._path

    return FakeFSPathClass(os.path.join("this", "is", "a", "fake", "path"))


def batch_make_numbered_dirs(rootdir, repeats):
    try:
        for i in range(repeats):
            dir_ = py.path.local.make_numbered_dir(prefix='repro-', rootdir=rootdir)
            file_ = dir_.join('foo')
            file_.write('%s' % i)
            actual = int(file_.read())
            assert actual == i, 'int(file_.read()) is %s instead of %s' % (actual, i)
            dir_.join('.lock').remove(ignore_errors=True)
        return True
    except KeyboardInterrupt:
        # makes sure that interrupting test session won't hang it
        os.exit(2)


class TestLocalPath(common.CommonFSTests):
    def test_join_normpath(self, tmpdir):
        assert tmpdir.join(".") == tmpdir
        p = tmpdir.join("../%s" % tmpdir.basename)
        assert p == tmpdir
        p = tmpdir.join("..//%s/" % tmpdir.basename)
        assert p == tmpdir

    @skiponwin32
    def test_dirpath_abs_no_abs(self, tmpdir):
        p = tmpdir.join('foo')
        assert p.dirpath('/bar') == tmpdir.join('bar')
        assert tmpdir.dirpath('/bar', abs=True) == local('/bar')

    def test_gethash(self, tmpdir):
        md5 = py.builtin._tryimport('md5', 'hashlib').md5
        lib = py.builtin._tryimport('sha', 'hashlib')
        sha = getattr(lib, 'sha1', getattr(lib, 'sha', None))
        fn = tmpdir.join("testhashfile")
        data = 'hello'.encode('ascii')
        fn.write(data, mode="wb")
        assert fn.computehash("md5") == md5(data).hexdigest()
        assert fn.computehash("sha1") == sha(data).hexdigest()
        py.test.raises(ValueError, fn.computehash, "asdasd")

    def test_remove_removes_readonly_file(self, tmpdir):
        readonly_file = tmpdir.join('readonly').ensure()
        readonly_file.chmod(0)
        readonly_file.remove()
        assert not readonly_file.check(exists=1)

    def test_remove_removes_readonly_dir(self, tmpdir):
        readonly_dir = tmpdir.join('readonlydir').ensure(dir=1)
        readonly_dir.chmod(int("500", 8))
        readonly_dir.remove()
        assert not readonly_dir.check(exists=1)

    def test_remove_removes_dir_and_readonly_file(self, tmpdir):
        readonly_dir = tmpdir.join('readonlydir').ensure(dir=1)
        readonly_file = readonly_dir.join('readonlyfile').ensure()
        readonly_file.chmod(0)
        readonly_dir.remove()
        assert not readonly_dir.check(exists=1)

    def test_remove_routes_ignore_errors(self, tmpdir, monkeypatch):
        l = []
        monkeypatch.setattr(
            'shutil.rmtree',
            lambda *args, **kwargs: l.append(kwargs))
        tmpdir.remove()
        assert not l[0]['ignore_errors']
        for val in (True, False):
            l[:] = []
            tmpdir.remove(ignore_errors=val)
            assert l[0]['ignore_errors'] == val

    def test_initialize_curdir(self):
        assert str(local()) == os.getcwd()

    @skiponwin32
    def test_chdir_gone(self, path1):
        p = path1.ensure("dir_to_be_removed", dir=1)
        p.chdir()
        p.remove()
        pytest.raises(py.error.ENOENT, py.path.local)
        assert path1.chdir() is None
        assert os.getcwd() == str(path1)

        with pytest.raises(py.error.ENOENT):
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
        old = py.path.local()
        with dir.as_cwd() as x:
            assert x == old
            assert py.path.local() == dir
        assert os.getcwd() == str(old)

    def test_as_cwd_exception(self, path1):
        old = py.path.local()
        dir = path1.ensure("subdir", dir=1)
        with pytest.raises(ValueError):
            with dir.as_cwd():
                raise ValueError()
        assert old == py.path.local()

    def test_initialize_reldir(self, path1):
        with path1.as_cwd():
            p = local('samplefile')
            assert p.check()

    def test_tilde_expansion(self, monkeypatch, tmpdir):
        monkeypatch.setenv("HOME", str(tmpdir))
        p = py.path.local("~", expanduser=True)
        assert p == os.path.expanduser("~")

    @pytest.mark.skipif(
        not sys.platform.startswith("win32"), reason="case insensitive only on windows"
    )
    def test_eq_hash_are_case_insensitive_on_windows(self):
        a = py.path.local("/some/path")
        b = py.path.local("/some/PATH")
        assert a == b
        assert hash(a) == hash(b)
        assert a in {b}
        assert a in {b: 'b'}

    def test_eq_with_strings(self, path1):
        path1 = path1.join('sampledir')
        path2 = str(path1)
        assert path1 == path2
        assert path2 == path1
        path3 = path1.join('samplefile')
        assert path3 != path2
        assert path2 != path3

    def test_eq_with_none(self, path1):
        assert path1 != None  # noqa: E711

    @pytest.mark.skipif(
        sys.platform.startswith("win32"), reason="cannot remove cwd on Windows"
    )
    @pytest.mark.skipif(
        sys.version_info < (3, 0) or sys.version_info >= (3, 5),
        reason="only with Python 3 before 3.5"
    )
    def test_eq_with_none_and_custom_fspath(self, monkeypatch, path1):
        import os
        import shutil
        import tempfile

        d = tempfile.mkdtemp()
        monkeypatch.chdir(d)
        shutil.rmtree(d)

        monkeypatch.delitem(sys.modules, 'pathlib', raising=False)
        monkeypatch.setattr(sys, 'path', [''] + sys.path)

        with pytest.raises(FileNotFoundError):
            import pathlib  # noqa: F401

        assert path1 != None  # noqa: E711

    def test_eq_non_ascii_unicode(self, path1):
        path2 = path1.join(u'temp')
        path3 = path1.join(u'ação')
        path4 = path1.join(u'ディレクトリ')

        assert path2 != path3
        assert path2 != path4
        assert path4 != path3

    def test_gt_with_strings(self, path1):
        path2 = path1.join('sampledir')
        path3 = str(path1.join("ttt"))
        assert path3 > path2
        assert path2 < path3
        assert path2 < "ttt"
        assert "ttt" > path2
        path4 = path1.join("aaa")
        l = [path2, path4, path3]
        assert sorted(l) == [path4, path2, path3]

    def test_open_and_ensure(self, path1):
        p = path1.join("sub1", "sub2", "file")
        with p.open("w", ensure=1) as f:
            f.write("hello")
        assert p.read() == "hello"

    def test_write_and_ensure(self, path1):
        p = path1.join("sub1", "sub2", "file")
        p.write("hello", ensure=1)
        assert p.read() == "hello"

    @py.test.mark.parametrize('bin', (False, True))
    def test_dump(self, tmpdir, bin):
        path = tmpdir.join("dumpfile%s" % int(bin))
        try:
            d = {'answer': 42}
            path.dump(d, bin=bin)
            f = path.open('rb+')
            import pickle
            dnew = pickle.load(f)
            assert d == dnew
        finally:
            f.close()

    @failsonjywin32
    def test_setmtime(self):
        import tempfile
        import time
        try:
            fd, name = tempfile.mkstemp()
            os.close(fd)
        except AttributeError:
            name = tempfile.mktemp()
            open(name, 'w').close()
        try:
            mtime = int(time.time())-100
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
        newfile = tmpdir.join('test1', 'test')
        newfile.ensure()
        assert newfile.check(file=1)
        newfile.write("42")
        newfile.ensure()
        s = newfile.read()
        assert s == "42"

    def test_ensure_filepath_withoutdir(self, tmpdir):
        newfile = tmpdir.join('test1file')
        t = newfile.ensure()
        assert t == newfile
        assert newfile.check(file=1)

    def test_ensure_dirpath(self, tmpdir):
        newfile = tmpdir.join('test1', 'testfile')
        t = newfile.ensure(dir=1)
        assert t == newfile
        assert newfile.check(dir=1)

    def test_ensure_non_ascii_unicode(self, tmpdir):
        newfile = tmpdir.join(u'ação',u'ディレクトリ')
        t = newfile.ensure(dir=1)
        assert t == newfile
        assert newfile.check(dir=1)

    def test_init_from_path(self, tmpdir):
        l = local()
        l2 = local(l)
        assert l2 == l

        wc = py.path.svnwc('.')
        l3 = local(wc)
        assert l3 is not wc
        assert l3.strpath == wc.strpath
        assert not hasattr(l3, 'commit')

    @py.test.mark.xfail(run=False, reason="unreliable est for long filenames")
    def test_long_filenames(self, tmpdir):
        if sys.platform == "win32":
            py.test.skip("win32: work around needed for path length limit")
        # see http://codespeak.net/pipermail/py-dev/2008q2/000922.html

        # testing paths > 260 chars (which is Windows' limitation, but
        # depending on how the paths are used), but > 4096 (which is the
        # Linux' limitation) - the behaviour of paths with names > 4096 chars
        # is undetermined
        newfilename = '/test' * 60
        l = tmpdir.join(newfilename)
        l.ensure(file=True)
        l.write('foo')
        l2 = tmpdir.join(newfilename)
        assert l2.read() == 'foo'

    def test_visit_depth_first(self, tmpdir):
        tmpdir.ensure("a", "1")
        tmpdir.ensure("b", "2")
        p3 = tmpdir.ensure("breadth")
        l = list(tmpdir.visit(lambda x: x.check(file=1)))
        assert len(l) == 3
        # check that breadth comes last
        assert l[2] == p3

    def test_visit_rec_fnmatch(self, tmpdir):
        p1 = tmpdir.ensure("a", "123")
        tmpdir.ensure(".b", "345")
        l = list(tmpdir.visit("???", rec="[!.]*"))
        assert len(l) == 1
        # check that breadth comes last
        assert l[0] == p1

    def test_fnmatch_file_abspath(self, tmpdir):
        b = tmpdir.join("a", "b")
        assert b.fnmatch(os.sep.join("ab"))
        pattern = os.sep.join([str(tmpdir), "*", "b"])
        assert b.fnmatch(pattern)

    def test_sysfind(self):
        name = sys.platform == "win32" and "cmd" or "test"
        x = py.path.local.sysfind(name)
        assert x.check(file=1)
        assert py.path.local.sysfind('jaksdkasldqwe') is None
        assert py.path.local.sysfind(name, paths=[]) is None
        x2 = py.path.local.sysfind(name, paths=[x.dirpath()])
        assert x2 == x

    def test_fspath_protocol_other_class(self, fake_fspath_obj):
        # py.path is always absolute
        py_path = py.path.local(fake_fspath_obj)
        str_path = fake_fspath_obj.__fspath__()
        assert py_path.check(endswith=str_path)
        assert py_path.join(fake_fspath_obj).strpath == os.path.join(
                py_path.strpath, str_path)

    def test_make_numbered_dir_multiprocess_safe(self, tmpdir):
        # https://github.com/pytest-dev/py/issues/30
        pool = multiprocessing.Pool()
        results = [pool.apply_async(batch_make_numbered_dirs, [tmpdir, 100]) for _ in range(20)]
        for r in results:
            assert r.get()


class TestExecutionOnWindows:
    pytestmark = win32only

    def test_sysfind_bat_exe_before(self, tmpdir, monkeypatch):
        monkeypatch.setenv("PATH", str(tmpdir), prepend=os.pathsep)
        tmpdir.ensure("hello")
        h = tmpdir.ensure("hello.bat")
        x = py.path.local.sysfind("hello")
        assert x == h


class TestExecution:
    pytestmark = skiponwin32

    def test_sysfind_no_permisson_ignored(self, monkeypatch, tmpdir):
        noperm = tmpdir.ensure('noperm', dir=True)
        monkeypatch.setenv("PATH", noperm, prepend=":")
        noperm.chmod(0)
        assert py.path.local.sysfind('jaksdkasldqwe') is None

    def test_sysfind_absolute(self):
        x = py.path.local.sysfind('test')
        assert x.check(file=1)
        y = py.path.local.sysfind(str(x))
        assert y.check(file=1)
        assert y == x

    def test_sysfind_multiple(self, tmpdir, monkeypatch):
        monkeypatch.setenv('PATH', "%s:%s" % (
                            tmpdir.ensure('a'),
                            tmpdir.join('b')),
                           prepend=":")
        tmpdir.ensure('b', 'a')
        x = py.path.local.sysfind(
            'a', checker=lambda x: x.dirpath().basename == 'b')
        assert x.basename == 'a'
        assert x.dirpath().basename == 'b'
        assert py.path.local.sysfind('a', checker=lambda x: None) is None

    def test_sysexec(self):
        x = py.path.local.sysfind('ls')
        out = x.sysexec('-a')
        for x in py.path.local().listdir():
            assert out.find(x.basename) != -1

    def test_sysexec_failing(self):
        x = py.path.local.sysfind('false')
        with pytest.raises(py.process.cmdexec.Error):
            x.sysexec('aksjdkasjd')

    def test_make_numbered_dir(self, tmpdir):
        tmpdir.ensure('base.not_an_int', dir=1)
        for i in range(10):
            numdir = local.make_numbered_dir(prefix='base.', rootdir=tmpdir,
                                             keep=2, lock_timeout=0)
            assert numdir.check()
            assert numdir.basename == 'base.%d' % i
            if i >= 1:
                assert numdir.new(ext=str(i-1)).check()
            if i >= 2:
                assert numdir.new(ext=str(i-2)).check()
            if i >= 3:
                assert not numdir.new(ext=str(i-3)).check()

    def test_make_numbered_dir_case(self, tmpdir):
        """make_numbered_dir does not make assumptions on the underlying
        filesystem based on the platform and will assume it _could_ be case
        insensitive.

        See issues:
        - https://github.com/pytest-dev/pytest/issues/708
        - https://github.com/pytest-dev/pytest/issues/3451
        """
        d1 = local.make_numbered_dir(
            prefix='CAse.', rootdir=tmpdir, keep=2, lock_timeout=0,
        )
        d2 = local.make_numbered_dir(
            prefix='caSE.', rootdir=tmpdir, keep=2, lock_timeout=0,
        )
        assert str(d1).lower() != str(d2).lower()
        assert str(d2).endswith('.1')

    def test_make_numbered_dir_NotImplemented_Error(self, tmpdir, monkeypatch):
        def notimpl(x, y):
            raise NotImplementedError(42)
        monkeypatch.setattr(os, 'symlink', notimpl)
        x = tmpdir.make_numbered_dir(rootdir=tmpdir, lock_timeout=0)
        assert x.relto(tmpdir)
        assert x.check()

    def test_locked_make_numbered_dir(self, tmpdir):
        for i in range(10):
            numdir = local.make_numbered_dir(prefix='base2.', rootdir=tmpdir,
                                             keep=2)
            assert numdir.check()
            assert numdir.basename == 'base2.%d' % i
            for j in range(i):
                assert numdir.new(ext=str(j)).check()

    def test_error_preservation(self, path1):
        py.test.raises(EnvironmentError, path1.join('qwoeqiwe').mtime)
        py.test.raises(EnvironmentError, path1.join('qwoeqiwe').read)

    # def test_parentdirmatch(self):
    #    local.parentdirmatch('std', startmodule=__name__)
    #


class TestImport:
    def test_pyimport(self, path1):
        obj = path1.join('execfile.py').pyimport()
        assert obj.x == 42
        assert obj.__name__ == 'execfile'

    def test_pyimport_renamed_dir_creates_mismatch(self, tmpdir, monkeypatch):
        p = tmpdir.ensure("a", "test_x123.py")
        p.pyimport()
        tmpdir.join("a").move(tmpdir.join("b"))
        with pytest.raises(tmpdir.ImportMismatchError):
            tmpdir.join("b", "test_x123.py").pyimport()

        # Errors can be ignored.
        monkeypatch.setenv('PY_IGNORE_IMPORTMISMATCH', '1')
        tmpdir.join("b", "test_x123.py").pyimport()

        # PY_IGNORE_IMPORTMISMATCH=0 does not ignore error.
        monkeypatch.setenv('PY_IGNORE_IMPORTMISMATCH', '0')
        with pytest.raises(tmpdir.ImportMismatchError):
            tmpdir.join("b", "test_x123.py").pyimport()

    def test_pyimport_messy_name(self, tmpdir):
        # http://bitbucket.org/hpk42/py-trunk/issue/129
        path = tmpdir.ensure('foo__init__.py')
        path.pyimport()

    def test_pyimport_dir(self, tmpdir):
        p = tmpdir.join("hello_123")
        p_init = p.ensure("__init__.py")
        m = p.pyimport()
        assert m.__name__ == "hello_123"
        m = p_init.pyimport()
        assert m.__name__ == "hello_123"

    def test_pyimport_execfile_different_name(self, path1):
        obj = path1.join('execfile.py').pyimport(modname="0x.y.z")
        assert obj.x == 42
        assert obj.__name__ == '0x.y.z'

    def test_pyimport_a(self, path1):
        otherdir = path1.join('otherdir')
        mod = otherdir.join('a.py').pyimport()
        assert mod.result == "got it"
        assert mod.__name__ == 'otherdir.a'

    def test_pyimport_b(self, path1):
        otherdir = path1.join('otherdir')
        mod = otherdir.join('b.py').pyimport()
        assert mod.stuff == "got it"
        assert mod.__name__ == 'otherdir.b'

    def test_pyimport_c(self, path1):
        otherdir = path1.join('otherdir')
        mod = otherdir.join('c.py').pyimport()
        assert mod.value == "got it"

    def test_pyimport_d(self, path1):
        otherdir = path1.join('otherdir')
        mod = otherdir.join('d.py').pyimport()
        assert mod.value2 == "got it"

    def test_pyimport_and_import(self, tmpdir):
        tmpdir.ensure('xxxpackage', '__init__.py')
        mod1path = tmpdir.ensure('xxxpackage', 'module1.py')
        mod1 = mod1path.pyimport()
        assert mod1.__name__ == 'xxxpackage.module1'
        from xxxpackage import module1
        assert module1 is mod1

    def test_pyimport_check_filepath_consistency(self, monkeypatch, tmpdir):
        name = 'pointsback123'
        ModuleType = type(os)
        p = tmpdir.ensure(name + '.py')
        for ending in ('.pyc', '$py.class', '.pyo'):
            mod = ModuleType(name)
            pseudopath = tmpdir.ensure(name+ending)
            mod.__file__ = str(pseudopath)
            monkeypatch.setitem(sys.modules, name, mod)
            newmod = p.pyimport()
            assert mod == newmod
        monkeypatch.undo()
        mod = ModuleType(name)
        pseudopath = tmpdir.ensure(name+"123.py")
        mod.__file__ = str(pseudopath)
        monkeypatch.setitem(sys.modules, name, mod)
        excinfo = py.test.raises(pseudopath.ImportMismatchError, p.pyimport)
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
    pytestmark = py.test.mark.skipif("sys.version_info < (3, 5)")

    OPTS = {'ensuresyspath': 'importlib'}

    def test_pyimport(self, path1):
        obj = path1.join('execfile.py').pyimport(**self.OPTS)
        assert obj.x == 42
        assert obj.__name__ == 'execfile'

    def test_pyimport_dir_fails(self, tmpdir):
        p = tmpdir.join("hello_123")
        p.ensure("__init__.py")
        with pytest.raises(ImportError):
            p.pyimport(**self.OPTS)

    def test_pyimport_execfile_different_name(self, path1):
        obj = path1.join('execfile.py').pyimport(modname="0x.y.z", **self.OPTS)
        assert obj.x == 42
        assert obj.__name__ == '0x.y.z'

    def test_pyimport_relative_import_fails(self, path1):
        otherdir = path1.join('otherdir')
        with pytest.raises(ImportError):
            otherdir.join('a.py').pyimport(**self.OPTS)

    def test_pyimport_doesnt_use_sys_modules(self, tmpdir):
        p = tmpdir.ensure('file738jsk.py')
        mod = p.pyimport(**self.OPTS)
        assert mod.__name__ == 'file738jsk'
        assert 'file738jsk' not in sys.modules


def test_pypkgdir(tmpdir):
    pkg = tmpdir.ensure('pkg1', dir=1)
    pkg.ensure("__init__.py")
    pkg.ensure("subdir/__init__.py")
    assert pkg.pypkgpath() == pkg
    assert pkg.join('subdir', '__init__.py').pypkgpath() == pkg


def test_pypkgdir_unimportable(tmpdir):
    pkg = tmpdir.ensure('pkg1-1', dir=1)  # unimportable
    pkg.ensure("__init__.py")
    subdir = pkg.ensure("subdir/__init__.py").dirpath()
    assert subdir.pypkgpath() == subdir
    assert subdir.ensure("xyz.py").pypkgpath() == subdir
    assert not pkg.pypkgpath()


def test_isimportable():
    from py._path.local import isimportable
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
    assert py.path.local._gethomedir() == py.path.local(path)


def test_homedir_not_exists(monkeypatch):
    monkeypatch.delenv("HOME", raising=False)
    monkeypatch.delenv("HOMEDRIVE", raising=False)
    homedir = py.path.local._gethomedir()
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
        py.test.raises(NotImplementedError, "path1.stat().owner")
        py.test.raises(NotImplementedError, "path1.stat().group")

    def test_chmod_simple_int(self, path1):
        py.builtin.print_("path1 is", path1)
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
        t1 = path1.join('a_path')
        assert t1 == str(path1) + '\\a_path'
        t1 = path1.join('a_path/')
        assert t1 == str(path1) + '\\a_path'
        t1 = path1.join('dir/a_path')
        assert t1 == str(path1) + '\\dir\\a_path'

    def test_sysfind_in_currentdir(self, path1):
        cmd = py.path.local.sysfind('cmd')
        root = cmd.new(dirname='', basename='')  # c:\ in most installations
        with root.as_cwd():
            x = py.path.local.sysfind(cmd.relto(root))
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
        linkpath = tmpdir.join('test')
        filepath = tmpdir.join('file')
        filepath.write("Hello")
        nlink = filepath.stat().nlink
        linkpath.mklinkto(filepath)
        assert filepath.stat().nlink == nlink + 1

    def test_symlink_are_identical(self, tmpdir):
        filepath = tmpdir.join('file')
        filepath.write("Hello")
        linkpath = tmpdir.join('test')
        linkpath.mksymlinkto(filepath)
        assert linkpath.readlink() == str(filepath)

    def test_symlink_isfile(self, tmpdir):
        linkpath = tmpdir.join('test')
        filepath = tmpdir.join('file')
        filepath.write("")
        linkpath.mksymlinkto(filepath)
        assert linkpath.check(file=1)
        assert not linkpath.check(link=0, file=1)
        assert linkpath.islink()

    def test_symlink_relative(self, tmpdir):
        linkpath = tmpdir.join('test')
        filepath = tmpdir.join('file')
        filepath.write("Hello")
        linkpath.mksymlinkto(filepath, absolute=False)
        assert linkpath.readlink() == "file"
        assert filepath.read() == linkpath.read()

    def test_symlink_not_existing(self, tmpdir):
        linkpath = tmpdir.join('testnotexisting')
        assert not linkpath.check(link=1)
        assert linkpath.check(link=0)

    def test_relto_with_root(self, path1, tmpdir):
        y = path1.join('x').relto(py.path.local('/'))
        assert y[0] == str(path1)[1]

    def test_visit_recursive_symlink(self, tmpdir):
        linkpath = tmpdir.join('test')
        linkpath.mksymlinkto(tmpdir)
        visitor = tmpdir.visit(None, lambda x: x.check(link=0))
        assert list(visitor) == [linkpath]

    def test_symlink_isdir(self, tmpdir):
        linkpath = tmpdir.join('test')
        linkpath.mksymlinkto(tmpdir)
        assert linkpath.check(dir=1)
        assert not linkpath.check(link=0, dir=1)

    def test_symlink_remove(self, tmpdir):
        linkpath = tmpdir.join('test')
        linkpath.mksymlinkto(linkpath)  # point to itself
        assert linkpath.check(link=1)
        linkpath.remove()
        assert not linkpath.check()

    def test_realpath_file(self, tmpdir):
        linkpath = tmpdir.join('test')
        filepath = tmpdir.join('file')
        filepath.write("")
        linkpath.mksymlinkto(filepath)
        realpath = linkpath.realpath()
        assert realpath.basename == 'file'

    def test_owner(self, path1, tmpdir):
        from pwd import getpwuid
        from grp import getgrgid
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
        pytest.raises(py.error.ENOENT, lambda: path1.stat())
        res = path1.stat(raising=False)
        assert res is None

    def test_atime(self, tmpdir):
        import time
        path = tmpdir.ensure('samplefile')
        now = time.time()
        atime1 = path.atime()
        # we could wait here but timer resolution is very
        # system dependent
        path.read()
        time.sleep(ATIME_RESOLUTION)
        atime2 = path.atime()
        time.sleep(ATIME_RESOLUTION)
        duration = time.time() - now
        assert (atime2-atime1) <= duration

    def test_commondir(self, path1):
        # XXX This is here in local until we find a way to implement this
        #     using the subversion command line api.
        p1 = path1.join('something')
        p2 = path1.join('otherthing')
        assert p1.common(p2) == path1
        assert p2.common(p1) == path1

    def test_commondir_nocommon(self, path1):
        # XXX This is here in local until we find a way to implement this
        #     using the subversion command line api.
        p1 = path1.join('something')
        p2 = py.path.local(path1.sep+'blabla')
        assert p1.common(p2) == '/'

    def test_join_to_root(self, path1):
        root = path1.parts()[0]
        assert len(str(root)) == 1
        assert str(root.join('a')) == '/a'

    def test_join_root_to_root_with_no_abs(self, path1):
        nroot = path1.join('/')
        assert str(path1) == str(nroot)
        assert path1 == nroot

    def test_chmod_simple_int(self, path1):
        mode = path1.stat().mode
        path1.chmod(int(mode/2))
        try:
            assert path1.stat().mode != mode
        finally:
            path1.chmod(mode)
            assert path1.stat().mode == mode

    def test_chmod_rec_int(self, path1):
        # XXX fragile test
        def recfilter(x): return x.check(dotfile=0, link=0)
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
        unicode_fn = u"something-\342\200\223.txt"
        f = tmpdir.ensure("a", unicode_fn)
        a = f.dirpath()
        oldmode = f.stat().mode
        newmode = oldmode ^ 1
        f.chmod(newmode)
        b = tmpdir.join("b")
        a.copy(b, mode=True)
        assert b.join(f.basename).stat().mode == newmode

    def test_copy_stat_file(self, tmpdir):
        src = tmpdir.ensure('src')
        dst = tmpdir.join('dst')
        # a small delay before the copy
        time.sleep(ATIME_RESOLUTION)
        src.copy(dst, stat=True)
        oldstat = src.stat()
        newstat = dst.stat()
        assert oldstat.mode == newstat.mode
        assert (dst.atime() - src.atime()) < ATIME_RESOLUTION
        assert (dst.mtime() - src.mtime()) < ATIME_RESOLUTION

    def test_copy_stat_dir(self, tmpdir):
        test_files = ['a', 'b', 'c']
        src = tmpdir.join('src')
        for f in test_files:
            src.join(f).write(f, ensure=True)
        dst = tmpdir.join('dst')
        # a small delay before the copy
        time.sleep(ATIME_RESOLUTION)
        src.copy(dst, stat=True)
        for f in test_files:
            oldstat = src.join(f).stat()
            newstat = dst.join(f).stat()
            assert (newstat.atime - oldstat.atime) < ATIME_RESOLUTION
            assert (newstat.mtime - oldstat.mtime) < ATIME_RESOLUTION
            assert oldstat.mode == newstat.mode

    @failsonjython
    def test_chown_identity(self, path1):
        owner = path1.stat().owner
        group = path1.stat().group
        path1.chown(owner, group)

    @failsonjython
    def test_chown_dangling_link(self, path1):
        owner = path1.stat().owner
        group = path1.stat().group
        x = path1.join('hello')
        x.mksymlinkto('qlwkejqwlek')
        try:
            path1.chown(owner, group, rec=1)
        finally:
            x.remove(rec=0)

    @failsonjython
    def test_chown_identity_rec_mayfail(self, path1):
        owner = path1.stat().owner
        group = path1.stat().group
        path1.chown(owner, group)


class TestUnicodePy2Py3:
    def test_join_ensure(self, tmpdir, monkeypatch):
        if sys.version_info >= (3, 0) and "LANG" not in os.environ:
            pytest.skip("cannot run test without locale")
        x = py.path.local(tmpdir.strpath)
        part = "hällo"
        y = x.ensure(part)
        assert x.join(part) == y

    def test_listdir(self, tmpdir):
        if sys.version_info >= (3, 0) and "LANG" not in os.environ:
            pytest.skip("cannot run test without locale")
        x = py.path.local(tmpdir.strpath)
        part = "hällo"
        y = x.ensure(part)
        assert x.listdir(part)[0] == y

    @pytest.mark.xfail(
        reason="changing read/write might break existing usages")
    def test_read_write(self, tmpdir):
        x = tmpdir.join("hello")
        part = py.builtin._totext("hällo", "utf8")
        x.write(part)
        assert x.read() == part
        x.write(part.encode(sys.getdefaultencoding()))
        assert x.read() == part.encode(sys.getdefaultencoding())


class TestBinaryAndTextMethods:
    def test_read_binwrite(self, tmpdir):
        x = tmpdir.join("hello")
        part = py.builtin._totext("hällo", "utf8")
        part_utf8 = part.encode("utf8")
        x.write_binary(part_utf8)
        assert x.read_binary() == part_utf8
        s = x.read_text(encoding="utf8")
        assert s == part
        assert py.builtin._istext(s)

    def test_read_textwrite(self, tmpdir):
        x = tmpdir.join("hello")
        part = py.builtin._totext("hällo", "utf8")
        part_utf8 = part.encode("utf8")
        x.write_text(part, encoding="utf8")
        assert x.read_binary() == part_utf8
        assert x.read_text(encoding="utf8") == part

    def test_default_encoding(self, tmpdir):
        x = tmpdir.join("hello")
        # Can't use UTF8 as the default encoding (ASCII) doesn't support it
        part = py.builtin._totext("hello", "ascii")
        x.write_text(part, "ascii")
        s = x.read_text("ascii")
        assert s == part
        assert type(s) == type(part)

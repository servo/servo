# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import errno
import os
import stat
import sys

import attr
import six

import pytest
from _pytest import pathlib
from _pytest.pathlib import Path
from _pytest.warnings import SHOW_PYTEST_WARNINGS_ARG


def test_tmpdir_fixture(testdir):
    p = testdir.copy_example("tmpdir/tmpdir_fixture.py")
    results = testdir.runpytest(p)
    results.stdout.fnmatch_lines(["*1 passed*"])


def test_ensuretemp(recwarn):
    d1 = pytest.ensuretemp("hello")
    d2 = pytest.ensuretemp("hello")
    assert d1 == d2
    assert d1.check(dir=1)


@attr.s
class FakeConfig(object):
    basetemp = attr.ib()
    trace = attr.ib(default=None)

    @property
    def trace(self):
        return self

    def get(self, key):
        return lambda *k: None

    @property
    def option(self):
        return self


class TestTempdirHandler(object):
    def test_mktemp(self, tmp_path):

        from _pytest.tmpdir import TempdirFactory, TempPathFactory

        config = FakeConfig(tmp_path)
        t = TempdirFactory(TempPathFactory.from_config(config))
        tmp = t.mktemp("world")
        assert tmp.relto(t.getbasetemp()) == "world0"
        tmp = t.mktemp("this")
        assert tmp.relto(t.getbasetemp()).startswith("this")
        tmp2 = t.mktemp("this")
        assert tmp2.relto(t.getbasetemp()).startswith("this")
        assert tmp2 != tmp

    def test_tmppath_relative_basetemp_absolute(self, tmp_path, monkeypatch):
        """#4425"""
        from _pytest.tmpdir import TempPathFactory

        monkeypatch.chdir(tmp_path)
        config = FakeConfig("hello")
        t = TempPathFactory.from_config(config)
        assert t.getbasetemp().resolve() == (tmp_path / "hello").resolve()


class TestConfigTmpdir(object):
    def test_getbasetemp_custom_removes_old(self, testdir):
        mytemp = testdir.tmpdir.join("xyz")
        p = testdir.makepyfile(
            """
            def test_1(tmpdir):
                pass
        """
        )
        testdir.runpytest(p, "--basetemp=%s" % mytemp)
        mytemp.check()
        mytemp.ensure("hello")

        testdir.runpytest(p, "--basetemp=%s" % mytemp)
        mytemp.check()
        assert not mytemp.join("hello").check()


def test_basetemp(testdir):
    mytemp = testdir.tmpdir.mkdir("mytemp")
    p = testdir.makepyfile(
        """
        import pytest
        def test_1():
            pytest.ensuretemp("hello")
    """
    )
    result = testdir.runpytest(p, "--basetemp=%s" % mytemp, SHOW_PYTEST_WARNINGS_ARG)
    assert result.ret == 0
    assert mytemp.join("hello").check()


def test_tmpdir_always_is_realpath(testdir):
    # the reason why tmpdir should be a realpath is that
    # when you cd to it and do "os.getcwd()" you will anyway
    # get the realpath.  Using the symlinked path can thus
    # easily result in path-inequality
    # XXX if that proves to be a problem, consider using
    # os.environ["PWD"]
    realtemp = testdir.tmpdir.mkdir("myrealtemp")
    linktemp = testdir.tmpdir.join("symlinktemp")
    attempt_symlink_to(linktemp, str(realtemp))
    p = testdir.makepyfile(
        """
        def test_1(tmpdir):
            import os
            assert os.path.realpath(str(tmpdir)) == str(tmpdir)
    """
    )
    result = testdir.runpytest("-s", p, "--basetemp=%s/bt" % linktemp)
    assert not result.ret


def test_tmp_path_always_is_realpath(testdir, monkeypatch):
    # for reasoning see: test_tmpdir_always_is_realpath test-case
    realtemp = testdir.tmpdir.mkdir("myrealtemp")
    linktemp = testdir.tmpdir.join("symlinktemp")
    attempt_symlink_to(linktemp, str(realtemp))
    monkeypatch.setenv("PYTEST_DEBUG_TEMPROOT", str(linktemp))
    testdir.makepyfile(
        """
        def test_1(tmp_path):
            assert tmp_path.resolve() == tmp_path
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmpdir_too_long_on_parametrization(testdir):
    testdir.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize("arg", ["1"*1000])
        def test_some(arg, tmpdir):
            tmpdir.ensure("hello")
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmpdir_factory(testdir):
    testdir.makepyfile(
        """
        import pytest
        @pytest.fixture(scope='session')
        def session_dir(tmpdir_factory):
            return tmpdir_factory.mktemp('data', numbered=False)
        def test_some(session_dir):
            assert session_dir.isdir()
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmpdir_fallback_tox_env(testdir, monkeypatch):
    """Test that tmpdir works even if environment variables required by getpass
    module are missing (#1010).
    """
    monkeypatch.delenv("USER", raising=False)
    monkeypatch.delenv("USERNAME", raising=False)
    testdir.makepyfile(
        """
        import pytest
        def test_some(tmpdir):
            assert tmpdir.isdir()
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.fixture
def break_getuser(monkeypatch):
    monkeypatch.setattr("os.getuid", lambda: -1)
    # taken from python 2.7/3.4
    for envvar in ("LOGNAME", "USER", "LNAME", "USERNAME"):
        monkeypatch.delenv(envvar, raising=False)


@pytest.mark.usefixtures("break_getuser")
@pytest.mark.skipif(sys.platform.startswith("win"), reason="no os.getuid on windows")
def test_tmpdir_fallback_uid_not_found(testdir):
    """Test that tmpdir works even if the current process's user id does not
    correspond to a valid user.
    """

    testdir.makepyfile(
        """
        import pytest
        def test_some(tmpdir):
            assert tmpdir.isdir()
    """
    )
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.mark.usefixtures("break_getuser")
@pytest.mark.skipif(sys.platform.startswith("win"), reason="no os.getuid on windows")
def test_get_user_uid_not_found():
    """Test that get_user() function works even if the current process's
    user id does not correspond to a valid user (e.g. running pytest in a
    Docker container with 'docker run -u'.
    """
    from _pytest.tmpdir import get_user

    assert get_user() is None


@pytest.mark.skipif(not sys.platform.startswith("win"), reason="win only")
def test_get_user(monkeypatch):
    """Test that get_user() function works even if environment variables
    required by getpass module are missing from the environment on Windows
    (#1010).
    """
    from _pytest.tmpdir import get_user

    monkeypatch.delenv("USER", raising=False)
    monkeypatch.delenv("USERNAME", raising=False)
    assert get_user() is None


class TestNumberedDir(object):
    PREFIX = "fun-"

    def test_make(self, tmp_path):
        from _pytest.pathlib import make_numbered_dir

        for i in range(10):
            d = make_numbered_dir(root=tmp_path, prefix=self.PREFIX)
            assert d.name.startswith(self.PREFIX)
            assert d.name.endswith(str(i))

        symlink = tmp_path.joinpath(self.PREFIX + "current")
        if symlink.exists():
            # unix
            assert symlink.is_symlink()
            assert symlink.resolve() == d.resolve()

    def test_cleanup_lock_create(self, tmp_path):
        d = tmp_path.joinpath("test")
        d.mkdir()
        from _pytest.pathlib import create_cleanup_lock

        lockfile = create_cleanup_lock(d)
        with pytest.raises(EnvironmentError, match="cannot create lockfile in .*"):
            create_cleanup_lock(d)

        lockfile.unlink()

    def test_lock_register_cleanup_removal(self, tmp_path):
        from _pytest.pathlib import create_cleanup_lock, register_cleanup_lock_removal

        lock = create_cleanup_lock(tmp_path)

        registry = []
        register_cleanup_lock_removal(lock, register=registry.append)

        (cleanup_func,) = registry

        assert lock.is_file()

        cleanup_func(original_pid="intentionally_different")

        assert lock.is_file()

        cleanup_func()

        assert not lock.exists()

        cleanup_func()

        assert not lock.exists()

    def _do_cleanup(self, tmp_path):
        self.test_make(tmp_path)
        from _pytest.pathlib import cleanup_numbered_dir

        cleanup_numbered_dir(
            root=tmp_path,
            prefix=self.PREFIX,
            keep=2,
            consider_lock_dead_if_created_before=0,
        )

    def test_cleanup_keep(self, tmp_path):
        self._do_cleanup(tmp_path)
        a, b = (x for x in tmp_path.iterdir() if not x.is_symlink())
        print(a, b)

    def test_cleanup_locked(self, tmp_path):

        from _pytest import pathlib

        p = pathlib.make_numbered_dir(root=tmp_path, prefix=self.PREFIX)

        pathlib.create_cleanup_lock(p)

        assert not pathlib.ensure_deletable(
            p, consider_lock_dead_if_created_before=p.stat().st_mtime - 1
        )
        assert pathlib.ensure_deletable(
            p, consider_lock_dead_if_created_before=p.stat().st_mtime + 1
        )

    def test_cleanup_ignores_symlink(self, tmp_path):
        the_symlink = tmp_path / (self.PREFIX + "current")
        attempt_symlink_to(the_symlink, tmp_path / (self.PREFIX + "5"))
        self._do_cleanup(tmp_path)

    def test_removal_accepts_lock(self, tmp_path):
        folder = pathlib.make_numbered_dir(root=tmp_path, prefix=self.PREFIX)
        pathlib.create_cleanup_lock(folder)
        pathlib.maybe_delete_a_numbered_dir(folder)
        assert folder.is_dir()


class TestRmRf:
    def test_rm_rf(self, tmp_path):
        from _pytest.pathlib import rm_rf

        adir = tmp_path / "adir"
        adir.mkdir()
        rm_rf(adir)

        assert not adir.exists()

        adir.mkdir()
        afile = adir / "afile"
        afile.write_bytes(b"aa")

        rm_rf(adir)
        assert not adir.exists()

    def test_rm_rf_with_read_only_file(self, tmp_path):
        """Ensure rm_rf can remove directories with read-only files in them (#5524)"""
        from _pytest.pathlib import rm_rf

        fn = tmp_path / "dir/foo.txt"
        fn.parent.mkdir()

        fn.touch()

        self.chmod_r(fn)

        rm_rf(fn.parent)

        assert not fn.parent.is_dir()

    def chmod_r(self, path):
        mode = os.stat(str(path)).st_mode
        os.chmod(str(path), mode & ~stat.S_IWRITE)

    def test_rm_rf_with_read_only_directory(self, tmp_path):
        """Ensure rm_rf can remove read-only directories (#5524)"""
        from _pytest.pathlib import rm_rf

        adir = tmp_path / "dir"
        adir.mkdir()

        (adir / "foo.txt").touch()
        self.chmod_r(adir)

        rm_rf(adir)

        assert not adir.is_dir()

    def test_on_rm_rf_error(self, tmp_path):
        from _pytest.pathlib import on_rm_rf_error

        adir = tmp_path / "dir"
        adir.mkdir()

        fn = adir / "foo.txt"
        fn.touch()
        self.chmod_r(fn)

        # unknown exception
        with pytest.warns(pytest.PytestWarning):
            exc_info = (None, RuntimeError(), None)
            on_rm_rf_error(os.unlink, str(fn), exc_info, start_path=tmp_path)
            assert fn.is_file()

        # we ignore FileNotFoundError
        file_not_found = OSError()
        file_not_found.errno = errno.ENOENT
        exc_info = (None, file_not_found, None)
        assert not on_rm_rf_error(None, str(fn), exc_info, start_path=tmp_path)

        permission_error = OSError()
        permission_error.errno = errno.EACCES
        # unknown function
        with pytest.warns(pytest.PytestWarning):
            exc_info = (None, permission_error, None)
            on_rm_rf_error(None, str(fn), exc_info, start_path=tmp_path)
            assert fn.is_file()

        exc_info = (None, permission_error, None)
        on_rm_rf_error(os.unlink, str(fn), exc_info, start_path=tmp_path)
        assert not fn.is_file()


def attempt_symlink_to(path, to_path):
    """Try to make a symlink from "path" to "to_path", skipping in case this platform
    does not support it or we don't have sufficient privileges (common on Windows)."""
    if sys.platform.startswith("win") and six.PY2:
        pytest.skip("pathlib for some reason cannot make symlinks on Python 2")
    try:
        Path(path).symlink_to(Path(to_path))
    except OSError:
        pytest.skip("could not create symbolic link")


def test_tmpdir_equals_tmp_path(tmpdir, tmp_path):
    assert Path(tmpdir) == tmp_path


def test_basetemp_with_read_only_files(testdir):
    """Integration test for #5524"""
    testdir.makepyfile(
        """
        import os
        import stat

        def test(tmp_path):
            fn = tmp_path / 'foo.txt'
            fn.write_text(u'hello')
            mode = os.stat(str(fn)).st_mode
            os.chmod(str(fn), mode & ~stat.S_IREAD)
    """
    )
    result = testdir.runpytest("--basetemp=tmp")
    assert result.ret == 0
    # running a second time and ensure we don't crash
    result = testdir.runpytest("--basetemp=tmp")
    assert result.ret == 0

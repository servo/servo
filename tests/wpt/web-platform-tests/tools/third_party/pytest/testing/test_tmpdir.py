import os
import stat
import sys
import warnings
from pathlib import Path
from typing import Callable
from typing import cast
from typing import List

import attr

import pytest
from _pytest import pathlib
from _pytest.config import Config
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pathlib import cleanup_numbered_dir
from _pytest.pathlib import create_cleanup_lock
from _pytest.pathlib import make_numbered_dir
from _pytest.pathlib import maybe_delete_a_numbered_dir
from _pytest.pathlib import on_rm_rf_error
from _pytest.pathlib import register_cleanup_lock_removal
from _pytest.pathlib import rm_rf
from _pytest.pytester import Pytester
from _pytest.tmpdir import get_user
from _pytest.tmpdir import TempPathFactory


def test_tmp_path_fixture(pytester: Pytester) -> None:
    p = pytester.copy_example("tmpdir/tmp_path_fixture.py")
    results = pytester.runpytest(p)
    results.stdout.fnmatch_lines(["*1 passed*"])


@attr.s
class FakeConfig:
    basetemp = attr.ib()

    @property
    def trace(self):
        return self

    def get(self, key):
        return lambda *k: None

    @property
    def option(self):
        return self


class TestTmpPathHandler:
    def test_mktemp(self, tmp_path):
        config = cast(Config, FakeConfig(tmp_path))
        t = TempPathFactory.from_config(config, _ispytest=True)
        tmp = t.mktemp("world")
        assert str(tmp.relative_to(t.getbasetemp())) == "world0"
        tmp = t.mktemp("this")
        assert str(tmp.relative_to(t.getbasetemp())).startswith("this")
        tmp2 = t.mktemp("this")
        assert str(tmp2.relative_to(t.getbasetemp())).startswith("this")
        assert tmp2 != tmp

    def test_tmppath_relative_basetemp_absolute(self, tmp_path, monkeypatch):
        """#4425"""
        monkeypatch.chdir(tmp_path)
        config = cast(Config, FakeConfig("hello"))
        t = TempPathFactory.from_config(config, _ispytest=True)
        assert t.getbasetemp().resolve() == (tmp_path / "hello").resolve()


class TestConfigTmpPath:
    def test_getbasetemp_custom_removes_old(self, pytester: Pytester) -> None:
        mytemp = pytester.path.joinpath("xyz")
        p = pytester.makepyfile(
            """
            def test_1(tmp_path):
                pass
        """
        )
        pytester.runpytest(p, "--basetemp=%s" % mytemp)
        assert mytemp.exists()
        mytemp.joinpath("hello").touch()

        pytester.runpytest(p, "--basetemp=%s" % mytemp)
        assert mytemp.exists()
        assert not mytemp.joinpath("hello").exists()


testdata = [
    ("mypath", True),
    ("/mypath1", False),
    ("./mypath1", True),
    ("../mypath3", False),
    ("../../mypath4", False),
    ("mypath5/..", False),
    ("mypath6/../mypath6", True),
    ("mypath7/../mypath7/..", False),
]


@pytest.mark.parametrize("basename, is_ok", testdata)
def test_mktemp(pytester: Pytester, basename: str, is_ok: bool) -> None:
    mytemp = pytester.mkdir("mytemp")
    p = pytester.makepyfile(
        """
        def test_abs_path(tmp_path_factory):
            tmp_path_factory.mktemp('{}', numbered=False)
        """.format(
            basename
        )
    )

    result = pytester.runpytest(p, "--basetemp=%s" % mytemp)
    if is_ok:
        assert result.ret == 0
        assert mytemp.joinpath(basename).exists()
    else:
        assert result.ret == 1
        result.stdout.fnmatch_lines("*ValueError*")


def test_tmp_path_always_is_realpath(pytester: Pytester, monkeypatch) -> None:
    # the reason why tmp_path should be a realpath is that
    # when you cd to it and do "os.getcwd()" you will anyway
    # get the realpath.  Using the symlinked path can thus
    # easily result in path-inequality
    # XXX if that proves to be a problem, consider using
    # os.environ["PWD"]
    realtemp = pytester.mkdir("myrealtemp")
    linktemp = pytester.path.joinpath("symlinktemp")
    attempt_symlink_to(linktemp, str(realtemp))
    monkeypatch.setenv("PYTEST_DEBUG_TEMPROOT", str(linktemp))
    pytester.makepyfile(
        """
        def test_1(tmp_path):
            assert tmp_path.resolve() == tmp_path
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmp_path_too_long_on_parametrization(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize("arg", ["1"*1000])
        def test_some(arg, tmp_path):
            tmp_path.joinpath("hello").touch()
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmp_path_factory(pytester: Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest
        @pytest.fixture(scope='session')
        def session_dir(tmp_path_factory):
            return tmp_path_factory.mktemp('data', numbered=False)
        def test_some(session_dir):
            assert session_dir.is_dir()
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmp_path_fallback_tox_env(pytester: Pytester, monkeypatch) -> None:
    """Test that tmp_path works even if environment variables required by getpass
    module are missing (#1010).
    """
    monkeypatch.delenv("USER", raising=False)
    monkeypatch.delenv("USERNAME", raising=False)
    pytester.makepyfile(
        """
        def test_some(tmp_path):
            assert tmp_path.is_dir()
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.fixture
def break_getuser(monkeypatch):
    monkeypatch.setattr("os.getuid", lambda: -1)
    # taken from python 2.7/3.4
    for envvar in ("LOGNAME", "USER", "LNAME", "USERNAME"):
        monkeypatch.delenv(envvar, raising=False)


@pytest.mark.usefixtures("break_getuser")
@pytest.mark.skipif(sys.platform.startswith("win"), reason="no os.getuid on windows")
def test_tmp_path_fallback_uid_not_found(pytester: Pytester) -> None:
    """Test that tmp_path works even if the current process's user id does not
    correspond to a valid user.
    """

    pytester.makepyfile(
        """
        def test_some(tmp_path):
            assert tmp_path.is_dir()
    """
    )
    reprec = pytester.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.mark.usefixtures("break_getuser")
@pytest.mark.skipif(sys.platform.startswith("win"), reason="no os.getuid on windows")
def test_get_user_uid_not_found():
    """Test that get_user() function works even if the current process's
    user id does not correspond to a valid user (e.g. running pytest in a
    Docker container with 'docker run -u'.
    """
    assert get_user() is None


@pytest.mark.skipif(not sys.platform.startswith("win"), reason="win only")
def test_get_user(monkeypatch):
    """Test that get_user() function works even if environment variables
    required by getpass module are missing from the environment on Windows
    (#1010).
    """
    monkeypatch.delenv("USER", raising=False)
    monkeypatch.delenv("USERNAME", raising=False)
    assert get_user() is None


class TestNumberedDir:
    PREFIX = "fun-"

    def test_make(self, tmp_path):
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
        lockfile = create_cleanup_lock(d)
        with pytest.raises(OSError, match="cannot create lockfile in .*"):
            create_cleanup_lock(d)

        lockfile.unlink()

    def test_lock_register_cleanup_removal(self, tmp_path: Path) -> None:
        lock = create_cleanup_lock(tmp_path)

        registry: List[Callable[..., None]] = []
        register_cleanup_lock_removal(lock, register=registry.append)

        (cleanup_func,) = registry

        assert lock.is_file()

        cleanup_func(original_pid="intentionally_different")

        assert lock.is_file()

        cleanup_func()

        assert not lock.exists()

        cleanup_func()

        assert not lock.exists()

    def _do_cleanup(self, tmp_path: Path) -> None:
        self.test_make(tmp_path)
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
        p = make_numbered_dir(root=tmp_path, prefix=self.PREFIX)

        create_cleanup_lock(p)

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
        folder = make_numbered_dir(root=tmp_path, prefix=self.PREFIX)
        create_cleanup_lock(folder)
        maybe_delete_a_numbered_dir(folder)
        assert folder.is_dir()


class TestRmRf:
    def test_rm_rf(self, tmp_path):
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
        adir = tmp_path / "dir"
        adir.mkdir()

        (adir / "foo.txt").touch()
        self.chmod_r(adir)

        rm_rf(adir)

        assert not adir.is_dir()

    def test_on_rm_rf_error(self, tmp_path: Path) -> None:
        adir = tmp_path / "dir"
        adir.mkdir()

        fn = adir / "foo.txt"
        fn.touch()
        self.chmod_r(fn)

        # unknown exception
        with pytest.warns(pytest.PytestWarning):
            exc_info1 = (None, RuntimeError(), None)
            on_rm_rf_error(os.unlink, str(fn), exc_info1, start_path=tmp_path)
            assert fn.is_file()

        # we ignore FileNotFoundError
        exc_info2 = (None, FileNotFoundError(), None)
        assert not on_rm_rf_error(None, str(fn), exc_info2, start_path=tmp_path)

        # unknown function
        with pytest.warns(
            pytest.PytestWarning,
            match=r"^\(rm_rf\) unknown function None when removing .*foo.txt:\nNone: ",
        ):
            exc_info3 = (None, PermissionError(), None)
            on_rm_rf_error(None, str(fn), exc_info3, start_path=tmp_path)
            assert fn.is_file()

        # ignored function
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")
            with pytest.warns(None) as warninfo:  # type: ignore[call-overload]
                exc_info4 = (None, PermissionError(), None)
                on_rm_rf_error(os.open, str(fn), exc_info4, start_path=tmp_path)
                assert fn.is_file()
            assert not [x.message for x in warninfo]

        exc_info5 = (None, PermissionError(), None)
        on_rm_rf_error(os.unlink, str(fn), exc_info5, start_path=tmp_path)
        assert not fn.is_file()


def attempt_symlink_to(path, to_path):
    """Try to make a symlink from "path" to "to_path", skipping in case this platform
    does not support it or we don't have sufficient privileges (common on Windows)."""
    try:
        Path(path).symlink_to(Path(to_path))
    except OSError:
        pytest.skip("could not create symbolic link")


def test_basetemp_with_read_only_files(pytester: Pytester) -> None:
    """Integration test for #5524"""
    pytester.makepyfile(
        """
        import os
        import stat

        def test(tmp_path):
            fn = tmp_path / 'foo.txt'
            fn.write_text('hello')
            mode = os.stat(str(fn)).st_mode
            os.chmod(str(fn), mode & ~stat.S_IREAD)
    """
    )
    result = pytester.runpytest("--basetemp=tmp")
    assert result.ret == 0
    # running a second time and ensure we don't crash
    result = pytester.runpytest("--basetemp=tmp")
    assert result.ret == 0


def test_tmp_path_factory_handles_invalid_dir_characters(
    tmp_path_factory: TempPathFactory, monkeypatch: MonkeyPatch
) -> None:
    monkeypatch.setattr("getpass.getuser", lambda: "os/<:*?;>agnostic")
    # _basetemp / _given_basetemp are cached / set in parallel runs, patch them
    monkeypatch.setattr(tmp_path_factory, "_basetemp", None)
    monkeypatch.setattr(tmp_path_factory, "_given_basetemp", None)
    p = tmp_path_factory.getbasetemp()
    assert "pytest-of-unknown" in str(p)


@pytest.mark.skipif(not hasattr(os, "getuid"), reason="checks unix permissions")
def test_tmp_path_factory_create_directory_with_safe_permissions(
    tmp_path: Path, monkeypatch: MonkeyPatch
) -> None:
    """Verify that pytest creates directories under /tmp with private permissions."""
    # Use the test's tmp_path as the system temproot (/tmp).
    monkeypatch.setenv("PYTEST_DEBUG_TEMPROOT", str(tmp_path))
    tmp_factory = TempPathFactory(None, lambda *args: None, _ispytest=True)
    basetemp = tmp_factory.getbasetemp()

    # No world-readable permissions.
    assert (basetemp.stat().st_mode & 0o077) == 0
    # Parent too (pytest-of-foo).
    assert (basetemp.parent.stat().st_mode & 0o077) == 0


@pytest.mark.skipif(not hasattr(os, "getuid"), reason="checks unix permissions")
def test_tmp_path_factory_fixes_up_world_readable_permissions(
    tmp_path: Path, monkeypatch: MonkeyPatch
) -> None:
    """Verify that if a /tmp/pytest-of-foo directory already exists with
    world-readable permissions, it is fixed.

    pytest used to mkdir with such permissions, that's why we fix it up.
    """
    # Use the test's tmp_path as the system temproot (/tmp).
    monkeypatch.setenv("PYTEST_DEBUG_TEMPROOT", str(tmp_path))
    tmp_factory = TempPathFactory(None, lambda *args: None, _ispytest=True)
    basetemp = tmp_factory.getbasetemp()

    # Before - simulate bad perms.
    os.chmod(basetemp.parent, 0o777)
    assert (basetemp.parent.stat().st_mode & 0o077) != 0

    tmp_factory = TempPathFactory(None, lambda *args: None, _ispytest=True)
    basetemp = tmp_factory.getbasetemp()

    # After - fixed.
    assert (basetemp.parent.stat().st_mode & 0o077) == 0

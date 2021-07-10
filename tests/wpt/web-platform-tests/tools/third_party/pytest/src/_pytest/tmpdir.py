"""Support for providing temporary directories to test functions."""
import os
import re
import tempfile
from typing import Optional

import attr
import py

import pytest
from .pathlib import ensure_reset_dir
from .pathlib import LOCK_TIMEOUT
from .pathlib import make_numbered_dir
from .pathlib import make_numbered_dir_with_cleanup
from .pathlib import Path
from _pytest.compat import final
from _pytest.config import Config
from _pytest.fixtures import FixtureRequest
from _pytest.monkeypatch import MonkeyPatch


@final
@attr.s
class TempPathFactory:
    """Factory for temporary directories under the common base temp directory.

    The base directory can be configured using the ``--basetemp`` option.
    """

    _given_basetemp = attr.ib(
        type=Optional[Path],
        # Use os.path.abspath() to get absolute path instead of resolve() as it
        # does not work the same in all platforms (see #4427).
        # Path.absolute() exists, but it is not public (see https://bugs.python.org/issue25012).
        # Ignore type because of https://github.com/python/mypy/issues/6172.
        converter=attr.converters.optional(
            lambda p: Path(os.path.abspath(str(p)))  # type: ignore
        ),
    )
    _trace = attr.ib()
    _basetemp = attr.ib(type=Optional[Path], default=None)

    @classmethod
    def from_config(cls, config: Config) -> "TempPathFactory":
        """Create a factory according to pytest configuration."""
        return cls(
            given_basetemp=config.option.basetemp, trace=config.trace.get("tmpdir")
        )

    def _ensure_relative_to_basetemp(self, basename: str) -> str:
        basename = os.path.normpath(basename)
        if (self.getbasetemp() / basename).resolve().parent != self.getbasetemp():
            raise ValueError(
                "{} is not a normalized and relative path".format(basename)
            )
        return basename

    def mktemp(self, basename: str, numbered: bool = True) -> Path:
        """Create a new temporary directory managed by the factory.

        :param basename:
            Directory base name, must be a relative path.

        :param numbered:
            If ``True``, ensure the directory is unique by adding a numbered
            suffix greater than any existing one: ``basename="foo-"`` and ``numbered=True``
            means that this function will create directories named ``"foo-0"``,
            ``"foo-1"``, ``"foo-2"`` and so on.

        :returns:
            The path to the new directory.
        """
        basename = self._ensure_relative_to_basetemp(basename)
        if not numbered:
            p = self.getbasetemp().joinpath(basename)
            p.mkdir()
        else:
            p = make_numbered_dir(root=self.getbasetemp(), prefix=basename)
            self._trace("mktemp", p)
        return p

    def getbasetemp(self) -> Path:
        """Return base temporary directory."""
        if self._basetemp is not None:
            return self._basetemp

        if self._given_basetemp is not None:
            basetemp = self._given_basetemp
            ensure_reset_dir(basetemp)
            basetemp = basetemp.resolve()
        else:
            from_env = os.environ.get("PYTEST_DEBUG_TEMPROOT")
            temproot = Path(from_env or tempfile.gettempdir()).resolve()
            user = get_user() or "unknown"
            # use a sub-directory in the temproot to speed-up
            # make_numbered_dir() call
            rootdir = temproot.joinpath("pytest-of-{}".format(user))
            rootdir.mkdir(exist_ok=True)
            basetemp = make_numbered_dir_with_cleanup(
                prefix="pytest-", root=rootdir, keep=3, lock_timeout=LOCK_TIMEOUT
            )
        assert basetemp is not None, basetemp
        self._basetemp = t = basetemp
        self._trace("new basetemp", t)
        return t


@final
@attr.s
class TempdirFactory:
    """Backward comptibility wrapper that implements :class:``py.path.local``
    for :class:``TempPathFactory``."""

    _tmppath_factory = attr.ib(type=TempPathFactory)

    def mktemp(self, basename: str, numbered: bool = True) -> py.path.local:
        """Same as :meth:`TempPathFactory.mktemp`, but returns a ``py.path.local`` object."""
        return py.path.local(self._tmppath_factory.mktemp(basename, numbered).resolve())

    def getbasetemp(self) -> py.path.local:
        """Backward compat wrapper for ``_tmppath_factory.getbasetemp``."""
        return py.path.local(self._tmppath_factory.getbasetemp().resolve())


def get_user() -> Optional[str]:
    """Return the current user name, or None if getuser() does not work
    in the current environment (see #1010)."""
    import getpass

    try:
        return getpass.getuser()
    except (ImportError, KeyError):
        return None


def pytest_configure(config: Config) -> None:
    """Create a TempdirFactory and attach it to the config object.

    This is to comply with existing plugins which expect the handler to be
    available at pytest_configure time, but ideally should be moved entirely
    to the tmpdir_factory session fixture.
    """
    mp = MonkeyPatch()
    tmppath_handler = TempPathFactory.from_config(config)
    t = TempdirFactory(tmppath_handler)
    config._cleanup.append(mp.undo)
    mp.setattr(config, "_tmp_path_factory", tmppath_handler, raising=False)
    mp.setattr(config, "_tmpdirhandler", t, raising=False)


@pytest.fixture(scope="session")
def tmpdir_factory(request: FixtureRequest) -> TempdirFactory:
    """Return a :class:`_pytest.tmpdir.TempdirFactory` instance for the test session."""
    # Set dynamically by pytest_configure() above.
    return request.config._tmpdirhandler  # type: ignore


@pytest.fixture(scope="session")
def tmp_path_factory(request: FixtureRequest) -> TempPathFactory:
    """Return a :class:`_pytest.tmpdir.TempPathFactory` instance for the test session."""
    # Set dynamically by pytest_configure() above.
    return request.config._tmp_path_factory  # type: ignore


def _mk_tmp(request: FixtureRequest, factory: TempPathFactory) -> Path:
    name = request.node.name
    name = re.sub(r"[\W]", "_", name)
    MAXVAL = 30
    name = name[:MAXVAL]
    return factory.mktemp(name, numbered=True)


@pytest.fixture
def tmpdir(tmp_path: Path) -> py.path.local:
    """Return a temporary directory path object which is unique to each test
    function invocation, created as a sub directory of the base temporary
    directory.

    The returned object is a `py.path.local`_ path object.

    .. _`py.path.local`: https://py.readthedocs.io/en/latest/path.html
    """
    return py.path.local(tmp_path)


@pytest.fixture
def tmp_path(request: FixtureRequest, tmp_path_factory: TempPathFactory) -> Path:
    """Return a temporary directory path object which is unique to each test
    function invocation, created as a sub directory of the base temporary
    directory.

    The returned object is a :class:`pathlib.Path` object.

    .. note::

        In python < 3.6 this is a pathlib2.Path.
    """

    return _mk_tmp(request, tmp_path_factory)

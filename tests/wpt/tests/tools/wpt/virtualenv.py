# mypy: allow-untyped-defs

import logging
import os
import shutil
import site
import sys
import sysconfig
from shutil import which

# The `pkg_resources` module is provided by `setuptools`, which is itself a
# dependency of `virtualenv`. Tolerate its absence so that this module may be
# evaluated when that module is not available. Because users may not recognize
# the `pkg_resources` module by name, raise a more descriptive error if it is
# referenced during execution.
try:
    import pkg_resources as _pkg_resources
    get_pkg_resources = lambda: _pkg_resources
except ImportError:
    def get_pkg_resources():
        raise ValueError("The Python module `virtualenv` is not installed.")

from tools.wpt.utils import call

logger = logging.getLogger(__name__)

class Virtualenv:
    def __init__(self, path, skip_virtualenv_setup):
        self.path = path
        self.skip_virtualenv_setup = skip_virtualenv_setup
        if not skip_virtualenv_setup:
            self.virtualenv = [sys.executable, "-m", "venv"]
            self._working_set = None

    @property
    def exists(self):
        # We need to check also for lib_path because different python versions
        # create different library paths.
        return os.path.isdir(self.path) and os.path.isdir(self.lib_path)

    @property
    def broken_link(self):
        python_link = os.path.join(self.path, ".Python")
        return os.path.lexists(python_link) and not os.path.exists(python_link)

    def create(self):
        if os.path.exists(self.path):
            shutil.rmtree(self.path, ignore_errors=True)
            self._working_set = None
        call(*self.virtualenv, self.path)

    def get_paths(self):
        """Wrapper around sysconfig.get_paths(), returning the appropriate paths for the env."""
        if "venv" in sysconfig.get_scheme_names():
            # This should always be used on Python 3.11 and above.
            scheme = "venv"
        elif os.name == "nt":
            # This matches nt_venv, unless sysconfig has been modified.
            scheme = "nt"
        elif os.name == "posix":
            # This matches posix_venv, unless sysconfig has been modified.
            scheme = "posix_prefix"
        elif sys.version_info >= (3, 10):
            # Using the default scheme is somewhat fragile, as various Python
            # distributors (e.g., what Debian and Fedora package, and what Xcode
            # includes) change the default scheme away from the upstream
            # defaults, but it's about as good as we can do.
            scheme = sysconfig.get_default_scheme()
        else:
            # This is explicitly documented as having previously existed in the 3.10
            # docs, and has existed since CPython 2.7 and 3.1 (but not 3.0).
            scheme = sysconfig._get_default_scheme()

        vars = {
            "base": self.path,
            "platbase": self.path,
            "installed_base": self.path,
            "installed_platbase": self.path,
        }

        return sysconfig.get_paths(scheme, vars)

    @property
    def bin_path(self):
        return self.get_paths()["scripts"]

    @property
    def pip_path(self):
        path = which("pip3", path=self.bin_path)
        if path is None:
            path = which("pip", path=self.bin_path)
        if path is None:
            raise ValueError("pip3 or pip not found")
        return path

    @property
    def lib_path(self):
        # We always return platlib here, even if it differs to purelib, because we can
        # always install pure-Python code into the platlib safely too. It's also very
        # unlikely to differ for a venv.
        return self.get_paths()["platlib"]

    @property
    def working_set(self):
        if not self.exists:
            raise ValueError("trying to read working_set when venv doesn't exist")

        if self._working_set is None:
            self._working_set = get_pkg_resources().WorkingSet((self.lib_path,))

        return self._working_set

    def activate(self):
        if sys.platform == "darwin":
            # The default Python on macOS sets a __PYVENV_LAUNCHER__ environment
            # variable which affects invocation of python (e.g. via pip) in a
            # virtualenv. Unset it if present to avoid this. More background:
            # https://github.com/web-platform-tests/wpt/issues/27377
            # https://github.com/python/cpython/pull/9516
            os.environ.pop("__PYVENV_LAUNCHER__", None)

        paths = self.get_paths()

        # Setup the path and site packages as if we'd launched with the virtualenv active
        bin_dir = paths["scripts"]
        os.environ["PATH"] = os.pathsep.join([bin_dir] + os.environ.get("PATH", "").split(os.pathsep))

        # While not required (`./venv/bin/python3` won't set it, but
        # `source ./venv/bin/activate && python3` will), we have historically set this.
        os.environ["VIRTUAL_ENV"] = self.path

        prev_length = len(sys.path)

        # Add the venv library paths as sitedirs.
        for key in ["purelib", "platlib"]:
            site.addsitedir(paths[key])

        # Rearrange the path
        sys.path[:] = sys.path[prev_length:] + sys.path[0:prev_length]

        # Change prefixes, similar to what initconfig/site does for venvs.
        sys.exec_prefix = self.path
        sys.prefix = self.path

    def start(self):
        if not self.exists or self.broken_link:
            self.create()
        self.activate()

    def install(self, *requirements):
        try:
            self.working_set.require(*requirements)
        except Exception:
            pass
        else:
            return

        # `--prefer-binary` guards against race conditions when installation
        # occurs while packages are in the process of being published.
        call(self.pip_path, "install", "--prefer-binary", *requirements)

    def install_requirements(self, *requirements_paths):
        install = []
        # Check which requirements are already satisfied, to skip calling pip
        # at all in the case that we've already installed everything, and to
        # minimise the installs in other cases.
        for requirements_path in requirements_paths:
            with open(requirements_path) as f:
                try:
                    self.working_set.require(f.read())
                except Exception:
                    install.append(requirements_path)

        if install:
            # `--prefer-binary` guards against race conditions when installation
            # occurs while packages are in the process of being published.
            cmd = [self.pip_path, "install", "--prefer-binary"]
            for path in install:
                cmd.extend(["-r", path])
            call(*cmd)

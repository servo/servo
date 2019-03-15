import os
import shutil
import sys
import logging
from distutils.spawn import find_executable

from tools.wpt.utils import call

logger = logging.getLogger(__name__)

class Virtualenv(object):
    def __init__(self, path, skip_virtualenv_setup):
        self.path = path
        self.skip_virtualenv_setup = skip_virtualenv_setup
        if not skip_virtualenv_setup:
            self.virtualenv = find_executable("virtualenv")
            if not self.virtualenv:
                raise ValueError("virtualenv must be installed and on the PATH")

    @property
    def exists(self):
        return os.path.isdir(self.path)

    def create(self):
        if os.path.exists(self.path):
            shutil.rmtree(self.path)
        call(self.virtualenv, self.path, "-p", sys.executable)

    @property
    def bin_path(self):
        if sys.platform in ("win32", "cygwin"):
            return os.path.join(self.path, "Scripts")
        return os.path.join(self.path, "bin")

    @property
    def pip_path(self):
        path = find_executable("pip", self.bin_path)
        if path is None:
            raise ValueError("pip not found")
        return path

    def activate(self):
        path = os.path.join(self.bin_path, "activate_this.py")
        execfile(path, {"__file__": path})  # noqa: F821

    def start(self):
        if not self.exists:
            self.create()
        self.activate()

    def install(self, *requirements):
        # `--prefer-binary` guards against race conditions when installation
        # occurs while packages are in the process of being published.
        call(self.pip_path, "install", "--prefer-binary", *requirements)

    def install_requirements(self, requirements_path):
        # `--prefer-binary` guards against race conditions when installation
        # occurs while packages are in the process of being published.
        call(
            self.pip_path, "install", "--prefer-binary", "-r", requirements_path
        )

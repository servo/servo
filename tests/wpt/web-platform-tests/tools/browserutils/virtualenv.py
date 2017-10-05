import os
import sys
import logging
from distutils.spawn import find_executable

from utils import call

logger = logging.getLogger(__name__)

class Virtualenv(object):
    def __init__(self, path):
        self.path = path
        self.virtualenv = find_executable("virtualenv")
        if not self.virtualenv:
            raise ValueError("virtualenv must be installed and on the PATH")

    @property
    def exists(self):
        return os.path.isdir(self.path)

    def create(self):
        if os.path.exists(self.path):
            shutil.rmtree(self.path)
        call(self.virtualenv, self.path)

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
        execfile(path, {"__file__": path})

    def start(self):
        if not self.exists:
            self.create()
        self.activate()

    def install(self, *requirements):
        call(self.pip_path, "install", *requirements)

    def install_requirements(self, requirements_path):
        call(self.pip_path, "install", "-r", requirements_path)

import os
from os import path
import subprocess
import sys
import toml

from mach.registrar import Registrar

class cd:
    """Context manager for changing the current working directory"""
    def __init__(self, newPath):
        self.newPath = newPath

    def __enter__(self):
        self.savedPath = os.getcwd()
        os.chdir(self.newPath)

    def __exit__(self, etype, value, traceback):
        os.chdir(self.savedPath)

class CommandBase(object):
    """Base class for mach command providers.

    This mostly handles configuration management, such as .servobuild."""

    def __init__(self, context):
        self.context = context

        if not hasattr(self.context, "bootstrapped"):
            self.context.bootstrapped = False

        config_path = path.join(context.topdir, ".servobuild")
        if path.exists(config_path):
            self.config = toml.loads(open(config_path).read())
        else:
            self.config = {}

        # Handle missing/default items
        self.config.setdefault("tools", {})
        self.config["tools"].setdefault("system-rust", False)
        self.config["tools"].setdefault("system-cargo", False)
        self.config["tools"].setdefault("rust-root", "")
        self.config["tools"].setdefault("cargo-root", "")
        if not self.config["tools"]["system-rust"]:
            self.config["tools"]["rust-root"] = path.join(context.topdir, "rust")
        if not self.config["tools"]["system-cargo"]:
            self.config["tools"]["cargo-root"] = path.join(context.topdir, "cargo")

    def build_env(self):
        """Return an extended environment dictionary."""
        env = os.environ.copy()
        extra_path = []
        extra_lib = []
        if not self.config["tools"]["system-rust"] or self.config["tools"]["rust-root"]:
            extra_path += [path.join(self.config["tools"]["rust-root"], "bin")]
            extra_lib += [path.join(self.config["tools"]["rust-root"], "lib")]
        if not self.config["tools"]["system-cargo"] or self.config["tools"]["cargo-root"]:
            extra_path += [path.join(self.config["tools"]["cargo-root"], "bin")]

        if extra_path:
            env["PATH"] = "%s%s%s" % (os.pathsep.join(extra_path), os.pathsep, env["PATH"])
        if extra_lib:
            if sys.platform == "darwin":
                env["DYLD_LIBRARY_PATH"] = "%s%s%s" % \
                                           (os.pathsep.join(extra_lib),
                                            os.pathsep,
                                            env.get("DYLD_LIBRARY_PATH", ""))
            else:
                env["LD_LIBRARY_PATH"] = "%s%s%s" % \
                                         (os.pathsep.join(extra_lib),
                                          os.pathsep,
                                          env.get("LD_LIBRARY_PATH", ""))

        return env

    def ensure_bootstrapped(self):
        if self.context.bootstrapped: return

        submodules = subprocess.check_output(["git", "submodule", "status"])
        for line in submodules.split('\n'):
            components = line.strip().split(' ')
            if len(components) > 1 and components[0].startswith(('-', '+')):
                module_path = components[1]
                subprocess.check_call(["git", "submodule", "update",
                                       "--init", "--recursive", "--", module_path])

        if not self.config["tools"]["system-rust"] and \
           not path.exists(path.join(self.context.topdir, "rust", "bin", "rustc")):
            Registrar.dispatch("bootstrap-rust", context=self.context)
        if not self.config["tools"]["system-cargo"] and \
           not path.exists(path.join(self.context.topdir, "cargo", "bin", "cargo")):
            Registrar.dispatch("bootstrap-cargo", context=self.context)

        self.context.bootstrapped = True

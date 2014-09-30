import os
from os import path
import contextlib
import subprocess
import sys
import toml

from mach.registrar import Registrar


@contextlib.contextmanager
def cd(new_path):
    """Context manager for changing the current working directory"""
    previous_path = os.getcwd()
    try:
        os.chdir(new_path)
        yield
    finally:
        os.chdir(previous_path)


def host_triple():
    os_type = subprocess.check_output(["uname", "-s"]).strip().lower()
    if os_type == "linux":
        os_type = "unknown-linux-gnu"
    elif os_type == "darwin":
        os_type = "apple-darwin"
    elif os_type == "android":
        os_type == "linux-androideabi"
    else:
        os_type == "unknown"

    cpu_type = subprocess.check_output(["uname", "-m"]).strip().lower()
    if cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
        cpu_type = "i686"
    elif cpu_type in ["x86_64", "x86-64", "x64", "amd64"]:
        cpu_type = "x86_64"
    elif cpu_type == "arm":
        cpu_type = "arm"
    else:
        cpu_type = "unknown"

    return "%s-%s" % (cpu_type, os_type)


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
            self.config["tools"]["rust-root"] = path.join(
                context.topdir, "rust", *self.rust_snapshot_path().split("/"))
        if not self.config["tools"]["system-cargo"]:
            self.config["tools"]["cargo-root"] = path.join(
                context.topdir, "cargo")

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)

        self.config.setdefault("android", {})
        self.config["android"].setdefault("sdk", "")
        self.config["android"].setdefault("ndk", "")
        self.config["android"].setdefault("toolchain", "")

    _rust_snapshot_path = None

    def rust_snapshot_path(self):
        if self._rust_snapshot_path is None:
            filename = path.join(self.context.topdir, "rust-snapshot-hash")
            snapshot_hash = open(filename).read().strip()
            self._rust_snapshot_path = "%s-%s" % (snapshot_hash, host_triple())
        return self._rust_snapshot_path

    def build_env(self):
        """Return an extended environment dictionary."""
        env = os.environ.copy()
        extra_path = []
        extra_lib = []
        if not self.config["tools"]["system-rust"] \
                or self.config["tools"]["rust-root"]:
            extra_path += [path.join(self.config["tools"]["rust-root"], "bin")]
            extra_lib += [path.join(self.config["tools"]["rust-root"], "lib")]
        if not self.config["tools"]["system-cargo"] \
                or self.config["tools"]["cargo-root"]:
            extra_path += [
                path.join(self.config["tools"]["cargo-root"], "bin")]

        if extra_path:
            env["PATH"] = "%s%s%s" % (
                os.pathsep.join(extra_path), os.pathsep, env["PATH"])
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

        # Paths to Android build tools:
        if self.config["android"]["sdk"]:
            env["ANDROID_SDK"] = self.config["android"]["sdk"]
        if self.config["android"]["ndk"]:
            env["ANDROID_NDK"] = self.config["android"]["ndk"]
        if self.config["android"]["toolchain"]:
            env["ANDROID_TOOLCHAIN"] = self.config["android"]["toolchain"]

        return env

    def ensure_bootstrapped(self):
        if self.context.bootstrapped:
            return

        submodules = subprocess.check_output(["git", "submodule", "status"])
        for line in submodules.split('\n'):
            components = line.strip().split(' ')
            if len(components) > 1 and components[0].startswith(('-', '+')):
                module_path = components[1]
                subprocess.check_call(["git", "submodule", "update",
                                       "--init", "--recursive",
                                       "--", module_path])

        if not self.config["tools"]["system-rust"] and \
           not path.exists(path.join(
                self.config["tools"]["rust-root"], "bin", "rustc")):
            Registrar.dispatch("bootstrap-rust", context=self.context)
        if not self.config["tools"]["system-cargo"] and \
           not path.exists(path.join(
                self.context.topdir, "cargo", "bin", "cargo")):
            Registrar.dispatch("bootstrap-cargo", context=self.context)

        self.context.bootstrapped = True

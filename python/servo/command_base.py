# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

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

        def resolverelative(category, key):
            # Allow ~
            self.config[category][key] = path.expanduser(self.config[category][key])
            # Resolve relative paths
            self.config[category][key] = path.join(context.topdir,
                                                   self.config[category][key])

        if not hasattr(self.context, "bootstrapped"):
            self.context.bootstrapped = False

        config_path = path.join(context.topdir, ".servobuild")
        if path.exists(config_path):
            self.config = toml.loads(open(config_path).read())
        else:
            self.config = {}

        # Handle missing/default items
        self.config.setdefault("tools", {})
        default_cache_dir = os.environ.get("SERVO_CACHE_DIR",
                                           path.join(context.topdir, ".servo"))
        self.config["tools"].setdefault("cache-dir", default_cache_dir)
        resolverelative("tools", "cache-dir")

        self.config["tools"].setdefault("cargo-home-dir",
                                        path.join(context.topdir, ".cargo"))
        resolverelative("tools", "cargo-home-dir")

        context.sharedir = self.config["tools"]["cache-dir"]

        self.config["tools"].setdefault("system-rust", False)
        self.config["tools"].setdefault("system-cargo", False)
        self.config["tools"].setdefault("rust-root", "")
        self.config["tools"].setdefault("cargo-root", "")
        if not self.config["tools"]["system-rust"]:
            self.config["tools"]["rust-root"] = path.join(
                context.sharedir, "rust", *self.rust_snapshot_path().split("/"))
        if not self.config["tools"]["system-cargo"]:
            self.config["tools"]["cargo-root"] = path.join(
                context.sharedir, "cargo", self.cargo_build_id())

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)
        self.config["build"].setdefault("mode", "")
        self.config["build"].setdefault("debug-mozjs", False)

        self.config.setdefault("android", {})
        self.config["android"].setdefault("sdk", "")
        self.config["android"].setdefault("ndk", "")
        self.config["android"].setdefault("toolchain", "")

        self.config.setdefault("gonk", {})
        self.config["gonk"].setdefault("b2g", "")
        self.config["gonk"].setdefault("product", "flame")

    _rust_snapshot_path = None
    _cargo_build_id = None

    def rust_snapshot_path(self):
        if self._rust_snapshot_path is None:
            filename = path.join(self.context.topdir, "rust-snapshot-hash")
            snapshot_hash = open(filename).read().strip()
            self._rust_snapshot_path = "%s-%s" % (snapshot_hash, host_triple())
        return self._rust_snapshot_path

    def cargo_build_id(self):
        if self._cargo_build_id is None:
            filename = path.join(self.context.topdir, "cargo-nightly-build")
            self._cargo_build_id = open(filename).read().strip()
        return self._cargo_build_id

    def get_target_dir(self):
        if "CARGO_TARGET_DIR" in os.environ:
            return os.environ["CARGO_TARGET_DIR"]
        else:
            return path.join(self.context.topdir, "target")

    def get_binary_path(self, release, dev):
        base_path = self.get_target_dir()
        release_path = path.join(base_path, "release", "servo")
        dev_path = path.join(base_path, "debug", "servo")

        # Prefer release if both given
        if release and dev:
            dev = False

        release_exists = path.exists(release_path)
        dev_exists = path.exists(dev_path)

        if not release_exists and not dev_exists:
            print("No Servo binary found. Please run './mach build' and try again.")
            sys.exit()

        if release and release_exists:
            return release_path

        if dev and dev_exists:
            return dev_path

        if not dev and not release and release_exists and dev_exists:
            print("You have multiple profiles built. Please specify which "
                  "one to run with '--release' or '--dev'.")
            sys.exit()

        if not dev and not release:
            if release_exists:
                return release_path
            else:
                return dev_path

        print("The %s profile is not built. Please run './mach build%s' "
              "and try again." % ("release" if release else "dev",
                                  " --release" if release else ""))
        sys.exit()

    def build_env(self, gonk=False, hosts_file_path=None):
        """Return an extended environment dictionary."""
        env = os.environ.copy()
        extra_path = []
        extra_lib = []
        if not self.config["tools"]["system-rust"] \
                or self.config["tools"]["rust-root"]:
            env["RUST_ROOT"] = self.config["tools"]["rust-root"]
            # These paths are for when rust-root points to an unpacked installer
            extra_path += [path.join(self.config["tools"]["rust-root"], "rustc", "bin")]
            extra_lib += [path.join(self.config["tools"]["rust-root"], "rustc", "lib")]
            # These paths are for when rust-root points to a rustc sysroot
            extra_path += [path.join(self.config["tools"]["rust-root"], "bin")]
            extra_lib += [path.join(self.config["tools"]["rust-root"], "lib")]

        if not self.config["tools"]["system-cargo"] \
                or self.config["tools"]["cargo-root"]:
            # This path is for when rust-root points to an unpacked installer
            extra_path += [
                path.join(self.config["tools"]["cargo-root"], "cargo", "bin")]
            # This path is for when rust-root points to a rustc sysroot
            extra_path += [
                path.join(self.config["tools"]["cargo-root"], "bin")]

        if extra_path:
            env["PATH"] = "%s%s%s" % (
                os.pathsep.join(extra_path), os.pathsep, env["PATH"])

        if "CARGO_HOME" not in env:
            env["CARGO_HOME"] = self.config["tools"]["cargo-home-dir"]

        if "CARGO_TARGET_DIR" not in env:
            env["CARGO_TARGET_DIR"] = path.join(self.context.topdir, "target")

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

        if gonk:
            if self.config["gonk"]["b2g"]:
                env["GONKDIR"] = self.config["gonk"]["b2g"]
            if "GONKDIR" not in env:
                # Things can get pretty opaque if this hasn't been set
                print("Please set $GONKDIR in your environment or .servobuild file")
                sys.exit(1)
            if self.config["gonk"]["product"]:
                env["GONK_PRODUCT"] = self.config["gonk"]["product"]

            env["ARCH_DIR"] = "arch-arm"
            env["CPPFLAGS"] = (
                "-DANDROID -DTARGET_OS_GONK "
                "-DANDROID_VERSION=19 "
                "-DGR_GL_USE_NEW_SHADER_SOURCE_SIGNATURE=1 "
                "-isystem %(gonkdir)s/bionic/libc/%(archdir)s/include "
                "-isystem %(gonkdir)s/bionic/libc/include/ "
                "-isystem %(gonkdir)s/bionic/libc/kernel/common "
                "-isystem %(gonkdir)s/bionic/libc/kernel/%(archdir)s "
                "-isystem %(gonkdir)s/bionic/libm/include "
                "-I%(gonkdir)s/system "
                "-I%(gonkdir)s/system/core/include "
                "-I%(gonkdir)s/frameworks/native/opengl/include "
                "-I%(gonkdir)s/external/zlib "
            ) % {"gonkdir": env["GONKDIR"], "archdir": env["ARCH_DIR"]}
            env["CXXFLAGS"] = (
                "-O2 -mandroid -fPIC "
                "-isystem %(gonkdir)s/api/cpp/include "
                "-isystem %(gonkdir)s/external/stlport/stlport "
                "-isystem %(gonkdir)s/bionic "
                "-isystem %(gonkdir)s/bionic/libstdc++/include "
                "%(cppflags)s"
            ) % {"gonkdir": env["GONKDIR"], "cppflags": env["CPPFLAGS"]}
            env["CFLAGS"] = (
                "%(cxxflags)s"
            ) % {"cxxflags": env["CXXFLAGS"]}

            another_extra_path = path.join(
                env["GONKDIR"], "prebuilts", "gcc", "linux-x86", "arm", "arm-linux-androideabi-4.7", "bin")

            env["gonkdir"] = env["GONKDIR"]
            env["gonk_toolchain_prefix"] = (
                "%(toolchain)s/arm-linux-androideabi-"
            ) % {"toolchain": another_extra_path}

            env["PATH"] = "%s%s%s" % (another_extra_path, os.pathsep, env["PATH"])
            env["LDFLAGS"] = (
                "-mandroid -L%(gonkdir)s/out/target/product/%(gonkproduct)s/obj/lib "
                "-Wl,-rpath-link=%(gonkdir)s/out/target/product/%(gonkproduct)s/obj/lib "
                "--sysroot=%(gonkdir)s/out/target/product/%(gonkproduct)s/obj/"
            ) % {"gonkdir": env["GONKDIR"], "gonkproduct": env["GONK_PRODUCT"]}

            # Not strictly necessary for a vanilla build, but might be when tweaking the openssl build
            openssl_dir = (
                "%(gonkdir)s/out/target/product/%(gonkproduct)s/obj/lib"
            ) % {"gonkdir": env["GONKDIR"], "gonkproduct": env["GONK_PRODUCT"]}
            env["OPENSSL_LIB_DIR"] = openssl_dir
            env['OPENSSL_INCLUDE_DIR'] = path.join(env["GONKDIR"], "external/openssl/include")

        # FIXME: These are set because they are the variable names that
        # android-rs-glue expects. However, other submodules have makefiles that
        # reference the env var names above. Once glutin is enabled and set as
        # the default, we could modify the subproject makefiles to use the names
        # below and remove the vars above, to avoid duplication.
        if "ANDROID_SDK" in env:
            env["ANDROID_HOME"] = env["ANDROID_SDK"]
        if "ANDROID_NDK" in env:
            env["NDK_HOME"] = env["ANDROID_NDK"]
        if "ANDROID_TOOLCHAIN" in env:
            env["NDK_STANDALONE"] = env["ANDROID_TOOLCHAIN"]

        if hosts_file_path:
            env['HOST_FILE'] = hosts_file_path

        env['RUSTDOC'] = path.join(self.context.topdir, 'etc', 'rustdoc-with-private')
        env['RUSTC'] = path.join(self.context.topdir, 'etc', 'rustc-with-gold')

        return env

    def servo_crate(self):
        return path.join(self.context.topdir, "components", "servo")

    def android_support_dir(self):
        return path.join(self.context.topdir, "support", "android")

    def ensure_bootstrapped(self):
        if self.context.bootstrapped:
            return

        Registrar.dispatch("update-submodules", context=self.context)

        if not self.config["tools"]["system-rust"] and \
           not path.exists(path.join(
                self.config["tools"]["rust-root"], "rustc", "bin", "rustc")):
            Registrar.dispatch("bootstrap-rust", context=self.context)
        if not self.config["tools"]["system-cargo"] and \
           not path.exists(path.join(
                self.config["tools"]["cargo-root"], "cargo", "bin", "cargo")):
            Registrar.dispatch("bootstrap-cargo", context=self.context)

        self.context.bootstrapped = True

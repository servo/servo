# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from glob import glob
import gzip
import itertools
import locale
import os
from os import path
import contextlib
import subprocess
from subprocess import PIPE
import sys
import tarfile
import platform

import toml

from mach.registrar import Registrar

BIN_SUFFIX = ".exe" if sys.platform == "win32" else ""


@contextlib.contextmanager
def cd(new_path):
    """Context manager for changing the current working directory"""
    previous_path = os.getcwd()
    try:
        os.chdir(new_path)
        yield
    finally:
        os.chdir(previous_path)


@contextlib.contextmanager
def setlocale(name):
    """Context manager for changing the current locale"""
    saved_locale = locale.setlocale(locale.LC_ALL)
    try:
        yield locale.setlocale(locale.LC_ALL, name)
    finally:
        locale.setlocale(locale.LC_ALL, saved_locale)


def find_dep_path_newest(package, bin_path):
    deps_path = path.join(path.split(bin_path)[0], "build")
    candidates = []
    with cd(deps_path):
        for c in glob(package + '-*'):
            candidate_path = path.join(deps_path, c)
            if path.exists(path.join(candidate_path, "output")):
                candidates.append(candidate_path)
    if candidates:
        return max(candidates, key=lambda c: path.getmtime(path.join(c, "output")))
    return None


def archive_deterministically(dir_to_archive, dest_archive, prepend_path=None):
    """Create a .tar.gz archive in a deterministic (reproducible) manner.

    See https://reproducible-builds.org/docs/archives/ for more details."""

    def reset(tarinfo):
        """Helper to reset owner/group and modification time for tar entries"""
        tarinfo.uid = tarinfo.gid = 0
        tarinfo.uname = tarinfo.gname = "root"
        tarinfo.mtime = 0
        return tarinfo

    dest_archive = os.path.abspath(dest_archive)
    with cd(dir_to_archive):
        current_dir = "."
        file_list = [current_dir]
        for root, dirs, files in os.walk(current_dir):
            for name in itertools.chain(dirs, files):
                file_list.append(os.path.join(root, name))

        # Sort file entries with the fixed locale
        with setlocale('C'):
            file_list.sort(cmp=locale.strcoll)

        # Use a temporary file and atomic rename to avoid partially-formed
        # packaging (in case of exceptional situations like running out of disk space).
        # TODO do this in a temporary folder after #11983 is fixed
        temp_file = '{}.temp~'.format(dest_archive)
        with os.fdopen(os.open(temp_file, os.O_WRONLY | os.O_CREAT, 0644), 'w') as out_file:
            with gzip.GzipFile('wb', fileobj=out_file, mtime=0) as gzip_file:
                with tarfile.open(fileobj=gzip_file, mode='w:') as tar_file:
                    for entry in file_list:
                        arcname = entry
                        if prepend_path is not None:
                            arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                        tar_file.add(entry, filter=reset, recursive=False, arcname=arcname)
        os.rename(temp_file, dest_archive)


def host_triple():
    os_type = platform.system().lower()
    if os_type == "linux":
        os_type = "unknown-linux-gnu"
    elif os_type == "darwin":
        os_type = "apple-darwin"
    elif os_type == "android":
        os_type = "linux-androideabi"
    elif os_type == "windows":
        # If we are in a Visual Studio environment, use msvc
        if os.getenv("PLATFORM") is not None:
            os_type = "pc-windows-msvc"
        elif os.getenv("MSYSTEM") is not None:
            os_type = "pc-windows-gnu"
        else:
            os_type = "unknown"
    elif os_type.startswith("mingw64_nt-") or os_type.startswith("cygwin_nt-"):
        os_type = "pc-windows-gnu"
    elif os_type == "freebsd":
        os_type = "unknown-freebsd"
    else:
        os_type = "unknown"

    cpu_type = platform.machine().lower()
    if os_type.endswith("-msvc"):
        # vcvars*.bat should set it properly
        platform_env = os.environ.get("PLATFORM")
        if platform_env == "X86":
            cpu_type = "i686"
        elif platform_env == "X64":
            cpu_type = "x86_64"
        else:
            cpu_type = "unknown"
    elif cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
        cpu_type = "i686"
    elif cpu_type in ["x86_64", "x86-64", "x64", "amd64"]:
        cpu_type = "x86_64"
    elif cpu_type == "arm":
        cpu_type = "arm"
    else:
        cpu_type = "unknown"

    return "%s-%s" % (cpu_type, os_type)


def normalize_env(env):
    # There is a bug in subprocess where it doesn't like unicode types in
    # environment variables. Here, ensure all unicode are converted to
    # binary. utf-8 is our globally assumed default. If the caller doesn't
    # want UTF-8, they shouldn't pass in a unicode instance.
    normalized_env = {}
    for k, v in env.items():
        if isinstance(k, unicode):
            k = k.encode('utf-8', 'strict')

        if isinstance(v, unicode):
            v = v.encode('utf-8', 'strict')

        normalized_env[k] = v

    return normalized_env


def call(*args, **kwargs):
    """Wrap `subprocess.call`, printing the command if verbose=True."""
    verbose = kwargs.pop('verbose', False)
    if verbose:
        print(' '.join(args[0]))
    if 'env' in kwargs:
        kwargs['env'] = normalize_env(kwargs['env'])
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    return subprocess.call(*args, shell=sys.platform == 'win32', **kwargs)


def check_call(*args, **kwargs):
    """Wrap `subprocess.check_call`, printing the command if verbose=True.

    Also fix any unicode-containing `env`, for subprocess """
    verbose = kwargs.pop('verbose', False)

    if 'env' in kwargs:
        kwargs['env'] = normalize_env(kwargs['env'])

    if verbose:
        print(' '.join(args[0]))
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    proc = subprocess.Popen(*args, shell=sys.platform == 'win32', **kwargs)
    status = None
    # Leave it to the subprocess to handle Ctrl+C. If it terminates as
    # a result of Ctrl+C, proc.wait() will return a status code, and,
    # we get out of the loop. If it doesn't, like e.g. gdb, we continue
    # waiting.
    while status is None:
        try:
            status = proc.wait()
        except KeyboardInterrupt:
            pass

    if status:
        raise subprocess.CalledProcessError(status, ' '.join(*args))


def is_windows():
    """ Detect windows, mingw, cygwin """
    return sys.platform == 'win32' or sys.platform == 'msys' or sys.platform == 'cygwin'


def is_macosx():
    return sys.platform == 'darwin'


def is_linux():
    return sys.platform.startswith('linux')


def set_osmesa_env(bin_path, env):
    """Set proper LD_LIBRARY_PATH and DRIVE for software rendering on Linux and OSX"""
    if is_linux():
        dep_path = find_dep_path_newest('osmesa-src', bin_path)
        if not dep_path:
            return None
        osmesa_path = path.join(dep_path, "out", "lib", "gallium")
        env["LD_LIBRARY_PATH"] = osmesa_path
        env["GALLIUM_DRIVER"] = "softpipe"
    elif is_macosx():
        osmesa_path = path.join(find_dep_path_newest('osmesa-src', bin_path),
                                "out", "src", "gallium", "targets", "osmesa", ".libs")
        glapi_path = path.join(find_dep_path_newest('osmesa-src', bin_path),
                               "out", "src", "mapi", "shared-glapi", ".libs")
        if not (osmesa_path and glapi_path):
            return None
        env["DYLD_LIBRARY_PATH"] = osmesa_path + ":" + glapi_path
        env["GALLIUM_DRIVER"] = "softpipe"
    return env


class BuildNotFound(Exception):
    def __init__(self, message):
        self.message = message

    def __str__(self):
        return self.message


class CommandBase(object):
    """Base class for mach command providers.

    This mostly handles configuration management, such as .servobuild."""

    def __init__(self, context):
        self.context = context

        def get_env_bool(var, default):
            # Contents of env vars are strings by default. This returns the
            # boolean value of the specified environment variable, or the
            # speciried default if the var doesn't contain True or False
            return {'True': True, 'False': False}.get(os.environ.get(var), default)

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
            with open(config_path) as f:
                self.config = toml.loads(f.read())
        else:
            self.config = {}

        # Handle missing/default items
        self.config.setdefault("tools", {})
        default_cache_dir = os.environ.get("SERVO_CACHE_DIR",
                                           path.join(context.topdir, ".servo"))
        self.config["tools"].setdefault("cache-dir", default_cache_dir)
        resolverelative("tools", "cache-dir")

        default_cargo_home = os.environ.get("CARGO_HOME",
                                            path.join(context.topdir, ".cargo"))
        self.config["tools"].setdefault("cargo-home-dir", default_cargo_home)
        resolverelative("tools", "cargo-home-dir")

        context.sharedir = self.config["tools"]["cache-dir"]

        self.config["tools"].setdefault("system-rust", False)
        self.config["tools"].setdefault("system-cargo", False)
        self.config["tools"].setdefault("rust-root", "")
        self.config["tools"].setdefault("cargo-root", "")
        self.set_use_stable_rust(False)
        if not self.config["tools"]["system-cargo"]:
            self.config["tools"]["cargo-root"] = path.join(
                context.sharedir, "cargo", self.cargo_build_id())
        self.config["tools"].setdefault("rustc-with-gold", get_env_bool("SERVO_RUSTC_WITH_GOLD", True))

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)
        self.config["build"].setdefault("mode", "")
        self.config["build"].setdefault("debug-mozjs", False)
        self.config["build"].setdefault("ccache", "")
        self.config["build"].setdefault("rustflags", "")

        self.config.setdefault("android", {})
        self.config["android"].setdefault("sdk", "")
        self.config["android"].setdefault("ndk", "")
        self.config["android"].setdefault("toolchain", "")
        self.config["android"].setdefault("platform", "android-18")
        self.config["android"].setdefault("target", "arm-linux-androideabi")

    _use_stable_rust = False
    _rust_version = None
    _rust_version_is_stable = False
    _cargo_build_id = None

    def set_use_stable_rust(self, use_stable_rust=True):
        self._use_stable_rust = use_stable_rust
        if not self.config["tools"]["system-rust"]:
            self.config["tools"]["rust-root"] = path.join(
                self.context.sharedir, "rust", self.rust_path())

    def use_stable_rust(self):
        return self._use_stable_rust

    def rust_path(self):
        version = self.rust_version()
        if self._use_stable_rust:
            return os.path.join(version, "rustc-%s-%s" % (version, host_triple()))
        else:
            return os.path.join(version, "rustc-nightly-%s" % (host_triple()))

    def rust_version(self):
        if self._rust_version is None or self._use_stable_rust != self._rust_version_is_stable:
            filename = path.join(self.context.topdir,
                                 "rust-stable-version" if self._use_stable_rust else "rust-nightly-date")
            with open(filename) as f:
                self._rust_version = f.read().strip()
        return self._rust_version

    def cargo_build_id(self):
        if self._cargo_build_id is None:
            filename = path.join(self.context.topdir, "cargo-commit-hash")
            with open(filename) as f:
                self._cargo_build_id = f.read().strip()
        return self._cargo_build_id

    def get_top_dir(self):
        return self.context.topdir

    def get_target_dir(self):
        if "CARGO_TARGET_DIR" in os.environ:
            return os.environ["CARGO_TARGET_DIR"]
        else:
            return path.join(self.context.topdir, "target")

    def get_binary_path(self, release, dev, android=False):
        # TODO(autrilla): this function could still use work - it shouldn't
        # handle quitting, or printing. It should return the path, or an error.
        base_path = self.get_target_dir()

        if android:
            base_path = path.join(base_path, self.config["android"]["target"])

        binary_name = "servo" + BIN_SUFFIX
        release_path = path.join(base_path, "release", binary_name)
        dev_path = path.join(base_path, "debug", binary_name)

        # Prefer release if both given
        if release and dev:
            dev = False

        release_exists = path.exists(release_path)
        dev_exists = path.exists(dev_path)

        if not release_exists and not dev_exists:
            raise BuildNotFound('No Servo binary found.'
                                ' Perhaps you forgot to run `./mach build`?')

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

    def build_env(self, hosts_file_path=None, target=None, is_build=False):
        """Return an extended environment dictionary."""
        env = os.environ.copy()
        if sys.platform == "win32" and type(env['PATH']) == unicode:
            # On win32, the virtualenv's activate_this.py script sometimes ends up
            # turning os.environ['PATH'] into a unicode string.  This doesn't work
            # for passing env vars in to a process, so we force it back to ascii.
            # We don't use UTF8 since that won't be correct anyway; if you actually
            # have unicode stuff in your path, all this PATH munging would have broken
            # it in any case.
            env['PATH'] = env['PATH'].encode('ascii', 'ignore')
        extra_path = []
        extra_lib = []
        if "msvc" in (target or host_triple()):
            msvc_x64 = "64" if "x86_64" in (target or host_triple()) else ""
            msvc_deps_dir = path.join(self.context.sharedir, "msvc-dependencies")
            extra_path += [path.join(msvc_deps_dir, "cmake", "bin")]
            extra_path += [path.join(msvc_deps_dir, "ninja", "bin")]
            # Link openssl
            env["OPENSSL_INCLUDE_DIR"] = path.join(msvc_deps_dir, "openssl", "include")
            env["OPENSSL_LIB_DIR"] = path.join(msvc_deps_dir, "openssl", "lib" + msvc_x64)
            env["OPENSSL_LIBS"] = "ssleay32MD:libeay32MD"
            # Link moztools
            env["MOZTOOLS_PATH"] = path.join(msvc_deps_dir, "moztools", "bin")

        if is_windows():
            if not os.environ.get("NATIVE_WIN32_PYTHON"):
                env["NATIVE_WIN32_PYTHON"] = sys.executable

        if not self.config["tools"]["system-rust"] \
                or self.config["tools"]["rust-root"]:
            env["RUST_ROOT"] = self.config["tools"]["rust-root"]
            # Add mingw64 binary path before rust paths to avoid conflict with libstdc++-6.dll
            if sys.platform == "msys":
                extra_path += [path.join(os.sep, "mingw64", "bin")]
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
            env["PATH"] = "%s%s%s" % (os.pathsep.join(extra_path), os.pathsep, env["PATH"])

        env["CARGO_HOME"] = self.config["tools"]["cargo-home-dir"]

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
        if self.config["android"]["platform"]:
            env["ANDROID_PLATFORM"] = self.config["android"]["platform"]

        # These are set because they are the variable names that build-apk
        # expects. However, other submodules have makefiles that reference
        # the env var names above. Once glutin is enabled and set as the
        # default, we could modify the subproject makefiles to use the names
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

        if self.config["build"]["rustflags"]:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " " + self.config["build"]["rustflags"]

        # Don't run the gold linker if on Windows https://github.com/servo/servo/issues/9499
        if self.config["tools"]["rustc-with-gold"] and sys.platform not in ("win32", "msys"):
            if subprocess.call(['which', 'ld.gold'], stdout=PIPE, stderr=PIPE) == 0:
                env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C link-args=-fuse-ld=gold"

        if not (self.config["build"]["ccache"] == ""):
            env['CCACHE'] = self.config["build"]["ccache"]

        # Ensure Rust uses hard floats and SIMD on ARM devices
        if target:
            if target.startswith('arm') or target.startswith('aarch64'):
                env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C target-feature=+neon"

        env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -W unused-extern-crates"

        git_info = []
        if os.path.isdir('.git') and is_build:
            git_sha = subprocess.check_output([
                'git', 'rev-parse', '--short', 'HEAD'
            ]).strip()
            git_is_dirty = bool(subprocess.check_output([
                'git', 'status', '--porcelain'
            ]).strip())

            git_info.append('')
            git_info.append(git_sha)
            if git_is_dirty:
                git_info.append('dirty')

        env['GIT_INFO'] = '-'.join(git_info)

        return env

    def servo_crate(self):
        return path.join(self.context.topdir, "ports", "servo")

    def servo_features(self):
        """Return a list of optional features to enable for the Servo crate"""
        features = []
        if self.config["build"]["debug-mozjs"]:
            features += ["debugmozjs"]
        return features

    def android_support_dir(self):
        return path.join(self.context.topdir, "support", "android")

    def android_build_dir(self, dev):
        return path.join(self.get_target_dir(), "arm-linux-androideabi", "debug" if dev else "release")

    def ensure_bootstrapped(self, target=None):
        if self.context.bootstrapped:
            return

        target_platform = target or host_triple()

        rust_root = self.config["tools"]["rust-root"]
        rustc_path = path.join(
            rust_root, "rustc", "bin", "rustc" + BIN_SUFFIX
        )
        rustc_binary_exists = path.exists(rustc_path)

        base_target_path = path.join(rust_root, "rustc", "lib", "rustlib")

        target_path = path.join(base_target_path, target_platform)
        target_exists = path.exists(target_path)

        # Always check if all needed MSVC dependencies are installed
        if "msvc" in target_platform:
            Registrar.dispatch("bootstrap", context=self.context)

        if not (self.config['tools']['system-rust'] or (rustc_binary_exists and target_exists)):
            print("looking for rustc at %s" % (rustc_path))
            Registrar.dispatch("bootstrap-rust", context=self.context, target=filter(None, [target]),
                               stable=self._use_stable_rust)

        cargo_path = path.join(self.config["tools"]["cargo-root"], "cargo", "bin",
                               "cargo" + BIN_SUFFIX)
        cargo_binary_exists = path.exists(cargo_path)

        if not self.config["tools"]["system-cargo"] and not cargo_binary_exists:
            Registrar.dispatch("bootstrap-cargo", context=self.context)

        self.context.bootstrapped = True

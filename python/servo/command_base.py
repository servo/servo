# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import annotations

import contextlib
import errno
import json
import pathlib
from enum import Enum
from typing import Dict, List, Optional
import functools
import gzip
import itertools
import locale
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import urllib
import zipfile

from dataclasses import dataclass
from errno import ENOENT as NO_SUCH_FILE_OR_DIRECTORY
from glob import glob
from os import path
from subprocess import PIPE
from xml.etree.ElementTree import XML
from packaging.version import parse as parse_version

import toml

from mach.decorators import CommandArgument, CommandArgumentGroup
from mach.registrar import Registrar

import servo.platform
import servo.util as util
from servo.util import download_file, get_default_cache_dir

NIGHTLY_REPOSITORY_URL = "https://servo-builds2.s3.amazonaws.com/"
ASAN_LEAK_SUPPRESSION_FILE = "support/suppressed_leaks_for_asan.txt"


@dataclass
class BuildType:
    class Kind(Enum):
        DEV = 1
        RELEASE = 2
        CUSTOM = 3

    kind: Kind
    profile: Optional[str]

    def dev() -> BuildType:
        return BuildType(BuildType.Kind.DEV, None)

    def release() -> BuildType:
        return BuildType(BuildType.Kind.RELEASE, None)

    def prod() -> BuildType:
        return BuildType(BuildType.Kind.CUSTOM, "production")

    def custom(profile: str) -> BuildType:
        return BuildType(BuildType.Kind.CUSTOM, profile)

    def is_dev(self) -> bool:
        return self.kind == BuildType.Kind.DEV

    def is_release(self) -> bool:
        return self.kind == BuildType.Kind.RELEASE

    def is_prod(self) -> bool:
        return self.kind == BuildType.Kind.CUSTOM and self.profile == "production"

    def is_custom(self) -> bool:
        return self.kind == BuildType.Kind.CUSTOM

    def directory_name(self) -> str:
        if self.is_dev():
            return "debug"
        elif self.is_release():
            return "release"
        else:
            return self.profile

    def __eq__(self, other: object) -> bool:
        raise Exception("BUG: do not compare BuildType with ==")


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
        file_list = []
        for root, dirs, files in os.walk(current_dir):
            if dest_archive.endswith(".zip"):
                for f in files:
                    file_list.append(os.path.join(root, f))
            else:
                for name in itertools.chain(dirs, files):
                    file_list.append(os.path.join(root, name))

        # Sort file entries with the fixed locale
        with setlocale('C'):
            file_list.sort(key=functools.cmp_to_key(locale.strcoll))

        # Use a temporary file and atomic rename to avoid partially-formed
        # packaging (in case of exceptional situations like running out of disk space).
        # TODO do this in a temporary folder after #11983 is fixed
        temp_file = '{}.temp~'.format(dest_archive)
        with os.fdopen(os.open(temp_file, os.O_WRONLY | os.O_CREAT, 0o644), 'wb') as out_file:
            if dest_archive.endswith('.zip'):
                with zipfile.ZipFile(temp_file, 'w', zipfile.ZIP_DEFLATED) as zip_file:
                    for entry in file_list:
                        arcname = entry
                        if prepend_path is not None:
                            arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                        zip_file.write(entry, arcname=arcname)
            else:
                with gzip.GzipFile(mode='wb', fileobj=out_file, mtime=0) as gzip_file:
                    with tarfile.open(fileobj=gzip_file, mode='w:') as tar_file:
                        for entry in file_list:
                            arcname = entry
                            if prepend_path is not None:
                                arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                            tar_file.add(entry, filter=reset, recursive=False, arcname=arcname)
        os.rename(temp_file, dest_archive)


def call(*args, **kwargs):
    """Wrap `subprocess.call`, printing the command if verbose=True."""
    verbose = kwargs.pop('verbose', False)
    if verbose:
        print(' '.join(args[0]))
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    return subprocess.call(*args, shell=sys.platform == 'win32', **kwargs)


def check_output(*args, **kwargs) -> bytes:
    """Wrap `subprocess.call`, printing the command if verbose=True."""
    verbose = kwargs.pop('verbose', False)
    if verbose:
        print(' '.join(args[0]))
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    return subprocess.check_output(*args, shell=sys.platform == 'win32', **kwargs)


def check_call(*args, **kwargs):
    """Wrap `subprocess.check_call`, printing the command if verbose=True.

    Also fix any unicode-containing `env`, for subprocess """
    verbose = kwargs.pop('verbose', False)

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
    return sys.platform == 'win32'


def is_macosx():
    return sys.platform == 'darwin'


def is_linux():
    return sys.platform.startswith('linux')


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
        self.enable_media = False
        self.features = []
        self.cross_compile_target = None
        self.is_android_build = False
        self.target_path = util.get_target_dir()

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
            with open(config_path, "r", encoding="utf-8") as f:
                self.config = toml.loads(f.read())
        else:
            self.config = {}

        # Handle missing/default items
        self.config.setdefault("tools", {})
        self.config["tools"].setdefault("cache-dir", get_default_cache_dir(context.topdir))
        resolverelative("tools", "cache-dir")

        default_cargo_home = os.environ.get("CARGO_HOME",
                                            path.join(context.topdir, ".cargo"))
        self.config["tools"].setdefault("cargo-home-dir", default_cargo_home)
        resolverelative("tools", "cargo-home-dir")

        context.sharedir = self.config["tools"]["cache-dir"]

        self.config["tools"].setdefault("rustc-with-gold", get_env_bool("SERVO_RUSTC_WITH_GOLD", True))

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)
        self.config["build"].setdefault("mode", "")
        self.config["build"].setdefault("debug-assertions", False)
        self.config["build"].setdefault("debug-mozjs", False)
        self.config["build"].setdefault("media-stack", "auto")
        self.config["build"].setdefault("ccache", "")
        self.config["build"].setdefault("rustflags", "")
        self.config["build"].setdefault("incremental", None)
        self.config["build"].setdefault("webgl-backtrace", False)
        self.config["build"].setdefault("dom-backtrace", False)

        self.config.setdefault("android", {})
        self.config["android"].setdefault("sdk", "")
        self.config["android"].setdefault("ndk", "")
        self.config["android"].setdefault("toolchain", "")

        self.config.setdefault("ohos", {})
        self.config["ohos"].setdefault("ndk", "")

        # Set default android target
        self.setup_configuration_for_android_target("armv7-linux-androideabi")

    _rust_toolchain = None

    def rust_toolchain(self):
        if self._rust_toolchain:
            return self._rust_toolchain

        toolchain_file = path.join(self.context.topdir, "rust-toolchain.toml")
        self._rust_toolchain = toml.load(toolchain_file)['toolchain']['channel']
        return self._rust_toolchain

    def get_top_dir(self):
        return self.context.topdir

    def get_apk_path(self, build_type: BuildType):
        base_path = util.get_target_dir()
        base_path = path.join(base_path, "android", self.config["android"]["target"])
        apk_name = "servoapp.apk"
        return path.join(base_path, build_type.directory_name(), apk_name)

    def get_binary_path(self, build_type: BuildType, target=None, android=False, asan=False):
        if target is None and asan:
            target = servo.platform.host_triple()

        base_path = util.get_target_dir()

        if android:
            base_path = path.join(base_path, self.config["android"]["target"])
        elif target:
            base_path = path.join(base_path, target)
        if android or (target is not None and "-ohos" in target):
            return path.join(base_path, build_type.directory_name(), "libservoshell.so")

        binary_name = f"servo{servo.platform.get().executable_suffix()}"
        binary_path = path.join(base_path, build_type.directory_name(), binary_name)

        if not path.exists(binary_path):
            if target is None:
                print("WARNING: Fallback to host-triplet prefixed target dirctory for binary path.")
                return self.get_binary_path(build_type, target=servo.platform.host_triple(), android=android)
            else:
                raise BuildNotFound('No Servo binary found. Perhaps you forgot to run `./mach build`?')
        return binary_path

    def detach_volume(self, mounted_volume):
        print("Detaching volume {}".format(mounted_volume))
        try:
            subprocess.check_call(['hdiutil', 'detach', mounted_volume])
        except subprocess.CalledProcessError as e:
            print("Could not detach volume {} : {}".format(mounted_volume, e.returncode))
            sys.exit(1)

    def detach_volume_if_attached(self, mounted_volume):
        if os.path.exists(mounted_volume):
            self.detach_volume(mounted_volume)

    def mount_dmg(self, dmg_path):
        print("Mounting dmg {}".format(dmg_path))
        try:
            subprocess.check_call(['hdiutil', 'attach', dmg_path])
        except subprocess.CalledProcessError as e:
            print("Could not mount Servo dmg : {}".format(e.returncode))
            sys.exit(1)

    def extract_nightly(self, nightlies_folder, destination_folder, destination_file):
        print("Extracting to {} ...".format(destination_folder))
        if is_macosx():
            mounted_volume = path.join(path.sep, "Volumes", "Servo")
            self.detach_volume_if_attached(mounted_volume)
            self.mount_dmg(destination_file)
            # Servo folder is always this one
            servo_directory = path.join(path.sep, "Volumes", "Servo", "Servo.app", "Contents", "MacOS")
            print("Copying files from {} to {}".format(servo_directory, destination_folder))
            shutil.copytree(servo_directory, destination_folder)
            self.detach_volume(mounted_volume)
        else:
            if is_windows():
                command = 'msiexec /a {} /qn TARGETDIR={}'.format(
                    os.path.join(nightlies_folder, destination_file), destination_folder)
                if subprocess.call(command, stdout=PIPE, stderr=PIPE) != 0:
                    print("Could not extract the nightly executable from the msi package.")
                    sys.exit(1)
            else:
                with tarfile.open(os.path.join(nightlies_folder, destination_file), "r") as tar:
                    tar.extractall(destination_folder)

    def get_executable(self, destination_folder):
        if is_windows():
            return path.join(destination_folder, "PFiles", "Mozilla research", "Servo Tech Demo")
        if is_linux:
            return path.join(destination_folder, "servo", "servo")
        return path.join(destination_folder, "servo")

    def get_nightly_binary_path(self, nightly_date):
        if nightly_date is None:
            return
        if not nightly_date:
            print(
                "No nightly date has been provided although the --nightly or -n flag has been passed.")
            sys.exit(1)
        # Will alow us to fetch the relevant builds from the nightly repository
        os_prefix = "linux"
        if is_windows():
            os_prefix = "windows-msvc"
        if is_macosx():
            os_prefix = "mac"
        nightly_date = nightly_date.strip()
        # Fetch the filename to download from the build list
        repository_index = NIGHTLY_REPOSITORY_URL + "?list-type=2&prefix=nightly"
        req = urllib.request.Request(
            "{}/{}/{}".format(repository_index, os_prefix, nightly_date))
        try:
            response = urllib.request.urlopen(req).read()
            tree = XML(response)
            namespaces = {'ns': tree.tag[1:tree.tag.index('}')]}
            file_to_download = tree.find('ns:Contents', namespaces).find(
                'ns:Key', namespaces).text
        except urllib.error.URLError as e:
            print("Could not fetch the available nightly versions from the repository : {}".format(
                e.reason))
            sys.exit(1)
        except AttributeError:
            print("Could not fetch a nightly version for date {} and platform {}".format(
                nightly_date, os_prefix))
            sys.exit(1)

        nightly_target_directory = path.join(self.context.topdir, "target")
        # ':' is not an authorized character for a file name on Windows
        # make sure the OS specific separator is used
        target_file_path = file_to_download.replace(':', '-').split('/')
        destination_file = os.path.join(
            nightly_target_directory, os.path.join(*target_file_path))
        # Once extracted, the nightly folder name is the tar name without the extension
        # (eg /foo/bar/baz.tar.gz extracts to /foo/bar/baz)
        destination_folder = os.path.splitext(destination_file)[0]
        nightlies_folder = path.join(
            nightly_target_directory, 'nightly', os_prefix)

        # Make sure the target directory exists
        if not os.path.isdir(nightlies_folder):
            print("The nightly folder for the target does not exist yet. Creating {}".format(
                nightlies_folder))
            os.makedirs(nightlies_folder)

        # Download the nightly version
        if os.path.isfile(path.join(nightlies_folder, destination_file)):
            print("The nightly file {} has already been downloaded.".format(
                destination_file))
        else:
            print("The nightly {} does not exist yet, downloading it.".format(
                destination_file))
            download_file(destination_file, NIGHTLY_REPOSITORY_URL
                          + file_to_download, destination_file)

        # Extract the downloaded nightly version
        if os.path.isdir(destination_folder):
            print("The nightly folder {} has already been extracted.".format(
                destination_folder))
        else:
            self.extract_nightly(nightlies_folder, destination_folder, destination_file)

        return self.get_executable(destination_folder)

    def msvc_package_dir(self, package):
        return servo.platform.windows.get_dependency_dir(package)

    def build_env(self):
        """Return an extended environment dictionary."""
        env = os.environ.copy()

        # If we are installing on MacOS and Windows, we need to make sure that GStreamer's
        # `pkg-config` is on the path and takes precedence over other `pkg-config`s.
        if self.enable_media and not self.is_android_build:
            platform = servo.platform.get()
            gstreamer_root = platform.gstreamer_root(cross_compilation_target=self.cross_compile_target)
            if gstreamer_root:
                util.prepend_paths_to_env(env, "PATH", os.path.join(gstreamer_root, "bin"))

                # FIXME: This is necessary to run unit tests, because they depend on dylibs from the
                # GStreamer distribution (such as harfbuzz), but we only modify the rpath of the
                # target binary (servoshell / libsimpleservo).
                if platform.is_macos:
                    util.prepend_paths_to_env(env, "DYLD_LIBRARY_PATH", os.path.join(gstreamer_root, "lib"))

        if sys.platform != "win32":
            env.setdefault("CC", "clang")
            env.setdefault("CXX", "clang++")
        else:
            env.setdefault("CC", "clang-cl.exe")
            env.setdefault("CXX", "clang-cl.exe")

        if self.config["build"]["incremental"]:
            env["CARGO_INCREMENTAL"] = "1"
        elif self.config["build"]["incremental"] is not None:
            env["CARGO_INCREMENTAL"] = "0"

        env['RUSTFLAGS'] = env.get('RUSTFLAGS', "")

        if self.config["build"]["rustflags"]:
            env['RUSTFLAGS'] += " " + self.config["build"]["rustflags"]

        if not (self.config["build"]["ccache"] == ""):
            env['CCACHE'] = self.config["build"]["ccache"]

        env["CARGO_TARGET_DIR"] = servo.util.get_target_dir()

        # Work around https://github.com/servo/servo/issues/24446
        # Argument-less str.split normalizes leading, trailing, and double spaces
        env['RUSTFLAGS'] = " ".join(env['RUSTFLAGS'].split())

        # Suppress known false-positives during memory leak sanitizing.
        env["LSAN_OPTIONS"] = f"{env.get('LSAN_OPTIONS', '')}:suppressions={ASAN_LEAK_SUPPRESSION_FILE}"

        self.build_android_env_if_needed(env)
        self.build_ohos_env_if_needed(env)

        return env

    def build_android_env_if_needed(self, env: Dict[str, str]):
        if not self.is_android_build:
            return

        # Paths to Android build tools:
        if self.config["android"]["sdk"]:
            env["ANDROID_SDK_ROOT"] = self.config["android"]["sdk"]
        if self.config["android"]["ndk"]:
            env["ANDROID_NDK_ROOT"] = self.config["android"]["ndk"]

        toolchains = path.join(self.context.topdir, "android-toolchains")
        for kind in ["sdk", "ndk"]:
            default = os.path.join(toolchains, kind)
            if os.path.isdir(default):
                env.setdefault(f"ANDROID_{kind.upper()}_ROOT", default)

        if "IN_NIX_SHELL" in env and ("ANDROID_NDK_ROOT" not in env or "ANDROID_SDK_ROOT" not in env):
            print("Please set SERVO_ANDROID_BUILD=1 when starting the Nix shell to include the Android SDK/NDK.")
            sys.exit(1)
        if "ANDROID_NDK_ROOT" not in env:
            print("Please set the ANDROID_NDK_ROOT environment variable.")
            sys.exit(1)
        if "ANDROID_SDK_ROOT" not in env:
            print("Please set the ANDROID_SDK_ROOT environment variable.")
            sys.exit(1)

        android_platform = self.config["android"]["platform"]
        android_toolchain_name = self.config["android"]["toolchain_name"]
        android_lib = self.config["android"]["lib"]

        android_api = android_platform.replace('android-', '')

        # Check if the NDK version is 25
        if not os.path.isfile(path.join(env["ANDROID_NDK_ROOT"], 'source.properties')):
            print("ANDROID_NDK should have file `source.properties`.")
            print("The environment variable ANDROID_NDK_ROOT may be set at a wrong path.")
            sys.exit(1)
        with open(path.join(env["ANDROID_NDK_ROOT"], 'source.properties'), encoding="utf8") as ndk_properties:
            lines = ndk_properties.readlines()
            if lines[1].split(' = ')[1].split('.')[0] != '25':
                print("Servo currently only supports NDK r25c.")
                sys.exit(1)

        # Android builds also require having the gcc bits on the PATH and various INCLUDE
        # path munging if you do not want to install a standalone NDK. See:
        # https://dxr.mozilla.org/mozilla-central/source/build/autoconf/android.m4#139-161
        os_type = platform.system().lower()
        if os_type not in ["linux", "darwin"]:
            raise Exception("Android cross builds are only supported on Linux and macOS.")

        cpu_type = platform.machine().lower()
        host_suffix = "unknown"
        if cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
            host_suffix = "x86"
        elif cpu_type in ["x86_64", "x86-64", "x64", "amd64"]:
            host_suffix = "x86_64"
        host = os_type + "-" + host_suffix

        host_cc = env.get('HOST_CC') or shutil.which("clang")
        host_cxx = env.get('HOST_CXX') or shutil.which("clang++")

        llvm_toolchain = path.join(env['ANDROID_NDK_ROOT'], "toolchains", "llvm", "prebuilt", host)
        env['PATH'] = (env['PATH'] + ':' + path.join(llvm_toolchain, "bin"))

        def to_ndk_bin(prog):
            return path.join(llvm_toolchain, "bin", prog)

        # This workaround is due to an issue in the x86_64 Android NDK that introduces
        # an undefined reference to the symbol '__extendsftf2'.
        # See https://github.com/termux/termux-packages/issues/8029#issuecomment-1369150244
        if "x86_64" in self.cross_compile_target:
            libclangrt_filename = subprocess.run(
                [to_ndk_bin(f"x86_64-linux-android{android_api}-clang"), "--print-libgcc-file-name"],
                check=True,
                capture_output=True,
                encoding="utf8"
            ).stdout
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "")
            env["RUSTFLAGS"] += f"-C link-arg={libclangrt_filename}"

        env["RUST_TARGET"] = self.cross_compile_target
        env['HOST_CC'] = host_cc
        env['HOST_CXX'] = host_cxx
        env['HOST_CFLAGS'] = ''
        env['HOST_CXXFLAGS'] = ''
        env['TARGET_CC'] = to_ndk_bin("clang")
        env['TARGET_CPP'] = to_ndk_bin("clang") + " -E"
        env['TARGET_CXX'] = to_ndk_bin("clang++")

        env['TARGET_AR'] = to_ndk_bin("llvm-ar")
        env['TARGET_RANLIB'] = to_ndk_bin("llvm-ranlib")
        env['TARGET_OBJCOPY'] = to_ndk_bin("llvm-objcopy")
        env['TARGET_YASM'] = to_ndk_bin("yasm")
        env['TARGET_STRIP'] = to_ndk_bin("llvm-strip")
        env['RUST_FONTCONFIG_DLOPEN'] = "on"

        env["LIBCLANG_PATH"] = path.join(llvm_toolchain, "lib64")
        env["CLANG_PATH"] = to_ndk_bin("clang")

        # A cheat-sheet for some of the build errors caused by getting the search path wrong...
        #
        # fatal error: 'limits' file not found
        #   -- add -I cxx_include
        # unknown type name '__locale_t' (when running bindgen in mozjs_sys)
        #   -- add -isystem sysroot_include
        # error: use of undeclared identifier 'UINTMAX_C'
        #   -- add -D__STDC_CONSTANT_MACROS
        #
        # Also worth remembering: autoconf uses C for its configuration,
        # even for C++ builds, so the C flags need to line up with the C++ flags.
        env['TARGET_CFLAGS'] = "--target=" + android_toolchain_name
        env['TARGET_CXXFLAGS'] = "--target=" + android_toolchain_name

        # These two variables are needed for the mozjs compilation.
        env['ANDROID_API_LEVEL'] = android_api
        env["ANDROID_NDK_HOME"] = env["ANDROID_NDK_ROOT"]

        # The two variables set below are passed by our custom
        # support/android/toolchain.cmake to the NDK's CMake toolchain file
        env["ANDROID_ABI"] = android_lib
        env["ANDROID_PLATFORM"] = android_platform
        env["NDK_CMAKE_TOOLCHAIN_FILE"] = path.join(
            env['ANDROID_NDK_ROOT'], "build", "cmake", "android.toolchain.cmake")
        env["CMAKE_TOOLCHAIN_FILE"] = path.join(
            self.context.topdir, "support", "android", "toolchain.cmake")

        # Set output dir for gradle aar files
        env["AAR_OUT_DIR"] = path.join(self.context.topdir, "target", "android", "aar")
        if not os.path.exists(env['AAR_OUT_DIR']):
            os.makedirs(env['AAR_OUT_DIR'])

        env['TARGET_PKG_CONFIG_SYSROOT_DIR'] = path.join(llvm_toolchain, 'sysroot')

    def build_ohos_env_if_needed(self, env: Dict[str, str]):
        if not (self.cross_compile_target and self.cross_compile_target.endswith('-ohos')):
            return

        # Paths to OpenHarmony SDK and build tools:
        # Note: `OHOS_SDK_NATIVE` is the CMake variable name the `hvigor` build-system
        # uses for the native directory of the SDK, so we use the same name to be consistent.
        if "OHOS_SDK_NATIVE" not in env and self.config["ohos"]["ndk"]:
            env["OHOS_SDK_NATIVE"] = self.config["ohos"]["ndk"]

        if "OHOS_SDK_NATIVE" not in env:
            print("Please set the OHOS_SDK_NATIVE environment variable to the location of the `native` directory "
                  "in the OpenHarmony SDK.")
            sys.exit(1)

        ndk_root = pathlib.Path(env["OHOS_SDK_NATIVE"])

        if not ndk_root.is_dir():
            print(f"OHOS_SDK_NATIVE is not set to a valid directory: `{ndk_root}`")
            sys.exit(1)

        ndk_root = ndk_root.resolve()
        package_info = ndk_root.joinpath("oh-uni-package.json")
        try:
            with open(package_info) as meta_file:
                meta = json.load(meta_file)
            ohos_api_version = int(meta['apiVersion'])
            ohos_sdk_version = parse_version(meta['version'])
            if ohos_sdk_version < parse_version('4.0'):
                print("Warning: mach build currently assumes at least the OpenHarmony 4.0 SDK is used.")
            print(f"Info: The OpenHarmony SDK {ohos_sdk_version} is targeting API-level {ohos_api_version}")
        except Exception as e:
            print(f"Failed to read metadata information from {package_info}")
            print(f"Exception: {e}")

        # The OpenHarmony SDK for Windows hosts currently does not contain a libclang shared library,
        # which is required by `bindgen` (see issue
        # https://gitee.com/openharmony/third_party_llvm-project/issues/I8H50W). Using upstream `clang` is currently
        # also not easily possible, since `libcxx` support still needs to be upstreamed (
        # https://github.com/llvm/llvm-project/pull/73114).
        os_type = platform.system().lower()
        if os_type not in ["linux", "darwin"]:
            raise Exception("OpenHarmony builds are currently only supported on Linux and macOS Hosts.")

        llvm_toolchain = ndk_root.joinpath("llvm")
        llvm_bin = llvm_toolchain.joinpath("bin")
        ohos_sysroot = ndk_root.joinpath("sysroot")
        if not (llvm_toolchain.is_dir() and llvm_bin.is_dir()):
            print(f"Expected to find `llvm` and `llvm/bin` folder under $OHOS_SDK_NATIVE at `{llvm_toolchain}`")
            sys.exit(1)
        if not ohos_sysroot.is_dir():
            print(f"Could not find OpenHarmony sysroot in {ndk_root}")
            sys.exit(1)

        # Note: We don't use the `<target_triple>-clang` wrappers on purpose, since
        # a) the OH 4.0 SDK does not have them yet AND
        # b) the wrappers in the newer SDKs are bash scripts, which can cause problems
        # on windows, depending on how the wrapper is called.
        # Instead, we ensure that all the necessary flags for the c-compiler are set
        # via environment variables such as `TARGET_CFLAGS`.
        def to_sdk_llvm_bin(prog: str):
            if is_windows():
                prog = prog + '.exe'
            llvm_prog = llvm_bin.joinpath(prog)
            if not llvm_prog.is_file():
                raise FileNotFoundError(errno.ENOENT, os.strerror(errno.ENOENT), llvm_prog)
            return str(llvm_bin.joinpath(prog))

        # CC and CXX should already be set to appropriate host compilers by `build_env()`
        env['HOST_CC'] = env['CC']
        env['HOST_CXX'] = env['CXX']
        env['TARGET_AR'] = to_sdk_llvm_bin("llvm-ar")
        env['TARGET_RANLIB'] = to_sdk_llvm_bin("llvm-ranlib")
        env['TARGET_READELF'] = to_sdk_llvm_bin("llvm-readelf")
        env['TARGET_OBJCOPY'] = to_sdk_llvm_bin("llvm-objcopy")
        env['TARGET_STRIP'] = to_sdk_llvm_bin("llvm-strip")

        rust_target_triple = str(self.cross_compile_target).replace('-', '_')
        ndk_clang = to_sdk_llvm_bin("clang")
        ndk_clangxx = to_sdk_llvm_bin("clang++")
        env[f'CC_{rust_target_triple}'] = ndk_clang
        env[f'CXX_{rust_target_triple}'] = ndk_clangxx
        # The clang target name is different from the LLVM target name
        clang_target_triple = str(self.cross_compile_target).replace('-unknown-', '-')
        clang_target_triple_underscore = clang_target_triple.replace('-', '_')
        env[f'CC_{clang_target_triple_underscore}'] = ndk_clang
        env[f'CXX_{clang_target_triple_underscore}'] = ndk_clangxx
        # rustc linker
        env[f'CARGO_TARGET_{rust_target_triple.upper()}_LINKER'] = ndk_clang
        # We could also use a cross-compile wrapper
        env["RUSTFLAGS"] += f' -Clink-arg=--target={clang_target_triple}'
        env["RUSTFLAGS"] += f' -Clink-arg=--sysroot={ohos_sysroot}'

        env['HOST_CFLAGS'] = ''
        env['HOST_CXXFLAGS'] = ''
        ohos_cflags = ['-D__MUSL__', f' --target={clang_target_triple}', f' --sysroot={ohos_sysroot}']
        if clang_target_triple.startswith('armv7-'):
            ohos_cflags.extend(['-march=armv7-a', '-mfloat-abi=softfp', '-mtune=generic-armv7-a', '-mthumb'])
        ohos_cflags_str = " ".join(ohos_cflags)
        env['TARGET_CFLAGS'] = ohos_cflags_str
        env['TARGET_CPPFLAGS'] = '-D__MUSL__'
        env['TARGET_CXXFLAGS'] = ohos_cflags_str

        # CMake related flags
        cmake_toolchain_file = ndk_root.joinpath("build", "cmake", "ohos.toolchain.cmake")
        if cmake_toolchain_file.is_file():
            env[f'CMAKE_TOOLCHAIN_FILE_{rust_target_triple}'] = str(cmake_toolchain_file)
        else:
            print(
                f"Warning: Failed to find the OpenHarmony CMake Toolchain file - Expected it at {cmake_toolchain_file}")
        env[f'CMAKE_C_COMPILER_{rust_target_triple}'] = ndk_clang
        env[f'CMAKE_CXX_COMPILER_{rust_target_triple}'] = ndk_clangxx

        # pkg-config
        pkg_config_path = '{}:{}'.format(str(ohos_sysroot.joinpath("usr", "lib", "pkgconfig")),
                                         str(ohos_sysroot.joinpath("usr", "share", "pkgconfig")))
        env[f'PKG_CONFIG_SYSROOT_DIR_{rust_target_triple}'] = str(ohos_sysroot)
        env[f'PKG_CONFIG_PATH_{rust_target_triple}'] = pkg_config_path

        # bindgen / libclang-sys
        env["LIBCLANG_PATH"] = path.join(llvm_toolchain, "lib")
        env["CLANG_PATH"] = ndk_clangxx
        env[f'CXXSTDLIB_{clang_target_triple_underscore}'] = "c++"
        bindgen_extra_clangs_args_var = f'BINDGEN_EXTRA_CLANG_ARGS_{rust_target_triple}'
        bindgen_extra_clangs_args = env.get(bindgen_extra_clangs_args_var, "")
        bindgen_extra_clangs_args = bindgen_extra_clangs_args + " " + ohos_cflags_str
        env[bindgen_extra_clangs_args_var] = bindgen_extra_clangs_args

    @staticmethod
    def common_command_arguments(build_configuration=False, build_type=False):
        decorators = []
        if build_type:
            decorators += [
                CommandArgumentGroup('Build Type'),
                CommandArgument('--release', '-r', group="Build Type",
                                action='store_true',
                                help='Build in release mode'),
                CommandArgument('--dev', '--debug', '-d', group="Build Type",
                                action='store_true',
                                help='Build in development mode'),
                CommandArgument('--prod', '--production', group="Build Type",
                                action='store_true',
                                help='Build in release mode without debug assertions'),
                CommandArgument('--profile', group="Build Type",
                                help='Build with custom Cargo profile'),
                CommandArgument('--with-asan', action='store_true', help="Build with AddressSanitizer")
            ]

        if build_configuration:
            decorators += [
                CommandArgumentGroup('Cross Compilation'),
                CommandArgument(
                    '--target', '-t',
                    group="Cross Compilation",
                    default=None,
                    help='Cross compile for given target platform',
                ),
                CommandArgument(
                    '--android', default=None, action='store_true',
                    help='Build for Android. If --target is not specified, this '
                         'will choose a default target architecture.',
                ),
                CommandArgument('--win-arm64', action='store_true', help="Use arm64 Windows target"),
                CommandArgumentGroup('Feature Selection'),
                CommandArgument(
                    '--features', default=None, group="Feature Selection", nargs='+',
                    help='Space-separated list of features to also build',
                ),
                CommandArgument(
                    '--media-stack', default=None, group="Feature Selection",
                    choices=["gstreamer", "dummy"], help='Which media stack to use',
                ),
                CommandArgument(
                    '--debug-mozjs',
                    default=False,
                    group="Feature Selection",
                    action='store_true',
                    help='Enable debug assertions in mozjs',
                ),
                CommandArgument(
                    '--with-debug-assertions',
                    default=False,
                    group="Feature Selection",
                    action='store_true',
                    help='Enable debug assertions in release',
                ),
                CommandArgument(
                    '--with-frame-pointer',
                    default=None, group="Feature Selection",
                    action='store_true',
                    help='Build with frame pointer enabled, used by the background hang monitor.',
                ),
                CommandArgument(
                    '--use-crown',
                    default=False,
                    action='store_true',
                    help="Enable Servo's `crown` linter tool"
                )
            ]

        def decorator_function(original_function):
            def configuration_decorator(self, *args, **kwargs):
                if build_type:
                    # If `build_type` already exists in kwargs we are doing a recursive dispatch.
                    if 'build_type' not in kwargs:
                        kwargs['build_type'] = self.configure_build_type(
                            kwargs['release'], kwargs['dev'], kwargs['prod'], kwargs['profile'],
                        )
                    kwargs.pop('release', None)
                    kwargs.pop('dev', None)
                    kwargs.pop('prod', None)
                    kwargs.pop('profile', None)

                if build_configuration:
                    self.configure_cross_compilation(kwargs['target'], kwargs['android'], kwargs['win_arm64'])
                    self.features = kwargs.get("features", None) or []
                    self.enable_media = self.is_media_enabled(kwargs['media_stack'])

                return original_function(self, *args, **kwargs)

            decorators.reverse()
            for decorator in decorators:
                decorator(configuration_decorator)

            return configuration_decorator

        return decorator_function

    def configure_build_type(self, release: bool, dev: bool, prod: bool, profile: Optional[str]) -> BuildType:
        option_count = release + dev + prod + (profile is not None)

        if option_count > 1:
            print("Please specify either --dev (-d) for a development")
            print("  build, or --release (-r) for an optimized build,")
            print("  or --profile PROFILE for a custom Cargo profile.")
            sys.exit(1)
        elif option_count < 1:
            if self.config["build"]["mode"] == "dev":
                print("No build type specified, but .servobuild specified `--dev`.")
                return BuildType.dev()
            elif self.config["build"]["mode"] == "release":
                print("No build type specified, but .servobuild specified `--release`.")
                return BuildType.release()
            else:
                print("No build type specified so assuming `--dev`.")
                return BuildType.dev()

        if release:
            return BuildType.release()
        elif dev:
            return BuildType.dev()
        elif prod:
            return BuildType.prod()
        else:
            return BuildType.custom(profile)

    def configure_cross_compilation(
            self,
            cross_compile_target: Optional[str],
            android: Optional[str],
            win_arm64: Optional[str]):
        # Force the UWP-enabled target if the convenience UWP flags are passed.
        if android is None:
            android = self.config["build"]["android"]
        if android:
            if not cross_compile_target:
                cross_compile_target = self.config["android"]["target"]
            assert cross_compile_target
            assert self.setup_configuration_for_android_target(cross_compile_target)
        elif cross_compile_target:
            # If a target was specified, it might also be an android target,
            # so set up the configuration in that case.
            self.setup_configuration_for_android_target(cross_compile_target)

        self.cross_compile_target = cross_compile_target
        self.is_android_build = (cross_compile_target and "android" in cross_compile_target)
        self.target_path = servo.util.get_target_dir()
        if self.is_android_build:
            assert self.cross_compile_target
            self.target_path = path.join(self.target_path, "android", self.cross_compile_target)

        if self.cross_compile_target:
            print(f"Targeting '{self.cross_compile_target}' for cross-compilation")

    def is_media_enabled(self, media_stack: Optional[str]):
        """Determine whether media is enabled based on the value of the build target
           platform and the value of the '--media-stack' command-line argument.
           Returns true if media is enabled."""
        if not media_stack:
            if self.config["build"]["media-stack"] != "auto":
                media_stack = self.config["build"]["media-stack"]
                assert media_stack
            elif not self.cross_compile_target:
                media_stack = "gstreamer"
            else:
                media_stack = "dummy"

        # This is a workaround for Ubuntu 20.04, which doesn't support a new enough GStreamer.
        # Once we drop support for this platform (it's currently needed for wpt.fyi runners),
        # we can remove this workaround and officially only support Ubuntu 22.04 and up.
        platform = servo.platform.get()
        if not self.cross_compile_target and platform.is_linux and \
                not platform.is_gstreamer_installed(self.cross_compile_target):
            return False

        return media_stack != "dummy"

    def run_cargo_build_like_command(
        self, command: str, cargo_args: List[str],
        env=None, verbose=False,
        debug_mozjs=False, with_debug_assertions=False,
        with_frame_pointer=False,
        use_crown=False,
        target_override: Optional[str] = None,
        **_kwargs
    ):
        env = env or self.build_env()

        # Android GStreamer integration is handled elsewhere.
        # NB: On non-Linux platforms we cannot check whether GStreamer is installed until
        # environment variables are set via `self.build_env()`.
        platform = servo.platform.get()
        if self.enable_media and not self.is_android_build and \
                not platform.is_gstreamer_installed(self.cross_compile_target):
            raise FileNotFoundError(
                "GStreamer libraries not found (>= version 1.18)."
                "Please see installation instructions in README.md"
            )

        args = []
        if "--manifest-path" not in cargo_args:
            args += [
                "--manifest-path",
                path.join(self.context.topdir, "ports", "servoshell", "Cargo.toml"),
            ]
        if target_override:
            args += ["--target", target_override]
        elif self.cross_compile_target:
            args += ["--target", self.cross_compile_target]
            if self.is_android_build or '-ohos' in self.cross_compile_target:
                # Note: in practice `cargo rustc` should just be used unconditionally.
                assert command != 'build', "For Android / OpenHarmony `cargo rustc` must be used instead of cargo build"
                if command == 'rustc':
                    args += ["--lib", "--crate-type=cdylib"]

        if use_crown:
            if 'CARGO_BUILD_RUSTC' in env:
                current_rustc = env['CARGO_BUILD_RUSTC']
                if current_rustc != 'crown':
                    print('Error: `mach` was called with `--use-crown` while `CARGO_BUILD_RUSTC` was'
                          f'already set to `{current_rustc}` in the parent environment.\n'
                          'These options conflict, please specify only one of them.')
                    sys.exit(1)
            env['CARGO_BUILD_RUSTC'] = 'crown'
            # Changing `RUSTC` or `CARGO_BUILD_RUSTC` does not cause `cargo check` to
            # recheck files with the new compiler. `cargo build` is not affected and
            # triggers a rebuild as expected. To also make `check` work as expected,
            # we add a dummy `cfg` to RUSTFLAGS when using crown, so as to have different
            # RUSTFLAGS when using `crown`, to reliably trigger re-checking.
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " --cfg=crown"

        if "-p" not in cargo_args:  # We're building specific package, that may not have features
            features = list(self.features)
            if self.enable_media:
                features.append("media-gstreamer")
            if self.config["build"]["debug-mozjs"] or debug_mozjs:
                features.append("debugmozjs")

            if with_frame_pointer:
                env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C force-frame-pointers=yes"
                features.append("profilemozjs")
            if self.config["build"]["webgl-backtrace"]:
                features.append("webgl-backtrace")
            if self.config["build"]["dom-backtrace"]:
                features.append("dom-backtrace")
            args += ["--features", " ".join(features)]

        if with_debug_assertions or self.config["build"]["debug-assertions"]:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C debug_assertions"

        return call(["cargo", command] + args + cargo_args, env=env, verbose=verbose)

    def android_adb_path(self, env):
        if "ANDROID_SDK_ROOT" in env:
            sdk_adb = path.join(env["ANDROID_SDK_ROOT"], "platform-tools", "adb")
            if path.exists(sdk_adb):
                return sdk_adb
        return "adb"

    def android_emulator_path(self, env):
        if "ANDROID_SDK_ROOT" in env:
            sdk_adb = path.join(env["ANDROID_SDK_ROOT"], "emulator", "emulator")
            if path.exists(sdk_adb):
                return sdk_adb
        return "emulator"

    def setup_configuration_for_android_target(self, target: str):
        """If cross-compilation targets Android, configure the Android
           build by writing the appropriate toolchain configuration values
           into the stored configuration."""
        if target == "armv7-linux-androideabi":
            self.config["android"]["platform"] = "android-30"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = "arm-linux-androideabi"
            self.config["android"]["arch"] = "arm"
            self.config["android"]["lib"] = "armeabi-v7a"
            self.config["android"]["toolchain_name"] = "armv7a-linux-androideabi30"
            return True
        elif target == "aarch64-linux-android":
            self.config["android"]["platform"] = "android-30"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = target
            self.config["android"]["arch"] = "arm64"
            self.config["android"]["lib"] = "arm64-v8a"
            self.config["android"]["toolchain_name"] = "aarch64-linux-androideabi30"
            return True
        elif target == "i686-linux-android":
            # https://github.com/jemalloc/jemalloc/issues/1279
            self.config["android"]["platform"] = "android-30"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = target
            self.config["android"]["arch"] = "x86"
            self.config["android"]["lib"] = "x86"
            self.config["android"]["toolchain_name"] = "i686-linux-android30"
            return True
        elif target == "x86_64-linux-android":
            self.config["android"]["platform"] = "android-30"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = target
            self.config["android"]["arch"] = "x86_64"
            self.config["android"]["lib"] = "x86_64"
            self.config["android"]["toolchain_name"] = "x86_64-linux-android30"
            return True
        return False

    def ensure_bootstrapped(self):
        if self.context.bootstrapped:
            return

        servo.platform.get().passive_bootstrap()

        needs_toolchain_install = self.cross_compile_target and \
            self.cross_compile_target not in \
            check_output(["rustup", "target", "list", "--installed"],
                         cwd=self.context.topdir).decode()
        if needs_toolchain_install:
            check_call(["rustup", "target", "add", self.cross_compile_target],
                       cwd=self.context.topdir)

        self.context.bootstrapped = True

    def ensure_rustup_version(self):
        try:
            version_line = subprocess.check_output(
                ["rustup" + servo.platform.get().executable_suffix(), "--version"],
                # Silence "info: This is the version for the rustup toolchain manager,
                # not the rustc compiler."
                stderr=open(os.devnull, "wb")
            )
        except OSError as e:
            if e.errno == NO_SUCH_FILE_OR_DIRECTORY:
                print("It looks like rustup is not installed. See instructions at "
                      "https://github.com/servo/servo/#setting-up-your-environment")
                print()
                sys.exit(1)
            raise
        version = tuple(map(int, re.match(br"rustup (\d+)\.(\d+)\.(\d+)", version_line).groups()))
        version_needed = (1, 23, 0)
        if version < version_needed:
            print("rustup is at version %s.%s.%s, Servo requires %s.%s.%s or more recent." % (version + version_needed))
            print("Try running 'rustup self update'.")
            sys.exit(1)

    def ensure_clobbered(self, target_dir=None):
        if target_dir is None:
            target_dir = util.get_target_dir()
        auto = True if os.environ.get('AUTOCLOBBER', False) else False
        src_clobber = os.path.join(self.context.topdir, 'CLOBBER')
        target_clobber = os.path.join(target_dir, 'CLOBBER')

        if not os.path.exists(target_dir):
            os.makedirs(target_dir)

        if not os.path.exists(target_clobber):
            # Simply touch the file.
            with open(target_clobber, 'a'):
                pass

        if auto:
            if os.path.getmtime(src_clobber) > os.path.getmtime(target_clobber):
                print('Automatically clobbering target directory: {}'.format(target_dir))

                try:
                    Registrar.dispatch("clean", context=self.context, verbose=True)
                    print('Successfully completed auto clobber.')
                except subprocess.CalledProcessError as error:
                    sys.exit(error)
            else:
                print("Clobber not needed.")

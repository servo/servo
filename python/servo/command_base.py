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
from enum import Enum
from typing import Any, Dict, List, Optional
import functools
import gzip
import itertools
import locale
import os
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

import toml

from mach.decorators import CommandArgument, CommandArgumentGroup
from mach.registrar import Registrar

from servo.platform.build_target import BuildTarget, AndroidTarget, OpenHarmonyTarget
from servo.util import download_file, get_default_cache_dir

import servo.platform
import servo.util as util

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
        for c in glob(package + "-*"):
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
        with setlocale("C"):
            file_list.sort(key=functools.cmp_to_key(locale.strcoll))

        # Use a temporary file and atomic rename to avoid partially-formed
        # packaging (in case of exceptional situations like running out of disk space).
        # TODO do this in a temporary folder after #11983 is fixed
        temp_file = "{}.temp~".format(dest_archive)
        with os.fdopen(os.open(temp_file, os.O_WRONLY | os.O_CREAT, 0o644), "wb") as out_file:
            if dest_archive.endswith(".zip"):
                with zipfile.ZipFile(temp_file, "w", zipfile.ZIP_DEFLATED) as zip_file:
                    for entry in file_list:
                        arcname = entry
                        if prepend_path is not None:
                            arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                        zip_file.write(entry, arcname=arcname)
            else:
                with gzip.GzipFile(mode="wb", fileobj=out_file, mtime=0) as gzip_file:
                    with tarfile.open(fileobj=gzip_file, mode="w:") as tar_file:
                        for entry in file_list:
                            arcname = entry
                            if prepend_path is not None:
                                arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                            tar_file.add(entry, filter=reset, recursive=False, arcname=arcname)
        os.rename(temp_file, dest_archive)


def call(*args, **kwargs):
    """Wrap `subprocess.call`, printing the command if verbose=True."""
    verbose = kwargs.pop("verbose", False)
    if verbose:
        print(" ".join(args[0]))
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    return subprocess.call(*args, shell=sys.platform == "win32", **kwargs)


def check_output(*args, **kwargs) -> bytes:
    """Wrap `subprocess.call`, printing the command if verbose=True."""
    verbose = kwargs.pop("verbose", False)
    if verbose:
        print(" ".join(args[0]))
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    return subprocess.check_output(*args, shell=sys.platform == "win32", **kwargs)


def check_call(*args, **kwargs):
    """Wrap `subprocess.check_call`, printing the command if verbose=True.

    Also fix any unicode-containing `env`, for subprocess"""
    verbose = kwargs.pop("verbose", False)

    if verbose:
        print(" ".join(args[0]))
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    proc = subprocess.Popen(*args, shell=sys.platform == "win32", **kwargs)
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
        raise subprocess.CalledProcessError(status, " ".join(*args))


def is_windows():
    return sys.platform == "win32"


def is_macosx():
    return sys.platform == "darwin"


def is_linux():
    return sys.platform.startswith("linux")


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

        # Default to native build target. This will later be overriden
        # by `configure_build_target`
        self.target = BuildTarget.from_triple(None)

        def get_env_bool(var, default):
            # Contents of env vars are strings by default. This returns the
            # boolean value of the specified environment variable, or the
            # speciried default if the var doesn't contain True or False
            return {"True": True, "False": False}.get(os.environ.get(var), default)

        def resolverelative(category, key):
            # Allow ~
            self.config[category][key] = path.expanduser(self.config[category][key])
            # Resolve relative paths
            self.config[category][key] = path.join(context.topdir, self.config[category][key])

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

        default_cargo_home = os.environ.get("CARGO_HOME", path.join(context.topdir, ".cargo"))
        self.config["tools"].setdefault("cargo-home-dir", default_cargo_home)
        resolverelative("tools", "cargo-home-dir")

        context.sharedir = self.config["tools"]["cache-dir"]

        self.config["tools"].setdefault("rustc-with-gold", get_env_bool("SERVO_RUSTC_WITH_GOLD", True))

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)
        self.config["build"].setdefault("ohos", False)
        self.config["build"].setdefault("mode", "")
        self.config["build"].setdefault("debug-assertions", False)
        self.config["build"].setdefault("debug-mozjs", False)
        self.config["build"].setdefault("media-stack", "auto")
        self.config["build"].setdefault("ccache", "")
        self.config["build"].setdefault("rustflags", "")
        self.config["build"].setdefault("incremental", None)
        self.config["build"].setdefault("webgl-backtrace", False)
        self.config["build"].setdefault("dom-backtrace", False)
        self.config["build"].setdefault("with_asan", False)

        self.config.setdefault("android", {})
        self.config["android"].setdefault("sdk", "")
        self.config["android"].setdefault("ndk", "")
        self.config["android"].setdefault("toolchain", "")

        self.config.setdefault("ohos", {})
        self.config["ohos"].setdefault("ndk", "")

    _rust_toolchain = None

    def rust_toolchain(self):
        if self._rust_toolchain:
            return self._rust_toolchain

        toolchain_file = path.join(self.context.topdir, "rust-toolchain.toml")
        self._rust_toolchain = toml.load(toolchain_file)["toolchain"]["channel"]
        return self._rust_toolchain

    def get_top_dir(self):
        return self.context.topdir

    def get_binary_path(self, build_type: BuildType, asan: bool = False):
        base_path = util.get_target_dir()
        if asan or self.target.is_cross_build():
            base_path = path.join(base_path, self.target.triple())
        binary_name = self.target.binary_name()
        binary_path = path.join(base_path, build_type.directory_name(), binary_name)

        if not path.exists(binary_path):
            raise BuildNotFound("No Servo binary found. Perhaps you forgot to run `./mach build`?")

        return binary_path

    def detach_volume(self, mounted_volume):
        print("Detaching volume {}".format(mounted_volume))
        try:
            subprocess.check_call(["hdiutil", "detach", mounted_volume])
        except subprocess.CalledProcessError as e:
            print("Could not detach volume {} : {}".format(mounted_volume, e.returncode))
            sys.exit(1)

    def detach_volume_if_attached(self, mounted_volume):
        if os.path.exists(mounted_volume):
            self.detach_volume(mounted_volume)

    def mount_dmg(self, dmg_path):
        print("Mounting dmg {}".format(dmg_path))
        try:
            subprocess.check_call(["hdiutil", "attach", dmg_path])
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
                command = "msiexec /a {} /qn TARGETDIR={}".format(
                    os.path.join(nightlies_folder, destination_file), destination_folder
                )
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
            print("No nightly date has been provided although the --nightly or -n flag has been passed.")
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
        req = urllib.request.Request("{}/{}/{}".format(repository_index, os_prefix, nightly_date))
        try:
            response = urllib.request.urlopen(req).read()
            tree = XML(response)
            namespaces = {"ns": tree.tag[1 : tree.tag.index("}")]}
            file_to_download = tree.find("ns:Contents", namespaces).find("ns:Key", namespaces).text
        except urllib.error.URLError as e:
            print("Could not fetch the available nightly versions from the repository : {}".format(e.reason))
            sys.exit(1)
        except AttributeError:
            print("Could not fetch a nightly version for date {} and platform {}".format(nightly_date, os_prefix))
            sys.exit(1)

        nightly_target_directory = path.join(self.context.topdir, "target")
        # ':' is not an authorized character for a file name on Windows
        # make sure the OS specific separator is used
        target_file_path = file_to_download.replace(":", "-").split("/")
        destination_file = os.path.join(nightly_target_directory, os.path.join(*target_file_path))
        # Once extracted, the nightly folder name is the tar name without the extension
        # (eg /foo/bar/baz.tar.gz extracts to /foo/bar/baz)
        destination_folder = os.path.splitext(destination_file)[0]
        nightlies_folder = path.join(nightly_target_directory, "nightly", os_prefix)

        # Make sure the target directory exists
        if not os.path.isdir(nightlies_folder):
            print("The nightly folder for the target does not exist yet. Creating {}".format(nightlies_folder))
            os.makedirs(nightlies_folder)

        # Download the nightly version
        if os.path.isfile(path.join(nightlies_folder, destination_file)):
            print("The nightly file {} has already been downloaded.".format(destination_file))
        else:
            print("The nightly {} does not exist yet, downloading it.".format(destination_file))
            download_file(destination_file, NIGHTLY_REPOSITORY_URL + file_to_download, destination_file)

        # Extract the downloaded nightly version
        if os.path.isdir(destination_folder):
            print("The nightly folder {} has already been extracted.".format(destination_folder))
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
        if self.enable_media:
            platform = servo.platform.get()
            gstreamer_root = platform.gstreamer_root(self.target)
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

        env["RUSTFLAGS"] = env.get("RUSTFLAGS", "")

        if self.config["build"]["rustflags"]:
            env["RUSTFLAGS"] += " " + self.config["build"]["rustflags"]

        if not (self.config["build"]["ccache"] == ""):
            env["CCACHE"] = self.config["build"]["ccache"]

        env["CARGO_TARGET_DIR"] = servo.util.get_target_dir()

        # Work around https://github.com/servo/servo/issues/24446
        # Argument-less str.split normalizes leading, trailing, and double spaces
        env["RUSTFLAGS"] = " ".join(env["RUSTFLAGS"].split())

        # Suppress known false-positives during memory leak sanitizing.
        env["LSAN_OPTIONS"] = f"{env.get('LSAN_OPTIONS', '')}:suppressions={ASAN_LEAK_SUPPRESSION_FILE}"

        self.target.configure_build_environment(env, self.config, self.context.topdir)

        if sys.platform == "win32" and "windows" not in self.target.triple():
            # aws-lc-rs only supports the Ninja Generator when cross-compiling on windows hosts to non-windows.
            env["TARGET_CMAKE_GENERATOR"] = "Ninja"
            if shutil.which("ninja") is None:
                print("Error: Cross-compiling servo on windows requires the Ninja tool to be installed and in PATH.")
                print("Hint: Ninja-build is available on github at: https://github.com/ninja-build/ninja/releases")
                exit(1)
            # `tr` is also required by the CMake build rules of `aws-lc-rs`
            if shutil.which("tr") is None:
                print("Error: Cross-compiling servo on windows requires the `tr` tool, which was not found.")
                print("Hint: Try running ./mach from `git bash` instead of powershell.")
                exit(1)

        return env

    @staticmethod
    def common_command_arguments(
        build_configuration=False, build_type=False, binary_selection=False, package_configuration=False
    ):
        decorators = []
        if build_type or binary_selection:
            decorators += [
                CommandArgumentGroup("Build Type"),
                CommandArgument(
                    "--release", "-r", group="Build Type", action="store_true", help="Build in release mode"
                ),
                CommandArgument(
                    "--dev", "--debug", "-d", group="Build Type", action="store_true", help="Build in development mode"
                ),
                CommandArgument(
                    "--prod",
                    "--production",
                    group="Build Type",
                    action="store_true",
                    help="Build in release mode without debug assertions",
                ),
                CommandArgument("--profile", group="Build Type", help="Build with custom Cargo profile"),
                CommandArgument("--with-asan", action="store_true", help="Build with AddressSanitizer"),
            ]

        if build_configuration:
            decorators += [
                CommandArgumentGroup("Cross Compilation"),
                CommandArgument(
                    "--target",
                    "-t",
                    group="Cross Compilation",
                    default=None,
                    help="Cross compile for given target platform",
                ),
                CommandArgument(
                    "--android",
                    default=None,
                    action="store_true",
                    help="Build for Android. If --target is not specified, this "
                    f"will choose the default target architecture ({AndroidTarget.DEFAULT_TRIPLE}).",
                ),
                CommandArgument(
                    "--ohos",
                    default=None,
                    action="store_true",
                    help="Build for OpenHarmony. If --target is not specified, this "
                    f"will choose a default target architecture ({OpenHarmonyTarget.DEFAULT_TRIPLE}).",
                ),
                CommandArgument("--win-arm64", action="store_true", help="Use arm64 Windows target"),
                CommandArgumentGroup("Feature Selection"),
                CommandArgument(
                    "--features",
                    default=None,
                    group="Feature Selection",
                    nargs="+",
                    help="Space-separated list of features to also build",
                ),
                CommandArgument(
                    "--media-stack",
                    default=None,
                    group="Feature Selection",
                    choices=["gstreamer", "dummy"],
                    help="Which media stack to use",
                ),
                CommandArgument(
                    "--debug-mozjs",
                    default=False,
                    group="Feature Selection",
                    action="store_true",
                    help="Enable debug assertions in mozjs",
                ),
                CommandArgument(
                    "--with-debug-assertions",
                    default=False,
                    group="Feature Selection",
                    action="store_true",
                    help="Enable debug assertions in release",
                ),
                CommandArgument(
                    "--with-frame-pointer",
                    default=None,
                    group="Feature Selection",
                    action="store_true",
                    help="Build with frame pointer enabled, used by the background hang monitor.",
                ),
                CommandArgument(
                    "--use-crown", default=False, action="store_true", help="Enable Servo's `crown` linter tool"
                ),
            ]
        if package_configuration:
            decorators += [
                CommandArgumentGroup("Packaging options"),
                CommandArgument(
                    "--flavor",
                    default=None,
                    group="Packaging options",
                    help="Product flavor to be used when packaging with Gradle/Hvigor (android/ohos).",
                ),
            ]

        if binary_selection:
            decorators += [
                CommandArgumentGroup("Binary selection"),
                CommandArgument("--bin", default=None, help="Launch with specific binary"),
                CommandArgument("--nightly", "-n", default=None, help="Specify a YYYY-MM-DD nightly build to run"),
            ]

        def decorator_function(original_function):
            def configuration_decorator(self, *args, **kwargs):
                if build_type or binary_selection:
                    # If `build_type` already exists in kwargs we are doing a recursive dispatch.
                    if "build_type" not in kwargs:
                        kwargs["build_type"] = self.configure_build_type(
                            kwargs["release"],
                            kwargs["dev"],
                            kwargs["prod"],
                            kwargs["profile"],
                        )
                    kwargs.pop("release", None)
                    kwargs.pop("dev", None)
                    kwargs.pop("prod", None)
                    kwargs.pop("profile", None)

                if build_configuration:
                    self.configure_build_target(kwargs)
                    self.features = kwargs.get("features", None) or []
                    self.enable_media = self.is_media_enabled(kwargs["media_stack"])

                if binary_selection:
                    if "servo_binary" not in kwargs:
                        kwargs["servo_binary"] = (
                            kwargs.get("bin")
                            or self.get_nightly_binary_path(kwargs.get("nightly"))
                            or self.get_binary_path(kwargs.get("build_type"), asan=kwargs.get("with_asan"))
                        )
                    kwargs.pop("bin")
                    kwargs.pop("nightly")
                    if not build_type:
                        kwargs.pop("build_type")
                        kwargs.pop("with_asan")

                return original_function(self, *args, **kwargs)

            decorators.reverse()
            for decorator in decorators:
                decorator(configuration_decorator)

            return configuration_decorator

        return decorator_function

    @staticmethod
    def allow_target_configuration(original_function):
        def target_configuration_decorator(self, *args, **kwargs):
            self.configure_build_target(kwargs, suppress_log=True)
            kwargs.pop("target", False)
            kwargs.pop("android", False)
            kwargs.pop("ohos", False)
            return original_function(self, *args, **kwargs)

        return target_configuration_decorator

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
            elif self.config["build"]["mode"] != "":
                profile = self.config["build"]["mode"]
                print(f"No build type specified, but .servobuild specified custom profile `{profile}`.")
                return BuildType.custom(profile)
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

    def configure_build_target(self, kwargs: Dict[str, Any], suppress_log: bool = False):
        if hasattr(self.context, "target"):
            # This call is for a dispatched command and we've already configured
            # the target, so just use it.
            self.target = self.context.target
            return

        android = kwargs.get("android") or self.config["build"]["android"]
        ohos = kwargs.get("ohos") or self.config["build"]["ohos"]
        target_triple = kwargs.get("target")

        if android and ohos:
            print("Cannot build both android and ohos targets simultaneously.")
            sys.exit(1)

        if android and target_triple:
            print("Please specify either --target or --android.")
            sys.exit(1)

        #  Set the default Android target
        if android and not target_triple:
            target_triple = AndroidTarget.DEFAULT_TRIPLE

        if ohos and target_triple:
            print("Please specify either --target or --ohos.")
            sys.exit(1)

        #  Set the default OpenHarmony target
        if ohos and not target_triple:
            target_triple = OpenHarmonyTarget.DEFAULT_TRIPLE

        self.target = BuildTarget.from_triple(target_triple)

        self.context.target = self.target
        if self.target.is_cross_build() and not suppress_log:
            print(f"Targeting '{self.target.triple()}' for cross-compilation")

    def is_android(self):
        return isinstance(self.target, AndroidTarget)

    def is_openharmony(self):
        return isinstance(self.target, OpenHarmonyTarget)

    def is_media_enabled(self, media_stack: Optional[str]):
        """Determine whether media is enabled based on the value of the build target
        platform and the value of the '--media-stack' command-line argument.
        Returns true if media is enabled."""
        if not media_stack:
            if self.config["build"]["media-stack"] != "auto":
                media_stack = self.config["build"]["media-stack"]
                assert media_stack
            elif not self.target.is_cross_build():
                media_stack = "gstreamer"
            else:
                media_stack = "dummy"

        # This is a workaround for Ubuntu 20.04, which doesn't support a new enough GStreamer.
        # Once we drop support for this platform (it's currently needed for wpt.fyi runners),
        # we can remove this workaround and officially only support Ubuntu 22.04 and up.
        platform = servo.platform.get()
        if not self.target.is_cross_build() and platform.is_linux and not platform.is_gstreamer_installed(self.target):
            return False

        return media_stack != "dummy"

    def run_cargo_build_like_command(
        self,
        command: str,
        cargo_args: List[str],
        env=None,
        verbose=False,
        debug_mozjs=False,
        with_debug_assertions=False,
        with_frame_pointer=False,
        use_crown=False,
        target_override: Optional[str] = None,
        **_kwargs,
    ):
        env = env or self.build_env()

        # NB: On non-Linux platforms we cannot check whether GStreamer is installed until
        # environment variables are set via `self.build_env()`.
        platform = servo.platform.get()
        if self.enable_media and not platform.is_gstreamer_installed(self.target):
            raise FileNotFoundError(
                "GStreamer libraries not found (>= version 1.18).Please see installation instructions in README.md"
            )

        args = []
        if "--manifest-path" not in cargo_args:
            args += [
                "--manifest-path",
                path.join(self.context.topdir, "ports", "servoshell", "Cargo.toml"),
            ]

        if self.target.is_cross_build():
            args += ["--target", self.target.triple()]
            if type(self.target) in [AndroidTarget, OpenHarmonyTarget]:
                # Note: in practice `cargo rustc` should just be used unconditionally.
                assert command != "build", "For Android / OpenHarmony `cargo rustc` must be used instead of cargo build"
                if command == "rustc":
                    args += ["--lib", "--crate-type=cdylib"]
        elif target_override:
            args += ["--target", target_override]

        features = []

        if use_crown:
            if "CARGO_BUILD_RUSTC" in env:
                current_rustc = env["CARGO_BUILD_RUSTC"]
                if current_rustc != "crown":
                    print(
                        "Error: `mach` was called with `--use-crown` while `CARGO_BUILD_RUSTC` was"
                        f"already set to `{current_rustc}` in the parent environment.\n"
                        "These options conflict, please specify only one of them."
                    )
                    sys.exit(1)
            env["CARGO_BUILD_RUSTC"] = "crown"
            # Modyfing `RUSTC` or `CARGO_BUILD_RUSTC` to use a linter does not cause
            # `cargo check` to rebuild. To work around this bug use a `crown` feature
            # to invalidate caches and force a rebuild / relint.
            # See https://github.com/servo/servo/issues/35072#issuecomment-2600749483
            features += ["crown"]

        if "-p" not in cargo_args:  # We're building specific package, that may not have features
            features += list(self.features)
            if self.enable_media:
                features.append("media-gstreamer")
            if self.config["build"]["debug-mozjs"] or debug_mozjs:
                features.append("debugmozjs")

            if with_frame_pointer:
                env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -C force-frame-pointers=yes"
                features.append("profilemozjs")
            if self.config["build"]["webgl-backtrace"]:
                features.append("webgl-backtrace")
            if self.config["build"]["dom-backtrace"]:
                features.append("js_backtrace")
            args += ["--features", " ".join(features)]

        if with_debug_assertions or self.config["build"]["debug-assertions"]:
            env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -C debug_assertions"

        # mozjs gets its Python from `env['PYTHON3']`, which defaults to `python3`,
        # but uv venv on Windows only provides a `python`, not `python3`.
        env["PYTHON3"] = "python"

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

    def ensure_bootstrapped(self):
        if self.context.bootstrapped:
            return

        servo.platform.get().passive_bootstrap()
        self.context.bootstrapped = True

        # Toolchain installation is handled automatically for non cross compilation builds.
        if not self.target.is_cross_build():
            return

        installed_targets = check_output(["rustup", "target", "list", "--installed"], cwd=self.context.topdir).decode()
        if self.target.triple() not in installed_targets:
            check_call(["rustup", "target", "add", self.target.triple()], cwd=self.context.topdir)

    def ensure_rustup_version(self):
        try:
            version_line = subprocess.check_output(
                ["rustup" + servo.platform.get().executable_suffix(), "--version"],
                # Silence "info: This is the version for the rustup toolchain manager,
                # not the rustc compiler."
                stderr=open(os.devnull, "wb"),
            )
        except OSError as e:
            if e.errno == NO_SUCH_FILE_OR_DIRECTORY:
                print(
                    "It looks like rustup is not installed. See instructions at "
                    "https://github.com/servo/servo/#setting-up-your-environment"
                )
                print()
                sys.exit(1)
            raise
        version = tuple(map(int, re.match(rb"rustup (\d+)\.(\d+)\.(\d+)", version_line).groups()))
        version_needed = (1, 23, 0)
        if version < version_needed:
            print("rustup is at version %s.%s.%s, Servo requires %s.%s.%s or more recent." % (version + version_needed))
            print("Try running 'rustup self update'.")
            sys.exit(1)

    def ensure_clobbered(self, target_dir=None):
        if target_dir is None:
            target_dir = util.get_target_dir()
        auto = True if os.environ.get("AUTOCLOBBER", False) else False
        src_clobber = os.path.join(self.context.topdir, "CLOBBER")
        target_clobber = os.path.join(target_dir, "CLOBBER")

        if not os.path.exists(target_dir):
            os.makedirs(target_dir)

        if not os.path.exists(target_clobber):
            # Simply touch the file.
            with open(target_clobber, "a"):
                pass

        if auto:
            if os.path.getmtime(src_clobber) > os.path.getmtime(target_clobber):
                print("Automatically clobbering target directory: {}".format(target_dir))

                try:
                    Registrar.dispatch("clean", context=self.context, verbose=True)
                    print("Successfully completed auto clobber.")
                except subprocess.CalledProcessError as error:
                    sys.exit(error)
            else:
                print("Clobber not needed.")

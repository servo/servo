# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function

import contextlib
from typing import List, Optional
import functools
import gzip
import itertools
import json
import locale
import os
import platform
import re
import shutil
import six
import subprocess
import sys
import tarfile
import urllib
import zipfile

from errno import ENOENT as NO_SUCH_FILE_OR_DIRECTORY
from glob import glob
from os import path
from subprocess import PIPE

import toml
import servo.platform
import servo.util as util

from xml.etree.ElementTree import XML
from servo.util import download_file, get_default_cache_dir
from mach.decorators import CommandArgument, CommandArgumentGroup
from mach.registrar import Registrar

BIN_SUFFIX = ".exe" if sys.platform == "win32" else ""
NIGHTLY_REPOSITORY_URL = "https://servo-builds2.s3.amazonaws.com/"


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


def check_output(*args, **kwargs):
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
        self.features = []
        self.cross_compile_target = None
        self.is_uwp_build = False
        self.is_android_build = False

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
        self.config["tools"].setdefault("cache-dir", get_default_cache_dir(context.topdir))
        resolverelative("tools", "cache-dir")

        default_cargo_home = os.environ.get("CARGO_HOME",
                                            path.join(context.topdir, ".cargo"))
        self.config["tools"].setdefault("cargo-home-dir", default_cargo_home)
        resolverelative("tools", "cargo-home-dir")

        context.sharedir = self.config["tools"]["cache-dir"]

        self.config["tools"].setdefault("use-rustup", True)
        self.config["tools"].setdefault("rustc-with-gold", get_env_bool("SERVO_RUSTC_WITH_GOLD", True))

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)
        self.config["build"].setdefault("mode", "")
        self.config["build"].setdefault("debug-assertions", False)
        self.config["build"].setdefault("debug-mozjs", False)
        self.config["build"].setdefault("layout-2020", True)
        self.config["build"].setdefault("media-stack", "auto")
        self.config["build"].setdefault("ccache", "")
        self.config["build"].setdefault("rustflags", "")
        self.config["build"].setdefault("incremental", None)
        self.config["build"].setdefault("thinlto", False)
        self.config["build"].setdefault("webgl-backtrace", False)
        self.config["build"].setdefault("dom-backtrace", False)

        self.config.setdefault("android", {})
        self.config["android"].setdefault("sdk", "")
        self.config["android"].setdefault("ndk", "")
        self.config["android"].setdefault("toolchain", "")

        # Set default android target
        self.setup_configuration_for_android_target("armv7-linux-androideabi")

    _rust_toolchain = None

    def rust_toolchain(self):
        if self._rust_toolchain is None:
            filename = path.join(self.context.topdir, "rust-toolchain")
            with open(filename) as f:
                self._rust_toolchain = f.read().strip()

            if platform.system() == "Windows":
                self._rust_toolchain += "-x86_64-pc-windows-msvc"

        return self._rust_toolchain

    def call_rustup_run(self, args, **kwargs):
        if self.config["tools"]["use-rustup"]:
            assert self.context.bootstrapped
            args = ["rustup" + BIN_SUFFIX, "run", "--install", self.rust_toolchain()] + args
        else:
            args[0] += BIN_SUFFIX
        return call(args, **kwargs)

    def get_top_dir(self):
        return self.context.topdir

    def get_apk_path(self, release):
        base_path = util.get_target_dir()
        base_path = path.join(base_path, "android", self.config["android"]["target"])
        apk_name = "servoapp.apk"
        build_type = "release" if release else "debug"
        return path.join(base_path, build_type, apk_name)

    def get_binary_path(self, release, dev, target=None, android=False, simpleservo=False):
        # TODO(autrilla): this function could still use work - it shouldn't
        # handle quitting, or printing. It should return the path, or an error.
        base_path = util.get_target_dir()

        binary_name = "servo" + BIN_SUFFIX

        if android:
            base_path = path.join(base_path, "android", self.config["android"]["target"])
            simpleservo = True
        elif target:
            base_path = path.join(base_path, target)

        if simpleservo:
            if sys.platform == "win32":
                binary_name = "simpleservo.dll"
            elif sys.platform == "darwin":
                binary_name = "libsimpleservo.dylib"
            else:
                binary_name = "libsimpleservo.so"

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
        return path.join(self.context.sharedir, "msvc-dependencies", package,
                         servo.platform.windows.DEPENDENCIES[package])

    def vs_dirs(self):
        assert 'windows' in servo.platform.host_triple()
        vsinstalldir = os.environ.get('VSINSTALLDIR')
        vs_version = os.environ.get('VisualStudioVersion')
        if vsinstalldir and vs_version:
            msbuild_version = get_msbuild_version(vs_version)
        else:
            (vsinstalldir, vs_version, msbuild_version) = find_highest_msvc_version()
        msbuildinstalldir = os.path.join(vsinstalldir, "MSBuild", msbuild_version, "Bin")
        vcinstalldir = os.environ.get("VCINSTALLDIR", "") or os.path.join(vsinstalldir, "VC")
        return {
            'msbuild': msbuildinstalldir,
            'vsdir': vsinstalldir,
            'vs_version': vs_version,
            'vcdir': vcinstalldir,
        }

    def build_env(self, is_build=False):
        """Return an extended environment dictionary."""
        env = os.environ.copy()

        if "media-dummy" not in self.features:
            servo.platform.get().set_gstreamer_environment_variables_if_necessary(
                env, cross_compilation_target=self.cross_compile_target,
                check_installation=is_build)

        extra_path = []
        effective_target = self.cross_compile_target or servo.platform.host_triple()
        if "msvc" in effective_target:
            extra_path += [path.join(self.msvc_package_dir("cmake"), "bin")]
            extra_path += [path.join(self.msvc_package_dir("llvm"), "bin")]
            extra_path += [path.join(self.msvc_package_dir("ninja"), "bin")]
            extra_path += [self.msvc_package_dir("nuget")]

            env.setdefault("CC", "clang-cl.exe")
            env.setdefault("CXX", "clang-cl.exe")
            if self.is_uwp_build:
                env.setdefault("TARGET_CFLAGS", "")
                env.setdefault("TARGET_CXXFLAGS", "")
                env["TARGET_CFLAGS"] += " -DWINAPI_FAMILY=WINAPI_FAMILY_APP"
                env["TARGET_CXXFLAGS"] += " -DWINAPI_FAMILY=WINAPI_FAMILY_APP"

            arch = effective_target.split('-')[0]
            vcpkg_arch = {
                "x86_64": "x64-windows",
                "i686": "x86-windows",
                "aarch64": "arm64-windows",
            }
            target_arch = vcpkg_arch[arch]
            if self.is_uwp_build:
                target_arch += "-uwp"
            openssl_base_dir = path.join(self.msvc_package_dir("openssl"), target_arch)

            # Link openssl
            env["OPENSSL_INCLUDE_DIR"] = path.join(openssl_base_dir, "include")
            env["OPENSSL_LIB_DIR"] = path.join(openssl_base_dir, "lib")
            env["OPENSSL_LIBS"] = "libssl:libcrypto"
            # Link moztools, used for building SpiderMonkey
            moztools_paths = [
                path.join(self.msvc_package_dir("moztools"), "bin"),
                path.join(self.msvc_package_dir("moztools"), "msys", "bin"),
            ]
            # In certain cases we need to ensure that tools with conflicting MSYS versions
            # can be placed in the PATH ahead of the moztools directories.
            moztools_path_prepend = env.get("MOZTOOLS_PATH_PREPEND", None)
            if moztools_path_prepend:
                moztools_paths.insert(0, moztools_path_prepend)
            env["MOZTOOLS_PATH"] = os.pathsep.join(moztools_paths)
            # Link autoconf 2.13, used for building SpiderMonkey
            env["AUTOCONF"] = path.join(self.msvc_package_dir("moztools"), "msys", "local", "bin", "autoconf-2.13")
            # Link LLVM
            env["LIBCLANG_PATH"] = path.join(self.msvc_package_dir("llvm"), "lib")

            if not os.environ.get("NATIVE_WIN32_PYTHON"):
                env["NATIVE_WIN32_PYTHON"] = sys.executable
            # Always build harfbuzz from source
            env["HARFBUZZ_SYS_NO_PKG_CONFIG"] = "true"

        if sys.platform != "win32":
            env.setdefault("CC", "clang")
            env.setdefault("CXX", "clang++")

        if extra_path:
            util.append_paths_to_env(env, "PATH", extra_path)

        if self.config["build"]["incremental"]:
            env["CARGO_INCREMENTAL"] = "1"
        elif self.config["build"]["incremental"] is not None:
            env["CARGO_INCREMENTAL"] = "0"

        # Paths to Android build tools:
        if self.config["android"]["sdk"]:
            env["ANDROID_SDK"] = self.config["android"]["sdk"]
        if self.config["android"]["ndk"]:
            env["ANDROID_NDK"] = self.config["android"]["ndk"]
        if self.config["android"]["toolchain"]:
            env["ANDROID_TOOLCHAIN"] = self.config["android"]["toolchain"]
        if self.config["android"]["platform"]:
            env["ANDROID_PLATFORM"] = self.config["android"]["platform"]

        toolchains = path.join(self.context.topdir, "android-toolchains")
        for kind in ["sdk", "ndk"]:
            default = os.path.join(toolchains, kind)
            if os.path.isdir(default):
                env.setdefault("ANDROID_" + kind.upper(), default)

        tools = os.path.join(toolchains, "sdk", "platform-tools")
        if os.path.isdir(tools):
            env["PATH"] = "%s%s%s" % (tools, os.pathsep, env["PATH"])

        # These are set because they are the variable names that build-apk
        # expects. However, other submodules have makefiles that reference
        # the env var names above. Once winit is enabled and set as the
        # default, we could modify the subproject makefiles to use the names
        # below and remove the vars above, to avoid duplication.
        if "ANDROID_SDK" in env:
            env["ANDROID_HOME"] = env["ANDROID_SDK"]
        if "ANDROID_NDK" in env:
            env["NDK_HOME"] = env["ANDROID_NDK"]
        if "ANDROID_TOOLCHAIN" in env:
            env["NDK_STANDALONE"] = env["ANDROID_TOOLCHAIN"]

        if self.config["build"]["rustflags"]:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " " + self.config["build"]["rustflags"]

        if self.config["tools"]["rustc-with-gold"]:
            if shutil.which('ld.gold'):
                env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C link-args=-fuse-ld=gold"

        if not (self.config["build"]["ccache"] == ""):
            env['CCACHE'] = self.config["build"]["ccache"]

        # Ensure Rust uses hard floats and SIMD on ARM devices
        if self.cross_compile_target and (
            self.cross_compile_target.startswith('arm')
                or self.cross_compile_target.startswith('aarch64')):
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C target-feature=+neon"

        env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -W unused-extern-crates"
        env["CARGO_TARGET_DIR"] = servo.util.get_target_dir()

        git_info = []
        if os.path.isdir('.git') and is_build:
            # Get the subject of the commit
            git_commit_subject = subprocess.check_output([
                'git', 'show', '-s', '--format=%s', 'HEAD'
            ]).strip()

            git_sha = None
            # Check if it's a bundle commit
            if git_commit_subject.startswith(b"Shallow version of commit "):
                # This is a bundle commit
                # Get the SHA-1 from the bundle subject: "Shallow version of commit {sha1}"
                git_sha = git_commit_subject.split(b' ')[-1].strip()
                # Shorten hash
                # NOTE: Partially verifies the hash, but it will still pass if it's, e.g., a tree
                git_sha = subprocess.check_output([
                    'git', 'rev-parse', '--short', git_sha.decode('ascii')
                ])
            else:
                # This is a regular commit
                git_sha = subprocess.check_output([
                    'git', 'rev-parse', '--short', 'HEAD'
                ]).strip()

            git_is_dirty = bool(subprocess.check_output([
                'git', 'status', '--porcelain'
            ]).strip())

            git_info.append('')
            git_info.append(six.ensure_str(git_sha))
            if git_is_dirty:
                git_info.append('dirty')

        env['GIT_INFO'] = '-'.join(git_info)

        if self.config["build"]["thinlto"]:
            env['RUSTFLAGS'] += " -Z thinlto"

        # Work around https://github.com/servo/servo/issues/24446
        # Argument-less str.split normalizes leading, trailing, and double spaces
        env['RUSTFLAGS'] = " ".join(env['RUSTFLAGS'].split())

        return env

    @staticmethod
    def build_like_command_arguments(original_function):
        decorators = [
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
            CommandArgument(
                '--uwp',
                group="Cross Compilation",
                action='store_true',
                help='Build for HoloLens (x64)'),
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
                '--libsimpleservo',
                default=None,
                group="Feature Selection",
                action='store_true',
                help='Build the libsimpleservo library instead of the servo executable',
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
                '--with-layout-2020', '--layout-2020',
                group="Feature Selection", default=None, action='store_true'),
            CommandArgument(
                '--with-layout-2013', '--layout-2013', group="Feature Selection", default=None, action='store_true'),
            CommandArgument('--without-wgl', group="Feature Selection", default=None, action='store_true'),
        ]

        def configuration_decorator(self, *args, **kwargs):
            self.configure_cross_compilation(
                kwargs['target'], kwargs['android'],
                kwargs['uwp'], kwargs['win_arm64'])

            self.features = kwargs.get("features", None) or []
            self.configure_media_stack(kwargs['media_stack'])
            return original_function(self, *args, **kwargs)

        decorators.reverse()
        decorated_function = configuration_decorator
        for decorator in decorators:
            decorated_function = decorator(decorated_function)
        return decorated_function

    def configure_cross_compilation(
            self,
            cross_compile_target: Optional[str],
            android: Optional[str],
            uwp: Optional[str],
            win_arm64: Optional[str]):
        # Force the UWP-enabled target if the convenience UWP flags are passed.
        if uwp and not cross_compile_target:
            if win_arm64:
                cross_compile_target = 'aarch64-uwp-windows-msvc'
            else:
                cross_compile_target = 'x86_64-uwp-windows-msvc'

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
        self.is_uwp_build = uwp or (cross_compile_target and "uwp" in cross_compile_target)
        self.is_android_build = (cross_compile_target and "android" in cross_compile_target)

        if self.cross_compile_target:
            print(f"Targeting '{self.cross_compile_target}' for cross-compilation")

    def configure_media_stack(self, media_stack: Optional[str]):
        """Determine what media stack to use based on the value of the build target
           platform and the value of the '--media-stack' command-line argument.
           The chosen media stack is written into the `features` instance variable."""
        if not media_stack:
            if self.config["build"]["media-stack"] != "auto":
                media_stack = self.config["build"]["media-stack"]
                assert media_stack
            elif (
                not self.cross_compile_target
                or ("armv7" in self.cross_compile_target and self.is_android_build)
                or "x86_64" in self.cross_compile_target
                or "uwp" in self.cross_compile_target
            ):
                media_stack = "gstreamer"
            else:
                media_stack = "dummy"
        self.features += ["media-" + media_stack]

    def run_cargo_build_like_command(
        self, command: str, cargo_args: List[str],
        env=None, verbose=False,
        libsimpleservo=False,
        debug_mozjs=False, with_debug_assertions=False,
        with_frame_pointer=False, without_wgl=False,
        with_layout_2020=False, with_layout_2013=False,
        **_kwargs
    ):
        env = env or self.build_env()

        args = []
        if "--manifest-path" not in cargo_args:
            if libsimpleservo or self.is_android_build:
                if self.is_android_build:
                    api = "jniapi"
                else:
                    api = "capi"
                port = path.join("libsimpleservo", api)
            else:
                port = "winit"
            args += [
                "--manifest-path",
                path.join(self.context.topdir, "ports", port, "Cargo.toml"),
            ]
        if self.cross_compile_target:
            args += ["--target", self.cross_compile_target]

        if "-p" not in cargo_args:  # We're building specific package, that may not have features
            features = list(self.features)
            if self.config["build"]["debug-mozjs"] or debug_mozjs:
                features.append("debugmozjs")

            features.append("native-bluetooth")

            if self.is_uwp_build:
                features.append("no-wgl")
                features.append("uwp")

            if with_layout_2020 or (self.config["build"]["layout-2020"] and not with_layout_2013):
                features.append("layout-2020")
            elif "layout-2020" not in features:
                features.append("layout-2013")
            if with_frame_pointer:
                env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C force-frame-pointers=yes"
                features.append("profilemozjs")
            if without_wgl:
                features.append("no-wgl")
            if self.config["build"]["webgl-backtrace"]:
                features.append("webgl-backtrace")
            if self.config["build"]["dom-backtrace"]:
                features.append("dom-backtrace")
            args += ["--features", " ".join(features)]

        if with_debug_assertions or self.config["build"]["debug-assertions"]:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C debug_assertions"

        if self.is_uwp_build:
            cargo_args += ["-Z", "build-std"]
        return self.call_rustup_run(["cargo", command] + args + cargo_args, env=env, verbose=verbose)

    def android_support_dir(self):
        return path.join(self.context.topdir, "support", "android")

    def android_aar_dir(self):
        return path.join(self.context.topdir, "target", "android", "aar")

    def android_adb_path(self, env):
        if "ANDROID_SDK" in env:
            sdk_adb = path.join(env["ANDROID_SDK"], "platform-tools", "adb")
            if path.exists(sdk_adb):
                return sdk_adb
        return "adb"

    def android_emulator_path(self, env):
        if "ANDROID_SDK" in env:
            sdk_adb = path.join(env["ANDROID_SDK"], "emulator", "emulator")
            if path.exists(sdk_adb):
                return sdk_adb
        return "emulator"

    def setup_configuration_for_android_target(self, target: str):
        """If cross-compilation targets Android, configure the Android
           build by writing the appropriate toolchain configuration values
           into the stored configuration."""
        if target == "armv7-linux-androideabi":
            self.config["android"]["platform"] = "android-21"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = "arm-linux-androideabi"
            self.config["android"]["arch"] = "arm"
            self.config["android"]["lib"] = "armeabi-v7a"
            self.config["android"]["toolchain_name"] = "arm-linux-androideabi"
            return True
        elif target == "aarch64-linux-android":
            self.config["android"]["platform"] = "android-21"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = target
            self.config["android"]["arch"] = "arm64"
            self.config["android"]["lib"] = "arm64-v8a"
            self.config["android"]["toolchain_name"] = target
            return True
        elif target == "i686-linux-android":
            # https://github.com/jemalloc/jemalloc/issues/1279
            self.config["android"]["platform"] = "android-21"
            self.config["android"]["target"] = target
            self.config["android"]["toolchain_prefix"] = "x86"
            self.config["android"]["arch"] = "x86"
            self.config["android"]["lib"] = "x86"
            self.config["android"]["toolchain_name"] = target
            return True
        return False

    def ensure_bootstrapped(self, rustup_components=None):
        if self.context.bootstrapped:
            return

        # Always check if all needed MSVC dependencies are installed
        target_platform = self.cross_compile_target or servo.platform.host_triple()
        if "msvc" in target_platform:
            Registrar.dispatch("bootstrap", context=self.context)

        if self.config["tools"]["use-rustup"]:
            self.ensure_rustup_version()
            toolchain = self.rust_toolchain()

            status = subprocess.call(
                ["rustup", "run", toolchain, "rustc", "--version"],
                stdout=open(os.devnull, "wb"),
                stderr=subprocess.STDOUT,
            )
            if status:
                check_call(["rustup", "toolchain", "install", "--profile", "minimal", toolchain])

            installed = check_output(
                ["rustup", "component", "list", "--installed", "--toolchain", toolchain]
            )
            required_components = {
                # For components/script_plugins, https://github.com/rust-lang/rust/pull/67469
                "rustc-dev",
                # https://github.com/rust-lang/rust/issues/72594#issuecomment-633779564
                "llvm-tools-preview",
            }
            for component in set(rustup_components or []) | required_components:
                if component.encode("utf-8") not in installed:
                    check_call(["rustup", "component", "add", "--toolchain", toolchain, component])

            needs_toolchain_install = self.cross_compile_target \
                and not self.is_uwp_build \
                and self.cross_compile_target.encode("utf-8") not in check_output(
                    ["rustup", "target", "list", "--installed", "--toolchain", toolchain]
                )
            if needs_toolchain_install:
                check_call(["rustup", "target", "add", "--toolchain", toolchain, self.cross_compile_target])

        self.context.bootstrapped = True

    def ensure_rustup_version(self):
        try:
            version_line = subprocess.check_output(
                ["rustup" + BIN_SUFFIX, "--version"],
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
        version_needed = (1, 21, 0)
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


def find_highest_msvc_version_ext():
    def vswhere(args):
        program_files = (os.environ.get('PROGRAMFILES(X86)')
                         or os.environ.get('PROGRAMFILES'))
        if not program_files:
            return []
        vswhere = os.path.join(program_files, 'Microsoft Visual Studio',
                               'Installer', 'vswhere.exe')
        if not os.path.exists(vswhere):
            return []
        return json.loads(check_output([vswhere, '-format', 'json'] + args).decode(errors='ignore'))

    for install in vswhere(['-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
                            '-requires', 'Microsoft.VisualStudio.Component.Windows10SDK']):
        version = install['installationVersion'].split('.')[0] + '.0'
        yield (install['installationPath'], version, "Current" if version == '16.0' else version)


def find_highest_msvc_version():
    editions = ["Enterprise", "Professional", "Community", "BuildTools"]
    prog_files = os.environ.get("ProgramFiles(x86)")
    base_vs_path = os.path.join(prog_files, "Microsoft Visual Studio")

    vs_versions = ["2019", "2017"]
    versions = {
        ("2019", "vs"): "16.0",
        ("2017", "vs"): "15.0",
    }

    for version in vs_versions:
        for edition in editions:
            vs_version = versions[version, "vs"]
            msbuild_version = get_msbuild_version(vs_version)

            vsinstalldir = os.path.join(base_vs_path, version, edition)
            if os.path.exists(vsinstalldir):
                return (vsinstalldir, vs_version, msbuild_version)

    versions = sorted(find_highest_msvc_version_ext(), key=lambda tup: float(tup[1]))
    if not versions:
        print(f"Can't find MSBuild.exe installation under {base_vs_path}. "
              "Please set the VSINSTALLDIR and VisualStudioVersion environment variables")
        sys.exit(1)
    return versions[0]


def get_msbuild_version(vs_version):
    if vs_version in ("15.0", "14.0"):
        msbuild_version = vs_version
    else:
        msbuild_version = "Current"
    return msbuild_version

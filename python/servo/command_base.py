# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from errno import ENOENT as NO_SUCH_FILE_OR_DIRECTORY
from glob import glob
import shutil
import gzip
import itertools
import locale
import os
from os import path
import platform
import re
import contextlib
import subprocess
from subprocess import PIPE
import sys
import tarfile
import zipfile
from xml.etree.ElementTree import XML
from servo.util import download_file
import urllib2
from bootstrap import check_gstreamer_lib

from mach.decorators import CommandArgument
from mach.registrar import Registrar
import toml

from servo.packages import WINDOWS_MSVC as msvc_deps
from servo.util import host_triple

BIN_SUFFIX = ".exe" if sys.platform == "win32" else ""
NIGHTLY_REPOSITORY_URL = "https://servo-builds.s3.amazonaws.com/"


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
            file_list.sort(cmp=locale.strcoll)

        # Use a temporary file and atomic rename to avoid partially-formed
        # packaging (in case of exceptional situations like running out of disk space).
        # TODO do this in a temporary folder after #11983 is fixed
        temp_file = '{}.temp~'.format(dest_archive)
        with os.fdopen(os.open(temp_file, os.O_WRONLY | os.O_CREAT, 0644), 'w') as out_file:
            if dest_archive.endswith('.zip'):
                with zipfile.ZipFile(temp_file, 'w', zipfile.ZIP_DEFLATED) as zip_file:
                    for entry in file_list:
                        arcname = entry
                        if prepend_path is not None:
                            arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                        zip_file.write(entry, arcname=arcname)
            else:
                with gzip.GzipFile('wb', fileobj=out_file, mtime=0) as gzip_file:
                    with tarfile.open(fileobj=gzip_file, mode='w:') as tar_file:
                        for entry in file_list:
                            arcname = entry
                            if prepend_path is not None:
                                arcname = os.path.normpath(os.path.join(prepend_path, arcname))
                            tar_file.add(entry, filter=reset, recursive=False, arcname=arcname)
        os.rename(temp_file, dest_archive)


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


def check_output(*args, **kwargs):
    """Wrap `subprocess.call`, printing the command if verbose=True."""
    verbose = kwargs.pop('verbose', False)
    if verbose:
        print(' '.join(args[0]))
    if 'env' in kwargs:
        kwargs['env'] = normalize_env(kwargs['env'])
    # we have to use shell=True in order to get PATH handling
    # when looking for the binary on Windows
    return subprocess.check_output(*args, shell=sys.platform == 'win32', **kwargs)


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
    return sys.platform == 'win32'


def is_macosx():
    return sys.platform == 'darwin'


def is_linux():
    return sys.platform.startswith('linux')


def append_to_path_env(string, env, name):
    variable = ""
    if name in env:
        variable = env[name]
        if len(variable) > 0:
            variable += os.pathsep
    variable += string
    env[name] = variable


def set_osmesa_env(bin_path, env):
    """Set proper LD_LIBRARY_PATH and DRIVE for software rendering on Linux and OSX"""
    if is_linux():
        dep_path = find_dep_path_newest('osmesa-src', bin_path)
        if not dep_path:
            return None
        osmesa_path = path.join(dep_path, "out", "lib", "gallium")
        append_to_path_env(osmesa_path, env, "LD_LIBRARY_PATH")
        env["GALLIUM_DRIVER"] = "softpipe"
    elif is_macosx():
        osmesa_dep_path = find_dep_path_newest('osmesa-src', bin_path)
        if not osmesa_dep_path:
            return None
        osmesa_path = path.join(osmesa_dep_path,
                                "out", "src", "gallium", "targets", "osmesa", ".libs")
        glapi_path = path.join(osmesa_dep_path,
                               "out", "src", "mapi", "shared-glapi", ".libs")
        append_to_path_env(osmesa_path + ":" + glapi_path, env, "DYLD_LIBRARY_PATH")
        env["GALLIUM_DRIVER"] = "softpipe"
    return env


def gstreamer_root(target, env, topdir=None):
    if is_windows():
        arch = {
            "x86_64": "X86_64",
            "x86": "X86",
            "aarch64": "ARM64",
        }
        gst_x64 = arch[target.split('-')[0]]
        gst_default_path = path.join("C:\\gstreamer\\1.0", gst_x64)
        gst_env = "GSTREAMER_1_0_ROOT_" + gst_x64
        if env.get(gst_env) is not None:
            return env.get(gst_env)
        elif os.path.exists(path.join(gst_default_path, "bin", "ffi-7.dll")):
            return gst_default_path
    elif sys.platform == "linux2":
        return path.join(topdir, "support", "linux", "gstreamer", "gst")
    return None


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

        self.config["tools"].setdefault("use-rustup", True)
        self.config["tools"].setdefault("rustc-with-gold", get_env_bool("SERVO_RUSTC_WITH_GOLD", True))

        self.config.setdefault("build", {})
        self.config["build"].setdefault("android", False)
        self.config["build"].setdefault("mode", "")
        self.config["build"].setdefault("debug-mozjs", False)
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
        self.handle_android_target("armv7-linux-androideabi")

    _default_toolchain = None

    def toolchain(self):
        return self.default_toolchain()

    def default_toolchain(self):
        if self._default_toolchain is None:
            filename = path.join(self.context.topdir, "rust-toolchain")
            with open(filename) as f:
                self._default_toolchain = f.read().strip()
        return self._default_toolchain

    def call_rustup_run(self, args, **kwargs):
        if self.config["tools"]["use-rustup"]:
            try:
                version_line = subprocess.check_output(["rustup" + BIN_SUFFIX, "--version"])
            except OSError as e:
                if e.errno == NO_SUCH_FILE_OR_DIRECTORY:
                    print "It looks like rustup is not installed. See instructions at " \
                          "https://github.com/servo/servo/#setting-up-your-environment"
                    print
                    return 1
                raise
            version = tuple(map(int, re.match("rustup (\d+)\.(\d+)\.(\d+)", version_line).groups()))
            if version < (1, 11, 0):
                print "rustup is at version %s.%s.%s, Servo requires 1.11.0 or more recent." % version
                print "Try running 'rustup self update'."
                return 1
            toolchain = self.toolchain()
            if platform.system() == "Windows":
                toolchain += "-x86_64-pc-windows-msvc"
            args = ["rustup" + BIN_SUFFIX, "run", "--install", toolchain] + args
        else:
            args[0] += BIN_SUFFIX
        return call(args, **kwargs)

    def get_top_dir(self):
        return self.context.topdir

    def get_target_dir(self):
        if "CARGO_TARGET_DIR" in os.environ:
            return os.environ["CARGO_TARGET_DIR"]
        else:
            return path.join(self.context.topdir, "target")

    def get_apk_path(self, release):
        base_path = self.get_target_dir()
        base_path = path.join(base_path, "android", self.config["android"]["target"])
        apk_name = "servoapp.apk"
        build_type = "release" if release else "debug"
        return path.join(base_path, build_type, apk_name)

    def get_binary_path(self, release, dev, target=None, android=False, magicleap=False, simpleservo=False):
        # TODO(autrilla): this function could still use work - it shouldn't
        # handle quitting, or printing. It should return the path, or an error.
        base_path = self.get_target_dir()

        binary_name = "servo" + BIN_SUFFIX

        if magicleap:
            base_path = path.join(base_path, "magicleap", "aarch64-linux-android")
            binary_name = "libmlservo.a"
        elif android:
            base_path = path.join(base_path, "android", self.config["android"]["target"])
            simpleservo = True
        elif target:
            base_path = path.join(base_path, target)

        if simpleservo:
            binary_name = "simpleservo.dll" if sys.platform == "win32" else "libsimpleservo.so"

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
        req = urllib2.Request(
            "{}/{}/{}".format(repository_index, os_prefix, nightly_date))
        try:
            response = urllib2.urlopen(req).read()
            tree = XML(response)
            namespaces = {'ns': tree.tag[1:tree.tag.index('}')]}
            file_to_download = tree.find('ns:Contents', namespaces).find(
                'ns:Key', namespaces).text
        except urllib2.URLError as e:
            print("Could not fetch the available nightly versions from the repository : {}".format(
                e.reason))
            sys.exit(1)
        except AttributeError as e:
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
            download_file(destination_file, NIGHTLY_REPOSITORY_URL +
                          file_to_download, destination_file)

        # Extract the downloaded nightly version
        if os.path.isdir(destination_folder):
            print("The nightly folder {} has already been extracted.".format(
                destination_folder))
        else:
            self.extract_nightly(nightlies_folder, destination_folder, destination_file)

        return self.get_executable(destination_folder)

    def needs_gstreamer_env(self, target, env, uwp=False):
        if uwp:
            return False
        try:
            if check_gstreamer_lib():
                return False
        except:
            # Some systems don't have pkg-config; we can't probe in this case
            # and must hope for the best
            return False
        effective_target = target or host_triple()
        if "x86_64" not in effective_target or "android" in effective_target:
            # We don't build gstreamer for non-x86_64 / android yet
            return False
        if sys.platform == "linux2" or is_windows():
            if path.isdir(gstreamer_root(effective_target, env, self.get_top_dir())):
                return True
            else:
                raise Exception("Your system's gstreamer libraries are out of date \
(we need at least 1.12). Please run ./mach bootstrap-gstreamer")
        else:
                raise Exception("Your system's gstreamer libraries are out of date \
(we need at least 1.12). If you're unable to \
install them, let us know by filing a bug!")
        return False

    def set_run_env(self, android=False):
        """Some commands, like test-wpt, don't use a full build env,
           but may still need dynamic search paths. This command sets that up"""
        if not android and self.needs_gstreamer_env(None, os.environ):
            gstpath = gstreamer_root(host_triple(), os.environ, self.get_top_dir())
            if gstpath is None:
                return
            os.environ["LD_LIBRARY_PATH"] = path.join(gstpath, "lib")
            os.environ["GST_PLUGIN_SYSTEM_PATH"] = path.join(gstpath, "lib", "gstreamer-1.0")
            os.environ["PKG_CONFIG_PATH"] = path.join(gstpath, "lib", "pkgconfig")
            os.environ["GST_PLUGIN_SCANNER"] = path.join(gstpath, "libexec", "gstreamer-1.0", "gst-plugin-scanner")

    def msvc_package_dir(self, package):
        return path.join(self.context.sharedir, "msvc-dependencies", package, msvc_deps[package])

    def vs_dirs(self):
        assert 'windows' in host_triple()
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

    def build_env(self, hosts_file_path=None, target=None, is_build=False, test_unit=False, uwp=False):
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
            extra_path += [path.join(self.msvc_package_dir("cmake"), "bin")]
            extra_path += [path.join(self.msvc_package_dir("llvm"), "bin")]
            extra_path += [path.join(self.msvc_package_dir("ninja"), "bin")]
            extra_path += [self.msvc_package_dir("nuget")]
            extra_path += [path.join(self.msvc_package_dir("xargo"))]

            arch = (target or host_triple()).split('-')[0]
            vcpkg_arch = {
                "x86_64": "x64-windows",
                "i686": "x86-windows",
                "aarch64": "arm64-windows",
            }
            target_arch = vcpkg_arch[arch]
            if uwp:
                target_arch += "-uwp"
            openssl_base_dir = path.join(self.msvc_package_dir("openssl"), target_arch)

            # Link openssl
            env["OPENSSL_INCLUDE_DIR"] = path.join(openssl_base_dir, "include")
            env["OPENSSL_LIB_DIR"] = path.join(openssl_base_dir, "lib")
            env["OPENSSL_LIBS"] = "libssl:libcrypto"
            # Link moztools, used for building SpiderMonkey
            env["MOZTOOLS_PATH"] = os.pathsep.join([
                path.join(self.msvc_package_dir("moztools"), "bin"),
                path.join(self.msvc_package_dir("moztools"), "msys", "bin"),
            ])
            # Link autoconf 2.13, used for building SpiderMonkey
            env["AUTOCONF"] = path.join(self.msvc_package_dir("moztools"), "msys", "local", "bin", "autoconf-2.13")
            # Link LLVM
            env["LIBCLANG_PATH"] = path.join(self.msvc_package_dir("llvm"), "lib")

            if not os.environ.get("NATIVE_WIN32_PYTHON"):
                env["NATIVE_WIN32_PYTHON"] = sys.executable
            # Always build harfbuzz from source
            env["HARFBUZZ_SYS_NO_PKG_CONFIG"] = "true"

        if is_build and self.needs_gstreamer_env(target or host_triple(), env, uwp):
            gstpath = gstreamer_root(target or host_triple(), env, self.get_top_dir())
            extra_path += [path.join(gstpath, "bin")]
            libpath = path.join(gstpath, "lib")
            # we append in the reverse order so that system gstreamer libraries
            # do not get precedence
            extra_path = [libpath] + extra_path
            extra_lib = [libpath] + extra_path
            append_to_path_env(path.join(libpath, "pkgconfig"), env, "PKG_CONFIG_PATH")

        if sys.platform == "linux2":
            distro, version, _ = platform.linux_distribution()
            if distro == "Ubuntu" and (version == "16.04" or version == "14.04"):
                env["HARFBUZZ_SYS_NO_PKG_CONFIG"] = "true"

        if extra_path:
            append_to_path_env(os.pathsep.join(extra_path), env, "PATH")

        if self.config["build"]["incremental"]:
            env["CARGO_INCREMENTAL"] = "1"
        elif self.config["build"]["incremental"] is not None:
            env["CARGO_INCREMENTAL"] = "0"

        if extra_lib:
            path_var = "DYLD_LIBRARY_PATH" if sys.platform == "darwin" else "LD_LIBRARY_PATH"
            append_to_path_env(os.pathsep.join(extra_lib), env, path_var)

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

        if not test_unit:
            # This wrapper script is in bash and doesn't work on Windows
            # where we want to run doctests as part of `./mach test-unit`
            env['RUSTDOC'] = path.join(self.context.topdir, 'etc', 'rustdoc-with-private')

        if self.config["build"]["rustflags"]:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " " + self.config["build"]["rustflags"]

        # Don't run the gold linker if on Windows https://github.com/servo/servo/issues/9499
        if self.config["tools"]["rustc-with-gold"] and sys.platform != "win32":
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

        if self.config["build"]["thinlto"]:
            env['RUSTFLAGS'] += " -Z thinlto"

        return env

    @staticmethod
    def build_like_command_arguments(decorated_function):
        decorators = [
            CommandArgument(
                '--target', '-t',
                default=None,
                help='Cross compile for given target platform',
            ),
            CommandArgument(
                '--android',
                default=None,
                action='store_true',
                help='Build for Android',
            ),
            CommandArgument(
                '--magicleap',
                default=None,
                action='store_true',
                help='Build for Magic Leap',
            ),
            CommandArgument(
                '--libsimpleservo',
                default=None,
                action='store_true',
                help='Build the libsimpleservo library instead of the servo executable',
            ),
            CommandArgument(
                '--features',
                default=None,
                help='Space-separated list of features to also build',
                nargs='+',
            ),
            CommandArgument(
                '--debug-mozjs',
                default=None,
                action='store_true',
                help='Enable debug assertions in mozjs',
            ),
            CommandArgument(
                '--with-debug-assertions',
                default=None,
                action='store_true',
                help='Enable debug assertions in release',
            ),
            CommandArgument(
                '--with-frame-pointer',
                default=None,
                action='store_true',
                help='Build with frame pointer enabled, used by the background hang monitor.',
            ),
            CommandArgument(
                '--uwp',
                default=None,
                action='store_true',
                help='Build for HoloLens (x64)'),
            CommandArgument('--with-raqote', default=None, action='store_true'),
            CommandArgument('--with-layout-2020', default=None, action='store_true'),
            CommandArgument('--without-wgl', default=None, action='store_true'),
        ]

        for decorator in decorators:
            decorated_function = decorator(decorated_function)
        return decorated_function

    def pick_target_triple(self, target, android, magicleap):
        if android is None:
            android = self.config["build"]["android"]
        if target and android:
            assert self.handle_android_target(target)
        if android and not target:
            target = self.config["android"]["target"]
        if magicleap and not target:
            target = "aarch64-linux-android"
        if target and not android and not magicleap:
            android = self.handle_android_target(target)
        return target, android

    def run_cargo_build_like_command(
        self, command, cargo_args,
        env=None, verbose=False,
        target=None, android=False, magicleap=False, libsimpleservo=False,
        features=None, debug_mozjs=False, with_debug_assertions=False,
        with_frame_pointer=False, with_raqote=False, with_layout_2020=False, without_wgl=False,
        uwp=False,
    ):
        env = env or self.build_env()
        target, android = self.pick_target_triple(target, android, magicleap)

        args = []
        if "--manifest-path" not in args:
            if libsimpleservo or android:
                if android:
                    api = "jniapi"
                else:
                    api = "capi"
                port = path.join("libsimpleservo", api)
            else:
                port = "glutin"
            args += [
                "--manifest-path",
                path.join(self.context.topdir, "ports", port, "Cargo.toml"),
            ]
        if target:
            args += ["--target", target]

        if features is None:  # If we're passed a list, mutate it even if it's empty
            features = []
        if self.config["build"]["debug-mozjs"] or debug_mozjs:
            features.append("debugmozjs")
        if not magicleap:
            features.append("native-bluetooth")
        if uwp:
            features.append("canvas2d-raqote")
            features.append("no_wgl")
            features.append("uwp")
        else:
            # Non-UWP builds provide their own libEGL via mozangle.
            features.append("egl")
        if with_raqote and "canvas2d-azure" not in features:
            features.append("canvas2d-raqote")
        elif "canvas2d-raqote" not in features:
            features.append("canvas2d-azure")
        if with_layout_2020 and "layout-2013" not in features:
            features.append("layout-2020")
        elif "layout-2020" not in features:
            features.append("layout-2013")
        if with_frame_pointer:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C force-frame-pointers=yes"
            features.append("profilemozjs")
        if without_wgl:
            features.append("no_wgl")
        if self.config["build"]["webgl-backtrace"]:
            features.append("webgl-backtrace")
        if self.config["build"]["dom-backtrace"]:
            features.append("dom-backtrace")
        if with_debug_assertions:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C debug_assertions"

        assert "--features" not in cargo_args
        args += ["--features", " ".join(features)]

        if target and 'uwp' in target:
            return call(["xargo", command] + args + cargo_args, env=env, verbose=verbose)
        else:
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

    def handle_android_target(self, target):
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

    def ensure_bootstrapped(self, target=None):
        if self.context.bootstrapped:
            return

        target_platform = target or host_triple()

        # Always check if all needed MSVC dependencies are installed
        if "msvc" in target_platform:
            Registrar.dispatch("bootstrap", context=self.context)

        self.context.bootstrapped = True

    def ensure_clobbered(self, target_dir=None):
        if target_dir is None:
            target_dir = self.get_target_dir()
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
    print("Can't find MSBuild.exe installation under %s." % base_vs_path)
    sys.exit(1)


def get_msbuild_version(vs_version):
    if vs_version in ("15.0", "14.0"):
        msbuild_version = vs_version
    else:
        msbuild_version = "Current"
    return msbuild_version

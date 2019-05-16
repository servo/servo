# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import datetime
import os
import os.path as path
import platform
import shutil
import subprocess
import sys
import urllib
import zipfile
import stat

from time import time

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)
from mach.registrar import Registrar

from mach_bootstrap import _get_exec_path
from servo.command_base import CommandBase, cd, call, check_call, BIN_SUFFIX
from servo.util import host_triple


def format_duration(seconds):
    return str(datetime.timedelta(seconds=int(seconds)))


def notify_linux(title, text):
    try:
        import dbus
        bus = dbus.SessionBus()
        notify_obj = bus.get_object("org.freedesktop.Notifications", "/org/freedesktop/Notifications")
        method = notify_obj.get_dbus_method("Notify", "org.freedesktop.Notifications")
        method(title, 0, "", text, "", [], {"transient": True}, -1)
    except:
        raise Exception("Optional Python module 'dbus' is not installed.")


def notify_win(title, text):
    try:
        from servo.win32_toast import WindowsToast
        w = WindowsToast()
        w.balloon_tip(title, text)
    except:
        from ctypes import Structure, windll, POINTER, sizeof
        from ctypes.wintypes import DWORD, HANDLE, WINFUNCTYPE, BOOL, UINT

        class FLASHWINDOW(Structure):
            _fields_ = [("cbSize", UINT),
                        ("hwnd", HANDLE),
                        ("dwFlags", DWORD),
                        ("uCount", UINT),
                        ("dwTimeout", DWORD)]

        FlashWindowExProto = WINFUNCTYPE(BOOL, POINTER(FLASHWINDOW))
        FlashWindowEx = FlashWindowExProto(("FlashWindowEx", windll.user32))
        FLASHW_CAPTION = 0x01
        FLASHW_TRAY = 0x02
        FLASHW_TIMERNOFG = 0x0C

        params = FLASHWINDOW(sizeof(FLASHWINDOW),
                             windll.kernel32.GetConsoleWindow(),
                             FLASHW_CAPTION | FLASHW_TRAY | FLASHW_TIMERNOFG, 3, 0)
        FlashWindowEx(params)


def notify_darwin(title, text):
    try:
        import Foundation

        bundleDict = Foundation.NSBundle.mainBundle().infoDictionary()
        bundleIdentifier = 'CFBundleIdentifier'
        if bundleIdentifier not in bundleDict:
            bundleDict[bundleIdentifier] = 'mach'

        note = Foundation.NSUserNotification.alloc().init()
        note.setTitle_(title)
        note.setInformativeText_(text)

        now = Foundation.NSDate.dateWithTimeInterval_sinceDate_(0, Foundation.NSDate.date())
        note.setDeliveryDate_(now)

        centre = Foundation.NSUserNotificationCenter.defaultUserNotificationCenter()
        centre.scheduleNotification_(note)
    except ImportError:
        raise Exception("Optional Python module 'pyobjc' is not installed.")


def notify_with_command(command):
    def notify(title, text):
        if call([command, title, text]) != 0:
            raise Exception("Could not run '%s'." % command)
    return notify


def notify_build_done(config, elapsed, success=True):
    """Generate desktop notification when build is complete and the
    elapsed build time was longer than 30 seconds."""
    if elapsed > 30:
        notify(config, "Servo build",
               "%s in %s" % ("Completed" if success else "FAILED", format_duration(elapsed)))


def notify(config, title, text):
    """Generate a desktop notification using appropriate means on
    supported platforms Linux, Windows, and Mac OS.  On unsupported
    platforms, this function acts as a no-op.

    If notify-command is set in the [tools] section of the configuration,
    that is used instead."""
    notify_command = config["tools"].get("notify-command")
    if notify_command:
        func = notify_with_command(notify_command)
    else:
        platforms = {
            "linux": notify_linux,
            "linux2": notify_linux,
            "win32": notify_win,
            "darwin": notify_darwin
        }
        func = platforms.get(sys.platform)

    if func is not None:
        try:
            func(title, text)
        except Exception as e:
            extra = getattr(e, "message", "")
            print("[Warning] Could not generate notification! %s" % extra, file=sys.stderr)


@CommandProvider
class MachCommands(CommandBase):
    @Command('build',
             description='Build Servo',
             category='build')
    @CommandArgument('--target', '-t',
                     default=None,
                     help='Cross compile for given target platform')
    @CommandArgument('--release', '-r',
                     action='store_true',
                     help='Build in release mode')
    @CommandArgument('--dev', '-d',
                     action='store_true',
                     help='Build in development mode')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    @CommandArgument('--features',
                     default=None,
                     help='Space-separated list of features to also build',
                     nargs='+')
    @CommandArgument('--android',
                     default=None,
                     action='store_true',
                     help='Build for Android')
    @CommandArgument('--magicleap',
                     default=None,
                     action='store_true',
                     help='Build for Magic Leap')
    @CommandArgument('--no-package',
                     action='store_true',
                     help='For Android, disable packaging into a .apk after building')
    @CommandArgument('--debug-mozjs',
                     default=None,
                     action='store_true',
                     help='Enable debug assertions in mozjs')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    @CommandArgument('--very-verbose', '-vv',
                     action='store_true',
                     help='Print very verbose output')
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    @CommandArgument('--with-debug-assertions',
                     default=None,
                     action='store_true',
                     help='Enable debug assertions in release')
    @CommandArgument('--libsimpleservo',
                     default=None,
                     action='store_true',
                     help='Build the libsimpleservo library instead of the servo executable')
    @CommandArgument('--with-frame-pointer',
                     default=None,
                     action='store_true',
                     help='Build with frame pointer enabled, used by the background hang monitor.')
    def build(self, target=None, release=False, dev=False, jobs=None,
              features=None, android=None, magicleap=None, no_package=False, verbose=False, very_verbose=False,
              debug_mozjs=False, params=None, with_debug_assertions=False,
              libsimpleservo=False, with_frame_pointer=False):

        opts = params or []

        if android is None:
            android = self.config["build"]["android"]
        features = features or self.servo_features()

        if target and android:
            print("Please specify either --target or --android.")
            sys.exit(1)

        if android:
            target = self.config["android"]["target"]

        if not magicleap:
            features += ["native-bluetooth"]

        if magicleap and not target:
            target = "aarch64-linux-android"

        if target and not android and not magicleap:
            android = self.handle_android_target(target)

        target_path = base_path = self.get_target_dir()
        if android:
            target_path = path.join(target_path, "android")
            base_path = path.join(target_path, target)
        elif magicleap:
            target_path = path.join(target_path, "magicleap")
            base_path = path.join(target_path, target)
        release_path = path.join(base_path, "release", "servo")
        dev_path = path.join(base_path, "debug", "servo")

        release_exists = path.exists(release_path)
        dev_exists = path.exists(dev_path)

        if not (release or dev):
            if self.config["build"]["mode"] == "dev":
                dev = True
            elif self.config["build"]["mode"] == "release":
                release = True
            elif release_exists and not dev_exists:
                release = True
            elif dev_exists and not release_exists:
                dev = True
            else:
                print("Please specify either --dev (-d) for a development")
                print("  build, or --release (-r) for an optimized build.")
                sys.exit(1)

        if release and dev:
            print("Please specify either --dev or --release.")
            sys.exit(1)

        if release:
            opts += ["--release"]
            servo_path = release_path
        else:
            servo_path = dev_path

        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if very_verbose:
            opts += ["-vv"]

        if target:
            if self.config["tools"]["use-rustup"]:
                # 'rustup target add' fails if the toolchain is not installed at all.
                self.call_rustup_run(["rustc", "--version"])

                check_call(["rustup" + BIN_SUFFIX, "target", "add",
                            "--toolchain", self.toolchain(), target])

            opts += ["--target", target]

        env = self.build_env(target=target, is_build=True)
        self.ensure_bootstrapped(target=target)
        self.ensure_clobbered()

        self.add_manifest_path(opts, android, libsimpleservo)

        if debug_mozjs:
            features += ["debugmozjs"]

        if with_frame_pointer:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C force-frame-pointers=yes"
            features += ["profilemozjs"]

        if self.config["build"]["webgl-backtrace"]:
            features += ["webgl-backtrace"]
        if self.config["build"]["dom-backtrace"]:
            features += ["dom-backtrace"]

        if features:
            opts += ["--features", "%s" % ' '.join(features)]

        build_start = time()
        env["CARGO_TARGET_DIR"] = target_path

        if with_debug_assertions:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C debug_assertions"

        if sys.platform == "win32":
            env["CC"] = "clang-cl.exe"
            env["CXX"] = "clang-cl.exe"

        host = host_triple()
        if 'apple-darwin' in host and (not target or target == host):
            if 'CXXFLAGS' not in env:
                env['CXXFLAGS'] = ''
            env["CXXFLAGS"] += "-mmacosx-version-min=10.10"

        if android:
            if "ANDROID_NDK" not in env:
                print("Please set the ANDROID_NDK environment variable.")
                sys.exit(1)
            if "ANDROID_SDK" not in env:
                print("Please set the ANDROID_SDK environment variable.")
                sys.exit(1)

            android_platform = self.config["android"]["platform"]
            android_toolchain_name = self.config["android"]["toolchain_name"]
            android_toolchain_prefix = self.config["android"]["toolchain_prefix"]
            android_lib = self.config["android"]["lib"]
            android_arch = self.config["android"]["arch"]

            # Build OpenSSL for android
            env["OPENSSL_VERSION"] = "1.0.2k"
            make_cmd = ["make"]
            if jobs is not None:
                make_cmd += ["-j" + jobs]
            openssl_dir = path.join(target_path, target, "native", "openssl")
            if not path.exists(openssl_dir):
                os.makedirs(openssl_dir)
            shutil.copy(path.join(self.android_support_dir(), "openssl.makefile"), openssl_dir)
            shutil.copy(path.join(self.android_support_dir(), "openssl.sh"), openssl_dir)

            # Check if the NDK version is 15
            if not os.path.isfile(path.join(env["ANDROID_NDK"], 'source.properties')):
                print("ANDROID_NDK should have file `source.properties`.")
                print("The environment variable ANDROID_NDK may be set at a wrong path.")
                sys.exit(1)
            with open(path.join(env["ANDROID_NDK"], 'source.properties')) as ndk_properties:
                lines = ndk_properties.readlines()
                if lines[1].split(' = ')[1].split('.')[0] != '15':
                    print("Currently only support NDK 15. Please re-run `./mach bootstrap-android`.")
                    sys.exit(1)

            env["RUST_TARGET"] = target
            env["ANDROID_TOOLCHAIN_NAME"] = android_toolchain_name
            with cd(openssl_dir):
                status = call(
                    make_cmd + ["-f", "openssl.makefile"],
                    env=env,
                    verbose=verbose)
                if status:
                    return status
            openssl_dir = path.join(openssl_dir, "openssl-{}".format(env["OPENSSL_VERSION"]))
            env['OPENSSL_LIB_DIR'] = openssl_dir
            env['OPENSSL_INCLUDE_DIR'] = path.join(openssl_dir, "include")
            env['OPENSSL_STATIC'] = 'TRUE'
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

            host_cc = env.get('HOST_CC') or _get_exec_path(["clang"]) or _get_exec_path(["gcc"])
            host_cxx = env.get('HOST_CXX') or _get_exec_path(["clang++"]) or _get_exec_path(["g++"])

            llvm_toolchain = path.join(env['ANDROID_NDK'], "toolchains", "llvm", "prebuilt", host)
            gcc_toolchain = path.join(env['ANDROID_NDK'], "toolchains",
                                      android_toolchain_prefix + "-4.9", "prebuilt", host)
            gcc_libs = path.join(gcc_toolchain, "lib", "gcc", android_toolchain_name, "4.9.x")

            env['PATH'] = (path.join(llvm_toolchain, "bin") + ':'
                           + path.join(gcc_toolchain, "bin") + ':'
                           + env['PATH'])
            env['ANDROID_SYSROOT'] = path.join(env['ANDROID_NDK'], "sysroot")
            support_include = path.join(env['ANDROID_NDK'], "sources", "android", "support", "include")
            cpufeatures_include = path.join(env['ANDROID_NDK'], "sources", "android", "cpufeatures")
            cxx_include = path.join(env['ANDROID_NDK'], "sources", "cxx-stl",
                                    "llvm-libc++", "include")
            clang_include = path.join(llvm_toolchain, "lib64", "clang", "3.8", "include")
            cxxabi_include = path.join(env['ANDROID_NDK'], "sources", "cxx-stl",
                                       "llvm-libc++abi", "include")
            sysroot_include = path.join(env['ANDROID_SYSROOT'], "usr", "include")
            arch_include = path.join(sysroot_include, android_toolchain_name)
            android_platform_dir = path.join(env['ANDROID_NDK'], "platforms", android_platform, "arch-" + android_arch)
            arch_libs = path.join(android_platform_dir, "usr", "lib")
            clang_include = path.join(llvm_toolchain, "lib64", "clang", "5.0", "include")
            android_api = android_platform.replace('android-', '')
            env['HOST_CC'] = host_cc
            env['HOST_CXX'] = host_cxx
            env['HOST_CFLAGS'] = ''
            env['HOST_CXXFLAGS'] = ''
            env['CC'] = path.join(llvm_toolchain, "bin", "clang")
            env['CPP'] = path.join(llvm_toolchain, "bin", "clang") + " -E"
            env['CXX'] = path.join(llvm_toolchain, "bin", "clang++")
            env['ANDROID_TOOLCHAIN'] = gcc_toolchain
            env['ANDROID_TOOLCHAIN_DIR'] = gcc_toolchain
            env['ANDROID_VERSION'] = android_api
            env['ANDROID_PLATFORM_DIR'] = android_platform_dir
            env['GCC_TOOLCHAIN'] = gcc_toolchain
            gcc_toolchain_bin = path.join(gcc_toolchain, android_toolchain_name, "bin")
            env['AR'] = path.join(gcc_toolchain_bin, "ar")
            env['RANLIB'] = path.join(gcc_toolchain_bin, "ranlib")
            env['OBJCOPY'] = path.join(gcc_toolchain_bin, "objcopy")
            env['YASM'] = path.join(env['ANDROID_NDK'], 'prebuilt', host, 'bin', 'yasm')
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
            env['CFLAGS'] = ' '.join([
                "--target=" + target,
                "--sysroot=" + env['ANDROID_SYSROOT'],
                "--gcc-toolchain=" + gcc_toolchain,
                "-isystem", sysroot_include,
                "-I" + arch_include,
                "-B" + arch_libs,
                "-L" + arch_libs,
                "-D__ANDROID_API__=" + android_api,
            ])
            env['CXXFLAGS'] = ' '.join([
                "--target=" + target,
                "--sysroot=" + env['ANDROID_SYSROOT'],
                "--gcc-toolchain=" + gcc_toolchain,
                "-I" + cpufeatures_include,
                "-I" + cxx_include,
                "-I" + clang_include,
                "-isystem", sysroot_include,
                "-I" + cxxabi_include,
                "-I" + clang_include,
                "-I" + arch_include,
                "-I" + support_include,
                "-L" + gcc_libs,
                "-B" + arch_libs,
                "-L" + arch_libs,
                "-D__ANDROID_API__=" + android_api,
                "-D__STDC_CONSTANT_MACROS",
                "-D__NDK_FPABI__=",
            ])
            env['CPPFLAGS'] = ' '.join([
                "--target=" + target,
                "--sysroot=" + env['ANDROID_SYSROOT'],
                "-I" + arch_include,
            ])
            env["NDK_ANDROID_VERSION"] = android_api
            env["ANDROID_ABI"] = android_lib
            env["ANDROID_PLATFORM"] = android_platform
            env["ANDROID_TOOLCHAIN_NAME"] = "clang"
            env["NDK_CMAKE_TOOLCHAIN_FILE"] = path.join(env['ANDROID_NDK'], "build", "cmake", "android.toolchain.cmake")
            env["CMAKE_TOOLCHAIN_FILE"] = path.join(self.android_support_dir(), "toolchain.cmake")
            # Set output dir for gradle aar files
            aar_out_dir = self.android_aar_dir()
            if not os.path.exists(aar_out_dir):
                os.makedirs(aar_out_dir)
            env["AAR_OUT_DIR"] = aar_out_dir
            # GStreamer and its dependencies use pkg-config and this flag is required
            # to make it work in a cross-compilation context.
            env["PKG_CONFIG_ALLOW_CROSS"] = '1'
            # Build the name of the package containing all GStreamer dependencies
            # according to the build target.
            gst_lib = "gst-build-{}".format(self.config["android"]["lib"])
            gst_lib_zip = "gstreamer-{}-1.14.3-20190201-081639.zip".format(self.config["android"]["lib"])
            gst_dir = os.path.join(target_path, "gstreamer")
            gst_lib_path = os.path.join(gst_dir, gst_lib)
            pkg_config_path = os.path.join(gst_lib_path, "pkgconfig")
            env["PKG_CONFIG_PATH"] = pkg_config_path
            if not os.path.exists(gst_lib_path):
                # Download GStreamer dependencies if they have not already been downloaded
                # This bundle is generated with `libgstreamer_android_gen`
                # Follow these instructions to build and deploy new binaries
                # https://github.com/servo/libgstreamer_android_gen#build
                print("Downloading GStreamer dependencies")
                gst_url = "https://servo-deps.s3.amazonaws.com/gstreamer/%s" % gst_lib_zip
                print(gst_url)
                urllib.urlretrieve(gst_url, gst_lib_zip)
                zip_ref = zipfile.ZipFile(gst_lib_zip, "r")
                zip_ref.extractall(gst_dir)
                os.remove(gst_lib_zip)

                # Change pkgconfig info to make all GStreamer dependencies point
                # to the libgstreamer_android.so bundle.
                for each in os.listdir(pkg_config_path):
                    if each.endswith('.pc'):
                        print("Setting pkgconfig info for %s" % each)
                        pc = os.path.join(pkg_config_path, each)
                        expr = "s#libdir=.*#libdir=%s#g" % gst_lib_path
                        subprocess.call(["perl", "-i", "-pe", expr, pc])

        if magicleap:
            if platform.system() not in ["Darwin"]:
                raise Exception("Magic Leap builds are only supported on macOS. "
                                "If you only wish to test if your code builds, "
                                "run ./mach build -p libmlservo.")

            ml_sdk = env.get("MAGICLEAP_SDK")
            if not ml_sdk:
                raise Exception("Magic Leap builds need the MAGICLEAP_SDK environment variable")
            if not os.path.exists(ml_sdk):
                raise Exception("Path specified by MAGICLEAP_SDK does not exist.")

            ml_support = path.join(self.get_top_dir(), "support", "magicleap")

            # We pretend to be an Android build
            env.setdefault("ANDROID_VERSION", "21")
            env.setdefault("ANDROID_NDK", env["MAGICLEAP_SDK"])
            env.setdefault("ANDROID_NDK_VERSION", "16.0.0")
            env.setdefault("ANDROID_PLATFORM_DIR", path.join(env["MAGICLEAP_SDK"], "lumin"))
            env.setdefault("ANDROID_TOOLCHAIN_DIR", path.join(env["MAGICLEAP_SDK"], "tools", "toolchains"))
            env.setdefault("ANDROID_CLANG", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "clang"))

            # A random collection of search paths
            env.setdefault("STLPORT_LIBS", " ".join([
                "-L" + path.join(env["MAGICLEAP_SDK"], "lumin", "stl", "libc++-lumin", "lib"),
                "-lc++"
            ]))
            env.setdefault("STLPORT_CPPFLAGS", " ".join([
                "-I" + path.join(env["MAGICLEAP_SDK"], "lumin", "stl", "libc++-lumin", "include")
            ]))
            env.setdefault("CPPFLAGS", " ".join([
                "--no-standard-includes",
                "--sysroot=" + env["ANDROID_PLATFORM_DIR"],
                "-I" + path.join(env["ANDROID_PLATFORM_DIR"], "usr", "include"),
                "-isystem" + path.join(env["ANDROID_TOOLCHAIN_DIR"], "lib64", "clang", "3.8", "include"),
            ]))
            env.setdefault("CFLAGS", " ".join([
                env["CPPFLAGS"],
                "-L" + path.join(env["ANDROID_TOOLCHAIN_DIR"], "lib", "gcc", target, "4.9.x"),
            ]))
            env.setdefault("CXXFLAGS", " ".join([
                # Sigh, Angle gets confused if there's another EGL around
                "-I./gfx/angle/checkout/include",
                env["STLPORT_CPPFLAGS"],
                env["CFLAGS"]
            ]))

            # The toolchain commands
            env.setdefault("AR", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-ar"))
            env.setdefault("AS", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-clang"))
            env.setdefault("CC", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-clang"))
            env.setdefault("CPP", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-clang -E"))
            env.setdefault("CXX", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-clang++"))
            env.setdefault("LD", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-ld"))
            env.setdefault("OBJCOPY", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-objcopy"))
            env.setdefault("OBJDUMP", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-objdump"))
            env.setdefault("RANLIB", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-ranlib"))
            env.setdefault("STRIP", path.join(env["ANDROID_TOOLCHAIN_DIR"], "bin", "aarch64-linux-android-strip"))

            # Undo all of that when compiling build tools for the host
            env.setdefault("HOST_CFLAGS", "")
            env.setdefault("HOST_CXXFLAGS", "")
            env.setdefault("HOST_CC", "/usr/local/opt/llvm/bin/clang")
            env.setdefault("HOST_CXX", "/usr/local/opt/llvm/bin/clang++")
            env.setdefault("HOST_LD", "ld")

            # Some random build configurations
            env.setdefault("HARFBUZZ_SYS_NO_PKG_CONFIG", "1")
            env.setdefault("PKG_CONFIG_ALLOW_CROSS", "1")
            env.setdefault("CMAKE_TOOLCHAIN_FILE", path.join(ml_support, "toolchain.cmake"))
            env.setdefault("_LIBCPP_INLINE_VISIBILITY", "__attribute__((__always_inline__))")

            # The Open SSL configuration
            env.setdefault("OPENSSL_DIR", path.join(target_path, target, "native", "openssl"))
            env.setdefault("OPENSSL_VERSION", "1.0.2k")
            env.setdefault("OPENSSL_STATIC", "1")

            # Override the linker set in .cargo/config
            env.setdefault("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", path.join(ml_support, "fake-ld.sh"))

            # Only build libmlservo
            opts += ["--package", "libmlservo"]

            # Download and build OpenSSL if necessary
            status = call(path.join(ml_support, "openssl.sh"), env=env, verbose=verbose)
            if status:
                return status

        if very_verbose:
            print (["Calling", "cargo", "build"] + opts)
            for key in env:
                print((key, env[key]))

        status = self.call_rustup_run(["cargo", "build"] + opts, env=env, verbose=verbose)
        elapsed = time() - build_start

        # Do some additional things if the build succeeded
        if status == 0:
            if android and not no_package:
                flavor = None
                if "googlevr" in features:
                    flavor = "googlevr"
                elif "oculusvr" in features:
                    flavor = "oculusvr"
                rv = Registrar.dispatch("package", context=self.context,
                                        release=release, dev=dev, target=target, flavor=flavor)
                if rv:
                    return rv

            if sys.platform == "win32":
                servo_exe_dir = path.join(base_path, "debug" if dev else "release")

                msvc_x64 = "64" if "x86_64" in (target or host_triple()) else ""
                # on msvc builds, use editbin to change the subsystem to windows, but only
                # on release builds -- on debug builds, it hides log output
                if not dev:
                    call(["editbin", "/nologo", "/subsystem:windows", path.join(servo_exe_dir, "servo.exe")],
                         verbose=verbose)
                # on msvc, we need to copy in some DLLs in to the servo.exe dir
                for ssl_lib in ["libcryptoMD.dll", "libsslMD.dll"]:
                    shutil.copy(path.join(env['OPENSSL_LIB_DIR'], "../bin" + msvc_x64, ssl_lib),
                                servo_exe_dir)
                # Search for the generated nspr4.dll
                build_path = path.join(servo_exe_dir, "build")

                def package_generated_shared_libraries(libs, build_path, servo_exe_dir):
                    for root, dirs, files in os.walk(build_path):
                        remaining_libs = list(libs)
                        for lib in libs:
                            if lib in files:
                                shutil.copy(path.join(root, lib), servo_exe_dir)
                                remaining_libs.remove(lib)
                                continue
                        libs = remaining_libs
                        if not libs:
                            return
                    for lib in libs:
                        print("WARNING: could not find " + lib)

                package_generated_shared_libraries(["nspr4.dll", "libEGL.dll"], build_path, servo_exe_dir)

                # copy needed gstreamer DLLs in to servo.exe dir
                gst_x64 = "X86_64" if msvc_x64 == "64" else "X86"
                gst_root = ""
                gst_default_path = path.join("C:\\gstreamer\\1.0", gst_x64)
                gst_env = "GSTREAMER_1_0_ROOT_" + gst_x64
                if os.path.exists(path.join(gst_default_path, "bin", "libffi-7.dll")) or \
                   os.path.exists(path.join(gst_default_path, "bin", "ffi-7.dll")):
                    gst_root = gst_default_path
                elif os.environ.get(gst_env) is not None:
                    gst_root = os.environ.get(gst_env)
                else:
                    print("Could not found GStreamer installation directory.")
                    status = 1
                gst_dlls = [
                    ["libffi-7.dll", "ffi-7.dll"],
                    ["libgio-2.0-0.dll", "gio-2.0-0.dll"],
                    ["libglib-2.0-0.dll", "glib-2.0-0.dll"],
                    ["libgmodule-2.0-0.dll", "gmodule-2.0-0.dll"],
                    ["libgobject-2.0-0.dll", "gobject-2.0-0.dll"],
                    ["libgstapp-1.0-0.dll", "gstapp-1.0-0.dll"],
                    ["libgstaudio-1.0-0.dll", "gstaudio-1.0-0.dll"],
                    ["libgstbase-1.0-0.dll", "gstbase-1.0-0.dll"],
                    ["libgstgl-1.0-0.dll", "gstgl-1.0-0.dll"],
                    ["libgstpbutils-1.0-0.dll", "gstpbutils-1.0-0.dll"],
                    ["libgstplayer-1.0-0.dll", "gstplayer-1.0-0.dll"],
                    ["libgstreamer-1.0-0.dll", "gstreamer-1.0-0.dll"],
                    ["libgstrtp-1.0-0.dll", "gstrtp-1.0-0.dll"],
                    ["libgstsdp-1.0-0.dll", "gstsdp-1.0-0.dll"],
                    ["libgsttag-1.0-0.dll", "gsttag-1.0-0.dll"],
                    ["libgstvideo-1.0-0.dll", "gstvideo-1.0-0.dll"],
                    ["libgstwebrtc-1.0-0.dll", "gstwebrtc-1.0-0.dll"],
                    ["libintl-8.dll", "intl-8.dll"],
                    ["liborc-0.4-0.dll", "orc-0.4-0.dll"],
                    ["libwinpthread-1.dll", "winpthread-1.dll"],
                    ["libz.dll", "libz-1.dll", "z-1.dll"]
                ]
                if gst_root:
                    for gst_lib in gst_dlls:
                        if isinstance(gst_lib, str):
                            gst_lib = [gst_lib]
                        for lib in gst_lib:
                            try:
                                shutil.copy(path.join(gst_root, "bin", lib),
                                            servo_exe_dir)
                                break
                            except:
                                pass
                        else:
                            print("ERROR: could not find required GStreamer DLL: " + str(gst_lib))
                            sys.exit(1)

                # copy some MSVC DLLs to servo.exe dir
                msvc_redist_dir = None
                vs_platform = os.environ.get("PLATFORM", "").lower()
                vc_dir = os.environ.get("VCINSTALLDIR", "")
                vs_version = os.environ.get("VisualStudioVersion", "")
                msvc_deps = [
                    "api-ms-win-crt-runtime-l1-1-0.dll",
                    "msvcp140.dll",
                    "vcruntime140.dll",
                ]
                # Check if it's Visual C++ Build Tools or Visual Studio 2015
                vs14_vcvars = path.join(vc_dir, "vcvarsall.bat")
                is_vs14 = True if os.path.isfile(vs14_vcvars) or vs_version == "14.0" else False
                if is_vs14:
                    msvc_redist_dir = path.join(vc_dir, "redist", vs_platform, "Microsoft.VC140.CRT")
                elif vs_version == "15.0":
                    redist_dir = path.join(os.environ.get("VCINSTALLDIR", ""), "Redist", "MSVC")
                    if os.path.isdir(redist_dir):
                        for p in os.listdir(redist_dir)[::-1]:
                            redist_path = path.join(redist_dir, p)
                            for v in ["VC141", "VC150"]:
                                # there are two possible paths
                                # `x64\Microsoft.VC*.CRT` or `onecore\x64\Microsoft.VC*.CRT`
                                redist1 = path.join(redist_path, vs_platform, "Microsoft.{}.CRT".format(v))
                                redist2 = path.join(redist_path, "onecore", vs_platform, "Microsoft.{}.CRT".format(v))
                                if os.path.isdir(redist1):
                                    msvc_redist_dir = redist1
                                    break
                                elif os.path.isdir(redist2):
                                    msvc_redist_dir = redist2
                                    break
                            if msvc_redist_dir:
                                break
                if msvc_redist_dir:
                    redist_dirs = [
                        msvc_redist_dir,
                        path.join(os.environ["WindowsSdkDir"], "Redist", "ucrt", "DLLs", vs_platform),
                    ]
                    for msvc_dll in msvc_deps:
                        dll_found = False
                        for dll_dir in redist_dirs:
                            dll = path.join(dll_dir, msvc_dll)
                            servo_dir_dll = path.join(servo_exe_dir, msvc_dll)
                            if os.path.isfile(dll):
                                if os.path.isfile(servo_dir_dll):
                                    # avoid permission denied error when overwrite dll in servo build directory
                                    os.chmod(servo_dir_dll, stat.S_IWUSR)
                                shutil.copy(dll, servo_exe_dir)
                                dll_found = True
                                break
                        if not dll_found:
                            print("DLL file `{}` not found!".format(msvc_dll))
                            status = 1

            elif sys.platform == "darwin":
                # On the Mac, set a lovely icon. This makes it easier to pick out the Servo binary in tools
                # like Instruments.app.
                try:
                    import Cocoa
                    icon_path = path.join(self.get_top_dir(), "resources", "servo.png")
                    icon = Cocoa.NSImage.alloc().initWithContentsOfFile_(icon_path)
                    if icon is not None:
                        Cocoa.NSWorkspace.sharedWorkspace().setIcon_forFile_options_(icon,
                                                                                     servo_path,
                                                                                     0)
                except ImportError:
                    pass

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(self.config, elapsed, status == 0)

        print("Build %s in %s" % ("Completed" if status == 0 else "FAILED", format_duration(elapsed)))
        return status

    @Command('clean',
             description='Clean the build directory.',
             category='build')
    @CommandArgument('--manifest-path',
                     default=None,
                     help='Path to the manifest to the package to clean')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    def clean(self, manifest_path=None, params=[], verbose=False):
        self.ensure_bootstrapped()

        virtualenv_path = path.join(self.get_top_dir(), 'python', '_virtualenv')
        if path.exists(virtualenv_path):
            print('Removing virtualenv directory: %s' % virtualenv_path)
            shutil.rmtree(virtualenv_path)

        opts = []
        if manifest_path:
            opts += ["--manifest-path", manifest_path]
        if verbose:
            opts += ["-v"]
        opts += params
        return check_call(["cargo", "clean"] + opts,
                          env=self.build_env(), cwd=self.ports_glutin_crate(), verbose=verbose)

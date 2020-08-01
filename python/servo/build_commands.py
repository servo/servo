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
import locale
import os
import os.path as path
import platform
import shutil
import subprocess
import sys
import six.moves.urllib as urllib
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
from servo.command_base import CommandBase, cd, call, check_call, append_to_path_env, gstreamer_root
from servo.gstreamer import windows_dlls, windows_plugins, macos_dylibs, macos_plugins
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
    except ImportError:
        raise Exception("Optional Python module 'dbus' is not installed.")


def notify_win(title, text):
    try:
        from servo.win32_toast import WindowsToast
        w = WindowsToast()
        w.balloon_tip(title, text)
    except WindowsError:
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
    @CommandArgument('--release', '-r',
                     action='store_true',
                     help='Build in release mode')
    @CommandArgument('--dev', '-d',
                     action='store_true',
                     help='Build in development mode')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    @CommandArgument('--no-package',
                     action='store_true',
                     help='For Android, disable packaging into a .apk after building')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    @CommandArgument('--very-verbose', '-vv',
                     action='store_true',
                     help='Print very verbose output')
    @CommandArgument('--uwp',
                     action='store_true',
                     help='Build for HoloLens (x64)')
    @CommandArgument('--win-arm64', action='store_true', help="Use arm64 Windows target")
    @CommandArgument('--with-asan', action='store_true', help="Enable AddressSanitizer")
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    @CommandBase.build_like_command_arguments
    def build(self, release=False, dev=False, jobs=None, params=None, media_stack=None,
              no_package=False, verbose=False, very_verbose=False,
              target=None, android=False, magicleap=False, libsimpleservo=False,
              features=None, uwp=False, win_arm64=False, with_asan=False, **kwargs):
        # Force the UWP-enabled target if the convenience UWP flags are passed.
        if uwp and not target:
            if win_arm64:
                target = 'aarch64-uwp-windows-msvc'
            else:
                target = 'x86_64-uwp-windows-msvc'

        opts = params or []
        features = features or []

        target, android = self.pick_target_triple(target, android, magicleap)

        # Infer UWP build if only provided a target.
        if not uwp:
            uwp = target and 'uwp' in target

        features += self.pick_media_stack(media_stack, target)

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

        env = self.build_env(target=target, is_build=True, uwp=uwp, features=features)
        self.ensure_bootstrapped(target=target)
        self.ensure_clobbered()

        build_start = time()
        env["CARGO_TARGET_DIR"] = target_path

        host = host_triple()
        target_triple = target or host_triple()
        if 'apple-darwin' in host and target_triple == host:
            if 'CXXFLAGS' not in env:
                env['CXXFLAGS'] = ''
            env["CXXFLAGS"] += "-mmacosx-version-min=10.10"

        if 'windows' in host:
            vs_dirs = self.vs_dirs()

        if host != target_triple and 'windows' in target_triple:
            if os.environ.get('VisualStudioVersion') or os.environ.get('VCINSTALLDIR'):
                print("Can't cross-compile for Windows inside of a Visual Studio shell.\n"
                      "Please run `python mach build [arguments]` to bypass automatic "
                      "Visual Studio shell, and make sure the VisualStudioVersion and "
                      "VCINSTALLDIR environment variables are not set.")
                sys.exit(1)
            vcinstalldir = vs_dirs['vcdir']
            if not os.path.exists(vcinstalldir):
                print("Can't find Visual C++ %s installation at %s." % (vs_dirs['vs_version'], vcinstalldir))
                sys.exit(1)

            env['PKG_CONFIG_ALLOW_CROSS'] = "1"

        if uwp:
            # Ensure libstd is ready for the new UWP target.
            check_call(["rustup", "component", "add", "rust-src"])
            env['RUST_SYSROOT'] = path.expanduser('~\\.xargo')

            # Don't try and build a desktop port.
            libsimpleservo = True

            arches = {
                "aarch64": {
                    "angle": "arm64",
                    "gst": "ARM64",
                    "gst_root": "arm64",
                },
                "x86_64": {
                    "angle": "x64",
                    "gst": "X86_64",
                    "gst_root": "x64",
                },
            }
            arch = arches.get(target_triple.split('-')[0])
            if not arch:
                print("Unsupported UWP target.")
                sys.exit(1)

            # Ensure that the NuGet ANGLE package containing libEGL is accessible
            # to the Rust linker.
            append_to_path_env(angle_root(target_triple, env), env, "LIB")

            # Don't want to mix non-UWP libraries with vendored UWP libraries.
            if "gstreamer" in env['LIB']:
                print("Found existing GStreamer library path in LIB. Please remove it.")
                sys.exit(1)

            # Override any existing GStreamer installation with the vendored libraries.
            env["GSTREAMER_1_0_ROOT_" + arch['gst']] = path.join(
                self.msvc_package_dir("gstreamer-uwp"), arch['gst_root']
            )
            env["PKG_CONFIG_PATH"] = path.join(
                self.msvc_package_dir("gstreamer-uwp"), arch['gst_root'],
                "lib", "pkgconfig"
            )

        if 'windows' in host:
            process = subprocess.Popen('("%s" %s > nul) && "python" -c "import os; print(repr(os.environ))"' %
                                       (os.path.join(vs_dirs['vcdir'], "Auxiliary", "Build", "vcvarsall.bat"), "x64"),
                                       stdout=subprocess.PIPE, shell=True)
            stdout, stderr = process.communicate()
            exitcode = process.wait()
            encoding = locale.getpreferredencoding()  # See https://stackoverflow.com/a/9228117
            if exitcode == 0:
                decoded = stdout.decode(encoding)
                if decoded.startswith("environ("):
                    decoded = decoded.strip()[8:-1]
                os.environ.update(eval(decoded))
            else:
                print("Failed to run vcvarsall. stderr:")
                print(stderr.decode(encoding))
                exit(1)

        # Ensure that GStreamer libraries are accessible when linking.
        if 'windows' in target_triple:
            gst_root = gstreamer_root(target_triple, env)
            if gst_root:
                append_to_path_env(os.path.join(gst_root, "lib"), env, "LIB")

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
            env["OPENSSL_VERSION"] = "1.1.1d"
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

            env['PATH'] = (path.join(llvm_toolchain, "bin") + ':' + env['PATH'])
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
            gst_lib_zip = "gstreamer-{}-1.16.0-20190517-095630.zip".format(self.config["android"]["lib"])
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
                urllib.request.urlretrieve(gst_url, gst_lib_zip)
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
            env.setdefault("OPENSSL_VERSION", "1.1.1d")
            env.setdefault("OPENSSL_STATIC", "1")

            # GStreamer configuration
            env.setdefault("GSTREAMER_DIR", path.join(target_path, target, "native", "gstreamer-1.16.0"))
            env.setdefault("GSTREAMER_URL", "https://servo-deps.s3.amazonaws.com/gstreamer/gstreamer-magicleap-1.16.0-20190823-104505.tgz")
            env.setdefault("PKG_CONFIG_PATH", path.join(env["GSTREAMER_DIR"], "system", "lib64", "pkgconfig"))

            # Override the linker set in .cargo/config
            env.setdefault("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", path.join(ml_support, "fake-ld.sh"))

            # Only build libmlservo
            opts += ["--package", "libmlservo"]

            # Download and build OpenSSL if necessary
            status = call(path.join(ml_support, "openssl.sh"), env=env, verbose=verbose)
            if status:
                return status

            # Download prebuilt Gstreamer if necessary
            if not os.path.exists(path.join(env["GSTREAMER_DIR"], "system")):
                if not os.path.exists(env["GSTREAMER_DIR"] + ".tgz"):
                    check_call([
                        'curl',
                        '-L',
                        '-f',
                        '-o', env["GSTREAMER_DIR"] + ".tgz",
                        env["GSTREAMER_URL"],
                    ])
                check_call([
                    'mkdir',
                    '-p',
                    env["GSTREAMER_DIR"],
                ])
                check_call([
                    'tar',
                    'xzf',
                    env["GSTREAMER_DIR"] + ".tgz",
                    '-C', env["GSTREAMER_DIR"],
                ])

        # https://internals.rust-lang.org/t/exploring-crate-graph-build-times-with-cargo-build-ztimings/10975
        # Prepend so that e.g. `-Ztimings` (which means `-Ztimings=info,html`)
        # given on the command line can override it
        opts = ["-Ztimings=info"] + opts

        if with_asan:
            check_call(["rustup", "component", "add", "rust-src"])
            env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -Zsanitizer=address"
            target = target or host
            opts += ["-Zbuild-std"]
            #env.setdefault("CFLAGS", "")
            #env.setdefault("CXXFLAGS", "")
            #env["CFLAGS"] += " -fsanitize=address"
            #env["CXXFLAGS"] += " -fsanitize=address"

        if very_verbose:
            print(["Calling", "cargo", "build"] + opts)
            for key in env:
                print((key, env[key]))

        if sys.platform == "win32":
            env.setdefault("CC", "clang-cl.exe")
            env.setdefault("CXX", "clang-cl.exe")
            if uwp:
                env.setdefault("TARGET_CFLAGS", "")
                env.setdefault("TARGET_CXXFLAGS", "")
                env["TARGET_CFLAGS"] += " -DWINAPI_FAMILY=WINAPI_FAMILY_APP"
                env["TARGET_CXXFLAGS"] += " -DWINAPI_FAMILY=WINAPI_FAMILY_APP"
        else:
            env.setdefault("CC", "clang")
            env.setdefault("CXX", "clang++")

        status = self.run_cargo_build_like_command(
            "build", opts, env=env, verbose=verbose,
            target=target, android=android, magicleap=magicleap, libsimpleservo=libsimpleservo, uwp=uwp,
            features=features, **kwargs
        )

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
                servo_exe_dir = os.path.dirname(
                    self.get_binary_path(release, dev, target=target, simpleservo=libsimpleservo)
                )
                assert os.path.exists(servo_exe_dir)

                # on msvc builds, use editbin to change the subsystem to windows, but only
                # on release builds -- on debug builds, it hides log output
                if not dev and not libsimpleservo:
                    call(["editbin", "/nologo", "/subsystem:windows", path.join(servo_exe_dir, "servo.exe")],
                         verbose=verbose)
                # on msvc, we need to copy in some DLLs in to the servo.exe dir and the directory for unit tests.
                for ssl_lib in ["libssl.dll", "libcrypto.dll"]:
                    ssl_path = path.join(env['OPENSSL_LIB_DIR'], "../bin", ssl_lib)
                    shutil.copy(ssl_path, servo_exe_dir)
                    shutil.copy(ssl_path, path.join(servo_exe_dir, "deps"))

                build_path = path.join(servo_exe_dir, "build")
                assert os.path.exists(build_path)

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
                            return True
                    for lib in libs:
                        print("WARNING: could not find " + lib)

                # UWP build has its own ANGLE library that it packages.
                if not uwp:
                    print("Packaging EGL DLLs")
                    egl_libs = ["libEGL.dll", "libGLESv2.dll"]
                    if not package_generated_shared_libraries(egl_libs, build_path, servo_exe_dir):
                        status = 1

                # copy needed gstreamer DLLs in to servo.exe dir
                print("Packaging gstreamer DLLs")
                if not package_gstreamer_dlls(env, servo_exe_dir, target_triple, uwp):
                    status = 1

                # UWP app packaging already bundles all required DLLs for us.
                print("Packaging MSVC DLLs")
                if not package_msvc_dlls(servo_exe_dir, target_triple, vs_dirs['vcdir'], vs_dirs['vs_version']):
                    status = 1

            elif sys.platform == "darwin":
                servo_exe_dir = os.path.dirname(
                    self.get_binary_path(release, dev, target=target, simpleservo=libsimpleservo)
                )
                assert os.path.exists(servo_exe_dir)

                if not package_gstreamer_dylibs(servo_exe_dir):
                    return 1

                # On the Mac, set a lovely icon. This makes it easier to pick out the Servo binary in tools
                # like Instruments.app.
                try:
                    import Cocoa
                    icon_path = path.join(self.get_top_dir(), "resources", "servo_1024.png")
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

        virtualenv_fname = '_virtualenv%d.%d' % (sys.version_info[0], sys.version_info[1])
        virtualenv_path = path.join(self.get_top_dir(), 'python', virtualenv_fname)
        if path.exists(virtualenv_path):
            print('Removing virtualenv directory: %s' % virtualenv_path)
            shutil.rmtree(virtualenv_path)

        self.clean_uwp()

        opts = ["--manifest-path", manifest_path or path.join(self.context.topdir, "Cargo.toml")]
        if verbose:
            opts += ["-v"]
        opts += params
        return check_call(["cargo", "clean"] + opts, env=self.build_env(), verbose=verbose)

    @Command('clean-uwp',
             description='Clean the support/hololens/ directory.',
             category='build')
    def clean_uwp(self):
        uwp_artifacts = [
            "support/hololens/x64/",
            "support/hololens/ARM/",
            "support/hololens/ARM64/",
            "support/hololens/ServoApp/x64/",
            "support/hololens/ServoApp/ARM/",
            "support/hololens/ServoApp/ARM64/",
            "support/hololens/ServoApp/Generated Files/",
            "support/hololens/ServoApp/BundleArtifacts/",
            "support/hololens/ServoApp/support/",
            "support/hololens/ServoApp/Debug/",
            "support/hololens/ServoApp/Release/",
            "support/hololens/packages/",
            "support/hololens/AppPackages/",
            "support/hololens/ServoApp/ServoApp.vcxproj.user",
        ]

        for uwp_artifact in uwp_artifacts:
            artifact = path.join(self.get_top_dir(), uwp_artifact)
            if path.exists(artifact):
                if path.isdir(artifact):
                    shutil.rmtree(artifact)
                else:
                    os.remove(artifact)


def angle_root(target, nuget_env):
    arch = {
        "aarch64": "arm64",
        "x86_64": "x64",
    }
    angle_arch = arch[target.split('-')[0]]

    package_name = "ANGLE.WindowsStore.Servo"

    import xml.etree.ElementTree as ET
    tree = ET.parse(os.path.join('support', 'hololens', 'ServoApp', 'packages.config'))
    root = tree.getroot()
    for package in root.iter('package'):
        if package.get('id') == package_name:
            package_version = package.get('version')
            break
    else:
        raise Exception("Couldn't locate ANGLE package")

    angle_default_path = path.join(os.getcwd(), "support", "hololens", "packages",
                                   package_name + "." + package_version, "bin", "UAP", angle_arch)

    # Nuget executable command
    nuget_app = path.join(os.getcwd(), "support", "hololens", "ServoApp.sln")
    if not os.path.exists(angle_default_path):
        check_call(['nuget.exe', 'restore', nuget_app], env=nuget_env)

    return angle_default_path


def package_gstreamer_dylibs(servo_exe_dir):
    missing = []
    gst_dylibs = macos_dylibs() + macos_plugins()
    for gst_lib in gst_dylibs:
        try:
            dest_path = os.path.join(servo_exe_dir, os.path.basename(gst_lib))
            if os.path.isfile(dest_path):
                os.remove(dest_path)
            shutil.copy(gst_lib, servo_exe_dir)
        except Exception as e:
            print(e)
            missing += [str(gst_lib)]

    for gst_lib in missing:
        print("ERROR: could not find required GStreamer DLL: " + gst_lib)
    return not missing


def package_gstreamer_dlls(env, servo_exe_dir, target, uwp):
    gst_root = gstreamer_root(target, env)
    if not gst_root:
        print("Could not find GStreamer installation directory.")
        return False

    # All the shared libraries required for starting up and loading plugins.
    gst_dlls = [
        "avcodec-58.dll",
        "avfilter-7.dll",
        "avformat-58.dll",
        "avutil-56.dll",
        "bz2.dll",
        "ffi-7.dll",
        "gio-2.0-0.dll",
        "glib-2.0-0.dll",
        "gmodule-2.0-0.dll",
        "gobject-2.0-0.dll",
        "intl-8.dll",
        "orc-0.4-0.dll",
        "swresample-3.dll",
        "z-1.dll",
    ]

    gst_dlls += windows_dlls(uwp)

    if uwp:
        # These come from a more recent version of ffmpeg and
        # aren't present in the official GStreamer 1.16 release.
        gst_dlls += [
            "avresample-4.dll",
            "postproc-55.dll",
            "swscale-5.dll",
            "x264-157.dll",
        ]
    else:
        # These are built with MinGW and are not yet compatible
        # with UWP's restrictions.
        gst_dlls += [
            "graphene-1.0-0.dll",
            "libcrypto-1_1-x64.dll",
            "libgmp-10.dll",
            "libgnutls-30.dll",
            "libhogweed-4.dll",
            "libjpeg-8.dll",
            "libnettle-6.dll.",
            "libogg-0.dll",
            "libopus-0.dll",
            "libpng16-16.dll",
            "libssl-1_1-x64.dll",
            "libtasn1-6.dll",
            "libtheora-0.dll",
            "libtheoradec-1.dll",
            "libtheoraenc-1.dll",
            "libusrsctp-1.dll",
            "libvorbis-0.dll",
            "libvorbisenc-2.dll",
            "libwinpthread-1.dll",
            "nice-10.dll",
        ]

    missing = []
    for gst_lib in gst_dlls:
        try:
            shutil.copy(path.join(gst_root, "bin", gst_lib), servo_exe_dir)
        except Exception:
            missing += [str(gst_lib)]

    for gst_lib in missing:
        print("ERROR: could not find required GStreamer DLL: " + gst_lib)
    if missing:
        return False

    # Only copy a subset of the available plugins.
    gst_dlls = windows_plugins(uwp)

    gst_plugin_path_root = os.environ.get("GSTREAMER_PACKAGE_PLUGIN_PATH") or gst_root
    gst_plugin_path = path.join(gst_plugin_path_root, "lib", "gstreamer-1.0")
    if not os.path.exists(gst_plugin_path):
        print("ERROR: couldn't find gstreamer plugins at " + gst_plugin_path)
        return False

    missing = []
    for gst_lib in gst_dlls:
        try:
            shutil.copy(path.join(gst_plugin_path, gst_lib), servo_exe_dir)
        except Exception:
            missing += [str(gst_lib)]

    for gst_lib in missing:
        print("ERROR: could not find required GStreamer DLL: " + gst_lib)
    return not missing


def package_msvc_dlls(servo_exe_dir, target, vcinstalldir, vs_version):
    # copy some MSVC DLLs to servo.exe dir
    msvc_redist_dir = None
    vs_platforms = {
        "x86_64": "x64",
        "i686": "x86",
        "aarch64": "arm64",
    }
    target_arch = target.split('-')[0]
    vs_platform = vs_platforms[target_arch]
    vc_dir = vcinstalldir or os.environ.get("VCINSTALLDIR", "")
    if not vs_version:
        vs_version = os.environ.get("VisualStudioVersion", "")
    msvc_deps = [
        "msvcp140.dll",
        "vcruntime140.dll",
    ]
    if target_arch != "aarch64" and "uwp" not in target and vs_version in ("14.0", "15.0", "16.0"):
        msvc_deps += ["api-ms-win-crt-runtime-l1-1-0.dll"]

    # Check if it's Visual C++ Build Tools or Visual Studio 2015
    vs14_vcvars = path.join(vc_dir, "vcvarsall.bat")
    is_vs14 = True if os.path.isfile(vs14_vcvars) or vs_version == "14.0" else False
    if is_vs14:
        msvc_redist_dir = path.join(vc_dir, "redist", vs_platform, "Microsoft.VC140.CRT")
    elif vs_version in ("15.0", "16.0"):
        redist_dir = path.join(vc_dir, "Redist", "MSVC")
        if os.path.isdir(redist_dir):
            for p in os.listdir(redist_dir)[::-1]:
                redist_path = path.join(redist_dir, p)
                for v in ["VC141", "VC142", "VC150", "VC160"]:
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
    if not msvc_redist_dir:
        print("Couldn't locate MSVC redistributable directory")
        return False
    redist_dirs = [
        msvc_redist_dir,
    ]
    if "WindowsSdkDir" in os.environ:
        redist_dirs += [path.join(os.environ["WindowsSdkDir"], "Redist", "ucrt", "DLLs", vs_platform)]
    missing = []
    for msvc_dll in msvc_deps:
        for dll_dir in redist_dirs:
            dll = path.join(dll_dir, msvc_dll)
            servo_dir_dll = path.join(servo_exe_dir, msvc_dll)
            if os.path.isfile(dll):
                if os.path.isfile(servo_dir_dll):
                    # avoid permission denied error when overwrite dll in servo build directory
                    os.chmod(servo_dir_dll, stat.S_IWUSR)
                shutil.copy(dll, servo_exe_dir)
                break
        else:
            missing += [msvc_dll]

    for msvc_dll in missing:
        print("DLL file `{}` not found!".format(msvc_dll))
    return not missing

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
import sys
import shutil

from time import time

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

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
    def build(self, target=None, release=False, dev=False, jobs=None,
              features=None, android=None, verbose=False, very_verbose=False,
              debug_mozjs=False, params=None, with_debug_assertions=False):

        opts = params or []
        opts += ["--manifest-path", self.servo_manifest()]

        if android is None:
            android = self.config["build"]["android"]
        features = features or self.servo_features()

        base_path = self.get_target_dir()
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

        if target and android:
            print("Please specify either --target or --android.")
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

        if android:
            target = self.config["android"]["target"]

        if target:
            if self.config["tools"]["use-rustup"]:
                # 'rustup target add' fails if the toolchain is not installed at all.
                self.call_rustup_run(["rustc", "--version"])

                check_call(["rustup" + BIN_SUFFIX, "target", "add",
                            "--toolchain", self.toolchain(), target])

            opts += ["--target", target]
            if not android:
                android = self.handle_android_target(target)

        self.ensure_bootstrapped(target=target)
        self.ensure_clobbered()

        if debug_mozjs:
            features += ["debugmozjs"]

        if features:
            opts += ["--features", "%s" % ' '.join(features)]

        build_start = time()
        env = self.build_env(target=target, is_build=True)

        if with_debug_assertions:
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C debug_assertions"

        if android:
            if "ANDROID_NDK" not in os.environ:
                print("Please set the ANDROID_NDK environment variable.")
                sys.exit(1)
            if "ANDROID_SDK" not in os.environ:
                print("Please set the ANDROID_SDK environment variable.")
                sys.exit(1)

            android_platform = self.config["android"]["platform"]
            android_toolchain = self.config["android"]["toolchain_name"]
            android_arch = "arch-" + self.config["android"]["arch"]

            # Build OpenSSL for android
            env["OPENSSL_VERSION"] = "1.0.2k"
            make_cmd = ["make"]
            if jobs is not None:
                make_cmd += ["-j" + jobs]
            android_dir = self.android_build_dir(dev)
            openssl_dir = path.join(android_dir, "native", "openssl")
            if not path.exists(openssl_dir):
                os.makedirs(openssl_dir)
            shutil.copy(path.join(self.android_support_dir(), "openssl.makefile"), openssl_dir)
            shutil.copy(path.join(self.android_support_dir(), "openssl.sh"), openssl_dir)

            # Check if the NDK version is 12
            if not os.path.isfile(path.join(env["ANDROID_NDK"], 'source.properties')):
                print("ANDROID_NDK should have file `source.properties`.")
                print("The environment variable ANDROID_NDK may be set at a wrong path.")
                sys.exit(1)
            with open(path.join(env["ANDROID_NDK"], 'source.properties')) as ndk_properties:
                lines = ndk_properties.readlines()
                if lines[1].split(' = ')[1].split('.')[0] != '12':
                    print("Currently only support NDK 12.")
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

            env['PATH'] = path.join(
                env['ANDROID_NDK'], "toolchains", android_toolchain, "prebuilt", host, "bin"
            ) + ':' + env['PATH']
            env['ANDROID_SYSROOT'] = path.join(env['ANDROID_NDK'], "platforms", android_platform, android_arch)
            support_include = path.join(env['ANDROID_NDK'], "sources", "android", "support", "include")
            cxx_include = path.join(
                env['ANDROID_NDK'], "sources", "cxx-stl", "llvm-libc++", "libcxx", "include")
            cxxabi_include = path.join(
                env['ANDROID_NDK'], "sources", "cxx-stl", "llvm-libc++abi", "libcxxabi", "include")
            env['CFLAGS'] = ' '.join([
                "--sysroot", env['ANDROID_SYSROOT'],
                "-I" + support_include])
            env['CXXFLAGS'] = ' '.join([
                "--sysroot", env['ANDROID_SYSROOT'],
                "-I" + support_include,
                "-I" + cxx_include,
                "-I" + cxxabi_include])
            env["NDK_ANDROID_VERSION"] = android_platform.replace("android-", "")
            env['CPPFLAGS'] = ' '.join(["--sysroot", env['ANDROID_SYSROOT']])
            env["CMAKE_ANDROID_ARCH_ABI"] = self.config["android"]["lib"]
            env["CMAKE_TOOLCHAIN_FILE"] = path.join(self.android_support_dir(), "toolchain.cmake")
            # Set output dir for gradle aar files
            aar_out_dir = self.android_aar_dir()
            if not os.path.exists(aar_out_dir):
                os.makedirs(aar_out_dir)
            env["AAR_OUT_DIR"] = aar_out_dir

        status = self.call_rustup_run(["cargo", "build"] + opts, env=env, verbose=verbose)
        elapsed = time() - build_start

        # Do some additional things if the build succeeded
        if status == 0:
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

        opts = []
        if manifest_path:
            opts += ["--manifest-path", manifest_path]
        if verbose:
            opts += ["-v"]
        opts += params
        return check_call(["cargo", "clean"] + opts,
                          env=self.build_env(), cwd=self.servo_crate(), verbose=verbose)

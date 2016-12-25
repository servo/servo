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

from servo.command_base import CommandBase, cd, call, BIN_SUFFIX, host_triple, find_dep_path_newest


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


def notify_build_done(elapsed, success=True):
    """Generate desktop notification when build is complete and the
    elapsed build time was longer than 30 seconds."""
    if elapsed > 30:
        notify("Servo build", "%s in %s" % ("Completed" if success else "FAILED", format_duration(elapsed)))


def notify(title, text):
    """Generate a desktop notification using appropriate means on
    supported platforms Linux, Windows, and Mac OS.  On unsupported
    platforms, this function acts as a no-op."""
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
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    @CommandArgument('--with-debug-assertions',
                     default=None,
                     action='store_true',
                     help='Enable debug assertions in release')
    def build(self, target=None, release=False, dev=False, jobs=None,
              features=None, android=None, verbose=False, debug_mozjs=False, params=None,
              with_debug_assertions=False):
        if android is None:
            android = self.config["build"]["android"]
        features = features or self.servo_features()

        opts = params or []

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
        if android:
            target = self.config["android"]["target"]

        if target:
            opts += ["--target", target]

        self.ensure_bootstrapped(target=target)

        if debug_mozjs:
            features += ["debugmozjs"]

        if features:
            opts += ["--features", "%s" % ' '.join(features)]

        build_start = time()
        env = self.build_env(target=target, is_build=True)

        if with_debug_assertions:
            env["RUSTFLAGS"] = "-C debug_assertions"

        if android:
            # Build OpenSSL for android
            make_cmd = ["make"]
            if jobs is not None:
                make_cmd += ["-j" + jobs]
            android_dir = self.android_build_dir(dev)
            openssl_dir = path.join(android_dir, "native", "openssl")
            if not path.exists(openssl_dir):
                os.makedirs(openssl_dir)
            shutil.copy(path.join(self.android_support_dir(), "openssl.makefile"), openssl_dir)
            shutil.copy(path.join(self.android_support_dir(), "openssl.sh"), openssl_dir)
            env["ANDROID_NDK_ROOT"] = env["ANDROID_NDK"]
            with cd(openssl_dir):
                status = call(
                    make_cmd + ["-f", "openssl.makefile"],
                    env=env,
                    verbose=verbose)
                if status:
                    return status
            openssl_dir = path.join(openssl_dir, "openssl-1.0.1t")
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
                env['ANDROID_NDK'], "toolchains", "arm-linux-androideabi-4.9", "prebuilt", host, "bin"
            ) + ':' + env['PATH']
            env['ANDROID_SYSROOT'] = path.join(env['ANDROID_NDK'], "platforms", "android-18", "arch-arm")
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

        cargo_binary = "cargo" + BIN_SUFFIX

        if sys.platform in ("win32", "msys"):
            if "msvc" not in host_triple():
                env[b'RUSTFLAGS'] = b'-C link-args=-Wl,--subsystem,windows'

        status = call(
            [cargo_binary, "build"] + opts,
            env=env, cwd=self.servo_crate(), verbose=verbose)
        elapsed = time() - build_start

        # Do some additional things if the build succeeded
        if status == 0:
            if sys.platform in ("win32", "msys"):
                servo_exe_dir = path.join(base_path, "debug" if dev else "release")
                # On windows, copy in our manifest
                shutil.copy(path.join(self.get_top_dir(), "components", "servo", "servo.exe.manifest"),
                            servo_exe_dir)
                if "msvc" in (target or host_triple()):
                    msvc_x64 = "64" if "x86_64" in (target or host_triple()) else ""
                    # on msvc builds, use editbin to change the subsystem to windows, but only
                    # on release builds -- on debug builds, it hides log output
                    if not dev:
                        call(["editbin", "/nologo", "/subsystem:windows", path.join(servo_exe_dir, "servo.exe")],
                             verbose=verbose)
                    # on msvc, we need to copy in some DLLs in to the servo.exe dir
                    for ssl_lib in ["ssleay32md.dll", "libeay32md.dll"]:
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
        notify_build_done(elapsed, status == 0)

        print("Build %s in %s" % ("Completed" if status == 0 else "FAILED", format_duration(elapsed)))
        return status

    @Command('build-cef',
             description='Build the Chromium Embedding Framework library',
             category='build')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    @CommandArgument('--release', '-r',
                     action='store_true',
                     help='Build in release mode')
    @CommandArgument('--with-debug-assertions',
                     default=None,
                     action='store_true',
                     help='Enable debug assertions in release')
    def build_cef(self, jobs=None, verbose=False, release=False,
                  with_debug_assertions=False):
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        servo_features = self.servo_features()
        if servo_features:
            opts += ["--features", "%s" % ' '.join(servo_features)]

        build_start = time()
        env = self.build_env(is_build=True)

        if with_debug_assertions:
            env["RUSTFLAGS"] = "-C debug_assertions"

        with cd(path.join("ports", "cef")):
            ret = call(["cargo", "build"] + opts,
                       env=env,
                       verbose=verbose)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(elapsed)

        print("CEF build completed in %s" % format_duration(elapsed))

        return ret

    @Command('build-geckolib',
             description='Build a static library of components used by Gecko',
             category='build')
    @CommandArgument('--with-gecko',
                     default=None,
                     help='Build with Gecko dist directory')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    @CommandArgument('--verbose', '-v',
                     action='store_true',
                     help='Print verbose output')
    @CommandArgument('--release', '-r',
                     action='store_true',
                     help='Build in release mode')
    def build_geckolib(self, with_gecko=None, jobs=None, verbose=False, release=False):
        self.set_use_stable_rust()
        self.ensure_bootstrapped()

        env = self.build_env(is_build=True)
        geckolib_build_path = path.join(self.context.topdir, "target", "geckolib").encode("UTF-8")
        env["CARGO_TARGET_DIR"] = geckolib_build_path

        ret = None
        opts = []
        if with_gecko is not None:
            opts += ["--features", "bindgen"]
            env["MOZ_DIST"] = path.abspath(with_gecko)
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        if with_gecko is not None:
            print("Generating atoms data...")
            run_file = path.join(self.context.topdir, "components",
                                 "style", "binding_tools", "regen_atoms.py")
            run_globals = {"__file__": run_file}
            execfile(run_file, run_globals)
            run_globals["generate_atoms"](env["MOZ_DIST"])

        build_start = time()
        with cd(path.join("ports", "geckolib")):
            ret = call(["cargo", "build"] + opts, env=env, verbose=verbose)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(elapsed)

        print("GeckoLib build completed in %s" % format_duration(elapsed))

        if with_gecko is not None:
            print("Copying binding files to style/gecko_bindings...")
            build_path = path.join(geckolib_build_path, "release" if release else "debug", "")
            target_style_path = find_dep_path_newest("style", build_path)
            out_gecko_path = path.join(target_style_path, "out", "gecko")
            bindings_path = path.join(self.context.topdir, "components", "style", "gecko_bindings")
            for f in ["bindings.rs", "structs_debug.rs", "structs_release.rs"]:
                shutil.copy(path.join(out_gecko_path, f), bindings_path)

        return ret

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
    def clean(self, manifest_path, params, verbose=False):
        self.ensure_bootstrapped()

        opts = []
        if manifest_path:
            opts += ["--manifest-path", manifest_path]
        if verbose:
            opts += ["-v"]
        opts += params
        return call(["cargo", "clean"] + opts,
                    env=self.build_env(), cwd=self.servo_crate(), verbose=verbose)

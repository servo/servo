# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import os
import os.path as path
import sys
import shutil

from time import time

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, call


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


def notify_build_done(elapsed):
    """Generate desktop notification when build is complete and the
    elapsed build time was longer than 30 seconds."""
    if elapsed > 30:
        notify("Servo build", "Completed in %0.2fs" % elapsed)


def notify(title, text):
    """Generate a desktop notification using appropriate means on
    supported platforms Linux, Windows, and Mac OS.  On unsupported
    platforms, this function acts as a no-op."""
    platforms = {
        "linux": notify_linux,
        "linux2": notify_linux,
        "win": notify_win,
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
    def build(self, target=None, release=False, dev=False, jobs=None,
              features=None, android=None, verbose=False, debug_mozjs=False, params=None):
        if android is None:
            android = self.config["build"]["android"]
        features = features or []

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

        targets = []
        if release:
            opts += ["--release"]
        if target:
            opts += ["--target", target]
            targets.append(target)
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if android:
            opts += ["--target", self.config["android"]["target"]]
            targets.append("arm-linux-androideabi")

        self.ensure_bootstrapped(targets=targets)

        if debug_mozjs or self.config["build"]["debug-mozjs"]:
            features += ["script/debugmozjs"]

        if android:
            features += ["android_glue"]

        if features:
            opts += ["--features", "%s" % ' '.join(features)]

        build_start = time()
        env = self.build_env()
        if android:
            # Ensure Rust uses hard floats on Android
            env['RUSTFLAGS'] = env.get('RUSTFLAGS', "") + " -C target-feature=+neon"
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
            with cd(openssl_dir):
                status = call(
                    make_cmd + ["-f", "openssl.makefile"],
                    env=self.build_env(),
                    verbose=verbose)
                if status:
                    return status
            openssl_dir = path.join(openssl_dir, "openssl-1.0.1k")
            env['OPENSSL_LIB_DIR'] = openssl_dir
            env['OPENSSL_INCLUDE_DIR'] = path.join(openssl_dir, "include")
            env['OPENSSL_STATIC'] = 'TRUE'

        if not (self.config["build"]["ccache"] == ""):
            env['CCACHE'] = self.config["build"]["ccache"]

        status = call(
            ["cargo", "build"] + opts,
            env=env, cwd=self.servo_crate(), verbose=verbose)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(elapsed)

        print("Build completed in %0.2fs" % elapsed)
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
    def build_cef(self, jobs=None, verbose=False, release=False):
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        build_start = time()
        with cd(path.join("ports", "cef")):
            ret = call(["cargo", "build"] + opts,
                       env=self.build_env(), verbose=verbose)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(elapsed)

        print("CEF build completed in %0.2fs" % elapsed)

        return ret

    @Command('build-geckolib',
             description='Build a static library of components used by Gecko',
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
    def build_geckolib(self, jobs=None, verbose=False, release=False):
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        build_start = time()
        with cd(path.join("ports", "geckolib")):
            ret = call(["cargo", "build"] + opts,
                       env=self.build_env(), verbose=verbose)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(elapsed)

        print("GeckoLib build completed in %0.2fs" % elapsed)

        return ret

    @Command('build-gonk',
             description='Build the Gonk port',
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
    def build_gonk(self, jobs=None, verbose=False, release=False):
        targets = ["arm-linux-androideabi"]
        self.ensure_bootstrapped(targets=targets)

        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        opts += ["--target", self.config["android"]["target"]]
        env = self.build_env(gonk=True)
        build_start = time()
        with cd(path.join("ports", "gonk")):
            ret = call(["cargo", "build"] + opts, env=env, verbose=verbose)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify_build_done(elapsed)

        print("Gonk build completed in %0.2fs" % elapsed)

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

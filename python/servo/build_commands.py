from __future__ import print_function, unicode_literals

import sys
import os
import os.path as path
import subprocess
from time import time

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd

def is_headless_build():
    return int(os.getenv('SERVO_HEADLESS', 0)) == 1

# Function to generate desktop notification once build is completed & limit exceeded!
def notify(elapsed):
    if elapsed < 30:
        return

    if sys.platform.startswith('linux'):
        try:
            import dbus
            bus = dbus.SessionBus()
            notify_obj = bus.get_object('org.freedesktop.Notifications', '/org/freedesktop/Notifications')
            method = notify_obj.get_dbus_method('Notify', 'org.freedesktop.Notifications')
            method('Servo Build System', 0, '', ' Servo build complete!', '', [], [], -1)
        except:
            print("[Warning] Could not generate notification! Please make sure that the python dbus module is installed!")

    elif sys.platform.startswith('win'):
        try:
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
        except:
            print("[Warning] Could not generate notification! Please make sure that the required libraries are installed!")

    elif sys.platform.startswith('darwin'):
        try:
            from distutils.spawn import find_executable
            notifier = find_executable('terminal-notifier')
            if not notifier:
                raise Exception('`terminal-notifier` not found')
            subprocess.call([notifier, '-title', 'Servo Build System',
                             '-group', 'servobuild', '-message', 'Servo build complete!'])
        except:
            print("[Warning] Could not generate notification! Make sure that `terminal-notifier is installed!")


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
              android=None, verbose=False, debug_mozjs=False, params=None):
        self.ensure_bootstrapped()

        if android is None:
            android = self.config["build"]["android"]

        opts = params or []
        features = []

        if release:
            opts += ["--release"]
        if target:
            opts += ["--target", target]
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if android:
            # Ensure the APK builder submodule has been built first
            apk_builder_dir = "support/android-rs-glue"
            with cd(path.join(apk_builder_dir, "apk-builder")):
                subprocess.call(["cargo", "build"], env=self.build_env())

            opts += ["--target", "arm-linux-androideabi"]

        if debug_mozjs or self.config["build"]["debug-mozjs"]:
            features += ["script/debugmozjs"]

        if is_headless_build():
            opts += ["--no-default-features"]
            features += ["headless"]

        if android:
            features += ["android_glue"]

        if features:
            opts += ["--features", "%s" % ' '.join(features)]

        build_start = time()
        env = self.build_env()
        if android:
            # Build OpenSSL for android
            make_cmd = ["make"]
            if jobs is not None:
                make_cmd += ["-j" + jobs]
            with cd(self.android_support_dir()):
                status = subprocess.call(
                    make_cmd + ["-f", "openssl.makefile"],
                    env=self.build_env())
                if status:
                    return status
            openssl_dir = path.join(self.android_support_dir(), "openssl-1.0.1k")
            env['OPENSSL_LIB_DIR'] = openssl_dir
            env['OPENSSL_INCLUDE_DIR'] = path.join(openssl_dir, "include")
            env['OPENSSL_STATIC'] = 'TRUE'

        status = subprocess.call(
            ["cargo", "build"] + opts,
            env=env, cwd=self.servo_crate())
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify(elapsed)

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
            ret = subprocess.call(["cargo", "build"] + opts,
                                  env=self.build_env())
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify(elapsed)

        print("CEF build completed in %0.2fs" % elapsed)

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
        self.ensure_bootstrapped()

        ret = None
        opts = []
        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if release:
            opts += ["--release"]

        opts += ["--target", "arm-linux-androideabi"]
        env=self.build_env(gonk=True)
        build_start = time()
        with cd(path.join("ports", "gonk")):
            ret = subprocess.call(["cargo", "build"] + opts, env=env)
        elapsed = time() - build_start

        # Generate Desktop Notification if elapsed-time > some threshold value
        notify(elapsed)

        print("Gonk build completed in %0.2fs" % elapsed)

        return ret


    @Command('build-tests',
             description='Build the Servo test suites',
             category='build')
    @CommandArgument('--jobs', '-j',
                     default=None,
                     help='Number of jobs to run in parallel')
    def build_tests(self, jobs=None):
        self.ensure_bootstrapped()
        args = ["cargo", "test", "--no-run"]
        if is_headless_build():
            args += ["--no-default-features", "--features", "headless"]
        return subprocess.call(
            args,
            env=self.build_env(), cwd=self.servo_crate())

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
        return subprocess.call(["cargo", "clean"] + opts,
                               env=self.build_env(), cwd=self.servo_crate())

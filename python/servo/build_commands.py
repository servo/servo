# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import datetime
import os
import os.path as path
import pathlib
import shutil
import stat
import subprocess
import sys
import urllib

from time import time
from typing import Dict, Optional
import zipfile

import notifypy

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)
from mach.registrar import Registrar

import servo.platform
import servo.util
import servo.visual_studio

from servo.command_base import BuildType, CommandBase, call, check_call
from servo.gstreamer import windows_dlls, windows_plugins, macos_plugins

SUPPORTED_ASAN_TARGETS = ["aarch64-apple-darwin", "aarch64-unknown-linux-gnu",
                          "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]


@CommandProvider
class MachCommands(CommandBase):
    @Command('build', description='Build Servo', category='build')
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
    @CommandArgument('params', nargs='...',
                     help="Command-line arguments to be passed through to Cargo")
    @CommandBase.common_command_arguments(build_configuration=True, build_type=True)
    def build(self, build_type: BuildType, jobs=None, params=None, no_package=False,
              verbose=False, very_verbose=False, with_asan=False, **kwargs):
        opts = params or []

        if build_type.is_release():
            opts += ["--release"]
        elif build_type.is_dev():
            pass  # there is no argument for debug
        else:
            opts += ["--profile", build_type.profile]

        if jobs is not None:
            opts += ["-j", jobs]
        if verbose:
            opts += ["-v"]
        if very_verbose:
            opts += ["-vv"]

        env = self.build_env()
        self.ensure_bootstrapped()
        self.ensure_clobbered()

        host = servo.platform.host_triple()
        target_triple = self.cross_compile_target or servo.platform.host_triple()

        if with_asan:
            if target_triple not in SUPPORTED_ASAN_TARGETS:
                print("AddressSanitizer is currently not supported on this platform\n",
                      "See https://doc.rust-lang.org/beta/unstable-book/compiler-flags/sanitizer.html")
                sys.exit(1)

            # do not use crown (clashes with different rust version)
            env["RUSTC"] = "rustc"

            # Enable usage of unstable rust flags
            env["RUSTC_BOOTSTRAP"] = "1"

            # Enable asan
            env["RUSTFLAGS"] = env.get("RUSTFLAGS", "") + " -Zsanitizer=address"
            opts += ["-Zbuild-std"]
            kwargs["target_override"] = target_triple
            # TODO: Investigate sanitizers in C/C++ code:
            # env.setdefault("CFLAGS", "")
            # env.setdefault("CXXFLAGS", "")
            # env["CFLAGS"] += " -fsanitize=address"
            # env["CXXFLAGS"] += " -fsanitize=address"

            # asan replaces system allocator with asan allocator
            # we need to make sure that we do not replace it with jemalloc
            self.features.append("servo_allocator/use-system-allocator")

        build_start = time()

        if host != target_triple and 'windows' in target_triple:
            if os.environ.get('VisualStudioVersion') or os.environ.get('VCINSTALLDIR'):
                print("Can't cross-compile for Windows inside of a Visual Studio shell.\n"
                      "Please run `python mach build [arguments]` to bypass automatic "
                      "Visual Studio shell, and make sure the VisualStudioVersion and "
                      "VCINSTALLDIR environment variables are not set.")
                sys.exit(1)

        # Gather Cargo build timings (https://doc.rust-lang.org/cargo/reference/timings.html).
        opts = ["--timings"] + opts

        if very_verbose:
            print(["Calling", "cargo", "build"] + opts)
            for key in env:
                print((key, env[key]))

        status = self.run_cargo_build_like_command(
            "rustc", opts, env=env, verbose=verbose, **kwargs)

        if status == 0:
            built_binary = self.get_binary_path(
                build_type,
                target=self.cross_compile_target,
                android=self.is_android_build,
                asan=with_asan
            )

            if self.is_android_build and not no_package:
                rv = Registrar.dispatch("package", context=self.context, build_type=build_type,
                                        target=self.cross_compile_target, flavor=None)
                if rv:
                    return rv

            if sys.platform == "win32":
                if not copy_windows_dlls_to_build_directory(built_binary, target_triple):
                    status = 1

            elif sys.platform == "darwin":
                servo_bin_dir = os.path.dirname(built_binary)
                assert os.path.exists(servo_bin_dir)

                if self.enable_media:
                    print("Packaging gstreamer dylibs")
                    if not package_gstreamer_dylibs(self.cross_compile_target, built_binary):
                        return 1

                # On the Mac, set a lovely icon. This makes it easier to pick out the Servo binary in tools
                # like Instruments.app.
                try:
                    import Cocoa
                    icon_path = path.join(self.get_top_dir(), "resources", "servo_1024.png")
                    icon = Cocoa.NSImage.alloc().initWithContentsOfFile_(icon_path)
                    if icon is not None:
                        Cocoa.NSWorkspace.sharedWorkspace().setIcon_forFile_options_(icon,
                                                                                     built_binary,
                                                                                     0)
                except ImportError:
                    pass

        # Generate Desktop Notification if elapsed-time > some threshold value

        elapsed = time() - build_start
        elapsed_delta = datetime.timedelta(seconds=int(elapsed))
        build_message = f"{'Succeeded' if status == 0 else 'Failed'} in {elapsed_delta}"
        self.notify("Servo build", build_message)
        print(build_message)

        return status

    def download_and_build_android_dependencies_if_needed(self, env: Dict[str, str]):
        if not self.is_android_build:
            return

        # Build the name of the package containing all GStreamer dependencies
        # according to the build target.
        android_lib = self.config["android"]["lib"]
        gst_lib = f"gst-build-{android_lib}"
        gst_lib_zip = f"gstreamer-{android_lib}-1.16.0-20190517-095630.zip"
        gst_lib_path = os.path.join(self.target_path, "gstreamer", gst_lib)
        pkg_config_path = os.path.join(gst_lib_path, "pkgconfig")
        env["PKG_CONFIG_PATH"] = pkg_config_path
        if not os.path.exists(gst_lib_path):
            # Download GStreamer dependencies if they have not already been downloaded
            # This bundle is generated with `libgstreamer_android_gen`
            # Follow these instructions to build and deploy new binaries
            # https://github.com/servo/libgstreamer_android_gen#build
            gst_url = f"https://servo-deps-2.s3.amazonaws.com/gstreamer/{gst_lib_zip}"
            print(f"Downloading GStreamer dependencies ({gst_url})")

            urllib.request.urlretrieve(gst_url, gst_lib_zip)
            zip_ref = zipfile.ZipFile(gst_lib_zip, "r")
            zip_ref.extractall(os.path.join(self.target_path, "gstreamer"))
            os.remove(gst_lib_zip)

            # Change pkgconfig info to make all GStreamer dependencies point
            # to the libgstreamer_android.so bundle.
            for each in os.listdir(pkg_config_path):
                if each.endswith('.pc'):
                    print(f"Setting pkgconfig info for {each}")
                    target_path = os.path.join(pkg_config_path, each)
                    expr = f"s#libdir=.*#libdir={gst_lib_path}#g"
                    subprocess.call(["perl", "-i", "-pe", expr, target_path])

    @Command('clean',
             description='Clean the target/ and python/_venv[version]/ directories',
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

        virtualenv_fname = '_venv%d.%d' % (sys.version_info[0], sys.version_info[1])
        virtualenv_path = path.join(self.get_top_dir(), 'python', virtualenv_fname)
        if path.exists(virtualenv_path):
            print('Removing virtualenv directory: %s' % virtualenv_path)
            shutil.rmtree(virtualenv_path)

        opts = ["--manifest-path", manifest_path or path.join(self.context.topdir, "Cargo.toml")]
        if verbose:
            opts += ["-v"]
        opts += params
        return check_call(["cargo", "clean"] + opts, env=self.build_env(), verbose=verbose)

    def notify(self, title: str, message: str):
        """Generate desktop notification when build is complete and the
        elapsed build time was longer than 30 seconds.

        If notify-command is set in the [tools] section of the configuration,
        that is used instead."""
        notify_command = self.config["tools"].get("notify-command")

        # notifypy does not know how to send transient notifications, so we use a custom
        # notifier on Linux. If transient notifications are not used, then notifications
        # pile up in the notification center and must be cleared manually.
        class LinuxNotifier(notifypy.BaseNotifier):
            def __init__(self, **kwargs):
                pass

            def send_notification(self, **kwargs):
                try:
                    import dbus
                    bus = dbus.SessionBus()
                    notify_obj = bus.get_object("org.freedesktop.Notifications", "/org/freedesktop/Notifications")
                    method = notify_obj.get_dbus_method("Notify", "org.freedesktop.Notifications")
                    method(
                        kwargs.get("application_name"),
                        0,  # Don't replace previous notification.
                        kwargs.get("notification_icon", ""),
                        kwargs.get("notification_title"),
                        kwargs.get("notification_subtitle"),
                        [],  # actions
                        {"transient": True},  # hints
                        -1  # timeout
                    )
                except Exception as exception:
                    print(f"[Warning] Could not generate notification: {exception}",
                          file=sys.stderr)
                return True

        if notify_command:
            if call([notify_command, title, message]) != 0:
                print("[Warning] Could not generate notification: "
                      f"Could not run '{notify_command}'.", file=sys.stderr)
        else:
            try:
                notifier = LinuxNotifier if sys.platform.startswith("linux") else None
                notification = notifypy.Notify(use_custom_notifier=notifier)
                notification.title = title
                notification.message = message
                notification.icon = path.join(self.get_top_dir(), "resources", "servo_64.png")
                notification.send(block=False)
            except notifypy.exceptions.UnsupportedPlatform as e:
                print(f"[Warning] Could not generate notification: {e}", file=sys.stderr)


def otool(s):
    o = subprocess.Popen(['/usr/bin/otool', '-L', s], stdout=subprocess.PIPE)
    for line in map(lambda s: s.decode('ascii'), o.stdout):
        if line[0] == '\t':
            yield line.split(' ', 1)[0][1:]


def install_name_tool(binary, *args):
    try:
        subprocess.check_call(['install_name_tool', *args, binary])
    except subprocess.CalledProcessError as e:
        print("install_name_tool exited with return value %d" % e.returncode)


def change_link_name(binary, old, new):
    install_name_tool(binary, '-change', old, f"@executable_path/{new}")


def is_system_library(lib):
    return lib.startswith("/System/Library") or lib.startswith("/usr/lib") or ".asan." in lib


def is_relocatable_library(lib):
    return lib.startswith("@rpath/")


def change_non_system_libraries_path(libraries, relative_path, binary):
    for lib in libraries:
        if is_system_library(lib) or is_relocatable_library(lib):
            continue
        new_path = path.join(relative_path, path.basename(lib))
        change_link_name(binary, lib, new_path)


def resolve_rpath(lib, rpath_root):
    if not is_relocatable_library(lib):
        return lib

    rpaths = ['', '../', 'gstreamer-1.0/']
    for rpath in rpaths:
        full_path = rpath_root + lib.replace('@rpath/', rpath)
        if path.exists(full_path):
            return path.normpath(full_path)

    raise Exception("Unable to satisfy rpath dependency: " + lib)


def copy_dependencies(binary_path, lib_path, gst_lib_dir):
    relative_path = path.relpath(lib_path, path.dirname(binary_path)) + "/"

    # Update binary libraries
    binary_dependencies = set(otool(binary_path))
    change_non_system_libraries_path(binary_dependencies, relative_path, binary_path)

    plugins = [os.path.join(gst_lib_dir, "gstreamer-1.0", plugin) for plugin in macos_plugins()]
    binary_dependencies = binary_dependencies.union(plugins)

    # Update dependencies libraries
    need_checked = binary_dependencies
    checked = set()
    while need_checked:
        checking = set(need_checked)
        need_checked = set()
        for f in checking:
            # No need to check these for their dylibs
            if is_system_library(f):
                continue
            full_path = resolve_rpath(f, gst_lib_dir)
            need_relinked = set(otool(full_path))
            new_path = path.join(lib_path, path.basename(full_path))
            if not path.exists(new_path):
                shutil.copyfile(full_path, new_path)
            change_non_system_libraries_path(need_relinked, relative_path, new_path)
            need_checked.update(need_relinked)
        checked.update(checking)
        need_checked.difference_update(checked)


def package_gstreamer_dylibs(cross_compilation_target, servo_bin):
    gst_root = servo.platform.get().gstreamer_root(cross_compilation_target)

    # This might be None if we are cross-compiling.
    if not gst_root:
        return True

    lib_dir = path.join(path.dirname(servo_bin), "lib")
    if os.path.exists(lib_dir):
        shutil.rmtree(lib_dir)
    os.mkdir(lib_dir)
    try:
        copy_dependencies(servo_bin, lib_dir, path.join(gst_root, 'lib', ''))
    except Exception as e:
        print("ERROR: could not package required dylibs")
        print(e)
        return False
    return True


def copy_windows_dlls_to_build_directory(servo_binary: str, target_triple: str) -> bool:
    servo_exe_dir = os.path.dirname(servo_binary)
    assert os.path.exists(servo_exe_dir)

    build_path = path.join(servo_exe_dir, "build")
    assert os.path.exists(build_path)

    # Copy in the built EGL and GLES libraries from where they were built to
    # the final build dirctory
    def find_and_copy_built_dll(dll_name):
        try:
            file_to_copy = next(pathlib.Path(build_path).rglob(dll_name))
            shutil.copy(file_to_copy, servo_exe_dir)
        except StopIteration:
            print(f"WARNING: could not find {dll_name}")

    print(" • Copying ANGLE DLLs to binary directory...")
    find_and_copy_built_dll("libEGL.dll")
    find_and_copy_built_dll("libGLESv2.dll")

    print(" • Copying GStreamer DLLs to binary directory...")
    if not package_gstreamer_dlls(servo_exe_dir, target_triple):
        return False

    print(" • Copying MSVC DLLs to binary directory...")
    if not package_msvc_dlls(servo_exe_dir, target_triple):
        return False

    return True


def package_gstreamer_dlls(servo_exe_dir: str, target: str):
    gst_root = servo.platform.get().gstreamer_root(cross_compilation_target=target)
    if not gst_root:
        print("Could not find GStreamer installation directory.")
        return False

    missing = []
    for gst_lib in windows_dlls():
        try:
            shutil.copy(path.join(gst_root, "bin", gst_lib), servo_exe_dir)
        except Exception:
            missing += [str(gst_lib)]

    for gst_lib in missing:
        print("ERROR: could not find required GStreamer DLL: " + gst_lib)
    if missing:
        return False

    # Only copy a subset of the available plugins.
    gst_dlls = windows_plugins()

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


def package_msvc_dlls(servo_exe_dir: str, target: str):
    def copy_file(dll_path: Optional[str]) -> bool:
        if not dll_path or not os.path.exists(dll_path):
            print(f"WARNING: Could not find DLL at {dll_path}", file=sys.stderr)
            return False
        servo_dir_dll = path.join(servo_exe_dir, os.path.basename(dll_path))
        # Avoid permission denied error when overwriting DLLs.
        if os.path.isfile(servo_dir_dll):
            os.chmod(servo_dir_dll, stat.S_IWUSR)
        print(f"    • Copying {dll_path}")
        shutil.copy(dll_path, servo_exe_dir)
        return True

    vs_platform = {
        "x86_64": "x64",
        "i686": "x86",
        "aarch64": "arm64",
    }[target.split('-')[0]]

    for msvc_redist_dir in servo.visual_studio.find_msvc_redist_dirs(vs_platform):
        if copy_file(os.path.join(msvc_redist_dir, "msvcp140.dll")) and \
           copy_file(os.path.join(msvc_redist_dir, "vcruntime140.dll")):
            break

    # Different SDKs install the file into different directory structures within the
    # Windows SDK installation directory, so use a glob to search for a path like
    # "**\x64\api-ms-win-crt-runtime-l1-1-0.dll".
    windows_sdk_dir = servo.visual_studio.find_windows_sdk_installation_path()
    dll_name = "api-ms-win-crt-runtime-l1-1-0.dll"
    file_to_copy = next(pathlib.Path(windows_sdk_dir).rglob(os.path.join("**", vs_platform, dll_name)))
    copy_file(file_to_copy)

    return True

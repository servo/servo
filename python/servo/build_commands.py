# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import datetime
import locale
import os
import os.path as path
import shutil
import stat
import subprocess
import sys
import urllib

from time import time
from typing import Dict
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

from servo.command_base import BuildType, CommandBase, call, check_call
from servo.gstreamer import windows_dlls, windows_plugins, macos_plugins


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
              verbose=False, very_verbose=False, libsimpleservo=False, **kwargs):
        opts = params or []
        has_media_stack = "media-gstreamer" in self.features

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

        env = self.build_env(is_build=True)
        self.ensure_bootstrapped()
        self.ensure_clobbered()

        build_start = time()

        host = servo.platform.host_triple()
        if 'windows' in host:
            vs_dirs = self.vs_dirs()

        target_triple = self.cross_compile_target or servo.platform.host_triple()
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

        # Gather Cargo build timings (https://doc.rust-lang.org/cargo/reference/timings.html).
        opts = ["--timings"] + opts

        if very_verbose:
            print(["Calling", "cargo", "build"] + opts)
            for key in env:
                print((key, env[key]))

        self.download_and_build_android_dependencies_if_needed(env)
        status = self.run_cargo_build_like_command(
            "build", opts, env=env, verbose=verbose,
            libsimpleservo=libsimpleservo, **kwargs
        )

        # Do some additional things if the build succeeded
        if status == 0:
            if self.is_android_build and not no_package:
                flavor = None
                if "googlevr" in self.features:
                    flavor = "googlevr"
                elif "oculusvr" in self.features:
                    flavor = "oculusvr"
                rv = Registrar.dispatch("package", context=self.context, build_type=build_type,
                                        target=self.cross_compile_target, flavor=flavor)
                if rv:
                    return rv

            if sys.platform == "win32":
                servo_exe_dir = os.path.dirname(
                    self.get_binary_path(build_type, target=self.cross_compile_target, simpleservo=libsimpleservo)
                )
                assert os.path.exists(servo_exe_dir)

                build_path = path.join(servo_exe_dir, "build")
                assert os.path.exists(build_path)

                # on msvc, we need to copy in some DLLs in to the servo.exe dir and the directory for unit tests.
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

                print("Packaging EGL DLLs")
                egl_libs = ["libEGL.dll", "libGLESv2.dll"]
                if not package_generated_shared_libraries(egl_libs, build_path, servo_exe_dir):
                    status = 1

                # copy needed gstreamer DLLs in to servo.exe dir
                if has_media_stack:
                    print("Packaging gstreamer DLLs")
                    if not package_gstreamer_dlls(env, servo_exe_dir, target_triple):
                        status = 1

                # UWP app packaging already bundles all required DLLs for us.
                print("Packaging MSVC DLLs")
                if not package_msvc_dlls(servo_exe_dir, target_triple, vs_dirs['vcdir'], vs_dirs['vs_version']):
                    status = 1

            elif sys.platform == "darwin":
                servo_path = self.get_binary_path(
                    build_type, target=self.cross_compile_target, simpleservo=libsimpleservo)
                servo_bin_dir = os.path.dirname(servo_path)
                assert os.path.exists(servo_bin_dir)

                if has_media_stack:
                    print("Packaging gstreamer dylibs")
                    if not package_gstreamer_dylibs(self.cross_compile_target, servo_path):
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
             description='Clean the target/ and python/_virtualenv[version]/ directories',
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
            notifier = LinuxNotifier if sys.platform.startswith("linux") else None
            notification = notifypy.Notify(use_custom_notifier=notifier)
            notification.title = title
            notification.message = message
            notification.icon = path.join(self.get_top_dir(), "resources", "servo_64.png")
            notification.send(block=False)


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
    return lib.startswith("/System/Library") or lib.startswith("/usr/lib")


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


def copy_dependencies(binary_path, lib_path, gst_root):
    relative_path = path.relpath(lib_path, path.dirname(binary_path)) + "/"

    # Update binary libraries
    binary_dependencies = set(otool(binary_path))
    change_non_system_libraries_path(binary_dependencies, relative_path, binary_path)
    binary_dependencies = binary_dependencies.union(macos_plugins())

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
            full_path = resolve_rpath(f, gst_root)
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


def package_gstreamer_dlls(env, servo_exe_dir, target):
    gst_root = servo.platform.get().gstreamer_root(cross_compilation_target=target)
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
        "graphene-1.0-0.dll",
        "intl-8.dll",
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
        "orc-0.4-0.dll",
        "swresample-3.dll",
        "z-1.dll",
    ] + windows_dlls()

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
    if target_arch != "aarch64" and vs_version in ("14.0", "15.0", "16.0"):
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

# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import sys
import os.path as path
sys.path.append(path.join(path.dirname(sys.argv[0]), "components", "style", "properties", "Mako-0.9.1.zip"))

import json
import os
import shutil
import subprocess
import mako.template

from mach.registrar import Registrar

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from mako.template import Template

from servo.command_base import (
    archive_deterministically,
    BuildNotFound,
    cd,
    CommandBase,
    is_macosx,
    is_windows,
    get_browserhtml_path,
)
from servo.command_base import find_dep_path_newest


def delete(path):
    try:
        os.remove(path)         # Succeeds if path was a file
    except OSError:             # Or, if path was a directory...
        shutil.rmtree(path)     # Remove it and all its contents.


def otool(s):
    o = subprocess.Popen(['/usr/bin/otool', '-L', s], stdout=subprocess.PIPE)
    for l in o.stdout:
        if l[0] == '\t':
            yield l.split(' ', 1)[0][1:]


def listfiles(directory):
    return [f for f in os.listdir(directory)
            if path.isfile(path.join(directory, f))]


def install_name_tool(old, new, binary):
    try:
        subprocess.check_call(['install_name_tool', '-change', old, '@executable_path/' + new, binary])
    except subprocess.CalledProcessError as e:
        print("install_name_tool exited with return value %d" % e.returncode)


def is_system_library(lib):
    return lib.startswith("/System/Library") or lib.startswith("/usr/lib")


def change_non_system_libraries_path(libraries, relative_path, binary):
    for lib in libraries:
        if is_system_library(lib):
            continue
        new_path = path.join(relative_path, path.basename(lib))
        install_name_tool(lib, new_path, binary)


def copy_dependencies(binary_path, lib_path):
    relative_path = path.relpath(lib_path, path.dirname(binary_path)) + "/"

    # Update binary libraries
    binary_dependencies = set(otool(binary_path))
    change_non_system_libraries_path(binary_dependencies, relative_path, binary_path)

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
            need_relinked = set(otool(f))
            new_path = path.join(lib_path, path.basename(f))
            if not path.exists(new_path):
                shutil.copyfile(f, new_path)
            change_non_system_libraries_path(need_relinked, relative_path, new_path)
            need_checked.update(need_relinked)
        checked.update(checking)
        need_checked.difference_update(checked)


def copy_windows_dependencies(binary_path, destination):
    try:
        [shutil.copy(path.join(binary_path, d), destination) for d in ["libcryptoMD.dll", "libsslMD.dll"]]
    except:
        deps = [
            "libstdc++-6.dll",
            "libwinpthread-1.dll",
            "libbz2-1.dll",
            "libgcc_s_seh-1.dll",
            "libexpat-1.dll",
            "zlib1.dll",
            "libiconv-2.dll",
            "libintl-8.dll",
            "libcryptoMD.dll",
            "libsslMD.dll",
        ]
        for d in deps:
            dep_path = path.join("C:\\msys64\\mingw64\\bin", d)
            if path.exists(dep_path):
                shutil.copy(dep_path, path.join(destination, d))


def change_prefs(resources_path, platform):
    print("Swapping prefs")
    prefs_path = path.join(resources_path, "prefs.json")
    package_prefs_path = path.join(resources_path, "package-prefs.json")
    os_type = "os:{}".format(platform)
    with open(prefs_path) as prefs, open(package_prefs_path) as package_prefs:
        prefs = json.load(prefs)
        package_prefs = json.load(package_prefs)
        for pref in package_prefs:
            if os_type in pref:
                prefs[pref.split(";")[1]] = package_prefs[pref]
            if pref in prefs:
                prefs[pref] = package_prefs[pref]
        with open(prefs_path, "w") as out:
            json.dump(prefs, out, sort_keys=True, indent=2)
    delete(package_prefs_path)


@CommandProvider
class PackageCommands(CommandBase):
    @Command('package',
             description='Package Servo',
             category='package')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Package the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Package the dev build')
    @CommandArgument('--android',
                     default=None,
                     action='store_true',
                     help='Package Android')
    def package(self, release=False, dev=False, android=None, debug=False, debugger=None):
        env = self.build_env()
        if android is None:
            android = self.config["build"]["android"]
        binary_path = self.get_binary_path(release, dev, android=android)
        dir_to_root = self.get_top_dir()
        target_dir = path.dirname(binary_path)
        if android:
            if dev:
                env["NDK_DEBUG"] = "1"
                env["ANT_FLAVOR"] = "debug"
                dev_flag = "-d"
            else:
                env["ANT_FLAVOR"] = "release"
                dev_flag = ""

            output_apk = "{}.apk".format(binary_path)

            dir_to_apk = path.join(target_dir, "apk")
            if path.exists(dir_to_apk):
                print("Cleaning up from previous packaging")
                delete(dir_to_apk)
            shutil.copytree(path.join(dir_to_root, "support", "android", "apk"), dir_to_apk)

            blurdroid_path = find_dep_path_newest('blurdroid', binary_path)
            if blurdroid_path is None:
                print("Could not find blurdroid package; perhaps you haven't built Servo.")
                return 1
            else:
                dir_to_libs = path.join(dir_to_apk, "libs")
                if not path.exists(dir_to_libs):
                    os.makedirs(dir_to_libs)
                shutil.copy2(blurdroid_path + '/out/blurdroid.jar', dir_to_libs)
            try:
                with cd(path.join("support", "android", "build-apk")):
                    subprocess.check_call(["cargo", "run", "--", dev_flag, "-o", output_apk, "-t", target_dir,
                                           "-r", dir_to_root], env=env)
            except subprocess.CalledProcessError as e:
                print("Packaging Android exited with return value %d" % e.returncode)
                return e.returncode
        elif is_macosx():
            print("Creating Servo.app")
            dir_to_dmg = path.join(target_dir, 'dmg')
            dir_to_app = path.join(dir_to_dmg, 'Servo.app')
            dir_to_resources = path.join(dir_to_app, 'Contents', 'Resources')
            if path.exists(dir_to_dmg):
                print("Cleaning up from previous packaging")
                delete(dir_to_dmg)
            browserhtml_path = get_browserhtml_path(binary_path)

            print("Copying files")
            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
            shutil.copytree(browserhtml_path, path.join(dir_to_resources, 'browserhtml'))
            shutil.copy2(path.join(dir_to_root, 'Info.plist'), path.join(dir_to_app, 'Contents', 'Info.plist'))

            content_dir = path.join(dir_to_app, 'Contents', 'MacOS')
            os.makedirs(content_dir)
            shutil.copy2(binary_path, content_dir)

            change_prefs(dir_to_resources, "macosx")

            print("Finding dylibs and relinking")
            copy_dependencies(path.join(content_dir, 'servo'), content_dir)

            print("Adding version to Credits.rtf")
            version_command = [binary_path, '--version']
            p = subprocess.Popen(version_command,
                                 stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE,
                                 universal_newlines=True)
            version, stderr = p.communicate()
            if p.returncode != 0:
                raise Exception("Error occurred when getting Servo version: " + stderr)
            version = "Nightly version: " + version

            template_path = path.join(dir_to_resources, 'Credits.rtf.mako')
            credits_path = path.join(dir_to_resources, 'Credits.rtf')
            with open(template_path) as template_file:
                template = mako.template.Template(template_file.read())
                with open(credits_path, "w") as credits_file:
                    credits_file.write(template.render(version=version))
            delete(template_path)

            print("Writing run-servo")
            bhtml_path = path.join('${0%/*}', '..', 'Resources', 'browserhtml', 'index.html')
            runservo = os.open(
                path.join(content_dir, 'run-servo'),
                os.O_WRONLY | os.O_CREAT,
                int("0755", 8)
            )
            os.write(runservo, '#!/bin/bash\nexec ${0%/*}/servo ' + bhtml_path)
            os.close(runservo)

            print("Creating dmg")
            os.symlink('/Applications', path.join(dir_to_dmg, 'Applications'))
            dmg_path = path.join(target_dir, "servo-tech-demo.dmg")

            try:
                subprocess.check_call(['hdiutil', 'create', '-volname', 'Servo', dmg_path, '-srcfolder', dir_to_dmg])
            except subprocess.CalledProcessError as e:
                print("Packaging MacOS dmg exited with return value %d" % e.returncode)
                return e.returncode
            print("Cleaning up")
            delete(dir_to_dmg)
            print("Packaged Servo into " + dmg_path)

            print("Creating brew package")
            dir_to_brew = path.join(target_dir, 'brew_tmp')
            dir_to_tar = path.join(target_dir, 'brew')
            if not path.exists(dir_to_tar):
                os.makedirs(dir_to_tar)
            tar_path = path.join(dir_to_tar, "servo.tar.gz")
            if path.exists(dir_to_brew):
                print("Cleaning up from previous packaging")
                delete(dir_to_brew)
            if path.exists(tar_path):
                print("Deleting existing package")
                os.remove(tar_path)
            shutil.copytree(path.join(dir_to_root, 'resources'), path.join(dir_to_brew, 'resources'))
            os.makedirs(path.join(dir_to_brew, 'bin'))
            shutil.copy2(binary_path, path.join(dir_to_brew, 'bin', 'servo'))
            # Note that in the context of Homebrew, libexec is reserved for private use by the formula
            # and therefore is not symlinked into HOMEBREW_PREFIX.
            os.makedirs(path.join(dir_to_brew, 'libexec'))
            copy_dependencies(path.join(dir_to_brew, 'bin', 'servo'), path.join(dir_to_brew, 'libexec'))
            archive_deterministically(dir_to_brew, tar_path, prepend_path='servo/')
            delete(dir_to_brew)
            print("Packaged Servo into " + tar_path)
        elif is_windows():
            dir_to_msi = path.join(target_dir, 'msi')
            if path.exists(dir_to_msi):
                print("Cleaning up from previous packaging")
                delete(dir_to_msi)
            os.makedirs(dir_to_msi)
            browserhtml_path = get_browserhtml_path(binary_path)

            print("Copying files")
            dir_to_temp = path.join(dir_to_msi, 'temp')
            dir_to_temp_servo = path.join(dir_to_temp, 'servo')
            dir_to_resources = path.join(dir_to_temp_servo, 'resources')
            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
            shutil.copytree(browserhtml_path, path.join(dir_to_temp_servo, 'browserhtml'))
            shutil.copy(binary_path, dir_to_temp_servo)
            shutil.copy("{}.manifest".format(binary_path), dir_to_temp_servo)
            copy_windows_dependencies(target_dir, dir_to_temp_servo)

            change_prefs(dir_to_resources, "windows")

            # generate Servo.wxs
            template_path = path.join(dir_to_root, "support", "windows", "Servo.wxs.mako")
            template = Template(open(template_path).read())
            wxs_path = path.join(dir_to_msi, "Servo.wxs")
            open(wxs_path, "w").write(template.render(
                exe_path=target_dir,
                dir_to_temp=dir_to_temp_servo,
                resources_path=dir_to_resources))

            # run candle and light
            print("Creating MSI")
            try:
                with cd(dir_to_msi):
                    subprocess.check_call(['candle', wxs_path])
            except subprocess.CalledProcessError as e:
                print("WiX candle exited with return value %d" % e.returncode)
                return e.returncode
            try:
                wxsobj_path = "{}.wixobj".format(path.splitext(wxs_path)[0])
                with cd(dir_to_msi):
                    subprocess.check_call(['light', wxsobj_path])
            except subprocess.CalledProcessError as e:
                print("WiX light exited with return value %d" % e.returncode)
                return e.returncode
            print("Packaged Servo into " + path.join(dir_to_msi, "Servo.msi"))

            print("Creating ZIP")
            shutil.make_archive(path.join(dir_to_msi, "Servo"), "zip", dir_to_temp)
            print("Packaged Servo into " + path.join(dir_to_msi, "Servo.zip"))

            print("Cleaning up")
            delete(dir_to_temp)
        else:
            dir_to_temp = path.join(target_dir, 'packaging-temp')
            browserhtml_path = get_browserhtml_path(binary_path)
            if path.exists(dir_to_temp):
                # TODO(aneeshusa): lock dir_to_temp to prevent simultaneous builds
                print("Cleaning up from previous packaging")
                delete(dir_to_temp)

            print("Copying files")
            dir_to_resources = path.join(dir_to_temp, 'resources')
            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
            shutil.copytree(browserhtml_path, path.join(dir_to_temp, 'browserhtml'))
            shutil.copy(binary_path, dir_to_temp)

            change_prefs(dir_to_resources, "linux")

            print("Creating tarball")
            tar_path = path.join(target_dir, 'servo-tech-demo.tar.gz')

            archive_deterministically(dir_to_temp, tar_path, prepend_path='servo/')

            print("Cleaning up")
            delete(dir_to_temp)
            print("Packaged Servo into " + tar_path)

    @Command('install',
             description='Install Servo (currently, Android and Windows only)',
             category='package')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Install the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Install the dev build')
    @CommandArgument('--android',
                     action='store_true',
                     help='Install on Android')
    def install(self, release=False, dev=False, android=False):
        try:
            binary_path = self.get_binary_path(release, dev, android=android)
        except BuildNotFound:
            print("Servo build not found. Building servo...")
            result = Registrar.dispatch(
                "build", context=self.context, release=release, dev=dev, android=android
            )
            if result:
                return result
            try:
                binary_path = self.get_binary_path(release, dev, android=android)
            except BuildNotFound:
                print("Rebuilding Servo did not solve the missing build problem.")
                return 1

        if android:
            pkg_path = binary_path + ".apk"
            exec_command = ["adb", "install", "-r", pkg_path]
        elif is_windows():
            pkg_path = path.join(path.dirname(binary_path), 'msi', 'Servo.msi')
            exec_command = ["msiexec", "/i", pkg_path]

        if not path.exists(pkg_path):
            result = Registrar.dispatch(
                "package", context=self.context, release=release, dev=dev, android=android
            )
            if result != 0:
                return result

        print(" ".join(exec_command))
        return subprocess.call(exec_command, env=self.build_env())

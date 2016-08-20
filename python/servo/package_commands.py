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

import os
import shutil
import subprocess
import mako.template

from mach.registrar import Registrar
from datetime import datetime

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
)
from servo.post_build_commands import find_dep_path_newest


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
        if android:
            if dev:
                env["NDK_DEBUG"] = "1"
                env["ANT_FLAVOR"] = "debug"
                dev_flag = "-d"
            else:
                env["ANT_FLAVOR"] = "release"
                dev_flag = ""

            target_dir = path.dirname(binary_path)
            output_apk = "{}.apk".format(binary_path)
            try:
                with cd(path.join("support", "android", "build-apk")):
                    subprocess.check_call(["cargo", "run", "--", dev_flag, "-o", output_apk, "-t", target_dir,
                                           "-r", self.get_top_dir()], env=env)
            except subprocess.CalledProcessError as e:
                print("Packaging Android exited with return value %d" % e.returncode)
                return e.returncode
        elif is_macosx():

            dir_to_build = '/'.join(binary_path.split('/')[:-1])
            dir_to_root = '/'.join(binary_path.split('/')[:-3])
            now = datetime.utcnow()

            print("Creating Servo.app")
            dir_to_dmg = '/'.join(binary_path.split('/')[:-2]) + '/dmg'
            dir_to_app = dir_to_dmg + '/Servo.app'
            dir_to_resources = dir_to_app + '/Contents/Resources/'
            if path.exists(dir_to_dmg):
                print("Cleaning up from previous packaging")
                delete(dir_to_dmg)
            browserhtml_path = find_dep_path_newest('browserhtml', binary_path)
            if browserhtml_path is None:
                print("Could not find browserhtml package; perhaps you haven't built Servo.")
                return 1

            print("Copying files")
            shutil.copytree(dir_to_root + '/resources', dir_to_resources)
            shutil.copytree(browserhtml_path, dir_to_resources + browserhtml_path.split('/')[-1])
            shutil.copy2(dir_to_root + '/Info.plist', dir_to_app + '/Contents/Info.plist')
            os.makedirs(dir_to_app + '/Contents/MacOS/')
            shutil.copy2(dir_to_build + '/servo', dir_to_app + '/Contents/MacOS/')

            print("Swapping prefs")
            delete(dir_to_resources + '/prefs.json')
            shutil.copy2(dir_to_resources + 'package-prefs.json', dir_to_resources + 'prefs.json')
            delete(dir_to_resources + '/package-prefs.json')

            print("Finding dylibs and relinking")
            copy_dependencies(dir_to_app + '/Contents/MacOS/servo', dir_to_app + '/Contents/MacOS/')

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

            template_path = os.path.join(dir_to_resources, 'Credits.rtf.mako')
            credits_path = os.path.join(dir_to_resources, 'Credits.rtf')
            with open(template_path) as template_file:
                template = mako.template.Template(template_file.read())
                with open(credits_path, "w") as credits_file:
                    credits_file.write(template.render(version=version))
            delete(template_path)

            print("Writing run-servo")
            bhtml_path = path.join('${0%/*}/../Resources', browserhtml_path.split('/')[-1], 'out', 'index.html')
            runservo = os.open(dir_to_app + '/Contents/MacOS/run-servo', os.O_WRONLY | os.O_CREAT, int("0755", 8))
            os.write(runservo, '#!/bin/bash\nexec ${0%/*}/servo ' + bhtml_path)
            os.close(runservo)

            print("Creating dmg")
            os.symlink('/Applications', dir_to_dmg + '/Applications')
            dmg_path = '/'.join(dir_to_build.split('/')[:-1]) + '/'
            time = now.replace(microsecond=0).isoformat()
            time = time.replace(':', '-')
            dmg_path += time + "-servo-tech-demo.dmg"
            try:
                subprocess.check_call(['hdiutil', 'create', '-volname', 'Servo', dmg_path, '-srcfolder', dir_to_dmg])
            except subprocess.CalledProcessError as e:
                print("Packaging MacOS dmg exited with return value %d" % e.returncode)
                return e.returncode
            print("Cleaning up")
            delete(dir_to_dmg)
            print("Packaged Servo into " + dmg_path)

            print("Creating brew package")
            dir_to_brew = '/'.join(binary_path.split('/')[:-2]) + '/brew_tmp/'
            dir_to_tar = '/'.join(dir_to_build.split('/')[:-1]) + '/brew/'
            if not path.exists(dir_to_tar):
                os.makedirs(dir_to_tar)
            tar_path = dir_to_tar + now.strftime("servo-%Y-%m-%d.tar.gz")
            if path.exists(dir_to_brew):
                print("Cleaning up from previous packaging")
                delete(dir_to_brew)
            if path.exists(tar_path):
                print("Deleting existing package")
                os.remove(tar_path)
            shutil.copytree(dir_to_root + '/resources', dir_to_brew + "/resources/")
            os.makedirs(dir_to_brew + '/bin/')
            shutil.copy2(dir_to_build + '/servo', dir_to_brew + '/bin/servo')
            # Note that in the context of Homebrew, libexec is reserved for private use by the formula
            # and therefore is not symlinked into HOMEBREW_PREFIX.
            os.makedirs(dir_to_brew + '/libexec/')
            copy_dependencies(dir_to_brew + '/bin/servo', dir_to_brew + '/libexec/')
            archive_deterministically(dir_to_brew, tar_path, prepend_path='servo/')
            delete(dir_to_brew)
            print("Packaged Servo into " + tar_path)

        elif is_windows():
            dir_to_package = path.dirname(binary_path)
            dir_to_root = self.get_top_dir()
            dir_to_msi = path.join(dir_to_package, 'msi')
            if path.exists(dir_to_msi):
                print("Cleaning up from previous packaging")
                delete(dir_to_msi)
            os.makedirs(dir_to_msi)
            top_path = dir_to_root
            browserhtml_path = find_dep_path_newest('browserhtml', binary_path)
            if browserhtml_path is None:
                print("Could not find browserhtml package; perhaps you haven't built Servo.")
                return 1
            browserhtml_path = path.join(browserhtml_path, "out")
            # generate Servo.wxs
            template_path = path.join(dir_to_root, "support", "windows", "Servo.wxs.mako")
            template = Template(open(template_path).read())
            wxs_path = path.join(dir_to_msi, "Servo.wxs")
            open(wxs_path, "w").write(template.render(
                exe_path=dir_to_package,
                top_path=top_path,
                browserhtml_path=browserhtml_path))
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
            msi_path = path.join(dir_to_msi, "Servo.msi")
            print("Packaged Servo into {}".format(msi_path))
        else:
            dir_to_package = '/'.join(binary_path.split('/')[:-1])
            dir_to_root = '/'.join(binary_path.split('/')[:-3])
            resources_dir = dir_to_package + '/resources'
            if os.path.exists(resources_dir):
                delete(resources_dir)
            shutil.copytree(dir_to_root + '/resources', resources_dir)
            browserhtml_path = find_dep_path_newest('browserhtml', binary_path)
            if browserhtml_path is None:
                print("Could not find browserhtml package; perhaps you haven't built Servo.")
                return 1
            print("Deleting unused files")
            keep = ['servo', 'resources', 'build']
            for f in os.listdir(dir_to_package + '/'):
                if f not in keep:
                    delete(dir_to_package + '/' + f)
            for f in os.listdir(dir_to_package + '/build/'):
                if 'browserhtml' not in f:
                    delete(dir_to_package + '/build/' + f)
            print("Writing runservo.sh")
            # TODO: deduplicate this arg list from post_build_commands
            servo_args = ['-w', '-b',
                          '--pref', 'dom.mozbrowser.enabled',
                          '--pref', 'dom.forcetouch.enabled',
                          '--pref', 'shell.builtin-key-shortcuts.enabled=false',
                          path.join('./build/' + browserhtml_path.split('/')[-1], 'out', 'index.html')]

            runservo = os.open(dir_to_package + '/runservo.sh', os.O_WRONLY | os.O_CREAT, int("0755", 8))
            os.write(runservo, "#!/usr/bin/env sh\n./servo " + ' '.join(servo_args))
            os.close(runservo)
            print("Creating tarball")
            tar_path = '/'.join(dir_to_package.split('/')[:-1]) + '/'
            time = datetime.utcnow().replace(microsecond=0).isoformat()
            time = time.replace(':', "-")
            tar_path += time + "-servo-tech-demo.tar.gz"

            archive_deterministically(dir_to_package, tar_path, prepend_path='servo/')

            print("Packaged Servo into " + tar_path)

    @Command('install',
             description='Install Servo (currently, Android only)',
             category='package')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Install the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Install the dev build')
    def install(self, release=False, dev=False):
        try:
            binary_path = self.get_binary_path(release, dev, android=True)
        except BuildNotFound:
            print("Servo build not found. Building servo...")
            result = Registrar.dispatch(
                "build", context=self.context, release=release, dev=dev
            )
            if result:
                return result
            try:
                binary_path = self.get_binary_path(release, dev, android=True)
            except BuildNotFound:
                print("Rebuilding Servo did not solve the missing build problem.")
                return 1

        apk_path = binary_path + ".apk"
        if not path.exists(apk_path):
            result = Registrar.dispatch("package", context=self.context, release=release, dev=dev)
            if result != 0:
                return result

        print(["adb", "install", "-r", apk_path])
        return subprocess.call(["adb", "install", "-r", apk_path], env=self.build_env())

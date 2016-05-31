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
import subprocess

from mach.registrar import Registrar

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, call, check_call, BuildNotFound
from servo.post_build_commands import find_dep_path_newest

@CommandProvider
class PackageCommands(CommandBase):
    @Command('package',
             description='Package Servo (currently, Android APK only)',
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

            target_dir = os.path.dirname(binary_path)
            output_apk = "{}.apk".format(binary_path)
            try:
                with cd(path.join("support", "android", "build-apk")):
                    subprocess.check_call(["cargo", "run", "--", dev_flag, "-o", output_apk, "-t", target_dir,
                                           "-r", self.get_top_dir()], env=env)
            except subprocess.CalledProcessError as e:
                print("Packaging Android exited with return value %d" % e.returncode)
                return e.returncode
        else:
            dir_to_package = '/'.join(binary_path.split('/')[:-1])
            # write a run_servo.sh for correct browser.html invocation
            browserhtml_path = find_dep_path_newest('browserhtml', binary_path)
            if browserhtml_path is None:
                print("Could not find browserhtml package; perhaps you haven't built Servo.")
                return 1
            servo_args = ['-w', '-b',
                          '--pref', 'dom.mozbrowser.enabled',
                          '--pref', 'dom.forcetouch.enabled',
                          '--pref', 'shell.quit-on-escape.enabled=false',
                          path.join(browserhtml_path, 'out', 'index.html')]

            runservo = os.open(dir_to_package + '/runservo.sh', os.O_WRONLY | os.O_CREAT, int("0755", 8))
            os.write(runservo, "./servo " + ' '.join(servo_args))
            os.close(runservo)
            # TODO: delete unneeded files from target directory
            # tar up the whole target directory


    @Command('install',
             description='Install Servo (currently, Android only)',
             category='package')
    @CommandArgument('--release', '-r', action='store_true',
                     help='Package the release build')
    @CommandArgument('--dev', '-d', action='store_true',
                     help='Package the dev build')
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
            if result is not 0:
                return result

        print(["adb", "install", "-r", apk_path])
        return subprocess.call(["adb", "install", "-r", apk_path], env=self.build_env())

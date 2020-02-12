# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import absolute_import, print_function, unicode_literals

from datetime import datetime
import hashlib
import json
import os
import os.path as path
import platform
import shutil
import subprocess
import sys
import tempfile
import six.moves.urllib as urllib

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)
from mach.registrar import Registrar
# Note: mako cannot be imported at the top level because it breaks mach bootstrap
sys.path.append(path.join(path.dirname(__file__), "..", "..",
                          "components", "style", "properties", "Mako-0.9.1.zip"))

from servo.command_base import (
    archive_deterministically,
    BuildNotFound,
    cd,
    CommandBase,
    is_macosx,
    is_windows,
)
from servo.util import delete


PACKAGES = {
    'android': [
        'target/android/armv7-linux-androideabi/release/servoapp.apk',
        'target/android/armv7-linux-androideabi/release/servoview.aar',
    ],
    'linux': [
        'target/release/servo-tech-demo.tar.gz',
    ],
    'mac': [
        'target/release/servo-tech-demo.dmg',
    ],
    'macbrew': [
        'target/release/brew/servo.tar.gz',
    ],
    'magicleap': [
        'target/magicleap/aarch64-linux-android/release/Servo.mpk',
    ],
    'maven': [
        'target/android/gradle/servoview/maven/org/mozilla/servoview/servoview-armv7/',
        'target/android/gradle/servoview/maven/org/mozilla/servoview/servoview-x86/',
    ],
    'windows-msvc': [
        r'target\release\msi\Servo.exe',
        r'target\release\msi\Servo.zip',
    ],
    'uwp': [
        r'support\hololens\AppPackages\ServoApp\ServoApp_1.0.0.0_Test.zip',
    ],
}


TemporaryDirectory = None
if sys.version_info >= (3, 2):
    TemporaryDirectory = tempfile.TemporaryDirectory
else:
    import contextlib

    # Not quite as robust as tempfile.TemporaryDirectory,
    # but good enough for most purposes
    @contextlib.contextmanager
    def TemporaryDirectory(**kwargs):
        dir_name = tempfile.mkdtemp(**kwargs)
        try:
            yield dir_name
        except Exception as e:
            shutil.rmtree(dir_name)
            raise e


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
    for f in os.listdir(binary_path):
        if os.path.isfile(path.join(binary_path, f)) and f.endswith(".dll"):
            shutil.copy(path.join(binary_path, f), destination)


def change_prefs(resources_path, platform, vr=False):
    print("Swapping prefs")
    prefs_path = path.join(resources_path, "prefs.json")
    package_prefs_path = path.join(resources_path, "package-prefs.json")
    with open(prefs_path) as prefs, open(package_prefs_path) as package_prefs:
        prefs = json.load(prefs)
        pref_sets = []
        package_prefs = json.load(package_prefs)
        if "all" in package_prefs:
            pref_sets += [package_prefs["all"]]
        if vr and "vr" in package_prefs:
            pref_sets += [package_prefs["vr"]]
        if platform in package_prefs:
            pref_sets += [package_prefs[platform]]
        for pref_set in pref_sets:
            for pref in pref_set:
                if pref in prefs:
                    prefs[pref] = pref_set[pref]
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
    @CommandArgument('--magicleap',
                     default=None,
                     action='store_true',
                     help='Package Magic Leap')
    @CommandArgument('--target', '-t',
                     default=None,
                     help='Package for given target platform')
    @CommandArgument('--flavor', '-f',
                     default=None,
                     help='Package using the given Gradle flavor')
    @CommandArgument('--maven',
                     default=None,
                     action='store_true',
                     help='Create a local Maven repository')
    @CommandArgument('--uwp',
                     default=None,
                     action='append',
                     help='Create an APPX package')
    @CommandArgument('--ms-app-store',
                     default=None,
                     action='store_true')
    def package(self, release=False, dev=False, android=None, magicleap=None, debug=False,
                debugger=None, target=None, flavor=None, maven=False, uwp=None, ms_app_store=False):
        if android is None:
            android = self.config["build"]["android"]
        if target and android:
            print("Please specify either --target or --android.")
            sys.exit(1)
        if not android:
            android = self.handle_android_target(target)
        else:
            target = self.config["android"]["target"]
        if target and magicleap:
            print("Please specify either --target or --magicleap.")
            sys.exit(1)
        if magicleap:
            target = "aarch64-linux-android"
        env = self.build_env(target=target)
        binary_path = self.get_binary_path(
            release, dev, target=target, android=android, magicleap=magicleap,
            simpleservo=uwp is not None
        )
        dir_to_root = self.get_top_dir()
        target_dir = path.dirname(binary_path)
        if uwp:
            vs_info = self.vs_dirs()
            build_uwp(uwp, dev, vs_info['msbuild'], not ms_app_store)
        elif magicleap:
            if platform.system() not in ["Darwin"]:
                raise Exception("Magic Leap builds are only supported on macOS.")
            if not env.get("MAGICLEAP_SDK"):
                raise Exception("Magic Leap builds need the MAGICLEAP_SDK environment variable")
            if not env.get("MLCERT"):
                raise Exception("Magic Leap builds need the MLCERT environment variable")
            # GStreamer configuration
            env.setdefault("GSTREAMER_DIR", path.join(
                self.get_target_dir(), "magicleap", target, "native", "gstreamer-1.16.0"
            ))

            mabu = path.join(env.get("MAGICLEAP_SDK"), "mabu")
            packages = [
                "./support/magicleap/Servo.package",
            ]
            if dev:
                build_type = "lumin_debug"
            else:
                build_type = "lumin_release"
            for package in packages:
                argv = [
                    mabu,
                    "-o", target_dir,
                    "-t", build_type,
                    "-r",
                    "GSTREAMER_DIR=" + env["GSTREAMER_DIR"],
                    package
                ]
                try:
                    subprocess.check_call(argv, env=env)
                except subprocess.CalledProcessError as e:
                    print("Packaging Magic Leap exited with return value %d" % e.returncode)
                    return e.returncode
        elif android:
            android_target = self.config["android"]["target"]
            if "aarch64" in android_target:
                build_type = "Arm64"
            elif "armv7" in android_target:
                build_type = "Armv7"
            elif "i686" in android_target:
                build_type = "x86"
            else:
                build_type = "Arm"

            if dev:
                build_mode = "Debug"
            else:
                build_mode = "Release"

            flavor_name = "Main"
            if flavor is not None:
                flavor_name = flavor.title()

            vr = flavor == "googlevr" or flavor == "oculusvr"

            dir_to_resources = path.join(self.get_top_dir(), 'target', 'android', 'resources')
            if path.exists(dir_to_resources):
                delete(dir_to_resources)

            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
            change_prefs(dir_to_resources, "android", vr=vr)

            variant = ":assemble" + flavor_name + build_type + build_mode
            apk_task_name = ":servoapp" + variant
            aar_task_name = ":servoview" + variant
            maven_task_name = ":servoview:uploadArchive"
            argv = ["./gradlew", "--no-daemon", apk_task_name, aar_task_name]
            if maven:
                argv.append(maven_task_name)
            try:
                with cd(path.join("support", "android", "apk")):
                    subprocess.check_call(argv, env=env)
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

            print("Copying files")
            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
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

            import mako.template
            template_path = path.join(dir_to_resources, 'Credits.rtf.mako')
            credits_path = path.join(dir_to_resources, 'Credits.rtf')
            with open(template_path) as template_file:
                template = mako.template.Template(template_file.read())
                with open(credits_path, "w") as credits_file:
                    credits_file.write(template.render(version=version))
            delete(template_path)

            print("Creating dmg")
            os.symlink('/Applications', path.join(dir_to_dmg, 'Applications'))
            dmg_path = path.join(target_dir, "servo-tech-demo.dmg")

            if path.exists(dmg_path):
                print("Deleting existing dmg")
                os.remove(dmg_path)

            try:
                subprocess.check_call(['hdiutil', 'create',
                                       '-volname', 'Servo',
                                       '-megabytes', '900',
                                       dmg_path,
                                       '-srcfolder', dir_to_dmg])
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

            print("Copying files")
            dir_to_temp = path.join(dir_to_msi, 'temp')
            dir_to_resources = path.join(dir_to_temp, 'resources')
            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
            shutil.copy(binary_path, dir_to_temp)
            copy_windows_dependencies(target_dir, dir_to_temp)

            change_prefs(dir_to_resources, "windows")

            # generate Servo.wxs
            import mako.template
            template_path = path.join(dir_to_root, "support", "windows", "Servo.wxs.mako")
            template = mako.template.Template(open(template_path).read())
            wxs_path = path.join(dir_to_msi, "Installer.wxs")
            open(wxs_path, "w").write(template.render(
                exe_path=target_dir,
                dir_to_temp=dir_to_temp,
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
            dir_to_installer = path.join(dir_to_msi, "Installer.msi")
            print("Packaged Servo into " + dir_to_installer)

            # Generate bundle with Servo installer.
            print("Creating bundle")
            shutil.copy(path.join(dir_to_root, 'support', 'windows', 'Servo.wxs'), dir_to_msi)
            bundle_wxs_path = path.join(dir_to_msi, 'Servo.wxs')
            try:
                with cd(dir_to_msi):
                    subprocess.check_call(['candle', bundle_wxs_path, '-ext', 'WixBalExtension'])
            except subprocess.CalledProcessError as e:
                print("WiX candle exited with return value %d" % e.returncode)
                return e.returncode
            try:
                wxsobj_path = "{}.wixobj".format(path.splitext(bundle_wxs_path)[0])
                with cd(dir_to_msi):
                    subprocess.check_call(['light', wxsobj_path, '-ext', 'WixBalExtension'])
            except subprocess.CalledProcessError as e:
                print("WiX light exited with return value %d" % e.returncode)
                return e.returncode
            print("Packaged Servo into " + path.join(dir_to_msi, "Servo.exe"))

            print("Creating ZIP")
            zip_path = path.join(dir_to_msi, "Servo.zip")
            archive_deterministically(dir_to_temp, zip_path, prepend_path='servo/')
            print("Packaged Servo into " + zip_path)

            print("Cleaning up")
            delete(dir_to_temp)
            delete(dir_to_installer)
        else:
            dir_to_temp = path.join(target_dir, 'packaging-temp')
            if path.exists(dir_to_temp):
                # TODO(aneeshusa): lock dir_to_temp to prevent simultaneous builds
                print("Cleaning up from previous packaging")
                delete(dir_to_temp)

            print("Copying files")
            dir_to_resources = path.join(dir_to_temp, 'resources')
            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
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
    @CommandArgument('--magicleap',
                     default=None,
                     action='store_true',
                     help='Install on Magic Leap')
    @CommandArgument('--emulator',
                     action='store_true',
                     help='For Android, install to the only emulated device')
    @CommandArgument('--usb',
                     action='store_true',
                     help='For Android, install to the only USB device')
    @CommandArgument('--target', '-t',
                     default=None,
                     help='Install the given target platform')
    def install(self, release=False, dev=False, android=False, magicleap=False, emulator=False, usb=False, target=None):
        if target and android:
            print("Please specify either --target or --android.")
            sys.exit(1)
        if not android:
            android = self.handle_android_target(target)
        if target and magicleap:
            print("Please specify either --target or --magicleap.")
            sys.exit(1)
        if magicleap:
            target = "aarch64-linux-android"
        env = self.build_env(target=target)
        try:
            binary_path = self.get_binary_path(release, dev, android=android, magicleap=magicleap)
        except BuildNotFound:
            print("Servo build not found. Building servo...")
            result = Registrar.dispatch(
                "build", context=self.context, release=release, dev=dev, android=android, magicleap=magicleap,
            )
            if result:
                return result
            try:
                binary_path = self.get_binary_path(release, dev, android=android, magicleap=magicleap)
            except BuildNotFound:
                print("Rebuilding Servo did not solve the missing build problem.")
                return 1

        if magicleap:
            if not env.get("MAGICLEAP_SDK"):
                raise Exception("Magic Leap installs need the MAGICLEAP_SDK environment variable")
            mldb = path.join(env.get("MAGICLEAP_SDK"), "tools", "mldb", "mldb")
            pkg_path = path.join(path.dirname(binary_path), "Servo.mpk")
            exec_command = [
                mldb,
                "install", "-u",
                pkg_path,
            ]
        elif android:
            pkg_path = self.get_apk_path(release)
            exec_command = [self.android_adb_path(env)]
            if emulator and usb:
                print("Cannot install to both emulator and USB at the same time.")
                return 1
            if emulator:
                exec_command += ["-e"]
            if usb:
                exec_command += ["-d"]
            exec_command += ["install", "-r", pkg_path]
        elif is_windows():
            pkg_path = path.join(path.dirname(binary_path), 'msi', 'Servo.msi')
            exec_command = ["msiexec", "/i", pkg_path]

        if not path.exists(pkg_path):
            print("Servo package not found. Packaging servo...")
            result = Registrar.dispatch(
                "package", context=self.context, release=release, dev=dev, android=android, magicleap=magicleap,
            )
            if result != 0:
                return result

        print(" ".join(exec_command))
        return subprocess.call(exec_command, env=env)

    @Command('upload-nightly',
             description='Upload Servo nightly to S3',
             category='package')
    @CommandArgument('platform',
                     choices=PACKAGES.keys(),
                     help='Package platform type to upload')
    @CommandArgument('--secret-from-taskcluster',
                     action='store_true',
                     help='Retrieve the appropriate secrets from taskcluster.')
    def upload_nightly(self, platform, secret_from_taskcluster):
        import boto3

        def get_taskcluster_secret(name):
            url = (
                os.environ.get("TASKCLUSTER_PROXY_URL", "http://taskcluster") +
                "/api/secrets/v1/secret/project/servo/" +
                name
            )
            return json.load(urllib.request.urlopen(url))["secret"]

        def get_s3_secret():
            aws_access_key = None
            aws_secret_access_key = None
            if secret_from_taskcluster:
                secret = get_taskcluster_secret("s3-upload-credentials")
                aws_access_key = secret["aws_access_key_id"]
                aws_secret_access_key = secret["aws_secret_access_key"]
            return (aws_access_key, aws_secret_access_key)

        def nightly_filename(package, timestamp):
            return '{}-{}'.format(
                timestamp.isoformat() + 'Z',  # The `Z` denotes UTC
                path.basename(package)
            )

        def upload_to_s3(platform, package, timestamp):
            (aws_access_key, aws_secret_access_key) = get_s3_secret()
            s3 = boto3.client(
                's3',
                aws_access_key_id=aws_access_key,
                aws_secret_access_key=aws_secret_access_key
            )
            BUCKET = 'servo-builds'

            nightly_dir = 'nightly/{}'.format(platform)
            filename = nightly_filename(package, timestamp)
            package_upload_key = '{}/{}'.format(nightly_dir, filename)
            extension = path.basename(package).partition('.')[2]
            latest_upload_key = '{}/servo-latest.{}'.format(nightly_dir, extension)

            s3.upload_file(package, BUCKET, package_upload_key)
            copy_source = {
                'Bucket': BUCKET,
                'Key': package_upload_key,
            }
            s3.copy(copy_source, BUCKET, latest_upload_key)

        def update_maven(directory):
            (aws_access_key, aws_secret_access_key) = get_s3_secret()
            s3 = boto3.client(
                's3',
                aws_access_key_id=aws_access_key,
                aws_secret_access_key=aws_secret_access_key
            )
            BUCKET = 'servo-builds'

            nightly_dir = 'nightly/maven'
            dest_key_base = directory.replace("target/android/gradle/servoview/maven", nightly_dir)
            if dest_key_base[-1] == '/':
                dest_key_base = dest_key_base[:-1]

            # Given a directory with subdirectories like 0.0.1.20181005.caa4d190af...
            for artifact_dir in os.listdir(directory):
                base_dir = os.path.join(directory, artifact_dir)
                if not os.path.isdir(base_dir):
                    continue
                package_upload_base = "{}/{}".format(dest_key_base, artifact_dir)
                # Upload all of the files inside the subdirectory.
                for f in os.listdir(base_dir):
                    file_upload_key = "{}/{}".format(package_upload_base, f)
                    print("Uploading %s to %s" % (os.path.join(base_dir, f), file_upload_key))
                    s3.upload_file(os.path.join(base_dir, f), BUCKET, file_upload_key)

        def update_brew(package, timestamp):
            print("Updating brew formula")

            package_url = 'https://download.servo.org/nightly/macbrew/{}'.format(
                nightly_filename(package, timestamp)
            )
            with open(package) as p:
                digest = hashlib.sha256(p.read()).hexdigest()

            brew_version = timestamp.strftime('%Y.%m.%d')

            with TemporaryDirectory(prefix='homebrew-servo') as tmp_dir:
                def call_git(cmd, **kwargs):
                    subprocess.check_call(
                        ['git', '-C', tmp_dir] + cmd,
                        **kwargs
                    )

                call_git([
                    'clone',
                    'https://github.com/servo/homebrew-servo.git',
                    '.',
                ])

                script_dir = path.dirname(path.realpath(__file__))
                with open(path.join(script_dir, 'servo-binary-formula.rb.in')) as f:
                    formula = f.read()
                formula = formula.replace('PACKAGEURL', package_url)
                formula = formula.replace('SHA', digest)
                formula = formula.replace('VERSION', brew_version)
                with open(path.join(tmp_dir, 'Formula', 'servo-bin.rb'), 'w') as f:
                    f.write(formula)

                call_git(['add', path.join('.', 'Formula', 'servo-bin.rb')])
                call_git([
                    '-c', 'user.name=Tom Servo',
                    '-c', 'user.email=servo@servo.org',
                    'commit',
                    '--message=Version Bump: {}'.format(brew_version),
                ])

                if secret_from_taskcluster:
                    token = get_taskcluster_secret('github-homebrew-token')["token"]
                else:
                    token = os.environ['GITHUB_HOMEBREW_TOKEN']

                push_url = 'https://{}@github.com/servo/homebrew-servo.git'
                # TODO(aneeshusa): Use subprocess.DEVNULL with Python 3.3+
                with open(os.devnull, 'wb') as DEVNULL:
                    call_git([
                        'push',
                        '-qf',
                        push_url.format(token),
                        'master',
                    ], stdout=DEVNULL, stderr=DEVNULL)

        timestamp = datetime.utcnow().replace(microsecond=0)
        for package in PACKAGES[platform]:
            if path.isdir(package):
                continue
            if not path.isfile(package):
                print("Could not find package for {} at {}".format(
                    platform,
                    package
                ), file=sys.stderr)
                return 1
            upload_to_s3(platform, package, timestamp)

        if platform == 'maven':
            for package in PACKAGES[platform]:
                update_maven(package)

        if platform == 'macbrew':
            packages = PACKAGES[platform]
            assert(len(packages) == 1)
            update_brew(packages[0], timestamp)

        return 0


def build_uwp(platforms, dev, msbuild_dir, sign_package):
    if any(map(lambda p: p not in ['x64', 'x86', 'arm64'], platforms)):
        raise Exception("Unsupported appx platforms: " + str(platforms))
    if dev and len(platforms) > 1:
        raise Exception("Debug package with multiple architectures is unsupported")

    if dev:
        Configuration = "Debug"
    else:
        Configuration = "Release"

    msbuild = path.join(msbuild_dir, "msbuild.exe")
    build_file_template = path.join('support', 'hololens', 'package.msbuild')
    with open(build_file_template) as f:
        template_contents = f.read()
        build_file = tempfile.NamedTemporaryFile(delete=False)
        build_file.write(
            template_contents
            .replace("%%BUILD_PLATFORMS%%", ';'.join(platforms))
            .replace("%%PACKAGE_PLATFORMS%%", '|'.join(platforms))
            .replace("%%CONFIGURATION%%", Configuration)
            .replace("%%SOLUTION%%", path.join(os.getcwd(), 'support', 'hololens', 'ServoApp.sln'))
        )
        build_file.close()
        # Generate an appxbundle.
        subprocess.check_call([msbuild, "/m", build_file.name, "/p:AppxPackageSigningEnabled=" + str(sign_package)])
        os.unlink(build_file.name)

    print("Creating ZIP")
    out_dir = path.join(os.getcwd(), 'support', 'hololens', 'AppPackages', 'ServoApp')
    name = 'ServoApp_1.0.0.0_%sTest' % ('Debug_' if dev else '')
    artifacts_dir = path.join(out_dir, name)
    zip_path = path.join(out_dir, name + ".zip")
    archive_deterministically(artifacts_dir, zip_path, prepend_path='servo/')
    print("Packaged Servo into " + zip_path)

# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from datetime import datetime
import random
import time
from typing import List
from github import Github

import hashlib
import io
import json
import os
import os.path as path
import shutil
import subprocess
import sys

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)
from mach.registrar import Registrar

from servo.command_base import (
    BuildType,
    archive_deterministically,
    BuildNotFound,
    cd,
    CommandBase,
    is_macosx,
    is_windows,
)
from servo.build_commands import copy_dependencies
from servo.gstreamer import macos_gst_root
from servo.util import delete, get_target_dir

PACKAGES = {
    'android': [
        'android/armv7-linux-androideabi/production/servoapp.apk',
        'android/armv7-linux-androideabi/production/servoview.aar',
    ],
    'linux': [
        'production/servo-tech-demo.tar.gz',
    ],
    'mac': [
        'production/servo-tech-demo.dmg',
    ],
    'maven': [
        'android/gradle/servoview/maven/org/servo/servoview/servoview-armv7/',
        'android/gradle/servoview/maven/org/servo/servoview/servoview-x86/',
    ],
    'windows-msvc': [
        r'production\msi\Servo.exe',
        r'production\msi\Servo.zip',
    ],
}


def packages_for_platform(platform):
    target_dir = get_target_dir()

    for package in PACKAGES[platform]:
        yield path.join(target_dir, package)


def listfiles(directory):
    return [f for f in os.listdir(directory)
            if path.isfile(path.join(directory, f))]


def copy_windows_dependencies(binary_path, destination):
    for f in os.listdir(binary_path):
        if os.path.isfile(path.join(binary_path, f)) and f.endswith(".dll"):
            shutil.copy(path.join(binary_path, f), destination)


def change_prefs(resources_path, platform):
    print("Swapping prefs")
    prefs_path = path.join(resources_path, "prefs.json")
    package_prefs_path = path.join(resources_path, "package-prefs.json")
    with open(prefs_path) as prefs, open(package_prefs_path) as package_prefs:
        prefs = json.load(prefs)
        pref_sets = []
        package_prefs = json.load(package_prefs)
        if "all" in package_prefs:
            pref_sets += [package_prefs["all"]]
        if platform in package_prefs:
            pref_sets += [package_prefs[platform]]
        for pref_set in pref_sets:
            for pref in pref_set:
                if pref in prefs:
                    prefs[pref] = pref_set[pref]
        with open(prefs_path, "w") as out:
            json.dump(prefs, out, sort_keys=True, indent=2)
    delete(package_prefs_path)


def check_call_with_randomized_backoff(args: List[str], retries: int) -> int:
    """
    Run the given command-line arguments via `subprocess.check_call()`. If the command
    fails sleep for a random number of seconds between 2 and 5 and then try to the command
    again, the given number of times.
    """
    try:
        return subprocess.check_call(args)
    except subprocess.CalledProcessError as e:
        if retries == 0:
            raise e

        sleep_time = random.uniform(2, 5)
        print(f"Running {args} failed with {e.returncode}. Trying again in {sleep_time}s")
        time.sleep(sleep_time)
        return check_call_with_randomized_backoff(args, retries - 1)


@CommandProvider
class PackageCommands(CommandBase):
    @Command('package',
             description='Package Servo',
             category='package')
    @CommandArgument('--android',
                     default=None,
                     action='store_true',
                     help='Package Android')
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
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def package(self, build_type: BuildType, android=None, target=None, flavor=None, maven=False, with_asan=False):
        if android is None:
            android = self.config["build"]["android"]
        if target and android:
            print("Please specify either --target or --android.")
            sys.exit(1)
        if not android:
            android = self.setup_configuration_for_android_target(target)
        else:
            target = self.config["android"]["target"]

        self.cross_compile_target = target
        env = self.build_env()
        binary_path = self.get_binary_path(build_type, target=target, android=android, asan=with_asan)
        dir_to_root = self.get_top_dir()
        target_dir = path.dirname(binary_path)
        if android:
            android_target = self.config["android"]["target"]
            if "aarch64" in android_target:
                arch_string = "Arm64"
            elif "armv7" in android_target:
                arch_string = "Armv7"
            elif "i686" in android_target:
                arch_string = "x86"
            elif "x86_64" in android_target:
                arch_string = "x64"
            else:
                arch_string = "Arm"

            if build_type.is_dev():
                build_type_string = "Debug"
            elif build_type.is_release():
                build_type_string = "Release"
            else:
                raise Exception("TODO what should this be?")

            flavor_name = "Basic"
            if flavor is not None:
                flavor_name = flavor.title()

            dir_to_resources = path.join(self.get_top_dir(), 'target', 'android', 'resources')
            if path.exists(dir_to_resources):
                delete(dir_to_resources)

            shutil.copytree(path.join(dir_to_root, 'resources'), dir_to_resources)
            change_prefs(dir_to_resources, "android")

            variant = ":assemble" + flavor_name + arch_string + build_type_string
            apk_task_name = ":servoapp" + variant
            aar_task_name = ":servoview" + variant
            argv = ["./gradlew", "--no-daemon", apk_task_name, aar_task_name]
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
            lib_dir = path.join(content_dir, 'lib')
            os.makedirs(lib_dir)
            shutil.copy2(binary_path, content_dir)

            change_prefs(dir_to_resources, "macosx")

            print("Finding dylibs and relinking")
            dmg_binary = path.join(content_dir, "servo")
            dir_to_gst_lib = path.join(macos_gst_root(), 'lib', '')
            copy_dependencies(dmg_binary, lib_dir, dir_to_gst_lib)

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

            # `hdiutil` gives "Resource busy" failures on GitHub Actions at times. This
            # is an attempt to get around those issues by retrying the command a few times
            # after a random wait.
            try:
                check_call_with_randomized_backoff(
                    ['hdiutil', 'create', '-volname', 'Servo',
                     '-megabytes', '900', dmg_path,
                     '-srcfolder', dir_to_dmg],
                    retries=3)
            except subprocess.CalledProcessError as e:
                print("Packaging MacOS dmg exited with return value %d" % e.returncode)
                return e.returncode

            print("Cleaning up")
            delete(dir_to_dmg)
            print("Packaged Servo into " + dmg_path)

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
    @CommandArgument('--android',
                     action='store_true',
                     help='Install on Android')
    @CommandArgument('--emulator',
                     action='store_true',
                     help='For Android, install to the only emulated device')
    @CommandArgument('--usb',
                     action='store_true',
                     help='For Android, install to the only USB device')
    @CommandArgument('--target', '-t',
                     default=None,
                     help='Install the given target platform')
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True)
    def install(self, build_type: BuildType, android=False, emulator=False, usb=False, target=None, with_asan=False):
        if target and android:
            print("Please specify either --target or --android.")
            sys.exit(1)
        if not android:
            android = self.setup_configuration_for_android_target(target)
        self.cross_compile_target = target

        env = self.build_env()
        try:
            binary_path = self.get_binary_path(build_type, android=android, asan=with_asan)
        except BuildNotFound:
            print("Servo build not found. Building servo...")
            result = Registrar.dispatch(
                "build", context=self.context, build_type=build_type, android=android,
            )
            if result:
                return result
            try:
                binary_path = self.get_binary_path(build_type, android=android, asan=with_asan)
            except BuildNotFound:
                print("Rebuilding Servo did not solve the missing build problem.")
                return 1
        if android:
            pkg_path = self.get_apk_path(build_type)
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
                "package", context=self.context, build_type=build_type, android=android,
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
    @CommandArgument('--secret-from-environment',
                     action='store_true',
                     help='Retrieve the appropriate secrets from the environment.')
    @CommandArgument('--github-release-id',
                     default=None,
                     type=int,
                     help='The github release to upload the nightly builds.')
    def upload_nightly(self, platform, secret_from_environment, github_release_id):
        import boto3

        def get_s3_secret():
            aws_access_key = None
            aws_secret_access_key = None
            if secret_from_environment:
                secret = json.loads(os.environ['S3_UPLOAD_CREDENTIALS'])
                aws_access_key = secret["aws_access_key_id"]
                aws_secret_access_key = secret["aws_secret_access_key"]
            return (aws_access_key, aws_secret_access_key)

        def nightly_filename(package, timestamp):
            return '{}-{}'.format(
                timestamp.isoformat() + 'Z',  # The `Z` denotes UTC
                path.basename(package)
            )

        def upload_to_github_release(platform, package, package_hash):
            if not github_release_id:
                return

            extension = path.basename(package).partition('.')[2]
            g = Github(os.environ['NIGHTLY_REPO_TOKEN'])
            nightly_repo = g.get_repo(os.environ['NIGHTLY_REPO'])
            release = nightly_repo.get_release(github_release_id)
            package_hash_fileobj = io.BytesIO(package_hash.encode('utf-8'))

            asset_name = f'servo-latest.{extension}'
            release.upload_asset(package, name=asset_name)
            release.upload_asset_from_memory(
                package_hash_fileobj,
                package_hash_fileobj.getbuffer().nbytes,
                name=f'{asset_name}.sha256')

        def upload_to_s3(platform, package, package_hash, timestamp):
            (aws_access_key, aws_secret_access_key) = get_s3_secret()
            s3 = boto3.client(
                's3',
                aws_access_key_id=aws_access_key,
                aws_secret_access_key=aws_secret_access_key
            )

            cloudfront = boto3.client(
                'cloudfront',
                aws_access_key_id=aws_access_key,
                aws_secret_access_key=aws_secret_access_key
            )

            BUCKET = 'servo-builds2'
            DISTRIBUTION_ID = 'EJ8ZWSJKFCJS2'

            nightly_dir = f'nightly/{platform}'
            filename = nightly_filename(package, timestamp)
            package_upload_key = '{}/{}'.format(nightly_dir, filename)
            extension = path.basename(package).partition('.')[2]
            latest_upload_key = '{}/servo-latest.{}'.format(nightly_dir, extension)

            package_hash_fileobj = io.BytesIO(package_hash.encode('utf-8'))
            latest_hash_upload_key = f'{latest_upload_key}.sha256'

            s3.upload_file(package, BUCKET, package_upload_key)

            copy_source = {
                'Bucket': BUCKET,
                'Key': package_upload_key,
            }
            s3.copy(copy_source, BUCKET, latest_upload_key)
            s3.upload_fileobj(
                package_hash_fileobj, BUCKET, latest_hash_upload_key, ExtraArgs={'ContentType': 'text/plain'}
            )

            # Invalidate previous "latest" nightly files from
            # CloudFront edge caches
            cloudfront.create_invalidation(
                DistributionId=DISTRIBUTION_ID,
                InvalidationBatch={
                    'CallerReference': f'{latest_upload_key}-{timestamp}',
                    'Paths': {
                        'Quantity': 1,
                        'Items': [
                            f'/{latest_upload_key}*'
                        ]
                    }
                }
            )

        def update_maven(directory):
            (aws_access_key, aws_secret_access_key) = get_s3_secret()
            s3 = boto3.client(
                's3',
                aws_access_key_id=aws_access_key,
                aws_secret_access_key=aws_secret_access_key
            )
            BUCKET = 'servo-builds2'

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

        timestamp = datetime.utcnow().replace(microsecond=0)
        for package in packages_for_platform(platform):
            if path.isdir(package):
                continue
            if not path.isfile(package):
                print("Could not find package for {} at {}".format(
                    platform,
                    package
                ), file=sys.stderr)
                return 1

            # Compute the hash
            SHA_BUF_SIZE = 1048576  # read in 1 MiB chunks
            sha256_digest = hashlib.sha256()
            with open(package, 'rb') as package_file:
                while True:
                    data = package_file.read(SHA_BUF_SIZE)
                    if not data:
                        break
                    sha256_digest.update(data)
            package_hash = sha256_digest.hexdigest()

            upload_to_s3(platform, package, package_hash, timestamp)
            upload_to_github_release(platform, package, package_hash)

        if platform == 'maven':
            for package in packages_for_platform(platform):
                update_maven(package)

        return 0

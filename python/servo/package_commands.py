# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import random
from typing import List

import time
import re
import os
import os.path as path
import shutil
import subprocess
import sys

import servo.gstreamer
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
    check_output,
    CommandBase,
    is_windows,
)
from servo.util import delete, append_paths_to_env

from python.servo.platform.build_target import SanitizerKind
from servo.platform.build_target import is_android, is_openharmony


def listfiles(directory: str) -> list[str]:
    return [f for f in os.listdir(directory) if path.isfile(path.join(directory, f))]


def copy_windows_dependencies(binary_path: str, destination: str) -> None:
    for f in os.listdir(binary_path):
        if os.path.isfile(path.join(binary_path, f)) and f.endswith(".dll"):
            shutil.copy(path.join(binary_path, f), destination)


def check_call_with_randomized_backoff(args: list[str], retries: int) -> int:
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
    @Command("package", description="Package Servo", category="package")
    @CommandArgument("--android", default=None, action="store_true", help="Package Android")
    @CommandArgument("--ohos", default=None, action="store_true", help="Package OpenHarmony")
    @CommandArgument("--target", "-t", default=None, help="Package for given target platform")
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True, package_configuration=True)
    @CommandBase.allow_target_configuration
    def package(
        self, build_type: BuildType, flavor: str | None = None, sanitizer: SanitizerKind = SanitizerKind.NONE
    ) -> int | None:
        env = self.build_env()
        binary_path = self.get_binary_path(build_type, sanitizer=sanitizer)
        dir_to_root = self.get_top_dir()
        target_dir = path.dirname(binary_path)
        if is_android(self.target):
            target_triple = self.target.triple()
            if "aarch64" in target_triple:
                arch_string = "Arm64"
            elif "armv7" in target_triple:
                arch_string = "Armv7"
            elif "i686" in target_triple:
                arch_string = "x86"
            elif "x86_64" in target_triple:
                arch_string = "x64"
            else:
                arch_string = "Arm"

            if build_type.is_dev():
                build_type_string = "Debug"
            elif build_type.is_release() or build_type.is_prod():
                build_type_string = "Release"
            else:
                print(f"Servo was built with custom cargo profile `{build_type.profile}`.")
                print("Using Debug build for gradle.")
                build_type_string = "Debug"
            # Inform the android build of where `libservoshell.so` is located.
            env["SERVO_TARGET_DIR"] = target_dir

            flavor_name = "Basic"
            if flavor is not None:
                flavor_name = flavor.title()

            dir_to_resources = path.join(self.get_top_dir(), "target", "android", "resources")
            if path.exists(dir_to_resources):
                delete(dir_to_resources)

            shutil.copytree(path.join(dir_to_root, "resources"), dir_to_resources)

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
        elif is_openharmony(self.target):
            # hvigor doesn't support an option to place output files in a specific directory
            # so copy the source files into the target/openharmony directory first.
            ohos_app_dir = path.join(self.get_top_dir(), "support", "openharmony")
            build_mode = build_type.directory_name()
            ohos_target_dir = path.join(self.get_top_dir(), "target", "openharmony", self.target.triple(), build_mode)
            if path.exists(ohos_target_dir):
                print("Cleaning up from previous packaging")
                delete(ohos_target_dir)
            shutil.copytree(ohos_app_dir, ohos_target_dir)
            resources_src_dir = path.join(self.get_top_dir(), "resources")
            resources_app_dir = path.join(ohos_target_dir, "AppScope", "resources", "resfile", "servo")
            os.makedirs(resources_app_dir, exist_ok=True)
            shutil.copytree(resources_src_dir, resources_app_dir, dirs_exist_ok=True)

            # Map non-debug profiles to 'release' buildMode HAP.
            if build_type.is_custom():
                build_mode = "release"

            flavor_name = "default"
            if flavor is not None:
                flavor_name = flavor

            hvigor_command = [
                "--no-daemon",
                "assembleHap",
                "-p",
                f"product={flavor_name}",
                "-p",
                f"buildMode={build_mode}",
            ]
            if sanitizer.is_asan():
                hvigor_command.extend(["-p", "ohos-debug-asan=true"])
            elif sanitizer.is_tsan():
                hvigor_command.extend(["-p", "ohos-enable-tsan=true"])

            # Detect if PATH already has hvigor, or else fallback to npm installation
            # provided via HVIGOR_PATH
            if "HVIGOR_PATH" not in env:
                try:
                    with cd(ohos_target_dir):
                        version = check_output(["hvigorw", "--version", "--no-daemon"])
                    print(f"Found `hvigorw` with version {version.strip()} in system PATH")
                    hvigor_command[0:0] = ["hvigorw"]
                except FileNotFoundError:
                    print(
                        "Unable to find `hvigor` tool. Please either modify PATH to include the"
                        "path to hvigorw or set the HVIGOR_PATH environment variable to the npm"
                        "installation containing `node_modules` directory with hvigor modules."
                    )
                    sys.exit(1)
                except subprocess.CalledProcessError as e:
                    print(f"hvigor exited with the following error: {e}")
                    print(f"stdout: `{e.stdout}`")
                    print(f"stderr: `{e.stderr}`")
                    sys.exit(1)

            else:
                env["NODE_PATH"] = env["HVIGOR_PATH"] + "/node_modules"
                hvigor_script = f"{env['HVIGOR_PATH']}/node_modules/@ohos/hvigor/bin/hvigor.js"
                hvigor_command[0:0] = ["node", hvigor_script]
            abi_string = self.target.abi_string()
            ohos_libs_dir = path.join(ohos_target_dir, "entry", "libs", abi_string)
            os.makedirs(ohos_libs_dir)
            # The libservoshell.so binary that was built needs to be copied
            # into the app folder heirarchy where hvigor expects it.
            print(f"Copying {binary_path} to {ohos_libs_dir}")
            shutil.copy(binary_path, ohos_libs_dir)
            try:
                with cd(ohos_target_dir):
                    print("Calling", hvigor_command)
                    subprocess.check_call(hvigor_command, env=env)
            except subprocess.CalledProcessError as e:
                print("Packaging OpenHarmony exited with return value %d" % e.returncode)
                return e.returncode
        elif "darwin" in self.target.triple():
            print("Creating Servo.app")
            dir_to_dmg = path.join(target_dir, "dmg")
            dir_to_app = path.join(dir_to_dmg, "Servo.app")
            dir_to_resources = path.join(dir_to_app, "Contents", "Resources")
            if path.exists(dir_to_dmg):
                print("Cleaning up from previous packaging")
                delete(dir_to_dmg)

            print("Copying files")
            shutil.copytree(path.join(dir_to_root, "resources"), dir_to_resources)
            shutil.copy2(path.join(dir_to_root, "Info.plist"), path.join(dir_to_app, "Contents", "Info.plist"))

            content_dir = path.join(dir_to_app, "Contents", "MacOS")
            lib_dir = path.join(content_dir, "lib")
            os.makedirs(lib_dir)
            shutil.copy2(binary_path, content_dir)

            print("Packaging GStreamer...")
            dmg_binary = path.join(content_dir, "servo")
            servo.gstreamer.package_gstreamer_dylibs(dmg_binary, lib_dir, self.target)

            print("Adding version to Credits.rtf")
            version_command = [binary_path, "--version"]
            p = subprocess.Popen(
                version_command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True
            )
            version, stderr = p.communicate()
            if p.returncode != 0:
                raise Exception("Error occurred when getting Servo version: " + stderr)
            version = "Nightly version: " + version

            import mako.template

            template_path = path.join(dir_to_resources, "Credits.rtf.mako")
            credits_path = path.join(dir_to_resources, "Credits.rtf")
            with open(template_path) as template_file:
                template = mako.template.Template(template_file.read())
                with open(credits_path, "w") as credits_file:
                    credits_file.write(template.render(version=version))
            delete(template_path)

            print("Creating dmg")
            os.symlink("/Applications", path.join(dir_to_dmg, "Applications"))
            dmg_path = path.join(target_dir, "servo-tech-demo.dmg")

            if path.exists(dmg_path):
                print("Deleting existing dmg")
                os.remove(dmg_path)

            # `hdiutil` gives "Resource busy" failures on GitHub Actions at times. This
            # is an attempt to get around those issues by retrying the command a few times
            # after a random wait.
            try:
                check_call_with_randomized_backoff(
                    ["hdiutil", "create", "-volname", "Servo", "-megabytes", "900", dmg_path, "-srcfolder", dir_to_dmg],
                    retries=3,
                )
            except subprocess.CalledProcessError as e:
                print("Packaging MacOS dmg exited with return value %d" % e.returncode)
                return e.returncode

            print("Cleaning up")
            delete(dir_to_dmg)
            print("Packaged Servo into " + dmg_path)

        elif "windows" in self.target.triple():
            dir_to_msi = path.join(target_dir, "msi")
            if path.exists(dir_to_msi):
                print("Cleaning up from previous packaging")
                delete(dir_to_msi)
            os.makedirs(dir_to_msi)

            print("Copying files")
            dir_to_temp = path.join(dir_to_msi, "temp")
            dir_to_resources = path.join(dir_to_temp, "resources")
            shutil.copytree(path.join(dir_to_root, "resources"), dir_to_resources)
            shutil.copy(binary_path, dir_to_temp)
            copy_windows_dependencies(target_dir, dir_to_temp)

            # generate Servo.wxs
            import mako.template

            template_path = path.join(dir_to_root, "support", "windows", "Servo.wxs.mako")
            template = mako.template.Template(open(template_path).read())
            wxs_path = path.join(dir_to_msi, "Installer.wxs")
            open(wxs_path, "w").write(
                template.render(exe_path=target_dir, dir_to_temp=dir_to_temp, resources_path=dir_to_resources)
            )

            # If the WiX installer set the WIX env var, then add it to PATH.
            # TODO: When WIX is upgraded to v6, this won't be needed any more.
            if "WIX" in env:
                append_paths_to_env(env, "PATH", path.join(env["WIX"], "bin"))

            # run candle and light
            print("Creating MSI")
            try:
                with cd(dir_to_msi):
                    subprocess.check_call(["candle", wxs_path])
            except subprocess.CalledProcessError as e:
                print("WiX candle exited with return value %d" % e.returncode)
                return e.returncode
            try:
                wxsobj_path = "{}.wixobj".format(path.splitext(wxs_path)[0])
                with cd(dir_to_msi):
                    subprocess.check_call(["light", wxsobj_path])
            except subprocess.CalledProcessError as e:
                print("WiX light exited with return value %d" % e.returncode)
                return e.returncode
            dir_to_installer = path.join(dir_to_msi, "Installer.msi")
            print("Packaged Servo into " + dir_to_installer)

            # Generate bundle with Servo installer.
            print("Creating bundle")
            shutil.copy(path.join(dir_to_root, "support", "windows", "Servo.wxs"), dir_to_msi)
            bundle_wxs_path = path.join(dir_to_msi, "Servo.wxs")
            try:
                with cd(dir_to_msi):
                    subprocess.check_call(["candle", bundle_wxs_path, "-ext", "WixBalExtension"])
            except subprocess.CalledProcessError as e:
                print("WiX candle exited with return value %d" % e.returncode)
                return e.returncode
            try:
                wxsobj_path = "{}.wixobj".format(path.splitext(bundle_wxs_path)[0])
                with cd(dir_to_msi):
                    subprocess.check_call(["light", wxsobj_path, "-ext", "WixBalExtension"])
            except subprocess.CalledProcessError as e:
                print("WiX light exited with return value %d" % e.returncode)
                return e.returncode
            print("Packaged Servo into " + path.join(dir_to_msi, "Servo.exe"))

            print("Creating ZIP")
            zip_path = path.join(dir_to_msi, "Servo.zip")
            archive_deterministically(dir_to_temp, zip_path, prepend_path="servo/")
            print("Packaged Servo into " + zip_path)

            print("Cleaning up")
            delete(dir_to_temp)
            delete(dir_to_installer)
        else:
            dir_to_temp = path.join(target_dir, "packaging-temp")
            if path.exists(dir_to_temp):
                # TODO(aneeshusa): lock dir_to_temp to prevent simultaneous builds
                print("Cleaning up from previous packaging")
                delete(dir_to_temp)

            print("Copying files")
            dir_to_resources = path.join(dir_to_temp, "resources")
            shutil.copytree(path.join(dir_to_root, "resources"), dir_to_resources)
            shutil.copy(binary_path, dir_to_temp)

            print("Creating tarball")
            tar_path = path.join(target_dir, "servo-tech-demo.tar.gz")

            archive_deterministically(dir_to_temp, tar_path, prepend_path="servo/")

            print("Cleaning up")
            delete(dir_to_temp)
            print("Packaged Servo into " + tar_path)

    @Command("install", description="Install Servo (currently, Android and Windows only)", category="package")
    @CommandArgument("--android", action="store_true", help="Install on Android")
    @CommandArgument("--ohos", action="store_true", help="Install on OpenHarmony")
    @CommandArgument("--emulator", action="store_true", help="For Android, install to the only emulated device")
    @CommandArgument("--usb", action="store_true", help="For Android, install to the only USB device")
    @CommandArgument("--target", "-t", default=None, help="Install the given target platform")
    @CommandBase.common_command_arguments(build_configuration=False, build_type=True, package_configuration=True)
    @CommandBase.allow_target_configuration
    def install(
        self,
        build_type: BuildType,
        emulator: bool = False,
        usb: bool = False,
        sanitizer: SanitizerKind = SanitizerKind.NONE,
        flavor: str | None = None,
    ) -> int:
        env = self.build_env()
        try:
            binary_path = self.get_binary_path(build_type, sanitizer=sanitizer)
        except BuildNotFound:
            print("Servo build not found. Building servo...")
            result = Registrar.dispatch("build", context=self.context, build_type=build_type, flavor=flavor)
            if result:
                return result
            try:
                binary_path = self.get_binary_path(build_type, sanitizer=sanitizer)
            except BuildNotFound:
                print("Rebuilding Servo did not solve the missing build problem.")
                return 1

        if is_android(self.target):
            pkg_path = self.target.get_package_path(build_type.directory_name())
            exec_command = [self.android_adb_path(env)]
            if emulator and usb:
                print("Cannot install to both emulator and USB at the same time.")
                return 1
            if emulator:
                exec_command += ["-e"]
            if usb:
                exec_command += ["-d"]
            exec_command += ["install", "-r", pkg_path]
        elif is_openharmony(self.target):
            pkg_path = self.target.get_package_path(build_type.directory_name(), flavor=flavor)
            hdc_path = path.join(env["OHOS_SDK_NATIVE"], "../", "toolchains", "hdc")
            exec_command = [hdc_path, "install", "-r", pkg_path]
        elif is_windows():
            pkg_path = path.join(path.dirname(binary_path), "msi", "Servo.msi")
            exec_command = ["msiexec", "/i", pkg_path]

        if not path.exists(pkg_path):
            print("Servo package not found. Packaging servo...")
            result = Registrar.dispatch("package", context=self.context, build_type=build_type, flavor=flavor)
            if result != 0:
                return result

        print(" ".join(exec_command))
        return subprocess.call(exec_command, env=env)

    @Command("upload-nightly", description="Upload Servo nightly to S3", category="package")
    @CommandArgument("platform", help="Package platform type to upload")
    @CommandArgument(
        "--secret-from-environment", action="store_true", help="Retrieve the appropriate secrets from the environment."
    )
    @CommandArgument(
        "--github-release-id", default=None, type=int, help="The github release to upload the nightly builds."
    )
    @CommandArgument("packages", nargs="+", help="The packages to upload.")
    def upload_nightly(
        self, platform: str, secret_from_environment: bool, github_release_id: int | None, packages: List[str]
    ) -> int:
        print("Error: This command was moved to a dedicated script: etc/ci/upload_nightly.py", file=sys.stderr)
        return 1

    @Command(
        "release", description="Perform necessary updates before release a new servoshell version", category="package"
    )
    @CommandArgument("target", type=str, help="Target version to bump to")
    @CommandArgument("--allow-dirty", action="store_true", help="Allow working directory to be dirty")
    def bump_version(self, target: str, allow_dirty: bool) -> int:
        if not allow_dirty:
            # Check if the working directory is clean
            status_output = check_output(["git", "status", "--porcelain"]).strip()
            if status_output:
                print("Working directory is dirty. Please commit or stash your changes before bumping version.")
                print("To bypass this check, use --allow-dirty.")
                return 1
        print("\r ➤  Bumping version number...")
        replacements = {
            "ports/servoshell/Cargo.toml": r'^version ?= ?"(?P<version>.*?)"',
            "ports/servoshell/platform/windows/servo.exe.manifest": r'assemblyIdentity[^\/>]+version="(?P<version>.*?).0\"[^\/>]*\/>',
            "support/windows/Servo.wxs.mako": r'<Product(.|\n)*Version="(?P<version>.*?)".*>',
            "Info.plist": r"<key>CFBundleShortVersionString</key>\n\s*<string>(?P<version>.*?)</string>",
            "support/android/apk/servoapp/build.gradle.kts": r'versionName\s*=\s*"(?P<version>.*?)"',
            "support/openharmony/oh-package.json5": r'"version"\s*:\s*"(?P<version>.*?)"',
            "support/openharmony/entry/oh-package.json5": r'"version"\s*:\s*"(?P<version>.*?)"',
        }

        for filename, expression in replacements.items():
            filepath = path.join(self.get_top_dir(), filename)
            with open(filepath, "r") as file:
                content = file.read()

            compiled_pattern = re.compile(expression, re.MULTILINE)

            new_content, count = compiled_pattern.subn(
                lambda m: m.group(0).replace(m.group("version"), target),
                content,
            )

            if count == 0:
                print(f"No occurrences found in {filename} to replace.")
                return 1
            elif count > 1:
                print(f"Warning: Multiple ({count}) occurrences found in {filename}. Only one expected.")
                # Print all occurrences for debugging
                matches = compiled_pattern.findall(content)
                for match in matches:
                    print(f"Found occurrence: {match}")
                return 1

            with open(filepath, "w") as file:
                file.write(new_content)

            print(f"Updated occurrence in {filename}.")
        print("\r ➤  Updating license.html...")
        # cargo about generate etc/about.hbs > resources/resource_protocol/license.html
        try:
            # Remove resources/resource_protocol/license.html before regenerating it
            license_html_path = path.join("resources", "resource_protocol", "license.html")
            if path.exists(license_html_path):
                os.remove(license_html_path)
            subprocess.check_call(
                [
                    "cargo",
                    "about",
                    "generate",
                    "etc/about.hbs",
                ],
                stdout=open("resources/resource_protocol/license.html", "w"),
            )
        except subprocess.CalledProcessError as e:
            print("Updating license.html exited with return value %d" % e.returncode)
            return e.returncode
        return 0

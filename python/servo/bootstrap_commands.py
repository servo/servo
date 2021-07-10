# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import absolute_import, print_function, unicode_literals

import base64
import json
import os
import os.path as path
import platform
import re
import subprocess
import sys
import traceback
import six.moves.urllib as urllib
import glob

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

import servo.bootstrap as bootstrap
from servo.command_base import CommandBase, cd, check_call
from servo.util import delete, download_bytes, download_file, extract, check_hash


@CommandProvider
class MachCommands(CommandBase):
    @Command('bootstrap',
             description='Install required packages for building.',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Boostrap without confirmation')
    def bootstrap(self, force=False):
        # This entry point isn't actually invoked, ./mach bootstrap is directly
        # called by mach (see mach_bootstrap.bootstrap_command_only) so that
        # it can install dependencies without needing mach's dependencies
        return bootstrap.bootstrap(self.context, force=force)

    @Command('bootstrap-salt',
             description='Install and set up the salt environment.',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Boostrap without confirmation')
    def bootstrap_salt(self, force=False):
        return bootstrap.bootstrap(self.context, force=force, specific="salt")

    @Command('bootstrap-gstreamer',
             description='Set up a local copy of the gstreamer libraries (linux only).',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Boostrap without confirmation')
    def bootstrap_gstreamer(self, force=False):
        return bootstrap.bootstrap(self.context, force=force, specific="gstreamer")

    @Command('bootstrap-android',
             description='Install the Android SDK and NDK.',
             category='bootstrap')
    @CommandArgument('--build',
                     action='store_true',
                     help='Install Android-specific dependencies for building')
    @CommandArgument('--emulator-x86',
                     action='store_true',
                     help='Install Android x86 emulator and system image')
    @CommandArgument('--accept-all-licences',
                     action='store_true',
                     help='For non-interactive use')
    def bootstrap_android(self, build=False, emulator_x86=False, accept_all_licences=False):
        if not (build or emulator_x86):
            print("Must specify `--build` or `--emulator-x86` or both.")

        ndk = "android-ndk-r15c-{system}-{arch}"
        tools = "sdk-tools-{system}-4333796"

        emulator_platform = "android-28"
        emulator_image = "system-images;%s;google_apis;x86" % emulator_platform

        known_sha1 = {
            # https://dl.google.com/android/repository/repository2-1.xml
            "sdk-tools-darwin-4333796.zip": "ed85ea7b59bc3483ce0af4c198523ba044e083ad",
            "sdk-tools-linux-4333796.zip": "8c7c28554a32318461802c1291d76fccfafde054",
            "sdk-tools-windows-4333796.zip": "aa298b5346ee0d63940d13609fe6bec621384510",

            # https://developer.android.com/ndk/downloads/older_releases
            "android-ndk-r15c-windows-x86.zip": "f2e47121feb73ec34ced5e947cbf1adc6b56246e",
            "android-ndk-r15c-windows-x86_64.zip": "970bb2496de0eada74674bb1b06d79165f725696",
            "android-ndk-r15c-darwin-x86_64.zip": "ea4b5d76475db84745aa8828000d009625fc1f98",
            "android-ndk-r15c-linux-x86_64.zip": "0bf02d4e8b85fd770fd7b9b2cdec57f9441f27a2",
        }

        toolchains = path.join(self.context.topdir, "android-toolchains")
        if not path.isdir(toolchains):
            os.makedirs(toolchains)

        def download(target_dir, name, flatten=False):
            final = path.join(toolchains, target_dir)
            if path.isdir(final):
                return

            base_url = "https://dl.google.com/android/repository/"
            filename = name + ".zip"
            url = base_url + filename
            archive = path.join(toolchains, filename)

            if not path.isfile(archive):
                download_file(filename, url, archive)
            check_hash(archive, known_sha1[filename], "sha1")
            print("Extracting " + filename)
            remove = True  # Set to False to avoid repeated downloads while debugging this script
            if flatten:
                extracted = final + "_"
                extract(archive, extracted, remove=remove)
                contents = os.listdir(extracted)
                assert len(contents) == 1
                os.rename(path.join(extracted, contents[0]), final)
                os.rmdir(extracted)
            else:
                extract(archive, final, remove=remove)

        system = platform.system().lower()
        machine = platform.machine().lower()
        arch = {"i386": "x86"}.get(machine, machine)
        if build:
            download("ndk", ndk.format(system=system, arch=arch), flatten=True)
        download("sdk", tools.format(system=system))

        components = []
        if emulator_x86:
            components += [
                "platform-tools",
                "emulator",
                "platforms;" + emulator_platform,
                emulator_image,
            ]
        if build:
            components += [
                "platform-tools",
                "platforms;android-18",
            ]

        sdkmanager = [path.join(toolchains, "sdk", "tools", "bin", "sdkmanager")] + components
        if accept_all_licences:
            yes = subprocess.Popen(["yes"], stdout=subprocess.PIPE)
            process = subprocess.Popen(
                sdkmanager, stdin=yes.stdout, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            )
            # Reduce progress bar spam by removing duplicate lines.
            # Printing the same line again with \r is a no-op in a real terminal,
            # but each line is shown individually in Taskcluster's log viewer.
            previous_line = None
            line = b""
            while 1:
                # Read one byte at a time because in Python:
                # * readline() blocks until "\n", which doesn't come before the prompt
                # * read() blocks until EOF, which doesn't come before the prompt
                # * read(n) keeps reading until it gets n bytes or EOF,
                #   but we don't know reliably how many bytes to read until the prompt
                byte = process.stdout.read(1)
                if len(byte) == 0:
                    print(line)
                    break
                line += byte
                if byte == b'\n' or byte == b'\r':
                    if line != previous_line:
                        print(line.decode("utf-8", "replace"), end="")
                        sys.stdout.flush()
                    previous_line = line
                    line = b""
            exit_code = process.wait()
            yes.terminate()
            if exit_code:
                return exit_code
        else:
            subprocess.check_call(sdkmanager)

        if emulator_x86:
            avd_path = path.join(toolchains, "avd", "servo-x86")
            process = subprocess.Popen(stdin=subprocess.PIPE, stdout=subprocess.PIPE, args=[
                path.join(toolchains, "sdk", "tools", "bin", "avdmanager"),
                "create", "avd",
                "--path", avd_path,
                "--name", "servo-x86",
                "--package", emulator_image,
                "--force",
            ])
            output = b""
            while 1:
                # Read one byte at a time, see comment above.
                byte = process.stdout.read(1)
                if len(byte) == 0:
                    break
                output += byte
                # There seems to be no way to disable this prompt:
                if output.endswith(b"Do you wish to create a custom hardware profile? [no]"):
                    process.stdin.write("no\n")
            assert process.wait() == 0
            with open(path.join(avd_path, "config.ini"), "a") as f:
                f.write("disk.dataPartition.size=2G\n")

    @Command('update-hsts-preload',
             description='Download the HSTS preload list',
             category='bootstrap')
    def bootstrap_hsts_preload(self, force=False):
        preload_filename = "hsts_preload.json"
        preload_path = path.join(self.context.topdir, "resources")

        chromium_hsts_url = "https://chromium.googlesource.com/chromium/src" + \
            "/net/+/master/http/transport_security_state_static.json?format=TEXT"

        try:
            content_base64 = download_bytes("Chromium HSTS preload list", chromium_hsts_url)
        except urllib.error.URLError:
            print("Unable to download chromium HSTS preload list; are you connected to the internet?")
            sys.exit(1)

        content_decoded = base64.b64decode(content_base64)

        # The chromium "json" has single line comments in it which, of course,
        # are non-standard/non-valid json. Simply strip them out before parsing
        content_json = re.sub(r'(^|\s+)//.*$', '', content_decoded, flags=re.MULTILINE)

        try:
            pins_and_static_preloads = json.loads(content_json)
            entries = {
                "entries": [
                    {
                        "host": e["name"],
                        "include_subdomains": e.get("include_subdomains", False)
                    }
                    for e in pins_and_static_preloads["entries"]
                ]
            }

            with open(path.join(preload_path, preload_filename), 'w') as fd:
                json.dump(entries, fd, indent=4)
        except ValueError:
            print("Unable to parse chromium HSTS preload list, has the format changed?")
            sys.exit(1)

    @Command('update-pub-domains',
             description='Download the public domains list and update resources/public_domains.txt',
             category='bootstrap')
    def bootstrap_pub_suffix(self, force=False):
        list_url = "https://publicsuffix.org/list/public_suffix_list.dat"
        dst_filename = path.join(self.context.topdir, "resources", "public_domains.txt")
        not_implemented_case = re.compile(r'^[^*]+\*')

        try:
            content = download_bytes("Public suffix list", list_url)
        except urllib.error.URLError:
            print("Unable to download the public suffix list; are you connected to the internet?")
            sys.exit(1)

        lines = [line.strip() for line in content.decode("utf8").split("\n")]
        suffixes = [line for line in lines if not line.startswith("//") and not line == ""]

        with open(dst_filename, "wb") as fo:
            for suffix in suffixes:
                if not_implemented_case.match(suffix):
                    print("Warning: the new list contains a case that servo can't handle: %s" % suffix)
                fo.write(suffix.encode("idna") + "\n")

    @Command('clean-nightlies',
             description='Clean unused nightly builds of Rust and Cargo',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Actually remove stuff')
    @CommandArgument('--keep',
                     default='1',
                     help='Keep up to this many most recent nightlies')
    def clean_nightlies(self, force=False, keep=None):
        print("Current Rust version for Servo: {}".format(self.rust_toolchain()))
        old_toolchains = []
        keep = int(keep)
        stdout = subprocess.check_output(['git', 'log', '--format=%H', 'rust-toolchain'])
        for i, commit_hash in enumerate(stdout.split(), 1):
            if i > keep:
                toolchain = subprocess.check_output(
                    ['git', 'show', '%s:rust-toolchain' % commit_hash])
                old_toolchains.append(toolchain.strip())

        removing_anything = False
        stdout = subprocess.check_output(['rustup', 'toolchain', 'list'])
        for toolchain_with_host in stdout.split():
            for old in old_toolchains:
                if toolchain_with_host.startswith(old):
                    removing_anything = True
                    if force:
                        print("Removing {}".format(toolchain_with_host))
                        check_call(["rustup", "uninstall", toolchain_with_host])
                    else:
                        print("Would remove {}".format(toolchain_with_host))
        if not removing_anything:
            print("Nothing to remove.")
        elif not force:
            print("Nothing done. "
                  "Run `./mach clean-nightlies -f` to actually remove.")

    @Command('clean-cargo-cache',
             description='Clean unused Cargo packages',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Actually remove stuff')
    @CommandArgument('--show-size', '-s',
                     action='store_true',
                     help='Show packages size')
    @CommandArgument('--keep',
                     default='1',
                     help='Keep up to this many most recent dependencies')
    def clean_cargo_cache(self, force=False, show_size=False, keep=None):
        def get_size(path):
            if os.path.isfile(path):
                return os.path.getsize(path) / (1024 * 1024.0)
            total_size = 0
            for dirpath, dirnames, filenames in os.walk(path):
                for f in filenames:
                    fp = os.path.join(dirpath, f)
                    total_size += os.path.getsize(fp)
            return total_size / (1024 * 1024.0)

        removing_anything = False
        packages = {
            'crates': {},
            'git': {},
        }
        import toml
        if os.environ.get("CARGO_HOME", ""):
            cargo_dir = os.environ.get("CARGO_HOME")
        else:
            home_dir = os.path.expanduser("~")
            cargo_dir = path.join(home_dir, ".cargo")
        if not os.path.isdir(cargo_dir):
            return
        cargo_file = open(path.join(self.context.topdir, "Cargo.lock"))
        content = toml.load(cargo_file)

        for package in content.get("package", []):
            source = package.get("source", "")
            version = package["version"]
            if source == u"registry+https://github.com/rust-lang/crates.io-index":
                crate_name = "{}-{}".format(package["name"], version)
                if not packages["crates"].get(crate_name, False):
                    packages["crates"][package["name"]] = {
                        "current": [],
                        "exist": [],
                    }
                packages["crates"][package["name"]]["current"].append(crate_name)
            elif source.startswith("git+"):
                name = source.split("#")[0].split("/")[-1].replace(".git", "")
                branch = ""
                crate_name = "{}-{}".format(package["name"], source.split("#")[1])
                crate_branch = name.split("?")
                if len(crate_branch) > 1:
                    branch = crate_branch[1].replace("branch=", "")
                    name = crate_branch[0]

                if not packages["git"].get(name, False):
                    packages["git"][name] = {
                        "current": [],
                        "exist": [],
                    }
                packages["git"][name]["current"].append(source.split("#")[1][:7])
                if branch:
                    packages["git"][name]["current"].append(branch)

        crates_dir = path.join(cargo_dir, "registry")
        crates_cache_dir = ""
        crates_src_dir = ""
        if os.path.isdir(path.join(crates_dir, "cache")):
            for p in os.listdir(path.join(crates_dir, "cache")):
                crates_cache_dir = path.join(crates_dir, "cache", p)
                crates_src_dir = path.join(crates_dir, "src", p)

        git_dir = path.join(cargo_dir, "git")
        git_db_dir = path.join(git_dir, "db")
        git_checkout_dir = path.join(git_dir, "checkouts")
        if os.path.isdir(git_db_dir):
            git_db_list = filter(lambda f: not f.startswith('.'), os.listdir(git_db_dir))
        else:
            git_db_list = []
        if os.path.isdir(git_checkout_dir):
            git_checkout_list = os.listdir(git_checkout_dir)
        else:
            git_checkout_list = []

        for d in list(set(git_db_list + git_checkout_list)):
            crate_name = d.replace("-{}".format(d.split("-")[-1]), "")
            if not packages["git"].get(crate_name, False):
                packages["git"][crate_name] = {
                    "current": [],
                    "exist": [],
                }
            if os.path.isdir(path.join(git_checkout_dir, d)):
                with cd(path.join(git_checkout_dir, d)):
                    git_crate_hash = glob.glob('*')
                if not git_crate_hash or not os.path.isdir(path.join(git_db_dir, d)):
                    packages["git"][crate_name]["exist"].append(("del", d, ""))
                    continue
                for d2 in git_crate_hash:
                    dep_path = path.join(git_checkout_dir, d, d2)
                    if os.path.isdir(dep_path):
                        packages["git"][crate_name]["exist"].append((path.getmtime(dep_path), d, d2))
            elif os.path.isdir(path.join(git_db_dir, d)):
                packages["git"][crate_name]["exist"].append(("del", d, ""))

        if crates_src_dir:
            for d in os.listdir(crates_src_dir):
                crate_name = re.sub(r"\-\d+(\.\d+){1,3}.+", "", d)
                if not packages["crates"].get(crate_name, False):
                    packages["crates"][crate_name] = {
                        "current": [],
                        "exist": [],
                    }
                packages["crates"][crate_name]["exist"].append(d)

        total_size = 0
        for packages_type in ["git", "crates"]:
            sorted_packages = sorted(packages[packages_type])
            for crate_name in sorted_packages:
                crate_count = 0
                existed_crates = packages[packages_type][crate_name]["exist"]
                for exist in sorted(existed_crates, reverse=True):
                    current_crate = packages[packages_type][crate_name]["current"]
                    size = 0
                    exist_name = path.join(exist[1], exist[2]) if packages_type == "git" else exist
                    exist_item = exist[2] if packages_type == "git" else exist
                    if exist_item not in current_crate:
                        crate_count += 1
                        if int(crate_count) >= int(keep) or not current_crate or \
                           exist[0] == "del" or exist[2] == "master":
                            removing_anything = True
                            crate_paths = []
                            if packages_type == "git":
                                exist_checkout_path = path.join(git_checkout_dir, exist[1])
                                exist_db_path = path.join(git_db_dir, exist[1])
                                exist_path = path.join(git_checkout_dir, exist_name)

                                if exist[0] == "del":
                                    if os.path.isdir(exist_checkout_path):
                                        crate_paths.append(exist_checkout_path)
                                    if os.path.isdir(exist_db_path):
                                        crate_paths.append(exist_db_path)
                                    crate_count += -1
                                else:
                                    crate_paths.append(exist_path)

                                    exist_checkout_list = glob.glob(path.join(exist_checkout_path, '*'))
                                    if len(exist_checkout_list) <= 1:
                                        crate_paths.append(exist_checkout_path)
                                        if os.path.isdir(exist_db_path):
                                            crate_paths.append(exist_db_path)
                            else:
                                crate_paths.append(path.join(crates_cache_dir, "{}.crate".format(exist)))
                                crate_paths.append(path.join(crates_src_dir, exist))

                            size = sum(get_size(p) for p in crate_paths) if show_size else 0
                            total_size += size
                            print_msg = (exist_name, " ({}MB)".format(round(size, 2)) if show_size else "", cargo_dir)
                            if force:
                                print("Removing `{}`{} package from {}".format(*print_msg))
                                for crate_path in crate_paths:
                                    if os.path.exists(crate_path):
                                        try:
                                            delete(crate_path)
                                        except Exception:
                                            print(traceback.format_exc())
                                            print("Delete %s failed!" % crate_path)
                            else:
                                print("Would remove `{}`{} package from {}".format(*print_msg))

        if removing_anything and show_size:
            print("\nTotal size of {} MB".format(round(total_size, 2)))

        if not removing_anything:
            print("Nothing to remove.")
        elif not force:
            print("\nNothing done. "
                  "Run `./mach clean-cargo-cache -f` to actually remove.")

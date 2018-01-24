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
import re
import subprocess
import sys
import urllib2
import glob

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

import servo.bootstrap as bootstrap
from servo.command_base import CommandBase, cd, check_call
from servo.util import delete, download_bytes


@CommandProvider
class MachCommands(CommandBase):
    @Command('env',
             description='Print environment setup commands',
             category='bootstrap')
    def env(self):
        env = self.build_env()
        print("export RUSTFLAGS=%s" % env["RUSTFLAGS"])
        print("export PATH=%s" % env["PATH"])
        if sys.platform == "darwin":
            print("export DYLD_LIBRARY_PATH=%s" % env["DYLD_LIBRARY_PATH"])
        else:
            print("export LD_LIBRARY_PATH=%s" % env["LD_LIBRARY_PATH"])

    @Command('bootstrap',
             description='Install required packages for building.',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Boostrap without confirmation')
    def bootstrap(self, force=False):
        return bootstrap.bootstrap(self.context, force=force)

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
        except urllib2.URLError:
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
        except ValueError, e:
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
        except urllib2.URLError:
            print("Unable to download the public suffix list; are you connected to the internet?")
            sys.exit(1)

        lines = [l.strip() for l in content.decode("utf8").split("\n")]
        suffixes = [l for l in lines if not l.startswith("//") and not l == ""]

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
        default_toolchain = self.default_toolchain()
        geckolib_toolchain = self.geckolib_toolchain()
        print("Current Rust version for Servo: {}".format(default_toolchain))
        print("Current Rust version for geckolib: {}".format(geckolib_toolchain))
        old_toolchains = []
        keep = int(keep)
        for toolchain_file in ['rust-toolchain', 'geckolib-rust-toolchain']:
            stdout = subprocess.check_output(['git', 'log', '--format=%H', toolchain_file])
            for i, commit_hash in enumerate(stdout.split(), 1):
                if i > keep:
                    toolchain = subprocess.check_output(
                        ['git', 'show', '%s:%s' % (commit_hash, toolchain_file)])
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
        git_db_list = filter(lambda f: not f.startswith('.'), os.listdir(git_db_dir))
        git_checkout_list = os.listdir(git_checkout_dir)

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
                                        except:
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

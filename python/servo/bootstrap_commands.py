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
import shutil
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
from servo.command_base import CommandBase, BIN_SUFFIX, cd
from servo.util import delete, download_bytes, download_file, extract, host_triple


@CommandProvider
class MachCommands(CommandBase):
    @Command('env',
             description='Print environment setup commands',
             category='bootstrap')
    def env(self):
        env = self.build_env()
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

    @Command('bootstrap-rust',
             description='Download the Rust compiler',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Force download even if a copy already exists')
    @CommandArgument('--target',
                     action='append',
                     default=[],
                     help='Download rust stdlib for specified target')
    @CommandArgument('--stable',
                     action='store_true',
                     help='Use stable rustc version')
    def bootstrap_rustc(self, force=False, target=[], stable=False):
        self.set_use_stable_rust(stable)
        rust_dir = path.join(self.context.sharedir, "rust", self.rust_path())
        install_dir = path.join(self.context.sharedir, "rust", self.rust_install_dir())
        version = self.rust_stable_version() if stable else "nightly"
        static_s3 = "https://static-rust-lang-org.s3.amazonaws.com/dist"

        if not force and path.exists(path.join(rust_dir, "rustc", "bin", "rustc" + BIN_SUFFIX)):
            print("Rust compiler already downloaded.", end=" ")
            print("Use |bootstrap-rust --force| to download again.")
        else:
            if path.isdir(rust_dir):
                shutil.rmtree(rust_dir)
            os.makedirs(rust_dir)

            # The nightly Rust compiler is hosted on the nightly server under the date with a name
            # rustc-nightly-HOST-TRIPLE.tar.gz, whereas the stable compiler is named
            # rustc-VERSION-HOST-TRIPLE.tar.gz. We just need to pull down and extract it,
            # giving a directory name that will be the same as the tarball name (rustc is
            # in that directory).
            if stable:
                base_url = static_s3
            else:
                nightly_commit_hash = self.rust_nightly_date()

                base_url = "https://s3.amazonaws.com/rust-lang-ci/rustc-builds"
                if not self.config["build"]["llvm-assertions"]:
                    base_url += "-alt"
                base_url += "/" + nightly_commit_hash

            rustc_url = base_url + "/rustc-%s-%s.tar.gz" % (version, host_triple())
            tgz_file = rust_dir + '-rustc.tar.gz'
            download_file("Rust compiler", rustc_url, tgz_file)

            print("Extracting Rust compiler...")
            extract(tgz_file, install_dir)
            print("Rust compiler ready.")

        # Each Rust stdlib has a name of the form `rust-std-nightly-TRIPLE.tar.gz` for the nightly
        # releases, or rust-std-VERSION-TRIPLE.tar.gz for stable releases, with
        # a directory of the name `rust-std-TRIPLE` inside and then a `lib` directory.
        # This `lib` directory needs to be extracted and merged with the `rustc/lib`
        # directory from the host compiler above.
        lib_dir = path.join(install_dir,
                            "rustc-%s-%s" % (version, host_triple()),
                            "rustc", "lib", "rustlib")

        # ensure that the libs for the host's target is downloaded
        host_target = host_triple()
        if host_target not in target:
            target.append(host_target)

        for target_triple in target:
            target_lib_dir = path.join(lib_dir, target_triple)
            if path.exists(target_lib_dir):
                # No need to check for force. If --force the directory is already deleted
                print("Rust lib for target {} already downloaded.".format(target_triple), end=" ")
                print("Use |bootstrap-rust --force| to download again.")
                continue

            tarball = "rust-std-%s-%s.tar.gz" % (version, target_triple)
            tgz_file = path.join(install_dir, tarball)
            if self.use_stable_rust():
                std_url = static_s3 + "/" + tarball
            else:
                ci = "https://s3.amazonaws.com/rust-lang-ci/rustc-builds"
                std_url = ci + "/" + self.rust_nightly_date() + "/" + tarball

            download_file("Host rust library for target %s" % target_triple, std_url, tgz_file)
            print("Extracting Rust stdlib for target %s..." % target_triple)
            extract(tgz_file, install_dir)
            shutil.copytree(path.join(install_dir,
                                      "rust-std-%s-%s" % (version, target_triple),
                                      "rust-std-%s" % target_triple,
                                      "lib", "rustlib", target_triple),
                            path.join(install_dir,
                                      "rustc-%s-%s" % (version, host_triple()),
                                      "rustc",
                                      "lib", "rustlib", target_triple))
            shutil.rmtree(path.join(install_dir, "rust-std-%s-%s" % (version, target_triple)))

            print("Rust {} libs ready.".format(target_triple))

    @Command('bootstrap-rust-docs',
             description='Download the Rust documentation',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Force download even if docs already exist')
    def bootstrap_rustc_docs(self, force=False):
        self.ensure_bootstrapped()
        rust_root = self.config["tools"]["rust-root"]
        docs_dir = path.join(rust_root, "doc")
        if not force and path.exists(docs_dir):
            print("Rust docs already downloaded.", end=" ")
            print("Use |bootstrap-rust-docs --force| to download again.")
            return

        if path.isdir(docs_dir):
            shutil.rmtree(docs_dir)
        docs_name = self.rust_path().replace("rustc-", "rust-docs-")
        docs_url = ("https://static-rust-lang-org.s3.amazonaws.com/dist/%s/rust-docs-nightly-%s.tar.gz"
                    % (self.rust_nightly_date(), host_triple()))
        tgz_file = path.join(rust_root, 'doc.tar.gz')

        download_file("Rust docs", docs_url, tgz_file)

        print("Extracting Rust docs...")
        temp_dir = path.join(rust_root, "temp_docs")
        if path.isdir(temp_dir):
            shutil.rmtree(temp_dir)
        extract(tgz_file, temp_dir)
        shutil.move(path.join(temp_dir, docs_name.split("/")[1],
                              "rust-docs", "share", "doc", "rust", "html"),
                    docs_dir)
        shutil.rmtree(temp_dir)
        print("Rust docs ready.")

    @Command('bootstrap-cargo',
             description='Download the Cargo build tool',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Force download even if cargo already exists')
    def bootstrap_cargo(self, force=False):
        cargo_dir = path.join(self.context.sharedir, "cargo", self.rust_nightly_date())
        if not force and path.exists(path.join(cargo_dir, "cargo", "bin", "cargo" + BIN_SUFFIX)):
            print("Cargo already downloaded.", end=" ")
            print("Use |bootstrap-cargo --force| to download again.")
            return

        if path.isdir(cargo_dir):
            shutil.rmtree(cargo_dir)
        os.makedirs(cargo_dir)

        tgz_file = "cargo-nightly-%s.tar.gz" % host_triple()
        nightly_url = "https://s3.amazonaws.com/rust-lang-ci/rustc-builds/%s/%s" % \
            (self.rust_nightly_date(), tgz_file)

        download_file("Cargo nightly", nightly_url, tgz_file)

        print("Extracting Cargo nightly...")
        nightly_dir = path.join(cargo_dir,
                                path.basename(tgz_file).replace(".tar.gz", ""))
        extract(tgz_file, cargo_dir, movedir=nightly_dir)
        print("Cargo ready.")

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
        rust_current_nightly = self.rust_nightly_date()
        rust_current_stable = self.rust_stable_version()
        print("Current Rust nightly version: {}".format(rust_current_nightly))
        print("Current Rust stable version: {}".format(rust_current_stable))
        to_keep = set()
        if int(keep) == 1:
            # Optimize keep=1 case to not invoke git
            to_keep.add(rust_current_nightly)
            to_keep.add(rust_current_stable)
        else:
            for version_file in ['rust-toolchain', 'rust-stable-version']:
                cmd = subprocess.Popen(
                    ['git', 'log', '--oneline', '--no-color', '-n', keep, '--patch', version_file],
                    stdout=subprocess.PIPE,
                    universal_newlines=True
                )
                stdout, _ = cmd.communicate()
                for line in stdout.splitlines():
                    if line.startswith(b"+") and not line.startswith(b"+++"):
                        to_keep.add(line[1:])

        removing_anything = False
        for tool in ["rust", "cargo"]:
            base = path.join(self.context.sharedir, tool)
            if not path.isdir(base):
                continue
            for name in os.listdir(base):
                if name.startswith("rust-"):
                    name = name[len("rust-"):]
                # We append `-alt` if LLVM assertions aren't enabled,
                # so use just the commit hash itself.
                # This may occasionally leave an extra nightly behind
                # but won't remove too many nightlies.
                if name.partition('-')[0] not in to_keep:
                    removing_anything = True
                    full_path = path.join(base, name)
                    if force:
                        print("Removing {}".format(full_path))
                        try:
                            delete(full_path)
                        except OSError as e:
                            print("Removal failed with error {}".format(e))
                    else:
                        print("Would remove {}".format(full_path))
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
    @CommandArgument('--custom-path', '-c',
                     action='store_true',
                     help='Get Cargo path from CARGO_HOME environment variable')
    def clean_cargo_cache(self, force=False, show_size=False, keep=None, custom_path=False):
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
        if os.environ.get("CARGO_HOME", "") and custom_path:
            cargo_dir = os.environ.get("CARGO_HOME")
        else:
            cargo_dir = path.join(self.context.topdir, ".cargo")
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
                                        delete(crate_path)
                            else:
                                print("Would remove `{}`{} package from {}".format(*print_msg))

        if removing_anything and show_size:
            print("\nTotal size of {} MB".format(round(total_size, 2)))

        if not removing_anything:
            print("Nothing to remove.")
        elif not force:
            print("\nNothing done. "
                  "Run `./mach clean-cargo-cache -f` to actually remove.")

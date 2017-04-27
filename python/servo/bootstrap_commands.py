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

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

import servo.bootstrap as bootstrap
from servo.command_base import CommandBase, BIN_SUFFIX
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
        version = self.rust_version()
        rust_path = self.rust_path()
        rust_dir = path.join(self.context.sharedir, "rust", rust_path)
        install_dir = path.join(self.context.sharedir, "rust", version)
        if not self.config["build"]["llvm-assertions"]:
            if not self.use_stable_rust():
                install_dir += "-alt"

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
                tarball = "rustc-%s-%s.tar.gz" % (version, host_triple())
                rustc_url = "https://static-rust-lang-org.s3.amazonaws.com/dist/" + tarball
            else:
                tarball = "%s/rustc-nightly-%s.tar.gz" % (version, host_triple())
                base_url = "https://s3.amazonaws.com/rust-lang-ci/rustc-builds"
                if not self.config["build"]["llvm-assertions"]:
                    base_url += "-alt"
                rustc_url = base_url + "/" + tarball
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
        nightly_suffix = "" if stable else "-nightly"
        stable_version = "-{}".format(version) if stable else ""
        lib_dir = path.join(install_dir,
                            "rustc{}{}-{}".format(nightly_suffix, stable_version, host_triple()),
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

            if self.use_stable_rust():
                std_url = ("https://static-rust-lang-org.s3.amazonaws.com/dist/rust-std-%s-%s.tar.gz"
                           % (version, target_triple))
                tgz_file = install_dir + ('rust-std-%s-%s.tar.gz' % (version, target_triple))
            else:
                std_url = ("https://s3.amazonaws.com/rust-lang-ci/rustc-builds/%s/rust-std-nightly-%s.tar.gz"
                           % (version, target_triple))
                tgz_file = install_dir + ('rust-std-nightly-%s.tar.gz' % target_triple)

            download_file("Host rust library for target %s" % target_triple, std_url, tgz_file)
            print("Extracting Rust stdlib for target %s..." % target_triple)
            extract(tgz_file, install_dir)
            shutil.copytree(path.join(install_dir,
                                      "rust-std%s%s-%s" % (nightly_suffix, stable_version, target_triple),
                                      "rust-std-%s" % target_triple, "lib", "rustlib", target_triple),
                            path.join(install_dir,
                                      "rustc%s%s-%s" % (nightly_suffix, stable_version, host_triple()),
                                      "rustc", "lib", "rustlib", target_triple))
            shutil.rmtree(path.join(install_dir,
                          "rust-std%s%s-%s" % (nightly_suffix, stable_version, target_triple)))

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
        docs_url = ("https://static-rust-lang-org.s3.amazonaws.com/dist/rust-docs-nightly-%s.tar.gz"
                    % host_triple())
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
        cargo_dir = path.join(self.context.sharedir, "cargo",
                              self.cargo_build_id())
        if not force and path.exists(path.join(cargo_dir, "cargo", "bin", "cargo" + BIN_SUFFIX)):
            print("Cargo already downloaded.", end=" ")
            print("Use |bootstrap-cargo --force| to download again.")
            return

        if path.isdir(cargo_dir):
            shutil.rmtree(cargo_dir)
        os.makedirs(cargo_dir)

        tgz_file = "cargo-nightly-%s.tar.gz" % host_triple()
        nightly_url = "https://s3.amazonaws.com/rust-lang-ci/cargo-builds/%s/%s" % \
            (self.cargo_build_id(), tgz_file)

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
        rust_current = self.rust_version()
        cargo_current = self.cargo_build_id()
        print("Current Rust version: {}".format(rust_current))
        print("Current Cargo version: {}".format(cargo_current))
        to_keep = {
            'rust': set(),
            'cargo': set(),
        }
        if int(keep) == 1:
            # Optimize keep=1 case to not invoke git
            to_keep['rust'].add(rust_current)
            to_keep['cargo'].add(cargo_current)
        else:
            for tool in ["rust", "cargo"]:
                commit_file = '{}-commit-hash'.format(tool)
                cmd = subprocess.Popen(
                    ['git', 'log', '--oneline', '--no-color', '-n', keep, '--patch', commit_file],
                    stdout=subprocess.PIPE,
                    universal_newlines=True
                )
                stdout, _ = cmd.communicate()
                for line in stdout.splitlines():
                    if line.startswith("+") and not line.startswith("+++"):
                        to_keep[tool].add(line[1:])

        removing_anything = False
        for tool in ["rust", "cargo"]:
            base = path.join(self.context.sharedir, tool)
            for name in os.listdir(base):
                # We append `-alt` if LLVM assertions aren't enabled,
                # so use just the commit hash itself.
                # This may occasionally leave an extra nightly behind
                # but won't remove too many nightlies.
                if name.partition('-')[0] not in to_keep[tool]:
                    removing_anything = True
                    full_path = path.join(base, name)
                    if force:
                        print("Removing {}".format(full_path))
                        delete(full_path)
                    else:
                        print("Would remove {}".format(full_path))
        if not removing_anything:
            print("Nothing to remove.")
        elif not force:
            print("Nothing done. "
                  "Run `./mach clean-nightlies -f` to actually remove.")

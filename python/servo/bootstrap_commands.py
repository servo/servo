# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function, unicode_literals

import base64
import json
import os
import os.path as path
import re
import shutil
import subprocess
import sys
import StringIO
import tarfile
import urllib2
from distutils.version import LooseVersion

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, host_triple


def download(desc, src, writer):
    print("Downloading %s..." % desc)
    dumb = (os.environ.get("TERM") == "dumb") or (not sys.stdout.isatty())

    try:
        resp = urllib2.urlopen(src)

        fsize = None
        if resp.info().getheader('Content-Length'):
            fsize = int(resp.info().getheader('Content-Length').strip())

        recved = 0
        chunk_size = 8192

        while True:
            chunk = resp.read(chunk_size)
            if not chunk:
                break
            recved += len(chunk)
            if not dumb:
                if fsize is not None:
                    pct = recved * 100.0 / fsize
                    print("\rDownloading %s: %5.1f%%" % (desc, pct), end="")

                sys.stdout.flush()
            writer.write(chunk)

        if not dumb:
            print()
    except urllib2.HTTPError, e:
        print("Download failed (%d): %s - %s" % (e.code, e.reason, src))

        cpu_type = subprocess.check_output(["uname", "-m"]).strip().lower()
        if e.code == 404 and cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
            # i686
            print("Note: Servo does not currently bootstrap 32bit snapshots of Rust")
            print("See https://github.com/servo/servo/issues/3899")

        sys.exit(1)


def download_file(desc, src, dst):
    with open(dst, 'wb') as fd:
        download(desc, src, fd)


def download_bytes(desc, src):
    content_writer = StringIO.StringIO()
    download(desc, src, content_writer)
    return content_writer.getvalue()


def extract(src, dst, movedir=None):
    tarfile.open(src).extractall(dst)

    if movedir:
        for f in os.listdir(movedir):
            frm = path.join(movedir, f)
            to = path.join(dst, f)
            os.rename(frm, to)
        os.rmdir(movedir)

    os.remove(src)


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

    @Command('bootstrap-rust',
             description='Download the Rust compiler snapshot',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Force download even if a snapshot already exists')
    def bootstrap_rustc(self, force=False):
        rust_dir = path.join(
            self.context.sharedir, "rust", *self.rust_snapshot_path().split("/"))
        if not force and path.exists(path.join(rust_dir, "rustc", "bin", "rustc")):
            print("Snapshot Rust compiler already downloaded.", end=" ")
            print("Use |bootstrap-rust --force| to download again.")
            return

        if path.isdir(rust_dir):
            shutil.rmtree(rust_dir)
        os.makedirs(rust_dir)

        snapshot_url = ("https://servo-rust.s3.amazonaws.com/%s.tar.gz"
                        % self.rust_snapshot_path())
        tgz_file = rust_dir + '.tar.gz'

        download_file("Rust snapshot", snapshot_url, tgz_file)

        print("Extracting Rust snapshot...")
        snap_dir = path.join(rust_dir,
                             path.basename(tgz_file).replace(".tar.gz", ""))
        extract(tgz_file, rust_dir, movedir=snap_dir)
        print("Snapshot Rust ready.")

    @Command('bootstrap-rust-docs',
             description='Download the Rust documentation',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Force download even if docs already exist')
    def bootstrap_rustc_docs(self, force=False):
        self.ensure_bootstrapped()
        hash_dir = path.join(self.context.sharedir, "rust",
                             self.rust_snapshot_path().split("/")[0])
        docs_dir = path.join(hash_dir, self.rust_snapshot_path().split("/")[1], "doc")
        if not force and path.exists(docs_dir):
            print("Snapshot Rust docs already downloaded.", end=" ")
            print("Use |bootstrap-rust-docs --force| to download again.")
            return

        if path.isdir(docs_dir):
            shutil.rmtree(docs_dir)
        docs_name = self.rust_snapshot_path().replace("rustc-", "rust-docs-")
        snapshot_url = ("https://servo-rust.s3.amazonaws.com/%s.tar.gz"
                        % docs_name)
        tgz_file = path.join(hash_dir, 'doc.tar.gz')

        download_file("Rust docs", snapshot_url, tgz_file)

        print("Extracting Rust docs...")
        temp_dir = path.join(hash_dir, "temp_docs")
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
        if not force and path.exists(path.join(cargo_dir, "bin", "cargo")):
            print("Cargo already downloaded.", end=" ")
            print("Use |bootstrap-cargo --force| to download again.")
            return

        if path.isdir(cargo_dir):
            shutil.rmtree(cargo_dir)
        os.makedirs(cargo_dir)

        tgz_file = "cargo-nightly-%s.tar.gz" % host_triple()
        nightly_url = "https://static-rust-lang-org.s3.amazonaws.com/cargo-dist/%s/%s" % \
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
        content_json = re.sub(r'//.*$', '', content_decoded, flags=re.MULTILINE)

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

    @Command('update-submodules',
             description='Update submodules',
             category='bootstrap')
    def update_submodules(self):
        # Ensure that the installed git version is >= 1.8.1
        gitversion_output = subprocess.check_output(["git", "--version"])
        gitversion = LooseVersion(gitversion_output.split(" ")[-1])
        if gitversion < LooseVersion("1.8.1"):
            print("Git version 1.8.1 or above required. Current version is {}"
                  .format(gitversion))
            sys.exit(1)
        submodules = subprocess.check_output(["git", "submodule", "status"])
        for line in submodules.split('\n'):
            components = line.strip().split(' ')
            if len(components) > 1:
                module_path = components[1]
                if path.exists(module_path):
                    with cd(module_path):
                        output = subprocess.check_output(
                            ["git", "status", "--porcelain"])
                        if len(output) != 0:
                            print("error: submodule %s is not clean"
                                  % module_path)
                            print("\nClean the submodule and try again.")
                            return 1
        subprocess.check_call(
            ["git", "submodule", "--quiet", "sync", "--recursive"])
        subprocess.check_call(
            ["git", "submodule", "update", "--init", "--recursive"])

    @Command('clean-snapshots',
             description='Clean unused snapshots of Rust and Cargo',
             category='bootstrap')
    @CommandArgument('--force', '-f',
                     action='store_true',
                     help='Actually remove stuff')
    def clean_snapshots(self, force=False):
        rust_current = self.rust_snapshot_path().split('/')[0]
        cargo_current = self.cargo_build_id()
        print("Current Rust version: " + rust_current)
        print("Current Cargo version: " + cargo_current)
        removing_anything = False
        for current, base in [(rust_current, "rust"), (cargo_current, "cargo")]:
            base = path.join(self.context.sharedir, base)
            for name in os.listdir(base):
                if name != current:
                    removing_anything = True
                    name = path.join(base, name)
                    if force:
                        print("Removing " + name)
                        shutil.rmtree(name)
                    else:
                        print("Would remove " + name)
        if not removing_anything:
            print("Nothing to remove.")
        elif not force:
            print("Nothing done. "
                  "Run `./mach clean-snapshots -f` to actually remove.")

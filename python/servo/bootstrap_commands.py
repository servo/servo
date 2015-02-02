from __future__ import print_function, unicode_literals

import os
import os.path as path
import shutil
import subprocess
import sys
import tarfile
import urllib

from mach.decorators import (
    CommandArgument,
    CommandProvider,
    Command,
)

from servo.command_base import CommandBase, cd, host_triple

class PanickyUrlOpener(urllib.FancyURLopener):
  def http_error_default(self, url, fp, errcode, errmsg, headers):
    print("Download failed (%d): %s - %s" % (errcode, errmsg, url))

    cpu_type = subprocess.check_output(["uname", "-m"]).strip().lower()
    if errcode == 404 and cpu_type in ["i386", "i486", "i686", "i768", "x86"]:
        # i686
        print("Note: Servo does not currently bootstrap 32bit snapshots of Rust")
        print("See https://github.com/servo/servo/issues/3899")

    sys.exit(1)

def download(desc, src, dst):
    recved = [0]

    def report(count, bsize, fsize):
        recved[0] += bsize
        pct = recved[0] * 100.0 / fsize
        print("\rDownloading %s: %5.1f%%" % (desc, pct), end="")
        sys.stdout.flush()

    print("Downloading %s..." % desc)
    dumb = os.environ.get("TERM") == "dumb"
    PanickyUrlOpener().retrieve(src, dst, None if dumb else report)
    if not dumb:
        print()


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
        if not force and path.exists(path.join(rust_dir, "bin", "rustc")):
            print("Snapshot Rust compiler already downloaded.", end=" ")
            print("Use |bootstrap-rust --force| to download again.")
            return

        if path.isdir(rust_dir):
            shutil.rmtree(rust_dir)
        os.makedirs(rust_dir)

        snapshot_url = ("https://servo-rust.s3.amazonaws.com/%s.tar.gz"
                        % self.rust_snapshot_path())
        tgz_file = rust_dir + '.tar.gz'

        download("Rust snapshot", snapshot_url, tgz_file)

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

        download("Rust docs", snapshot_url, tgz_file)

        print("Extracting Rust docs...")
        temp_dir = path.join(hash_dir, "temp_docs")
        shutil.rmtree(temp_dir)
        extract(tgz_file, temp_dir)
        shutil.move(path.join(temp_dir, docs_name.split("/")[1], "share", "doc", "rust", "html"), docs_dir)
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
            print("Use |bootstrap_cargo --force| to download again.")
            return

        if path.isdir(cargo_dir):
            shutil.rmtree(cargo_dir)
        os.makedirs(cargo_dir)

        tgz_file = "cargo-nightly-%s.tar.gz" % host_triple()
        nightly_url = "https://static-rust-lang-org.s3.amazonaws.com/cargo-dist/%s/%s" % \
            (self.cargo_build_id(), tgz_file)

        download("Cargo nightly", nightly_url, tgz_file)

        print("Extracting Cargo nightly...")
        nightly_dir = path.join(cargo_dir,
                                path.basename(tgz_file).replace(".tar.gz", ""))
        extract(tgz_file, cargo_dir, movedir=nightly_dir)
        print("Cargo ready.")

    @Command('update-submodules',
             description='Update submodules',
             category='bootstrap')
    def update_submodules(self):
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

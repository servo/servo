# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import shutil
import subprocess
from typing import Optional

from .build_target import BuildTarget


class Base:
    def __init__(self, triple: str) -> None:
        self.environ = os.environ.copy()
        self.triple = triple
        self.is_windows = False
        self.is_linux = False
        self.is_macos = False

    def gstreamer_root(self, target: BuildTarget) -> Optional[str]:
        raise NotImplementedError("Do not know how to get GStreamer path for platform.")

    def executable_suffix(self) -> str:
        return ""

    def _platform_bootstrap(self, force: bool) -> bool:
        raise NotImplementedError("Bootstrap installation detection not yet available.")

    def _platform_bootstrap_gstreamer(self, target: BuildTarget, force: bool) -> bool:
        raise NotImplementedError("GStreamer bootstrap support is not yet available for your OS.")

    def is_gstreamer_installed(self, target: BuildTarget) -> bool:
        gstreamer_root = self.gstreamer_root(target)
        if gstreamer_root:
            pkg_config = os.path.join(gstreamer_root, "bin", "pkg-config")
        else:
            pkg_config = "pkg-config"

        try:
            return (
                subprocess.call(
                    [pkg_config, "--atleast-version=1.18", "gstreamer-1.0"],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                == 0
            )
        except FileNotFoundError:
            return False

    def bootstrap(self, force: bool, skip_platform: bool, skip_lints: bool) -> None:
        installed_something = False
        if not skip_platform:
            installed_something |= self._platform_bootstrap(force)
        self.install_rust_toolchain()
        if not skip_lints:
            installed_something |= self.install_taplo(force)
            installed_something |= self.install_cargo_deny(force)
            installed_something |= self.install_crown(force)

        if not installed_something:
            print("Dependencies were already installed!")

    def install_rust_toolchain(self) -> None:
        # rustup 1.28.0, and rustup 1.28.1+ with RUSTUP_AUTO_INSTALL=0, require us to explicitly
        # install the Rust toolchain before trying to use it.
        print(" * Installing Rust toolchain...")
        if subprocess.call(["rustup", "show", "active-toolchain"]) != 0:
            if subprocess.call(["rustup", "toolchain", "install"]) != 0:
                raise EnvironmentError("Installation of Rust toolchain failed.")

    def install_taplo(self, force: bool) -> bool:
        if not force and shutil.which("taplo") is not None:
            return False

        print(" * Installing taplo...")
        if subprocess.call(["cargo", "install", "taplo-cli", "--locked"]) != 0:
            raise EnvironmentError("Installation of taplo failed.")

        return True

    def install_cargo_deny(self, force: bool) -> bool:
        def cargo_deny_installed() -> bool:
            if force or not shutil.which("cargo-deny"):
                return False
            # Tidy needs at least version 0.18.1 installed.
            result = subprocess.run(["cargo-deny", "--version"], encoding="utf-8", capture_output=True)
            (major, minor, micro) = result.stdout.strip().split(" ")[1].split(".", 2)
            return (int(major), int(minor), int(micro)) >= (0, 18, 1)

        if cargo_deny_installed():
            return False

        print(" * Installing cargo-deny...")
        if subprocess.call(["cargo", "install", "cargo-deny", "--locked"]) != 0:
            raise EnvironmentError("Installation of cargo-deny failed.")
        return True

    def install_crown(self, force: bool) -> bool:
        print(" * Installing crown (the Servo linter)...")
        if subprocess.call(["cargo", "install", "--path", "support/crown"]) != 0:
            raise EnvironmentError("Installation of crown failed.")

        return True

    def passive_bootstrap(self) -> bool:
        """A bootstrap method that is called without explicitly invoking `./mach bootstrap`
        but that is executed in the process of other `./mach` commands. This should be
        as fast as possible."""
        return False

    def bootstrap_gstreamer(self, force: bool) -> None:
        target = BuildTarget.from_triple(self.triple)
        if not self._platform_bootstrap_gstreamer(target, force):
            root = self.gstreamer_root(target)
            if root:
                print(f"GStreamer found at: {root}")
            else:
                print("GStreamer already installed system-wide.")

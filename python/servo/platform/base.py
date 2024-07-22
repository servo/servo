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


class Base:
    def __init__(self, triple: str):
        self.environ = os.environ.copy()
        self.triple = triple
        self.is_windows = False
        self.is_linux = False
        self.is_macos = False

    def gstreamer_root(self, _cross_compilation_target: Optional[str]) -> Optional[str]:
        raise NotImplementedError("Do not know how to get GStreamer path for platform.")

    def executable_suffix(self) -> str:
        return ""

    def _platform_bootstrap(self, _force: bool) -> bool:
        raise NotImplementedError("Bootstrap installation detection not yet available.")

    def _platform_bootstrap_gstreamer(self, _force: bool) -> bool:
        raise NotImplementedError(
            "GStreamer bootstrap support is not yet available for your OS."
        )

    def is_gstreamer_installed(self, cross_compilation_target: Optional[str]) -> bool:
        gstreamer_root = self.gstreamer_root(cross_compilation_target)
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

    def bootstrap(self, force: bool, skip_platform: bool, skip_lints: bool):
        installed_something = False
        if not skip_platform:
            installed_something |= self._platform_bootstrap(force)
        if not skip_lints:
            installed_something |= self.install_taplo(force)
            installed_something |= self.install_cargo_deny(force)
            installed_something |= self.install_crown(force)

        if not installed_something:
            print("Dependencies were already installed!")

    def install_taplo(self, force: bool) -> bool:
        if not force and shutil.which("taplo") is not None:
            return False

        print(" * Installing taplo...")
        if subprocess.call(["cargo", "install", "taplo-cli", "--locked"]) != 0:
            raise EnvironmentError("Installation of taplo failed.")

        return True

    def install_cargo_deny(self, force: bool) -> bool:
        if not force and shutil.which("cargo-deny") is not None:
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

    def bootstrap_gstreamer(self, force: bool):
        if not self._platform_bootstrap_gstreamer(force):
            root = self.gstreamer_root(None)
            if root:
                print(f"GStreamer found at: {root}")
            else:
                print("GStreamer already installed system-wide.")

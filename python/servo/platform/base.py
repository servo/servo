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
from typing import Dict, Optional

from .. import util


class Base:
    def __init__(self, triple: str):
        self.environ = os.environ.copy()
        self.triple = triple
        self.is_windows = False
        self.is_linux = False
        self.is_macos = False

    def set_gstreamer_environment_variables_if_necessary(
        self, env: Dict[str, str], cross_compilation_target: Optional[str], check_installation=True
    ):
        # Environment variables are not needed when cross-compiling on any platform other
        # than Windows. GStreamer for Android is handled elsewhere.
        if cross_compilation_target and (not self.is_windows or "android" in cross_compilation_target):
            return

        # We may not need to update environment variables if GStreamer is installed
        # for the system on Linux.
        gstreamer_root = self.gstreamer_root(cross_compilation_target)
        if gstreamer_root:
            util.prepend_paths_to_env(env, "PATH", os.path.join(gstreamer_root, "bin"))
            util.prepend_paths_to_env(
                env, "PKG_CONFIG_PATH", os.path.join(gstreamer_root, "lib", "pkgconfig")
            )
            util.prepend_paths_to_env(
                env,
                self.library_path_variable_name(),
                os.path.join(gstreamer_root, "lib"),
            )
            env["GST_PLUGIN_SCANNER"] = os.path.join(
                gstreamer_root,
                "libexec",
                "gstreamer-1.0",
                f"gst-plugin-scanner{self.executable_suffix()}",
            )
            env["GST_PLUGIN_SYSTEM_PATH"] = os.path.join(gstreamer_root, "lib", "gstreamer-1.0")

        # If we are not cross-compiling GStreamer must be installed for the system. In
        # the cross-compilation case, we might be picking it up from another directory.
        if check_installation and not self.is_gstreamer_installed(cross_compilation_target):
            raise FileNotFoundError(
                "GStreamer libraries not found (>= version 1.16)."
                "Please see installation instructions in README.md"
            )

    def gstreamer_root(self, _cross_compilation_target: Optional[str]) -> Optional[str]:
        raise NotImplementedError("Do not know how to get GStreamer path for platform.")

    def library_path_variable_name(self):
        raise NotImplementedError("Do not know how to set library path for platform.")

    def linker_flag(self) -> str:
        return ""

    def executable_suffix(self) -> str:
        return ""

    def _platform_bootstrap(self, _force: bool) -> bool:
        raise NotImplementedError("Bootstrap installation detection not yet available.")

    def _platform_bootstrap_gstreamer(self, _force: bool) -> bool:
        raise NotImplementedError(
            "GStreamer bootstrap support is not yet available for your OS."
        )

    def is_gstreamer_installed(self, cross_compilation_target: Optional[str]) -> bool:
        env = os.environ.copy()
        self.set_gstreamer_environment_variables_if_necessary(
            env, cross_compilation_target, check_installation=False)
        return (
            subprocess.call(
                ["pkg-config", "--atleast-version=1.16", "gstreamer-1.0"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
            )
            == 0
        )

    def bootstrap(self, force: bool):
        installed_something = self._platform_bootstrap(force)
        installed_something |= self.install_taplo(force)
        installed_something |= self.install_crown(force)
        if not installed_something:
            print("Dependencies were already installed!")

    def install_taplo(self, force: bool) -> bool:
        if not force and shutil.which("taplo") is not None:
            return False

        if subprocess.call(["cargo", "install", "taplo-cli", "--locked"],
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE) != 0:
            raise EnvironmentError("Installation of taplo failed.")

        return True

    def install_crown(self, force: bool) -> bool:
        # We need to override the rustc set in cargo/config.toml because crown
        # may not be installed yet.
        env = dict(os.environ)
        env["CARGO_BUILD_RUSTC"] = "rustc"

        if subprocess.call(["cargo", "install", "--path", "support/crown"],
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env) != 0:
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

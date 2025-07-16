# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import subprocess
import tempfile
from typing import Optional

from .. import util
from .base import Base
from .build_target import BuildTarget

URL_BASE = "https://github.com/servo/servo-build-deps/releases/download/macOS"
GSTREAMER_PLUGIN_VERSION = "1.22.3"
GSTREAMER_URL = f"{URL_BASE}/gstreamer-1.0-1.22.3-universal.pkg"
GSTREAMER_DEVEL_URL = f"{URL_BASE}/gstreamer-1.0-devel-1.22.3-universal.pkg"
GSTREAMER_ROOT = "/Library/Frameworks/GStreamer.framework/Versions/1.0"


class MacOS(Base):
    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        self.is_macos = True

    def gstreamer_root(self, target: BuildTarget) -> Optional[str]:
        # We do not support building with gstreamer while cross-compiling on MacOS.
        if target.is_cross_build() or not os.path.exists(GSTREAMER_ROOT):
            return None
        return GSTREAMER_ROOT

    def is_gstreamer_installed(self, target: BuildTarget) -> bool:
        # Servo only supports the official GStreamer distribution on MacOS.
        return not target.is_cross_build() and os.path.exists(GSTREAMER_ROOT)

    def _platform_bootstrap(self, force: bool) -> bool:
        installed_something = False
        try:
            brewfile = os.path.join(util.SERVO_ROOT, "support", "macos", "Brewfile")
            output = subprocess.check_output(["brew", "bundle", "install", "--file", brewfile]).decode("utf-8")
            print(output)
            installed_something = "Installing" in output
        except subprocess.CalledProcessError as e:
            print("Could not run homebrew. Is it installed?")
            raise e
        target = BuildTarget.from_triple(None)
        installed_something |= self._platform_bootstrap_gstreamer(target, False)
        return installed_something

    def _platform_bootstrap_gstreamer(self, target: BuildTarget, force: bool) -> bool:
        if not force and self.is_gstreamer_installed(target):
            return False

        with tempfile.TemporaryDirectory() as temp_dir:
            libs_pkg = os.path.join(temp_dir, GSTREAMER_URL.rsplit("/", maxsplit=1)[-1])
            devel_pkg = os.path.join(temp_dir, GSTREAMER_DEVEL_URL.rsplit("/", maxsplit=1)[-1])

            util.download_file("GStreamer libraries", GSTREAMER_URL, libs_pkg)
            util.download_file("GStreamer development support", GSTREAMER_DEVEL_URL, devel_pkg)

            print("Installing GStreamer packages...")
            subprocess.check_call(
                [
                    "sudo",
                    "sh",
                    "-c",
                    f"installer -pkg '{libs_pkg}' -target / &&installer -pkg '{devel_pkg}' -target /",
                ]
            )

            assert self.is_gstreamer_installed(target)
            return True

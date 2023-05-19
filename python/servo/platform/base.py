# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import subprocess


class Base:
    def _platform_bootstrap(self, _cache_dir: str, _force: bool) -> bool:
        raise NotImplementedError("Bootstrap installation detection not yet available.")

    def _platform_bootstrap_gstreamer(self, _cache_dir: str, _force: bool) -> bool:
        raise NotImplementedError("GStreamer bootstrap support is not yet available for your OS.")

    def _platform_is_gstreamer_installed(self) -> bool:
        return subprocess.call(
            ["pkg-config", "--atleast-version=1.16", "gstreamer-1.0"],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE) == 0

    def bootstrap(self, cache_dir: str, force: bool):
        if not self._platform_bootstrap(cache_dir, force):
            print("Dependencies were already installed!")

    def bootstrap_gstreamer(self, cache_dir: str, force: bool):
        if not self._platform_bootstrap_gstreamer(cache_dir, force):
            print("Dependencies were already installed!")

    def is_gstreamer_installed(self) -> bool:
        return self._platform_is_gstreamer_installed()

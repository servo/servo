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

from .base import Base
from ..gstreamer import macos_gst_root


class MacOS(Base):
    def __init__(self):
        pass

    def _platform_is_gstreamer_installed(self) -> bool:
        # We override homebrew gstreamer if installed and always use pkgconfig
        # from official gstreamer framework.
        try:
            gst_root = macos_gst_root()
            env = os.environ.copy()
            env["PATH"] = os.path.join(gst_root, "bin")
            env["PKG_CONFIG_PATH"] = os.path.join(gst_root, "lib", "pkgconfig")
            has_gst = subprocess.call(
                ["pkg-config", "--atleast-version=1.21", "gstreamer-1.0"],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env) == 0
            gst_lib_dir = subprocess.check_output(
                ["pkg-config", "--variable=libdir", "gstreamer-1.0"], env=env)
            return has_gst and gst_lib_dir.startswith(bytes(gst_root, 'utf-8'))
        except FileNotFoundError:
            return False

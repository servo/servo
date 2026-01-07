# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import distro
from typing import Optional, Any

from .base import Base
from .build_target import BuildTarget
from .linux_standalone import linux_platform_bootstrap


class Linux(Base):
    def __init__(self, *args: str, **kwargs: Any) -> None:
        super().__init__(*args, **kwargs)
        self.is_linux = True
        self.distro = distro.name()

    def _platform_bootstrap(self, force: bool, yes: bool) -> bool:
        return linux_platform_bootstrap(self.distro, force, yes)

    def gstreamer_root(self, target: BuildTarget) -> Optional[str]:
        return None

    def _platform_bootstrap_gstreamer(self, target: BuildTarget, force: bool) -> bool:
        raise EnvironmentError(
            "Bootstrapping GStreamer on Linux is not supported. "
            + "Please install it using your distribution package manager."
        )

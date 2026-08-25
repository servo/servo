#!/usr/bin/env python3

# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

"""
This is a simple script intended to be used as a `RUSTC_WORKSPACE_WRAPPER`, adding the
required flags for code coverage instrumentation, only to the local libraries in our
workspace. This reduces the runtime overhead and the profile size significantly.
We are not interested in the code coverage metrics of outside dependencies anyway.
"""

import os
import sys


def main():
    # The first argument is the path to rustc, followed by its arguments
    args = sys.argv[1:]
    args += ["-Cinstrument-coverage=true", "--cfg=coverage"]

    # Execute rustc with the modified arguments
    os.execvp(args[0], args)


if __name__ == "__main__":
    main()

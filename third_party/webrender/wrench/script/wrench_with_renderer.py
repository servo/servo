#!/usr/bin/python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import subprocess
import sys


def is_linux():
    return sys.platform.startswith('linux')


if is_linux():
    requested_renderer = sys.argv[1]
    renderer = "default"

    if requested_renderer == 'hardware':
        pass
    elif requested_renderer == 'llvmpipe':
        os.environ["LIBGL_ALWAYS_SOFTWARE"] = "1"
        os.environ["GALLIUM_DRIVER"] = "llvmpipe"
    elif requested_renderer == 'softpipe':
        os.environ["LIBGL_ALWAYS_SOFTWARE"] = "1"
        os.environ["GALLIUM_DRIVER"] = "softpipe"
    elif requested_renderer == 'swiftshader':
        # TODO: Set up LD_LIBRARY_PATH to SwiftShader libraries automatically.
        renderer = 'es3'
    else:
        print('Unknown renderer ' + requested_renderer)
        sys.exit(1)

    cmd = [
        'cargo',
        'run',
        '--release',
        '--',
        '--no-block',               # Run as fast as possible, for benchmarking
        '--no-picture-caching',     # Disable picture caching, to test rasterization performance
        '--no-subpixel-aa',         # SwiftShader doesn't support dual source blending
        '--renderer',               # Select GL3/ES3 renderer API
        renderer,
        'load'
    ]

    cmd += sys.argv[2:]
    print('Running: ' + ' '.join(cmd))
    subprocess.check_call(cmd)
else:
    print('This script is only supported on Linux')

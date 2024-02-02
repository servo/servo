#!/bin/sh
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# The beginning of this script is both valid shell and valid python,
# such that the script starts with the shell and is reexecuted with
# the right python.
''':' && if [ ! -z "$MSYSTEM" ] ; then exec python "$0" "$@" ; else which python3 > /dev/null 2> /dev/null && exec python3 "$0" "$@" || exec python "$0" "$@" ; fi
'''

import os
import sys

if sys.version_info < (3, 10):
    print("mach does not support python < 3.10, please install python 3 >= 3.10")
    sys.exit(1)


def main(args):
    topdir = os.path.abspath(os.path.dirname(sys.argv[0]))
    sys.path.insert(0, os.path.join(topdir, "python"))
    import mach_bootstrap
    if len(sys.argv) > 1 and sys.argv[1] == "bootstrap":
        sys.exit(mach_bootstrap.bootstrap_command_only(topdir))
    else:
        mach = mach_bootstrap.bootstrap(topdir)
        sys.exit(mach.run(sys.argv[1:]))


if __name__ == '__main__':
    sys.dont_write_bytecode = True

    need_nix_shell = os.path.exists('/etc/NIXOS') or 'MACH_USE_NIX' in os.environ
    if need_nix_shell and not 'IN_NIX_SHELL' in os.environ:
        import subprocess
        from shlex import quote
        mach_dir = os.path.abspath(os.path.dirname(__file__))
        build_android_args = ['--arg', 'buildAndroid', 'true'] if 'SERVO_ANDROID_BUILD' in os.environ else []
        print('NOTE: Entering nix-shell etc/shell.nix')
        try:
            # sys argv already contains the ./mach part, so we just need to pass it as-is
            result = subprocess.run(['nix-shell', mach_dir + '/etc/shell.nix'] + build_android_args + ['--run', ' '.join(map(quote, sys.argv))])
            sys.exit(result.returncode)
        except KeyboardInterrupt:
            sys.exit(0)
    else:
        main(sys.argv)

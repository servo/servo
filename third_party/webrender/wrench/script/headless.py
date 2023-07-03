#!/usr/bin/python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Build and run wrench with off-screen software rendering (OSMesa/LLVMpipe)
# for platform-independent results. This is good for running reference tests.
#
# Usage: headless.py ARGS
#
# Pass ARGS through to wrench, after '--headless' and '--no-scissor'.
#
# Environment variables:
#
# WRENCH_HEADLESS_TARGET: If set, don't rebuild wrench. Instead, the value should
#     be the path to an already-built cargo 'target' directory. This is useful
#     for running a cross-compiled wrench.
#
# CARGOFLAGS: Extra flags to be passed to 'cargo build'. Split on whitespace.
#
# OPTIMIZED: This script uses the release build by default, but if this variable
#     is set to '0' or 'false', the script uses the debug build.
#
# DEBUGGER: If set, run wrench under a debugger. Permitted values are 'rr' (run
#     under 'rr record'), or 'gdb', 'rust-gdb', or 'cgdb' (run under the given
#     debugger, and arrange to supply ARGS to the wrench debuggee, not the
#     debugger)

from __future__ import print_function
import contextlib
import os
import subprocess
import sys
from os import path
from glob import glob


@contextlib.contextmanager
def cd(new_path):
    """Context manager for changing the current working directory"""
    previous_path = os.getcwd()
    try:
        os.chdir(new_path)
        yield
    finally:
        os.chdir(previous_path)


def find_dep_path_newest(package, bin_path):
    deps_path = path.join(path.split(bin_path)[0], "build")
    with cd(deps_path):
        candidates = glob(package + '-*')
    candidates = (path.join(deps_path, c) for c in candidates)
    """ For some reason cargo can create an extra osmesa-src without libs """
    candidates = (c for c in candidates if path.exists(path.join(c, 'out')))
    candidate_times = sorted(((path.getmtime(c), c) for c in candidates), reverse=True)
    if len(candidate_times) > 0:
        return candidate_times[0][1]
    return None


def is_windows():
    """ Detect windows, mingw, cygwin """
    return sys.platform == 'win32' or sys.platform == 'msys' or sys.platform == 'cygwin'


def is_macos():
    return sys.platform == 'darwin'


def is_linux():
    return sys.platform.startswith('linux')


def debugger():
    if "DEBUGGER" in os.environ:
        return os.environ["DEBUGGER"]
    return None


def use_gdb():
    return debugger() in ['gdb', 'cgdb', 'rust-gdb']


def use_rr():
    return debugger() == 'rr'


def optimized_build():
    if "OPTIMIZED" in os.environ:
        opt = os.environ["OPTIMIZED"]
        return opt not in ["0", "false"]
    return True


def set_osmesa_env(bin_path):
    """Set proper LD_LIBRARY_PATH and DRIVE for software rendering on Linux and OSX"""
    if is_linux():
        osmesa_path = path.join(find_dep_path_newest('osmesa-src', bin_path), "out", "lib", "gallium")
        print(osmesa_path)
        os.environ["LD_LIBRARY_PATH"] = osmesa_path
        os.environ["GALLIUM_DRIVER"] = "softpipe"
    elif is_macos():
        osmesa_path = path.join(find_dep_path_newest('osmesa-src', bin_path),
                                "out", "src", "gallium", "targets", "osmesa", ".libs")
        glapi_path = path.join(find_dep_path_newest('osmesa-src', bin_path),
                               "out", "src", "mapi", "shared-glapi", ".libs")
        os.environ["DYLD_LIBRARY_PATH"] = osmesa_path + ":" + glapi_path
        os.environ["GALLIUM_DRIVER"] = "softpipe"


extra_flags = os.getenv('CARGOFLAGS', None)
extra_flags = extra_flags.split(' ') if extra_flags else []

wrench_headless_target = os.getenv('WRENCH_HEADLESS_TARGET', None)

if wrench_headless_target:
    target_folder = wrench_headless_target
else:
    target_folder = '../target/'

if optimized_build():
    target_folder += 'release/'
else:
    target_folder += 'debug/'

# For CI purposes, don't build if WRENCH_HEADLESS_TARGET is set.
# This environment variable is used to point to the location of a cross-compiled
# wrench for the CI on some platforms.
if not wrench_headless_target:
    build_cmd = ['cargo', 'build'] + extra_flags + ['--verbose', '--features', 'headless']
    if optimized_build():
        build_cmd += ['--release']
    subprocess.check_call(build_cmd)

dbg_cmd = []
if use_rr():
    dbg_cmd = ['rr', 'record']
elif use_gdb():
    dbg_cmd = [debugger(), '--args']
elif debugger():
    print("Unknown debugger: " + debugger())
    sys.exit(1)

set_osmesa_env(target_folder)
# TODO(gw): We have an occasional accuracy issue or bug (could be WR or OSMesa)
#           where the output of a previous test that uses intermediate targets can
#           cause 1.0 / 255.0 pixel differences in a subsequent test. For now, we
#           run tests with no-scissor mode, which ensures a complete target clear
#           between test runs. But we should investigate this further...
cmd = dbg_cmd + [target_folder + 'wrench', '--no-scissor', '-h'] + sys.argv[1:]
print('Running: `' + ' '.join(cmd) + '`')
subprocess.check_call(cmd)

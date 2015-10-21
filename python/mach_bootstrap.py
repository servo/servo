# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function, unicode_literals

import os
import platform
import subprocess
import sys
from distutils.spawn import find_executable

SEARCH_PATHS = [
    os.path.join("python", "mach"),
    os.path.join("tests", "wpt"),
    os.path.join("tests", "wpt", "harness"),
]

# Individual files providing mach commands.
MACH_MODULES = [
    os.path.join('python', 'servo', 'bootstrap_commands.py'),
    os.path.join('python', 'servo', 'build_commands.py'),
    os.path.join('python', 'servo', 'testing_commands.py'),
    os.path.join('python', 'servo', 'post_build_commands.py'),
    os.path.join('python', 'servo', 'devenv_commands.py'),
]


CATEGORIES = {
    'bootstrap': {
        'short': 'Bootstrap Commands',
        'long': 'Bootstrap the build system',
        'priority': 90,
    },
    'build': {
        'short': 'Build Commands',
        'long': 'Interact with the build system',
        'priority': 80,
    },
    'post-build': {
        'short': 'Post-build Commands',
        'long': 'Common actions performed after completing a build.',
        'priority': 70,
    },
    'testing': {
        'short': 'Testing',
        'long': 'Run tests.',
        'priority': 60,
    },
    'devenv': {
        'short': 'Development Environment',
        'long': 'Set up and configure your development environment.',
        'priority': 50,
    },
    'build-dev': {
        'short': 'Low-level Build System Interaction',
        'long': 'Interact with specific parts of the build system.',
        'priority': 20,
    },
    'misc': {
        'short': 'Potpourri',
        'long': 'Potent potables and assorted snacks.',
        'priority': 10,
    },
    'disabled': {
        'short': 'Disabled',
        'long': 'The disabled commands are hidden by default. Use -v to display them. These commands are unavailable '
                'for your current context, run "mach <command>" to see why.',
        'priority': 0,
    }
}


def _get_exec(*names):
    for name in names:
        path = find_executable(name)
        if path is not None:
            return path
    return None


# Possible names of executables, sorted from most to least specific
PYTHON_NAMES = ["python-2.7", "python2.7", "python2", "python"]
VIRTUALENV_NAMES = ["virtualenv-2.7", "virtualenv2.7", "virtualenv2", "virtualenv"]
PIP_NAMES = ["pip-2.7", "pip2.7", "pip2", "pip"]


def _activate_virtualenv(topdir):
    virtualenv_path = os.path.join(topdir, "python", "_virtualenv")
    python = _get_exec(*PYTHON_NAMES)
    if python is None:
        sys.exit("Python is not installed. Please install it prior to running mach.")

    if not os.path.exists(virtualenv_path):
        virtualenv = _get_exec(*VIRTUALENV_NAMES)
        if virtualenv is None:
            sys.exit("Python virtualenv is not installed. Please install it prior to running mach.")

        try:
            subprocess.check_call([virtualenv, "-p", python, virtualenv_path])
        except (subprocess.CalledProcessError, OSError):
            sys.exit("Python virtualenv failed to execute properly.")

    activate_path = os.path.join(virtualenv_path, "bin", "activate_this.py")
    execfile(activate_path, dict(__file__=activate_path))

    # TODO: Right now, we iteratively install all the requirements by invoking
    # `pip install` each time. If it were the case that there were conflicting
    # requirements, we wouldn't know about them. Once
    # https://github.com/pypa/pip/issues/988 is addressed, then we can just
    # chain each of the requirements files into the same `pip install` call
    # and it will check for conflicts.
    requirements_paths = [
        os.path.join("python", "requirements.txt"),
        os.path.join("tests", "wpt", "harness", "requirements.txt"),
        os.path.join("tests", "wpt", "harness", "requirements_servo.txt"),
    ]
    for req_rel_path in requirements_paths:
        req_path = os.path.join(topdir, req_rel_path)
        marker_file = req_rel_path.replace(os.path.sep, '-')
        marker_path = os.path.join(virtualenv_path, marker_file)
        try:
            if os.path.getmtime(req_path) + 10 < os.path.getmtime(marker_path):
                continue
        except OSError:
            open(marker_path, 'w').close()

        pip = _get_exec(*PIP_NAMES)
        if pip is None:
            sys.exit("Python pip is not installed. Please install it prior to running mach.")

        try:
            subprocess.check_call([pip, "install", "-q", "-r", req_path])
        except (subprocess.CalledProcessError, OSError):
            sys.exit("Pip failed to execute properly.")

        os.utime(marker_path, None)


def bootstrap(topdir):
    topdir = os.path.abspath(topdir)

    # Ensure we are running Python 2.7+. We put this check here so we generate a
    # user-friendly error message rather than a cryptic stack trace on module
    # import.
    if not (3, 0) > sys.version_info >= (2, 7):
        print('Python 2.7 or above (but not Python 3) is required to run mach.')
        print('You are running Python', platform.python_version())
        sys.exit(1)

    _activate_virtualenv(topdir)

    def populate_context(context, key=None):
        if key is None:
            return
        if key == 'topdir':
            return topdir
        raise AttributeError(key)

    sys.path[0:0] = [os.path.join(topdir, path) for path in SEARCH_PATHS]
    import mach.main
    mach = mach.main.Mach(os.getcwd())
    mach.populate_context_handler = populate_context

    for category, meta in CATEGORIES.items():
        mach.define_category(category, meta['short'], meta['long'],
                             meta['priority'])

    for path in MACH_MODULES:
        mach.load_commands_from_file(os.path.join(topdir, path))

    return mach

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function, unicode_literals

import os
import platform
import sys
from distutils.spawn import find_executable
from subprocess import PIPE, Popen

SEARCH_PATHS = [
    os.path.join("python", "tidy"),
    os.path.join("tests", "wpt"),
    os.path.join("tests", "wpt", "harness"),
]

# Individual files providing mach commands.
MACH_MODULES = [
    os.path.join('python', 'servo', 'bootstrap_commands.py'),
    os.path.join('python', 'servo', 'build_commands.py'),
    os.path.join('python', 'servo', 'testing_commands.py'),
    os.path.join('python', 'servo', 'post_build_commands.py'),
    os.path.join('python', 'servo', 'package_commands.py'),
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
    'package': {
        'short': 'Package',
        'long': 'Create objects to distribute',
        'priority': 15,
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

# Possible names of executables
# NOTE: Windows Python doesn't provide versioned executables, so we must use
# the plain names. On MSYS, we still use Windows Python.
if sys.platform in ['msys', 'win32']:
    PYTHON_NAMES = ["python"]
    VIRTUALENV_NAMES = ["virtualenv"]
    PIP_NAMES = ["pip"]
else:
    PYTHON_NAMES = ["python-2.7", "python2.7", "python2", "python"]
    VIRTUALENV_NAMES = ["virtualenv-2.7", "virtualenv2.7", "virtualenv2", "virtualenv"]
    PIP_NAMES = ["pip-2.7", "pip2.7", "pip2", "pip"]


def _get_exec_path(names, is_valid_path=lambda _path: True):
    for name in names:
        path = find_executable(name)
        if path and is_valid_path(path):
            return path
    return None


def _get_virtualenv_script_dir():
    # Virtualenv calls its scripts folder "bin" on linux/OSX/MSYS64 but "Scripts" on Windows
    if os.name == "nt" and os.sep != "/":
        return "Scripts"
    return "bin"


def _activate_virtualenv(topdir):
    virtualenv_path = os.path.join(topdir, "python", "_virtualenv")
    check_exec_path = lambda path: path.startswith(virtualenv_path)
    python = _get_exec_path(PYTHON_NAMES)   # If there was no python, mach wouldn't have run at all!
    if not python:
        sys.exit('Failed to find python executable for starting virtualenv.')

    script_dir = _get_virtualenv_script_dir()
    activate_path = os.path.join(virtualenv_path, script_dir, "activate_this.py")
    if not (os.path.exists(virtualenv_path) and os.path.exists(activate_path)):
        virtualenv = _get_exec_path(VIRTUALENV_NAMES)
        if not virtualenv:
            sys.exit("Python virtualenv is not installed. Please install it prior to running mach.")

        process = Popen([virtualenv, "-p", python, virtualenv_path], stdout=PIPE, stderr=PIPE)
        process.wait()
        if process.returncode:
            out, err = process.communicate()
            print('Python virtualenv failed to execute properly:')
            sys.exit('Output: %s\nError: %s' % (out, err))

    execfile(activate_path, dict(__file__=activate_path))

    python = _get_exec_path(PYTHON_NAMES, is_valid_path=check_exec_path)
    if not python:
        sys.exit("Python executable in virtualenv failed to activate.")

    # TODO: Right now, we iteratively install all the requirements by invoking
    # `pip install` each time. If it were the case that there were conflicting
    # requirements, we wouldn't know about them. Once
    # https://github.com/pypa/pip/issues/988 is addressed, then we can just
    # chain each of the requirements files into the same `pip install` call
    # and it will check for conflicts.
    requirements_paths = [
        os.path.join("python", "requirements.txt"),
        os.path.join("tests", "wpt", "harness", "requirements.txt"),
        os.path.join("tests", "wpt", "harness", "requirements_firefox.txt"),
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
            pass

        pip = _get_exec_path(PIP_NAMES, is_valid_path=check_exec_path)
        if not pip:
            sys.exit("Python pip is either not installed or not found in virtualenv.")

        process = Popen([pip, "install", "-q", "-r", req_path], stdout=PIPE, stderr=PIPE)
        process.wait()
        if process.returncode:
            out, err = process.communicate()
            print('Pip failed to execute properly:')
            sys.exit('Output: %s\nError: %s' % (out, err))

        open(marker_path, 'w').close()


def _ensure_case_insensitive_if_windows():
    # The folder is called 'python'. By deliberately checking for it with the wrong case, we determine if the file
    # system is case sensitive or not.
    if _is_windows() and not os.path.exists('Python'):
        print('Cannot run mach in a path on a case-sensitive file system on Windows.')
        print('For more details, see https://github.com/pypa/virtualenv/issues/935')
        sys.exit(1)


def _is_windows():
    return sys.platform == 'win32' or sys.platform == 'msys'


def bootstrap(topdir):
    _ensure_case_insensitive_if_windows()

    topdir = os.path.abspath(topdir)

    # We don't support paths with Unicode characters for now
    # https://github.com/servo/servo/issues/10002
    try:
        topdir.decode('ascii')
    except UnicodeDecodeError:
        print('Cannot run mach in a path with Unicode characters.')
        print('Current path:', topdir)
        sys.exit(1)

    # We don't support paths with spaces for now
    # https://github.com/servo/servo/issues/9442
    if ' ' in topdir:
        print('Cannot run mach in a path with spaces.')
        print('Current path:', topdir)
        sys.exit(1)

    # We don't support MinGW Python
    if os.path.join(os.sep, 'mingw64', 'bin') in sys.executable:
        print('Cannot run mach with MinGW or MSYS Python.')
        print('\nPlease add the path to Windows Python (usually /c/Python27) to your path.')
        print('You can do this by appending the line:')
        print('    export PATH=/c/Python27:$PATH')
        print('to your ~/.profile.')
        sys.exit(1)

    # Ensure we are running Python 2.7+. We put this check here so we generate a
    # user-friendly error message rather than a cryptic stack trace on module import.
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
        mach.define_category(category, meta['short'], meta['long'], meta['priority'])

    for path in MACH_MODULES:
        mach.load_commands_from_file(os.path.join(topdir, path))

    return mach

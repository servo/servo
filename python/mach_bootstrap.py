# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

from __future__ import print_function, unicode_literals

import os
import platform
import sys
from distutils.spawn import find_executable
from subprocess import Popen
import shutil
from tempfile import TemporaryFile

SEARCH_PATHS = [
    os.path.join("python", "tidy"),
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
PYTHON_NAMES = ["python-2.7", "python2.7", "python2", "python"]


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


def _process_exec(args):
    with TemporaryFile() as out:
        with TemporaryFile() as err:
            process = Popen(args, stdout=out, stderr=err)
            process.wait()
            if process.returncode:
                print('"%s" failed with error code %d:' % ('" "'.join(args), process.returncode))

                if sys.version_info >= (3, 0):
                    stdout = sys.stdout.buffer
                else:
                    stdout = sys.stdout

                print('Output:')
                out.seek(0)
                stdout.flush()
                shutil.copyfileobj(out, stdout)
                stdout.flush()

                print('Error:')
                err.seek(0)
                stdout.flush()
                shutil.copyfileobj(err, stdout)
                stdout.flush()

                sys.exit(1)


def wpt_path(is_firefox, topdir, *paths):
    if is_firefox:
        rel = os.path.join("..", "testing", "web-platform")
    else:
        rel = os.path.join("tests", "wpt")

    return os.path.join(topdir, rel, *paths)


def wptrunner_path(is_firefox, topdir, *paths):
    wpt_root = wpt_path(is_firefox, topdir)
    if is_firefox:
        rel = os.path.join(wpt_root, "tests", "tools", "wptrunner")
    else:
        rel = os.path.join(wpt_root, "web-platform-tests", "tools", "wptrunner")

    return os.path.join(topdir, rel, *paths)


def wptserve_path(is_firefox, topdir, *paths):
    wpt_root = wpt_path(is_firefox, topdir)
    if is_firefox:
        rel = os.path.join(wpt_root, "tests", "tools", "wptserve")
    else:
        rel = os.path.join(wpt_root, "web-platform-tests", "tools", "wptserve")

    return os.path.join(topdir, rel, *paths)


def _activate_virtualenv(topdir, is_firefox):
    virtualenv_path = os.path.join(topdir, "python", "_virtualenv%d.%d" % (sys.version_info[0], sys.version_info[1]))
    python = sys.executable   # If there was no python, mach wouldn't have run at all!
    if not python:
        sys.exit('Failed to find python executable for starting virtualenv.')

    script_dir = _get_virtualenv_script_dir()
    activate_path = os.path.join(virtualenv_path, script_dir, "activate_this.py")
    need_pip_upgrade = False
    if not (os.path.exists(virtualenv_path) and os.path.exists(activate_path)):
        import imp
        try:
            imp.find_module('virtualenv')
        except ImportError:
            sys.exit("Python virtualenv is not installed. Please install it prior to running mach.")

        _process_exec([python, "-m", "virtualenv", "-p", python, "--system-site-packages", virtualenv_path])

        # We want to upgrade pip when virtualenv created for the first time
        need_pip_upgrade = True

    exec(compile(open(activate_path).read(), activate_path, 'exec'), dict(__file__=activate_path))

    python = _get_exec_path(PYTHON_NAMES,
                            is_valid_path=lambda path: path.startswith(virtualenv_path))
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
        wptrunner_path(is_firefox, topdir, "requirements.txt",),
        wptrunner_path(is_firefox, topdir, "requirements_firefox.txt"),
        wptrunner_path(is_firefox, topdir, "requirements_servo.txt"),
    ]

    if need_pip_upgrade:
        # Upgrade pip when virtualenv is created to fix the issue
        # https://github.com/servo/servo/issues/11074
        _process_exec([python, "-m", "pip", "install", "-I", "-U", "pip"])

    for req_rel_path in requirements_paths:
        req_path = os.path.join(topdir, req_rel_path)
        marker_file = req_rel_path.replace(os.path.sep, '-')
        marker_path = os.path.join(virtualenv_path, marker_file)

        try:
            if os.path.getmtime(req_path) + 10 < os.path.getmtime(marker_path):
                continue
        except OSError:
            pass

        _process_exec([python, "-m", "pip", "install", "-I", "-r", req_path])

        open(marker_path, 'w').close()


def _ensure_case_insensitive_if_windows():
    # The folder is called 'python'. By deliberately checking for it with the wrong case, we determine if the file
    # system is case sensitive or not.
    if _is_windows() and not os.path.exists('Python'):
        print('Cannot run mach in a path on a case-sensitive file system on Windows.')
        print('For more details, see https://github.com/pypa/virtualenv/issues/935')
        sys.exit(1)


def _is_windows():
    return sys.platform == 'win32'


class DummyContext(object):
    pass


def is_firefox_checkout(topdir):
    parentdir = os.path.normpath(os.path.join(topdir, '..'))
    is_firefox = os.path.isfile(os.path.join(parentdir,
                                             'build/mach_bootstrap.py'))
    return is_firefox


def bootstrap_command_only(topdir):
    # we should activate the venv before importing servo.boostrap
    # because the module requires non-standard python packages
    _activate_virtualenv(topdir, is_firefox_checkout(topdir))

    from servo.bootstrap import bootstrap

    context = DummyContext()
    context.topdir = topdir
    force = False
    if len(sys.argv) == 3 and sys.argv[2] == "-f":
        force = True
    bootstrap(context, force)
    return 0


def bootstrap(topdir):
    _ensure_case_insensitive_if_windows()

    topdir = os.path.abspath(topdir)

    len(sys.argv) > 1 and sys.argv[1] == "bootstrap"

    # We don't support paths with Unicode characters for now
    # https://github.com/servo/servo/issues/10002
    try:
        # Trick to support both python2 and python3
        topdir.encode().decode('ascii')
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

    # Ensure we are running Python 3.5+. We put this check here so we generate a
    # user-friendly error message rather than a cryptic stack trace on module import.
    if sys.version_info < (3, 5):
        print('Python3 (>=3.5) is required to run mach.')
        print('You are running Python', platform.python_version())
        sys.exit(1)

    is_firefox = is_firefox_checkout(topdir)

    _activate_virtualenv(topdir, is_firefox)

    def populate_context(context, key=None):
        if key is None:
            return
        if key == 'topdir':
            return topdir
        raise AttributeError(key)

    sys.path[0:0] = [os.path.join(topdir, path) for path in SEARCH_PATHS]

    sys.path[0:0] = [wpt_path(is_firefox, topdir),
                     wptrunner_path(is_firefox, topdir),
                     wptserve_path(is_firefox, topdir)]

    import mach.main
    mach = mach.main.Mach(os.getcwd())
    mach.populate_context_handler = populate_context

    for category, meta in CATEGORIES.items():
        mach.define_category(category, meta['short'], meta['long'], meta['priority'])

    for path in MACH_MODULES:
        # explicitly provide a module name
        # workaround for https://bugzilla.mozilla.org/show_bug.cgi?id=1549636
        file = os.path.basename(path)
        module_name = os.path.splitext(file)[0]
        mach.load_commands_from_file(os.path.join(topdir, path), module_name)

    return mach

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import hashlib
import os
import platform
import site
import subprocess
import sys

SCRIPT_PATH = os.path.abspath(os.path.dirname(__file__))
TOP_DIR = os.path.abspath(os.path.join(SCRIPT_PATH, ".."))
WPT_PATH = os.path.join(TOP_DIR, "tests", "wpt")
WPT_TOOLS_PATH = os.path.join(WPT_PATH, "tests", "tools")
WPT_RUNNER_PATH = os.path.join(WPT_TOOLS_PATH, "wptrunner")
WPT_SERVE_PATH = os.path.join(WPT_TOOLS_PATH, "wptserve")

SEARCH_PATHS = [
    os.path.join("python", "mach"),
    os.path.join("third_party", "mozdebug"),
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


# venv calls its scripts folder "bin" on non-Windows and "Scripts" on Windows.
def _get_virtualenv_script_dir():
    if os.name == "nt" and os.sep != "/":
        return "Scripts"
    return "bin"


# venv names its lib folder something like "lib/python3.11/site-packages" on
# non-Windows and "Lib\site-packages" on Window.
def _get_virtualenv_lib_dir():
    if os.name == "nt" and os.sep != "/":
        return os.path.join("Lib", "site-packages")
    return os.path.join(
        "lib",
        f"python{sys.version_info[0]}.{sys.version_info[1]}",
        "site-packages"
    )


def _process_exec(args):
    try:
        subprocess.check_output(args, stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as exception:
        print(exception.output.decode(sys.stdout.encoding))
        print(f"Process failed with return code: {exception.returncode}")
        sys.exit(1)


def install_virtual_env_requirements(project_path: str, python: str, virtualenv_path: str):
    requirements_paths = [
        os.path.join(project_path, "python", "requirements.txt"),
        os.path.join(project_path, WPT_TOOLS_PATH, "requirements_tests.txt",),
        os.path.join(project_path, WPT_RUNNER_PATH, "requirements.txt",),
    ]

    requirements_hasher = hashlib.sha256()
    for path in requirements_paths:
        with open(path, 'rb') as file:
            requirements_hasher.update(file.read())

    try:
        marker_path = os.path.join(virtualenv_path, "requirements.sha256")
        with open(marker_path, 'r') as marker_file:
            marker_hash = marker_file.read()
    except FileNotFoundError:
        marker_hash = None

    requirements_hash = requirements_hasher.hexdigest()

    if marker_hash != requirements_hash:
        print(" * Upgrading pip...")
        _process_exec([python, "-m", "pip", "install", "--upgrade", "pip"])

        print(" * Installing Python requirements...")
        _process_exec([python, "-m", "pip", "install", "-I",
                       "-r", requirements_paths[0],
                       "-r", requirements_paths[1],
                       "-r", requirements_paths[2]])
        with open(marker_path, "w") as marker_file:
            marker_file.write(requirements_hash)


def _activate_virtualenv(topdir):
    virtualenv_path = os.path.join(topdir, "python", "_venv%d.%d" % (sys.version_info[0], sys.version_info[1]))
    python = sys.executable

    if os.environ.get("VIRTUAL_ENV") != virtualenv_path:
        venv_script_path = os.path.join(virtualenv_path, _get_virtualenv_script_dir())
        if not os.path.exists(virtualenv_path):
            print(" * Setting up virtual environment...")
            _process_exec([python, "-m", "venv", "--system-site-packages", virtualenv_path])

        # This general approach is taken from virtualenv's `activate_this.py`.
        os.environ["PATH"] = os.pathsep.join([venv_script_path, *os.environ.get("PATH", "").split(os.pathsep)])
        os.environ["VIRTUAL_ENV"] = virtualenv_path

        prev_length = len(sys.path)
        lib_path = os.path.realpath(os.path.join(virtualenv_path, _get_virtualenv_lib_dir()))
        site.addsitedir(lib_path)
        sys.path[:] = sys.path[prev_length:] + sys.path[0:prev_length]

        sys.real_prefix = sys.prefix
        sys.prefix = virtualenv_path

        # Use the python in our venv for subprocesses, not the python we were originally run with.
        # Otherwise pip may still try to write to the wrong site-packages directory.
        python = os.path.join(venv_script_path, "python")

    install_virtual_env_requirements(topdir, python, virtualenv_path)


def _ensure_case_insensitive_if_windows():
    # The folder is called 'python'. By deliberately checking for it with the wrong case, we determine if the file
    # system is case sensitive or not.
    if _is_windows() and not os.path.exists('Python'):
        print('Cannot run mach in a path on a case-sensitive file system on Windows.')
        print('For more details, see https://github.com/pypa/virtualenv/issues/935')
        sys.exit(1)


def _is_windows():
    return sys.platform == 'win32'


def bootstrap_command_only(topdir):
    # we should activate the venv before importing servo.boostrap
    # because the module requires non-standard python packages
    _activate_virtualenv(topdir)

    # We cannot import these modules until the virtual environment
    # is active because they depend on modules installed via the
    # virtual environment.
    # pylint: disable=import-outside-toplevel
    import servo.platform
    import servo.util

    try:
        force = '-f' in sys.argv or '--force' in sys.argv
        skip_platform = '--skip-platform' in sys.argv
        skip_lints = '--skip-lints' in sys.argv
        servo.platform.get().bootstrap(force, skip_platform, skip_lints)
    except NotImplementedError as exception:
        print(exception)
        return 1

    return 0


def bootstrap(topdir):
    _ensure_case_insensitive_if_windows()

    topdir = os.path.abspath(topdir)

    # We don't support paths with spaces for now
    # https://github.com/servo/servo/issues/9442
    if ' ' in topdir:
        print('Cannot run mach in a path with spaces.')
        print('Current path:', topdir)
        sys.exit(1)

    # Ensure we are running Python 3.10+. We put this check here so we generate a
    # user-friendly error message rather than a cryptic stack trace on module import.
    if sys.version_info < (3, 10):
        print('Python3 (>=3.10) is required to run mach.')
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
    sys.path[0:0] = [WPT_PATH, WPT_RUNNER_PATH, WPT_SERVE_PATH]

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

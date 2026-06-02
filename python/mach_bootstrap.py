# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys
from typing import TYPE_CHECKING

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
    os.path.join("python", "servo", "bootstrap_commands.py"),
    os.path.join("python", "servo", "build_commands.py"),
    os.path.join("python", "servo", "testing_commands.py"),
    os.path.join("python", "servo", "post_build_commands.py"),
    os.path.join("python", "servo", "package_commands.py"),
    os.path.join("python", "servo", "devenv_commands.py"),
]

CATEGORIES = {
    "bootstrap": {
        "short": "Bootstrap Commands",
        "long": "Bootstrap the build system",
        "priority": 90,
    },
    "build": {
        "short": "Build Commands",
        "long": "Interact with the build system",
        "priority": 80,
    },
    "post-build": {
        "short": "Post-build Commands",
        "long": "Common actions performed after completing a build.",
        "priority": 70,
    },
    "testing": {
        "short": "Testing",
        "long": "Run tests.",
        "priority": 60,
    },
    "devenv": {
        "short": "Development Environment",
        "long": "Set up and configure your development environment.",
        "priority": 50,
    },
    "build-dev": {
        "short": "Low-level Build System Interaction",
        "long": "Interact with specific parts of the build system.",
        "priority": 20,
    },
    "package": {
        "short": "Package",
        "long": "Create objects to distribute",
        "priority": 15,
    },
    "misc": {
        "short": "Potpourri",
        "long": "Potent potables and assorted snacks.",
        "priority": 10,
    },
    "disabled": {
        "short": "Disabled",
        "long": "The disabled commands are hidden by default. Use -v to display them. These commands are unavailable "
        'for your current context, run "mach <command>" to see why.',
        "priority": 0,
    },
}


if TYPE_CHECKING:
    from mach.main import Mach


def filter_warnings() -> None:
    # Turn off warnings about deprecated syntax in our indirect dependencies.
    # TODO: Find a better approach for doing this.
    import warnings

    warnings.filterwarnings("ignore", category=SyntaxWarning, module=r".*.venv")


def _ensure_case_insensitive_if_windows() -> None:
    # The folder is called 'python'. By deliberately checking for it with the wrong case, we determine if the file
    # system is case sensitive or not.
    if _is_windows() and not os.path.exists("Python"):
        print("Cannot run mach in a path on a case-sensitive file system on Windows.")
        print("For more details, see https://github.com/pypa/virtualenv/issues/935")
        sys.exit(1)


def _is_windows() -> bool:
    return sys.platform == "win32"


def bootstrap_command_only(topdir: str) -> int:
    # We cannot import these modules until the virtual environment
    # is active because they depend on modules installed via the
    # virtual environment.
    # pylint: disable=import-outside-toplevel
    import servo.platform
    import servo.util

    try:
        force = "-f" in sys.argv or "--force" in sys.argv
        yes = "--yes" in sys.argv or "-y" in sys.argv
        skip_platform = "--skip-platform" in sys.argv
        skip_lints = "--skip-lints" in sys.argv
        skip_nextest = "--skip-nextest" in sys.argv
        servo.platform.get().bootstrap(force, yes, skip_platform, skip_lints, skip_nextest)
    except NotImplementedError as exception:
        print(exception)
        return 1

    return 0


def bootstrap(topdir: str) -> "Mach":
    _ensure_case_insensitive_if_windows()

    topdir = os.path.abspath(topdir)

    # We don't support paths with spaces for now
    # https://github.com/servo/servo/issues/9616
    if " " in topdir and (not _is_windows()):
        print("Cannot run mach in a path with spaces.")
        print("Current path:", topdir)
        sys.exit(1)

    filter_warnings()

    def populate_context(context: None, key: None | str = None) -> str | None:
        if key is None:
            return
        if key == "topdir":
            return topdir
        raise AttributeError(key)

    sys.path[0:0] = [os.path.join(topdir, path) for path in SEARCH_PATHS]
    sys.path[0:0] = [WPT_PATH, WPT_RUNNER_PATH, WPT_SERVE_PATH]

    import mach.main

    mach = mach.main.Mach(os.getcwd())
    # pyrefly: ignore[bad-assignment]
    mach.populate_context_handler = populate_context

    for category, meta in CATEGORIES.items():
        mach.define_category(category, meta["short"], meta["long"], meta["priority"])

    for path in MACH_MODULES:
        # explicitly provide a module name
        # workaround for https://bugzilla.mozilla.org/show_bug.cgi?id=1549636
        file = os.path.basename(path)
        module_name = os.path.splitext(file)[0]
        mach.load_commands_from_file(os.path.join(topdir, path), module_name)

    return mach

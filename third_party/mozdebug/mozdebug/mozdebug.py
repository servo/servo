#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, print_function

import json
import os
import mozinfo
import shutil
import sys
from collections import namedtuple
from subprocess import check_output

__all__ = [
    "get_debugger_info",
    "get_default_debugger_name",
    "DebuggerSearch",
    "get_default_valgrind_args",
    "DebuggerInfo",
]

"""
Map of debugging programs to information about them, like default arguments
and whether or not they are interactive.

To add support for a new debugger, simply add the relative entry in
_DEBUGGER_INFO and optionally update the _DEBUGGER_PRIORITIES.
"""
_DEBUGGER_INFO = {
    # gdb requires that you supply the '--args' flag in order to pass arguments
    # after the executable name to the executable.
    "gdb": {"interactive": True, "args": ["-q", "--args"]},
    "cgdb": {"interactive": True, "args": ["-q", "--args"]},
    "rust-gdb": {"interactive": True, "args": ["-q", "--args"]},
    "lldb": {"interactive": True, "args": ["--"], "requiresEscapedArgs": True},
    "rust-lldb": {"interactive": True, "args": ["--"], "requiresEscapedArgs": True},
    # Visual Studio Debugger Support.
    "devenv.exe": {"interactive": True, "args": ["-debugexe"]},
    # Visual C++ Express Debugger Support.
    "wdexpress.exe": {"interactive": True, "args": ["-debugexe"]},
    # Windows Development Kit super-debugger.
    "windbg.exe": {
        "interactive": True,
    },
}

# Maps each OS platform to the preferred debugger programs found in _DEBUGGER_INFO.
_DEBUGGER_PRIORITIES = {
    "win": ["devenv.exe", "wdexpress.exe"],
    "linux": ["gdb", "cgdb", "lldb"],
    "mac": ["lldb", "gdb"],
    "android": ["lldb"],
    "unknown": ["gdb"],
}


DebuggerInfo = namedtuple(
    "DebuggerInfo", ["path", "interactive", "args", "requiresEscapedArgs"]
)


def _windbg_installation_paths():
    programFilesSuffixes = ["", " (x86)"]
    programFiles = "C:/Program Files"
    # Try the most recent versions first.
    windowsKitsVersions = ["10", "8.1", "8"]

    for suffix in programFilesSuffixes:
        windowsKitsPrefix = os.path.join(programFiles + suffix, "Windows Kits")
        for version in windowsKitsVersions:
            yield os.path.join(
                windowsKitsPrefix, version, "Debuggers", "x64", "windbg.exe"
            )


def _vswhere_path():
    try:
        import buildconfig

        path = os.path.join(buildconfig.topsrcdir, "build", "win32", "vswhere.exe")
        if os.path.isfile(path):
            return path
    except ImportError:
        pass
    # Hope it's available on PATH!
    return "vswhere.exe"


def get_debugger_path(debugger):
    """
    Get the full path of the debugger.

    :param debugger: The name of the debugger.
    """

    if mozinfo.os == "mac" and debugger == "lldb":
        # On newer OSX versions System Integrity Protections prevents us from
        # setting certain env vars for a process such as DYLD_LIBRARY_PATH if
        # it's in a protected directory such as /usr/bin. This is the case for
        # lldb, so we try to find an instance under the Xcode install instead.

        # Attempt to use the xcrun util to find the path.
        try:
            path = check_output(
                ["xcrun", "--find", "lldb"], universal_newlines=True
            ).strip()
            if path:
                return path
        except Exception:
            # Just default to find_executable instead.
            pass

    if mozinfo.os == "win" and debugger == "devenv.exe":
        # Attempt to use vswhere to find the path.
        try:
            encoding = "mbcs" if sys.platform == "win32" else "utf-8"
            vswhere = _vswhere_path()
            vsinfo = check_output([vswhere, "-format", "json", "-latest"])
            vsinfo = json.loads(vsinfo.decode(encoding, "replace"))
            return os.path.join(
                vsinfo[0]["installationPath"], "Common7", "IDE", "devenv.exe"
            )
        except Exception:
            # Just default to find_executable instead.
            pass

    return shutil.which(debugger)


def get_debugger_info(debugger, debuggerArgs=None, debuggerInteractive=False):
    """
    Get the information about the requested debugger.

    Returns a dictionary containing the |path| of the debugger executable,
    if it will run in |interactive| mode, its arguments and whether it needs
    to escape arguments it passes to the debugged program (|requiresEscapedArgs|).
    If the debugger cannot be found in the system, returns |None|.

    :param debugger: The name of the debugger.
    :param debuggerArgs: If specified, it's the arguments to pass to the debugger,
    as a string. Any debugger-specific separator arguments are appended after these
    arguments.
    :param debuggerInteractive: If specified, forces the debugger to be interactive.
    """

    debuggerPath = None

    if debugger:
        # Append '.exe' to the debugger on Windows if it's not present,
        # so things like '--debugger=devenv' work.
        if os.name == "nt" and not debugger.lower().endswith(".exe"):
            debugger += ".exe"

        debuggerPath = get_debugger_path(debugger)

    if not debuggerPath:
        # windbg is not installed with the standard set of tools, and it's
        # entirely possible that the user hasn't added the install location to
        # PATH, so we have to be a little more clever than normal to locate it.
        # Just try to look for it in the standard installed location(s).
        if debugger == "windbg.exe":
            for candidate in _windbg_installation_paths():
                if os.path.exists(candidate):
                    debuggerPath = candidate
                    break
        else:
            if os.path.exists(debugger):
                debuggerPath = debugger

    if not debuggerPath:
        print("Error: Could not find debugger %s." % debugger)
        print("Is it installed? Is it in your PATH?")
        return None

    debuggerName = os.path.basename(debuggerPath).lower()

    def get_debugger_info(type, default):
        if debuggerName in _DEBUGGER_INFO and type in _DEBUGGER_INFO[debuggerName]:
            return _DEBUGGER_INFO[debuggerName][type]
        return default

    # Define a namedtuple to access the debugger information from the outside world.
    debugger_arguments = []

    if debuggerArgs:
        # Append the provided debugger arguments at the end of the arguments list.
        debugger_arguments += debuggerArgs.split()

    debugger_arguments += get_debugger_info("args", [])

    # Override the default debugger interactive mode if needed.
    debugger_interactive = get_debugger_info("interactive", False)
    if debuggerInteractive:
        debugger_interactive = debuggerInteractive

    d = DebuggerInfo(
        debuggerPath,
        debugger_interactive,
        debugger_arguments,
        get_debugger_info("requiresEscapedArgs", False),
    )

    return d


# Defines the search policies to use in get_default_debugger_name.


class DebuggerSearch:
    OnlyFirst = 1
    KeepLooking = 2


def get_default_debugger_name(search=DebuggerSearch.OnlyFirst):
    """
    Get the debugger name for the default debugger on current platform.

    :param search: If specified, stops looking for the debugger if the
     default one is not found (|DebuggerSearch.OnlyFirst|) or keeps
     looking for other compatible debuggers (|DebuggerSearch.KeepLooking|).
    """

    mozinfo.find_and_update_from_json()
    os = mozinfo.info["os"]

    # Find out which debuggers are preferred for use on this platform.
    debuggerPriorities = _DEBUGGER_PRIORITIES[
        os if os in _DEBUGGER_PRIORITIES else "unknown"
    ]

    # Finally get the debugger information.
    for debuggerName in debuggerPriorities:
        debuggerPath = get_debugger_path(debuggerName)
        if debuggerPath:
            return debuggerName
        elif not search == DebuggerSearch.KeepLooking:
            return None

    return None


# Defines default values for Valgrind flags.
#
# --smc-check=all-non-file is required to deal with code generation and
#   patching by the various JITS.  Note that this is only necessary on
#   x86 and x86_64, but not on ARM.  This flag is only necessary for
#   Valgrind versions prior to 3.11.
#
# --vex-iropt-register-updates=allregs-at-mem-access is required so that
#   Valgrind generates correct register values whenever there is a
#   segfault that is caught and handled.  In particular OdinMonkey
#   requires this.  More recent Valgrinds (3.11 and later) provide
#   --px-default=allregs-at-mem-access and
#   --px-file-backed=unwindregs-at-mem-access
#   which provide a significantly cheaper alternative, by restricting the
#   precise exception behaviour to JIT generated code only.
#
# --trace-children=yes is required to get Valgrind to follow into
#   content and other child processes.  The resulting output can be
#   difficult to make sense of, and --child-silent-after-fork=yes
#   helps by causing Valgrind to be silent for the child in the period
#   after fork() but before its subsequent exec().
#
# --trace-children-skip lists processes that we are not interested
#   in tracing into.
#
# --leak-check=full requests full stack traces for all leaked blocks
#   detected at process exit.
#
# --show-possibly-lost=no requests blocks for which only an interior
#   pointer was found to be considered not leaked.
#
#
# TODO: pass in the user supplied args for V (--valgrind-args=) and
# use this to detect if a different tool has been selected.  If so
# adjust tool-specific args appropriately.
#
# TODO: pass in the path to the Valgrind to be used (--valgrind=), and
# check what flags it accepts.  Possible args that might be beneficial:
#
# --num-transtab-sectors=24   [reduces re-jitting overheads in long runs]
# --px-default=allregs-at-mem-access
# --px-file-backed=unwindregs-at-mem-access
#                             [these reduce PX overheads as described above]
#


def get_default_valgrind_args():
    return [
        "--fair-sched=yes",
        "--smc-check=all-non-file",
        "--vex-iropt-register-updates=allregs-at-mem-access",
        "--trace-children=yes",
        "--child-silent-after-fork=yes",
        (
            "--trace-children-skip="
            + "/usr/bin/hg,/bin/rm,*/bin/certutil,*/bin/pk12util,"
            + "*/bin/ssltunnel,*/bin/uname,*/bin/which,*/bin/ps,"
            + "*/bin/grep,*/bin/java,*/bin/lsb_release"
        ),
    ] + get_default_valgrind_tool_specific_args()


# The default tool is Memcheck.  Feeding these arguments to a different
# Valgrind tool will cause it to fail at startup, so don't do that!


def get_default_valgrind_tool_specific_args():
    return [
        "--partial-loads-ok=yes",
        "--leak-check=summary",
        "--show-possibly-lost=no",
        "--show-mismatched-frees=no",
    ]

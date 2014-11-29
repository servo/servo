#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import mozinfo
from collections import namedtuple
from distutils.spawn import find_executable

__all__ = ['get_debugger_info',
           'get_default_debugger_name',
           'DebuggerSearch']

'''
Map of debugging programs to information about them, like default arguments
and whether or not they are interactive.

To add support for a new debugger, simply add the relative entry in
_DEBUGGER_INFO and optionally update the _DEBUGGER_PRIORITIES.
'''
_DEBUGGER_INFO = {
    # gdb requires that you supply the '--args' flag in order to pass arguments
    # after the executable name to the executable.
    'gdb': {
        'interactive': True,
        'args': ['-q', '--args']
    },

    'cgdb': {
        'interactive': True,
        'args': ['-q', '--args']
    },

    'lldb': {
        'interactive': True,
        'args': ['--'],
        'requiresEscapedArgs': True
    },

    # Visual Studio Debugger Support.
    'devenv.exe': {
        'interactive': True,
        'args': ['-debugexe']
    },

    # Visual C++ Express Debugger Support.
    'wdexpress.exe': {
        'interactive': True,
        'args': ['-debugexe']
    },

    # valgrind doesn't explain much about leaks unless you set the
    # '--leak-check=full' flag. But there are a lot of objects that are
    # semi-deliberately leaked, so we set '--show-possibly-lost=no' to avoid
    # uninteresting output from those objects. We set '--smc-check==all-non-file'
    # and '--vex-iropt-register-updates=allregs-at-mem-access' so that valgrind
    # deals properly with JIT'd JavaScript code.
    'valgrind': {
        'interactive': False,
        'args': ['--leak-check=full',
                '--show-possibly-lost=no',
                '--smc-check=all-non-file',
                '--vex-iropt-register-updates=allregs-at-mem-access']
    }
}

# Maps each OS platform to the preferred debugger programs found in _DEBUGGER_INFO.
_DEBUGGER_PRIORITIES = {
      'win': ['devenv.exe', 'wdexpress.exe'],
      'linux': ['gdb', 'cgdb', 'lldb'],
      'mac': ['lldb', 'gdb'],
      'unknown': ['gdb']
}

def get_debugger_info(debugger, debuggerArgs = None, debuggerInteractive = False):
    '''
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
    '''

    debuggerPath = None

    if debugger:
        # Append '.exe' to the debugger on Windows if it's not present,
        # so things like '--debugger=devenv' work.
        if (os.name == 'nt'
            and not debugger.lower().endswith('.exe')):
            debugger += '.exe'

        debuggerPath = find_executable(debugger)

    if not debuggerPath:
        print 'Error: Could not find debugger %s.' % debugger
        return None

    debuggerName = os.path.basename(debuggerPath).lower()

    def get_debugger_info(type, default):
        if debuggerName in _DEBUGGER_INFO and type in _DEBUGGER_INFO[debuggerName]:
            return _DEBUGGER_INFO[debuggerName][type]
        return default

    # Define a namedtuple to access the debugger information from the outside world.
    DebuggerInfo = namedtuple(
        'DebuggerInfo',
        ['path', 'interactive', 'args', 'requiresEscapedArgs']
    )

    debugger_arguments = []

    if debuggerArgs:
        # Append the provided debugger arguments at the end of the arguments list.
        debugger_arguments += debuggerArgs.split()

    debugger_arguments += get_debugger_info('args', [])

    # Override the default debugger interactive mode if needed.
    debugger_interactive = get_debugger_info('interactive', False)
    if debuggerInteractive:
        debugger_interactive = debuggerInteractive

    d = DebuggerInfo(
        debuggerPath,
        debugger_interactive,
        debugger_arguments,
        get_debugger_info('requiresEscapedArgs', False)
    )

    return d

# Defines the search policies to use in get_default_debugger_name.
class DebuggerSearch:
  OnlyFirst = 1
  KeepLooking = 2

def get_default_debugger_name(search=DebuggerSearch.OnlyFirst):
    '''
    Get the debugger name for the default debugger on current platform.

    :param search: If specified, stops looking for the debugger if the
     default one is not found (|DebuggerSearch.OnlyFirst|) or keeps
     looking for other compatible debuggers (|DebuggerSearch.KeepLooking|).
    '''

    # Find out which debuggers are preferred for use on this platform.
    debuggerPriorities = _DEBUGGER_PRIORITIES[mozinfo.os if mozinfo.os in _DEBUGGER_PRIORITIES else 'unknown']

    # Finally get the debugger information.
    for debuggerName in debuggerPriorities:
        debuggerPath = find_executable(debuggerName)
        if debuggerPath:
            return debuggerName
        elif not search == DebuggerSearch.KeepLooking:
            return None

    return None

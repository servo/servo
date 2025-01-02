# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# This module provides mixins to perform process execution.

from __future__ import absolute_import, unicode_literals

import logging
import os
import signal
import subprocess
import sys

from mozbuild.util import ensure_subprocess_env
from mozprocess.processhandler import ProcessHandlerMixin

from .logging import LoggingMixin


# Perform detection of operating system environment. This is used by command
# execution. We only do this once to save redundancy. Yes, this can fail module
# loading. That is arguably OK.
if 'SHELL' in os.environ:
    _current_shell = os.environ['SHELL']
elif 'MOZILLABUILD' in os.environ:
    _current_shell = os.environ['MOZILLABUILD'] + '/msys/bin/sh.exe'
elif 'COMSPEC' in os.environ:
    _current_shell = os.environ['COMSPEC']
else:
    raise Exception('Could not detect environment shell!')

_in_msys = False

if os.environ.get('MSYSTEM', None) in ('MINGW32', 'MINGW64'):
    _in_msys = True

    if not _current_shell.lower().endswith('.exe'):
        _current_shell += '.exe'


class ProcessExecutionMixin(LoggingMixin):
    """Mix-in that provides process execution functionality."""

    def run_process(self, args=None, cwd=None, append_env=None,
                    explicit_env=None, log_name=None, log_level=logging.INFO,
                    line_handler=None, require_unix_environment=False,
                    ensure_exit_code=0, ignore_children=False, pass_thru=False):
        """Runs a single process to completion.

        Takes a list of arguments to run where the first item is the
        executable. Runs the command in the specified directory and
        with optional environment variables.

        append_env -- Dict of environment variables to append to the current
            set of environment variables.
        explicit_env -- Dict of environment variables to set for the new
            process. Any existing environment variables will be ignored.

        require_unix_environment if True will ensure the command is executed
        within a UNIX environment. Basically, if we are on Windows, it will
        execute the command via an appropriate UNIX-like shell.

        ignore_children is proxied to mozprocess's ignore_children.

        ensure_exit_code is used to ensure the exit code of a process matches
        what is expected. If it is an integer, we raise an Exception if the
        exit code does not match this value. If it is True, we ensure the exit
        code is 0. If it is False, we don't perform any exit code validation.

        pass_thru is a special execution mode where the child process inherits
        this process's standard file handles (stdin, stdout, stderr) as well as
        additional file descriptors. It should be used for interactive processes
        where buffering from mozprocess could be an issue. pass_thru does not
        use mozprocess. Therefore, arguments like log_name, line_handler,
        and ignore_children have no effect.
        """
        args = self._normalize_command(args, require_unix_environment)

        self.log(logging.INFO, 'new_process', {'args': ' '.join(args)}, '{args}')

        def handleLine(line):
            # Converts str to unicode on Python 2 and bytes to str on Python 3.
            if isinstance(line, bytes):
                line = line.decode(sys.stdout.encoding or 'utf-8', 'replace')

            if line_handler:
                line_handler(line)

            if not log_name:
                return

            self.log(log_level, log_name, {'line': line.rstrip()}, '{line}')

        use_env = {}
        if explicit_env:
            use_env = explicit_env
        else:
            use_env.update(os.environ)

            if append_env:
                use_env.update(append_env)

        self.log(logging.DEBUG, 'process', {'env': use_env}, 'Environment: {env}')

        use_env = ensure_subprocess_env(use_env)
        if pass_thru:
            proc = subprocess.Popen(args, cwd=cwd, env=use_env)
            status = None
            # Leave it to the subprocess to handle Ctrl+C. If it terminates as
            # a result of Ctrl+C, proc.wait() will return a status code, and,
            # we get out of the loop. If it doesn't, like e.g. gdb, we continue
            # waiting.
            while status is None:
                try:
                    status = proc.wait()
                except KeyboardInterrupt:
                    pass
        else:
            p = ProcessHandlerMixin(args, cwd=cwd, env=use_env,
                                    processOutputLine=[handleLine],
                                    universal_newlines=True,
                                    ignore_children=ignore_children)
            p.run()
            p.processOutput()
            status = None
            sig = None
            while status is None:
                try:
                    if sig is None:
                        status = p.wait()
                    else:
                        status = p.kill(sig=sig)
                except KeyboardInterrupt:
                    if sig is None:
                        sig = signal.SIGINT
                    elif sig == signal.SIGINT:
                        # If we've already tried SIGINT, escalate.
                        sig = signal.SIGKILL

        if ensure_exit_code is False:
            return status

        if ensure_exit_code is True:
            ensure_exit_code = 0

        if status != ensure_exit_code:
            raise Exception('Process executed with non-0 exit code %d: %s' % (status, args))

        return status

    def _normalize_command(self, args, require_unix_environment):
        """Adjust command arguments to run in the necessary environment.

        This exists mainly to facilitate execution of programs requiring a *NIX
        shell when running on Windows. The caller specifies whether a shell
        environment is required. If it is and we are running on Windows but
        aren't running in the UNIX-like msys environment, then we rewrite the
        command to execute via a shell.
        """
        assert isinstance(args, list) and len(args)

        if not require_unix_environment or not _in_msys:
            return args

        # Always munge Windows-style into Unix style for the command.
        prog = args[0].replace('\\', '/')

        # PyMake removes the C: prefix. But, things seem to work here
        # without it. Not sure what that's about.

        # We run everything through the msys shell. We need to use
        # '-c' and pass all the arguments as one argument because that is
        # how sh works.
        cline = subprocess.list2cmdline([prog] + args[1:])
        return [_current_shell, '-c', cline]

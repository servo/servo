# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, unicode_literals


class CommandContext(object):
    """Holds run-time state so it can easily be passed to command providers."""
    def __init__(self, cwd=None, settings=None, log_manager=None, commands=None, **kwargs):
        self.cwd = cwd
        self.settings = settings
        self.log_manager = log_manager
        self.commands = commands
        self.command_attrs = {}

        for k, v in kwargs.items():
            setattr(self, k, v)


class MachError(Exception):
    """Base class for all errors raised by mach itself."""


class NoCommandError(MachError):
    """No command was passed into mach."""


class UnknownCommandError(MachError):
    """Raised when we attempted to execute an unknown command."""

    def __init__(self, command, verb, suggested_commands=None):
        MachError.__init__(self)

        self.command = command
        self.verb = verb
        self.suggested_commands = suggested_commands or []


class UnrecognizedArgumentError(MachError):
    """Raised when an unknown argument is passed to mach."""

    def __init__(self, command, arguments):
        MachError.__init__(self)

        self.command = command
        self.arguments = arguments


class FailedCommandError(Exception):
    """Raised by commands to signal a handled failure to be printed by mach

    When caught by mach a FailedCommandError will print message and exit
    with ''exit_code''. The optional ''reason'' is a string in cases where
    other scripts may wish to handle the exception, though this is generally
    intended to communicate failure to mach.
    """

    def __init__(self, message, exit_code=1, reason=''):
        Exception.__init__(self, message)
        self.exit_code = exit_code
        self.reason = reason


class MissingFileError(MachError):
    """Attempted to load a mach commands file that doesn't exist."""

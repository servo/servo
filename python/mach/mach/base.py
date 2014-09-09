# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import unicode_literals


class CommandContext(object):
    """Holds run-time state so it can easily be passed to command providers."""
    def __init__(self, cwd=None, settings=None, log_manager=None,
        commands=None, **kwargs):
        self.cwd = cwd
        self.settings = settings
        self.log_manager = log_manager
        self.commands = commands

        for k,v in kwargs.items():
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


class MethodHandler(object):
    """Describes a Python method that implements a mach command.

    Instances of these are produced by mach when it processes classes
    defining mach commands.
    """
    __slots__ = (
        # The Python class providing the command. This is the class type not
        # an instance of the class. Mach will instantiate a new instance of
        # the class if the command is executed.
        'cls',

        # Whether the __init__ method of the class should receive a mach
        # context instance. This should only affect the mach driver and how
        # it instantiates classes.
        'pass_context',

        # The name of the method providing the command. In other words, this
        # is the str name of the attribute on the class type corresponding to
        # the name of the function.
        'method',

        # The name of the command.
        'name',

        # String category this command belongs to.
        'category',

        # Description of the purpose of this command.
        'description',

        # Whether to allow all arguments from the parser.
        'allow_all_arguments',

        # Functions used to 'skip' commands if they don't meet the conditions
        # in a given context.
        'conditions',

        # argparse.ArgumentParser instance to use as the basis for command
        # arguments.
        'parser',

        # Arguments added to this command's parser. This is a 2-tuple of
        # positional and named arguments, respectively.
        'arguments',
    )

    def __init__(self, cls, method, name, category=None, description=None,
        allow_all_arguments=False, conditions=None, parser=None, arguments=None,
        pass_context=False):

        self.cls = cls
        self.method = method
        self.name = name
        self.category = category
        self.description = description
        self.allow_all_arguments = allow_all_arguments
        self.conditions = conditions or []
        self.parser = parser
        self.arguments = arguments or []
        self.pass_context = pass_context


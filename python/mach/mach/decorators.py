# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, unicode_literals

import argparse
import collections
import inspect
import types

from .base import MachError
from .config import ConfigProvider
from .registrar import Registrar


class _MachCommand(object):
    """Container for mach command metadata.

    Mach commands contain lots of attributes. This class exists to capture them
    in a sane way so tuples, etc aren't used instead.
    """
    __slots__ = (
        # Content from decorator arguments to define the command.
        'name',
        'subcommand',
        'category',
        'description',
        'conditions',
        '_parser',
        'arguments',
        'argument_group_names',

        # Describes how dispatch is performed.

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

        # Dict of string to _MachCommand defining sub-commands for this
        # command.
        'subcommand_handlers',
    )

    def __init__(self, name=None, subcommand=None, category=None,
                 description=None, conditions=None, parser=None):
        self.name = name
        self.subcommand = subcommand
        self.category = category
        self.description = description
        self.conditions = conditions or []
        self._parser = parser
        self.arguments = []
        self.argument_group_names = []

        self.cls = None
        self.pass_context = None
        self.method = None
        self.subcommand_handlers = {}

    @property
    def parser(self):
        # Creating CLI parsers at command dispatch time can be expensive. Make
        # it possible to lazy load them by using functions.
        if callable(self._parser):
            self._parser = self._parser()

        return self._parser

    @property
    def docstring(self):
        return self.cls.__dict__[self.method].__doc__

    def __ior__(self, other):
        if not isinstance(other, _MachCommand):
            raise ValueError('can only operate on _MachCommand instances')

        for a in self.__slots__:
            if not getattr(self, a):
                setattr(self, a, getattr(other, a))

        return self


def CommandProvider(cls):
    """Class decorator to denote that it provides subcommands for Mach.

    When this decorator is present, mach looks for commands being defined by
    methods inside the class.
    """

    # The implementation of this decorator relies on the parse-time behavior of
    # decorators. When the module is imported, the method decorators (like
    # @Command and @CommandArgument) are called *before* this class decorator.
    # The side-effect of the method decorators is to store specifically-named
    # attributes on the function types. We just scan over all functions in the
    # class looking for the side-effects of the method decorators.

    # Tell mach driver whether to pass context argument to __init__.
    pass_context = False

    if inspect.ismethod(cls.__init__):
        spec = inspect.getargspec(cls.__init__)

        if len(spec.args) > 2:
            msg = 'Mach @CommandProvider class %s implemented incorrectly. ' + \
                  '__init__() must take 1 or 2 arguments. From %s'
            msg = msg % (cls.__name__, inspect.getsourcefile(cls))
            raise MachError(msg)

        if len(spec.args) == 2:
            pass_context = True

    seen_commands = set()

    # We scan __dict__ because we only care about the classes own attributes,
    # not inherited ones. If we did inherited attributes, we could potentially
    # define commands multiple times. We also sort keys so commands defined in
    # the same class are grouped in a sane order.
    for attr in sorted(cls.__dict__.keys()):
        value = cls.__dict__[attr]

        if not isinstance(value, types.FunctionType):
            continue

        command = getattr(value, '_mach_command', None)
        if not command:
            continue

        # Ignore subcommands for now: we handle them later.
        if command.subcommand:
            continue

        seen_commands.add(command.name)

        if not command.conditions and Registrar.require_conditions:
            continue

        msg = 'Mach command \'%s\' implemented incorrectly. ' + \
              'Conditions argument must take a list ' + \
              'of functions. Found %s instead.'

        if not isinstance(command.conditions, collections.Iterable):
            msg = msg % (command.name, type(command.conditions))
            raise MachError(msg)

        for c in command.conditions:
            if not hasattr(c, '__call__'):
                msg = msg % (command.name, type(c))
                raise MachError(msg)

        command.cls = cls
        command.method = attr
        command.pass_context = pass_context

        Registrar.register_command_handler(command)

    # Now do another pass to get sub-commands. We do this in two passes so
    # we can check the parent command existence without having to hold
    # state and reconcile after traversal.
    for attr in sorted(cls.__dict__.keys()):
        value = cls.__dict__[attr]

        if not isinstance(value, types.FunctionType):
            continue

        command = getattr(value, '_mach_command', None)
        if not command:
            continue

        # It is a regular command.
        if not command.subcommand:
            continue

        if command.name not in seen_commands:
            raise MachError('Command referenced by sub-command does not '
                'exist: %s' % command.name)

        if command.name not in Registrar.command_handlers:
            continue

        command.cls = cls
        command.method = attr
        command.pass_context = pass_context
        parent = Registrar.command_handlers[command.name]

        if parent._parser:
            raise MachError('cannot declare sub commands against a command '
                'that has a parser installed: %s' % command)
        if command.subcommand in parent.subcommand_handlers:
            raise MachError('sub-command already defined: %s' % command.subcommand)

        parent.subcommand_handlers[command.subcommand] = command

    return cls


class Command(object):
    """Decorator for functions or methods that provide a mach command.

    The decorator accepts arguments that define basic attributes of the
    command. The following arguments are recognized:

         category -- The string category to which this command belongs. Mach's
             help will group commands by category.

         description -- A brief description of what the command does.

         parser -- an optional argparse.ArgumentParser instance or callable
             that returns an argparse.ArgumentParser instance to use as the
             basis for the command arguments.

    For example:

        @Command('foo', category='misc', description='Run the foo action')
        def foo(self):
            pass
    """
    def __init__(self, name, **kwargs):
        self._mach_command = _MachCommand(name=name, **kwargs)

    def __call__(self, func):
        if not hasattr(func, '_mach_command'):
            func._mach_command = _MachCommand()

        func._mach_command |= self._mach_command

        return func

class SubCommand(object):
    """Decorator for functions or methods that provide a sub-command.

    Mach commands can have sub-commands. e.g. ``mach command foo`` or
    ``mach command bar``. Each sub-command has its own parser and is
    effectively its own mach command.

    The decorator accepts arguments that define basic attributes of the
    sub command:

        command -- The string of the command this sub command should be
        attached to.

        subcommand -- The string name of the sub command to register.

        description -- A textual description for this sub command.
    """
    def __init__(self, command, subcommand, description=None):
        self._mach_command = _MachCommand(name=command, subcommand=subcommand,
                                          description=description)

    def __call__(self, func):
        if not hasattr(func, '_mach_command'):
            func._mach_command = _MachCommand()

        func._mach_command |= self._mach_command

        return func

class CommandArgument(object):
    """Decorator for additional arguments to mach subcommands.

    This decorator should be used to add arguments to mach commands. Arguments
    to the decorator are proxied to ArgumentParser.add_argument().

    For example:

        @Command('foo', help='Run the foo action')
        @CommandArgument('-b', '--bar', action='store_true', default=False,
            help='Enable bar mode.')
        def foo(self):
            pass
    """
    def __init__(self, *args, **kwargs):
        if kwargs.get('nargs') == argparse.REMAINDER:
            # These are the assertions we make in dispatcher.py about
            # those types of CommandArguments.
            assert len(args) == 1
            assert all(k in ('default', 'nargs', 'help', 'group') for k in kwargs)
        self._command_args = (args, kwargs)

    def __call__(self, func):
        if not hasattr(func, '_mach_command'):
            func._mach_command = _MachCommand()

        func._mach_command.arguments.insert(0, self._command_args)

        return func


class CommandArgumentGroup(object):
    """Decorator for additional argument groups to mach commands.

    This decorator should be used to add arguments groups to mach commands.
    Arguments to the decorator are proxied to
    ArgumentParser.add_argument_group().

    For example:

        @Command('foo', helps='Run the foo action')
        @CommandArgumentGroup('group1')
        @CommandArgument('-b', '--bar', group='group1', action='store_true',
            default=False, help='Enable bar mode.')
        def foo(self):
            pass

    The name should be chosen so that it makes sense as part of the phrase
    'Command Arguments for <name>' because that's how it will be shown in the
    help message.
    """
    def __init__(self, group_name):
        self._group_name = group_name

    def __call__(self, func):
        if not hasattr(func, '_mach_command'):
            func._mach_command = _MachCommand()

        func._mach_command.argument_group_names.insert(0, self._group_name)

        return func


def SettingsProvider(cls):
    """Class decorator to denote that this class provides Mach settings.

    When this decorator is encountered, the underlying class will automatically
    be registered with the Mach registrar and will (likely) be hooked up to the
    mach driver.

    This decorator is only allowed on mach.config.ConfigProvider classes.
    """
    if not issubclass(cls, ConfigProvider):
        raise MachError('@SettingsProvider encountered on class that does ' +
                        'not derived from mach.config.ConfigProvider.')

    Registrar.register_settings_provider(cls)

    return cls


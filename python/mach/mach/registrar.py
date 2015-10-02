# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, unicode_literals

from .base import MachError

INVALID_COMMAND_CONTEXT = r'''
It looks like you tried to run a mach command from an invalid context. The %s
command failed to meet the following conditions: %s

Run |mach help| to show a list of all commands available to the current context.
'''.lstrip()


class MachRegistrar(object):
    """Container for mach command and config providers."""

    def __init__(self):
        self.command_handlers = {}
        self.commands_by_category = {}
        self.settings_providers = set()
        self.categories = {}
        self.require_conditions = False

    def register_command_handler(self, handler):
        name = handler.name

        if not handler.category:
            raise MachError('Cannot register a mach command without a '
                'category: %s' % name)

        if handler.category not in self.categories:
            raise MachError('Cannot register a command to an undefined '
                'category: %s -> %s' % (name, handler.category))

        self.command_handlers[name] = handler
        self.commands_by_category[handler.category].add(name)

    def register_settings_provider(self, cls):
        self.settings_providers.add(cls)

    def register_category(self, name, title, description, priority=50):
        self.categories[name] = (title, description, priority)
        self.commands_by_category[name] = set()

    @classmethod
    def _condition_failed_message(cls, name, conditions):
        msg = ['\n']
        for c in conditions:
            part = ['  %s' % c.__name__]
            if c.__doc__ is not None:
                part.append(c.__doc__)
            msg.append(' - '.join(part))
        return INVALID_COMMAND_CONTEXT % (name, '\n'.join(msg))

    def _run_command_handler(self, handler, context=None, debug_command=False, **kwargs):
        cls = handler.cls

        if handler.pass_context and not context:
            raise Exception('mach command class requires context.')

        if context:
            prerun = getattr(context, 'pre_dispatch_handler', None)
            if prerun:
                prerun(context, handler, args=kwargs)

        if handler.pass_context:
            instance = cls(context)
        else:
            instance = cls()

        if handler.conditions:
            fail_conditions = []
            for c in handler.conditions:
                if not c(instance):
                    fail_conditions.append(c)

            if fail_conditions:
                print(self._condition_failed_message(handler.name, fail_conditions))
                return 1

        fn = getattr(instance, handler.method)

        if debug_command:
            import pdb
            result = pdb.runcall(fn, **kwargs)
        else:
            result = fn(**kwargs)

        result = result or 0
        assert isinstance(result, (int, long))
        return result

    def dispatch(self, name, context=None, argv=None, **kwargs):
        """Dispatch/run a command.

        Commands can use this to call other commands.
        """
        # TODO handler.subcommand_handlers are ignored
        handler = self.command_handlers[name]

        if handler.parser:
            parser = handler.parser

            # save and restore existing defaults so **kwargs don't persist across
            # subsequent invocations of Registrar.dispatch()
            old_defaults = parser._defaults.copy()
            parser.set_defaults(**kwargs)
            kwargs, _ = parser.parse_known_args(argv or [])
            kwargs = vars(kwargs)
            parser._defaults = old_defaults

        return self._run_command_handler(handler, context=context, **kwargs)



Registrar = MachRegistrar()

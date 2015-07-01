# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, # You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, print_function, unicode_literals

from mach.decorators import (
    CommandProvider,
    Command,
    CommandArgument,
)


@CommandProvider
class BuiltinCommands(object):
    def __init__(self, context):
        self.context = context

    @Command('mach-commands', category='misc',
        description='List all mach commands.')
    def commands(self):
        print("\n".join(self.context.commands.command_handlers.keys()))

    @Command('mach-debug-commands', category='misc',
        description='Show info about available mach commands.')
    @CommandArgument('match', metavar='MATCH', default=None, nargs='?',
        help='Only display commands containing given substring.')
    def debug_commands(self, match=None):
        import inspect

        handlers = self.context.commands.command_handlers
        for command in sorted(handlers.keys()):
            if match and match not in command:
                continue

            handler = handlers[command]
            cls = handler.cls
            method = getattr(cls, getattr(handler, 'method'))

            print(command)
            print('=' * len(command))
            print('')
            print('File: %s' % inspect.getsourcefile(method))
            print('Class: %s' % cls.__name__)
            print('Method: %s' % handler.method)
            print('')


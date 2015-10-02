# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, unicode_literals

import argparse
import difflib
import sys

from operator import itemgetter

from .base import (
    MachError,
    NoCommandError,
    UnknownCommandError,
    UnrecognizedArgumentError,
)


class CommandFormatter(argparse.HelpFormatter):
    """Custom formatter to format just a subcommand."""

    def add_usage(self, *args):
        pass


class CommandAction(argparse.Action):
    """An argparse action that handles mach commands.

    This class is essentially a reimplementation of argparse's sub-parsers
    feature. We first tried to use sub-parsers. However, they were missing
    features like grouping of commands (http://bugs.python.org/issue14037).

    The way this works involves light magic and a partial understanding of how
    argparse works.

    Arguments registered with an argparse.ArgumentParser have an action
    associated with them. An action is essentially a class that when called
    does something with the encountered argument(s). This class is one of those
    action classes.

    An instance of this class is created doing something like:

        parser.add_argument('command', action=CommandAction, registrar=r)

    Note that a mach.registrar.Registrar instance is passed in. The Registrar
    holds information on all the mach commands that have been registered.

    When this argument is registered with the ArgumentParser, an instance of
    this class is instantiated. One of the subtle but important things it does
    is tell the argument parser that it's interested in *all* of the remaining
    program arguments. So, when the ArgumentParser calls this action, we will
    receive the command name plus all of its arguments.

    For more, read the docs in __call__.
    """
    def __init__(self, option_strings, dest, required=True, default=None,
        registrar=None, context=None):
        # A proper API would have **kwargs here. However, since we are a little
        # hacky, we intentionally omit it as a way of detecting potentially
        # breaking changes with argparse's implementation.
        #
        # In a similar vein, default is passed in but is not needed, so we drop
        # it.
        argparse.Action.__init__(self, option_strings, dest, required=required,
            help=argparse.SUPPRESS, nargs=argparse.REMAINDER)

        self._mach_registrar = registrar
        self._context = context

    def __call__(self, parser, namespace, values, option_string=None):
        """This is called when the ArgumentParser has reached our arguments.

        Since we always register ourselves with nargs=argparse.REMAINDER,
        values should be a list of remaining arguments to parse. The first
        argument should be the name of the command to invoke and all remaining
        arguments are arguments for that command.

        The gist of the flow is that we look at the command being invoked. If
        it's *help*, we handle that specially (because argparse's default help
        handler isn't satisfactory). Else, we create a new, independent
        ArgumentParser instance for just the invoked command (based on the
        information contained in the command registrar) and feed the arguments
        into that parser. We then merge the results with the main
        ArgumentParser.
        """
        if namespace.help:
            # -h or --help is in the global arguments.
            self._handle_main_help(parser, namespace.verbose)
            sys.exit(0)
        elif values:
            command = values[0].lower()
            args = values[1:]
            if command == 'help':
                if args and args[0] not in ['-h', '--help']:
                    # Make sure args[0] is indeed a command.
                    self._handle_command_help(parser, args[0])
                else:
                    self._handle_main_help(parser, namespace.verbose)
                sys.exit(0)
            elif '-h' in args or '--help' in args:
                # -h or --help is in the command arguments.
                if '--' in args:
                    # -- is in command arguments
                    if '-h' in args[:args.index('--')] or '--help' in args[:args.index('--')]:
                        # Honor -h or --help only if it appears before --
                        self._handle_command_help(parser, command)
                        sys.exit(0)
                else:
                    self._handle_command_help(parser, command)
                    sys.exit(0)


        else:
            raise NoCommandError()

        # Command suggestion
        if command not in self._mach_registrar.command_handlers:
            # Make sure we don't suggest any deprecated commands.
            names = [h.name for h in self._mach_registrar.command_handlers.values()
                        if h.cls.__name__ == 'DeprecatedCommands']
            # We first try to look for a valid command that is very similar to the given command.
            suggested_commands = difflib.get_close_matches(command, names, cutoff=0.8)
            # If we find more than one matching command, or no command at all, we give command suggestions instead
            # (with a lower matching threshold). All commands that start with the given command (for instance: 'mochitest-plain',
            # 'mochitest-chrome', etc. for 'mochitest-') are also included.
            if len(suggested_commands) != 1:
                suggested_commands = set(difflib.get_close_matches(command, names, cutoff=0.5))
                suggested_commands |= {cmd for cmd in names if cmd.startswith(command)}
                raise UnknownCommandError(command, 'run', suggested_commands)
            sys.stderr.write("We're assuming the '%s' command is '%s' and we're executing it for you.\n\n" % (command, suggested_commands[0]))
            command = suggested_commands[0]

        handler = self._mach_registrar.command_handlers.get(command)

        usage = '%(prog)s [global arguments] ' + command + \
            ' [command arguments]'

        subcommand = None

        # If there are sub-commands, parse the intent out immediately.
        if handler.subcommand_handlers:
            if not args:
                self._handle_subcommand_main_help(parser, handler)
                sys.exit(0)
            elif len(args) == 1 and args[0] in ('help', '--help'):
                self._handle_subcommand_main_help(parser, handler)
                sys.exit(0)
            # mach <command> help <subcommand>
            elif len(args) == 2 and args[0] == 'help':
                subcommand = args[1]
                subhandler = handler.subcommand_handlers[subcommand]
                self._handle_subcommand_help(parser, command, subcommand, subhandler)
                sys.exit(0)
            # We are running a sub command.
            else:
                subcommand = args[0]
                if subcommand[0] == '-':
                    raise MachError('%s invoked improperly. A sub-command name '
                        'must be the first argument after the command name.' %
                        command)

                if subcommand not in handler.subcommand_handlers:
                    raise UnknownCommandError(subcommand, 'run',
                        handler.subcommand_handlers.keys())

                handler = handler.subcommand_handlers[subcommand]
                usage = '%(prog)s [global arguments] ' + command + ' ' + \
                    subcommand + ' [command arguments]'
                args.pop(0)

        # We create a new parser, populate it with the command's arguments,
        # then feed all remaining arguments to it, merging the results
        # with ourselves. This is essentially what argparse subparsers
        # do.

        parser_args = {
            'add_help': False,
            'usage': usage,
        }

        remainder = None

        if handler.parser:
            subparser = handler.parser
            subparser.context = self._context
            for arg in subparser._actions[:]:
                if arg.nargs == argparse.REMAINDER:
                    subparser._actions.remove(arg)
                    remainder = (arg.dest,), {'default': arg.default,
                                              'nargs': arg.nargs,
                                              'help': arg.help}
        else:
            subparser = argparse.ArgumentParser(**parser_args)

        for arg in handler.arguments:
            # Remove our group keyword; it's not needed here.
            group_name = arg[1].get('group')
            if group_name:
                del arg[1]['group']

            if arg[1].get('nargs') == argparse.REMAINDER:
                # parse_known_args expects all argparse.REMAINDER ('...')
                # arguments to be all stuck together. Instead, we want them to
                # pick any extra argument, wherever they are.
                # Assume a limited CommandArgument for those arguments.
                assert len(arg[0]) == 1
                assert all(k in ('default', 'nargs', 'help') for k in arg[1])
                remainder = arg
            else:
                subparser.add_argument(*arg[0], **arg[1])

        # We define the command information on the main parser result so as to
        # not interfere with arguments passed to the command.
        setattr(namespace, 'mach_handler', handler)
        setattr(namespace, 'command', command)
        setattr(namespace, 'subcommand', subcommand)

        command_namespace, extra = subparser.parse_known_args(args)
        setattr(namespace, 'command_args', command_namespace)
        if remainder:
            (name,), options = remainder
            # parse_known_args usefully puts all arguments after '--' in
            # extra, but also puts '--' there. We don't want to pass it down
            # to the command handler. Note that if multiple '--' are on the
            # command line, only the first one is removed, so that subsequent
            # ones are passed down.
            if '--' in extra:
                extra.remove('--')

            # Commands with argparse.REMAINDER arguments used to force the
            # other arguments to be '+' prefixed. If a user now passes such
            # an argument, if will silently end up in extra. So, check if any
            # of the allowed arguments appear in a '+' prefixed form, and error
            # out if that's the case.
            for args, _ in handler.arguments:
                for arg in args:
                    arg = arg.replace('-', '+', 1)
                    if arg in extra:
                        raise UnrecognizedArgumentError(command, [arg])

            if extra:
                setattr(command_namespace, name, extra)
            else:
                setattr(command_namespace, name, options.get('default', []))
        elif extra and handler.cls.__name__ != 'DeprecatedCommands':
            raise UnrecognizedArgumentError(command, extra)

    def _handle_main_help(self, parser, verbose):
        # Since we don't need full sub-parser support for the main help output,
        # we create groups in the ArgumentParser and populate each group with
        # arguments corresponding to command names. This has the side-effect
        # that argparse renders it nicely.
        r = self._mach_registrar
        disabled_commands = []

        cats = [(k, v[2]) for k, v in r.categories.items()]
        sorted_cats = sorted(cats, key=itemgetter(1), reverse=True)
        for category, priority in sorted_cats:
            group = None

            for command in sorted(r.commands_by_category[category]):
                handler = r.command_handlers[command]

                # Instantiate a handler class to see if it should be filtered
                # out for the current context or not. Condition functions can be
                # applied to the command's decorator.
                if handler.conditions:
                    if handler.pass_context:
                        instance = handler.cls(self._context)
                    else:
                        instance = handler.cls()

                    is_filtered = False
                    for c in handler.conditions:
                        if not c(instance):
                            is_filtered = True
                            break
                    if is_filtered:
                        description = handler.description
                        disabled_command = {'command': command, 'description': description}
                        disabled_commands.append(disabled_command)
                        continue

                if group is None:
                    title, description, _priority = r.categories[category]
                    group = parser.add_argument_group(title, description)

                description = handler.description
                group.add_argument(command, help=description,
                    action='store_true')

        if disabled_commands and 'disabled' in r.categories:
            title, description, _priority = r.categories['disabled']
            group = parser.add_argument_group(title, description)
            if verbose == True:
                for c in disabled_commands:
                    group.add_argument(c['command'], help=c['description'],
                                       action='store_true')

        parser.print_help()

    def _populate_command_group(self, parser, handler, group):
        extra_groups = {}
        for group_name in handler.argument_group_names:
            group_full_name = 'Command Arguments for ' + group_name
            extra_groups[group_name] = \
                parser.add_argument_group(group_full_name)

        for arg in handler.arguments:
            # Apply our group keyword.
            group_name = arg[1].get('group')
            if group_name:
                del arg[1]['group']
                group = extra_groups[group_name]
            group.add_argument(*arg[0], **arg[1])

    def _handle_command_help(self, parser, command):
        handler = self._mach_registrar.command_handlers.get(command)

        if not handler:
            raise UnknownCommandError(command, 'query')

        if handler.subcommand_handlers:
            self._handle_subcommand_main_help(parser, handler)
            return

        # This code is worth explaining. Because we are doing funky things with
        # argument registration to allow the same option in both global and
        # command arguments, we can't simply put all arguments on the same
        # parser instance because argparse would complain. We can't register an
        # argparse subparser here because it won't properly show help for
        # global arguments. So, we employ a strategy similar to command
        # execution where we construct a 2nd, independent ArgumentParser for
        # just the command data then supplement the main help's output with
        # this 2nd parser's. We use a custom formatter class to ignore some of
        # the help output.
        parser_args = {
            'formatter_class': CommandFormatter,
            'add_help': False,
        }

        if handler.parser:
            c_parser = handler.parser
            c_parser.context = self._context
            c_parser.formatter_class = NoUsageFormatter
            # Accessing _action_groups is a bit shady. We are highly dependent
            # on the argparse implementation not changing. We fail fast to
            # detect upstream changes so we can intelligently react to them.
            group = c_parser._action_groups[1]

            # By default argparse adds two groups called "positional arguments"
            # and "optional arguments". We want to rename these to reflect standard
            # mach terminology.
            c_parser._action_groups[0].title = 'Command Parameters'
            c_parser._action_groups[1].title = 'Command Arguments'

            if not handler.description:
                handler.description = c_parser.description
                c_parser.description = None
        else:
            c_parser = argparse.ArgumentParser(**parser_args)
            group = c_parser.add_argument_group('Command Arguments')

        self._populate_command_group(c_parser, handler, group)

        # Set the long help of the command to the docstring (if present) or
        # the command decorator description argument (if present).
        if handler.docstring:
            parser.description = format_docstring(handler.docstring)
        elif handler.description:
            parser.description = handler.description

        parser.usage = '%(prog)s [global arguments] ' + command + \
            ' [command arguments]'

        # This is needed to preserve line endings in the description field,
        # which may be populated from a docstring.
        parser.formatter_class = argparse.RawDescriptionHelpFormatter
        parser.print_help()
        print('')
        c_parser.print_help()

    def _handle_subcommand_main_help(self, parser, handler):
        parser.usage = '%(prog)s [global arguments] ' + handler.name + \
            ' subcommand [subcommand arguments]'
        group = parser.add_argument_group('Sub Commands')

        for subcommand, subhandler in sorted(handler.subcommand_handlers.iteritems()):
            group.add_argument(subcommand, help=subhandler.description,
                action='store_true')

        if handler.docstring:
            parser.description = format_docstring(handler.docstring)

        parser.formatter_class = argparse.RawDescriptionHelpFormatter

        parser.print_help()

    def _handle_subcommand_help(self, parser, command, subcommand, handler):
        parser.usage = '%(prog)s [global arguments] ' + command + \
            ' ' + subcommand + ' [command arguments]'

        c_parser = argparse.ArgumentParser(add_help=False,
            formatter_class=CommandFormatter)
        group = c_parser.add_argument_group('Sub Command Arguments')
        self._populate_command_group(c_parser, handler, group)

        if handler.docstring:
            parser.description = format_docstring(handler.docstring)

        parser.formatter_class = argparse.RawDescriptionHelpFormatter

        parser.print_help()
        print('')
        c_parser.print_help()


class NoUsageFormatter(argparse.HelpFormatter):
    def _format_usage(self, *args, **kwargs):
        return ""


def format_docstring(docstring):
    """Format a raw docstring into something suitable for presentation.

    This function is based on the example function in PEP-0257.
    """
    if not docstring:
        return ''
    lines = docstring.expandtabs().splitlines()
    indent = sys.maxint
    for line in lines[1:]:
        stripped = line.lstrip()
        if stripped:
            indent = min(indent, len(line) - len(stripped))
    trimmed = [lines[0].strip()]
    if indent < sys.maxint:
        for line in lines[1:]:
            trimmed.append(line[indent:].rstrip())
    while trimmed and not trimmed[-1]:
        trimmed.pop()
    while trimmed and not trimmed[0]:
        trimmed.pop(0)
    return '\n'.join(trimmed)

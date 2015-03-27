#!/usr/bin/env python

import argparse
import unstable
import format as formatlog
import logmerge

def get_parser():
    parser = argparse.ArgumentParser("structlog",
                                     description="Tools for dealing with structured logs")

    commands = {"unstable": (unstable.get_parser, unstable.main),
                "format": (formatlog.get_parser, formatlog.main),
                "logmerge": (logmerge.get_parser, logmerge.main)}

    sub_parser = parser.add_subparsers(title='Subcommands')

    for command, (parser_func, main_func) in commands.iteritems():
        parent = parser_func(False)
        command_parser = sub_parser.add_parser(command,
                                               description=parent.description,
                                               parents=[parent])
        command_parser.set_defaults(func=main_func)

    return parser

def main():
    parser = get_parser()
    args = parser.parse_args()
    args.func(**vars(args))

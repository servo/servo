import argparse
import json
import logging
import os
import sys

from tools import localpaths  # noqa: F401

from six import iteritems
from . import virtualenv


here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))


def load_commands():
    rv = {}
    with open(os.path.join(here, "paths"), "r") as f:
        paths = [item.strip().replace("/", os.path.sep) for item in f if item.strip()]
    for path in paths:
        abs_path = os.path.join(wpt_root, path, "commands.json")
        base_dir = os.path.dirname(abs_path)
        with open(abs_path, "r") as f:
            data = json.load(f)
            for command, props in iteritems(data):
                assert "path" in props
                assert "script" in props
                rv[command] = {
                    "path": os.path.join(base_dir, props["path"]),
                    "script": props["script"],
                    "parser": props.get("parser"),
                    "parse_known": props.get("parse_known", False),
                    "help": props.get("help"),
                    "virtualenv": props.get("virtualenv", True),
                    "install": props.get("install", []),
                    "requirements": [os.path.join(base_dir, item)
                                     for item in props.get("requirements", [])]
                }
    return rv


def parse_args(argv, commands = load_commands()):
    parser = argparse.ArgumentParser()
    parser.add_argument("--venv", action="store", help="Path to an existing virtualenv to use")
    parser.add_argument("--skip-venv-setup", action="store_true",
                        dest="skip_venv_setup",
                        help="Whether to use the virtualenv as-is. Must set --venv as well")
    parser.add_argument("--debug", action="store_true", help="Run the debugger in case of an exception")
    parser.add_argument("--py3", action="store_true", help="Run with python3")
    subparsers = parser.add_subparsers(dest="command")
    for command, props in iteritems(commands):
        subparsers.add_parser(command, help=props["help"], add_help=False)

    args, extra = parser.parse_known_args(argv)

    return args, extra


def import_command(prog, command, props):
    # This currently requires the path to be a module,
    # which probably isn't ideal but it means that relative
    # imports inside the script work
    rel_path = os.path.relpath(props["path"], wpt_root)

    parts = os.path.splitext(rel_path)[0].split(os.path.sep)

    mod_name = ".".join(parts)

    mod = __import__(mod_name)
    for part in parts[1:]:
        mod = getattr(mod, part)

    script = getattr(mod, props["script"])
    if props["parser"] is not None:
        parser = getattr(mod, props["parser"])()
        parser.prog = "%s %s" % (os.path.basename(prog), command)
    else:
        parser = None

    return script, parser


def create_complete_parser():
    """Eagerly load all subparsers. This involves more work than is required
    for typical command-line usage. It is maintained for the purposes of
    documentation generation as implemented in WPT's top-level `/docs`
    directory."""

    commands = load_commands()
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers()

    for command in commands:
        props = commands[command]

        if props["virtualenv"]:
            setup_virtualenv(None, False, props)

        subparser = import_command('wpt', command, props)[1]
        if not subparser:
            continue

        subparsers.add_parser(command,
                              help=props["help"],
                              add_help=False,
                              parents=[subparser])

    return parser


def setup_virtualenv(path, skip_venv_setup, props):
    if skip_venv_setup and path is None:
        raise ValueError("Must set --venv when --skip-venv-setup is used")
    should_skip_setup = path is not None and skip_venv_setup
    if path is None:
        path = os.path.join(wpt_root, "_venv")
    venv = virtualenv.Virtualenv(path, should_skip_setup)
    if not should_skip_setup:
        venv.start()
        for name in props["install"]:
            venv.install(name)
        for path in props["requirements"]:
            venv.install_requirements(path)
    return venv


def main(prog=None, argv=None):
    logging.basicConfig(level=logging.INFO)

    if prog is None:
        prog = sys.argv[0]
    if argv is None:
        argv = sys.argv[1:]

    commands = load_commands()

    main_args, command_args = parse_args(argv, commands)

    command = main_args.command
    props = commands[command]
    venv = None
    if props["virtualenv"]:
        venv = setup_virtualenv(main_args.venv, main_args.skip_venv_setup, props)
    script, parser = import_command(prog, command, props)
    if parser:
        if props["parse_known"]:
            kwargs, extras = parser.parse_known_args(command_args)
            extras = (extras,)
            kwargs = vars(kwargs)
        else:
            extras = ()
            kwargs = vars(parser.parse_args(command_args))
    else:
        extras = ()
        kwargs = {}

    if venv is not None:
        args = (venv,) + extras
    else:
        args = extras

    if script:
        try:
            rv = script(*args, **kwargs)
            if rv is not None:
                sys.exit(int(rv))
        except Exception:
            if main_args.debug:
                import pdb
                pdb.post_mortem()
            else:
                raise
    sys.exit(0)


if __name__ == "__main__":
    main()

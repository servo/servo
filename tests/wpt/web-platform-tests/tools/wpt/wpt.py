# mypy: allow-untyped-defs

import argparse
import json
import logging
import multiprocessing
import os
import sys

from tools import localpaths  # noqa: F401

from . import virtualenv


here = os.path.dirname(__file__)
wpt_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))


def load_conditional_requirements(props, base_dir):
    """Load conditional requirements from commands.json."""

    conditional_requirements = props.get("conditional_requirements")
    if not conditional_requirements:
        return {}

    commandline_flag_requirements = {}
    for key, value in conditional_requirements.items():
        if key == "commandline_flag":
            for flag_name, requirements_paths in value.items():
                commandline_flag_requirements[flag_name] = [
                    os.path.join(base_dir, path) for path in requirements_paths]
        else:
            raise KeyError(
                f'Unsupported conditional requirement key: {key}')

    return {
        "commandline_flag": commandline_flag_requirements,
    }


def load_commands():
    rv = {}
    with open(os.path.join(here, "paths")) as f:
        paths = [item.strip().replace("/", os.path.sep) for item in f if item.strip()]
    for path in paths:
        abs_path = os.path.join(wpt_root, path, "commands.json")
        base_dir = os.path.dirname(abs_path)
        with open(abs_path) as f:
            data = json.load(f)
            for command, props in data.items():
                assert "path" in props
                assert "script" in props
                rv[command] = {
                    "path": os.path.join(base_dir, props["path"]),
                    "script": props["script"],
                    "parser": props.get("parser"),
                    "parse_known": props.get("parse_known", False),
                    "help": props.get("help"),
                    "virtualenv": props.get("virtualenv", True),
                    "requirements": [os.path.join(base_dir, item)
                                     for item in props.get("requirements", [])]
                }

                rv[command]["conditional_requirements"] = load_conditional_requirements(
                    props, base_dir)

                if rv[command]["requirements"] or rv[command]["conditional_requirements"]:
                    assert rv[command]["virtualenv"]
    return rv


def parse_args(argv, commands=load_commands()):
    parser = argparse.ArgumentParser()
    parser.add_argument("--venv", action="store", help="Path to an existing virtualenv to use")
    parser.add_argument("--skip-venv-setup", action="store_true",
                        dest="skip_venv_setup",
                        help="Whether to use the virtualenv as-is. Must set --venv as well")
    parser.add_argument("--debug", action="store_true", help="Run the debugger in case of an exception")
    subparsers = parser.add_subparsers(dest="command")
    for command, props in commands.items():
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
        parser.prog = f"{os.path.basename(prog)} {command}"
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

    # We should already be in a virtual environment from the top-level
    # `wpt build-docs` command but we need to look up the environment to
    # find out where it's located.
    venv_path = os.environ["VIRTUAL_ENV"]
    venv = virtualenv.Virtualenv(venv_path, True)

    for command in commands:
        props = commands[command]

        for path in props.get("requirements", []):
            venv.install_requirements(path)

        subparser = import_command('wpt', command, props)[1]
        if not subparser:
            continue

        subparsers.add_parser(command,
                              help=props["help"],
                              add_help=False,
                              parents=[subparser])

    return parser


def venv_dir():
    return f"_venv{sys.version_info[0]}"


def setup_virtualenv(path, skip_venv_setup, props):
    if skip_venv_setup and path is None:
        raise ValueError("Must set --venv when --skip-venv-setup is used")
    should_skip_setup = path is not None and skip_venv_setup
    if path is None:
        path = os.path.join(wpt_root, venv_dir())
    venv = virtualenv.Virtualenv(path, should_skip_setup)
    if not should_skip_setup:
        venv.start()
        for path in props["requirements"]:
            venv.install_requirements(path)
    return venv


def install_command_flag_requirements(venv, kwargs, requirements):
    for command_flag_name, requirement_paths in requirements.items():
        if command_flag_name in kwargs:
            for path in requirement_paths:
                venv.install_requirements(path)


def main(prog=None, argv=None):
    logging.basicConfig(level=logging.INFO)
    # Ensure we use the spawn start method for all multiprocessing
    try:
        multiprocessing.set_start_method('spawn')
    except RuntimeError as e:
        # This can happen if we call back into wpt having already set the context
        start_method = multiprocessing.get_start_method()
        if start_method != "spawn":
            logging.critical("The multiprocessing start method was set to %s by a caller", start_method)
            raise e

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
        requirements = props["conditional_requirements"].get("commandline_flag")
        if requirements is not None and not main_args.skip_venv_setup:
            install_command_flag_requirements(venv, kwargs, requirements)
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
    main()  # type: ignore

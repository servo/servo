import sys

from mozlog.structured import structuredlog, commandline

from .. import wptcommandline

from .update import WPTUpdate

def remove_logging_args(args):
    """Take logging args out of the dictionary of command line arguments so
    they are not passed in as kwargs to the update code. This is particularly
    necessary here because the arguments are often of type file, which cannot
    be serialized.

    :param args: Dictionary of command line arguments.
    """
    for name in list(args.keys()):
        if name.startswith("log_"):
            args.pop(name)


def setup_logging(args, defaults):
    """Use the command line arguments to set up the logger.

    :param args: Dictionary of command line arguments.
    :param defaults: Dictionary of {formatter_name: stream} to use if
                     no command line logging is specified"""
    logger = commandline.setup_logging("web-platform-tests-update", args, defaults)

    remove_logging_args(args)

    return logger


def run_update(logger, **kwargs):
    updater = WPTUpdate(logger, **kwargs)
    return updater.run()


def main():
    args = wptcommandline.parse_args_update()
    logger = setup_logging(args, {"mach": sys.stdout})
    assert structuredlog.get_default_logger() is not None
    success = run_update(logger, **args)
    sys.exit(0 if success else 1)

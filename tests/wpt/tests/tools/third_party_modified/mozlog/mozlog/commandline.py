# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import optparse
import os
import sys
from collections import defaultdict

from . import formatters, handlers
from .structuredlog import StructuredLogger, set_default_logger

log_formatters = {
    "raw": (
        formatters.JSONFormatter,
        "Raw structured log messages " "(provided by mozlog)",
    ),
    "unittest": (
        formatters.UnittestFormatter,
        "Unittest style output " "(provided by mozlog)",
    ),
    "xunit": (
        formatters.XUnitFormatter,
        "xUnit compatible XML " "(provided by mozlog)",
    ),
    "html": (formatters.HTMLFormatter, "HTML report " "(provided by mozlog)"),
    "mach": (formatters.MachFormatter, "Human-readable output " "(provided by mozlog)"),
    "tbpl": (formatters.TbplFormatter, "TBPL style log format " "(provided by mozlog)"),
    "grouped": (
        formatters.GroupingFormatter,
        "Grouped summary of test results " "(provided by mozlog)",
    ),
    "errorsummary": (formatters.ErrorSummaryFormatter, argparse.SUPPRESS),
}

TEXT_FORMATTERS = ("raw", "mach")
"""a subset of formatters for non test harnesses related applications"""


DOCS_URL = "https://firefox-source-docs.mozilla.org/mozbase/mozlog.html"


def level_filter_wrapper(formatter, level):
    return handlers.LogLevelFilter(formatter, level)


def verbose_wrapper(formatter, verbose):
    formatter.verbose = verbose
    return formatter


def compact_wrapper(formatter, compact):
    formatter.compact = compact
    return formatter


def buffer_handler_wrapper(handler, buffer_limit):
    if buffer_limit == "UNLIMITED":
        buffer_limit = None
    else:
        buffer_limit = int(buffer_limit)
    return handlers.BufferHandler(handler, buffer_limit)


def screenshot_wrapper(formatter, enable_screenshot):
    formatter.enable_screenshot = enable_screenshot
    return formatter


def valgrind_handler_wrapper(handler):
    return handlers.ValgrindHandler(handler)


def default_formatter_options(log_type, overrides):
    formatter_option_defaults = {"raw": {"level": "debug"}}
    rv = {"verbose": False, "level": "info"}
    rv.update(formatter_option_defaults.get(log_type, {}))

    if overrides is not None:
        rv.update(overrides)

    return rv


fmt_options = {
    # <option name>: (<wrapper function>, description, <applicable formatters>, action)
    # "action" is used by the commandline parser in use.
    "verbose": (
        verbose_wrapper,
        "Enables verbose mode for the given formatter.",
        {"mach"},
        "store_true",
    ),
    "compact": (
        compact_wrapper,
        "Enables compact mode for the given formatter.",
        {"tbpl"},
        "store_true",
    ),
    "level": (
        level_filter_wrapper,
        "A least log level to subscribe to for the given formatter "
        "(debug, info, error, etc.)",
        {"mach", "raw", "tbpl"},
        "store",
    ),
    "buffer": (
        buffer_handler_wrapper,
        "If specified, enables message buffering at the given buffer size limit.",
        ["mach", "tbpl"],
        "store",
    ),
    "screenshot": (
        screenshot_wrapper,
        "Enable logging reftest-analyzer compatible screenshot data.",
        {"mach"},
        "store_true",
    ),
    "no-screenshot": (
        screenshot_wrapper,
        "Disable logging reftest-analyzer compatible screenshot data.",
        {"mach"},
        "store_false",
    ),
}


def log_file(name):
    if name == "-":
        return sys.stdout
    # ensure we have a correct dirpath by using realpath
    dirpath = os.path.dirname(os.path.realpath(name))
    if not os.path.exists(dirpath):
        os.makedirs(dirpath)
    return open(name, "w")


def add_logging_group(parser, include_formatters=None):
    """
    Add logging options to an argparse ArgumentParser or
    optparse OptionParser.

    Each formatter has a corresponding option of the form --log-{name}
    where {name} is the name of the formatter. The option takes a value
    which is either a filename or "-" to indicate stdout.

    :param parser: The ArgumentParser or OptionParser object that should have
                   logging options added.
    :param include_formatters: List of formatter names that should be included
                               in the option group. Default to None, meaning
                               all the formatters are included. A common use
                               of this option is to specify
                               :data:`TEXT_FORMATTERS` to include only the
                               most useful formatters for a command line tool
                               that is not related to test harnesses.
    """
    group_name = "Output Logging"
    group_description = (
        "Each option represents a possible logging format "
        "and takes a filename to write that format to, "
        "or '-' to write to stdout. Some options are "
        "provided by the mozlog utility; see %s "
        "for extended documentation." % DOCS_URL
    )

    if include_formatters is None:
        include_formatters = list(log_formatters.keys())

    if isinstance(parser, optparse.OptionParser):
        group = optparse.OptionGroup(parser, group_name, group_description)
        parser.add_option_group(group)
        opt_log_type = "str"
        group_add = group.add_option
    else:
        group = parser.add_argument_group(group_name, group_description)
        opt_log_type = log_file
        group_add = group.add_argument

    for name, (cls, help_str) in log_formatters.items():
        if name in include_formatters:
            group_add(
                "--log-" + name, action="append", type=opt_log_type, help=help_str
            )

    for fmt in include_formatters:
        for optname, (cls, help_str, formatters_, action) in fmt_options.items():
            if fmt not in formatters_:
                continue
            if optname.startswith("no-") and action == "store_false":
                dest = optname.split("-", 1)[1]
            else:
                dest = optname
            dest = dest.replace("-", "_")
            group_add(
                "--log-%s-%s" % (fmt, optname),
                action=action,
                help=help_str,
                default=None,
                dest="log_%s_%s" % (fmt, dest),
            )


def setup_handlers(logger, formatters, formatter_options, allow_unused_options=False):
    """
    Add handlers to the given logger according to the formatters and
    options provided.

    :param logger: The logger configured by this function.
    :param formatters: A dict of {formatter, [streams]} to use in handlers.
    :param formatter_options: a dict of {formatter: {option: value}} to
                              to use when configuring formatters.
    """
    unused_options = set(formatter_options.keys()) - set(formatters.keys())
    if unused_options and not allow_unused_options:
        msg = "Options specified for unused formatter(s) (%s) have no effect" % list(
            unused_options
        )
        raise ValueError(msg)

    for fmt, streams in formatters.items():
        formatter_cls = log_formatters[fmt][0]
        formatter = formatter_cls()
        handler_wrappers_and_options = []

        for option, value in formatter_options[fmt].items():
            wrapper, wrapper_args = None, ()
            if option == "valgrind":
                wrapper = valgrind_handler_wrapper
            elif option == "buffer":
                wrapper, wrapper_args = fmt_options[option][0], (value,)
            else:
                formatter = fmt_options[option][0](formatter, value)

            if wrapper is not None:
                handler_wrappers_and_options.append((wrapper, wrapper_args))

        for value in streams:
            handler = handlers.StreamHandler(stream=value, formatter=formatter)
            for wrapper, wrapper_args in handler_wrappers_and_options:
                handler = wrapper(handler, *wrapper_args)
            logger.add_handler(handler)


def setup_logging(
    logger, args, defaults=None, formatter_defaults=None, allow_unused_options=False
):
    """
    Configure a structuredlogger based on command line arguments.

    The created structuredlogger will also be set as the default logger, and
    can be retrieved with :py:func:`~mozlog.get_default_logger`.

    :param logger: A StructuredLogger instance or string name. If a string, a
                   new StructuredLogger instance will be created using
                   `logger` as the name.
    :param args: A dictionary of {argument_name:value} produced from
                 parsing the command line arguments for the application
    :param defaults: A dictionary of {formatter name: output stream} to apply
                     when there is no logging supplied on the command line. If
                     this isn't supplied, reasonable defaults are chosen
                     (coloured mach formatting if stdout is a terminal, or raw
                     logs otherwise).
    :param formatter_defaults: A dictionary of {option_name: default_value} to provide
                               to the formatters in the absence of command line overrides.
    :rtype: StructuredLogger
    """

    if not isinstance(logger, StructuredLogger):
        logger = StructuredLogger(logger)
        # The likely intent when using this function is to get a brand new
        # logger, so reset state in case it was previously initialized.
        logger.reset_state()

    # Keep track of any options passed for formatters.
    formatter_options = {}
    # Keep track of formatters and list of streams specified.
    formatters = defaultdict(list)
    found = False
    found_stdout_logger = False
    if args is None:
        args = {}
    if not isinstance(args, dict):
        args = vars(args)

    if defaults is None:
        if sys.__stdout__.isatty():
            defaults = {"mach": sys.stdout}
        else:
            defaults = {"raw": sys.stdout}

    for name, values in args.items():
        parts = name.split("_")
        if len(parts) > 3:
            continue
        # Our args will be ['log', <formatter>]
        #               or ['log', <formatter>, <option>]
        #               or ['valgrind']
        if parts[0] == "log" and values is not None:
            if len(parts) == 1 or parts[1] not in log_formatters:
                continue
            if len(parts) == 2:
                _, formatter = parts
                for value in values:
                    found = True
                    if isinstance(value, str):
                        value = log_file(value)
                    if value == sys.stdout:
                        found_stdout_logger = True
                    formatters[formatter].append(value)
            if len(parts) == 3:
                _, formatter, opt = parts
                if formatter not in formatter_options:
                    formatter_options[formatter] = default_formatter_options(
                        formatter, formatter_defaults
                    )
                formatter_options[formatter][opt] = values

    # If there is no user-specified logging, go with the default options
    if not found:
        for name, value in defaults.items():
            formatters[name].append(value)

    elif not found_stdout_logger and sys.stdout in list(defaults.values()):
        for name, value in defaults.items():
            if value == sys.stdout:
                formatters[name].append(value)

    for name in formatters:
        if name not in formatter_options:
            formatter_options[name] = default_formatter_options(
                name, formatter_defaults
            )

    # If the user specified --valgrind, add it as an option for all formatters
    if args.get("valgrind", None) is not None:
        for name in formatters:
            formatter_options[name]["valgrind"] = True
    setup_handlers(logger, formatters, formatter_options, allow_unused_options)
    set_default_logger(logger)

    return logger

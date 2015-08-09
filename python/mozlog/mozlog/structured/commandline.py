# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
import os
import optparse

from collections import defaultdict
from structuredlog import StructuredLogger, set_default_logger
import handlers
import formatters

log_formatters = {
    'raw': (formatters.JSONFormatter, "Raw structured log messages"),
    'unittest': (formatters.UnittestFormatter, "Unittest style output"),
    'xunit': (formatters.XUnitFormatter, "xUnit compatible XML"),
    'html': (formatters.HTMLFormatter, "HTML report"),
    'mach': (formatters.MachFormatter, "Human-readable output"),
    'tbpl': (formatters.TbplFormatter, "TBPL style log format"),
}

TEXT_FORMATTERS = ('raw', 'mach')
"""a subset of formatters for non test harnesses related applications"""

def level_filter_wrapper(formatter, level):
    return handlers.LogLevelFilter(formatter, level)

def verbose_wrapper(formatter, verbose):
    formatter.verbose = verbose
    return formatter

def buffer_handler_wrapper(handler, buffer_limit):
    if buffer_limit == "UNLIMITED":
        buffer_limit = None
    else:
        buffer_limit = int(buffer_limit)
    return handlers.BufferingLogFilter(handler, buffer_limit)

formatter_option_defaults = {
    'verbose': False,
    'level': 'info',
}

fmt_options = {
    # <option name>: (<wrapper function>, description, <applicable formatters>, action)
    # "action" is used by the commandline parser in use.
    'verbose': (verbose_wrapper,
                "Enables verbose mode for the given formatter.",
                ["mach"], "store_true"),
    'level': (level_filter_wrapper,
              "A least log level to subscribe to for the given formatter (debug, info, error, etc.)",
              ["mach", "tbpl"], "store"),
    'buffer': (buffer_handler_wrapper,
               "If specified, enables message buffering at the given buffer size limit.",
               ["mach", "tbpl"], "store"),
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
    group_description = ("Each option represents a possible logging format "
                         "and takes a filename to write that format to, "
                         "or '-' to write to stdout.")

    if include_formatters is None:
        include_formatters = log_formatters.keys()

    if isinstance(parser, optparse.OptionParser):
        group = optparse.OptionGroup(parser,
                                     group_name,
                                     group_description)
        parser.add_option_group(group)
        opt_log_type = 'str'
        group_add = group.add_option
    else:
        group = parser.add_argument_group(group_name,
                                          group_description)
        opt_log_type = log_file
        group_add = group.add_argument

    for name, (cls, help_str) in log_formatters.iteritems():
        if name in include_formatters:
            group_add("--log-" + name, action="append", type=opt_log_type,
                      help=help_str)

    for optname, (cls, help_str, formatters, action) in fmt_options.iteritems():
        for fmt in formatters:
            # make sure fmt is in log_formatters and is accepted
            if fmt in log_formatters and fmt in include_formatters:
                group_add("--log-%s-%s" % (fmt, optname), action=action,
                          help=help_str, default=None)


def setup_handlers(logger, formatters, formatter_options):
    """
    Add handlers to the given logger according to the formatters and
    options provided.

    :param logger: The logger configured by this function.
    :param formatters: A dict of {formatter, [streams]} to use in handlers.
    :param formatter_options: a dict of {formatter: {option: value}} to
                              to use when configuring formatters.
    """
    unused_options = set(formatter_options.keys()) - set(formatters.keys())
    if unused_options:
        msg = ("Options specified for unused formatter(s) (%s) have no effect" %
               list(unused_options))
        raise ValueError(msg)

    for fmt, streams in formatters.iteritems():
        formatter_cls = log_formatters[fmt][0]
        formatter = formatter_cls()
        handler_wrapper, handler_option = None, ""
        for option, value in formatter_options[fmt].iteritems():
            if option == "buffer":
                handler_wrapper, handler_option = fmt_options[option][0], value
            else:
                formatter = fmt_options[option][0](formatter, value)

        for value in streams:
            handler = handlers.StreamHandler(stream=value, formatter=formatter)
            if handler_wrapper:
                handler = handler_wrapper(handler, handler_option)
            logger.add_handler(handler)


def setup_logging(suite, args, defaults=None):
    """
    Configure a structuredlogger based on command line arguments.

    The created structuredlogger will also be set as the default logger, and
    can be retrieved with :py:func:`~mozlog.structured.structuredlog.get_default_logger`.

    :param suite: The name of the testsuite being run
    :param args: A dictionary of {argument_name:value} produced from
                 parsing the command line arguments for the application
    :param defaults: A dictionary of {formatter name: output stream} to apply
                     when there is no logging supplied on the command line. If
                     this isn't supplied, reasonable defaults are chosen
                     (coloured mach formatting if stdout is a terminal, or raw
                     logs otherwise).

    :rtype: StructuredLogger
    """

    logger = StructuredLogger(suite)
    # Keep track of any options passed for formatters.
    formatter_options = defaultdict(lambda: formatter_option_defaults.copy())
    # Keep track of formatters and list of streams specified.
    formatters = defaultdict(list)
    found = False
    found_stdout_logger = False
    if not hasattr(args, 'iteritems'):
        args = vars(args)

    if defaults is None:
        if sys.__stdout__.isatty():
            defaults = {"mach": sys.stdout}
        else:
            defaults = {"raw": sys.stdout}

    for name, values in args.iteritems():
        parts = name.split('_')
        if len(parts) > 3:
            continue
        # Our args will be ['log', <formatter>] or ['log', <formatter>, <option>].
        if parts[0] == 'log' and values is not None:
            if len(parts) == 1 or parts[1] not in log_formatters:
                continue
            if len(parts) == 2:
                _, formatter = parts
                for value in values:
                    found = True
                    if isinstance(value, basestring):
                        value = log_file(value)
                    if value == sys.stdout:
                        found_stdout_logger = True
                    formatters[formatter].append(value)
            if len(parts) == 3:
                _, formatter, opt = parts
                formatter_options[formatter][opt] = values

    #If there is no user-specified logging, go with the default options
    if not found:
        for name, value in defaults.iteritems():
            formatters[name].append(value)

    elif not found_stdout_logger and sys.stdout in defaults.values():
        for name, value in defaults.iteritems():
            if value == sys.stdout:
                formatters[name].append(value)

    setup_handlers(logger, formatters, formatter_options)
    set_default_logger(logger)

    return logger

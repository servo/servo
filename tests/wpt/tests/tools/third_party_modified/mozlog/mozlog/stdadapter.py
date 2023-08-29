# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import logging

from .structuredlog import StructuredLogger, log_levels


class UnstructuredHandler(logging.Handler):
    def __init__(self, name=None, level=logging.NOTSET):
        self.structured = StructuredLogger(name)
        logging.Handler.__init__(self, level=level)

    def emit(self, record):
        if record.levelname in log_levels:
            log_func = getattr(self.structured, record.levelname.lower())
        else:
            log_func = self.logger.debug
        log_func(record.msg)

    def handle(self, record):
        self.emit(record)


class LoggingWrapper(object):
    def __init__(self, wrapped):
        self.wrapped = wrapped
        self.wrapped.addHandler(
            UnstructuredHandler(
                self.wrapped.name, logging.getLevelName(self.wrapped.level)
            )
        )

    def add_handler(self, handler):
        self.addHandler(handler)

    def remove_handler(self, handler):
        self.removeHandler(handler)

    def __getattr__(self, name):
        return getattr(self.wrapped, name)


def std_logging_adapter(logger):
    """Adapter for stdlib logging so that it produces structured
    messages rather than standard logging messages

    :param logger: logging.Logger to wrap"""
    return LoggingWrapper(logger)

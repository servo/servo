# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from .logger import Logger, getLogger


class LoggingMixin(object):
    """Expose a subset of logging functions to an inheriting class."""

    def set_logger(self, logger_instance=None, name=None):
        """Method for setting the underlying logger instance to be used."""

        if logger_instance and not isinstance(logger_instance, Logger):
            raise ValueError("logger_instance must be an instance of Logger")

        if name is None:
            name = ".".join([self.__module__, self.__class__.__name__])

        self._logger = logger_instance or getLogger(name)

    def _log_msg(self, cmd, *args, **kwargs):
        if not hasattr(self, "_logger"):
            self._logger = getLogger(
                ".".join([self.__module__, self.__class__.__name__])
            )
        getattr(self._logger, cmd)(*args, **kwargs)

    def log(self, *args, **kwargs):
        self._log_msg("log", *args, **kwargs)

    def info(self, *args, **kwargs):
        self._log_msg("info", *args, **kwargs)

    def error(self, *args, **kwargs):
        self._log_msg("error", *args, **kwargs)

    def warn(self, *args, **kwargs):
        self._log_msg("warn", *args, **kwargs)

    def log_structured(self, *args, **kwargs):
        self._log_msg("log_structured", *args, **kwargs)

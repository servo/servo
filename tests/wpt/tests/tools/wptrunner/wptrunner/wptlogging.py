# mypy: allow-untyped-defs

import logging
from threading import Thread
from types import TracebackType
from typing import Optional, Type

from mozlog import commandline, stdadapter, set_default_logger
from mozlog.structuredlog import StructuredLogger, log_levels


class LoggerManager:
    def __init__(self, args, defaults, formatter_defaults=None):
        self._logger = None
        self._owns_logger = False
        self._args = args
        self._defaults = defaults
        self._formatter_defaults = formatter_defaults

    def __enter__(self):
        self._logger = self._args.pop('log', None)
        if self._logger is not None:
            set_default_logger(self._logger)
            StructuredLogger._logger_states["web-platform-tests"] = self._logger._state
        else:
            self._logger = commandline.setup_logging("web-platform-tests", self._args, self._defaults,
                                                     formatter_defaults=self._formatter_defaults)
            self._owns_logger = True
        setup_stdlib_logger()

        for name in list(self._args.keys()):
            if name.startswith("log_"):
                self._args.pop(name)

        return self._logger

    def __exit__(self, *args, **kwargs):
        if self._logger is None:
            return

        if self._owns_logger:
            self._logger.shutdown()
        self._logger = None


def setup(args, defaults, formatter_defaults=None):
    # Legacy entry point
    return LoggerManager(args, defaults, formatter_defaults).__enter__()


def setup_stdlib_logger():
    logging.root.handlers = []
    logging.root = stdadapter.std_logging_adapter(logging.root)


class LogLevelRewriter:
    """Filter that replaces log messages at specified levels with messages
    at a different level.

    This can be used to e.g. downgrade log messages from ERROR to WARNING
    in some component where ERRORs are not critical.

    :param inner: Handler to use for messages that pass this filter
    :param from_levels: List of levels which should be affected
    :param to_level: Log level to set for the affected messages
    """
    def __init__(self, inner, from_levels, to_level):
        self.inner = inner
        self.from_levels = [item.upper() for item in from_levels]
        self.to_level = to_level.upper()

    def __call__(self, data):
        if data["action"] == "log" and data["level"].upper() in self.from_levels:
            data = data.copy()
            data["level"] = self.to_level
        return self.inner(data)


class LoggedAboveLevelHandler:
    """Filter that records whether any log message above a certain level has been
    seen.

    :param min_level: Minimum level to record as a str (e.g., "CRITICAL")

    """
    def __init__(self, min_level):
        self.min_level = log_levels[min_level.upper()]
        self.has_log = False

    def __call__(self, data):
        if (data["action"] == "log" and
            not self.has_log and
            log_levels[data["level"]] <= self.min_level):
            self.has_log = True


class QueueHandler(logging.Handler):
    def __init__(self, queue, level=logging.NOTSET):
        self.queue = queue
        logging.Handler.__init__(self, level=level)

    def createLock(self):
        # The queue provides its own locking
        self.lock = NullRLock()

    def emit(self, record):
        msg = self.format(record)
        data = {"action": "log",
                "level": record.levelname,
                "thread": record.threadName,
                "pid": record.process,
                "source": self.name,
                "message": msg}
        self.queue.put(data)



class NullRLock:
    """Implementation of the threading.RLock API that doesn't actually acquire a lock,
    for use in cases where there is another mechanism to provide the required
    invariants."""

    def acquire(self, blocking: bool = True, timeout: float = -1) -> bool:
        return True

    def release(self) -> None:
        return None

    def __enter__(self) -> bool:
        return True

    def __exit__(self, t: Optional[Type[BaseException]], v: Optional[BaseException], tb: Optional[TracebackType]) -> None:
        return None


class LogQueueThread(Thread):
    """Thread for handling log messages from a queue"""
    def __init__(self, queue, logger):
        self.queue = queue
        self.logger = logger
        super().__init__(name="Thread-Log")

    def run(self):
        while True:
            try:
                data = self.queue.get()
            except (EOFError, OSError):
                break
            if data is None:
                # A None message is used to shut down the logging thread
                break
            self.logger.log_raw(data)

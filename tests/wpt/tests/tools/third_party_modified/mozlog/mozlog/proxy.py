# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from threading import Thread

from .structuredlog import StructuredLogger, get_default_logger


class ProxyLogger(object):
    """
    A ProxyLogger behaves like a
    :class:`mozlog.structuredlog.StructuredLogger`.

    Each method and attribute access will be forwarded to the underlying
    StructuredLogger.

    RuntimeError will be raised when the default logger is not yet initialized.
    """

    def __init__(self, component=None):
        self.logger = None
        self._component = component

    def __getattr__(self, name):
        if self.logger is None:
            self.logger = get_default_logger(component=self._component)
            if self.logger is None:
                raise RuntimeError("Default logger is not initialized!")
        return getattr(self.logger, name)


def get_proxy_logger(component=None):
    """
    Returns a :class:`ProxyLogger` for the given component.
    """
    return ProxyLogger(component)


class QueuedProxyLogger(StructuredLogger):
    """Logger that logs via a queue.

    This is intended for multiprocessing use cases where there are
    some subprocesses which want to share a log handler with the main thread,
    without the overhead of having a multiprocessing lock for all logger
    access."""

    threads = {}

    def __init__(self, logger, mp_context=None):
        StructuredLogger.__init__(self, logger.name)

        if mp_context is None:
            import multiprocessing as mp_context

        if logger.name not in self.threads:
            self.threads[logger.name] = LogQueueThread(mp_context.Queue(), logger)
            self.threads[logger.name].start()
        self.queue = self.threads[logger.name].queue

    def _handle_log(self, data):
        self.queue.put(data)


class LogQueueThread(Thread):
    def __init__(self, queue, logger):
        self.queue = queue
        self.logger = logger
        Thread.__init__(self, name="Thread-Log")
        self.daemon = True

    def run(self):
        while True:
            try:
                msg = self.queue.get()
            except (EOFError, IOError):
                break
            if msg is None:
                break
            else:
                self.logger._handle_log(msg)

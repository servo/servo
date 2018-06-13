import logging
import sys
import threading
from StringIO import StringIO
from multiprocessing import Queue

from mozlog import commandline, stdadapter, set_default_logger
from mozlog.structuredlog import StructuredLogger

def setup(args, defaults):
    logger = args.pop('log', None)
    if logger:
        set_default_logger(logger)
        StructuredLogger._logger_states["web-platform-tests"] = logger._state
    else:
        logger = commandline.setup_logging("web-platform-tests", args, defaults)
    setup_stdlib_logger()

    for name in args.keys():
        if name.startswith("log_"):
            args.pop(name)

    return logger


def setup_stdlib_logger():
    logging.root.handlers = []
    logging.root = stdadapter.std_logging_adapter(logging.root)


class LogLevelRewriter(object):
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


class LogThread(threading.Thread):
    def __init__(self, queue, logger, level):
        self.queue = queue
        self.log_func = getattr(logger, level)
        threading.Thread.__init__(self, name="Thread-Log")
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
                self.log_func(msg)


class LoggingWrapper(StringIO):
    """Wrapper for file like objects to redirect output to logger
    instead"""

    def __init__(self, queue, prefix=None):
        StringIO.__init__(self)
        self.queue = queue
        self.prefix = prefix

    def write(self, data):
        if isinstance(data, str):
            try:
                data = data.decode("utf8")
            except UnicodeDecodeError:
                data = data.encode("string_escape").decode("ascii")

        if data.endswith("\n"):
            data = data[:-1]
        if data.endswith("\r"):
            data = data[:-1]
        if not data:
            return
        if self.prefix is not None:
            data = "%s: %s" % (self.prefix, data)
        self.queue.put(data)

    def flush(self):
        pass


class CaptureIO(object):
    def __init__(self, logger, do_capture):
        self.logger = logger
        self.do_capture = do_capture
        self.logging_queue = None
        self.logging_thread = None
        self.original_stdio = None

    def __enter__(self):
        if self.do_capture:
            self.original_stdio = (sys.stdout, sys.stderr)
            self.logging_queue = Queue()
            self.logging_thread = LogThread(self.logging_queue, self.logger, "info")
            sys.stdout = LoggingWrapper(self.logging_queue, prefix="STDOUT")
            sys.stderr = LoggingWrapper(self.logging_queue, prefix="STDERR")
            self.logging_thread.start()

    def __exit__(self, *args, **kwargs):
        if self.do_capture:
            sys.stdout, sys.stderr = self.original_stdio
            if self.logging_queue is not None:
                self.logger.info("Closing logging queue")
                self.logging_queue.put(None)
                if self.logging_thread is not None:
                    self.logging_thread.join(10)
                while not self.logging_queue.empty():
                    try:
                        self.logger.warning("Dropping log message: %r", self.logging_queue.get())
                    except Exception:
                        pass
                self.logging_queue.close()
                self.logger.info("queue closed")

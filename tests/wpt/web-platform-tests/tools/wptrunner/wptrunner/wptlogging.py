import logging
import sys
import threading
from StringIO import StringIO
from multiprocessing import Queue

from mozlog import commandline, get_default_logger, stdadapter, set_default_logger
from mozlog.structuredlog import StructuredLogger, log_levels


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
    def __init__(self, queue, logger):
        assert logger.component is None
        self.queue = queue
        self.loggers = {None: logger}
        threading.Thread.__init__(self, name="Thread-Log")
        self.daemon = True

    def get_logger(self, component=None):
        if component not in self.loggers:
            name = self.loggers[None].name
            self.loggers[component] = StructuredLogger(name, component)
        return self.loggers[component]

    def run(self):
        while True:
            try:
                command = self.queue.get()
            except (EOFError, IOError):
                break
            if command is None:
                break
            else:
                component, level, msg = command
                getattr(self.get_logger(component), level)(msg)


class FileLogging(StringIO):
    """Wrapper for file like objects to redirect output to logger
    instead"""

    def __init__(self, queue, prefix=None, level="info"):
        StringIO.__init__(self)
        self.queue = queue
        self.prefix = prefix
        self.level = level

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
        self.queue.put((None, self.level, data))

    def flush(self):
        pass


class CaptureIO(object):
    def __init__(self, queue, do_capture):
        self.logging_queue = queue
        self.do_capture = do_capture
        self.original_stdio = None

    def __enter__(self):
        if self.do_capture:
            self.original_stdio = (sys.stdout, sys.stderr)
            sys.stdout = FileLogging(self.logging_queue, prefix="STDOUT")
            sys.stderr = FileLogging(self.logging_queue, prefix="STDERR")

    def __exit__(self, *args, **kwargs):
        if self.do_capture:
            sys.stdout, sys.stderr = self.original_stdio


class StdQueueHandler(logging.Handler):
    def __init__(self, queue, component=None, level=logging.NOTSET):
        super(StdQueueHandler, self).__init__(level=level)
        self.component = component
        self.queue = queue

    def emit(self, record):
        if record.levelname in log_levels:
            level = record.levelname.lower()
        else:
            level = "debug"
        msg = record.msg
        if record.args:
            msg = msg % record.args
        self.queue.put((self.component, level, msg))

    def handle(self, record):
        self.emit(record)


class LoggingQueue(object):
    def __init__(self, logger):
        self.logger = logger
        self.queue = Queue()
        self.thread = None

    def __enter__(self):
        self.thread = LogThread(self.queue, self.logger)
        self.thread.start()
        return self.queue

    def __exit__(self, *args, **kwargs):
        self.logger.info("Closing logging queue")
        self.queue.put(None)
        if self.thread is not None:
            self.thread.join(10)
        while not self.queue.empty():
            try:
                self.logger.warning("Dropping log message: %r", self.queue.get())
            except Exception:
                pass
        self.queue.close()
        self.logger.info("queue closed")

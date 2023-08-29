# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
import threading
from io import BytesIO


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


class LoggingWrapper(BytesIO):
    """Wrapper for file like objects to redirect output to logger
    instead"""

    def __init__(self, queue, prefix=None):
        BytesIO.__init__(self)
        self.queue = queue
        self.prefix = prefix
        self.buffer = self

    def write(self, data):
        if isinstance(data, bytes):
            try:
                data = data.decode("utf8")
            except UnicodeDecodeError:
                data = data.decode("unicode_escape")

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
    def __init__(self, logger, do_capture, mp_context=None):
        if mp_context is None:
            import multiprocessing as mp_context
        self.logger = logger
        self.do_capture = do_capture
        self.logging_queue = None
        self.logging_thread = None
        self.original_stdio = None
        self.mp_context = mp_context

    def __enter__(self):
        if self.do_capture:
            self.original_stdio = (sys.stdout, sys.stderr)
            self.logging_queue = self.mp_context.Queue()
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
                        self.logger.warning(
                            "Dropping log message: %r", self.logging_queue.get()
                        )
                    except Exception:
                        pass
                self.logging_queue.close()
                self.logger.info("queue closed")

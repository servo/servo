# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from threading import Lock
import codecs

from ..structuredlog import log_levels


class BaseHandler(object):
    """A base handler providing message handling facilities to
    derived classes.

    :param inner: A handler-like callable that may receive messages
                  from a log user.
    """

    def __init__(self, inner):
        self.wrapped = []
        if hasattr(inner, "handle_message"):
            self.wrapped.append(inner)
        self.message_handlers = {}

    def register_message_handlers(self, topic, handlers):
        self.message_handlers[topic] = handlers

    def handle_message(self, topic, cmd, *args):
        """Handles a message for the given topic by calling a subclass-defined
        callback for the command.

        :param topic: The topic of the broadcasted message. Handlers opt-in to
                      receiving messages by identifying a topic when calling
                      register_message_handlers.
        :param command: The command to issue. This is a string that corresponds
                        to a callback provided by the target.
        :param arg: Arguments to pass to the identified message callback, if any.
        """
        rv = []
        if topic in self.message_handlers and cmd in self.message_handlers[topic]:
            rv.append(self.message_handlers[topic][cmd](*args))
        for inner in self.wrapped:
            rv.extend(inner.handle_message(topic, cmd, *args))
        return rv


class LogLevelFilter(BaseHandler):
    """Handler that filters out messages with action of log and a level
    lower than some specified level.

    :param inner: Handler to use for messages that pass this filter
    :param level: Minimum log level to process
    """
    def __init__(self, inner, level):
        BaseHandler.__init__(self, inner)
        self.inner = inner
        self.level = log_levels[level.upper()]

    def __call__(self, item):
        if (item["action"] != "log" or
            log_levels[item["level"].upper()] <= self.level):
            return self.inner(item)


class StreamHandler(BaseHandler):
    """Handler for writing to a file-like object

    :param stream: File-like object to write log messages to
    :param formatter: formatter to convert messages to string format
    """

    _lock = Lock()

    def __init__(self, stream, formatter):
        BaseHandler.__init__(self, formatter)
        assert stream is not None
        # This is a hack to deal with the case where we are passed a
        # StreamWriter (e.g. by mach for stdout). A StreamWriter requires
        # the code to handle unicode in exactly the opposite way compared
        # to a normal stream i.e. you always have to pass in a Unicode
        # object rather than a string object. Cope with that by extracting
        # the underlying raw stream.
        if isinstance(stream, codecs.StreamWriter):
            stream = stream.stream

        self.formatter = formatter
        self.stream = stream

    def __call__(self, data):
        """Write a log message.

        :param data: Structured log message dictionary."""
        formatted = self.formatter(data)
        if not formatted:
            return
        with self._lock:
            if isinstance(formatted, unicode):
                self.stream.write(formatted.encode("utf-8", "replace"))
            elif isinstance(formatted, str):
                self.stream.write(formatted)
            else:
                assert False, "Got output from the formatter of an unexpected type"

            self.stream.flush()

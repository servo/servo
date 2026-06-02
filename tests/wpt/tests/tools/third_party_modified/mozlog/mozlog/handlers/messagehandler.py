# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.


class MessageHandler(object):
    """A message handler providing message handling facilities to
    classes derived from BaseHandler and BaseFormatter. This is a
    composition class, to ensure handlers and formatters remain separate.

    :param inner: A handler-like callable that may receive messages
                  from a log user.
    """

    def __init__(self):
        self.message_handlers = {}
        self.wrapped = []

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
        if self.wrapped:
            for inner in self.wrapped:
                rv.extend(inner.handle_message(topic, cmd, *args))
        return rv

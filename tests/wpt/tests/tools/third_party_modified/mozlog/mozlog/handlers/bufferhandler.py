# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from .base import BaseHandler


class BufferHandler(BaseHandler):
    """Handler that maintains a circular buffer of messages based on the
    size and actions specified by a user.

    :param inner: The underlying handler used to emit messages.
    :param message_limit: The maximum number of messages to retain for
                          context. If None, the buffer will grow without limit.
    :param buffered_actions: The set of actions to include in the buffer
                             rather than log directly.
    """

    def __init__(self, inner, message_limit=100, buffered_actions=None):
        BaseHandler.__init__(self, inner)
        self.inner = inner
        self.message_limit = message_limit
        if buffered_actions is None:
            buffered_actions = ["log", "test_status"]
        self.buffered_actions = set(buffered_actions)
        self._buffering = True

        if self.message_limit is not None:
            self._buffer = [None] * self.message_limit
            self._buffer_pos = 0
        else:
            self._buffer = []

        self.message_handler.register_message_handlers(
            "buffer",
            {
                "on": self._enable_buffering,
                "off": self._disable_buffering,
                "flush": self._flush_buffered,
                "clear": self._clear_buffer,
            },
        )

    def __call__(self, data):
        action = data["action"]
        if "bypass_mozlog_buffer" in data:
            data.pop("bypass_mozlog_buffer")
            self.inner(data)
            return
        if not self._buffering or action not in self.buffered_actions:
            self.inner(data)
            return

        self._add_message(data)

    def _add_message(self, data):
        if self.message_limit is None:
            self._buffer.append(data)
        else:
            self._buffer[self._buffer_pos] = data
            self._buffer_pos = (self._buffer_pos + 1) % self.message_limit

    def _enable_buffering(self):
        self._buffering = True

    def _disable_buffering(self):
        self._buffering = False

    def _clear_buffer(self):
        """Clear the buffer of unwanted messages."""
        current_size = len([m for m in self._buffer if m is not None])
        if self.message_limit is not None:
            self._buffer = [None] * self.message_limit
        else:
            self._buffer = []
        return current_size

    def _flush_buffered(self):
        """Logs the contents of the current buffer"""
        for msg in self._buffer[self._buffer_pos:]:
            if msg is not None:
                self.inner(msg)
        for msg in self._buffer[: self._buffer_pos]:
            if msg is not None:
                self.inner(msg)
        return self._clear_buffer()

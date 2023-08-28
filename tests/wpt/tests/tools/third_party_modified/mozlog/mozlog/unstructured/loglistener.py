# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import socket
import socketserver


class LogMessageServer(socketserver.TCPServer):
    def __init__(self, server_address, logger, message_callback=None, timeout=3):
        socketserver.TCPServer.__init__(self, server_address, LogMessageHandler)
        self._logger = logger
        self._message_callback = message_callback
        self.timeout = timeout


class LogMessageHandler(socketserver.BaseRequestHandler):
    """Processes output from a connected log source, logging to an
    existing logger upon receipt of a well-formed log messsage."""

    def handle(self):
        """Continually listens for log messages."""
        self._partial_message = ""
        self.request.settimeout(self.server.timeout)

        while True:
            try:
                data = self.request.recv(1024)
                if not data:
                    return
                self.process_message(data.decode())
            except socket.timeout:
                return

    def process_message(self, data):
        """Processes data from a connected log source. Messages are assumed
        to be newline delimited, and generally well-formed JSON."""
        for part in data.split("\n"):
            msg_string = self._partial_message + part
            try:
                msg = json.loads(msg_string)
                self._partial_message = ""
                self.server._logger.log_structured(msg.get("action", "UNKNOWN"), msg)
                if self.server._message_callback:
                    self.server._message_callback()

            except ValueError:
                self._partial_message = msg_string

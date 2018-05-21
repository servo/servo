# Copyright 2011, Google Inc.
# All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are
# met:
#
#     * Redistributions of source code must retain the above copyright
# notice, this list of conditions and the following disclaimer.
#     * Redistributions in binary form must reproduce the above
# copyright notice, this list of conditions and the following disclaimer
# in the documentation and/or other materials provided with the
# distribution.
#     * Neither the name of Google Inc. nor the names of its
# contributors may be used to endorse or promote products derived from
# this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


"""Mocks for testing.
"""


import Queue
import threading

from mod_pywebsocket import common
from mod_pywebsocket.stream import StreamHixie75


class _MockConnBase(object):
    """Base class of mocks for mod_python.apache.mp_conn.

    This enables tests to check what is written to a (mock) mp_conn.
    """

    def __init__(self):
        self._write_data = []
        self.remote_addr = 'fake_address'

    def write(self, data):
        """Override mod_python.apache.mp_conn.write."""

        self._write_data.append(data)

    def written_data(self):
        """Get bytes written to this mock."""

        return ''.join(self._write_data)


class MockConn(_MockConnBase):
    """Mock for mod_python.apache.mp_conn.

    This enables tests to specify what should be read from a (mock) mp_conn as
    well as to check what is written to it.
    """

    def __init__(self, read_data):
        """Constructs an instance.

        Args:
            read_data: bytes that should be returned when read* methods are
            called.
        """

        _MockConnBase.__init__(self)
        self._read_data = read_data
        self._read_pos = 0

    def readline(self):
        """Override mod_python.apache.mp_conn.readline."""

        if self._read_pos >= len(self._read_data):
            return ''
        end_index = self._read_data.find('\n', self._read_pos) + 1
        if not end_index:
            end_index = len(self._read_data)
        return self._read_up_to(end_index)

    def read(self, length):
        """Override mod_python.apache.mp_conn.read."""

        if self._read_pos >= len(self._read_data):
            return ''
        end_index = min(len(self._read_data), self._read_pos + length)
        return self._read_up_to(end_index)

    def _read_up_to(self, end_index):
        line = self._read_data[self._read_pos:end_index]
        self._read_pos = end_index
        return line


class MockBlockingConn(_MockConnBase):
    """Blocking mock for mod_python.apache.mp_conn.

    This enables tests to specify what should be read from a (mock) mp_conn as
    well as to check what is written to it.
    Callers of read* methods will block if there is no bytes available.
    """

    def __init__(self):
        _MockConnBase.__init__(self)
        self._queue = Queue.Queue()

    def readline(self):
        """Override mod_python.apache.mp_conn.readline."""
        line = ''
        while True:
            c = self._queue.get()
            line += c
            if c == '\n':
                return line

    def read(self, length):
        """Override mod_python.apache.mp_conn.read."""

        data = ''
        for unused in range(length):
            data += self._queue.get()
        return data

    def put_bytes(self, bytes):
        """Put bytes to be read from this mock.

        Args:
            bytes: bytes to be read.
        """

        for byte in bytes:
            self._queue.put(byte)


class MockTable(dict):
    """Mock table.

    This mimics mod_python mp_table. Note that only the methods used by
    tests are overridden.
    """

    def __init__(self, copy_from={}):
        if isinstance(copy_from, dict):
            copy_from = copy_from.items()
        for key, value in copy_from:
            self.__setitem__(key, value)

    def __getitem__(self, key):
        return super(MockTable, self).__getitem__(key.lower())

    def __setitem__(self, key, value):
        super(MockTable, self).__setitem__(key.lower(), value)

    def get(self, key, def_value=None):
        return super(MockTable, self).get(key.lower(), def_value)


class MockRequest(object):
    """Mock request.

    This mimics mod_python request.
    """

    def __init__(self, uri=None, headers_in={}, connection=None, method='GET',
                 protocol='HTTP/1.1', is_https=False):
        """Construct an instance.

        Arguments:
            uri: URI of the request.
            headers_in: Request headers.
            connection: Connection used for the request.
            method: request method.
            is_https: Whether this request is over SSL.

        See the document of mod_python Request for details.
        """
        self.uri = uri
        self.unparsed_uri = uri
        self.connection = connection
        self.method = method
        self.protocol = protocol
        self.headers_in = MockTable(headers_in)
        # self.is_https_ needs to be accessible from tests.  To avoid name
        # conflict with self.is_https(), it is named as such.
        self.is_https_ = is_https
        self.ws_stream = StreamHixie75(self, True)
        self.ws_close_code = None
        self.ws_close_reason = None
        self.ws_version = common.VERSION_HYBI00
        self.ws_deflate = False

    def is_https(self):
        """Return whether this request is over SSL."""
        return self.is_https_


class MockDispatcher(object):
    """Mock for dispatch.Dispatcher."""

    def __init__(self):
        self.do_extra_handshake_called = False

    def do_extra_handshake(self, conn_context):
        self.do_extra_handshake_called = True

    def transfer_data(self, conn_context):
        pass


# vi:sts=4 sw=4 et

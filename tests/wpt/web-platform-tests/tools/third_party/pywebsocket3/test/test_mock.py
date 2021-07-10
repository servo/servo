#!/usr/bin/env python
#
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
"""Tests for mock module."""

from __future__ import absolute_import
import six.moves.queue
import threading
import unittest

import set_sys_path  # Update sys.path to locate mod_pywebsocket module.

from test import mock


class MockConnTest(unittest.TestCase):
    """A unittest for MockConn class."""
    def setUp(self):
        self._conn = mock.MockConn(b'ABC\r\nDEFG\r\n\r\nHIJK')

    def test_readline(self):
        self.assertEqual(b'ABC\r\n', self._conn.readline())
        self.assertEqual(b'DEFG\r\n', self._conn.readline())
        self.assertEqual(b'\r\n', self._conn.readline())
        self.assertEqual(b'HIJK', self._conn.readline())
        self.assertEqual(b'', self._conn.readline())

    def test_read(self):
        self.assertEqual(b'ABC\r\nD', self._conn.read(6))
        self.assertEqual(b'EFG\r\n\r\nHI', self._conn.read(9))
        self.assertEqual(b'JK', self._conn.read(10))
        self.assertEqual(b'', self._conn.read(10))

    def test_read_and_readline(self):
        self.assertEqual(b'ABC\r\nD', self._conn.read(6))
        self.assertEqual(b'EFG\r\n', self._conn.readline())
        self.assertEqual(b'\r\nHIJK', self._conn.read(9))
        self.assertEqual(b'', self._conn.readline())

    def test_write(self):
        self._conn.write(b'Hello\r\n')
        self._conn.write(b'World\r\n')
        self.assertEqual(b'Hello\r\nWorld\r\n', self._conn.written_data())


class MockBlockingConnTest(unittest.TestCase):
    """A unittest for MockBlockingConn class."""
    def test_read(self):
        """Tests that data put to MockBlockingConn by put_bytes method can be
        read from it.
        """
        class LineReader(threading.Thread):
            """A test class that launches a thread, calls readline on the
            specified conn repeatedly and puts the read data to the specified
            queue.
            """
            def __init__(self, conn, queue):
                threading.Thread.__init__(self)
                self._queue = queue
                self._conn = conn
                self.setDaemon(True)
                self.start()

            def run(self):
                while True:
                    data = self._conn.readline()
                    self._queue.put(data)

        conn = mock.MockBlockingConn()
        queue = six.moves.queue.Queue()
        reader = LineReader(conn, queue)
        self.assertTrue(queue.empty())
        conn.put_bytes(b'Foo bar\r\n')
        read = queue.get()
        self.assertEqual(b'Foo bar\r\n', read)


class MockTableTest(unittest.TestCase):
    """A unittest for MockTable class."""
    def test_create_from_dict(self):
        table = mock.MockTable({'Key': 'Value'})
        self.assertEqual('Value', table.get('KEY'))
        self.assertEqual('Value', table['key'])

    def test_create_from_list(self):
        table = mock.MockTable([('Key', 'Value')])
        self.assertEqual('Value', table.get('KEY'))
        self.assertEqual('Value', table['key'])

    def test_create_from_tuple(self):
        table = mock.MockTable((('Key', 'Value'), ))
        self.assertEqual('Value', table.get('KEY'))
        self.assertEqual('Value', table['key'])

    def test_set_and_get(self):
        table = mock.MockTable()
        self.assertEqual(None, table.get('Key'))
        table['Key'] = 'Value'
        self.assertEqual('Value', table.get('Key'))
        self.assertEqual('Value', table.get('key'))
        self.assertEqual('Value', table.get('KEY'))
        self.assertEqual('Value', table['Key'])
        self.assertEqual('Value', table['key'])
        self.assertEqual('Value', table['KEY'])


if __name__ == '__main__':
    unittest.main()

# vi:sts=4 sw=4 et

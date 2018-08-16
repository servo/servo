# -*- coding: utf-8 -*-
"""
hyper/http20/stream
~~~~~~~~~~~~~~~~~~~

Objects that make up the stream-level abstraction of hyper's HTTP/2 support.

These objects are not expected to be part of the public HTTP/2 API: they're
intended purely for use inside hyper's HTTP/2 abstraction.

Conceptually, a single HTTP/2 connection is made up of many streams: each
stream is an independent, bi-directional sequence of HTTP headers and data.
Each stream is identified by a monotonically increasing integer, assigned to
the stream by the endpoint that initiated the stream.
"""
from ..h2 import exceptions as h2Exceptions

from ..common.headers import HTTPHeaderMap
from .util import h2_safe_headers
import logging

log = logging.getLogger(__name__)

# Define the largest chunk of data we'll send in one go. Realistically, we
# should take the MSS into account but that's pretty dull, so let's just say
# 1kB and call it a day.
MAX_CHUNK = 1024


class Stream(object):
    """
    A single HTTP/2 stream.

    A stream is an independent, bi-directional sequence of HTTP headers and
    data. Each stream is identified by a single integer. From a HTTP
    perspective, a stream _approximately_ matches a single request-response
    pair.
    """
    def __init__(self,
                 stream_id,
                 window_manager,
                 connection,
                 send_outstanding_data,
                 recv_cb,
                 close_cb):
        self.stream_id = stream_id
        self.headers = HTTPHeaderMap()

        # Set to a key-value set of the response headers once their
        # HEADERS..CONTINUATION frame sequence finishes.
        self.response_headers = None

        # Set to a key-value set of the response trailers once their
        # HEADERS..CONTINUATION frame sequence finishes.
        self.response_trailers = None

        # A dict mapping the promised stream ID of a pushed resource to a
        # key-value set of its request headers. Entries are added once their
        # PUSH_PROMISE..CONTINUATION frame sequence finishes.
        self.promised_headers = {}

        # Unconsumed response data chunks. Empties after every call to _read().
        self.data = []

        # Whether the remote side has completed the stream.
        self.remote_closed = False

        # Whether we have closed the stream.
        self.local_closed = False

        # There are two flow control windows: one for data we're sending,
        # one for data being sent to us.
        self._in_window_manager = window_manager

        # Save off a reference to the state machine wrapped with lock.
        self._conn = connection

        # Save off a data callback.
        self._send_outstanding_data = send_outstanding_data
        self._recv_cb = recv_cb
        self._close_cb = close_cb

    def add_header(self, name, value, replace=False):
        """
        Adds a single HTTP header to the headers to be sent on the request.
        """
        if not replace:
            self.headers[name] = value
        else:
            self.headers.replace(name, value)

    def send_headers(self, end_stream=False):
        """
        Sends the complete saved header block on the stream.
        """
        headers = self.get_headers()
        with self._conn as conn:
            conn.send_headers(self.stream_id, headers, end_stream)
        self._send_outstanding_data()

        if end_stream:
            self.local_closed = True

    def send_data(self, data, final):
        """
        Send some data on the stream. If this is the end of the data to be
        sent, the ``final`` flag _must_ be set to True. If no data is to be
        sent, set ``data`` to ``None``.
        """
        # Define a utility iterator for file objects.
        def file_iterator(fobj):
            while True:
                data = fobj.read(MAX_CHUNK)
                yield data
                if len(data) < MAX_CHUNK:
                    break

        # Build the appropriate iterator for the data, in chunks of CHUNK_SIZE.
        if hasattr(data, 'read'):
            chunks = file_iterator(data)
        else:
            chunks = (data[i:i+MAX_CHUNK]
                      for i in range(0, len(data), MAX_CHUNK))

        for chunk in chunks:
            self._send_chunk(chunk, final)

    def _read(self, amt=None):
        """
        Read data from the stream. Unlike a normal read behaviour, this
        function returns _at least_ ``amt`` data, but may return more.
        """
        def listlen(list):
            return sum(map(len, list))

        # Keep reading until the stream is closed or we get enough data.
        while (not self.remote_closed and
                (amt is None or listlen(self.data) < amt)):
            self._recv_cb(stream_id=self.stream_id)

        result = b''.join(self.data)
        self.data = []
        return result

    def _read_one_frame(self):
        """
        Reads a single data frame from the stream and returns it.
        """
        # Keep reading until the stream is closed or we have a data frame.
        while not self.remote_closed and not self.data:
            self._recv_cb(stream_id=self.stream_id)

        try:
            return self.data.pop(0)
        except IndexError:
            return None

    def receive_response(self, event):
        """
        Receive response headers.
        """
        # TODO: If this is called while we're still sending data, we may want
        # to stop sending that data and check the response. Early responses to
        # big uploads are almost always a problem.
        self.response_headers = HTTPHeaderMap(event.headers)

    def receive_trailers(self, event):
        """
        Receive response trailers.
        """
        self.response_trailers = HTTPHeaderMap(event.headers)

    def receive_push(self, event):
        """
        Receive the request headers for a pushed stream.
        """
        self.promised_headers[event.pushed_stream_id] = event.headers

    def receive_data(self, event):
        """
        Receive a chunk of data.
        """
        size = event.flow_controlled_length
        increment = self._in_window_manager._handle_frame(size)

        # Append the data to the buffer.
        self.data.append(event.data)

        if increment:
            try:
                with self._conn as conn:
                    conn.increment_flow_control_window(
                        increment, stream_id=self.stream_id
                    )
            except h2Exceptions.StreamClosedError:
                # We haven't got to it yet, but the stream is already
                # closed. We don't need to increment the window in this
                # case!
                pass
            else:
                self._send_outstanding_data()

    def receive_end_stream(self, event):
        """
        All of the data is returned now.
        """
        self.remote_closed = True

    def receive_reset(self, event):
        """
        Stream forcefully reset.
        """
        self.remote_closed = True
        self._close_cb(self.stream_id)

    def get_headers(self):
        """
        Provides the headers to the connection object.
        """
        # Strip any headers invalid in H2.
        return h2_safe_headers(self.headers)

    def getheaders(self):
        """
        Once all data has been sent on this connection, returns a key-value set
        of the headers of the response to the original request.
        """
        # Keep reading until all headers are received.
        while self.response_headers is None:
            self._recv_cb(stream_id=self.stream_id)

        # Find the Content-Length header if present.
        self._in_window_manager.document_size = (
            int(self.response_headers.get(b'content-length', [0])[0])
        )

        return self.response_headers

    def gettrailers(self):
        """
        Once all data has been sent on this connection, returns a key-value set
        of the trailers of the response to the original request.

        .. warning:: Note that this method requires that the stream is
                     totally exhausted. This means that, if you have not
                     completely read from the stream, all stream data will be
                     read into memory.

        :returns: The key-value set of the trailers, or ``None`` if no trailers
                  were sent.
        """
        # Keep reading until the stream is done.
        while not self.remote_closed:
            self._recv_cb(stream_id=self.stream_id)

        return self.response_trailers

    def get_pushes(self, capture_all=False):
        """
        Returns a generator that yields push promises from the server. Note
        that this method is not idempotent; promises returned in one call will
        not be returned in subsequent calls. Iterating through generators
        returned by multiple calls to this method simultaneously results in
        undefined behavior.

        :param capture_all: If ``False``, the generator will yield all buffered
            push promises without blocking. If ``True``, the generator will
            first yield all buffered push promises, then yield additional ones
            as they arrive, and terminate when the original stream closes.
        """
        while True:
            for pair in self.promised_headers.items():
                yield pair
            self.promised_headers = {}
            if not capture_all or self.remote_closed:
                break
            self._recv_cb(stream_id=self.stream_id)

    def close(self, error_code=None):
        """
        Closes the stream. If the stream is currently open, attempts to close
        it as gracefully as possible.

        :param error_code: (optional) The error code to reset the stream with.
        :returns: Nothing.
        """
        # FIXME: I think this is overbroad, but for now it's probably ok.
        if not (self.remote_closed and self.local_closed):
            try:
                with self._conn as conn:
                    conn.reset_stream(self.stream_id, error_code or 0)
            except h2Exceptions.ProtocolError:
                # If for any reason we can't reset the stream, just
                # tolerate it.
                pass
            else:
                self._send_outstanding_data(tolerate_peer_gone=True)
            self.remote_closed = True
            self.local_closed = True

        self._close_cb(self.stream_id)

    @property
    def _out_flow_control_window(self):
        """
        The size of our outbound flow control window.
        """

        with self._conn as conn:
            return conn.local_flow_control_window(self.stream_id)

    def _send_chunk(self, data, final):
        """
        Implements most of the sending logic.

        Takes a single chunk of size at most MAX_CHUNK, wraps it in a frame and
        sends it. Optionally sets the END_STREAM flag if this is the last chunk
        (determined by being of size less than MAX_CHUNK) and no more data is
        to be sent.
        """
        # If we don't fit in the connection window, try popping frames off the
        # connection in hope that one might be a window update frame.
        while len(data) > self._out_flow_control_window:
            self._recv_cb()

        # If the length of the data is less than MAX_CHUNK, we're probably
        # at the end of the file. If this is the end of the data, mark it
        # as END_STREAM.
        end_stream = False
        if len(data) < MAX_CHUNK and final:
            end_stream = True

        # Send the frame and decrement the flow control window.
        with self._conn as conn:
            conn.send_data(
                stream_id=self.stream_id, data=data, end_stream=end_stream
            )
        self._send_outstanding_data()

        if end_stream:
            self.local_closed = True

# -*- coding: utf-8 -*-
"""
hyper/http20/connection
~~~~~~~~~~~~~~~~~~~~~~~

Objects that build hyper's connection-level HTTP/2 abstraction.
"""
import h2.connection
import h2.events
import h2.settings

from ..compat import ssl
from ..tls import wrap_socket, H2_NPN_PROTOCOLS, H2C_PROTOCOL
from ..common.exceptions import ConnectionResetError
from ..common.bufsocket import BufferedSocket
from ..common.headers import HTTPHeaderMap
from ..common.util import to_host_port_tuple, to_native_string, to_bytestring
from ..compat import unicode, bytes
from .stream import Stream
from .response import HTTP20Response, HTTP20Push
from .window import FlowControlManager
from .exceptions import ConnectionError, StreamResetError
from . import errors

import errno
import logging
import socket
import time
import threading

log = logging.getLogger(__name__)

DEFAULT_WINDOW_SIZE = 65535

TRANSIENT_SSL_ERRORS = (ssl.SSL_ERROR_WANT_READ, ssl.SSL_ERROR_WANT_WRITE)


class _LockedObject(object):
    """
    A wrapper class that hides a specific object behind a lock.

    The goal here is to provide a simple way to protect access to an object
    that cannot safely be simultaneously accessed from multiple threads. The
    intended use of this class is simple: take hold of it with a context
    manager, which returns the protected object.
    """
    def __init__(self, obj):
        self.lock = threading.RLock()
        self._obj = obj

    def __enter__(self):
        self.lock.acquire()
        return self._obj

    def __exit__(self, _exc_type, _exc_val, _exc_tb):
        self.lock.release()


class HTTP20Connection(object):
    """
    An object representing a single HTTP/2 connection to a server.

    This object behaves similarly to the Python standard library's
    ``HTTPConnection`` object, with a few critical differences.

    Most of the standard library's arguments to the constructor are irrelevant
    for HTTP/2 or not supported by hyper.

    :param host: The host to connect to. This may be an IP address or a
        hostname, and optionally may include a port: for example,
        ``'http2bin.org'``, ``'http2bin.org:443'`` or ``'127.0.0.1'``.
    :param port: (optional) The port to connect to. If not provided and one
        also isn't provided in the ``host`` parameter, defaults to 443.
    :param secure: (optional) Whether the request should use TLS. Defaults to
        ``False`` for most requests, but to ``True`` for any request issued to
        port 443.
    :param window_manager: (optional) The class to use to manage flow control
        windows. This needs to be a subclass of the
        :class:`BaseFlowControlManager
        <hyper.http20.window.BaseFlowControlManager>`. If not provided,
        :class:`FlowControlManager <hyper.http20.window.FlowControlManager>`
        will be used.
    :param enable_push: (optional) Whether the server is allowed to push
        resources to the client (see
        :meth:`get_pushes() <hyper.HTTP20Connection.get_pushes>`).
    :param ssl_context: (optional) A class with custom certificate settings.
        If not provided then hyper's default ``SSLContext`` is used instead.
    :param proxy_host: (optional) The proxy to connect to.  This can be an IP
        address or a host name and may include a port.
    :param proxy_port: (optional) The proxy port to connect to. If not provided
        and one also isn't provided in the ``proxy`` parameter, defaults to
        8080.
    """
    def __init__(self, host, port=None, secure=None, window_manager=None,
                 enable_push=False, ssl_context=None, proxy_host=None,
                 proxy_port=None, force_proto=None, **kwargs):
        """
        Creates an HTTP/2 connection to a specific server.
        """
        if port is None:
            self.host, self.port = to_host_port_tuple(host, default_port=443)
        else:
            self.host, self.port = host, port

        if secure is not None:
            self.secure = secure
        elif self.port == 443:
            self.secure = True
        else:
            self.secure = False

        self._enable_push = enable_push
        self.ssl_context = ssl_context

        # Setup proxy details if applicable.
        if proxy_host:
            if proxy_port is None:
                self.proxy_host, self.proxy_port = to_host_port_tuple(
                    proxy_host, default_port=8080
                )
            else:
                self.proxy_host, self.proxy_port = proxy_host, proxy_port
        else:
            self.proxy_host = None
            self.proxy_port = None

        #: The size of the in-memory buffer used to store data from the
        #: network. This is used as a performance optimisation. Increase buffer
        #: size to improve performance: decrease it to conserve memory.
        #: Defaults to 64kB.
        self.network_buffer_size = 65536

        self.force_proto = force_proto

        # Concurrency
        #
        # Use one lock (_lock) to synchronize any interaction with global
        # connection state, e.g. stream creation/deletion.
        #
        # It's ok to use the same in lock all these cases as they occur at
        # different/linked points in the connection's lifecycle.
        #
        # Use another 2 locks (_write_lock, _read_lock) to synchronize
        # - _send_cb
        # - _recv_cb
        # respectively.
        #
        # I.e, send/recieve on the connection and its streams are serialized
        # separately across the threads accessing the connection.  This is a
        # simple way of providing thread-safety.
        #
        # _write_lock and _read_lock synchronize all interactions between
        # streams and the connnection.  There is a third I/O callback,
        # _close_stream, passed to a stream's constructor.  It does not need to
        # be synchronized, it uses _send_cb internally (which is serialized);
        # its other activity (safe deletion of the stream from self.streams)
        # does not require synchronization.
        #
        # _read_lock may be acquired when already holding the _write_lock,
        # when they both held it is always by acquiring _write_lock first.
        #
        # Either _read_lock or _write_lock may be acquired whilst holding _lock
        # which should always be acquired before either of the other two.
        self._lock = threading.RLock()
        self._write_lock = threading.RLock()
        self._read_lock = threading.RLock()

        # Create the mutable state.
        self.__wm_class = window_manager or FlowControlManager
        self.__init_state()

        return

    def __init_state(self):
        """
        Initializes the 'mutable state' portions of the HTTP/2 connection
        object.

        This method exists to enable HTTP20Connection objects to be reused if
        they're closed, by resetting the connection object to its basic state
        whenever it ends up closed. Any situation that needs to recreate the
        connection can call this method and it will be done.

        This is one of the only methods in hyper that is truly private, as
        users should be strongly discouraged from messing about with connection
        objects themselves.
        """
        self._conn = _LockedObject(h2.connection.H2Connection())

        # Streams are stored in a dictionary keyed off their stream IDs. We
        # also save the most recent one for easy access without having to walk
        # the dictionary.
        #
        # We add a set of all streams that we or the remote party forcefully
        # closed with RST_STREAM, to avoid encountering issues where frames
        # were already in flight before the RST was processed.
        #
        # Finally, we add a set of streams that recently received data.  When
        # using multiple threads, this avoids reading on threads that have just
        # acquired the I/O lock whose streams have already had their data read
        # for them by prior threads.
        self.streams = {}
        self.recent_stream = None
        self.next_stream_id = 1
        self.reset_streams = set()
        self.recent_recv_streams = set()

        # The socket used to send data.
        self._sock = None

        # Instantiate a window manager.
        self.window_manager = self.__wm_class(65535)

        return

    def ping(self, opaque_data):
        """
        Send a PING frame.

        Concurrency
        -----------

        This method is thread-safe.

        :param opaque_data: A bytestring of length 8 that will be sent in the
                            PING frame.
        :returns: Nothing
        """
        self.connect()
        with self._write_lock:
            with self._conn as conn:
                conn.ping(to_bytestring(opaque_data))
            self._send_outstanding_data()

    def request(self, method, url, body=None, headers=None):
        """
        This will send a request to the server using the HTTP request method
        ``method`` and the selector ``url``. If the ``body`` argument is
        present, it should be string or bytes object of data to send after the
        headers are finished. Strings are encoded as UTF-8. To use other
        encodings, pass a bytes object. The Content-Length header is set to the
        length of the body field.

        Concurrency
        -----------

        This method is thread-safe.

        :param method: The request method, e.g. ``'GET'``.
        :param url: The URL to contact, e.g. ``'/path/segment'``.
        :param body: (optional) The request body to send. Must be a bytestring
            or a file-like object.
        :param headers: (optional) The headers to send on the request.
        :returns: A stream ID for the request.
        """
        headers = headers or {}

        # Concurrency
        #
        # It's necessary to hold a lock while this method runs to satisfy H2
        # protocol requirements.
        #
        # - putrequest obtains the next valid new stream_id
        # - endheaders sends a http2 message using the new stream_id
        #
        # If threads interleave these operations, it could result in messages
        # being sent in the wrong order, which can lead to the out-of-order
        # messages with lower stream IDs being closed prematurely.
        with self._write_lock:
            stream_id = self.putrequest(method, url)

            default_headers = (':method', ':scheme', ':authority', ':path')
            for name, value in headers.items():
                is_default = to_native_string(name) in default_headers
                self.putheader(name, value, stream_id, replace=is_default)

            # Convert the body to bytes if needed.
            if body and isinstance(body, (unicode, bytes)):
                body = to_bytestring(body)

            self.endheaders(message_body=body, final=True, stream_id=stream_id)

            return stream_id

    def _get_stream(self, stream_id):
        if stream_id is None:
            return self.recent_stream
        elif stream_id in self.reset_streams or stream_id not in self.streams:
            raise StreamResetError("Stream forcefully closed")
        else:
            return self.streams[stream_id]

    def get_response(self, stream_id=None):
        """
        Should be called after a request is sent to get a response from the
        server. If sending multiple parallel requests, pass the stream ID of
        the request whose response you want. Returns a
        :class:`HTTP20Response <hyper.HTTP20Response>` instance.
        If you pass no ``stream_id``, you will receive the oldest
        :class:`HTTPResponse <hyper.HTTP20Response>` still outstanding.

        Concurrency
        -----------

        This method is thread-safe.

        :param stream_id: (optional) The stream ID of the request for which to
            get a response.
        :returns: A :class:`HTTP20Response <hyper.HTTP20Response>` object.
        """
        stream = self._get_stream(stream_id)
        return HTTP20Response(stream.getheaders(), stream)

    def get_pushes(self, stream_id=None, capture_all=False):
        """
        Returns a generator that yields push promises from the server. **Note
        that this method is not idempotent**: promises returned in one call
        will not be returned in subsequent calls. Iterating through generators
        returned by multiple calls to this method simultaneously results in
        undefined behavior.

        :param stream_id: (optional) The stream ID of the request for which to
            get push promises.
        :param capture_all: (optional) If ``False``, the generator will yield
            all buffered push promises without blocking. If ``True``, the
            generator will first yield all buffered push promises, then yield
            additional ones as they arrive, and terminate when the original
            stream closes.
        :returns: A generator of :class:`HTTP20Push <hyper.HTTP20Push>` objects
            corresponding to the streams pushed by the server.
        """
        stream = self._get_stream(stream_id)
        for promised_stream_id, headers in stream.get_pushes(capture_all):
            yield HTTP20Push(
                HTTPHeaderMap(headers), self.streams[promised_stream_id]
            )

    def connect(self):
        """
        Connect to the server specified when the object was created. This is a
        no-op if we're already connected.

        Concurrency
        -----------

        This method is thread-safe. It may be called from multiple threads, and
        is a noop for all threads apart from the first.

        :returns: Nothing.

        """
        with self._lock:
            if self._sock is not None:
                return

            if not self.proxy_host:
                host = self.host
                port = self.port
            else:
                host = self.proxy_host
                port = self.proxy_port

            sock = socket.create_connection((host, port))

            if self.secure:
                assert not self.proxy_host, "Proxy with HTTPS not supported."
                sock, proto = wrap_socket(sock, host, self.ssl_context,
                                          force_proto=self.force_proto)
            else:
                proto = H2C_PROTOCOL

            log.debug("Selected NPN protocol: %s", proto)
            assert proto in H2_NPN_PROTOCOLS or proto == H2C_PROTOCOL

            self._sock = BufferedSocket(sock, self.network_buffer_size)

            self._send_preamble()

    def _connect_upgrade(self, sock):
        """
        Called by the generic HTTP connection when we're being upgraded. Locks
        in a new socket and places the backing state machine into an upgrade
        state, then sends the preamble.
        """
        self._sock = sock

        with self._conn as conn:
            conn.initiate_upgrade_connection()
            conn.update_settings(
                {h2.settings.SettingCodes.ENABLE_PUSH: int(self._enable_push)}
            )
        self._send_outstanding_data()

        # The server will also send an initial settings frame, so get it.
        # However, we need to make sure our stream state is set up properly
        # first, or any extra data we receive might cause us problems.
        s = self._new_stream(local_closed=True)
        self.recent_stream = s

        self._recv_cb()

    def _send_preamble(self):
        """
        Sends the necessary HTTP/2 preamble.
        """
        # We need to send the connection header immediately on this
        # connection, followed by an initial settings frame.
        with self._conn as conn:
            conn.initiate_connection()
            conn.update_settings(
                {h2.settings.SettingCodes.ENABLE_PUSH: int(self._enable_push)}
            )
        self._send_outstanding_data()

        # The server will also send an initial settings frame, so get it.
        self._recv_cb()

    def close(self, error_code=None):
        """
        Close the connection to the server.

        Concurrency
        -----------

        This method is thread-safe.

        :param error_code: (optional) The error code to reset all streams with.
        :returns: Nothing.
        """
        # Concurrency
        #
        # It's necessary to hold the lock here to ensure that threads closing
        # the connection see consistent state, and to prevent creation of
        # of new streams while the connection is being closed.
        #
        # I/O occurs while the lock is held; waiting threads will see a delay.
        with self._lock:
            # Close all streams
            for stream in list(self.streams.values()):
                log.debug("Close stream %d" % stream.stream_id)
                stream.close(error_code)

            # Send GoAway frame to the server
            try:
                with self._conn as conn:
                    conn.close_connection(error_code or 0)
                self._send_outstanding_data(tolerate_peer_gone=True)
            except Exception as e:  # pragma: no cover
                log.warn("GoAway frame could not be sent: %s" % e)

            if self._sock is not None:
                self._sock.close()
            self.__init_state()

    def _send_outstanding_data(self, tolerate_peer_gone=False,
                               send_empty=True):
        # Concurrency
        #
        # Hold _write_lock; getting and writing data from _conn is synchronized
        #
        # I/O occurs while the lock is held; waiting threads will see a delay.
        with self._write_lock:
            with self._conn as conn:
                data = conn.data_to_send()
            if data or send_empty:
                self._send_cb(data, tolerate_peer_gone=tolerate_peer_gone)

    def putrequest(self, method, selector, **kwargs):
        """
        This should be the first call for sending a given HTTP request to a
        server. It returns a stream ID for the given connection that should be
        passed to all subsequent request building calls.

        Concurrency
        -----------

        This method is thread-safe. It can be called from multiple threads,
        and each thread should receive a unique stream ID.

        :param method: The request method, e.g. ``'GET'``.
        :param selector: The path selector.
        :returns: A stream ID for the request.
        """
        # Create a new stream.
        s = self._new_stream()

        # To this stream we need to immediately add a few headers that are
        # HTTP/2 specific. These are: ":method", ":scheme", ":authority" and
        # ":path". We can set all of these now.
        s.add_header(":method", method)
        s.add_header(":scheme", "https" if self.secure else "http")
        s.add_header(":authority", self.host)
        s.add_header(":path", selector)

        # Save the stream.
        self.recent_stream = s

        return s.stream_id

    def putheader(self, header, argument, stream_id=None, replace=False):
        """
        Sends an HTTP header to the server, with name ``header`` and value
        ``argument``.

        Unlike the ``httplib`` version of this function, this version does not
        actually send anything when called. Instead, it queues the headers up
        to be sent when you call
        :meth:`endheaders() <hyper.HTTP20Connection.endheaders>`.

        This method ensures that headers conform to the HTTP/2 specification.
        In particular, it strips out the ``Connection`` header, as that header
        is no longer valid in HTTP/2. This is to make it easy to write code
        that runs correctly in both HTTP/1.1 and HTTP/2.

        :param header: The name of the header.
        :param argument: The value of the header.
        :param stream_id: (optional) The stream ID of the request to add the
            header to.
        :returns: Nothing.
        """
        stream = self._get_stream(stream_id)
        stream.add_header(header, argument, replace)

        return

    def endheaders(self, message_body=None, final=False, stream_id=None):
        """
        Sends the prepared headers to the server. If the ``message_body``
        argument is provided it will also be sent to the server as the body of
        the request, and the stream will immediately be closed. If the
        ``final`` argument is set to True, the stream will also immediately
        be closed: otherwise, the stream will be left open and subsequent calls
        to ``send()`` will be required.

        :param message_body: (optional) The body to send. May not be provided
            assuming that ``send()`` will be called.
        :param final: (optional) If the ``message_body`` parameter is provided,
            should be set to ``True`` if no further data will be provided via
            calls to :meth:`send() <hyper.HTTP20Connection.send>`.
        :param stream_id: (optional) The stream ID of the request to finish
            sending the headers on.
        :returns: Nothing.
        """
        self.connect()

        stream = self._get_stream(stream_id)

        headers_only = (message_body is None and final)

        # Concurrency:
        #
        # Hold _write_lock: synchronize access to the connection's HPACK
        # encoder and decoder and the subsquent write to the connection
        with self._write_lock:
            stream.send_headers(headers_only)

            # Send whatever data we have.
            if message_body is not None:
                stream.send_data(message_body, final)

            self._send_outstanding_data()

        return

    def send(self, data, final=False, stream_id=None):
        """
        Sends some data to the server. This data will be sent immediately
        (excluding the normal HTTP/2 flow control rules). If this is the last
        data that will be sent as part of this request, the ``final`` argument
        should be set to ``True``. This will cause the stream to be closed.

        :param data: The data to send.
        :param final: (optional) Whether this is the last bit of data to be
            sent on this request.
        :param stream_id: (optional) The stream ID of the request to send the
            data on.
        :returns: Nothing.
        """
        stream = self._get_stream(stream_id)
        stream.send_data(data, final)

        return

    def _new_stream(self, stream_id=None, local_closed=False):
        """
        Returns a new stream object for this connection.
        """
        # Concurrency
        #
        # Hold _lock: ensure that threads accessing the connection see
        # self.next_stream_id in a consistent state
        #
        # No I/O occurs, the delay in waiting threads depends on their number.
        with self._lock:
            s = Stream(
                stream_id or self.next_stream_id,
                self.__wm_class(DEFAULT_WINDOW_SIZE),
                self._conn,
                self._send_outstanding_data,
                self._recv_cb,
                self._stream_close_cb,
            )
            s.local_closed = local_closed
            self.streams[s.stream_id] = s
            self.next_stream_id += 2

            return s

    def _send_cb(self, data, tolerate_peer_gone=False):
        """
        This is the callback used by streams to send data on the connection.

        This acts as a dumb wrapper around the socket send method.
        """
        # Concurrency
        #
        # Hold _write_lock: ensures only writer at a time
        #
        # I/O occurs while the lock is held; waiting threads will see a delay.
        with self._write_lock:
            try:
                self._sock.sendall(data)
            except socket.error as e:
                if (not tolerate_peer_gone or
                        e.errno not in (errno.EPIPE, errno.ECONNRESET)):
                    raise

    def _adjust_receive_window(self, frame_len):
        """
        Adjusts the window size in response to receiving a DATA frame of length
        ``frame_len``. May send a WINDOWUPDATE frame if necessary.
        """
        # Concurrency
        #
        # Hold _write_lock; synchronize the window manager update and the
        # subsequent potential write to the connection
        #
        # I/O may occur while the lock is held; waiting threads may see a
        # delay.
        with self._write_lock:
            increment = self.window_manager._handle_frame(frame_len)

            if increment:
                with self._conn as conn:
                    conn.increment_flow_control_window(increment)
                self._send_outstanding_data(tolerate_peer_gone=True)

        return

    def _single_read(self):
        """
        Performs a single read from the socket and hands the data off to the
        h2 connection object.
        """
        # Begin by reading what we can from the socket.
        #
        # Concurrency
        #
        # Synchronizes reading the data
        #
        # I/O occurs while the lock is held; waiting threads will see a delay.
        with self._read_lock:
            if self._sock is None:
                raise ConnectionError('tried to read after connection close')
            self._sock.fill()
            data = self._sock.buffer.tobytes()
            self._sock.advance_buffer(len(data))
            with self._conn as conn:
                events = conn.receive_data(data)
            stream_ids = set(getattr(e, 'stream_id', -1) for e in events)
            stream_ids.discard(-1)  # sentinel
            stream_ids.discard(0)  # connection events
            self.recent_recv_streams |= stream_ids

        for event in events:
            if isinstance(event, h2.events.DataReceived):
                self._adjust_receive_window(event.flow_controlled_length)
                self.streams[event.stream_id].receive_data(event)
            elif isinstance(event, h2.events.PushedStreamReceived):
                if self._enable_push:
                    self._new_stream(event.pushed_stream_id, local_closed=True)
                    self.streams[event.parent_stream_id].receive_push(event)
                else:
                    # Servers are forbidden from sending push promises when
                    # the ENABLE_PUSH setting is 0, but the spec leaves the
                    # client action undefined when they do it anyway. So we
                    # just refuse the stream and go about our business.
                    self._send_rst_frame(event.pushed_stream_id, 7)
            elif isinstance(event, h2.events.ResponseReceived):
                self.streams[event.stream_id].receive_response(event)
            elif isinstance(event, h2.events.TrailersReceived):
                self.streams[event.stream_id].receive_trailers(event)
            elif isinstance(event, h2.events.StreamEnded):
                self.streams[event.stream_id].receive_end_stream(event)
            elif isinstance(event, h2.events.StreamReset):
                if event.stream_id not in self.reset_streams:
                    self.reset_streams.add(event.stream_id)
                    self.streams[event.stream_id].receive_reset(event)
            elif isinstance(event, h2.events.ConnectionTerminated):
                # If we get GoAway with error code zero, we are doing a
                # graceful shutdown and all is well. Otherwise, throw an
                # exception.
                self.close()

                # If an error occured, try to read the error description from
                # code registry otherwise use the frame's additional data.
                if event.error_code != 0:
                    try:
                        name, number, description = errors.get_data(
                            event.error_code
                        )
                    except ValueError:
                        error_string = (
                            "Encountered error code %d" % event.error_code
                        )
                    else:
                        error_string = (
                            "Encountered error %s %s: %s" %
                            (name, number, description)
                        )

                    raise ConnectionError(error_string)
            else:
                log.info("Received unhandled event %s", event)

        self._send_outstanding_data(tolerate_peer_gone=True, send_empty=False)

    def _recv_cb(self, stream_id=0):
        """
        This is the callback used by streams to read data from the connection.

        This stream reads what data it can, and throws it into the underlying
        connection, before farming out any events that fire to the relevant
        streams. If the socket remains readable, it will then optimistically
        continue to attempt to read.

        This is generally called by a stream, not by the connection itself, and
        it's likely that streams will read a frame that doesn't belong to them.

        :param stream_id: (optional) The stream ID of the stream reading data
            from the connection.

        """
        # Begin by reading what we can from the socket.
        #
        # Concurrency
        #
        # Ignore this read if some other thread has recently read data from
        # from the requested stream.
        #
        # The lock here looks broad, but is needed to ensure correct behavior
        # when there are multiple readers of the same stream.  It is
        # re-acquired in the calls to self._single_read.
        #
        # I/O occurs while the lock is held; waiting threads will see a delay.
        with self._read_lock:
            log.debug('recv for stream %d with %s already present',
                      stream_id,
                      self.recent_recv_streams)
            if stream_id in self.recent_recv_streams:
                self.recent_recv_streams.discard(stream_id)
                return

            # make sure to validate the stream is readable.
            # if the connection was reset, this stream id won't appear in
            # self.streams and will cause this call to raise an exception.
            if stream_id:
                self._get_stream(stream_id)

            # TODO: Re-evaluate this.
            self._single_read()
            count = 9
            retry_wait = 0.05  # can improve responsiveness to delay the retry

            while count and self._sock is not None and self._sock.can_read:
                # If the connection has been closed, bail out, but retry
                # on transient errors.
                try:
                    self._single_read()
                except ConnectionResetError:
                    break
                except ssl.SSLError as e:  # pragma: no cover
                    # these are transient errors that can occur while reading
                    # from ssl connections.
                    if e.args[0] in TRANSIENT_SSL_ERRORS:
                        continue
                    else:
                        raise
                except socket.error as e:  # pragma: no cover
                    if e.errno in (errno.EINTR, errno.EAGAIN):
                        # if 'interrupted' or 'try again', continue
                        time.sleep(retry_wait)
                        continue
                    elif e.errno == errno.ECONNRESET:
                        break
                    else:
                        raise

                count -= 1

    def _send_rst_frame(self, stream_id, error_code):
        """
        Send reset stream frame with error code and remove stream from map.
        """
        # Concurrency
        #
        # Hold _write_lock; synchronize generating the reset frame and writing
        # it
        #
        # I/O occurs while the lock is held; waiting threads will see a delay.
        with self._write_lock:
            with self._conn as conn:
                conn.reset_stream(stream_id, error_code=error_code)
            self._send_outstanding_data()

        # Concurrency
        #
        # Hold _lock; the stream storage is being updated. No I/O occurs, any
        # delay is proportional to the number of waiting threads.
        with self._lock:
            try:
                del self.streams[stream_id]
                self.recent_recv_streams.discard(stream_id)
            except KeyError as e:  # pragma: no cover
                log.warn(
                    "Stream with id %d does not exist: %s",
                    stream_id, e)

            # Keep track of the fact that we reset this stream in case there
            # are other frames in flight.
            self.reset_streams.add(stream_id)

    def _stream_close_cb(self, stream_id):
        """
        Called by a stream when it is closing, so that state can be cleared.
        """
        try:
            del self.streams[stream_id]
            self.recent_recv_streams.discard(stream_id)
        except KeyError:
            pass

    # The following two methods are the implementation of the context manager
    # protocol.
    def __enter__(self):
        return self

    def __exit__(self, type, value, tb):
        self.close()
        return False  # Never swallow exceptions.

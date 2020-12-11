# -*- coding: utf-8 -*-
"""
asyncio-server.py
~~~~~~~~~~~~~~~~~

A fully-functional WSGI server, written using hyper-h2. Requires asyncio.

To test it, try installing httpbin from pip (``pip install httpbin``) and then
running the server (``python asyncio-server.py httpbin:app``).

This server does not support HTTP/1.1: it is a HTTP/2-only WSGI server. The
purpose of this code is to demonstrate how to integrate hyper-h2 into a more
complex application, and to demonstrate several principles of concurrent
programming.

The architecture looks like this:

+---------------------------------+
|     1x HTTP/2 Server Thread     |
|        (running asyncio)        |
+---------------------------------+
+---------------------------------+
|    N WSGI Application Threads   |
|           (no asyncio)          |
+---------------------------------+

Essentially, we spin up an asyncio-based event loop in the main thread. This
launches one HTTP/2 Protocol instance for each inbound connection, all of which
will read and write data from within the main thread in an asynchronous manner.

When each HTTP request comes in, the server will build the WSGI environment
dictionary and create a ``Stream`` object. This object will hold the relevant
state for the request/response pair and will act as the WSGI side of the logic.
That object will then be passed to a background thread pool, and when a worker
is available the WSGI logic will begin to be executed. This model ensures that
the asyncio web server itself is never blocked by the WSGI application.

The WSGI application and the HTTP/2 server communicate via an asyncio queue,
together with locks and threading events. The locks themselves are implicit in
asyncio's "call_soon_threadsafe", which allows for a background thread to
register an action with the main asyncio thread. When the asyncio thread
eventually takes the action in question it sets as threading event, signaling
to the background thread that it is free to continue its work.

To make the WSGI application work with flow control, there is a very important
invariant that must be observed. Any WSGI action that would cause data to be
emitted to the network MUST be accompanied by a threading Event that is not
set until that data has been written to the transport. This ensures that the
WSGI application *blocks* until the data is actually sent. The reason we
require this invariant is that the HTTP/2 server may choose to re-order some
data chunks for flow control reasons: that is, the application for stream X may
have actually written its data first, but the server may elect to send the data
for stream Y first. This means that it's vital that there not be *two* writes
for stream X active at any one point or they may get reordered, which would be
particularly terrible.

Thus, the server must cooperate to ensure that each threading event only fires
when the *complete* data for that event has been written to the asyncio
transport. Any earlier will cause untold craziness.
"""
import asyncio
import importlib
import queue
import ssl
import sys
import threading

from h2.config import H2Configuration
from h2.connection import H2Connection
from h2.events import (
    DataReceived, RequestReceived, WindowUpdated, StreamEnded, StreamReset
)


# Used to signal that a request has completed.
#
# This is a convenient way to do "in-band" signaling of stream completion
# without doing anything so heavyweight as using a class. Essentially, we can
# test identity against this empty object. In fact, this is so convenient that
# we use this object for all streams, for data in both directions: in and out.
END_DATA_SENTINEL = object()

# The WSGI callable. Stored here so that the protocol instances can get hold
# of the data.
APPLICATION = None


class H2Protocol(asyncio.Protocol):
    def __init__(self):
        config = H2Configuration(client_side=False, header_encoding='utf-8')

        # Our server-side state machine.
        self.conn = H2Connection(config=config)

        # The backing transport.
        self.transport = None

        # A dictionary of ``Stream`` objects, keyed by their stream ID. This
        # makes it easy to route data to the correct WSGI application instance.
        self.streams = {}

        # A queue of data emitted by WSGI applications that has not yet been
        # sent. Each stream may only have one chunk of data in either this
        # queue or the flow_controlled_data dictionary at any one time.
        self._stream_data = asyncio.Queue()

        # Data that has been pulled off the queue that is for a stream blocked
        # behind flow control limitations. This is used to avoid spinning on
        # _stream_data queue when a stream cannot have its data sent. Data that
        # cannot be sent on the connection when it is popped off the queue gets
        # placed here until the stream flow control window opens up again.
        self._flow_controlled_data = {}

        # A reference to the loop in which this protocol runs. This is needed
        # to synchronise up with background threads.
        self._loop = asyncio.get_event_loop()

        # Any streams that have been remotely reset. We keep track of these to
        # ensure that we don't emit data from a WSGI application whose stream
        # has been cancelled.
        self._reset_streams = set()

        # Keep track of the loop sending task so we can kill it when the
        # connection goes away.
        self._send_loop_task = None

    def connection_made(self, transport):
        """
        The connection has been made. Here we need to save off our transport,
        do basic HTTP/2 connection setup, and then start our data writing
        coroutine.
        """
        self.transport = transport
        self.conn.initiate_connection()
        self.transport.write(self.conn.data_to_send())
        self._send_loop_task = self._loop.create_task(self.sending_loop())

    def connection_lost(self, exc):
        """
        With the end of the connection, we just want to cancel our data sending
        coroutine.
        """
        self._send_loop_task.cancel()

    def data_received(self, data):
        """
        Process inbound data.
        """
        events = self.conn.receive_data(data)

        for event in events:
            if isinstance(event, RequestReceived):
                self.request_received(event)
            elif isinstance(event, DataReceived):
                self.data_frame_received(event)
            elif isinstance(event, WindowUpdated):
                self.window_opened(event)
            elif isinstance(event, StreamEnded):
                self.end_stream(event)
            elif isinstance(event, StreamReset):
                self.reset_stream(event)

        outbound_data = self.conn.data_to_send()
        if outbound_data:
            self.transport.write(outbound_data)

    def window_opened(self, event):
        """
        The flow control window got opened.

        This is important because it's possible that we were unable to send
        some WSGI data because the flow control window was too small. If that
        happens, the sending_loop coroutine starts buffering data.

        As the window gets opened, we need to unbuffer the data. We do that by
        placing the data chunks back on the back of the send queue and letting
        the sending loop take another shot at sending them.

        This system only works because we require that each stream only have
        *one* data chunk in the sending queue at any time. The threading events
        force this invariant to remain true.
        """
        if event.stream_id:
            # This is specific to a single stream.
            if event.stream_id in self._flow_controlled_data:
                self._stream_data.put_nowait(
                    self._flow_controlled_data.pop(event.stream_id)
                )
        else:
            # This event is specific to the connection. Free up *all* the
            # streams. This is a bit tricky, but we *must not* yield the flow
            # of control here or it all goes wrong.
            for data in self._flow_controlled_data.values():
                self._stream_data.put_nowait(data)

            self._flow_controlled_data = {}

    @asyncio.coroutine
    def sending_loop(self):
        """
        A call that loops forever, attempting to send data. This sending loop
        contains most of the flow-control smarts of this class: it pulls data
        off of the asyncio queue and then attempts to send it.

        The difficulties here are all around flow control. Specifically, a
        chunk of data may be too large to send. In this case, what will happen
        is that this coroutine will attempt to send what it can and will then
        store the unsent data locally. When a flow control event comes in that
        data will be freed up and placed back onto the asyncio queue, causing
        it to pop back up into the sending logic of this coroutine.

        This method explicitly *does not* handle HTTP/2 priority. That adds an
        extra layer of complexity to what is already a fairly complex method,
        and we'll look at how to do it another time.

        This coroutine explicitly *does not end*.
        """
        while True:
            stream_id, data, event = yield from self._stream_data.get()

            # If this stream got reset, just drop the data on the floor. Note
            # that we need to reset the event here to make sure that
            # application doesn't lock up.
            if stream_id in self._reset_streams:
                event.set()

            # Check if the body is done. If it is, this is really easy! Again,
            # we *must* set the event here or the application will lock up.
            if data is END_DATA_SENTINEL:
                self.conn.end_stream(stream_id)
                self.transport.write(self.conn.data_to_send())
                event.set()
                continue

            # We need to send data, but not to exceed the flow control window.
            # For that reason, grab only the data that fits: we'll buffer the
            # rest.
            window_size = self.conn.local_flow_control_window(stream_id)
            chunk_size = min(window_size, len(data))
            data_to_send = data[:chunk_size]
            data_to_buffer = data[chunk_size:]

            if data_to_send:
                # There's a maximum frame size we have to respect. Because we
                # aren't paying any attention to priority here, we can quite
                # safely just split this string up into chunks of max frame
                # size and blast them out.
                #
                # In a *real* application you'd want to consider priority here.
                max_size = self.conn.max_outbound_frame_size
                chunks = (
                    data_to_send[x:x+max_size]
                    for x in range(0, len(data_to_send), max_size)
                )
                for chunk in chunks:
                    self.conn.send_data(stream_id, chunk)
                self.transport.write(self.conn.data_to_send())

            # If there's data left to buffer, we should do that. Put it in a
            # dictionary and *don't set the event*: the app must not generate
            # any more data until we got rid of all of this data.
            if data_to_buffer:
                self._flow_controlled_data[stream_id] = (
                    stream_id, data_to_buffer, event
                )
            else:
                # We sent everything. We can let the WSGI app progress.
                event.set()

    def request_received(self, event):
        """
        A HTTP/2 request has been received. We need to invoke the WSGI
        application in a background thread to handle it.
        """
        # First, we are going to want an object to hold all the relevant state
        # for this request/response. For that, we have a stream object. We
        # need to store the stream object somewhere reachable for when data
        # arrives later.
        s = Stream(event.stream_id, self)
        self.streams[event.stream_id] = s

        # Next, we need to build the WSGI environ dictionary.
        environ = _build_environ_dict(event.headers, s)

        # Finally, we want to throw these arguments out to a threadpool and
        # let it run.
        self._loop.run_in_executor(
            None,
            s.run_in_threadpool,
            APPLICATION,
            environ,
        )

    def data_frame_received(self, event):
        """
        Data has been received by WSGI server and needs to be dispatched to a
        running application.

        Note that the flow control window is not modified here. That's
        deliberate: see Stream.__next__ for a longer discussion of why.
        """
        # Grab the stream in question from our dictionary and pass it on.
        stream = self.streams[event.stream_id]
        stream.receive_data(event.data, event.flow_controlled_length)

    def end_stream(self, event):
        """
        The stream data is complete.
        """
        stream = self.streams[event.stream_id]
        stream.request_complete()

    def reset_stream(self, event):
        """
        A stream got forcefully reset.

        This is a tricky thing to deal with because WSGI doesn't really have a
        good notion for it. Essentially, you have to let the application run
        until completion, but not actually let it send any data.

        We do that by discarding any data we currently have for it, and then
        marking the stream as reset to allow us to spot when that stream is
        trying to send data and drop that data on the floor.

        We then *also* signal the WSGI application that no more data is
        incoming, to ensure that it does not attempt to do further reads of the
        data.
        """
        if event.stream_id in self._flow_controlled_data:
            del self._flow_controlled_data

        self._reset_streams.add(event.stream_id)
        self.end_stream(event)

    def data_for_stream(self, stream_id, data):
        """
        Thread-safe method called from outside the main asyncio thread in order
        to send data on behalf of a WSGI application.

        Places data being written by a stream on an asyncio queue. Returns a
        threading event that will fire when that data is sent.
        """
        event = threading.Event()
        self._loop.call_soon_threadsafe(
            self._stream_data.put_nowait,
            (stream_id, data, event)
        )
        return event

    def send_response(self, stream_id, headers):
        """
        Thread-safe method called from outside the main asyncio thread in order
        to send the HTTP response headers on behalf of a WSGI application.

        Returns a threading event that will fire when the headers have been
        emitted to the network.
        """
        event = threading.Event()

        def _inner_send(stream_id, headers, event):
            self.conn.send_headers(stream_id, headers, end_stream=False)
            self.transport.write(self.conn.data_to_send())
            event.set()

        self._loop.call_soon_threadsafe(
            _inner_send,
            stream_id,
            headers,
            event
        )
        return event

    def open_flow_control_window(self, stream_id, increment):
        """
        Opens a flow control window for the given stream by the given amount.
        Called from a WSGI thread. Does not return an event because there's no
        need to block on this action, it may take place at any time.
        """
        def _inner_open(stream_id, increment):
            self.conn.increment_flow_control_window(increment, stream_id)
            self.conn.increment_flow_control_window(increment, None)
            self.transport.write(self.conn.data_to_send())

        self._loop.call_soon_threadsafe(
            _inner_open,
            stream_id,
            increment,
        )


class Stream:
    """
    This class holds all of the state for a single stream. It also provides
    several of the callables used by the WSGI application. Finally, it provides
    the logic for actually interfacing with the WSGI application.

    For these reasons, the object has *strict* requirements on thread-safety.
    While the object can be initialized in the main WSGI thread, the
    ``run_in_threadpool`` method *must* be called from outside that thread. At
    that point, the main WSGI thread may only call specific methods.
    """
    def __init__(self, stream_id, protocol):
        self.stream_id = stream_id
        self._protocol = protocol

        # Queue for data that has been received from the network. This is a
        # thread-safe queue, to allow both the WSGI application to block on
        # receiving more data and to allow the asyncio server to keep sending
        # more data.
        #
        # This queue is unbounded in size, but in practice it cannot contain
        # too much data because the flow control window doesn't get adjusted
        # unless data is removed from it.
        self._received_data = queue.Queue()

        # This buffer is used to hold partial chunks of data from
        # _received_data that were not returned out of ``read`` and friends.
        self._temp_buffer = b''

        # Temporary variables that allow us to keep hold of the headers and
        # response status until such time as the application needs us to send
        # them.
        self._response_status = b''
        self._response_headers = []
        self._headers_emitted = False

        # Whether the application has received all the data from the network
        # or not. This allows us to short-circuit some reads.
        self._complete = False

    def receive_data(self, data, flow_controlled_size):
        """
        Called by the H2Protocol when more data has been received from the
        network.

        Places the data directly on the queue in a thread-safe manner without
        blocking. Does not introspect or process the data.
        """
        self._received_data.put_nowait((data, flow_controlled_size))

    def request_complete(self):
        """
        Called by the H2Protocol when all the request data has been received.

        This works by placing the ``END_DATA_SENTINEL`` on the queue. The
        reading code knows, when it sees the ``END_DATA_SENTINEL``, to expect
        no more data from the network. This ensures that the state of the
        application only changes when it has finished processing the data from
        the network, even though the server may have long-since finished
        receiving all the data for this request.
        """
        self._received_data.put_nowait((END_DATA_SENTINEL, None))

    def run_in_threadpool(self, wsgi_application, environ):
        """
        This method should be invoked in a threadpool. At the point this method
        is invoked, the only safe methods to call from the original thread are
        ``receive_data`` and ``request_complete``: any other method is unsafe.

        This method handles the WSGI logic. It invokes the application callable
        in this thread, passing control over to the WSGI application. It then
        ensures that the data makes it back to the HTTP/2 connection via
        the thread-safe APIs provided below.
        """
        result = wsgi_application(environ, self.start_response)

        try:
            for data in result:
                self.write(data)
        finally:
            # This signals that we're done with data. The server will know that
            # this allows it to clean up its state: we're done here.
            self.write(END_DATA_SENTINEL)

    # The next few methods are called by the WSGI application. Firstly, the
    # three methods provided by the input stream.
    def read(self, size=None):
        """
        Called by the WSGI application to read data.

        This method is the one of two that explicitly pumps the input data
        queue, which means it deals with the ``_complete`` flag and the
        ``END_DATA_SENTINEL``.
        """
        # If we've already seen the END_DATA_SENTINEL, return immediately.
        if self._complete:
            return b''

        # If we've been asked to read everything, just iterate over ourselves.
        if size is None:
            return b''.join(self)

        # Otherwise, as long as we don't have enough data, spin looking for
        # another data chunk.
        data = b''
        while len(data) < size:
            try:
                chunk = next(self)
            except StopIteration:
                break

            # Concatenating strings this way is slow, but that's ok, this is
            # just a demo.
            data += chunk

        # We have *at least* enough data to return, but we may have too much.
        # If we do, throw it on a buffer: we'll use it later.
        to_return = data[:size]
        self._temp_buffer = data[size:]
        return to_return

    def readline(self, hint=None):
        """
        Called by the WSGI application to read a single line of data.

        This method rigorously observes the ``hint`` parameter: it will only
        ever read that much data. It then splits the data on a newline
        character and throws everything it doesn't need into a buffer.
        """
        data = self.read(hint)
        first_newline = data.find(b'\n')
        if first_newline == -1:
            # No newline, return all the data
            return data

        # We want to slice the data so that the head *includes* the first
        # newline. Then, any data left in this line we don't care about should
        # be prepended to the internal buffer.
        head, tail = data[:first_newline + 1], data[first_newline + 1:]
        self._temp_buffer = tail + self._temp_buffer

        return head

    def readlines(self, hint=None):
        """
        Called by the WSGI application to read several lines of data.

        This method is really pretty stupid. It rigorously observes the
        ``hint`` parameter, and quite happily returns the input split into
        lines.
        """
        # This method is *crazy inefficient*, but it's also a pretty stupid
        # method to call.
        data = self.read(hint)
        lines = data.split(b'\n')

        # Split removes the newline character, but we want it, so put it back.
        lines = [line + b'\n' for line in lines]

        # Except if the last character was a newline character we now have an
        # extra line that is just a newline: pull that out.
        if lines[-1] == b'\n':
            lines = lines[:-1]
        return lines

    def start_response(self, status, response_headers, exc_info=None):
        """
        This is the PEP-3333 mandated start_response callable.

        All it does is store the headers for later sending, and return our
        ```write`` callable.
        """
        if self._headers_emitted and exc_info is not None:
            raise exc_info[1].with_traceback(exc_info[2])

        assert not self._response_status or exc_info is not None
        self._response_status = status
        self._response_headers = response_headers

        return self.write

    def write(self, data):
        """
        Provides some data to write.

        This function *blocks* until such time as the data is allowed by
        HTTP/2 flow control. This allows a client to slow or pause the response
        as needed.

        This function is not supposed to be used, according to PEP-3333, but
        once we have it it becomes quite convenient to use it, so this app
        actually runs all writes through this function.
        """
        if not self._headers_emitted:
            self._emit_headers()
        event = self._protocol.data_for_stream(self.stream_id, data)
        event.wait()
        return

    def _emit_headers(self):
        """
        Sends the response headers.

        This is only called from the write callable and should only ever be
        called once. It does some minor processing (converts the status line
        into a status code because reason phrases are evil) and then passes
        the headers on to the server. This call explicitly blocks until the
        server notifies us that the headers have reached the network.
        """
        assert self._response_status and self._response_headers
        assert not self._headers_emitted
        self._headers_emitted = True

        # We only need the status code
        status = self._response_status.split(" ", 1)[0]
        headers = [(":status", status)]
        headers.extend(self._response_headers)
        event = self._protocol.send_response(self.stream_id, headers)
        event.wait()
        return

    # These two methods implement the iterator protocol. This allows a WSGI
    # application to iterate over this Stream object to get the data.
    def __iter__(self):
        return self

    def __next__(self):
        # If the complete request has been read, abort immediately.
        if self._complete:
            raise StopIteration()

        # If we have data stored in a temporary buffer for any reason, return
        # that and clear the buffer.
        #
        # This can actually only happen when the application uses one of the
        # read* callables, but that's fine.
        if self._temp_buffer:
            buffered_data = self._temp_buffer
            self._temp_buffer = b''
            return buffered_data

        # Otherwise, pull data off the queue (blocking as needed). If this is
        # the end of the request, we're done here: mark ourselves as complete
        # and call it time. Otherwise, open the flow control window an
        # appropriate amount and hand the chunk off.
        chunk, chunk_size = self._received_data.get()
        if chunk is END_DATA_SENTINEL:
            self._complete = True
            raise StopIteration()

        # Let's talk a little bit about why we're opening the flow control
        # window *here*, and not in the server thread.
        #
        # The purpose of HTTP/2 flow control is to allow for servers and
        # clients to avoid needing to buffer data indefinitely because their
        # peer is producing data faster than they can consume it. As a result,
        # it's important that the flow control window be opened as late in the
        # processing as possible. In this case, we open the flow control window
        # exactly when the server hands the data to the application. This means
        # that the flow control window essentially signals to the remote peer
        # how much data hasn't even been *seen* by the application yet.
        #
        # If you wanted to be really clever you could consider not opening the
        # flow control window until the application asks for the *next* chunk
        # of data. That means that any buffers at the application level are now
        # included in the flow control window processing. In my opinion, the
        # advantage of that process does not outweigh the extra logical
        # complexity involved in doing it, so we don't bother here.
        #
        # Another note: you'll notice that we don't include the _temp_buffer in
        # our flow control considerations. This means you could in principle
        # lead us to buffer slightly more than one connection flow control
        # window's worth of data. That risk is considered acceptable for the
        # much simpler logic available here.
        #
        # Finally, this is a pretty dumb flow control window management scheme:
        # it causes us to emit a *lot* of window updates. A smarter server
        # would want to use the content-length header to determine whether
        # flow control window updates need to be emitted at all, and then to be
        # more efficient about emitting them to avoid firing them off really
        # frequently. For an example like this, there's very little gained by
        # worrying about that.
        self._protocol.open_flow_control_window(self.stream_id, chunk_size)

        return chunk


def _build_environ_dict(headers, stream):
    """
    Build the WSGI environ dictionary for a given request. To do that, we'll
    temporarily create a dictionary for the headers. While this isn't actually
    a valid way to represent headers, we know that the special headers we need
    can only have one appearance in the block.

    This code is arguably somewhat incautious: the conversion to dictionary
    should only happen in a way that allows us to correctly join headers that
    appear multiple times. That's acceptable in a demo app: in a productised
    version you'd want to fix it.
    """
    header_dict = dict(headers)
    path = header_dict.pop(u':path')
    try:
        path, query = path.split(u'?', 1)
    except ValueError:
        query = u""
    server_name = header_dict.pop(u':authority')
    try:
        server_name, port = server_name.split(u':', 1)
    except ValueError as e:
        port = "8443"

    environ = {
        u'REQUEST_METHOD': header_dict.pop(u':method'),
        u'SCRIPT_NAME': u'',
        u'PATH_INFO': path,
        u'QUERY_STRING': query,
        u'SERVER_NAME': server_name,
        u'SERVER_PORT': port,
        u'SERVER_PROTOCOL': u'HTTP/2',
        u'HTTPS': u"on",
        u'SSL_PROTOCOL': u'TLSv1.2',
        u'wsgi.version': (1, 0),
        u'wsgi.url_scheme': header_dict.pop(u':scheme'),
        u'wsgi.input': stream,
        u'wsgi.errors': sys.stderr,
        u'wsgi.multithread': True,
        u'wsgi.multiprocess': False,
        u'wsgi.run_once': False,
    }
    if u'content-type' in header_dict:
        environ[u'CONTENT_TYPE'] = header_dict[u'content-type']
    if u'content-length' in header_dict:
        environ[u'CONTENT_LENGTH'] = header_dict[u'content-length']
    for name, value in header_dict.items():
        environ[u'HTTP_' + name.upper()] = value
    return environ


# Set up the WSGI app.
application_string = sys.argv[1]
path, func = application_string.split(':', 1)
module = importlib.import_module(path)
APPLICATION = getattr(module, func)

# Set up TLS
ssl_context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
ssl_context.options |= (
    ssl.OP_NO_TLSv1 | ssl.OP_NO_TLSv1_1 | ssl.OP_NO_COMPRESSION
)
ssl_context.set_ciphers("ECDHE+AESGCM")
ssl_context.load_cert_chain(certfile="cert.crt", keyfile="cert.key")
ssl_context.set_alpn_protocols(["h2"])

# Do the asnycio bits
loop = asyncio.get_event_loop()
# Each client connection will create a new protocol instance
coro = loop.create_server(H2Protocol, '127.0.0.1', 8443, ssl=ssl_context)
server = loop.run_until_complete(coro)

# Serve requests until Ctrl+C is pressed
print('Serving on {}'.format(server.sockets[0].getsockname()))
try:
    loop.run_forever()
except KeyboardInterrupt:
    pass

# Close the server
server.close()
loop.run_until_complete(server.wait_closed())
loop.close()

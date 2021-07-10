Design
======

.. currentmodule:: websockets

This document describes the design of ``websockets``. It assumes familiarity
with the specification of the WebSocket protocol in :rfc:`6455`.

It's primarily intended at maintainers. It may also be useful for users who
wish to understand what happens under the hood.

.. warning::

    Internals described in this document may change at any time.

    Backwards compatibility is only guaranteed for `public APIs <api>`_.


Lifecycle
---------

State
.....

WebSocket connections go through a trivial state machine:

- ``CONNECTING``: initial state,
- ``OPEN``: when the opening handshake is complete,
- ``CLOSING``: when the closing handshake is started,
- ``CLOSED``: when the TCP connection is closed.

Transitions happen in the following places:

- ``CONNECTING -> OPEN``: in
  :meth:`~protocol.WebSocketCommonProtocol.connection_open` which runs when
  the :ref:`opening handshake <opening-handshake>` completes and the WebSocket
  connection is established — not to be confused with
  :meth:`~asyncio.Protocol.connection_made` which runs when the TCP connection
  is established;
- ``OPEN -> CLOSING``: in
  :meth:`~protocol.WebSocketCommonProtocol.write_frame` immediately before
  sending a close frame; since receiving a close frame triggers sending a
  close frame, this does the right thing regardless of which side started the
  :ref:`closing handshake <closing-handshake>`; also in
  :meth:`~protocol.WebSocketCommonProtocol.fail_connection` which duplicates
  a few lines of code from ``write_close_frame()`` and ``write_frame()``;
- ``* -> CLOSED``: in
  :meth:`~protocol.WebSocketCommonProtocol.connection_lost` which is always
  called exactly once when the TCP connection is closed.

Coroutines
..........

The following diagram shows which coroutines are running at each stage of the
connection lifecycle on the client side.

.. image:: lifecycle.svg
   :target: _images/lifecycle.svg

The lifecycle is identical on the server side, except inversion of control
makes the equivalent of :meth:`~client.connect` implicit.

Coroutines shown in green are called by the application. Multiple coroutines
may interact with the WebSocket connection concurrently.

Coroutines shown in gray manage the connection. When the opening handshake
succeeds, :meth:`~protocol.WebSocketCommonProtocol.connection_open` starts
two tasks:

- :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` runs
  :meth:`~protocol.WebSocketCommonProtocol.transfer_data` which handles
  incoming data and lets :meth:`~protocol.WebSocketCommonProtocol.recv`
  consume it. It may be canceled to terminate the connection. It never exits
  with an exception other than :exc:`~asyncio.CancelledError`. See :ref:`data
  transfer <data-transfer>` below.

- :attr:`~protocol.WebSocketCommonProtocol.keepalive_ping_task` runs
  :meth:`~protocol.WebSocketCommonProtocol.keepalive_ping` which sends Ping
  frames at regular intervals and ensures that corresponding Pong frames are
  received. It is canceled when the connection terminates. It never exits
  with an exception other than :exc:`~asyncio.CancelledError`.

- :attr:`~protocol.WebSocketCommonProtocol.close_connection_task` runs
  :meth:`~protocol.WebSocketCommonProtocol.close_connection` which waits for
  the data transfer to terminate, then takes care of closing the TCP
  connection. It must not be canceled. It never exits with an exception. See
  :ref:`connection termination <connection-termination>` below.

Besides, :meth:`~protocol.WebSocketCommonProtocol.fail_connection` starts
the same :attr:`~protocol.WebSocketCommonProtocol.close_connection_task` when
the opening handshake fails, in order to close the TCP connection.

Splitting the responsibilities between two tasks makes it easier to guarantee
that ``websockets`` can terminate connections:

- within a fixed timeout,
- without leaking pending tasks,
- without leaking open TCP connections,

regardless of whether the connection terminates normally or abnormally.

:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` completes when no
more data will be received on the connection. Under normal circumstances, it
exits after exchanging close frames.

:attr:`~protocol.WebSocketCommonProtocol.close_connection_task` completes when
the TCP connection is closed.


.. _opening-handshake:

Opening handshake
-----------------

``websockets`` performs the opening handshake when establishing a WebSocket
connection. On the client side, :meth:`~client.connect` executes it before
returning the protocol to the caller. On the server side, it's executed before
passing the protocol to the ``ws_handler`` coroutine handling the connection.

While the opening handshake is asymmetrical — the client sends an HTTP Upgrade
request and the server replies with an HTTP Switching Protocols response —
``websockets`` aims at keeping the implementation of both sides consistent
with one another.

On the client side, :meth:`~client.WebSocketClientProtocol.handshake`:

- builds a HTTP request based on the ``uri`` and parameters passed to
  :meth:`~client.connect`;
- writes the HTTP request to the network;
- reads a HTTP response from the network;
- checks the HTTP response, validates ``extensions`` and ``subprotocol``, and
  configures the protocol accordingly;
- moves to the ``OPEN`` state.

On the server side, :meth:`~server.WebSocketServerProtocol.handshake`:

- reads a HTTP request from the network;
- calls :meth:`~server.WebSocketServerProtocol.process_request` which may
  abort the WebSocket handshake and return a HTTP response instead; this
  hook only makes sense on the server side;
- checks the HTTP request, negotiates ``extensions`` and ``subprotocol``, and
  configures the protocol accordingly;
- builds a HTTP response based on the above and parameters passed to
  :meth:`~server.serve`;
- writes the HTTP response to the network;
- moves to the ``OPEN`` state;
- returns the ``path`` part of the ``uri``.

The most significant asymmetry between the two sides of the opening handshake
lies in the negotiation of extensions and, to a lesser extent, of the
subprotocol. The server knows everything about both sides and decides what the
parameters should be for the connection. The client merely applies them.

If anything goes wrong during the opening handshake, ``websockets``
:ref:`fails the connection <connection-failure>`.


.. _data-transfer:

Data transfer
-------------

Symmetry
........

Once the opening handshake has completed, the WebSocket protocol enters the
data transfer phase. This part is almost symmetrical. There are only two
differences between a server and a client:

- `client-to-server masking`_: the client masks outgoing frames; the server
  unmasks incoming frames;
- `closing the TCP connection`_: the server closes the connection immediately;
  the client waits for the server to do it.

.. _client-to-server masking: https://tools.ietf.org/html/rfc6455#section-5.3
.. _closing the TCP connection: https://tools.ietf.org/html/rfc6455#section-5.5.1

These differences are so minor that all the logic for `data framing`_, for
`sending and receiving data`_ and for `closing the connection`_ is implemented
in the same class, :class:`~protocol.WebSocketCommonProtocol`.

.. _data framing: https://tools.ietf.org/html/rfc6455#section-5
.. _sending and receiving data: https://tools.ietf.org/html/rfc6455#section-6
.. _closing the connection: https://tools.ietf.org/html/rfc6455#section-7

The :attr:`~protocol.WebSocketCommonProtocol.is_client` attribute tells which
side a protocol instance is managing. This attribute is defined on the
:attr:`~server.WebSocketServerProtocol` and
:attr:`~client.WebSocketClientProtocol` classes.

Data flow
.........

The following diagram shows how data flows between an application built on top
of ``websockets`` and a remote endpoint. It applies regardless of which side
is the server or the client.

.. image:: protocol.svg
   :target: _images/protocol.svg

Public methods are shown in green, private methods in yellow, and buffers in
orange. Methods related to connection termination are omitted; connection
termination is discussed in another section below.

Receiving data
..............

The left side of the diagram shows how ``websockets`` receives data.

Incoming data is written to a :class:`~asyncio.StreamReader` in order to
implement flow control and provide backpressure on the TCP connection.

:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`, which is started
when the WebSocket connection is established, processes this data.

When it receives data frames, it reassembles fragments and puts the resulting
messages in the :attr:`~protocol.WebSocketCommonProtocol.messages` queue.

When it encounters a control frame:

- if it's a close frame, it starts the closing handshake;
- if it's a ping frame, it answers with a pong frame;
- if it's a pong frame, it acknowledges the corresponding ping (unless it's an
  unsolicited pong).

Running this process in a task guarantees that control frames are processed
promptly. Without such a task, ``websockets`` would depend on the application
to drive the connection by having exactly one coroutine awaiting
:meth:`~protocol.WebSocketCommonProtocol.recv` at any time. While this
happens naturally in many use cases, it cannot be relied upon.

Then :meth:`~protocol.WebSocketCommonProtocol.recv` fetches the next message
from the :attr:`~protocol.WebSocketCommonProtocol.messages` queue, with some
complexity added for handling backpressure and termination correctly.

Sending data
............

The right side of the diagram shows how ``websockets`` sends data.

:meth:`~protocol.WebSocketCommonProtocol.send` writes one or several data
frames containing the message. While sending a fragmented message, concurrent
calls to :meth:`~protocol.WebSocketCommonProtocol.send` are put on hold until
all fragments are sent. This makes concurrent calls safe.

:meth:`~protocol.WebSocketCommonProtocol.ping` writes a ping frame and
yields a :class:`~asyncio.Future` which will be completed when a matching pong
frame is received.

:meth:`~protocol.WebSocketCommonProtocol.pong` writes a pong frame.

:meth:`~protocol.WebSocketCommonProtocol.close` writes a close frame and
waits for the TCP connection to terminate.

Outgoing data is written to a :class:`~asyncio.StreamWriter` in order to
implement flow control and provide backpressure from the TCP connection.

.. _closing-handshake:

Closing handshake
.................

When the other side of the connection initiates the closing handshake,
:meth:`~protocol.WebSocketCommonProtocol.read_message` receives a close
frame while in the ``OPEN`` state. It moves to the ``CLOSING`` state, sends a
close frame, and returns ``None``, causing
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` to terminate.

When this side of the connection initiates the closing handshake with
:meth:`~protocol.WebSocketCommonProtocol.close`, it moves to the ``CLOSING``
state and sends a close frame. When the other side sends a close frame,
:meth:`~protocol.WebSocketCommonProtocol.read_message` receives it in the
``CLOSING`` state and returns ``None``, also causing
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` to terminate.

If the other side doesn't send a close frame within the connection's close
timeout, ``websockets`` :ref:`fails the connection <connection-failure>`.

The closing handshake can take up to ``2 * close_timeout``: one
``close_timeout`` to write a close frame and one ``close_timeout`` to receive
a close frame.

Then ``websockets`` terminates the TCP connection.


.. _connection-termination:

Connection termination
----------------------

:attr:`~protocol.WebSocketCommonProtocol.close_connection_task`, which is
started when the WebSocket connection is established, is responsible for
eventually closing the TCP connection.

First :attr:`~protocol.WebSocketCommonProtocol.close_connection_task` waits
for :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` to terminate,
which may happen as a result of:

- a successful closing handshake: as explained above, this exits the infinite
  loop in :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`;
- a timeout while waiting for the closing handshake to complete: this cancels
  :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`;
- a protocol error, including connection errors: depending on the exception,
  :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` :ref:`fails the
  connection <connection-failure>` with a suitable code and exits.

:attr:`~protocol.WebSocketCommonProtocol.close_connection_task` is separate
from :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` to make it
easier to implement the timeout on the closing handshake. Canceling
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` creates no risk
of canceling :attr:`~protocol.WebSocketCommonProtocol.close_connection_task`
and failing to close the TCP connection, thus leaking resources.

Then :attr:`~protocol.WebSocketCommonProtocol.close_connection_task` cancels
:attr:`~protocol.WebSocketCommonProtocol.keepalive_ping`. This task has no
protocol compliance responsibilities. Terminating it to avoid leaking it is
the only concern.

Terminating the TCP connection can take up to ``2 * close_timeout`` on the
server side and ``3 * close_timeout`` on the client side. Clients start by
waiting for the server to close the connection, hence the extra
``close_timeout``. Then both sides go through the following steps until the
TCP connection is lost: half-closing the connection (only for non-TLS
connections), closing the connection, aborting the connection. At this point
the connection drops regardless of what happens on the network.


.. _connection-failure:

Connection failure
------------------

If the opening handshake doesn't complete successfully, ``websockets`` fails
the connection by closing the TCP connection.

Once the opening handshake has completed, ``websockets`` fails the connection
by canceling :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` and
sending a close frame if appropriate.

:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` exits, unblocking
:attr:`~protocol.WebSocketCommonProtocol.close_connection_task`, which closes
the TCP connection.


.. _server-shutdown:

Server shutdown
---------------

:class:`~websockets.server.WebSocketServer` closes asynchronously like
:class:`asyncio.Server`. The shutdown happen in two steps:

1. Stop listening and accepting new connections;
2. Close established connections with close code 1001 (going away) or, if
   the opening handshake is still in progress, with HTTP status code 503
   (Service Unavailable).

The first call to :class:`~websockets.server.WebSocketServer.close` starts a
task that performs this sequence. Further calls are ignored. This is the
easiest way to make :class:`~websockets.server.WebSocketServer.close` and
:class:`~websockets.server.WebSocketServer.wait_closed` idempotent.


.. _cancellation:

Cancellation
------------

User code
.........

``websockets`` provides a WebSocket application server. It manages connections
and passes them to user-provided connection handlers. This is an *inversion of
control* scenario: library code calls user code.

If a connection drops, the corresponding handler should terminate. If the
server shuts down, all connection handlers must terminate. Canceling
connection handlers would terminate them.

However, using cancellation for this purpose would require all connection
handlers to handle it properly. For example, if a connection handler starts
some tasks, it should catch :exc:`~asyncio.CancelledError`, terminate or
cancel these tasks, and then re-raise the exception.

Cancellation is tricky in :mod:`asyncio` applications, especially when it
interacts with finalization logic. In the example above, what if a handler
gets interrupted with :exc:`~asyncio.CancelledError` while it's finalizing
the tasks it started, after detecting that the connection dropped?

``websockets`` considers that cancellation may only be triggered by the caller
of a coroutine when it doesn't care about the results of that coroutine
anymore. (Source: `Guido van Rossum <https://groups.google.com/forum/#!msg
/python-tulip/LZQe38CR3bg/7qZ1p_q5yycJ>`_). Since connection handlers run
arbitrary user code, ``websockets`` has no way of deciding whether that code
is still doing something worth caring about.

For these reasons, ``websockets`` never cancels connection handlers. Instead
it expects them to detect when the connection is closed, execute finalization
logic if needed, and exit.

Conversely, cancellation isn't a concern for WebSocket clients because they
don't involve inversion of control.

Library
.......

Most :doc:`public APIs <api>` of ``websockets`` are coroutines. They may be
canceled, for example if the user starts a task that calls these coroutines
and cancels the task later. ``websockets`` must handle this situation.

Cancellation during the opening handshake is handled like any other exception:
the TCP connection is closed and the exception is re-raised. This can only
happen on the client side. On the server side, the opening handshake is
managed by ``websockets`` and nothing results in a cancellation.

Once the WebSocket connection is established, internal tasks
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` and
:attr:`~protocol.WebSocketCommonProtocol.close_connection_task` mustn't get
accidentally canceled if a coroutine that awaits them is canceled. In other
words, they must be shielded from cancellation.

:meth:`~protocol.WebSocketCommonProtocol.recv` waits for the next message in
the queue or for :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`
to terminate, whichever comes first. It relies on :func:`~asyncio.wait` for
waiting on two futures in parallel. As a consequence, even though it's waiting
on a :class:`~asyncio.Future` signaling the next message and on
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`, it doesn't
propagate cancellation to them.

:meth:`~protocol.WebSocketCommonProtocol.ensure_open` is called by
:meth:`~protocol.WebSocketCommonProtocol.send`,
:meth:`~protocol.WebSocketCommonProtocol.ping`, and
:meth:`~protocol.WebSocketCommonProtocol.pong`. When the connection state is
``CLOSING``, it waits for
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` but shields it to
prevent cancellation.

:meth:`~protocol.WebSocketCommonProtocol.close` waits for the data transfer
task to terminate with :func:`~asyncio.wait_for`. If it's canceled or if the
timeout elapses, :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`
is canceled, which is correct at this point.
:meth:`~protocol.WebSocketCommonProtocol.close` then waits for
:attr:`~protocol.WebSocketCommonProtocol.close_connection_task` but shields it
to prevent cancellation.

:meth:`~protocol.WebSocketCommonProtocol.close` and
:func:`~protocol.WebSocketCommonProtocol.fail_connection` are the only
places where :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` may
be canceled.

:attr:`~protocol.WebSocketCommonProtocol.close_connnection_task` starts by
waiting for :attr:`~protocol.WebSocketCommonProtocol.transfer_data_task`. It
catches :exc:`~asyncio.CancelledError` to prevent a cancellation of
:attr:`~protocol.WebSocketCommonProtocol.transfer_data_task` from propagating
to :attr:`~protocol.WebSocketCommonProtocol.close_connnection_task`.

.. _backpressure:

Backpressure
------------

.. note::

    This section discusses backpressure from the perspective of a server but
    the concept applies to clients symmetrically.

With a naive implementation, if a server receives inputs faster than it can
process them, or if it generates outputs faster than it can send them, data
accumulates in buffers, eventually causing the server to run out of memory and
crash.

The solution to this problem is backpressure. Any part of the server that
receives inputs faster than it can process them and send the outputs
must propagate that information back to the previous part in the chain.

``websockets`` is designed to make it easy to get backpressure right.

For incoming data, ``websockets`` builds upon :class:`~asyncio.StreamReader`
which propagates backpressure to its own buffer and to the TCP stream. Frames
are parsed from the input stream and added to a bounded queue. If the queue
fills up, parsing halts until the application reads a frame.

For outgoing data, ``websockets`` builds upon :class:`~asyncio.StreamWriter`
which implements flow control. If the output buffers grow too large, it waits
until they're drained. That's why all APIs that write frames are asynchronous.

Of course, it's still possible for an application to create its own unbounded
buffers and break the backpressure. Be careful with queues.


.. _buffers:

Buffers
-------

.. note::

    This section discusses buffers from the perspective of a server but it
    applies to clients as well.

An asynchronous systems works best when its buffers are almost always empty.

For example, if a client sends data too fast for a server, the queue of
incoming messages will be constantly full. The server will always be 32
messages (by default) behind the client. This consumes memory and increases
latency for no good reason. The problem is called bufferbloat.

If buffers are almost always full and that problem cannot be solved by adding
capacity — typically because the system is bottlenecked by the output and
constantly regulated by backpressure — reducing the size of buffers minimizes
negative consequences.

By default ``websockets`` has rather high limits. You can decrease them
according to your application's characteristics.

Bufferbloat can happen at every level in the stack where there is a buffer.
For each connection, the receiving side contains these buffers:

- OS buffers: tuning them is an advanced optimization.
- :class:`~asyncio.StreamReader` bytes buffer: the default limit is 64 KiB.
  You can set another limit by passing a ``read_limit`` keyword argument to
  :func:`~client.connect()` or :func:`~server.serve`.
- Incoming messages :class:`~collections.deque`: its size depends both on
  the size and the number of messages it contains. By default the maximum
  UTF-8 encoded size is 1 MiB and the maximum number is 32. In the worst case,
  after UTF-8 decoding, a single message could take up to 4 MiB of memory and
  the overall memory consumption could reach 128 MiB. You should adjust these
  limits by setting the ``max_size`` and ``max_queue`` keyword arguments of
  :func:`~client.connect()` or :func:`~server.serve` according to your
  application's requirements.

For each connection, the sending side contains these buffers:

- :class:`~asyncio.StreamWriter` bytes buffer: the default size is 64 KiB.
  You can set another limit by passing a ``write_limit`` keyword argument to
  :func:`~client.connect()` or :func:`~server.serve`.
- OS buffers: tuning them is an advanced optimization.

Concurrency
-----------

Awaiting any combination of :meth:`~protocol.WebSocketCommonProtocol.recv`,
:meth:`~protocol.WebSocketCommonProtocol.send`,
:meth:`~protocol.WebSocketCommonProtocol.close`
:meth:`~protocol.WebSocketCommonProtocol.ping`, or
:meth:`~protocol.WebSocketCommonProtocol.pong` concurrently is safe, including
multiple calls to the same method, with one exception and one limitation.

* **Only one coroutine can receive messages at a time.** This constraint
  avoids non-deterministic behavior (and simplifies the implementation). If a
  coroutine is awaiting :meth:`~protocol.WebSocketCommonProtocol.recv`,
  awaiting it again in another coroutine raises :exc:`RuntimeError`.

* **Sending a fragmented message forces serialization.** Indeed, the WebSocket
  protocol doesn't support multiplexing messages. If a coroutine is awaiting
  :meth:`~protocol.WebSocketCommonProtocol.send` to send a fragmented message,
  awaiting it again in another coroutine waits until the first call completes.
  This will be transparent in many cases. It may be a concern if the
  fragmented message is generated slowly by an asynchronous iterator.

Receiving frames is independent from sending frames. This isolates
:meth:`~protocol.WebSocketCommonProtocol.recv`, which receives frames, from
the other methods, which send frames.

While the connection is open, each frame is sent with a single write. Combined
with the concurrency model of :mod:`asyncio`, this enforces serialization. The
only other requirement is to prevent interleaving other data frames in the
middle of a fragmented message.

After the connection is closed, sending a frame raises
:exc:`~websockets.exceptions.ConnectionClosed`, which is safe.

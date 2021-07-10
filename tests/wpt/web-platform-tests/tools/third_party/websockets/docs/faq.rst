FAQ
===

.. currentmodule:: websockets

.. note::

    Many questions asked in :mod:`websockets`' issue tracker are actually
    about :mod:`asyncio`. Python's documentation about `developing with
    asyncio`_ is a good complement.

    .. _developing with asyncio: https://docs.python.org/3/library/asyncio-dev.html

Server side
-----------

Why does the server close the connection after processing one message?
......................................................................

Your connection handler exits after processing one message. Write a loop to
process multiple messages.

For example, if your handler looks like this::

    async def handler(websocket, path):
        print(websocket.recv())

change it like this::

    async def handler(websocket, path):
        async for message in websocket:
            print(message)

*Don't feel bad if this happens to you — it's the most common question in
websockets' issue tracker :-)*

Why can only one client connect at a time?
..........................................

Your connection handler blocks the event loop. Look for blocking calls.
Any call that may take some time must be asynchronous.

For example, if you have::

    async def handler(websocket, path):
        time.sleep(1)

change it to::

    async def handler(websocket, path):
        await asyncio.sleep(1)

This is part of learning asyncio. It isn't specific to websockets.

See also Python's documentation about `running blocking code`_.

.. _running blocking code: https://docs.python.org/3/library/asyncio-dev.html#running-blocking-code

How do I get access HTTP headers, for example cookies?
......................................................

To access HTTP headers during the WebSocket handshake, you can override
:attr:`~server.WebSocketServerProtocol.process_request`::

    async def process_request(self, path, request_headers):
        cookies = request_header["Cookie"]

See

Once the connection is established, they're available in
:attr:`~protocol.WebSocketServerProtocol.request_headers`::

    async def handler(websocket, path):
        cookies = websocket.request_headers["Cookie"]

How do I get the IP address of the client connecting to my server?
..................................................................

It's available in :attr:`~protocol.WebSocketCommonProtocol.remote_address`::

    async def handler(websocket, path):
        remote_ip = websocket.remote_address[0]

How do I set which IP addresses my server listens to?
.....................................................

Look at the ``host`` argument of :meth:`~asyncio.loop.create_server`.

:func:`serve` accepts the same arguments as
:meth:`~asyncio.loop.create_server`.

How do I close a connection properly?
.....................................

websockets takes care of closing the connection when the handler exits.

How do I run a HTTP server and WebSocket server on the same port?
.................................................................

This isn't supported.

Providing a HTTP server is out of scope for websockets. It only aims at
providing a WebSocket server.

There's limited support for returning HTTP responses with the
:attr:`~server.WebSocketServerProtocol.process_request` hook.
If you need more, pick a HTTP server and run it separately.

Client side
-----------

How do I close a connection properly?
.....................................

The easiest is to use :func:`connect` as a context manager::

    async with connect(...) as websocket:
        ...

How do I reconnect automatically when the connection drops?
...........................................................

See `issue 414`_.

.. _issue 414: https://github.com/aaugustin/websockets/issues/414

How do I disable TLS/SSL certificate verification?
..................................................

Look at the ``ssl`` argument of :meth:`~asyncio.loop.create_connection`.

:func:`connect` accepts the same arguments as
:meth:`~asyncio.loop.create_connection`.

Both sides
----------

How do I do two things in parallel? How do I integrate with another coroutine?
..............................................................................

You must start two tasks, which the event loop will run concurrently. You can
achieve this with :func:`asyncio.gather` or :func:`asyncio.wait`.

This is also part of learning asyncio and not specific to websockets.

Keep track of the tasks and make sure they terminate or you cancel them when
the connection terminates.

How do I create channels or topics?
...................................

websockets doesn't have built-in publish / subscribe for these use cases.

Depending on the scale of your service, a simple in-memory implementation may
do the job or you may need an external publish / subscribe component.

What does ``ConnectionClosedError: code = 1006`` mean?
......................................................

If you're seeing this traceback in the logs of a server:

.. code-block:: pytb

    Error in connection handler
    Traceback (most recent call last):
      ...
    asyncio.streams.IncompleteReadError: 0 bytes read on a total of 2 expected bytes

    The above exception was the direct cause of the following exception:

    Traceback (most recent call last):
      ...
    websockets.exceptions.ConnectionClosedError: code = 1006 (connection closed abnormally [internal]), no reason

or if a client crashes with this traceback:

.. code-block:: pytb

    Traceback (most recent call last):
      ...
    ConnectionResetError: [Errno 54] Connection reset by peer

    The above exception was the direct cause of the following exception:

    Traceback (most recent call last):
      ...
    websockets.exceptions.ConnectionClosedError: code = 1006 (connection closed abnormally [internal]), no reason

it means that the TCP connection was lost. As a consequence, the WebSocket
connection was closed without receiving a close frame, which is abnormal.

You can catch and handle :exc:`~exceptions.ConnectionClosed` to prevent it
from being logged.

There are several reasons why long-lived connections may be lost:

* End-user devices tend to lose network connectivity often and unpredictably
  because they can move out of wireless network coverage, get unplugged from
  a wired network, enter airplane mode, be put to sleep, etc.
* HTTP load balancers or proxies that aren't configured for long-lived
  connections may terminate connections after a short amount of time, usually
  30 seconds.

If you're facing a reproducible issue, :ref:`enable debug logs <debugging>` to
see when and how connections are closed.

Are there ``onopen``, ``onmessage``, ``onerror``, and ``onclose`` callbacks?
............................................................................

No, there aren't.

websockets provides high-level, coroutine-based APIs. Compared to callbacks,
coroutines make it easier to manage control flow in concurrent code.

If you prefer callback-based APIs, you should use another library.

Can I use ``websockets`` synchronously, without ``async`` / ``await``?
......................................................................

You can convert every asynchronous call to a synchronous call by wrapping it
in ``asyncio.get_event_loop().run_until_complete(...)``.

If this turns out to be impractical, you should use another library.

Miscellaneous
-------------

How do I set a timeout on ``recv()``?
.....................................

Use :func:`~asyncio.wait_for`::

    await asyncio.wait_for(websocket.recv(), timeout=10)

This technique works for most APIs, except for asynchronous context managers.
See `issue 574`_.

.. _issue 574: https://github.com/aaugustin/websockets/issues/574

How do I keep idle connections open?
....................................

websockets sends pings at 20 seconds intervals to keep the connection open.

In closes the connection if it doesn't get a pong within 20 seconds.

You can adjust this behavior with ``ping_interval`` and ``ping_timeout``.

How do I respond to pings?
..........................

websockets takes care of responding to pings with pongs.

Is there a Python 2 version?
............................

No, there isn't.

websockets builds upon asyncio which requires Python 3.



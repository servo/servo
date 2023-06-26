Cheat sheet
===========

.. currentmodule:: websockets

Server
------

* Write a coroutine that handles a single connection. It receives a WebSocket
  protocol instance and the URI path in argument.

  * Call :meth:`~protocol.WebSocketCommonProtocol.recv` and
    :meth:`~protocol.WebSocketCommonProtocol.send` to receive and send
    messages at any time.

  * When :meth:`~protocol.WebSocketCommonProtocol.recv` or
    :meth:`~protocol.WebSocketCommonProtocol.send` raises
    :exc:`~exceptions.ConnectionClosed`, clean up and exit. If you started
    other :class:`asyncio.Task`, terminate them before exiting.

  * If you aren't awaiting :meth:`~protocol.WebSocketCommonProtocol.recv`,
    consider awaiting :meth:`~protocol.WebSocketCommonProtocol.wait_closed`
    to detect quickly when the connection is closed.

  * You may :meth:`~protocol.WebSocketCommonProtocol.ping` or
    :meth:`~protocol.WebSocketCommonProtocol.pong` if you wish but it isn't
    needed in general.

* Create a server with :func:`~server.serve` which is similar to asyncio's
  :meth:`~asyncio.AbstractEventLoop.create_server`. You can also use it as an
  asynchronous context manager.

  * The server takes care of establishing connections, then lets the handler
    execute the application logic, and finally closes the connection after the
    handler exits normally or with an exception.

  * For advanced customization, you may subclass
    :class:`~server.WebSocketServerProtocol` and pass either this subclass or
    a factory function as the ``create_protocol`` argument.

Client
------

* Create a client with :func:`~client.connect` which is similar to asyncio's
  :meth:`~asyncio.BaseEventLoop.create_connection`. You can also use it as an
  asynchronous context manager.

  * For advanced customization, you may subclass
    :class:`~server.WebSocketClientProtocol` and pass either this subclass or
    a factory function as the ``create_protocol`` argument.

* Call :meth:`~protocol.WebSocketCommonProtocol.recv` and
  :meth:`~protocol.WebSocketCommonProtocol.send` to receive and send messages
  at any time.

* You may :meth:`~protocol.WebSocketCommonProtocol.ping` or
  :meth:`~protocol.WebSocketCommonProtocol.pong` if you wish but it isn't
  needed in general.

* If you aren't using :func:`~client.connect` as a context manager, call
  :meth:`~protocol.WebSocketCommonProtocol.close` to terminate the connection.

.. _debugging:

Debugging
---------

If you don't understand what ``websockets`` is doing, enable logging::

    import logging
    logger = logging.getLogger('websockets')
    logger.setLevel(logging.INFO)
    logger.addHandler(logging.StreamHandler())

The logs contain:

* Exceptions in the connection handler at the ``ERROR`` level
* Exceptions in the opening or closing handshake at the ``INFO`` level
* All frames at the ``DEBUG`` level — this can be very verbose

If you're new to ``asyncio``, you will certainly encounter issues that are
related to asynchronous programming in general rather than to ``websockets``
in particular. Fortunately Python's official documentation provides advice to
`develop with asyncio`_. Check it out: it's invaluable!

.. _develop with asyncio: https://docs.python.org/3/library/asyncio-dev.html

Passing additional arguments to the connection handler
------------------------------------------------------

When writing a server, if you need to pass additional arguments to the
connection handler, you can bind them with :func:`functools.partial`::

    import asyncio
    import functools
    import websockets

    async def handler(websocket, path, extra_argument):
        ...

    bound_handler = functools.partial(handler, extra_argument='spam')
    start_server = websockets.serve(bound_handler, '127.0.0.1', 8765)

    asyncio.get_event_loop().run_until_complete(start_server)
    asyncio.get_event_loop().run_forever()

Another way to achieve this result is to define the ``handler`` coroutine in
a scope where the ``extra_argument`` variable exists instead of injecting it
through an argument.

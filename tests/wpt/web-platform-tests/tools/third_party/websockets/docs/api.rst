API
===

Design
------

``websockets`` provides complete client and server implementations, as shown
in the :doc:`getting started guide <intro>`. These functions are built on top
of low-level APIs reflecting the two phases of the WebSocket protocol:

1. An opening handshake, in the form of an HTTP Upgrade request;

2. Data transfer, as framed messages, ending with a closing handshake.

The first phase is designed to integrate with existing HTTP software.
``websockets`` provides a minimal implementation to build, parse and validate
HTTP requests and responses.

The second phase is the core of the WebSocket protocol. ``websockets``
provides a complete implementation on top of ``asyncio`` with a simple API.

For convenience, public APIs can be imported directly from the
:mod:`websockets` package, unless noted otherwise. Anything that isn't listed
in this document is a private API.

High-level
----------

Server
......

.. automodule:: websockets.server

    .. autofunction:: serve(ws_handler, host=None, port=None, *, create_protocol=None, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None, compression='deflate', origins=None, extensions=None, subprotocols=None, extra_headers=None, process_request=None, select_subprotocol=None, **kwds)
        :async:

    .. autofunction:: unix_serve(ws_handler, path, *, create_protocol=None, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None, compression='deflate', origins=None, extensions=None, subprotocols=None, extra_headers=None, process_request=None, select_subprotocol=None, **kwds)
        :async:


    .. autoclass:: WebSocketServer

        .. automethod:: close
        .. automethod:: wait_closed
        .. autoattribute:: sockets

    .. autoclass:: WebSocketServerProtocol(ws_handler, ws_server, *, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None, origins=None, extensions=None, subprotocols=None, extra_headers=None, process_request=None, select_subprotocol=None)

        .. automethod:: handshake
        .. automethod:: process_request
        .. automethod:: select_subprotocol

Client
......

.. automodule:: websockets.client

    .. autofunction:: connect(uri, *, create_protocol=None, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None, compression='deflate', origin=None, extensions=None, subprotocols=None, extra_headers=None, **kwds)
        :async:

    .. autofunction:: unix_connect(path, uri="ws://localhost/", *, create_protocol=None, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None, compression='deflate', origin=None, extensions=None, subprotocols=None, extra_headers=None, **kwds)
        :async:

    .. autoclass:: WebSocketClientProtocol(*, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None, origin=None, extensions=None, subprotocols=None, extra_headers=None)

        .. automethod:: handshake

Shared
......

.. automodule:: websockets.protocol

    .. autoclass:: WebSocketCommonProtocol(*, ping_interval=20, ping_timeout=20, close_timeout=10, max_size=2 ** 20, max_queue=2 ** 5, read_limit=2 ** 16, write_limit=2 ** 16, loop=None)

        .. automethod:: close
        .. automethod:: wait_closed

        .. automethod:: recv
        .. automethod:: send

        .. automethod:: ping
        .. automethod:: pong

        .. autoattribute:: local_address
        .. autoattribute:: remote_address

        .. autoattribute:: open
        .. autoattribute:: closed

Types
.....

.. automodule:: websockets.typing

    .. autodata:: Data


Per-Message Deflate Extension
.............................

.. automodule:: websockets.extensions.permessage_deflate

    .. autoclass:: ServerPerMessageDeflateFactory

    .. autoclass:: ClientPerMessageDeflateFactory

HTTP Basic Auth
...............

.. automodule:: websockets.auth

    .. autofunction:: basic_auth_protocol_factory

    .. autoclass:: BasicAuthWebSocketServerProtocol

        .. automethod:: process_request

Exceptions
..........

.. automodule:: websockets.exceptions
    :members:

Low-level
---------

Opening handshake
.................

.. automodule:: websockets.handshake
    :members:

Data transfer
.............

.. automodule:: websockets.framing
    :members:

URI parser
..........

.. automodule:: websockets.uri
    :members:

Utilities
.........

.. automodule:: websockets.headers
    :members:

.. automodule:: websockets.http
    :members:

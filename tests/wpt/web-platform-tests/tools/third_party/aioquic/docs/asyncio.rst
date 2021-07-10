asyncio API
===========

The asyncio API provides a high-level QUIC API built on top of :mod:`asyncio`,
Python's standard asynchronous I/O framework.

``aioquic`` comes with a selection of examples, including:

- an HTTP/3 client
- an HTTP/3 server

The examples can be browsed on GitHub:

https://github.com/aiortc/aioquic/tree/master/examples

.. automodule:: aioquic.asyncio

Client
------

    .. autofunction:: connect

Server
------

    .. autofunction:: serve

Common
------

    .. autoclass:: QuicConnectionProtocol
        :members:

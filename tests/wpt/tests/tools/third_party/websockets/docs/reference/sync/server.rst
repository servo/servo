Server (:mod:`threading`)
=========================

.. automodule:: websockets.sync.server

Creating a server
-----------------

.. autofunction:: serve(handler, host=None, port=None, *, sock=None, ssl_context=None, origins=None, extensions=None, subprotocols=None, select_subprotocol=None, process_request=None, process_response=None, server_header="Python/x.y.z websockets/X.Y", compression="deflate", open_timeout=10, close_timeout=10, max_size=2 ** 20, logger=None, create_connection=None)

.. autofunction:: unix_serve(handler, path=None, *, sock=None, ssl_context=None, origins=None, extensions=None, subprotocols=None, select_subprotocol=None, process_request=None, process_response=None, server_header="Python/x.y.z websockets/X.Y", compression="deflate", open_timeout=10, close_timeout=10, max_size=2 ** 20, logger=None, create_connection=None)

Running a server
----------------

.. autoclass:: WebSocketServer

    .. automethod:: serve_forever

    .. automethod:: shutdown

    .. automethod:: fileno

Using a connection
------------------

.. autoclass:: ServerConnection

    .. automethod:: __iter__

    .. automethod:: recv

    .. automethod:: recv_streaming

    .. automethod:: send

    .. automethod:: close

    .. automethod:: ping

    .. automethod:: pong

    WebSocket connection objects also provide these attributes:

    .. autoattribute:: id

    .. autoattribute:: logger

    .. autoproperty:: local_address

    .. autoproperty:: remote_address

    The following attributes are available after the opening handshake,
    once the WebSocket connection is open:

    .. autoattribute:: request

    .. autoattribute:: response

    .. autoproperty:: subprotocol

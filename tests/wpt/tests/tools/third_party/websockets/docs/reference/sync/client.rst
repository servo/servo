Client (:mod:`threading`)
=========================

.. automodule:: websockets.sync.client

Opening a connection
--------------------

.. autofunction:: connect(uri, *, sock=None, ssl_context=None, server_hostname=None, origin=None, extensions=None, subprotocols=None, additional_headers=None, user_agent_header="Python/x.y.z websockets/X.Y", compression="deflate", open_timeout=10, close_timeout=10, max_size=2 ** 20, logger=None, create_connection=None)

.. autofunction:: unix_connect(path, uri=None, *, sock=None, ssl_context=None, server_hostname=None, origin=None, extensions=None, subprotocols=None, additional_headers=None, user_agent_header="Python/x.y.z websockets/X.Y", compression="deflate", open_timeout=10, close_timeout=10, max_size=2 ** 20, logger=None, create_connection=None)

Using a connection
------------------

.. autoclass:: ClientConnection

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

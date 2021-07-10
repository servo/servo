Extensions
==========

.. currentmodule:: websockets

The WebSocket protocol supports extensions_.

At the time of writing, there's only one `registered extension`_, WebSocket
Per-Message Deflate, specified in :rfc:`7692`.

.. _extensions: https://tools.ietf.org/html/rfc6455#section-9
.. _registered extension: https://www.iana.org/assignments/websocket/websocket.xhtml#extension-name

Per-Message Deflate
-------------------

:func:`~server.serve()` and :func:`~client.connect` enable the Per-Message
Deflate extension by default. You can disable this with ``compression=None``.

You can also configure the Per-Message Deflate extension explicitly if you
want to customize its parameters.

.. _per-message-deflate-configuration-example:

Here's an example on the server side::

    import websockets
    from websockets.extensions import permessage_deflate

    websockets.serve(
        ...,
        extensions=[
            permessage_deflate.ServerPerMessageDeflateFactory(
                server_max_window_bits=11,
                client_max_window_bits=11,
                compress_settings={'memLevel': 4},
            ),
        ],
    )

Here's an example on the client side::

    import websockets
    from websockets.extensions import permessage_deflate

    websockets.connect(
        ...,
        extensions=[
            permessage_deflate.ClientPerMessageDeflateFactory(
                server_max_window_bits=11,
                client_max_window_bits=11,
                compress_settings={'memLevel': 4},
            ),
        ],
    )

Refer to the API documentation of
:class:`~extensions.permessage_deflate.ServerPerMessageDeflateFactory` and
:class:`~extensions.permessage_deflate.ClientPerMessageDeflateFactory` for
details.

Writing an extension
--------------------

During the opening handshake, WebSocket clients and servers negotiate which
extensions will be used with which parameters. Then each frame is processed by
extensions before it's sent and after it's received.

As a consequence writing an extension requires implementing several classes:

1. Extension Factory: it negotiates parameters and instantiates the extension.
   Clients and servers require separate extension factories with distinct APIs.

2. Extension: it decodes incoming frames and encodes outgoing frames. If the
   extension is symmetrical, clients and servers can use the same class.

``websockets`` provides abstract base classes for extension factories and
extensions.

.. autoclass:: websockets.extensions.base.ServerExtensionFactory
    :members:

.. autoclass:: websockets.extensions.base.ClientExtensionFactory
    :members:

.. autoclass:: websockets.extensions.base.Extension
    :members:

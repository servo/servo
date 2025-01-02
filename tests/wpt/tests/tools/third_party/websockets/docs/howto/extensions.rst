Write an extension
==================

.. currentmodule:: websockets.extensions

During the opening handshake, WebSocket clients and servers negotiate which
extensions_ will be used with which parameters. Then each frame is processed
by extensions before being sent or after being received.

.. _extensions: https://www.rfc-editor.org/rfc/rfc6455.html#section-9

As a consequence, writing an extension requires implementing several classes:

* Extension Factory: it negotiates parameters and instantiates the extension.

  Clients and servers require separate extension factories with distinct APIs.

  Extension factories are the public API of an extension.

* Extension: it decodes incoming frames and encodes outgoing frames.

  If the extension is symmetrical, clients and servers can use the same
  class.

  Extensions are initialized by extension factories, so they don't need to be
  part of the public API of an extension.

websockets provides base classes for extension factories and extensions.
See :class:`ClientExtensionFactory`, :class:`ServerExtensionFactory`,
and :class:`Extension` for details.

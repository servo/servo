Plain Sockets Example Client
============================

This example is a basic HTTP/2 client written using plain Python `sockets`_, and
`ssl`_ TLS/SSL wrapper for socket objects.

This client is *not* a complete production-ready HTTP/2 client and only intended
as a demonstration sample.

This example shows the bare minimum that is needed to send an HTTP/2 request to
a server, and read back a response body.

.. literalinclude:: ../../examples/plain_sockets/plain_sockets_client.py
   :language: python
   :linenos:
   :encoding: utf-8

.. _sockets: https://docs.python.org/3/library/socket.html
.. _ssl: https://docs.python.org/3/library/ssl.html

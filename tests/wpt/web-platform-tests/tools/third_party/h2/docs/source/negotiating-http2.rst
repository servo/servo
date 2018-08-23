Negotiating HTTP/2
==================

`RFC 7540`_ specifies three methods of negotiating HTTP/2 connections. This document outlines how to use Hyper-h2 with each one.

.. _starting-alpn:

HTTPS URLs (ALPN and NPN)
-------------------------

Starting HTTP/2 for HTTPS URLs is outlined in `RFC 7540 Section 3.3`_. In this case, the client and server use a TLS extension to negotiate HTTP/2: typically either or both of `NPN`_ or `ALPN`_. How to use NPN and ALPN is currently not covered in this document: please consult the documentation for either the :mod:`ssl module <python:ssl>` in the standard library, or the :mod:`PyOpenSSL <pyopenssl:OpenSSL.SSL>` third-party modules, for more on this topic.

This method is the simplest to use once the TLS connection is established. To use it with Hyper-h2, after you've established the connection and confirmed that HTTP/2 has been negotiated with `ALPN`_, create a :class:`H2Connection <h2.connection.H2Connection>` object and call :meth:`H2Connection.initiate_connection <h2.connection.H2Connection.initiate_connection>`. This will ensure that the appropriate preamble data is placed in the data buffer. You should then immediately send the data returned by :meth:`H2Connection.data_to_send <h2.connection.H2Connection.data_to_send>` on your TLS connection.

At this point, you're free to use all the HTTP/2 functionality provided by Hyper-h2.

Server Setup Example
~~~~~~~~~~~~~~~~~~~~

This example uses the APIs as defined in Python 3.5. If you are using an older version of Python you may not have access to the APIs used here. As noted above, please consult the documentation for the :mod:`ssl module <python:ssl>` to confirm.

.. literalinclude:: ../../examples/fragments/server_https_setup_fragment.py
   :language: python
   :linenos:
   :encoding: utf-8


Client Setup Example
~~~~~~~~~~~~~~~~~~~~

The client example is very similar to the server example above. The :class:`SSLContext <python:ssl.SSLContext>` object requires some minor changes, as does the :class:`H2Connection <h2.connection.H2Connection>`, but the bulk of the code is the same.

.. literalinclude:: ../../examples/fragments/client_https_setup_fragment.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _starting-upgrade:

HTTP URLs (Upgrade)
-------------------

Starting HTTP/2 for HTTP URLs is outlined in `RFC 7540 Section 3.2`_. In this case, the client and server use the HTTP Upgrade mechanism originally described in `RFC 7230 Section 6.7`_. The client sends its initial HTTP/1.1 request with two extra headers. The first is ``Upgrade: h2c``, which requests upgrade to cleartext HTTP/2. The second is a ``HTTP2-Settings`` header, which contains a specially formatted string that encodes a HTTP/2 Settings frame.

To do this with Hyper-h2 you have two slightly different flows: one for clients, one for servers.

Clients
~~~~~~~

For a client, when sending the first request you should manually add your ``Upgrade`` header. You should then create a :class:`H2Connection <h2.connection.H2Connection>` object and call :meth:`H2Connection.initiate_upgrade_connection <h2.connection.H2Connection.initiate_upgrade_connection>` with no arguments. This method will return a bytestring to use as the value of your ``HTTP2-Settings`` header.

If the server returns a ``101`` status code, it has accepted the upgrade, and you should immediately send the data returned by :meth:`H2Connection.data_to_send <h2.connection.H2Connection.data_to_send>`. Now you should consume the entire ``101`` header block. All data after the ``101`` header block is HTTP/2 data that should be fed directly to :meth:`H2Connection.receive_data <h2.connection.H2Connection.receive_data>` and handled as normal with Hyper-h2.

If the server does not return a ``101`` status code then it is not upgrading. Continue with HTTP/1.1 as normal: you may throw away your :class:`H2Connection <h2.connection.H2Connection>` object, as it is of no further use.

The server will respond to your original request in HTTP/2. Please pay attention to the events received from Hyper-h2, as they will define the server's response.

Client Example
^^^^^^^^^^^^^^

The code below demonstrates how to handle a plaintext upgrade from the perspective of the client. For the purposes of keeping the example code as simple and generic as possible it uses the synchronous socket API that comes with the Python standard library: if you want to use asynchronous I/O, you will need to translate this code to the appropriate idiom.

.. literalinclude:: ../../examples/fragments/client_upgrade_fragment.py
   :language: python
   :linenos:
   :encoding: utf-8


Servers
~~~~~~~

If the first request you receive on a connection from the client contains an ``Upgrade`` header with the ``h2c`` token in it, and you're willing to upgrade, you should create a :class:`H2Connection <h2.connection.H2Connection>` object and call :meth:`H2Connection.initiate_upgrade_connection <h2.connection.H2Connection.initiate_upgrade_connection>` with the value of the ``HTTP2-Settings`` header (as a bytestring) as the only argument.

Then, you should send back a ``101`` response that contains ``h2c`` in the ``Upgrade`` header. That response will inform the client that you're switching to HTTP/2. Then, you should immediately send the data that is returned to you by :meth:`H2Connection.data_to_send <h2.connection.H2Connection.data_to_send>` on the connection: this is a necessary part of the HTTP/2 upgrade process.

At this point, you may now respond to the original HTTP/1.1 request in HTTP/2 by calling the appropriate methods on the :class:`H2Connection <h2.connection.H2Connection>` object. No further HTTP/1.1 may be sent on this connection: from this point onward, all data sent by you and the client will be HTTP/2 data.

Server Example
^^^^^^^^^^^^^^

The code below demonstrates how to handle a plaintext upgrade from the perspective of the server. For the purposes of keeping the example code as simple and generic as possible it uses the synchronous socket API that comes with the Python standard library: if you want to use asynchronous I/O, you will need to translate this code to the appropriate idiom.

.. literalinclude:: ../../examples/fragments/server_upgrade_fragment.py
   :language: python
   :linenos:
   :encoding: utf-8


Prior Knowledge
---------------

It's possible that you as a client know that a particular server supports HTTP/2, and that you do not need to perform any of the negotiations described above. In that case, you may follow the steps in :ref:`starting-alpn`, ignoring all references to ALPN and NPN: there's no need to perform the upgrade dance described in :ref:`starting-upgrade`.

.. _RFC 7540: https://tools.ietf.org/html/rfc7540
.. _RFC 7540 Section 3.2: https://tools.ietf.org/html/rfc7540#section-3.2
.. _RFC 7540 Section 3.3: https://tools.ietf.org/html/rfc7540#section-3.3
.. _NPN: https://en.wikipedia.org/wiki/Application-Layer_Protocol_Negotiation
.. _ALPN: https://en.wikipedia.org/wiki/Application-Layer_Protocol_Negotiation
.. _RFC 7230 Section 6.7: https://tools.ietf.org/html/rfc7230#section-6.7

Quick start
===========

.. currentmodule:: websockets

Here are a few examples to get you started quickly with websockets.

Say "Hello world!"
------------------

Here's a WebSocket server.

It receives a name from the client, sends a greeting, and closes the connection.

.. literalinclude:: ../../example/quickstart/server.py
    :caption: server.py
    :language: python
    :linenos:

:func:`~server.serve` executes the connection handler coroutine ``hello()``
once for each WebSocket connection. It closes the WebSocket connection when
the handler returns.

Here's a corresponding WebSocket client.

It sends a name to the server, receives a greeting, and closes the connection.

.. literalinclude:: ../../example/quickstart/client.py
    :caption: client.py
    :language: python
    :linenos:

Using :func:`~client.connect` as an asynchronous context manager ensures the
WebSocket connection is closed.

.. _secure-server-example:

Encrypt connections
-------------------

Secure WebSocket connections improve confidentiality and also reliability
because they reduce the risk of interference by bad proxies.

The ``wss`` protocol is to ``ws`` what ``https`` is to ``http``. The
connection is encrypted with TLS_ (Transport Layer Security). ``wss``
requires certificates like ``https``.

.. _TLS: https://developer.mozilla.org/en-US/docs/Web/Security/Transport_Layer_Security

.. admonition:: TLS vs. SSL
    :class: tip

    TLS is sometimes referred to as SSL (Secure Sockets Layer). SSL was an
    earlier encryption protocol; the name stuck.

Here's how to adapt the server to encrypt connections. You must download
:download:`localhost.pem <../../example/quickstart/localhost.pem>` and save it
in the same directory as ``server_secure.py``.

.. literalinclude:: ../../example/quickstart/server_secure.py
    :caption: server_secure.py
    :language: python
    :linenos:

Here's how to adapt the client similarly.

.. literalinclude:: ../../example/quickstart/client_secure.py
    :caption: client_secure.py
    :language: python
    :linenos:

In this example, the client needs a TLS context because the server uses a
self-signed certificate.

When connecting to a secure WebSocket server with a valid certificate — any
certificate signed by a CA that your Python installation trusts — you can
simply pass ``ssl=True`` to :func:`~client.connect`.

.. admonition:: Configure the TLS context securely
    :class: attention

    This example demonstrates the ``ssl`` argument with a TLS certificate shared
    between the client and the server. This is a simplistic setup.

    Please review the advice and security considerations in the documentation of
    the :mod:`ssl` module to configure the TLS context securely.

Connect from a browser
----------------------

The WebSocket protocol was invented for the web — as the name says!

Here's how to connect to a WebSocket server from a browser.

Run this script in a console:

.. literalinclude:: ../../example/quickstart/show_time.py
    :caption: show_time.py
    :language: python
    :linenos:

Save this file as ``show_time.html``:

.. literalinclude:: ../../example/quickstart/show_time.html
    :caption: show_time.html
    :language: html
    :linenos:

Save this file as ``show_time.js``:

.. literalinclude:: ../../example/quickstart/show_time.js
    :caption: show_time.js
    :language: js
    :linenos:

Then, open ``show_time.html`` in several browsers. Clocks tick irregularly.

Broadcast messages
------------------

Let's change the previous example to send the same timestamps to all browsers,
instead of generating independent sequences for each client.

Stop the previous script if it's still running and run this script in a console:

.. literalinclude:: ../../example/quickstart/show_time_2.py
    :caption: show_time_2.py
    :language: python
    :linenos:

Refresh ``show_time.html`` in all browsers. Clocks tick in sync.

Manage application state
------------------------

A WebSocket server can receive events from clients, process them to update the
application state, and broadcast the updated state to all connected clients.

Here's an example where any client can increment or decrement a counter. The
concurrency model of :mod:`asyncio` guarantees that updates are serialized.

Run this script in a console:

.. literalinclude:: ../../example/quickstart/counter.py
    :caption: counter.py
    :language: python
    :linenos:

Save this file as ``counter.html``:

.. literalinclude:: ../../example/quickstart/counter.html
    :caption: counter.html
    :language: html
    :linenos:

Save this file as ``counter.css``:

.. literalinclude:: ../../example/quickstart/counter.css
    :caption: counter.css
    :language: css
    :linenos:

Save this file as ``counter.js``:

.. literalinclude:: ../../example/quickstart/counter.js
    :caption: counter.js
    :language: js
    :linenos:

Then open ``counter.html`` file in several browsers and play with [+] and [-].

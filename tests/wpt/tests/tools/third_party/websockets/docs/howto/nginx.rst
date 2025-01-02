Deploy behind nginx
===================

This guide demonstrates a way to load balance connections across multiple
websockets server processes running on the same machine with nginx_.

We'll run server processes with Supervisor as described in :doc:`this guide
<supervisor>`.

.. _nginx: https://nginx.org/

Run server processes
--------------------

Save this app to ``app.py``:

.. literalinclude:: ../../example/deployment/nginx/app.py
    :emphasize-lines: 21,23

We'd like to nginx to connect to websockets servers via Unix sockets in order
to avoid the overhead of TCP for communicating between processes running in
the same OS.

We start the app with :func:`~websockets.server.unix_serve`. Each server
process listens on a different socket thanks to an environment variable set
by Supervisor to a different value.

Save this configuration to ``supervisord.conf``:

.. literalinclude:: ../../example/deployment/nginx/supervisord.conf

This configuration runs four instances of the app.

Install Supervisor and run it:

.. code-block:: console

    $ supervisord -c supervisord.conf -n

Configure and run nginx
-----------------------

Here's a simple nginx configuration to load balance connections across four
processes:

.. literalinclude:: ../../example/deployment/nginx/nginx.conf

We set ``daemon off`` so we can run nginx in the foreground for testing.

Then we combine the `WebSocket proxying`_ and `load balancing`_ guides:

* The WebSocket protocol requires HTTP/1.1. We must set the HTTP protocol
  version to 1.1, else nginx defaults to HTTP/1.0 for proxying.

* The WebSocket handshake involves the ``Connection`` and ``Upgrade`` HTTP
  headers. We must pass them to the upstream explicitly, else nginx drops
  them because they're hop-by-hop headers.

  We deviate from the `WebSocket proxying`_ guide because its example adds a
  ``Connection: Upgrade`` header to every upstream request, even if the
  original request didn't contain that header.

* In the upstream configuration, we set the load balancing method to
  ``least_conn`` in order to balance the number of active connections across
  servers. This is best for long running connections.

.. _WebSocket proxying: http://nginx.org/en/docs/http/websocket.html
.. _load balancing: http://nginx.org/en/docs/http/load_balancing.html

Save the configuration to ``nginx.conf``, install nginx, and run it:

.. code-block:: console

    $ nginx -c nginx.conf -p .

You can confirm that nginx proxies connections properly:

.. code-block:: console

    $ PYTHONPATH=src python -m websockets ws://localhost:8080/
    Connected to ws://localhost:8080/.
    > Hello!
    < Hello!
    Connection closed: 1000 (OK).

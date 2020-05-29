Examples
========

After checking out the code using git you can run:

.. code-block:: console

   $ pip install -e .
   $ pip install aiofiles asgiref dnslib httpbin starlette wsproto


HTTP/3
------

HTTP/3 server
.............

You can run the example server, which handles both HTTP/0.9 and HTTP/3:

.. code-block:: console

   $ python examples/http3_server.py --certificate tests/ssl_cert.pem --private-key tests/ssl_key.pem

HTTP/3 client
.............

You can run the example client to perform an HTTP/3 request:

.. code-block:: console

  $ python examples/http3_client.py --ca-certs tests/pycacert.pem https://localhost:4433/

Alternatively you can perform an HTTP/0.9 request:

.. code-block:: console

  $ python examples/http3_client.py --ca-certs tests/pycacert.pem --legacy-http https://localhost:4433/

You can also open a WebSocket over HTTP/3:

.. code-block:: console

  $ python examples/http3_client.py --ca-certs tests/pycacert.pem wss://localhost:4433/ws


DNS over QUIC
-------------

By default the server will use the `Google Public DNS`_ service, you can
override this with the ``--resolver`` argument.

.. code-block:: console

    $ python examples/doq_server.py --certificate tests/ssl_cert.pem --private-key tests/ssl_key.pem

You can then run the client with a specific query:

.. code-block:: console

    $ python examples/doq_client.py --ca-certs tests/pycacert.pem --dns_type "A" --query "quic.aiortc.org" --port 4784

.. _Google Public DNS: https://developers.google.com/speed/public-dns

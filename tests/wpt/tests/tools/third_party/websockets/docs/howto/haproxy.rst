Deploy behind HAProxy
=====================

This guide demonstrates a way to load balance connections across multiple
websockets server processes running on the same machine with HAProxy_.

We'll run server processes with Supervisor as described in :doc:`this guide
<supervisor>`.

.. _HAProxy: https://www.haproxy.org/

Run server processes
--------------------

Save this app to ``app.py``:

.. literalinclude:: ../../example/deployment/haproxy/app.py
    :emphasize-lines: 24

Each server process listens on a different port by extracting an incremental
index from an environment variable set by Supervisor.

Save this configuration to ``supervisord.conf``:

.. literalinclude:: ../../example/deployment/haproxy/supervisord.conf

This configuration runs four instances of the app.

Install Supervisor and run it:

.. code-block:: console

    $ supervisord -c supervisord.conf -n

Configure and run HAProxy
-------------------------

Here's a simple HAProxy configuration to load balance connections across four
processes:

.. literalinclude:: ../../example/deployment/haproxy/haproxy.cfg

In the backend configuration, we set the load balancing method to
``leastconn`` in order to balance the number of active connections across
servers. This is best for long running connections.

Save the configuration to ``haproxy.cfg``, install HAProxy, and run it:

.. code-block:: console

    $ haproxy -f haproxy.cfg

You can confirm that HAProxy proxies connections properly:

.. code-block:: console

    $ PYTHONPATH=src python -m websockets ws://localhost:8080/
    Connected to ws://localhost:8080/.
    > Hello!
    < Hello!
    Connection closed: 1000 (OK).

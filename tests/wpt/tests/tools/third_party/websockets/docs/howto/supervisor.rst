Deploy with Supervisor
======================

This guide proposes a simple way to deploy a websockets server directly on a
Linux or BSD operating system.

We'll configure Supervisor_ to run several server processes and to restart
them if needed.

.. _Supervisor: http://supervisord.org/

We'll bind all servers to the same port. The OS will take care of balancing
connections.

Create and activate a virtualenv:

.. code-block:: console

    $ python -m venv supervisor-websockets
    $ . supervisor-websockets/bin/activate

Install websockets and Supervisor:

.. code-block:: console

    $ pip install websockets
    $ pip install supervisor

Save this app to a file called ``app.py``:

.. literalinclude:: ../../example/deployment/supervisor/app.py

This is an echo server with two features added for the purpose of this guide:

* It shuts down gracefully when receiving a ``SIGTERM`` signal;
* It enables the ``reuse_port`` option of :meth:`~asyncio.loop.create_server`,
  which in turns sets ``SO_REUSEPORT`` on the accept socket.

Save this Supervisor configuration to ``supervisord.conf``:

.. literalinclude:: ../../example/deployment/supervisor/supervisord.conf

This is the minimal configuration required to keep four instances of the app
running, restarting them if they exit.

Now start Supervisor in the foreground:

.. code-block:: console

    $ supervisord -c supervisord.conf -n
    INFO Increased RLIMIT_NOFILE limit to 1024
    INFO supervisord started with pid 43596
    INFO spawned: 'websockets-test_00' with pid 43597
    INFO spawned: 'websockets-test_01' with pid 43598
    INFO spawned: 'websockets-test_02' with pid 43599
    INFO spawned: 'websockets-test_03' with pid 43600
    INFO success: websockets-test_00 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
    INFO success: websockets-test_01 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
    INFO success: websockets-test_02 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)
    INFO success: websockets-test_03 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)

In another shell, after activating the virtualenv, we can connect to the app —
press Ctrl-D to exit:

.. code-block:: console

    $ python -m websockets ws://localhost:8080/
    Connected to ws://localhost:8080/.
    > Hello!
    < Hello!
    Connection closed: 1000 (OK).

Look at the pid of an instance of the app in the logs and terminate it:

.. code-block:: console

    $ kill -TERM 43597

The logs show that Supervisor restarted this instance:

.. code-block:: console

    INFO exited: websockets-test_00 (exit status 0; expected)
    INFO spawned: 'websockets-test_00' with pid 43629
    INFO success: websockets-test_00 entered RUNNING state, process has stayed up for > than 1 seconds (startsecs)

Now let's check what happens when we shut down Supervisor, but first let's
establish a connection and leave it open:

.. code-block:: console

    $ python -m websockets ws://localhost:8080/
    Connected to ws://localhost:8080/.
    >

Look at the pid of supervisord itself in the logs and terminate it:

.. code-block:: console

    $ kill -TERM 43596

The logs show that Supervisor terminated all instances of the app before
exiting:

.. code-block:: console

    WARN received SIGTERM indicating exit request
    INFO waiting for websockets-test_00, websockets-test_01, websockets-test_02, websockets-test_03 to die
    INFO stopped: websockets-test_02 (exit status 0)
    INFO stopped: websockets-test_03 (exit status 0)
    INFO stopped: websockets-test_01 (exit status 0)
    INFO stopped: websockets-test_00 (exit status 0)

And you can see that the connection to the app was closed gracefully:

.. code-block:: console

    $ python -m websockets ws://localhost:8080/
    Connected to ws://localhost:8080/.
    Connection closed: 1001 (going away).

In this example, we've been sharing the same virtualenv for supervisor and
websockets.

In a real deployment, you would likely:

* Install Supervisor with the package manager of the OS.
* Create a virtualenv dedicated to your application.
* Add ``environment=PATH="path/to/your/virtualenv/bin"`` in the Supervisor
  configuration. Then ``python app.py`` runs in that virtualenv.


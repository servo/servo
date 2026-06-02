Deploy to Fly
================

This guide describes how to deploy a websockets server to Fly_.

.. _Fly: https://fly.io/

.. admonition:: The free tier of Fly is sufficient for trying this guide.
    :class: tip

    The `free tier`__ include up to three small VMs. This guide uses only one.

    __ https://fly.io/docs/about/pricing/

We're going to deploy a very simple app. The process would be identical for a
more realistic app.

Create application
------------------

Here's the implementation of the app, an echo server. Save it in a file called
``app.py``:

.. literalinclude:: ../../example/deployment/fly/app.py
    :language: python

This app implements typical requirements for running on a Platform as a Service:

* it provides a health check at ``/healthz``;
* it closes connections and exits cleanly when it receives a ``SIGTERM`` signal.

Create a ``requirements.txt`` file containing this line to declare a dependency
on websockets:

.. literalinclude:: ../../example/deployment/fly/requirements.txt
    :language: text

The app is ready. Let's deploy it!

Deploy application
------------------

Follow the instructions__ to install the Fly CLI, if you haven't done that yet.

__  https://fly.io/docs/hands-on/install-flyctl/

Sign up or log in to Fly.

Launch the app — you'll have to pick a different name because I'm already using
``websockets-echo``:

.. code-block:: console

    $ fly launch
    Creating app in ...
    Scanning source code
    Detected a Python app
    Using the following build configuration:
        Builder: paketobuildpacks/builder:base
    ? App Name (leave blank to use an auto-generated name): websockets-echo
    ? Select organization: ...
    ? Select region: ...
    Created app websockets-echo in organization ...
    Wrote config file fly.toml
    ? Would you like to set up a Postgresql database now? No
    We have generated a simple Procfile for you. Modify it to fit your needs and run "fly deploy" to deploy your application.

.. admonition:: This will build the image with a generic buildpack.
    :class: tip

    Fly can `build images`__ with a Dockerfile or a buildpack. Here, ``fly
    launch`` configures a generic Paketo buildpack.

    If you'd rather package the app with a Dockerfile, check out the guide to
    :ref:`containerize an application <containerize-application>`.

    __ https://fly.io/docs/reference/builders/

Replace the auto-generated ``fly.toml`` with:

.. literalinclude:: ../../example/deployment/fly/fly.toml
    :language: toml

This configuration:

* listens on port 443, terminates TLS, and forwards to the app on port 8080;
* declares a health check at ``/healthz``;
* requests a ``SIGTERM`` for terminating the app.

Replace the auto-generated ``Procfile`` with:

.. literalinclude:: ../../example/deployment/fly/Procfile
    :language: text

This tells Fly how to run the app.

Now you can deploy it:

.. code-block:: console

    $ fly deploy

    ... lots of output...

    ==> Monitoring deployment

    1 desired, 1 placed, 1 healthy, 0 unhealthy [health checks: 1 total, 1 passing]
    --> v0 deployed successfully

Validate deployment
-------------------

Let's confirm that your application is running as expected.

Since it's a WebSocket server, you need a WebSocket client, such as the
interactive client that comes with websockets.

If you're currently building a websockets server, perhaps you're already in a
virtualenv where websockets is installed. If not, you can install it in a new
virtualenv as follows:

.. code-block:: console

    $ python -m venv websockets-client
    $ . websockets-client/bin/activate
    $ pip install websockets

Connect the interactive client — you must replace ``websockets-echo`` with the
name of your Fly app in this command:

.. code-block:: console

    $ python -m websockets wss://websockets-echo.fly.dev/
    Connected to wss://websockets-echo.fly.dev/.
    >

Great! Your app is running!

Once you're connected, you can send any message and the server will echo it,
or press Ctrl-D to terminate the connection:

.. code-block:: console

    > Hello!
    < Hello!
    Connection closed: 1000 (OK).

You can also confirm that your application shuts down gracefully.

Connect an interactive client again — remember to replace ``websockets-echo``
with your app:

.. code-block:: console

    $ python -m websockets wss://websockets-echo.fly.dev/
    Connected to wss://websockets-echo.fly.dev/.
    >

In another shell, restart the app — again, replace ``websockets-echo`` with your
app:

.. code-block:: console

    $ fly restart websockets-echo
    websockets-echo is being restarted

Go back to the first shell. The connection is closed with code 1001 (going
away).

.. code-block:: console

    $ python -m websockets wss://websockets-echo.fly.dev/
    Connected to wss://websockets-echo.fly.dev/.
    Connection closed: 1001 (going away).

If graceful shutdown wasn't working, the server wouldn't perform a closing
handshake and the connection would be closed with code 1006 (abnormal closure).

Deploy to Render
================

This guide describes how to deploy a websockets server to Render_.

.. _Render: https://render.com/

.. admonition:: The free plan of Render is sufficient for trying this guide.
    :class: tip

    However, on a `free plan`__, connections are dropped after five minutes,
    which is quite short for WebSocket application.

    __ https://render.com/docs/free

We're going to deploy a very simple app. The process would be identical for a
more realistic app.

Create repository
-----------------

Deploying to Render requires a git repository. Let's initialize one:

.. code-block:: console

    $ mkdir websockets-echo
    $ cd websockets-echo
    $ git init -b main
    Initialized empty Git repository in websockets-echo/.git/
    $ git commit --allow-empty -m "Initial commit."
    [main (root-commit) 816c3b1] Initial commit.

Render requires the git repository to be hosted at GitHub or GitLab.

Sign up or log in to GitHub. Create a new repository named ``websockets-echo``.
Don't enable any of the initialization options offered by GitHub. Then, follow
instructions for pushing an existing repository from the command line.

After pushing, refresh your repository's homepage on GitHub. You should see an
empty repository with an empty initial commit.

Create application
------------------

Here's the implementation of the app, an echo server. Save it in a file called
``app.py``:

.. literalinclude:: ../../example/deployment/render/app.py
    :language: python

This app implements requirements for `zero downtime deploys`_:

* it provides a health check at ``/healthz``;
* it closes connections and exits cleanly when it receives a ``SIGTERM`` signal.

.. _zero downtime deploys: https://render.com/docs/deploys#zero-downtime-deploys

Create a ``requirements.txt`` file containing this line to declare a dependency
on websockets:

.. literalinclude:: ../../example/deployment/render/requirements.txt
    :language: text

Confirm that you created the correct files and commit them to git:

.. code-block:: console

    $ ls
    app.py           requirements.txt
    $ git add .
    $ git commit -m "Initial implementation."
    [main f26bf7f] Initial implementation.
    2 files changed, 37 insertions(+)
    create mode 100644 app.py
    create mode 100644 requirements.txt

Push the changes to GitHub:

.. code-block:: console

    $ git push
    ...
    To github.com:<username>/websockets-echo.git
       816c3b1..f26bf7f  main -> main

The app is ready. Let's deploy it!

Deploy application
------------------

Sign up or log in to Render.

Create a new web service. Connect the git repository that you just created.

Then, finalize the configuration of your app as follows:

* **Name**: websockets-echo
* **Start Command**: ``python app.py``

If you're just experimenting, select the free plan. Create the web service.

To configure the health check, go to Settings, scroll down to Health & Alerts,
and set:

* **Health Check Path**: /healthz

This triggers a new deployment.

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
name of your Render app in this command:

.. code-block:: console

    $ python -m websockets wss://websockets-echo.onrender.com/
    Connected to wss://websockets-echo.onrender.com/.
    >

Great! Your app is running!

Once you're connected, you can send any message and the server will echo it,
or press Ctrl-D to terminate the connection:

.. code-block:: console

    > Hello!
    < Hello!
    Connection closed: 1000 (OK).

You can also confirm that your application shuts down gracefully when you deploy
a new version. Due to limitations of Render's free plan, you must upgrade to a
paid plan before you perform this test.

Connect an interactive client again — remember to replace ``websockets-echo``
with your app:

.. code-block:: console

    $ python -m websockets wss://websockets-echo.onrender.com/
    Connected to wss://websockets-echo.onrender.com/.
    >

Trigger a new deployment with Manual Deploy > Deploy latest commit. When the
deployment completes, the connection is closed with code 1001 (going away).

.. code-block:: console

    $ python -m websockets wss://websockets-echo.onrender.com/
    Connected to wss://websockets-echo.onrender.com/.
    Connection closed: 1001 (going away).

If graceful shutdown wasn't working, the server wouldn't perform a closing
handshake and the connection would be closed with code 1006 (abnormal closure).

Remember to downgrade to a free plan if you upgraded just for testing this feature.

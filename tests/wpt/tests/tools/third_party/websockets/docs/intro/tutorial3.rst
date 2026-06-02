Part 3 - Deploy to the web
==========================

.. currentmodule:: websockets

.. admonition:: This is the third part of the tutorial.

    * In the :doc:`first part <tutorial1>`, you created a server and
      connected one browser; you could play if you shared the same browser.
    * In this :doc:`second part <tutorial2>`, you connected a second browser;
      you could play from different browsers on a local network.
    * In this :doc:`third part <tutorial3>`, you will deploy the game to the
      web; you can play from any browser connected to the Internet.

In the first and second parts of the tutorial, for local development, you ran
an HTTP server on ``http://localhost:8000/`` with:

.. code-block:: console

    $ python -m http.server

and a WebSocket server on ``ws://localhost:8001/`` with:

.. code-block:: console

    $ python app.py

Now you want to deploy these servers on the Internet. There's a vast range of
hosting providers to choose from. For the sake of simplicity, we'll rely on:

* GitHub Pages for the HTTP server;
* Heroku for the WebSocket server.

Commit project to git
---------------------

Perhaps you committed your work to git while you were progressing through the
tutorial. If you didn't, now is a good time, because GitHub and Heroku offer
git-based deployment workflows.

Initialize a git repository:

.. code-block:: console

    $ git init -b main
    Initialized empty Git repository in websockets-tutorial/.git/
    $ git commit --allow-empty -m "Initial commit."
    [main (root-commit) ...] Initial commit.

Add all files and commit:

.. code-block:: console

    $ git add .
    $ git commit -m "Initial implementation of Connect Four game."
    [main ...] Initial implementation of Connect Four game.
     6 files changed, 500 insertions(+)
     create mode 100644 app.py
     create mode 100644 connect4.css
     create mode 100644 connect4.js
     create mode 100644 connect4.py
     create mode 100644 index.html
     create mode 100644 main.js

Prepare the WebSocket server
----------------------------

Before you deploy the server, you must adapt it to meet requirements of
Heroku's runtime. This involves two small changes:

1. Heroku expects the server to `listen on a specific port`_, provided in the
   ``$PORT`` environment variable.

2. Heroku sends a ``SIGTERM`` signal when `shutting down a dyno`_, which
   should trigger a clean exit.

.. _listen on a specific port: https://devcenter.heroku.com/articles/preparing-a-codebase-for-heroku-deployment#4-listen-on-the-correct-port

.. _shutting down a dyno: https://devcenter.heroku.com/articles/dynos#shutdown

Adapt the ``main()`` coroutine accordingly:

.. code-block:: python

    import os
    import signal

.. literalinclude:: ../../example/tutorial/step3/app.py
    :pyobject: main

To catch the ``SIGTERM`` signal, ``main()`` creates a :class:`~asyncio.Future`
called ``stop`` and registers a signal handler that sets the result of this
future. The value of the future doesn't matter; it's only for waiting for
``SIGTERM``.

Then, by using :func:`~server.serve` as a context manager and exiting the
context when ``stop`` has a result, ``main()`` ensures that the server closes
connections cleanly and exits on ``SIGTERM``.

The app is now fully compatible with Heroku.

Deploy the WebSocket server
---------------------------

Create a ``requirements.txt`` file with this content to install ``websockets``
when building the image:

.. literalinclude:: ../../example/tutorial/step3/requirements.txt
    :language: text

.. admonition:: Heroku treats ``requirements.txt`` as a signal to `detect a Python app`_.
    :class: tip

    That's why you don't need to declare that you need a Python runtime.

.. _detect a Python app: https://devcenter.heroku.com/articles/python-support#recognizing-a-python-app

Create a ``Procfile`` file with this content to configure the command for
running the server:

.. literalinclude:: ../../example/tutorial/step3/Procfile
    :language: text

Commit your changes:

.. code-block:: console

    $ git add .
    $ git commit -m "Deploy to Heroku."
    [main ...] Deploy to Heroku.
     3 files changed, 12 insertions(+), 2 deletions(-)
     create mode 100644 Procfile
     create mode 100644 requirements.txt

Follow the `set-up instructions`_ to install the Heroku CLI and to log in, if
you haven't done that yet.

.. _set-up instructions: https://devcenter.heroku.com/articles/getting-started-with-python#set-up

Create a Heroku app. You must choose a unique name and replace
``websockets-tutorial`` by this name in the following command:

.. code-block:: console

    $ heroku create websockets-tutorial
    Creating ⬢ websockets-tutorial... done
    https://websockets-tutorial.herokuapp.com/ | https://git.heroku.com/websockets-tutorial.git

If you reuse a name that someone else already uses, you will receive this
error; if this happens, try another name:

.. code-block:: console

    $ heroku create websockets-tutorial
    Creating ⬢ websockets-tutorial... !
     ▸    Name websockets-tutorial is already taken

Deploy by pushing the code to Heroku:

.. code-block:: console

    $ git push heroku

    ... lots of output...

    remote:        Released v1
    remote:        https://websockets-tutorial.herokuapp.com/ deployed to Heroku
    remote:
    remote: Verifying deploy... done.
    To https://git.heroku.com/websockets-tutorial.git
     * [new branch]      main -> main

You can test the WebSocket server with the interactive client exactly like you
did in the first part of the tutorial. Replace ``websockets-tutorial`` by the
name of your app in the following command:

.. code-block:: console

    $ python -m websockets wss://websockets-tutorial.herokuapp.com/
    Connected to wss://websockets-tutorial.herokuapp.com/.
    > {"type": "init"}
    < {"type": "init", "join": "54ICxFae_Ip7TJE2", "watch": "634w44TblL5Dbd9a"}
    Connection closed: 1000 (OK).

It works!

Prepare the web application
---------------------------

Before you deploy the web application, perhaps you're wondering how it will
locate the WebSocket server? Indeed, at this point, its address is hard-coded
in ``main.js``:

.. code-block:: javascript

  const websocket = new WebSocket("ws://localhost:8001/");

You can take this strategy one step further by checking the address of the
HTTP server and determining the address of the WebSocket server accordingly.

Add this function to ``main.js``; replace ``python-websockets`` by your GitHub
username and ``websockets-tutorial`` by the name of your app on Heroku:

.. literalinclude:: ../../example/tutorial/step3/main.js
    :language: js
    :start-at: function getWebSocketServer
    :end-before: function initGame

Then, update the initialization to connect to this address instead:

.. code-block:: javascript

  const websocket = new WebSocket(getWebSocketServer());

Commit your changes:

.. code-block:: console

    $ git add .
    $ git commit -m "Configure WebSocket server address."
    [main ...] Configure WebSocket server address.
     1 file changed, 11 insertions(+), 1 deletion(-)

Deploy the web application
--------------------------

Go to GitHub and create a new repository called ``websockets-tutorial``.

Push your code to this repository. You must replace ``python-websockets`` by
your GitHub username in the following command:

.. code-block:: console

    $ git remote add origin git@github.com:python-websockets/websockets-tutorial.git
    $ git push -u origin main
    Enumerating objects: 11, done.
    Counting objects: 100% (11/11), done.
    Delta compression using up to 8 threads
    Compressing objects: 100% (10/10), done.
    Writing objects: 100% (11/11), 5.90 KiB | 2.95 MiB/s, done.
    Total 11 (delta 0), reused 0 (delta 0), pack-reused 0
    To github.com:<username>/websockets-tutorial.git
     * [new branch]      main -> main
    Branch 'main' set up to track remote branch 'main' from 'origin'.

Go back to GitHub, open the Settings tab of the repository and select Pages in
the menu. Select the main branch as source and click Save. GitHub tells you
that your site is published.

Follow the link and start a game!

Summary
-------

In this third part of the tutorial, you learned how to deploy a WebSocket
application with Heroku.

You can start a Connect Four game, send the JOIN link to a friend, and play
over the Internet!

Congratulations for completing the tutorial. Enjoy building real-time web
applications with websockets!

Solution
--------

.. literalinclude:: ../../example/tutorial/step3/app.py
    :caption: app.py
    :language: python
    :linenos:

.. literalinclude:: ../../example/tutorial/step3/index.html
    :caption: index.html
    :language: html
    :linenos:

.. literalinclude:: ../../example/tutorial/step3/main.js
    :caption: main.js
    :language: js
    :linenos:

.. literalinclude:: ../../example/tutorial/step3/Procfile
    :caption: Procfile
    :language: text
    :linenos:

.. literalinclude:: ../../example/tutorial/step3/requirements.txt
    :caption: requirements.txt
    :language: text
    :linenos:

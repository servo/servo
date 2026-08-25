Part 1 - Send & receive
=======================

.. currentmodule:: websockets

In this tutorial, you're going to build a web-based `Connect Four`_ game.

.. _Connect Four: https://en.wikipedia.org/wiki/Connect_Four

The web removes the constraint of being in the same room for playing a game.
Two players can connect over of the Internet, regardless of where they are,
and play in their browsers.

When a player makes a move, it should be reflected immediately on both sides.
This is difficult to implement over HTTP due to the request-response style of
the protocol.

Indeed, there is no good way to be notified when the other player makes a
move. Workarounds such as polling or long-polling introduce significant
overhead.

Enter `WebSocket <websocket>`_.

The WebSocket protocol provides two-way communication between a browser and a
server over a persistent connection. That's exactly what you need to exchange
moves between players, via a server.

.. admonition:: This is the first part of the tutorial.

    * In this :doc:`first part <tutorial1>`, you will create a server and
      connect one browser; you can play if you share the same browser.
    * In the :doc:`second part <tutorial2>`, you will connect a second
      browser; you can play from different browsers on a local network.
    * In the :doc:`third part <tutorial3>`, you will deploy the game to the
      web; you can play from any browser connected to the Internet.

Prerequisites
-------------

This tutorial assumes basic knowledge of Python and JavaScript.

If you're comfortable with :doc:`virtual environments <python:tutorial/venv>`,
you can use one for this tutorial. Else, don't worry: websockets doesn't have
any dependencies; it shouldn't create trouble in the default environment.

If you haven't installed websockets yet, do it now:

.. code-block:: console

    $ pip install websockets

Confirm that websockets is installed:

.. code-block:: console

    $ python -m websockets --version

.. admonition:: This tutorial is written for websockets |release|.
    :class: tip

    If you installed another version, you should switch to the corresponding
    version of the documentation.

Download the starter kit
------------------------

Create a directory and download these three files:
:download:`connect4.js <../../example/tutorial/start/connect4.js>`,
:download:`connect4.css <../../example/tutorial/start/connect4.css>`,
and :download:`connect4.py <../../example/tutorial/start/connect4.py>`.

The JavaScript module, along with the CSS file, provides a web-based user
interface. Here's its API.

.. js:module:: connect4

.. js:data:: PLAYER1

    Color of the first player.

.. js:data:: PLAYER2

    Color of the second player.

.. js:function:: createBoard(board)

    Draw a board.

    :param board: DOM element containing the board; must be initially empty.

.. js:function:: playMove(board, player, column, row)

    Play a move.

    :param board: DOM element containing the board.
    :param player: :js:data:`PLAYER1` or :js:data:`PLAYER2`.
    :param column: between ``0`` and ``6``.
    :param row: between ``0`` and ``5``.

The Python module provides a class to record moves and tell when a player
wins. Here's its API.

.. module:: connect4

.. data:: PLAYER1
    :value: "red"

    Color of the first player.

.. data:: PLAYER2
    :value: "yellow"

    Color of the second player.

.. class:: Connect4

    A Connect Four game.

    .. method:: play(player, column)

        Play a move.

        :param player: :data:`~connect4.PLAYER1` or :data:`~connect4.PLAYER2`.
        :param column: between ``0`` and ``6``.
        :returns: Row where the checker lands, between ``0`` and ``5``.
        :raises RuntimeError: if the move is illegal.

    .. attribute:: moves

        List of moves played during this game, as ``(player, column, row)``
        tuples.

    .. attribute:: winner

        :data:`~connect4.PLAYER1` or :data:`~connect4.PLAYER2` if they
        won; :obj:`None` if the game is still ongoing.

.. currentmodule:: websockets

Bootstrap the web UI
--------------------

Create an ``index.html`` file next to ``connect4.js`` and ``connect4.css``
with this content:

.. literalinclude:: ../../example/tutorial/step1/index.html
    :language: html

This HTML page contains an empty ``<div>`` element where you will draw the
Connect Four board. It loads a ``main.js`` script where you will write all
your JavaScript code.

Create a ``main.js`` file next to ``index.html``. In this script, when the
page loads, draw the board:

.. code-block:: javascript

    import { createBoard, playMove } from "./connect4.js";

    window.addEventListener("DOMContentLoaded", () => {
      // Initialize the UI.
      const board = document.querySelector(".board");
      createBoard(board);
    });

Open a shell, navigate to the directory containing these files, and start an
HTTP server:

.. code-block:: console

    $ python -m http.server

Open http://localhost:8000/ in a web browser. The page displays an empty board
with seven columns and six rows. You will play moves in this board later.

Bootstrap the server
--------------------

Create an ``app.py`` file next to ``connect4.py`` with this content:

.. code-block:: python

    #!/usr/bin/env python

    import asyncio

    import websockets


    async def handler(websocket):
        while True:
            message = await websocket.recv()
            print(message)


    async def main():
        async with websockets.serve(handler, "", 8001):
            await asyncio.Future()  # run forever


    if __name__ == "__main__":
        asyncio.run(main())

The entry point of this program is ``asyncio.run(main())``. It creates an
asyncio event loop, runs the ``main()`` coroutine, and shuts down the loop.

The ``main()`` coroutine calls :func:`~server.serve` to start a websockets
server. :func:`~server.serve` takes three positional arguments:

* ``handler`` is a coroutine that manages a connection. When a client
  connects, websockets calls ``handler`` with the connection in argument.
  When ``handler`` terminates, websockets closes the connection.
* The second argument defines the network interfaces where the server can be
  reached. Here, the server listens on all interfaces, so that other devices
  on the same local network can connect.
* The third argument is the port on which the server listens.

Invoking :func:`~server.serve` as an asynchronous context manager, in an
``async with`` block, ensures that the server shuts down properly when
terminating the program.

For each connection, the ``handler()`` coroutine runs an infinite loop that
receives messages from the browser and prints them.

Open a shell, navigate to the directory containing ``app.py``, and start the
server:

.. code-block:: console

    $ python app.py

This doesn't display anything. Hopefully the WebSocket server is running.
Let's make sure that it works. You cannot test the WebSocket server with a
web browser like you tested the HTTP server. However, you can test it with
websockets' interactive client.

Open another shell and run this command:

.. code-block:: console

    $ python -m websockets ws://localhost:8001/

You get a prompt. Type a message and press "Enter". Switch to the shell where
the server is running and check that the server received the message. Good!

Exit the interactive client with Ctrl-C or Ctrl-D.

Now, if you look at the console where you started the server, you can see the
stack trace of an exception:

.. code-block:: pytb

    connection handler failed
    Traceback (most recent call last):
      ...
      File "app.py", line 22, in handler
        message = await websocket.recv()
      ...
    websockets.exceptions.ConnectionClosedOK: received 1000 (OK); then sent 1000 (OK)

Indeed, the server was waiting for the next message
with :meth:`~legacy.protocol.WebSocketCommonProtocol.recv` when the client
disconnected. When this happens, websockets raises
a :exc:`~exceptions.ConnectionClosedOK` exception to let you know that you
won't receive another message on this connection.

This exception creates noise in the server logs, making it more difficult to
spot real errors when you add functionality to the server. Catch it in the
``handler()`` coroutine:

.. code-block:: python

    async def handler(websocket):
        while True:
            try:
                message = await websocket.recv()
            except websockets.ConnectionClosedOK:
                break
            print(message)

Stop the server with Ctrl-C and start it again:

.. code-block:: console

    $ python app.py

.. admonition:: You must restart the WebSocket server when you make changes.
    :class: tip

    The WebSocket server loads the Python code in ``app.py`` then serves every
    WebSocket request with this version of the code. As a consequence,
    changes to ``app.py`` aren't visible until you restart the server.

    This is unlike the HTTP server that you started earlier with ``python -m
    http.server``. For every request, this HTTP server reads the target file
    and sends it. That's why changes are immediately visible.

    It is possible to :doc:`restart the WebSocket server automatically
    <../howto/autoreload>` but this isn't necessary for this tutorial.

Try connecting and disconnecting the interactive client again.
The :exc:`~exceptions.ConnectionClosedOK` exception doesn't appear anymore.

This pattern is so common that websockets provides a shortcut for iterating
over messages received on the connection until the client disconnects:

.. code-block:: python

    async def handler(websocket):
        async for message in websocket:
            print(message)

Restart the server and check with the interactive client that its behavior
didn't change.

At this point, you bootstrapped a web application and a WebSocket server.
Let's connect them.

Transmit from browser to server
-------------------------------

In JavaScript, you open a WebSocket connection as follows:

.. code-block:: javascript

    const websocket = new WebSocket("ws://localhost:8001/");

Before you exchange messages with the server, you need to decide their format.
There is no universal convention for this.

Let's use JSON objects with a ``type`` key identifying the type of the event
and the rest of the object containing properties of the event.

Here's an event describing a move in the middle slot of the board:

.. code-block:: javascript

    const event = {type: "play", column: 3};

Here's how to serialize this event to JSON and send it to the server:

.. code-block:: javascript

    websocket.send(JSON.stringify(event));

Now you have all the building blocks to send moves to the server.

Add this function to ``main.js``:

.. literalinclude:: ../../example/tutorial/step1/main.js
    :language: js
    :start-at: function sendMoves
    :end-before: window.addEventListener

``sendMoves()`` registers a listener for ``click`` events on the board. The
listener figures out which column was clicked, builds a  event of type
``"play"``, serializes it, and sends it to the server.

Modify the initialization to open the WebSocket connection and call the
``sendMoves()`` function:

.. code-block:: javascript

    window.addEventListener("DOMContentLoaded", () => {
      // Initialize the UI.
      const board = document.querySelector(".board");
      createBoard(board);
      // Open the WebSocket connection and register event handlers.
      const websocket = new WebSocket("ws://localhost:8001/");
      sendMoves(board, websocket);
    });

Check that the HTTP server and the WebSocket server are still running. If you
stopped them, here are the commands to start them again:

.. code-block:: console

    $ python -m http.server

.. code-block:: console

    $ python app.py

Refresh http://localhost:8000/ in your web browser. Click various columns in
the board. The server receives messages with the expected column number.

There isn't any feedback in the board because you haven't implemented that
yet. Let's do it.

Transmit from server to browser
-------------------------------

In JavaScript, you receive WebSocket messages by listening to ``message``
events. Here's how to receive a message from the server and deserialize it
from JSON:

.. code-block:: javascript

      websocket.addEventListener("message", ({ data }) => {
        const event = JSON.parse(data);
        // do something with event
      });

You're going to need three types of messages from the server to the browser:

.. code-block:: javascript

    {type: "play", player: "red", column: 3, row: 0}
    {type: "win", player: "red"}
    {type: "error", message: "This slot is full."}

The JavaScript code receiving these messages will dispatch events depending on
their type and take appropriate action. For example, it will react to an
event of type ``"play"`` by displaying the move on the board with
the :js:func:`~connect4.playMove` function.

Add this function to ``main.js``:

.. literalinclude:: ../../example/tutorial/step1/main.js
    :language: js
    :start-at: function showMessage
    :end-before: function sendMoves

.. admonition:: Why does ``showMessage`` use ``window.setTimeout``?
    :class: hint

    When :js:func:`playMove` modifies the state of the board, the browser
    renders changes asynchronously. Conversely, ``window.alert()`` runs
    synchronously and blocks rendering while the alert is visible.

    If you called ``window.alert()`` immediately after :js:func:`playMove`,
    the browser could display the alert before rendering the move. You could
    get a "Player red wins!" alert without seeing red's last move.

    We're using ``window.alert()`` for simplicity in this tutorial. A real
    application would display these messages in the user interface instead.
    It wouldn't be vulnerable to this problem.

Modify the initialization to call the ``receiveMoves()`` function:

.. literalinclude:: ../../example/tutorial/step1/main.js
    :language: js
    :start-at: window.addEventListener

At this point, the user interface should receive events properly. Let's test
it by modifying the server to send some events.

Sending an event from Python is quite similar to JavaScript:

.. code-block:: python

    event = {"type": "play", "player": "red", "column": 3, "row": 0}
    await websocket.send(json.dumps(event))

.. admonition:: Don't forget to serialize the event with :func:`json.dumps`.
    :class: tip

    Else, websockets raises ``TypeError: data is a dict-like object``.

Modify the ``handler()`` coroutine in ``app.py`` as follows:

.. code-block:: python

    import json

    from connect4 import PLAYER1, PLAYER2

    async def handler(websocket):
        for player, column, row in [
            (PLAYER1, 3, 0),
            (PLAYER2, 3, 1),
            (PLAYER1, 4, 0),
            (PLAYER2, 4, 1),
            (PLAYER1, 2, 0),
            (PLAYER2, 1, 0),
            (PLAYER1, 5, 0),
        ]:
            event = {
                "type": "play",
                "player": player,
                "column": column,
                "row": row,
            }
            await websocket.send(json.dumps(event))
            await asyncio.sleep(0.5)
        event = {
            "type": "win",
            "player": PLAYER1,
        }
        await websocket.send(json.dumps(event))

Restart the WebSocket server and  refresh http://localhost:8000/ in your web
browser. Seven moves appear at 0.5 second intervals. Then an alert announces
the winner.

Good! Now you know how to communicate both ways.

Once you plug the game engine to process moves, you will have a fully
functional game.

Add the game logic
------------------

In the ``handler()`` coroutine, you're going to initialize a game:

.. code-block:: python

    from connect4 import Connect4

    async def handler(websocket):
        # Initialize a Connect Four game.
        game = Connect4()

        ...

Then, you're going to iterate over incoming messages and take these steps:

* parse an event of type ``"play"``, the only type of event that the user
  interface sends;
* play the move in the board with the :meth:`~connect4.Connect4.play` method,
  alternating between the two players;
* if :meth:`~connect4.Connect4.play` raises :exc:`RuntimeError` because the
  move is illegal, send an event of type ``"error"``;
* else, send an event of type ``"play"`` to tell the user interface where the
  checker lands;
* if the move won the game, send an event of type ``"win"``.

Try to implement this by yourself!

Keep in mind that you must restart the WebSocket server and reload the page in
the browser when you make changes.

When it works, you can play the game from a single browser, with players
taking alternate turns.

.. admonition:: Enable debug logs to see all messages sent and received.
    :class: tip

    Here's how to enable debug logs:

    .. code-block:: python

        import logging

        logging.basicConfig(format="%(message)s", level=logging.DEBUG)

If you're stuck, a solution is available at the bottom of this document.

Summary
-------

In this first part of the tutorial, you learned how to:

* build and run a WebSocket server in Python with :func:`~server.serve`;
* receive a message in a connection handler
  with :meth:`~server.WebSocketServerProtocol.recv`;
* send a message in a connection handler
  with :meth:`~server.WebSocketServerProtocol.send`;
* iterate over incoming messages with ``async for
  message in websocket: ...``;
* open a WebSocket connection in JavaScript with the ``WebSocket`` API;
* send messages in a browser with ``WebSocket.send()``;
* receive messages in a browser by listening to ``message`` events;
* design a set of events to be exchanged between the browser and the server.

You can now play a Connect Four game in a browser, communicating over a
WebSocket connection with a server where the game logic resides!

However, the two players share a browser, so the constraint of being in the
same room still applies.

Move on to the :doc:`second part <tutorial2>` of the tutorial to break this
constraint and play from separate browsers.

Solution
--------

.. literalinclude:: ../../example/tutorial/step1/app.py
    :caption: app.py
    :language: python
    :linenos:

.. literalinclude:: ../../example/tutorial/step1/index.html
    :caption: index.html
    :language: html
    :linenos:

.. literalinclude:: ../../example/tutorial/step1/main.js
    :caption: main.js
    :language: js
    :linenos:

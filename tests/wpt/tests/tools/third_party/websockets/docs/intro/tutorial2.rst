Part 2 - Route & broadcast
==========================

.. currentmodule:: websockets

.. admonition:: This is the second part of the tutorial.

    * In the :doc:`first part <tutorial1>`, you created a server and
      connected one browser; you could play if you shared the same browser.
    * In this :doc:`second part <tutorial2>`, you will connect a second
      browser; you can play from different browsers on a local network.
    * In the :doc:`third part <tutorial3>`, you will deploy the game to the
      web; you can play from any browser connected to the Internet.

In the first part of the tutorial, you opened a WebSocket connection from a
browser to a server and exchanged events to play moves. The state of the game
was stored in an instance of the :class:`~connect4.Connect4` class,
referenced as a local variable in the connection handler coroutine.

Now you want to open two WebSocket connections from two separate browsers, one
for each player, to the same server in order to play the same game. This
requires moving the state of the game to a place where both connections can
access it.

Share game state
----------------

As long as you're running a single server process, you can share state by
storing it in a global variable.

.. admonition:: What if you need to scale to multiple server processes?
    :class: hint

    In that case, you must design a way for the process that handles a given
    connection to be aware of relevant events for that client. This is often
    achieved with a publish / subscribe mechanism.

How can you make two connection handlers agree on which game they're playing?
When the first player starts a game, you give it an identifier. Then, you
communicate the identifier to the second player. When the second player joins
the game, you look it up with the identifier.

In addition to the game itself, you need to keep track of the WebSocket
connections of the two players. Since both players receive the same events,
you don't need to treat the two connections differently; you can store both
in the same set.

Let's sketch this in code.

A module-level :class:`dict` enables lookups by identifier:

.. code-block:: python

    JOIN = {}

When the first player starts the game, initialize and store it:

.. code-block:: python

    import secrets

    async def handler(websocket):
        ...

        # Initialize a Connect Four game, the set of WebSocket connections
        # receiving moves from this game, and secret access token.
        game = Connect4()
        connected = {websocket}

        join_key = secrets.token_urlsafe(12)
        JOIN[join_key] = game, connected

        try:

            ...

        finally:
            del JOIN[join_key]

When the second player joins the game, look it up:

.. code-block:: python

    async def handler(websocket):
        ...

        join_key = ...  # TODO

        # Find the Connect Four game.
        game, connected = JOIN[join_key]

        # Register to receive moves from this game.
        connected.add(websocket)
        try:

            ...

        finally:
            connected.remove(websocket)

Notice how we're carefully cleaning up global state with ``try: ...
finally: ...`` blocks. Else, we could leave references to games or
connections in global state, which would cause a memory leak.

In both connection handlers, you have a ``game`` pointing to the same
:class:`~connect4.Connect4` instance, so you can interact with the game,
and a ``connected`` set of connections, so you can send game events to
both players as follows:

.. code-block:: python

    async def handler(websocket):

        ...

        for connection in connected:
            await connection.send(json.dumps(event))

        ...

Perhaps you spotted a major piece missing from the puzzle. How does the second
player obtain ``join_key``? Let's design new events to carry this information.

To start a game, the first player sends an ``"init"`` event:

.. code-block:: javascript

    {type: "init"}

The connection handler for the first player creates a game as shown above and
responds with:

.. code-block:: javascript

    {type: "init", join: "<join_key>"}

With this information, the user interface of the first player can create a
link to ``http://localhost:8000/?join=<join_key>``. For the sake of simplicity,
we will assume that the first player shares this link with the second player
outside of the application, for example via an instant messaging service.

To join the game, the second player sends a different ``"init"`` event:

.. code-block:: javascript

    {type: "init", join: "<join_key>"}

The connection handler for the second player can look up the game with the
join key as shown above. There is no need to respond.

Let's dive into the details of implementing this design.

Start a game
------------

We'll start with the initialization sequence for the first player.

In ``main.js``, define a function to send an initialization event when the
WebSocket connection is established, which triggers an ``open`` event:

.. code-block:: javascript

    function initGame(websocket) {
      websocket.addEventListener("open", () => {
        // Send an "init" event for the first player.
        const event = { type: "init" };
        websocket.send(JSON.stringify(event));
      });
    }

Update the initialization sequence to call ``initGame()``:

.. literalinclude:: ../../example/tutorial/step2/main.js
    :language: js
    :start-at: window.addEventListener

In ``app.py``, define a new ``handler`` coroutine — keep a copy of the
previous one to reuse it later:

.. code-block:: python

    import secrets


    JOIN = {}


    async def start(websocket):
        # Initialize a Connect Four game, the set of WebSocket connections
        # receiving moves from this game, and secret access token.
        game = Connect4()
        connected = {websocket}

        join_key = secrets.token_urlsafe(12)
        JOIN[join_key] = game, connected

        try:
            # Send the secret access token to the browser of the first player,
            # where it'll be used for building a "join" link.
            event = {
                "type": "init",
                "join": join_key,
            }
            await websocket.send(json.dumps(event))

            # Temporary - for testing.
            print("first player started game", id(game))
            async for message in websocket:
                print("first player sent", message)

        finally:
            del JOIN[join_key]


    async def handler(websocket):
        # Receive and parse the "init" event from the UI.
        message = await websocket.recv()
        event = json.loads(message)
        assert event["type"] == "init"

        # First player starts a new game.
        await start(websocket)

In ``index.html``, add an ``<a>`` element to display the link to share with
the other player.

.. code-block:: html

      <body>
        <div class="actions">
          <a class="action join" href="">Join</a>
        </div>
        <!-- ... -->
      </body>

In ``main.js``, modify ``receiveMoves()`` to handle the ``"init"`` message and
set the target of that link:

.. code-block:: javascript

        switch (event.type) {
          case "init":
            // Create link for inviting the second player.
            document.querySelector(".join").href = "?join=" + event.join;
            break;
          // ...
        }

Restart the WebSocket server and reload http://localhost:8000/ in the browser.
There's a link labeled JOIN below the board with a target that looks like
http://localhost:8000/?join=95ftAaU5DJVP1zvb.

The server logs say ``first player started game ...``. If you click the board,
you see ``"play"`` events. There is no feedback in the UI, though, because
you haven't restored the game logic yet.

Before we get there, let's handle links with a ``join`` query parameter.

Join a game
-----------

We'll now update the initialization sequence to account for the second
player.

In ``main.js``, update ``initGame()`` to send the join key in the ``"init"``
message when it's in the URL:

.. code-block:: javascript

    function initGame(websocket) {
      websocket.addEventListener("open", () => {
        // Send an "init" event according to who is connecting.
        const params = new URLSearchParams(window.location.search);
        let event = { type: "init" };
        if (params.has("join")) {
          // Second player joins an existing game.
          event.join = params.get("join");
        } else {
          // First player starts a new game.
        }
        websocket.send(JSON.stringify(event));
      });
    }

In ``app.py``, update the ``handler`` coroutine to look for the join key in
the ``"init"`` message, then load that game:

.. code-block:: python

    async def error(websocket, message):
        event = {
            "type": "error",
            "message": message,
        }
        await websocket.send(json.dumps(event))


    async def join(websocket, join_key):
        # Find the Connect Four game.
        try:
            game, connected = JOIN[join_key]
        except KeyError:
            await error(websocket, "Game not found.")
            return

        # Register to receive moves from this game.
        connected.add(websocket)
        try:

            # Temporary - for testing.
            print("second player joined game", id(game))
            async for message in websocket:
                print("second player sent", message)

        finally:
            connected.remove(websocket)


    async def handler(websocket):
        # Receive and parse the "init" event from the UI.
        message = await websocket.recv()
        event = json.loads(message)
        assert event["type"] == "init"

        if "join" in event:
            # Second player joins an existing game.
            await join(websocket, event["join"])
        else:
            # First player starts a new game.
            await start(websocket)

Restart the WebSocket server and reload http://localhost:8000/ in the browser.

Copy the link labeled JOIN and open it in another browser. You may also open
it in another tab or another window of the same browser; however, that makes
it a bit tricky to remember which one is the first or second player.

.. admonition:: You must start a new game when you restart the server.
    :class: tip

    Since games are stored in the memory of the Python process, they're lost
    when you stop the server.

    Whenever you make changes to ``app.py``, you must restart the server,
    create a new game in a browser, and join it in another browser.

The server logs say ``first player started game ...`` and ``second player
joined game ...``. The numbers match, proving that the ``game`` local
variable in both connection handlers points to same object in the memory of
the Python process.

Click the board in either browser. The server receives ``"play"`` events from
the corresponding player.

In the initialization sequence, you're routing connections to ``start()`` or
``join()`` depending on the first message received by the server. This is a
common pattern in servers that handle different clients.

.. admonition:: Why not use different URIs for ``start()`` and ``join()``?
    :class: hint

    Instead of sending an initialization event, you could encode the join key
    in the WebSocket URI e.g. ``ws://localhost:8001/join/<join_key>``. The
    WebSocket server would parse ``websocket.path`` and route the connection,
    similar to how HTTP servers route requests.

    When you need to send sensitive data like authentication credentials to
    the server, sending it an event is considered more secure than encoding
    it in the URI because URIs end up in logs.

    For the purposes of this tutorial, both approaches are equivalent because
    the join key comes from an HTTP URL. There isn't much at risk anyway!

Now you can restore the logic for playing moves and you'll have a fully
functional two-player game.

Add the game logic
------------------

Once the initialization is done, the game is symmetrical, so you can write a
single coroutine to process the moves of both players:

.. code-block:: python

    async def play(websocket, game, player, connected):
        ...

With such a coroutine, you can replace the temporary code for testing in
``start()`` by:

.. code-block:: python

            await play(websocket, game, PLAYER1, connected)

and in ``join()`` by:

.. code-block:: python

            await play(websocket, game, PLAYER2, connected)

The ``play()`` coroutine will reuse much of the code you wrote in the first
part of the tutorial.

Try to implement this by yourself!

Keep in mind that you must restart the WebSocket server, reload the page to
start a new game with the first player, copy the JOIN link, and join the game
with the second player when you make changes.

When ``play()`` works, you can play the game from two separate browsers,
possibly running on separate computers on the same local network.

A complete solution is available at the bottom of this document.

Watch a game
------------

Let's add one more feature: allow spectators to watch the game.

The process for inviting a spectator can be the same as for inviting the
second player. You will have to duplicate all the initialization logic:

- declare a ``WATCH`` global variable similar to ``JOIN``;
- generate a watch key when creating a game; it must be different from the
  join key, or else a spectator could hijack a game by tweaking the URL;
- include the watch key in the ``"init"`` event sent to the first player;
- generate a WATCH link in the UI with a ``watch`` query parameter;
- update the ``initGame()`` function to handle such links;
- update the ``handler()`` coroutine to invoke a ``watch()`` coroutine for
  spectators;
- prevent ``sendMoves()`` from sending ``"play"`` events for spectators.

Once the initialization sequence is done, watching a game is as simple as
registering the WebSocket connection in the ``connected`` set in order to
receive game events and doing nothing until the spectator disconnects. You
can wait for a connection to terminate with
:meth:`~legacy.protocol.WebSocketCommonProtocol.wait_closed`:

.. code-block:: python

    async def watch(websocket, watch_key):

        ...

        connected.add(websocket)
        try:
            await websocket.wait_closed()
        finally:
            connected.remove(websocket)

The connection can terminate because the ``receiveMoves()`` function closed it
explicitly after receiving a ``"win"`` event, because the spectator closed
their browser, or because the network failed.

Again, try to implement this by yourself.

When ``watch()`` works, you can invite spectators to watch the game from other
browsers, as long as they're on the same local network.

As a further improvement, you may support adding spectators while a game is
already in progress. This requires replaying moves that were played before
the spectator was added to the ``connected`` set. Past moves are available in
the :attr:`~connect4.Connect4.moves` attribute of the game.

This feature is included in the solution proposed below.

Broadcast
---------

When you need to send a message to the two players and to all spectators,
you're using this pattern:

.. code-block:: python

    async def handler(websocket):

        ...

        for connection in connected:
            await connection.send(json.dumps(event))

        ...

Since this is a very common pattern in WebSocket servers, websockets provides
the :func:`broadcast` helper for this purpose:

.. code-block:: python

    async def handler(websocket):

        ...

        websockets.broadcast(connected, json.dumps(event))

        ...

Calling :func:`broadcast` once is more efficient than
calling :meth:`~legacy.protocol.WebSocketCommonProtocol.send` in a loop.

However, there's a subtle difference in behavior. Did you notice that there's
no ``await`` in the second version? Indeed, :func:`broadcast` is a function,
not a coroutine like :meth:`~legacy.protocol.WebSocketCommonProtocol.send`
or :meth:`~legacy.protocol.WebSocketCommonProtocol.recv`.

It's quite obvious why :meth:`~legacy.protocol.WebSocketCommonProtocol.recv`
is a coroutine. When you want to receive the next message, you have to wait
until the client sends it and the network transmits it.

It's less obvious why :meth:`~legacy.protocol.WebSocketCommonProtocol.send` is
a coroutine. If you send many messages or large messages, you could write
data faster than the network can transmit it or the client can read it. Then,
outgoing data will pile up in buffers, which will consume memory and may
crash your application.

To avoid this problem, :meth:`~legacy.protocol.WebSocketCommonProtocol.send`
waits until the write buffer drains. By slowing down the application as
necessary, this ensures that the server doesn't send data too quickly. This
is called backpressure and it's useful for building robust systems.

That said, when you're sending the same messages to many clients in a loop,
applying backpressure in this way can become counterproductive. When you're
broadcasting, you don't want to slow down everyone to the pace of the slowest
clients; you want to drop clients that cannot keep up with the data stream.
That's why :func:`broadcast` doesn't wait until write buffers drain.

For our Connect Four game, there's no difference in practice: the total amount
of data sent on a connection for a game of Connect Four is less than 64 KB,
so the write buffer never fills up and backpressure never kicks in anyway.

Summary
-------

In this second part of the tutorial, you learned how to:

* configure a connection by exchanging initialization messages;
* keep track of connections within a single server process;
* wait until a client disconnects in a connection handler;
* broadcast a message to many connections efficiently.

You can now play a Connect Four game from separate browser, communicating over
WebSocket connections with a server that synchronizes the game logic!

However, the two players have to be on the same local network as the server,
so the constraint of being in the same place still mostly applies.

Head over to the :doc:`third part <tutorial3>` of the tutorial to deploy the
game to the web and remove this constraint.

Solution
--------

.. literalinclude:: ../../example/tutorial/step2/app.py
    :caption: app.py
    :language: python
    :linenos:

.. literalinclude:: ../../example/tutorial/step2/index.html
    :caption: index.html
    :language: html
    :linenos:

.. literalinclude:: ../../example/tutorial/step2/main.js
    :caption: main.js
    :language: js
    :linenos:

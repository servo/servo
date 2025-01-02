Patterns
========

.. currentmodule:: websockets

Here are typical patterns for processing messages in a WebSocket server or
client. You will certainly implement some of them in your application.

This page gives examples of connection handlers for a server. However, they're
also applicable to a client, simply by assuming that ``websocket`` is a
connection created with :func:`~client.connect`.

WebSocket connections are long-lived. You will usually write a loop to process
several messages during the lifetime of a connection.

Consumer
--------

To receive messages from the WebSocket connection::

    async def consumer_handler(websocket):
        async for message in websocket:
            await consumer(message)

In this example, ``consumer()`` is a coroutine implementing your business
logic for processing a message received on the WebSocket connection. Each
message may be :class:`str` or :class:`bytes`.

Iteration terminates when the client disconnects.

Producer
--------

To send messages to the WebSocket connection::

    async def producer_handler(websocket):
        while True:
            message = await producer()
            await websocket.send(message)

In this example, ``producer()`` is a coroutine implementing your business
logic for generating the next message to send on the WebSocket connection.
Each message must be :class:`str` or :class:`bytes`.

Iteration terminates when the client disconnects
because :meth:`~server.WebSocketServerProtocol.send` raises a
:exc:`~exceptions.ConnectionClosed` exception,
which breaks out of the ``while  True`` loop.

Consumer and producer
---------------------

You can receive and send messages on the same WebSocket connection by
combining the consumer and producer patterns. This requires running two tasks
in parallel::

    async def handler(websocket):
        await asyncio.gather(
            consumer_handler(websocket),
            producer_handler(websocket),
        )

If a task terminates, :func:`~asyncio.gather` doesn't cancel the other task.
This can result in a situation where the producer keeps running after the
consumer finished, which may leak resources.

Here's a way to exit and close the WebSocket connection as soon as a task
terminates, after canceling the other task::

    async def handler(websocket):
        consumer_task = asyncio.create_task(consumer_handler(websocket))
        producer_task = asyncio.create_task(producer_handler(websocket))
        done, pending = await asyncio.wait(
            [consumer_task, producer_task],
            return_when=asyncio.FIRST_COMPLETED,
        )
        for task in pending:
            task.cancel()

Registration
------------

To keep track of currently connected clients, you can register them when they
connect and unregister them when they disconnect::

    connected = set()

    async def handler(websocket):
        # Register.
        connected.add(websocket)
        try:
            # Broadcast a message to all connected clients.
            websockets.broadcast(connected, "Hello!")
            await asyncio.sleep(10)
        finally:
            # Unregister.
            connected.remove(websocket)

This example maintains the set of connected clients in memory. This works as
long as you run a single process. It doesn't scale to multiple processes.

Publish–subscribe
-----------------

If you plan to run multiple processes and you want to communicate updates
between processes, then you must deploy a messaging system. You may find
publish-subscribe functionality useful.

A complete implementation of this idea with Redis is described in
the :doc:`Django integration guide <../howto/django>`.

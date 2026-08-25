Broadcasting messages
=====================

.. currentmodule:: websockets


.. admonition:: If you just want to send a message to all connected clients,
    use :func:`broadcast`.
    :class: tip

    If you want to learn about its design in depth, continue reading this
    document.

WebSocket servers often send the same message to all connected clients or to a
subset of clients for which the message is relevant.

Let's explore options for broadcasting a message, explain the design
of :func:`broadcast`, and discuss alternatives.

For each option, we'll provide a connection handler called ``handler()`` and a
function or coroutine called ``broadcast()`` that sends a message to all
connected clients.

Integrating them is left as an exercise for the reader. You could start with::

    import asyncio
    import websockets

    async def handler(websocket):
        ...

    async def broadcast(message):
        ...

    async def broadcast_messages():
        while True:
            await asyncio.sleep(1)
            message = ...  # your application logic goes here
            await broadcast(message)

    async def main():
        async with websockets.serve(handler, "localhost", 8765):
            await broadcast_messages()  # runs forever

    if __name__ == "__main__":
        asyncio.run(main())

``broadcast_messages()`` must yield control to the event loop between each
message, or else it will never let the server run. That's why it includes
``await asyncio.sleep(1)``.

A complete example is available in the `experiments/broadcast`_ directory.

.. _experiments/broadcast: https://github.com/python-websockets/websockets/tree/main/experiments/broadcast

The naive way
-------------

The most obvious way to send a message to all connected clients consists in
keeping track of them and sending the message to each of them.

Here's a connection handler that registers clients in a global variable::

    CLIENTS = set()

    async def handler(websocket):
        CLIENTS.add(websocket)
        try:
            await websocket.wait_closed()
        finally:
            CLIENTS.remove(websocket)

This implementation assumes that the client will never send any messages. If
you'd rather not make this assumption, you can change::

            await websocket.wait_closed()

to::

            async for _ in websocket:
                pass

Here's a coroutine that broadcasts a message to all clients::

    async def broadcast(message):
        for websocket in CLIENTS.copy():
            try:
                await websocket.send(message)
            except websockets.ConnectionClosed:
                pass

There are two tricks in this version of ``broadcast()``.

First, it makes a copy of ``CLIENTS`` before iterating it. Else, if a client
connects or disconnects while ``broadcast()`` is running, the loop would fail
with::

    RuntimeError: Set changed size during iteration

Second, it ignores :exc:`~exceptions.ConnectionClosed` exceptions because a
client could disconnect between the moment ``broadcast()`` makes a copy of
``CLIENTS`` and the moment it sends a message to this client. This is fine: a
client that disconnected doesn't belongs to "all connected clients" anymore.

The naive way can be very fast. Indeed, if all connections have enough free
space in their write buffers, ``await websocket.send(message)`` writes the
message and returns immediately, as it doesn't need to wait for the buffer to
drain. In this case, ``broadcast()`` doesn't yield control to the event loop,
which minimizes overhead.

The naive way can also fail badly. If the write buffer of a connection reaches
``write_limit``, ``broadcast()`` waits for the buffer to drain before sending
the message to other clients. This can cause a massive drop in performance.

As a consequence, this pattern works only when write buffers never fill up,
which is usually outside of the control of the server.

If you know for sure that you will never write more than ``write_limit`` bytes
within ``ping_interval + ping_timeout``, then websockets will terminate slow
connections before the write buffer has time to fill up.

Don't set extreme ``write_limit``, ``ping_interval``, and ``ping_timeout``
values to ensure that this condition holds. Set reasonable values and use the
built-in :func:`broadcast` function instead.

The concurrent way
------------------

The naive way didn't work well because it serialized writes, while the whole
point of asynchronous I/O is to perform I/O concurrently.

Let's modify ``broadcast()`` to send messages concurrently::

    async def send(websocket, message):
        try:
            await websocket.send(message)
        except websockets.ConnectionClosed:
            pass

    def broadcast(message):
        for websocket in CLIENTS:
            asyncio.create_task(send(websocket, message))

We move the error handling logic in a new coroutine and we schedule
a :class:`~asyncio.Task` to run it instead of executing it immediately.

Since ``broadcast()`` no longer awaits coroutines, we can make it a function
rather than a coroutine and do away with the copy of ``CLIENTS``.

This version of ``broadcast()`` makes clients independent from one another: a
slow client won't block others. As a side effect, it makes messages
independent from one another.

If you broadcast several messages, there is no strong guarantee that they will
be sent in the expected order. Fortunately, the event loop runs tasks in the
order in which they are created, so the order is correct in practice.

Technically, this is an implementation detail of the event loop. However, it
seems unlikely for an event loop to run tasks in an order other than FIFO.

If you wanted to enforce the order without relying this implementation detail,
you could be tempted to wait until all clients have received the message::

    async def broadcast(message):
        if CLIENTS:  # asyncio.wait doesn't accept an empty list
            await asyncio.wait([
                asyncio.create_task(send(websocket, message))
                for websocket in CLIENTS
            ])

However, this doesn't really work in practice. Quite often, it will block
until the slowest client times out.

Backpressure meets broadcast
----------------------------

At this point, it becomes apparent that backpressure, usually a good practice,
doesn't work well when broadcasting a message to thousands of clients.

When you're sending messages to a single client, you don't want to send them
faster than the network can transfer them and the client accept them. This is
why :meth:`~server.WebSocketServerProtocol.send` checks if the write buffer
is full and, if it is, waits until it drain, giving the network and the
client time to catch up. This provides backpressure.

Without backpressure, you could pile up data in the write buffer until the
server process runs out of memory and the operating system kills it.

The :meth:`~server.WebSocketServerProtocol.send` API is designed to enforce
backpressure by default. This helps users of websockets write robust programs
even if they never heard about backpressure.

For comparison, :class:`asyncio.StreamWriter` requires users to understand
backpressure and to await :meth:`~asyncio.StreamWriter.drain` explicitly
after each :meth:`~asyncio.StreamWriter.write`.

When broadcasting messages, backpressure consists in slowing down all clients
in an attempt to let the slowest client catch up. With thousands of clients,
the slowest one is probably timing out and isn't going to receive the message
anyway. So it doesn't make sense to synchronize with the slowest client.

How do we avoid running out of memory when slow clients can't keep up with the
broadcast rate, then? The most straightforward option is to disconnect them.

If a client gets too far behind, eventually it reaches the limit defined by
``ping_timeout`` and websockets terminates the connection. You can read the
discussion of :doc:`keepalive and timeouts <./timeouts>` for details.

How :func:`broadcast` works
---------------------------

The built-in :func:`broadcast` function is similar to the naive way. The main
difference is that it doesn't apply backpressure.

This provides the best performance by avoiding the overhead of scheduling and
running one task per client.

Also, when sending text messages, encoding to UTF-8 happens only once rather
than once per client, providing a small performance gain.

Per-client queues
-----------------

At this point, we deal with slow clients rather brutally: we disconnect then.

Can we do better? For example, we could decide to skip or to batch messages,
depending on how far behind a client is.

To implement this logic, we can create a queue of messages for each client and
run a task that gets messages from the queue and sends them to the client::

    import asyncio

    CLIENTS = set()

    async def relay(queue, websocket):
        while True:
            # Implement custom logic based on queue.qsize() and
            # websocket.transport.get_write_buffer_size() here.
            message = await queue.get()
            await websocket.send(message)

    async def handler(websocket):
        queue = asyncio.Queue()
        relay_task = asyncio.create_task(relay(queue, websocket))
        CLIENTS.add(queue)
        try:
            await websocket.wait_closed()
        finally:
            CLIENTS.remove(queue)
            relay_task.cancel()

Then we can broadcast a message by pushing it to all queues::

    def broadcast(message):
        for queue in CLIENTS:
            queue.put_nowait(message)

The queues provide an additional buffer between the ``broadcast()`` function
and clients. This makes it easier to support slow clients without excessive
memory usage because queued messages aren't duplicated to  write buffers
until ``relay()`` processes them.

Publish–subscribe
-----------------

Can we avoid centralizing the list of connected clients in a global variable?

If each client subscribes to a stream a messages, then broadcasting becomes as
simple as publishing a message to the stream.

Here's a message stream that supports multiple consumers::

    class PubSub:
        def __init__(self):
            self.waiter = asyncio.Future()

        def publish(self, value):
            waiter, self.waiter = self.waiter, asyncio.Future()
            waiter.set_result((value, self.waiter))

        async def subscribe(self):
            waiter = self.waiter
            while True:
                value, waiter = await waiter
                yield value

        __aiter__ = subscribe

    PUBSUB = PubSub()

The stream is implemented as a linked list of futures. It isn't necessary to
synchronize consumers. They can read the stream at their own pace,
independently from one another. Once all consumers read a message, there are
no references left, therefore the garbage collector deletes it.

The connection handler subscribes to the stream and sends messages::

    async def handler(websocket):
        async for message in PUBSUB:
            await websocket.send(message)

The broadcast function publishes to the stream::

    def broadcast(message):
        PUBSUB.publish(message)

Like per-client queues, this version supports slow clients with limited memory
usage. Unlike per-client queues, it makes it difficult to tell how far behind
a client is. The ``PubSub`` class could be extended or refactored to provide
this information.

The ``for`` loop is gone from this version of the ``broadcast()`` function.
However, there's still a ``for`` loop iterating on all clients hidden deep
inside :mod:`asyncio`. When ``publish()`` sets the result of the ``waiter``
future, :mod:`asyncio` loops on callbacks registered with this future and
schedules them. This is how connection handlers receive the next value from
the asynchronous iterator returned by ``subscribe()``.

Performance considerations
--------------------------

The built-in :func:`broadcast` function sends all messages without yielding
control to the event loop. So does the naive way when the network and clients
are fast and reliable.

For each client, a WebSocket frame is prepared and sent to the network. This
is the minimum amount of work required to broadcast a message.

It would be tempting to prepare a frame and reuse it for all connections.
However, this isn't possible in general for two reasons:

* Clients can negotiate different extensions. You would have to enforce the
  same extensions with the same parameters. For example, you would have to
  select some compression settings and reject clients that cannot support
  these settings.

* Extensions can be stateful, producing different encodings of the same
  message depending on previous messages. For example, you would have to
  disable context takeover to make compression stateless, resulting in poor
  compression rates.

All other patterns discussed above yield control to the event loop once per
client because messages are sent by different tasks. This makes them slower
than the built-in :func:`broadcast` function.

There is no major difference between the performance of per-client queues and
publish–subscribe.

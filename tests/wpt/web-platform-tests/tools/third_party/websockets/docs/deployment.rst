Deployment
==========

.. currentmodule:: websockets

Application server
------------------

The author of ``websockets`` isn't aware of best practices for deploying
network services based on :mod:`asyncio`, let alone application servers.

You can run a script similar to the :ref:`server example <server-example>`,
inside a supervisor if you deem that useful.

You can also add a wrapper to daemonize the process. Third-party libraries
provide solutions for that.

If you can share knowledge on this topic, please file an issue_. Thanks!

.. _issue: https://github.com/aaugustin/websockets/issues/new

Graceful shutdown
-----------------

You may want to close connections gracefully when shutting down the server,
perhaps after executing some cleanup logic. There are two ways to achieve this
with the object returned by :func:`~server.serve`:

- using it as a asynchronous context manager, or
- calling its ``close()`` method, then waiting for its ``wait_closed()``
  method to complete.

On Unix systems, shutdown is usually triggered by sending a signal.

Here's a full example for handling SIGTERM on Unix:

.. literalinclude:: ../example/shutdown.py
    :emphasize-lines: 13,17-19

This example is easily adapted to handle other signals. If you override the
default handler for SIGINT, which raises :exc:`KeyboardInterrupt`, be aware
that you won't be able to interrupt a program with Ctrl-C anymore when it's
stuck in a loop.

It's more difficult to achieve the same effect on Windows. Some third-party
projects try to help with this problem.

If your server doesn't run in the main thread, look at
:func:`~asyncio.AbstractEventLoop.call_soon_threadsafe`.

Memory usage
------------

.. _memory-usage:

In most cases, memory usage of a WebSocket server is proportional to the
number of open connections. When a server handles thousands of connections,
memory usage can become a bottleneck.

Memory usage of a single connection is the sum of:

1. the baseline amount of memory ``websockets`` requires for each connection,
2. the amount of data held in buffers before the application processes it,
3. any additional memory allocated by the application itself.

Baseline
........

Compression settings are the main factor affecting the baseline amount of
memory used by each connection.

By default ``websockets`` maximizes compression rate at the expense of memory
usage. If memory usage is an issue, lowering compression settings can help:

- Context Takeover is necessary to get good performance for almost all
  applications. It should remain enabled.
- Window Bits is a trade-off between memory usage and compression rate.
  It defaults to 15 and can be lowered. The default value isn't optimal
  for small, repetitive messages which are typical of WebSocket servers.
- Memory Level is a trade-off between memory usage and compression speed.
  It defaults to 8 and can be lowered. A lower memory level can actually
  increase speed thanks to memory locality, even if the CPU does more work!

See this :ref:`example <per-message-deflate-configuration-example>` for how to
configure compression settings.

Here's how various compression settings affect memory usage of a single
connection on a 64-bit system, as well a benchmark_ of compressed size and
compression time for a corpus of small JSON documents.

+-------------+-------------+--------------+--------------+------------------+------------------+
| Compression | Window Bits | Memory Level | Memory usage | Size vs. default | Time vs. default |
+=============+=============+==============+==============+==================+==================+
| *default*   | 15          | 8            | 325 KiB      | +0%              | +0%              +
+-------------+-------------+--------------+--------------+------------------+------------------+
|             | 14          | 7            | 181 KiB      | +1.5%            | -5.3%            |
+-------------+-------------+--------------+--------------+------------------+------------------+
|             | 13          | 6            | 110 KiB      | +2.8%            | -7.5%            |
+-------------+-------------+--------------+--------------+------------------+------------------+
|             | 12          | 5            | 73 KiB       | +4.4%            | -18.9%           |
+-------------+-------------+--------------+--------------+------------------+------------------+
|             | 11          | 4            | 55 KiB       | +8.5%            | -18.8%           |
+-------------+-------------+--------------+--------------+------------------+------------------+
| *disabled*  | N/A         | N/A          | 22 KiB       | N/A              | N/A              |
+-------------+-------------+--------------+--------------+------------------+------------------+

*Don't assume this example is representative! Compressed size and compression
time depend heavily on the kind of messages exchanged by the application!*

You can run the same benchmark for your application by creating a list of
typical messages and passing it to the ``_benchmark`` function_.

.. _benchmark: https://gist.github.com/aaugustin/fbea09ce8b5b30c4e56458eb081fe599
.. _function: https://gist.github.com/aaugustin/fbea09ce8b5b30c4e56458eb081fe599#file-compression-py-L48-L144

This `blog post by Ilya Grigorik`_ provides more details about how compression
settings affect memory usage and how to optimize them.

.. _blog post by Ilya Grigorik: https://www.igvita.com/2013/11/27/configuring-and-optimizing-websocket-compression/

This `experiment by Peter Thorson`_ suggests Window Bits = 11, Memory Level =
4 as a sweet spot for optimizing memory usage.

.. _experiment by Peter Thorson: https://www.ietf.org/mail-archive/web/hybi/current/msg10222.html

Buffers
.......

Under normal circumstances, buffers are almost always empty.

Under high load, if a server receives more messages than it can process,
bufferbloat can result in excessive memory use.

By default ``websockets`` has generous limits. It is strongly recommended to
adapt them to your application. When you call :func:`~server.serve`:

- Set ``max_size`` (default: 1 MiB, UTF-8 encoded) to the maximum size of
  messages your application generates.
- Set ``max_queue`` (default: 32) to the maximum number of messages your
  application expects to receive faster than it can process them. The queue
  provides burst tolerance without slowing down the TCP connection.

Furthermore, you can lower ``read_limit`` and ``write_limit`` (default:
64 KiB) to reduce the size of buffers for incoming and outgoing data.

The design document provides :ref:`more details about buffers<buffers>`.

Port sharing
------------

The WebSocket protocol is an extension of HTTP/1.1. It can be tempting to
serve both HTTP and WebSocket on the same port.

The author of ``websockets`` doesn't think that's a good idea, due to the
widely different operational characteristics of HTTP and WebSocket.

``websockets`` provide minimal support for responding to HTTP requests with
the :meth:`~server.WebSocketServerProtocol.process_request` hook. Typical
use cases include health checks. Here's an example:

.. literalinclude:: ../example/health_check_server.py
    :emphasize-lines: 9-11,17-19

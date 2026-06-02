Memory usage
============

.. currentmodule:: websockets

In most cases, memory usage of a WebSocket server is proportional to the
number of open connections. When a server handles thousands of connections,
memory usage can become a bottleneck.

Memory usage of a single connection is the sum of:

1. the baseline amount of memory websockets requires for each connection,
2. the amount of data held in buffers before the application processes it,
3. any additional memory allocated by the application itself.

Baseline
--------

Compression settings are the main factor affecting the baseline amount of
memory used by each connection.

With websockets' defaults, on the server side, a single connections uses
70 KiB of memory.

Refer to the :doc:`topic guide on compression <../topics/compression>` to
learn more about tuning compression settings.

Buffers
-------

Under normal circumstances, buffers are almost always empty.

Under high load, if a server receives more messages than it can process,
bufferbloat can result in excessive memory usage.

By default websockets has generous limits. It is strongly recommended to adapt
them to your application. When you call :func:`~server.serve`:

- Set ``max_size`` (default: 1 MiB, UTF-8 encoded) to the maximum size of
  messages your application generates.
- Set ``max_queue`` (default: 32) to the maximum number of messages your
  application expects to receive faster than it can process them. The queue
  provides burst tolerance without slowing down the TCP connection.

Furthermore, you can lower ``read_limit`` and ``write_limit`` (default:
64 KiB) to reduce the size of buffers for incoming and outgoing data.

The design document provides :ref:`more details about buffers <buffers>`.

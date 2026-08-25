Compression
===========

.. currentmodule:: websockets.extensions.permessage_deflate

Most WebSocket servers exchange JSON messages because they're convenient to
parse and serialize in a browser. These messages contain text data and tend to
be repetitive.

This makes the stream of messages highly compressible. Enabling compression
can reduce network traffic by more than 80%.

There's a standard for compressing messages. :rfc:`7692` defines WebSocket
Per-Message Deflate, a compression extension based on the Deflate_ algorithm.

.. _Deflate: https://en.wikipedia.org/wiki/Deflate

Configuring compression
-----------------------

:func:`~websockets.client.connect` and :func:`~websockets.server.serve` enable
compression by default because the reduction in network bandwidth is usually
worth the additional memory and CPU cost.

If you want to disable compression, set ``compression=None``::

    import websockets

    websockets.connect(..., compression=None)

    websockets.serve(..., compression=None)

If you want to customize compression settings, you can enable the Per-Message
Deflate extension explicitly with :class:`ClientPerMessageDeflateFactory` or
:class:`ServerPerMessageDeflateFactory`::

    import websockets
    from websockets.extensions import permessage_deflate

    websockets.connect(
        ...,
        extensions=[
            permessage_deflate.ClientPerMessageDeflateFactory(
                server_max_window_bits=11,
                client_max_window_bits=11,
                compress_settings={"memLevel": 4},
            ),
        ],
    )

    websockets.serve(
        ...,
        extensions=[
            permessage_deflate.ServerPerMessageDeflateFactory(
                server_max_window_bits=11,
                client_max_window_bits=11,
                compress_settings={"memLevel": 4},
            ),
        ],
    )

The Window Bits and Memory Level values in these examples reduce memory usage
at the expense of compression rate.

Compression settings
--------------------

When a client and a server enable the Per-Message Deflate extension, they
negotiate two parameters to guarantee compatibility between compression and
decompression. These parameters affect the trade-off between compression rate
and memory usage for both sides.

* **Context Takeover** means that the compression context is retained between
  messages. In other words, compression is applied to the stream of messages
  rather than to each message individually.

  Context takeover should remain enabled to get good performance on
  applications that send a stream of messages with similar structure,
  that is, most applications.

  This requires retaining the compression context and state between messages,
  which increases the memory footprint of a connection.

* **Window Bits** controls the size of the compression context. It must be
  an integer between 9 (lowest memory usage) and 15 (best compression).
  Setting it to 8 is possible but rejected by some versions of zlib.

  On the server side, websockets defaults to 12. Specifically, the compression
  window size (server to client) is always 12 while the decompression window
  (client to server) size may be 12 or 15 depending on whether the client
  supports configuring it.

  On the client side, websockets lets the server pick a suitable value, which
  has the same effect as defaulting to 15.

:mod:`zlib` offers additional parameters for tuning compression. They control
the trade-off between compression rate, memory usage, and CPU usage only for
compressing. They're transparent for decompressing. Unless mentioned
otherwise, websockets inherits defaults of :func:`~zlib.compressobj`.

* **Memory Level** controls the size of the compression state. It must be an
  integer between 1 (lowest memory usage) and 9 (best compression).

  websockets defaults to 5. This is lower than zlib's default of 8. Not only
  does a lower memory level reduce memory usage, but it can also increase
  speed thanks to memory locality.

* **Compression Level** controls the effort to optimize compression. It must
  be an integer between 1 (lowest CPU usage) and 9 (best compression).

* **Strategy** selects the compression strategy. The best choice depends on
  the type of data being compressed.


Tuning compression
------------------

For servers
...........

By default, websockets enables compression with conservative settings that
optimize memory usage at the cost of a slightly worse compression rate:
Window Bits = 12 and Memory Level = 5. This strikes a good balance for small
messages that are typical of WebSocket servers.

Here's how various compression settings affect memory usage of a single
connection on a 64-bit system, as well a benchmark of compressed size and
compression time for a corpus of small JSON documents.

=========== ============ ============ ================ ================
Window Bits Memory Level Memory usage Size vs. default Time vs. default
=========== ============ ============ ================ ================
15          8            322 KiB      -4.0%            +15%
14          7            178 KiB      -2.6%            +10%
13          6            106 KiB      -1.4%            +5%
**12**      **5**        **70 KiB**   **=**            **=**
11          4            52 KiB       +3.7%            -5%
10          3            43 KiB       +90%             +50%
9           2            39 KiB       +160%            +100%
—           —            19 KiB       +452%            —
=========== ============ ============ ================ ================

Window Bits and Memory Level don't have to move in lockstep. However, other
combinations don't yield significantly better results than those shown above.

Compressed size and compression time depend heavily on the kind of messages
exchanged by the application so this example may not apply to your use case.

You can adapt `compression/benchmark.py`_ by creating a list of typical
messages and passing it to the ``_run`` function.

Window Bits = 11 and Memory Level = 4 looks like the sweet spot in this table.

websockets defaults to Window Bits = 12 and Memory Level = 5 to stay away from
Window Bits = 10 or Memory Level = 3 where performance craters, raising doubts
on what could happen at Window Bits = 11 and Memory Level = 4 on a different
corpus.

Defaults must be safe for all applications, hence a more conservative choice.

.. _compression/benchmark.py: https://github.com/python-websockets/websockets/blob/main/experiments/compression/benchmark.py

The benchmark focuses on compression because it's more expensive than
decompression. Indeed, leaving aside small allocations, theoretical memory
usage is:

* ``(1 << (windowBits + 2)) + (1 << (memLevel + 9))`` for compression;
* ``1 << windowBits`` for decompression.

CPU usage is also higher for compression than decompression.

While it's always possible for a server to use a smaller window size for
compressing outgoing messages, using a smaller window size for decompressing
incoming messages requires collaboration from clients.

When a client doesn't support configuring the size of its compression window,
websockets enables compression with the largest possible decompression window.
In most use cases, this is more efficient than disabling compression both ways.

If you are very sensitive to memory usage, you can reverse this behavior by
setting the ``require_client_max_window_bits`` parameter of
:class:`ServerPerMessageDeflateFactory` to ``True``.

For clients
...........

By default, websockets enables compression with Memory Level = 5 but leaves
the Window Bits setting up to the server.

There's two good reasons and one bad reason for not optimizing the client side
like the server side:

1. If the maintainers of a server configured some optimized settings, we don't
   want to override them with more restrictive settings.

2. Optimizing memory usage doesn't matter very much for clients because it's
   uncommon to open thousands of client connections in a program.

3. On a more pragmatic note, some servers misbehave badly when a client
   configures compression settings. `AWS API Gateway`_ is the worst offender.

   .. _AWS API Gateway: https://github.com/python-websockets/websockets/issues/1065

   Unfortunately, even though websockets is right and AWS is wrong, many users
   jump to the conclusion that websockets doesn't work.

   Until the ecosystem levels up, interoperability with buggy servers seems
   more valuable than optimizing memory usage.


Further reading
---------------

This `blog post by Ilya Grigorik`_ provides more details about how compression
settings affect memory usage and how to optimize them.

.. _blog post by Ilya Grigorik: https://www.igvita.com/2013/11/27/configuring-and-optimizing-websocket-compression/

This `experiment by Peter Thorson`_ recommends Window Bits = 11 and Memory
Level = 4 for optimizing memory usage.

.. _experiment by Peter Thorson: https://mailarchive.ietf.org/arch/msg/hybi/F9t4uPufVEy8KBLuL36cZjCmM_Y/

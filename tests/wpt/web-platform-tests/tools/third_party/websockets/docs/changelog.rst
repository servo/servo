Changelog
---------

.. currentmodule:: websockets

8.2
...

*In development*

8.1
...

* Added compatibility with Python 3.8.

8.0.2
.....

* Restored the ability to pass a socket with the ``sock`` parameter of
  :func:`~server.serve`.

* Removed an incorrect assertion when a connection drops.

8.0.1
.....

* Restored the ability to import ``WebSocketProtocolError`` from
  ``websockets``.

8.0
...

.. warning::

    **Version 8.0 drops compatibility with Python 3.4 and 3.5.**

.. note::

    **Version 8.0 expects** ``process_request`` **to be a coroutine.**

    Previously, it could be a function or a coroutine.

    If you're passing a ``process_request`` argument to :func:`~server.serve`
    or :class:`~server.WebSocketServerProtocol`, or if you're overriding
    :meth:`~protocol.WebSocketServerProtocol.process_request` in a subclass,
    define it with ``async def`` instead of ``def``.

    For backwards compatibility, functions are still mostly supported, but
    mixing functions and coroutines won't work in some inheritance scenarios.

.. note::

    **Version 8.0 changes the behavior of the** ``max_queue`` **parameter.**

    If you were setting ``max_queue=0`` to make the queue of incoming messages
    unbounded, change it to ``max_queue=None``.

.. note::

    **Version 8.0 deprecates the** ``host`` **,** ``port`` **, and** ``secure``
    **attributes of** :class:`~protocol.WebSocketCommonProtocol`.

    Use :attr:`~protocol.WebSocketCommonProtocol.local_address` in servers and
    :attr:`~protocol.WebSocketCommonProtocol.remote_address` in clients
    instead of ``host`` and ``port``.

.. note::

    **Version 8.0 renames the** ``WebSocketProtocolError`` **exception**
    to :exc:`ProtocolError` **.**

    A ``WebSocketProtocolError`` alias provides backwards compatibility.

.. note::

    **Version 8.0 adds the reason phrase to the return type of the low-level
    API** :func:`~http.read_response` **.**

Also:

* :meth:`~protocol.WebSocketCommonProtocol.send`,
  :meth:`~protocol.WebSocketCommonProtocol.ping`, and
  :meth:`~protocol.WebSocketCommonProtocol.pong` support bytes-like types
  :class:`bytearray` and :class:`memoryview` in addition to :class:`bytes`.

* Added :exc:`~exceptions.ConnectionClosedOK` and
  :exc:`~exceptions.ConnectionClosedError` subclasses of
  :exc:`~exceptions.ConnectionClosed` to tell apart normal connection
  termination from errors.

* Added :func:`~auth.basic_auth_protocol_factory` to enforce HTTP Basic Auth
  on the server side.

* :func:`~client.connect` handles redirects from the server during the
  handshake.

* :func:`~client.connect` supports overriding ``host`` and ``port``.

* Added :func:`~client.unix_connect` for connecting to Unix sockets.

* Improved support for sending fragmented messages by accepting asynchronous
  iterators in :meth:`~protocol.WebSocketCommonProtocol.send`.

* Prevented spurious log messages about :exc:`~exceptions.ConnectionClosed`
  exceptions in keepalive ping task. If you were using ``ping_timeout=None``
  as a workaround, you can remove it.

* Changed :meth:`WebSocketServer.close() <server.WebSocketServer.close>` to
  perform a proper closing handshake instead of failing the connection.

* Avoided a crash when a ``extra_headers`` callable returns ``None``.

* Improved error messages when HTTP parsing fails.

* Enabled readline in the interactive client.

* Added type hints (:pep:`484`).

* Added a FAQ to the documentation.

* Added documentation for extensions.

* Documented how to optimize memory usage.

* Improved API documentation.

7.0
...

.. warning::

    **Version 7.0 renames the** ``timeout`` **argument of**
    :func:`~server.serve()` **and** :func:`~client.connect` **to**
    ``close_timeout`` **.**

    This prevents confusion with ``ping_timeout``.

    For backwards compatibility, ``timeout`` is still supported.

.. warning::

    **Version 7.0 changes how a server terminates connections when it's
    closed with** :meth:`~server.WebSocketServer.close` **.**

    Previously, connections handlers were canceled. Now, connections are
    closed with close code 1001 (going away). From the perspective of the
    connection handler, this is the same as if the remote endpoint was
    disconnecting. This removes the need to prepare for
    :exc:`~asyncio.CancelledError` in connection handlers.

    You can restore the previous behavior by adding the following line at the
    beginning of connection handlers::

        def handler(websocket, path):
            closed = asyncio.ensure_future(websocket.wait_closed())
            closed.add_done_callback(lambda task: task.cancel())

.. note::

    **Version 7.0 changes how a** :meth:`~protocol.WebSocketCommonProtocol.ping`
    **that hasn't received a pong yet behaves when the connection is closed.**

    The ping — as in ``ping = await websocket.ping()`` — used to be canceled
    when the connection is closed, so that ``await ping`` raised
    :exc:`~asyncio.CancelledError`. Now ``await ping`` raises
    :exc:`~exceptions.ConnectionClosed` like other public APIs.

.. note::

    **Version 7.0 raises a** :exc:`RuntimeError` **exception if two coroutines
    call** :meth:`~protocol.WebSocketCommonProtocol.recv` **concurrently.**

    Concurrent calls lead to non-deterministic behavior because there are no
    guarantees about which coroutine will receive which message.

Also:

* ``websockets`` sends Ping frames at regular intervals and closes the
  connection if it doesn't receive a matching Pong frame. See
  :class:`~protocol.WebSocketCommonProtocol` for details.

* Added ``process_request`` and ``select_subprotocol`` arguments to
  :func:`~server.serve` and :class:`~server.WebSocketServerProtocol` to
  customize :meth:`~server.WebSocketServerProtocol.process_request` and
  :meth:`~server.WebSocketServerProtocol.select_subprotocol` without
  subclassing :class:`~server.WebSocketServerProtocol`.

* Added support for sending fragmented messages.

* Added the :meth:`~protocol.WebSocketCommonProtocol.wait_closed` method to
  protocols.

* Added an interactive client: ``python -m websockets <uri>``.

* Changed the ``origins`` argument to represent the lack of an origin with
  ``None`` rather than ``''``.

* Fixed a data loss bug in :meth:`~protocol.WebSocketCommonProtocol.recv`:
  canceling it at the wrong time could result in messages being dropped.

* Improved handling of multiple HTTP headers with the same name.

* Improved error messages when a required HTTP header is missing.

6.0
...

.. warning::

    **Version 6.0 introduces the** :class:`~http.Headers` **class for managing
    HTTP headers and changes several public APIs:**

    * :meth:`~server.WebSocketServerProtocol.process_request` now receives a
      :class:`~http.Headers` instead of a :class:`~http.client.HTTPMessage` in
      the ``request_headers`` argument.

    * The :attr:`~protocol.WebSocketCommonProtocol.request_headers` and
      :attr:`~protocol.WebSocketCommonProtocol.response_headers` attributes of
      :class:`~protocol.WebSocketCommonProtocol` are :class:`~http.Headers`
      instead of :class:`~http.client.HTTPMessage`.

    * The :attr:`~protocol.WebSocketCommonProtocol.raw_request_headers` and
      :attr:`~protocol.WebSocketCommonProtocol.raw_response_headers`
      attributes of :class:`~protocol.WebSocketCommonProtocol` are removed.
      Use :meth:`~http.Headers.raw_items` instead.

    * Functions defined in the :mod:`~handshake` module now receive
      :class:`~http.Headers` in argument instead of ``get_header`` or
      ``set_header`` functions. This affects libraries that rely on
      low-level APIs.

    * Functions defined in the :mod:`~http` module now return HTTP headers as
      :class:`~http.Headers` instead of lists of ``(name, value)`` pairs.

    Since :class:`~http.Headers` and :class:`~http.client.HTTPMessage` provide
    similar APIs, this change won't affect most of the code dealing with HTTP
    headers.


Also:

* Added compatibility with Python 3.7.

5.0.1
.....

* Fixed a regression in the 5.0 release that broke some invocations of
  :func:`~server.serve()` and :func:`~client.connect`.

5.0
...

.. note::

    **Version 5.0 fixes a security issue introduced in version 4.0.**

    Version 4.0 was vulnerable to denial of service by memory exhaustion
    because it didn't enforce ``max_size`` when decompressing compressed
    messages (`CVE-2018-1000518`_).

    .. _CVE-2018-1000518: https://nvd.nist.gov/vuln/detail/CVE-2018-1000518

.. note::

    **Version 5.0 adds a** ``user_info`` **field to the return value of**
    :func:`~uri.parse_uri` **and** :class:`~uri.WebSocketURI` **.**

    If you're unpacking :class:`~exceptions.WebSocketURI` into four variables,
    adjust your code to account for that fifth field.

Also:

* :func:`~client.connect` performs HTTP Basic Auth when the URI contains
  credentials.

* Iterating on incoming messages no longer raises an exception when the
  connection terminates with close code 1001 (going away).

* A plain HTTP request now receives a 426 Upgrade Required response and
  doesn't log a stack trace.

* :func:`~server.unix_serve` can be used as an asynchronous context manager on
  Python ≥ 3.5.1.

* Added the :attr:`~protocol.WebSocketCommonProtocol.closed` property to
  protocols.

* If a :meth:`~protocol.WebSocketCommonProtocol.ping` doesn't receive a pong,
  it's canceled when the connection is closed.

* Reported the cause of :exc:`~exceptions.ConnectionClosed` exceptions.

* Added new examples in the documentation.

* Updated documentation with new features from Python 3.6.

* Improved several other sections of the documentation.

* Fixed missing close code, which caused :exc:`TypeError` on connection close.

* Fixed a race condition in the closing handshake that raised
  :exc:`~exceptions.InvalidState`.

* Stopped logging stack traces when the TCP connection dies prematurely.

* Prevented writing to a closing TCP connection during unclean shutdowns.

* Made connection termination more robust to network congestion.

* Prevented processing of incoming frames after failing the connection.

4.0.1
.....

* Fixed issues with the packaging of the 4.0 release.

4.0
...

.. warning::

    **Version 4.0 enables compression with the permessage-deflate extension.**

    In August 2017, Firefox and Chrome support it, but not Safari and IE.

    Compression should improve performance but it increases RAM and CPU use.

    If you want to disable compression, add ``compression=None`` when calling
    :func:`~server.serve()` or :func:`~client.connect`.

.. warning::

    **Version 4.0 drops compatibility with Python 3.3.**

.. note::

    **Version 4.0 removes the** ``state_name`` **attribute of protocols.**

    Use ``protocol.state.name`` instead of ``protocol.state_name``.

Also:

* :class:`~protocol.WebSocketCommonProtocol` instances can be used as
  asynchronous iterators on Python ≥ 3.6. They yield incoming messages.

* Added :func:`~server.unix_serve` for listening on Unix sockets.

* Added the :attr:`~server.WebSocketServer.sockets` attribute to the return
  value of :func:`~server.serve`.

* Reorganized and extended documentation.

* Aborted connections if they don't close within the configured ``timeout``.

* Rewrote connection termination to increase robustness in edge cases.

* Stopped leaking pending tasks when :meth:`~asyncio.Task.cancel` is called on
  a connection while it's being closed.

* Reduced verbosity of "Failing the WebSocket connection" logs.

* Allowed ``extra_headers`` to override ``Server`` and ``User-Agent`` headers.

3.4
...

* Renamed :func:`~server.serve()` and :func:`~client.connect`'s ``klass``
  argument to ``create_protocol`` to reflect that it can also be a callable.
  For backwards compatibility, ``klass`` is still supported.

* :func:`~server.serve` can be used as an asynchronous context manager on
  Python ≥ 3.5.1.

* Added support for customizing handling of incoming connections with
  :meth:`~server.WebSocketServerProtocol.process_request`.

* Made read and write buffer sizes configurable.

* Rewrote HTTP handling for simplicity and performance.

* Added an optional C extension to speed up low-level operations.

* An invalid response status code during :func:`~client.connect` now raises
  :class:`~exceptions.InvalidStatusCode` with a ``code`` attribute.

* Providing a ``sock`` argument to :func:`~client.connect` no longer
  crashes.

3.3
...

* Ensured compatibility with Python 3.6.

* Reduced noise in logs caused by connection resets.

* Avoided crashing on concurrent writes on slow connections.

3.2
...

* Added ``timeout``, ``max_size``, and ``max_queue`` arguments to
  :func:`~client.connect()` and :func:`~server.serve`.

* Made server shutdown more robust.

3.1
...

* Avoided a warning when closing a connection before the opening handshake.

* Added flow control for incoming data.

3.0
...

.. warning::

    **Version 3.0 introduces a backwards-incompatible change in the**
    :meth:`~protocol.WebSocketCommonProtocol.recv` **API.**

    **If you're upgrading from 2.x or earlier, please read this carefully.**

    :meth:`~protocol.WebSocketCommonProtocol.recv` used to return ``None``
    when the connection was closed. This required checking the return value of
    every call::

        message = await websocket.recv()
        if message is None:
            return

    Now it raises a :exc:`~exceptions.ConnectionClosed` exception instead.
    This is more Pythonic. The previous code can be simplified to::

        message = await websocket.recv()

    When implementing a server, which is the more popular use case, there's no
    strong reason to handle such exceptions. Let them bubble up, terminate the
    handler coroutine, and the server will simply ignore them.

    In order to avoid stranding projects built upon an earlier version, the
    previous behavior can be restored by passing ``legacy_recv=True`` to
    :func:`~server.serve`, :func:`~client.connect`,
    :class:`~server.WebSocketServerProtocol`, or
    :class:`~client.WebSocketClientProtocol`. ``legacy_recv`` isn't documented
    in their signatures but isn't scheduled for deprecation either.

Also:

* :func:`~client.connect` can be used as an asynchronous context manager on
  Python ≥ 3.5.1.

* Updated documentation with ``await`` and ``async`` syntax from Python 3.5.

* :meth:`~protocol.WebSocketCommonProtocol.ping` and
  :meth:`~protocol.WebSocketCommonProtocol.pong` support data passed as
  :class:`str` in addition to :class:`bytes`.

* Worked around an asyncio bug affecting connection termination under load.

* Made ``state_name`` attribute on protocols a public API.

* Improved documentation.

2.7
...

* Added compatibility with Python 3.5.

* Refreshed documentation.

2.6
...

* Added ``local_address`` and ``remote_address`` attributes on protocols.

* Closed open connections with code 1001 when a server shuts down.

* Avoided TCP fragmentation of small frames.

2.5
...

* Improved documentation.

* Provided access to handshake request and response HTTP headers.

* Allowed customizing handshake request and response HTTP headers.

* Supported running on a non-default event loop.

* Returned a 403 status code instead of 400 when the request Origin isn't
  allowed.

* Canceling :meth:`~protocol.WebSocketCommonProtocol.recv` no longer drops
  the next message.

* Clarified that the closing handshake can be initiated by the client.

* Set the close code and reason more consistently.

* Strengthened connection termination by simplifying the implementation.

* Improved tests, added tox configuration, and enforced 100% branch coverage.

2.4
...

* Added support for subprotocols.

* Supported non-default event loop.

* Added ``loop`` argument to :func:`~client.connect` and
  :func:`~server.serve`.

2.3
...

* Improved compliance of close codes.

2.2
...

* Added support for limiting message size.

2.1
...

* Added ``host``, ``port`` and ``secure`` attributes on protocols.

* Added support for providing and checking Origin_.

.. _Origin: https://tools.ietf.org/html/rfc6455#section-10.2

2.0
...

.. warning::

    **Version 2.0 introduces a backwards-incompatible change in the**
    :meth:`~protocol.WebSocketCommonProtocol.send`,
    :meth:`~protocol.WebSocketCommonProtocol.ping`, and
    :meth:`~protocol.WebSocketCommonProtocol.pong` **APIs.**

    **If you're upgrading from 1.x or earlier, please read this carefully.**

    These APIs used to be functions. Now they're coroutines.

    Instead of::

        websocket.send(message)

    you must now write::

        await websocket.send(message)

Also:

* Added flow control for outgoing data.

1.0
...

* Initial public release.

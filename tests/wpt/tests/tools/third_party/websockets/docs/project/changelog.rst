Changelog
=========

.. currentmodule:: websockets

.. _backwards-compatibility policy:

Backwards-compatibility policy
------------------------------

websockets is intended for production use. Therefore, stability is a goal.

websockets also aims at providing the best API for WebSocket in Python.

While we value stability, we value progress more. When an improvement requires
changing a public API, we make the change and document it in this changelog.

When possible with reasonable effort, we preserve backwards-compatibility for
five years after the release that introduced the change.

When a release contains backwards-incompatible API changes, the major version
is increased, else the minor version is increased. Patch versions are only for
fixing regressions shortly after a release.

Only documented APIs are public. Undocumented, private APIs may change without
notice.

12.0
----

*October 21, 2023*

Backwards-incompatible changes
..............................

.. admonition:: websockets 12.0 requires Python ≥ 3.8.
    :class: tip

    websockets 11.0 is the last version supporting Python 3.7.

Improvements
............

* Made convenience imports from ``websockets`` compatible with static code
  analysis tools such as auto-completion in an IDE or type checking with mypy_.

  .. _mypy: https://github.com/python/mypy

* Accepted a plain :class:`int` where an :class:`~http.HTTPStatus` is expected.

* Added :class:`~frames.CloseCode`.

11.0.3
------

*May 7, 2023*

Bug fixes
.........

* Fixed the :mod:`threading` implementation of servers on Windows.

11.0.2
------

*April 18, 2023*

Bug fixes
.........

* Fixed a deadlock in the :mod:`threading` implementation when closing a
  connection without reading all messages.

11.0.1
------

*April 6, 2023*

Bug fixes
.........

* Restored the C extension in the source distribution.

11.0
----

*April 2, 2023*

Backwards-incompatible changes
..............................

.. admonition:: The Sans-I/O implementation was moved.
    :class: caution

    Aliases provide compatibility for all previously public APIs according to
    the `backwards-compatibility policy`_.

    * The ``connection`` module was renamed to ``protocol``.

    * The ``connection.Connection``, ``server.ServerConnection``, and
      ``client.ClientConnection`` classes were renamed to ``protocol.Protocol``,
      ``server.ServerProtocol``, and ``client.ClientProtocol``.

.. admonition:: Sans-I/O protocol constructors now use keyword-only arguments.
    :class: caution

    If you instantiate :class:`~server.ServerProtocol` or
    :class:`~client.ClientProtocol` directly, make sure you are using keyword
    arguments.

.. admonition:: Closing a connection without an empty close frame is OK.
    :class: note

    Receiving an empty close frame now results in
    :exc:`~exceptions.ConnectionClosedOK` instead of
    :exc:`~exceptions.ConnectionClosedError`.

    As a consequence, calling ``WebSocket.close()`` without arguments in a
    browser isn't reported as an error anymore.

.. admonition:: :func:`~server.serve` times out on the opening handshake after 10 seconds by default.
    :class: note

    You can adjust the timeout with the ``open_timeout`` parameter. Set it to
    :obj:`None` to disable the timeout entirely.

New features
............

.. admonition:: websockets 11.0 introduces a implementation on top of :mod:`threading`.
    :class: important

    It may be more convenient if you don't need to manage many connections and
    you're more comfortable with :mod:`threading` than :mod:`asyncio`.

    It is particularly suited to client applications that establish only one
    connection. It may be used for servers handling few connections.

    See :func:`~sync.client.connect` and :func:`~sync.server.serve` for details.

* Added ``open_timeout`` to :func:`~server.serve`.

* Made it possible to close a server without closing existing connections.

* Added :attr:`~server.ServerProtocol.select_subprotocol` to customize
  negotiation of subprotocols in the Sans-I/O layer.

Improvements
............

* Added platform-independent wheels.

* Improved error handling in :func:`~websockets.broadcast`.

* Set ``server_hostname`` automatically on TLS connections when providing a
  ``sock`` argument to :func:`~sync.client.connect`.

10.4
----

*October 25, 2022*

New features
............

* Validated compatibility with Python 3.11.

* Added the :attr:`~legacy.protocol.WebSocketCommonProtocol.latency` property to
  protocols.

* Changed :attr:`~legacy.protocol.WebSocketCommonProtocol.ping` to return the
  latency of the connection.

* Supported overriding or removing the ``User-Agent`` header in clients and the
  ``Server`` header in servers.

* Added deployment guides for more Platform as a Service providers.

Improvements
............

* Improved FAQ.

10.3
----

*April 17, 2022*

Backwards-incompatible changes
..............................

.. admonition:: The ``exception`` attribute of :class:`~http11.Request` and :class:`~http11.Response` is deprecated.
    :class: note

    Use the ``handshake_exc`` attribute of :class:`~server.ServerProtocol` and
    :class:`~client.ClientProtocol` instead.

    See :doc:`../howto/sansio` for details.

Improvements
............

* Reduced noise in logs when :mod:`ssl` or :mod:`zlib` raise exceptions.

10.2
----

*February 21, 2022*

Improvements
............

* Made compression negotiation more lax for compatibility with Firefox.

* Improved FAQ and quick start guide.

Bug fixes
.........

* Fixed backwards-incompatibility in 10.1 for connection handlers created with
  :func:`functools.partial`.

* Avoided leaking open sockets when :func:`~client.connect` is canceled.

10.1
----

*November 14, 2021*

New features
............

* Added a tutorial.

* Made the second parameter of connection handlers optional. It will be
  deprecated in the next major release. The request path is available in
  the :attr:`~legacy.protocol.WebSocketCommonProtocol.path` attribute of
  the first argument.

  If you implemented the connection handler of a server as::

      async def handler(request, path):
          ...

  You should replace it by::

      async def handler(request):
          path = request.path  # if handler() uses the path argument
          ...

* Added ``python -m websockets --version``.

Improvements
............

* Added wheels for Python 3.10, PyPy 3.7, and for more platforms.

* Reverted optimization of default compression settings for clients, mainly to
  avoid triggering bugs in poorly implemented servers like `AWS API Gateway`_.

  .. _AWS API Gateway: https://github.com/python-websockets/websockets/issues/1065

* Mirrored the entire :class:`~asyncio.Server` API
  in :class:`~server.WebSocketServer`.

* Improved performance for large messages on ARM processors.

* Documented how to auto-reload on code changes in development.

Bug fixes
.........

* Avoided half-closing TCP connections that are already closed.

10.0
----

*September 9, 2021*

Backwards-incompatible changes
..............................

.. admonition:: websockets 10.0 requires Python ≥ 3.7.
    :class: tip

    websockets 9.1 is the last version supporting Python 3.6.

.. admonition:: The ``loop`` parameter is deprecated from all APIs.
    :class: caution

    This reflects a decision made in Python 3.8. See the release notes of
    Python 3.10 for details.

    The ``loop`` parameter is also removed
    from :class:`~server.WebSocketServer`. This should be transparent.

.. admonition:: :func:`~client.connect` times out after 10 seconds by default.
    :class: note

    You can adjust the timeout with the ``open_timeout`` parameter. Set it to
    :obj:`None` to disable the timeout entirely.

.. admonition:: The ``legacy_recv`` option is deprecated.
    :class: note

    See the release notes of websockets 3.0 for details.

.. admonition:: The signature of :exc:`~exceptions.ConnectionClosed` changed.
    :class: note

    If you raise :exc:`~exceptions.ConnectionClosed` or a subclass, rather
    than catch them when websockets raises them, you must change your code.

.. admonition:: A ``msg`` parameter was added to :exc:`~exceptions.InvalidURI`.
    :class: note

    If you raise :exc:`~exceptions.InvalidURI`, rather than catch it when
    websockets raises it, you must change your code.

New features
............

.. admonition:: websockets 10.0 introduces a `Sans-I/O API
    <https://sans-io.readthedocs.io/>`_ for easier integration
    in third-party libraries.
    :class: important

    If you're integrating websockets in a library, rather than just using it,
    look at the :doc:`Sans-I/O integration guide <../howto/sansio>`.

* Added compatibility with Python 3.10.

* Added :func:`~websockets.broadcast` to send a message to many clients.

* Added support for reconnecting automatically by using
  :func:`~client.connect` as an asynchronous iterator.

* Added ``open_timeout`` to :func:`~client.connect`.

* Documented how to integrate with `Django <https://www.djangoproject.com/>`_.

* Documented how to deploy websockets in production, with several options.

* Documented how to authenticate connections.

* Documented how to broadcast messages to many connections.

Improvements
............

* Improved logging. See the :doc:`logging guide <../topics/logging>`.

* Optimized default compression settings to reduce memory usage.

* Optimized processing of client-to-server messages when the C extension isn't
  available.

* Supported relative redirects in :func:`~client.connect`.

* Handled TCP connection drops during the opening handshake.

* Made it easier to customize authentication with
  :meth:`~auth.BasicAuthWebSocketServerProtocol.check_credentials`.

* Provided additional information in :exc:`~exceptions.ConnectionClosed`
  exceptions.

* Clarified several exceptions or log messages.

* Restructured documentation.

* Improved API documentation.

* Extended FAQ.

Bug fixes
.........

* Avoided a crash when receiving a ping while the connection is closing.

9.1
---

*May 27, 2021*

Security fix
............

.. admonition:: websockets 9.1 fixes a security issue introduced in 8.0.
    :class: important

    Version 8.0 was vulnerable to timing attacks on HTTP Basic Auth passwords
    (`CVE-2021-33880`_).

    .. _CVE-2021-33880: https://nvd.nist.gov/vuln/detail/CVE-2021-33880

9.0.2
-----

*May 15, 2021*

Bug fixes
.........

* Restored compatibility of ``python -m websockets`` with Python < 3.9.

* Restored compatibility with mypy.

9.0.1
-----

*May 2, 2021*

Bug fixes
.........

* Fixed issues with the packaging of the 9.0 release.

9.0
---

*May 1, 2021*

Backwards-incompatible changes
..............................

.. admonition:: Several modules are moved or deprecated.
    :class: caution

    Aliases provide compatibility for all previously public APIs according to
    the `backwards-compatibility policy`_

    * :class:`~datastructures.Headers` and
      :exc:`~datastructures.MultipleValuesError` are moved from
      ``websockets.http`` to :mod:`websockets.datastructures`. If you're using
      them, you should adjust the import path.

    * The ``client``, ``server``, ``protocol``, and ``auth`` modules were
      moved from the ``websockets`` package to a ``websockets.legacy``
      sub-package. Despite the name, they're still fully supported.

    * The ``framing``, ``handshake``, ``headers``, ``http``, and ``uri``
      modules in the ``websockets`` package are deprecated. These modules
      provided low-level APIs for reuse by other projects, but they didn't
      reach that goal. Keeping these APIs public makes it more difficult to
      improve websockets.

    These changes pave the path for a refactoring that should be a transparent
    upgrade for most uses and facilitate integration by other projects.

.. admonition:: Convenience imports from ``websockets`` are performed lazily.
    :class: note

    While Python supports this, tools relying on static code analysis don't.
    This breaks auto-completion in an IDE or type checking with mypy_.

    .. _mypy: https://github.com/python/mypy

    If you depend on such tools, use the real import paths, which can be found
    in the API documentation, for example::

        from websockets.client import connect
        from websockets.server import serve

New features
............

* Added compatibility with Python 3.9.

Improvements
............

* Added support for IRIs in addition to URIs.

* Added close codes 1012, 1013, and 1014.

* Raised an error when passing a :class:`dict` to
  :meth:`~legacy.protocol.WebSocketCommonProtocol.send`.

* Improved error reporting.

Bug fixes
.........

* Fixed sending fragmented, compressed messages.

* Fixed ``Host`` header sent when connecting to an IPv6 address.

* Fixed creating a client or a server with an existing Unix socket.

* Aligned maximum cookie size with popular web browsers.

* Ensured cancellation always propagates, even on Python versions where
  :exc:`~asyncio.CancelledError` inherits :exc:`Exception`.

8.1
---

*November 1, 2019*

New features
............

* Added compatibility with Python 3.8.

8.0.2
-----

*July 31, 2019*

Bug fixes
.........

* Restored the ability to pass a socket with the ``sock`` parameter of
  :func:`~server.serve`.

* Removed an incorrect assertion when a connection drops.

8.0.1
-----

*July 21, 2019*

Bug fixes
.........

* Restored the ability to import ``WebSocketProtocolError`` from
  ``websockets``.

8.0
---

*July 7, 2019*

Backwards-incompatible changes
..............................

.. admonition:: websockets 8.0 requires Python ≥ 3.6.
    :class: tip

    websockets 7.0 is the last version supporting Python 3.4 and 3.5.

.. admonition:: ``process_request`` is now expected to be a coroutine.
    :class: note

    If you're passing a ``process_request`` argument to
    :func:`~server.serve` or :class:`~server.WebSocketServerProtocol`, or if
    you're overriding
    :meth:`~server.WebSocketServerProtocol.process_request` in a subclass,
    define it with ``async def`` instead of ``def``. Previously, both were supported.

    For backwards compatibility, functions are still accepted, but mixing
    functions and coroutines won't work in some inheritance scenarios.

.. admonition:: ``max_queue`` must be :obj:`None` to disable the limit.
    :class: note

    If you were setting ``max_queue=0`` to make the queue of incoming messages
    unbounded, change it to ``max_queue=None``.

.. admonition:: The ``host``, ``port``, and ``secure`` attributes
    of :class:`~legacy.protocol.WebSocketCommonProtocol` are deprecated.
    :class: note

    Use :attr:`~legacy.protocol.WebSocketCommonProtocol.local_address` in
    servers and
    :attr:`~legacy.protocol.WebSocketCommonProtocol.remote_address` in clients
    instead of ``host`` and ``port``.

.. admonition:: ``WebSocketProtocolError`` is renamed
    to :exc:`~exceptions.ProtocolError`.
    :class: note

    An alias provides backwards compatibility.

.. admonition:: ``read_response()`` now returns the reason phrase.
    :class: note

    If you're using this low-level API, you must change your code.

New features
............

* Added :func:`~auth.basic_auth_protocol_factory` to enforce HTTP
  Basic Auth on the server side.

* :func:`~client.connect` handles redirects from the server during the
  handshake.

* :func:`~client.connect` supports overriding ``host`` and ``port``.

* Added :func:`~client.unix_connect` for connecting to Unix sockets.

* Added support for asynchronous generators
  in :meth:`~legacy.protocol.WebSocketCommonProtocol.send`
  to generate fragmented messages incrementally.

* Enabled readline in the interactive client.

* Added type hints (:pep:`484`).

* Added a FAQ to the documentation.

* Added documentation for extensions.

* Documented how to optimize memory usage.

Improvements
............

* :meth:`~legacy.protocol.WebSocketCommonProtocol.send`,
  :meth:`~legacy.protocol.WebSocketCommonProtocol.ping`, and
  :meth:`~legacy.protocol.WebSocketCommonProtocol.pong` support bytes-like
  types :class:`bytearray` and :class:`memoryview` in addition to
  :class:`bytes`.

* Added :exc:`~exceptions.ConnectionClosedOK` and
  :exc:`~exceptions.ConnectionClosedError` subclasses of
  :exc:`~exceptions.ConnectionClosed` to tell apart normal connection
  termination from errors.

* Changed :meth:`WebSocketServer.close()
  <server.WebSocketServer.close>` to perform a proper closing handshake
  instead of failing the connection.

* Improved error messages when HTTP parsing fails.

* Improved API documentation.

Bug fixes
.........

* Prevented spurious log messages about :exc:`~exceptions.ConnectionClosed`
  exceptions in keepalive ping task. If you were using ``ping_timeout=None``
  as a workaround, you can remove it.

* Avoided a crash when a ``extra_headers`` callable returns :obj:`None`.

7.0
---

*November 1, 2018*

Backwards-incompatible changes
..............................

.. admonition:: Keepalive is enabled by default.
    :class: important

    websockets now sends Ping frames at regular intervals and closes the
    connection if it doesn't receive a matching Pong frame.
    See :class:`~legacy.protocol.WebSocketCommonProtocol` for details.

.. admonition:: Termination of connections by :meth:`WebSocketServer.close()
    <server.WebSocketServer.close>` changes.
    :class: caution

    Previously, connections handlers were canceled. Now, connections are
    closed with close code 1001 (going away).

    From the perspective of the connection handler, this is the same as if the
    remote endpoint was disconnecting. This removes the need to prepare for
    :exc:`~asyncio.CancelledError` in connection handlers.

    You can restore the previous behavior by adding the following line at the
    beginning of connection handlers::

        def handler(websocket, path):
            closed = asyncio.ensure_future(websocket.wait_closed())
            closed.add_done_callback(lambda task: task.cancel())

.. admonition:: Calling :meth:`~legacy.protocol.WebSocketCommonProtocol.recv`
    concurrently raises a :exc:`RuntimeError`.
    :class: note

    Concurrent calls lead to non-deterministic behavior because there are no
    guarantees about which coroutine will receive which message.

.. admonition:: The ``timeout`` argument of :func:`~server.serve`
    and :func:`~client.connect` is renamed to ``close_timeout`` .
    :class: note

    This prevents confusion with ``ping_timeout``.

    For backwards compatibility, ``timeout`` is still supported.

.. admonition:: The ``origins`` argument of :func:`~server.serve` changes.
    :class: note

    Include :obj:`None` in the list rather than ``''`` to allow requests that
    don't contain an Origin header.

.. admonition:: Pending pings aren't canceled when the connection is closed.
    :class: note

    A ping — as in ``ping = await websocket.ping()`` — for which no pong was
    received yet used to be canceled when the connection is closed, so that
    ``await ping`` raised :exc:`~asyncio.CancelledError`.

    Now ``await ping`` raises :exc:`~exceptions.ConnectionClosed` like other
    public APIs.

New features
............

* Added ``process_request`` and ``select_subprotocol`` arguments to
  :func:`~server.serve` and
  :class:`~server.WebSocketServerProtocol` to facilitate customization of
  :meth:`~server.WebSocketServerProtocol.process_request` and
  :meth:`~server.WebSocketServerProtocol.select_subprotocol`.

* Added support for sending fragmented messages.

* Added the :meth:`~legacy.protocol.WebSocketCommonProtocol.wait_closed`
  method to protocols.

* Added an interactive client: ``python -m websockets <uri>``.

Improvements
............

* Improved handling of multiple HTTP headers with the same name.

* Improved error messages when a required HTTP header is missing.

Bug fixes
.........

* Fixed a data loss bug in
  :meth:`~legacy.protocol.WebSocketCommonProtocol.recv`:
  canceling it at the wrong time could result in messages being dropped.

6.0
---

*July 16, 2018*

Backwards-incompatible changes
..............................

.. admonition:: The :class:`~datastructures.Headers` class is introduced and
    several APIs are updated to use it.
    :class: caution

    * The ``request_headers`` argument
      of :meth:`~server.WebSocketServerProtocol.process_request` is now
      a :class:`~datastructures.Headers` instead of
      an ``http.client.HTTPMessage``.

    * The ``request_headers`` and ``response_headers`` attributes of
      :class:`~legacy.protocol.WebSocketCommonProtocol` are now
      :class:`~datastructures.Headers` instead of ``http.client.HTTPMessage``.

    * The ``raw_request_headers`` and ``raw_response_headers`` attributes of
      :class:`~legacy.protocol.WebSocketCommonProtocol` are removed. Use
      :meth:`~datastructures.Headers.raw_items` instead.

    * Functions defined in the ``handshake`` module now receive
      :class:`~datastructures.Headers` in argument instead of ``get_header``
      or ``set_header`` functions. This affects libraries that rely on
      low-level APIs.

    * Functions defined in the ``http`` module now return HTTP headers as
      :class:`~datastructures.Headers` instead of lists of ``(name, value)``
      pairs.

    Since :class:`~datastructures.Headers` and ``http.client.HTTPMessage``
    provide similar APIs, much of the code dealing with HTTP headers won't
    require changes.

New features
............

* Added compatibility with Python 3.7.

5.0.1
-----

*May 24, 2018*

Bug fixes
.........

* Fixed a regression in 5.0 that broke some invocations of
  :func:`~server.serve` and :func:`~client.connect`.

5.0
---

*May 22, 2018*

Security fix
............

.. admonition:: websockets 5.0 fixes a security issue introduced in 4.0.
    :class: important

    Version 4.0 was vulnerable to denial of service by memory exhaustion
    because it didn't enforce ``max_size`` when decompressing compressed
    messages (`CVE-2018-1000518`_).

    .. _CVE-2018-1000518: https://nvd.nist.gov/vuln/detail/CVE-2018-1000518

Backwards-incompatible changes
..............................

.. admonition:: A ``user_info`` field is added to the return value of
    ``parse_uri`` and ``WebSocketURI``.
    :class: note

    If you're unpacking ``WebSocketURI`` into four variables, adjust your code
    to account for that fifth field.

New features
............

* :func:`~client.connect` performs HTTP Basic Auth when the URI contains
  credentials.

* :func:`~server.unix_serve` can be used as an asynchronous context
  manager on Python ≥ 3.5.1.

* Added the :attr:`~legacy.protocol.WebSocketCommonProtocol.closed` property
  to protocols.

* Added new examples in the documentation.

Improvements
............

* Iterating on incoming messages no longer raises an exception when the
  connection terminates with close code 1001 (going away).

* A plain HTTP request now receives a 426 Upgrade Required response and
  doesn't log a stack trace.

* If a :meth:`~legacy.protocol.WebSocketCommonProtocol.ping` doesn't receive a
  pong, it's canceled when the connection is closed.

* Reported the cause of :exc:`~exceptions.ConnectionClosed` exceptions.

* Stopped logging stack traces when the TCP connection dies prematurely.

* Prevented writing to a closing TCP connection during unclean shutdowns.

* Made connection termination more robust to network congestion.

* Prevented processing of incoming frames after failing the connection.

* Updated documentation with new features from Python 3.6.

* Improved several sections of the documentation.

Bug fixes
.........

* Prevented :exc:`TypeError` due to missing close code on connection close.

* Fixed a race condition in the closing handshake that raised
  :exc:`~exceptions.InvalidState`.

4.0.1
-----

*November 2, 2017*

Bug fixes
.........

* Fixed issues with the packaging of the 4.0 release.

4.0
---

*November 2, 2017*

Backwards-incompatible changes
..............................

.. admonition:: websockets 4.0 requires Python ≥ 3.4.
    :class: tip

    websockets 3.4 is the last version supporting Python 3.3.

.. admonition:: Compression is enabled by default.
    :class: important

    In August 2017, Firefox and Chrome support the permessage-deflate
    extension, but not Safari and IE.

    Compression should improve performance but it increases RAM and CPU use.

    If you want to disable compression, add ``compression=None`` when calling
    :func:`~server.serve` or :func:`~client.connect`.

.. admonition:: The ``state_name`` attribute of protocols is deprecated.
    :class: note

    Use ``protocol.state.name`` instead of ``protocol.state_name``.

New features
............

* :class:`~legacy.protocol.WebSocketCommonProtocol` instances can be used as
  asynchronous iterators on Python ≥ 3.6. They yield incoming messages.

* Added :func:`~server.unix_serve` for listening on Unix sockets.

* Added the :attr:`~server.WebSocketServer.sockets` attribute to the
  return value of :func:`~server.serve`.

* Allowed ``extra_headers`` to override ``Server`` and ``User-Agent`` headers.

Improvements
............

* Reorganized and extended documentation.

* Rewrote connection termination to increase robustness in edge cases.

* Reduced verbosity of "Failing the WebSocket connection" logs.

Bug fixes
.........

* Aborted connections if they don't close within the configured ``timeout``.

* Stopped leaking pending tasks when :meth:`~asyncio.Task.cancel` is called on
  a connection while it's being closed.

3.4
---

*August 20, 2017*

Backwards-incompatible changes
..............................

.. admonition:: ``InvalidStatus`` is replaced
    by :class:`~exceptions.InvalidStatusCode`.
    :class: note

    This exception is raised when :func:`~client.connect` receives an invalid
    response status code from the server.

New features
............

* :func:`~server.serve` can be used as an asynchronous context manager
  on Python ≥ 3.5.1.

* Added support for customizing handling of incoming connections with
  :meth:`~server.WebSocketServerProtocol.process_request`.

* Made read and write buffer sizes configurable.

Improvements
............

* Renamed :func:`~server.serve` and :func:`~client.connect`'s
  ``klass`` argument to ``create_protocol`` to reflect that it can also be a
  callable. For backwards compatibility, ``klass`` is still supported.

* Rewrote HTTP handling for simplicity and performance.

* Added an optional C extension to speed up low-level operations.

Bug fixes
.........

* Providing a ``sock`` argument to :func:`~client.connect` no longer
  crashes.

3.3
---

*March 29, 2017*

New features
............

* Ensured compatibility with Python 3.6.

Improvements
............

* Reduced noise in logs caused by connection resets.

Bug fixes
.........

* Avoided crashing on concurrent writes on slow connections.

3.2
---

*August 17, 2016*

New features
............

* Added ``timeout``, ``max_size``, and ``max_queue`` arguments to
  :func:`~client.connect` and :func:`~server.serve`.

Improvements
............

* Made server shutdown more robust.

3.1
---

*April 21, 2016*

New features
............

* Added flow control for incoming data.

Bug fixes
.........

* Avoided a warning when closing a connection before the opening handshake.

3.0
---

*December 25, 2015*

Backwards-incompatible changes
..............................

.. admonition:: :meth:`~legacy.protocol.WebSocketCommonProtocol.recv` now
    raises an exception when the connection is closed.
    :class: caution

    :meth:`~legacy.protocol.WebSocketCommonProtocol.recv` used to return
    :obj:`None` when the connection was closed. This required checking the
    return value of every call::

        message = await websocket.recv()
        if message is None:
            return

    Now it raises a :exc:`~exceptions.ConnectionClosed` exception instead.
    This is more Pythonic. The previous code can be simplified to::

        message = await websocket.recv()

    When implementing a server, there's no strong reason to handle such
    exceptions. Let them bubble up, terminate the handler coroutine, and the
    server will simply ignore them.

    In order to avoid stranding projects built upon an earlier version, the
    previous behavior can be restored by passing ``legacy_recv=True`` to
    :func:`~server.serve`, :func:`~client.connect`,
    :class:`~server.WebSocketServerProtocol`, or
    :class:`~client.WebSocketClientProtocol`.

New features
............

* :func:`~client.connect` can be used as an asynchronous context
  manager on Python ≥ 3.5.1.

* :meth:`~legacy.protocol.WebSocketCommonProtocol.ping` and
  :meth:`~legacy.protocol.WebSocketCommonProtocol.pong` support data passed as
  :class:`str` in addition to :class:`bytes`.

* Made ``state_name`` attribute on protocols a public API.

Improvements
............

* Updated documentation with ``await`` and ``async`` syntax from Python 3.5.

* Worked around an :mod:`asyncio` bug affecting connection termination under
  load.

* Improved documentation.

2.7
---

*November 18, 2015*

New features
............

* Added compatibility with Python 3.5.

Improvements
............

* Refreshed documentation.

2.6
---

*August 18, 2015*

New features
............

* Added ``local_address`` and ``remote_address`` attributes on protocols.

* Closed open connections with code 1001 when a server shuts down.

Bug fixes
.........

* Avoided TCP fragmentation of small frames.

2.5
---

*July 28, 2015*

New features
............

* Provided access to handshake request and response HTTP headers.

* Allowed customizing handshake request and response HTTP headers.

* Added support for running on a non-default event loop.

Improvements
............

* Improved documentation.

* Sent a 403 status code instead of 400 when request Origin isn't allowed.

* Clarified that the closing handshake can be initiated by the client.

* Set the close code and reason more consistently.

* Strengthened connection termination.

Bug fixes
.........

* Canceling :meth:`~legacy.protocol.WebSocketCommonProtocol.recv` no longer
  drops the next message.

2.4
---

*January 31, 2015*

New features
............

* Added support for subprotocols.

* Added ``loop`` argument to :func:`~client.connect` and
  :func:`~server.serve`.

2.3
---

*November 3, 2014*

Improvements
............

* Improved compliance of close codes.

2.2
---

*July 28, 2014*

New features
............

* Added support for limiting message size.

2.1
---

*April 26, 2014*

New features
............

* Added ``host``, ``port`` and ``secure`` attributes on protocols.

* Added support for providing and checking Origin_.

.. _Origin: https://www.rfc-editor.org/rfc/rfc6455.html#section-10.2

2.0
---

*February 16, 2014*

Backwards-incompatible changes
..............................

.. admonition:: :meth:`~legacy.protocol.WebSocketCommonProtocol.send`,
    :meth:`~legacy.protocol.WebSocketCommonProtocol.ping`, and
    :meth:`~legacy.protocol.WebSocketCommonProtocol.pong` are now coroutines.
    :class: caution

    They used to be functions.

    Instead of::

        websocket.send(message)

    you must write::

        await websocket.send(message)

New features
............

* Added flow control for outgoing data.

1.0
---

*November 14, 2013*

New features
............

* Initial public release.

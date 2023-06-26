"""
:mod:`websockets.client` defines the WebSocket client APIs.

"""

import asyncio
import collections.abc
import functools
import logging
import warnings
from types import TracebackType
from typing import Any, Generator, List, Optional, Sequence, Tuple, Type, cast

from .exceptions import (
    InvalidHandshake,
    InvalidHeader,
    InvalidMessage,
    InvalidStatusCode,
    NegotiationError,
    RedirectHandshake,
    SecurityError,
)
from .extensions.base import ClientExtensionFactory, Extension
from .extensions.permessage_deflate import ClientPerMessageDeflateFactory
from .handshake import build_request, check_response
from .headers import (
    build_authorization_basic,
    build_extension,
    build_subprotocol,
    parse_extension,
    parse_subprotocol,
)
from .http import USER_AGENT, Headers, HeadersLike, read_response
from .protocol import WebSocketCommonProtocol
from .typing import ExtensionHeader, Origin, Subprotocol
from .uri import WebSocketURI, parse_uri


__all__ = ["connect", "unix_connect", "WebSocketClientProtocol"]

logger = logging.getLogger(__name__)


class WebSocketClientProtocol(WebSocketCommonProtocol):
    """
    :class:`~asyncio.Protocol` subclass implementing a WebSocket client.

    This class inherits most of its methods from
    :class:`~websockets.protocol.WebSocketCommonProtocol`.

    """

    is_client = True
    side = "client"

    def __init__(
        self,
        *,
        origin: Optional[Origin] = None,
        extensions: Optional[Sequence[ClientExtensionFactory]] = None,
        subprotocols: Optional[Sequence[Subprotocol]] = None,
        extra_headers: Optional[HeadersLike] = None,
        **kwargs: Any,
    ) -> None:
        self.origin = origin
        self.available_extensions = extensions
        self.available_subprotocols = subprotocols
        self.extra_headers = extra_headers
        super().__init__(**kwargs)

    def write_http_request(self, path: str, headers: Headers) -> None:
        """
        Write request line and headers to the HTTP request.

        """
        self.path = path
        self.request_headers = headers

        logger.debug("%s > GET %s HTTP/1.1", self.side, path)
        logger.debug("%s > %r", self.side, headers)

        # Since the path and headers only contain ASCII characters,
        # we can keep this simple.
        request = f"GET {path} HTTP/1.1\r\n"
        request += str(headers)

        self.transport.write(request.encode())

    async def read_http_response(self) -> Tuple[int, Headers]:
        """
        Read status line and headers from the HTTP response.

        If the response contains a body, it may be read from ``self.reader``
        after this coroutine returns.

        :raises ~websockets.exceptions.InvalidMessage: if the HTTP message is
            malformed or isn't an HTTP/1.1 GET response

        """
        try:
            status_code, reason, headers = await read_response(self.reader)
        except Exception as exc:
            raise InvalidMessage("did not receive a valid HTTP response") from exc

        logger.debug("%s < HTTP/1.1 %d %s", self.side, status_code, reason)
        logger.debug("%s < %r", self.side, headers)

        self.response_headers = headers

        return status_code, self.response_headers

    @staticmethod
    def process_extensions(
        headers: Headers,
        available_extensions: Optional[Sequence[ClientExtensionFactory]],
    ) -> List[Extension]:
        """
        Handle the Sec-WebSocket-Extensions HTTP response header.

        Check that each extension is supported, as well as its parameters.

        Return the list of accepted extensions.

        Raise :exc:`~websockets.exceptions.InvalidHandshake` to abort the
        connection.

        :rfc:`6455` leaves the rules up to the specification of each
        :extension.

        To provide this level of flexibility, for each extension accepted by
        the server, we check for a match with each extension available in the
        client configuration. If no match is found, an exception is raised.

        If several variants of the same extension are accepted by the server,
        it may be configured severel times, which won't make sense in general.
        Extensions must implement their own requirements. For this purpose,
        the list of previously accepted extensions is provided.

        Other requirements, for example related to mandatory extensions or the
        order of extensions, may be implemented by overriding this method.

        """
        accepted_extensions: List[Extension] = []

        header_values = headers.get_all("Sec-WebSocket-Extensions")

        if header_values:

            if available_extensions is None:
                raise InvalidHandshake("no extensions supported")

            parsed_header_values: List[ExtensionHeader] = sum(
                [parse_extension(header_value) for header_value in header_values], []
            )

            for name, response_params in parsed_header_values:

                for extension_factory in available_extensions:

                    # Skip non-matching extensions based on their name.
                    if extension_factory.name != name:
                        continue

                    # Skip non-matching extensions based on their params.
                    try:
                        extension = extension_factory.process_response_params(
                            response_params, accepted_extensions
                        )
                    except NegotiationError:
                        continue

                    # Add matching extension to the final list.
                    accepted_extensions.append(extension)

                    # Break out of the loop once we have a match.
                    break

                # If we didn't break from the loop, no extension in our list
                # matched what the server sent. Fail the connection.
                else:
                    raise NegotiationError(
                        f"Unsupported extension: "
                        f"name = {name}, params = {response_params}"
                    )

        return accepted_extensions

    @staticmethod
    def process_subprotocol(
        headers: Headers, available_subprotocols: Optional[Sequence[Subprotocol]]
    ) -> Optional[Subprotocol]:
        """
        Handle the Sec-WebSocket-Protocol HTTP response header.

        Check that it contains exactly one supported subprotocol.

        Return the selected subprotocol.

        """
        subprotocol: Optional[Subprotocol] = None

        header_values = headers.get_all("Sec-WebSocket-Protocol")

        if header_values:

            if available_subprotocols is None:
                raise InvalidHandshake("no subprotocols supported")

            parsed_header_values: Sequence[Subprotocol] = sum(
                [parse_subprotocol(header_value) for header_value in header_values], []
            )

            if len(parsed_header_values) > 1:
                subprotocols = ", ".join(parsed_header_values)
                raise InvalidHandshake(f"multiple subprotocols: {subprotocols}")

            subprotocol = parsed_header_values[0]

            if subprotocol not in available_subprotocols:
                raise NegotiationError(f"unsupported subprotocol: {subprotocol}")

        return subprotocol

    async def handshake(
        self,
        wsuri: WebSocketURI,
        origin: Optional[Origin] = None,
        available_extensions: Optional[Sequence[ClientExtensionFactory]] = None,
        available_subprotocols: Optional[Sequence[Subprotocol]] = None,
        extra_headers: Optional[HeadersLike] = None,
    ) -> None:
        """
        Perform the client side of the opening handshake.

        :param origin: sets the Origin HTTP header
        :param available_extensions: list of supported extensions in the order
            in which they should be used
        :param available_subprotocols: list of supported subprotocols in order
            of decreasing preference
        :param extra_headers: sets additional HTTP request headers; it must be
            a :class:`~websockets.http.Headers` instance, a
            :class:`~collections.abc.Mapping`, or an iterable of ``(name,
            value)`` pairs
        :raises ~websockets.exceptions.InvalidHandshake: if the handshake
            fails

        """
        request_headers = Headers()

        if wsuri.port == (443 if wsuri.secure else 80):  # pragma: no cover
            request_headers["Host"] = wsuri.host
        else:
            request_headers["Host"] = f"{wsuri.host}:{wsuri.port}"

        if wsuri.user_info:
            request_headers["Authorization"] = build_authorization_basic(
                *wsuri.user_info
            )

        if origin is not None:
            request_headers["Origin"] = origin

        key = build_request(request_headers)

        if available_extensions is not None:
            extensions_header = build_extension(
                [
                    (extension_factory.name, extension_factory.get_request_params())
                    for extension_factory in available_extensions
                ]
            )
            request_headers["Sec-WebSocket-Extensions"] = extensions_header

        if available_subprotocols is not None:
            protocol_header = build_subprotocol(available_subprotocols)
            request_headers["Sec-WebSocket-Protocol"] = protocol_header

        if extra_headers is not None:
            if isinstance(extra_headers, Headers):
                extra_headers = extra_headers.raw_items()
            elif isinstance(extra_headers, collections.abc.Mapping):
                extra_headers = extra_headers.items()
            for name, value in extra_headers:
                request_headers[name] = value

        request_headers.setdefault("User-Agent", USER_AGENT)

        self.write_http_request(wsuri.resource_name, request_headers)

        status_code, response_headers = await self.read_http_response()
        if status_code in (301, 302, 303, 307, 308):
            if "Location" not in response_headers:
                raise InvalidHeader("Location")
            raise RedirectHandshake(response_headers["Location"])
        elif status_code != 101:
            raise InvalidStatusCode(status_code)

        check_response(response_headers, key)

        self.extensions = self.process_extensions(
            response_headers, available_extensions
        )

        self.subprotocol = self.process_subprotocol(
            response_headers, available_subprotocols
        )

        self.connection_open()


class Connect:
    """
    Connect to the WebSocket server at the given ``uri``.

    Awaiting :func:`connect` yields a :class:`WebSocketClientProtocol` which
    can then be used to send and receive messages.

    :func:`connect` can also be used as a asynchronous context manager. In
    that case, the connection is closed when exiting the context.

    :func:`connect` is a wrapper around the event loop's
    :meth:`~asyncio.loop.create_connection` method. Unknown keyword arguments
    are passed to :meth:`~asyncio.loop.create_connection`.

    For example, you can set the ``ssl`` keyword argument to a
    :class:`~ssl.SSLContext` to enforce some TLS settings. When connecting to
    a ``wss://`` URI, if this argument isn't provided explicitly,
    :func:`ssl.create_default_context` is called to create a context.

    You can connect to a different host and port from those found in ``uri``
    by setting ``host`` and ``port`` keyword arguments. This only changes the
    destination of the TCP connection. The host name from ``uri`` is still
    used in the TLS handshake for secure connections and in the ``Host`` HTTP
    header.

    The ``create_protocol`` parameter allows customizing the
    :class:`~asyncio.Protocol` that manages the connection. It should be a
    callable or class accepting the same arguments as
    :class:`WebSocketClientProtocol` and returning an instance of
    :class:`WebSocketClientProtocol` or a subclass. It defaults to
    :class:`WebSocketClientProtocol`.

    The behavior of ``ping_interval``, ``ping_timeout``, ``close_timeout``,
    ``max_size``, ``max_queue``, ``read_limit``, and ``write_limit`` is
    described in :class:`~websockets.protocol.WebSocketCommonProtocol`.

    :func:`connect` also accepts the following optional arguments:

    * ``compression`` is a shortcut to configure compression extensions;
      by default it enables the "permessage-deflate" extension; set it to
      ``None`` to disable compression
    * ``origin`` sets the Origin HTTP header
    * ``extensions`` is a list of supported extensions in order of
      decreasing preference
    * ``subprotocols`` is a list of supported subprotocols in order of
      decreasing preference
    * ``extra_headers`` sets additional HTTP request headers; it can be a
      :class:`~websockets.http.Headers` instance, a
      :class:`~collections.abc.Mapping`, or an iterable of ``(name, value)``
      pairs

    :raises ~websockets.uri.InvalidURI: if ``uri`` is invalid
    :raises ~websockets.handshake.InvalidHandshake: if the opening handshake
        fails

    """

    MAX_REDIRECTS_ALLOWED = 10

    def __init__(
        self,
        uri: str,
        *,
        path: Optional[str] = None,
        create_protocol: Optional[Type[WebSocketClientProtocol]] = None,
        ping_interval: float = 20,
        ping_timeout: float = 20,
        close_timeout: Optional[float] = None,
        max_size: int = 2 ** 20,
        max_queue: int = 2 ** 5,
        read_limit: int = 2 ** 16,
        write_limit: int = 2 ** 16,
        loop: Optional[asyncio.AbstractEventLoop] = None,
        legacy_recv: bool = False,
        klass: Optional[Type[WebSocketClientProtocol]] = None,
        timeout: Optional[float] = None,
        compression: Optional[str] = "deflate",
        origin: Optional[Origin] = None,
        extensions: Optional[Sequence[ClientExtensionFactory]] = None,
        subprotocols: Optional[Sequence[Subprotocol]] = None,
        extra_headers: Optional[HeadersLike] = None,
        **kwargs: Any,
    ) -> None:
        # Backwards compatibility: close_timeout used to be called timeout.
        if timeout is None:
            timeout = 10
        else:
            warnings.warn("rename timeout to close_timeout", DeprecationWarning)
        # If both are specified, timeout is ignored.
        if close_timeout is None:
            close_timeout = timeout

        # Backwards compatibility: create_protocol used to be called klass.
        if klass is None:
            klass = WebSocketClientProtocol
        else:
            warnings.warn("rename klass to create_protocol", DeprecationWarning)
        # If both are specified, klass is ignored.
        if create_protocol is None:
            create_protocol = klass

        if loop is None:
            loop = asyncio.get_event_loop()

        wsuri = parse_uri(uri)
        if wsuri.secure:
            kwargs.setdefault("ssl", True)
        elif kwargs.get("ssl") is not None:
            raise ValueError(
                "connect() received a ssl argument for a ws:// URI, "
                "use a wss:// URI to enable TLS"
            )

        if compression == "deflate":
            if extensions is None:
                extensions = []
            if not any(
                extension_factory.name == ClientPerMessageDeflateFactory.name
                for extension_factory in extensions
            ):
                extensions = list(extensions) + [
                    ClientPerMessageDeflateFactory(client_max_window_bits=True)
                ]
        elif compression is not None:
            raise ValueError(f"unsupported compression: {compression}")

        factory = functools.partial(
            create_protocol,
            ping_interval=ping_interval,
            ping_timeout=ping_timeout,
            close_timeout=close_timeout,
            max_size=max_size,
            max_queue=max_queue,
            read_limit=read_limit,
            write_limit=write_limit,
            loop=loop,
            host=wsuri.host,
            port=wsuri.port,
            secure=wsuri.secure,
            legacy_recv=legacy_recv,
            origin=origin,
            extensions=extensions,
            subprotocols=subprotocols,
            extra_headers=extra_headers,
        )

        if path is None:
            host: Optional[str]
            port: Optional[int]
            if kwargs.get("sock") is None:
                host, port = wsuri.host, wsuri.port
            else:
                # If sock is given, host and port shouldn't be specified.
                host, port = None, None
            # If host and port are given, override values from the URI.
            host = kwargs.pop("host", host)
            port = kwargs.pop("port", port)
            create_connection = functools.partial(
                loop.create_connection, factory, host, port, **kwargs
            )
        else:
            create_connection = functools.partial(
                loop.create_unix_connection, factory, path, **kwargs
            )

        # This is a coroutine function.
        self._create_connection = create_connection
        self._wsuri = wsuri

    def handle_redirect(self, uri: str) -> None:
        # Update the state of this instance to connect to a new URI.
        old_wsuri = self._wsuri
        new_wsuri = parse_uri(uri)

        # Forbid TLS downgrade.
        if old_wsuri.secure and not new_wsuri.secure:
            raise SecurityError("redirect from WSS to WS")

        same_origin = (
            old_wsuri.host == new_wsuri.host and old_wsuri.port == new_wsuri.port
        )

        # Rewrite the host and port arguments for cross-origin redirects.
        # This preserves connection overrides with the host and port
        # arguments if the redirect points to the same host and port.
        if not same_origin:
            # Replace the host and port argument passed to the protocol factory.
            factory = self._create_connection.args[0]
            factory = functools.partial(
                factory.func,
                *factory.args,
                **dict(factory.keywords, host=new_wsuri.host, port=new_wsuri.port),
            )
            # Replace the host and port argument passed to create_connection.
            self._create_connection = functools.partial(
                self._create_connection.func,
                *(factory, new_wsuri.host, new_wsuri.port),
                **self._create_connection.keywords,
            )

        # Set the new WebSocket URI. This suffices for same-origin redirects.
        self._wsuri = new_wsuri

    # async with connect(...)

    async def __aenter__(self) -> WebSocketClientProtocol:
        return await self

    async def __aexit__(
        self,
        exc_type: Optional[Type[BaseException]],
        exc_value: Optional[BaseException],
        traceback: Optional[TracebackType],
    ) -> None:
        await self.ws_client.close()

    # await connect(...)

    def __await__(self) -> Generator[Any, None, WebSocketClientProtocol]:
        # Create a suitable iterator by calling __await__ on a coroutine.
        return self.__await_impl__().__await__()

    async def __await_impl__(self) -> WebSocketClientProtocol:
        for redirects in range(self.MAX_REDIRECTS_ALLOWED):
            transport, protocol = await self._create_connection()
            # https://github.com/python/typeshed/pull/2756
            transport = cast(asyncio.Transport, transport)
            protocol = cast(WebSocketClientProtocol, protocol)

            try:
                try:
                    await protocol.handshake(
                        self._wsuri,
                        origin=protocol.origin,
                        available_extensions=protocol.available_extensions,
                        available_subprotocols=protocol.available_subprotocols,
                        extra_headers=protocol.extra_headers,
                    )
                except Exception:
                    protocol.fail_connection()
                    await protocol.wait_closed()
                    raise
                else:
                    self.ws_client = protocol
                    return protocol
            except RedirectHandshake as exc:
                self.handle_redirect(exc.uri)
        else:
            raise SecurityError("too many redirects")

    # yield from connect(...)

    __iter__ = __await__


connect = Connect


def unix_connect(path: str, uri: str = "ws://localhost/", **kwargs: Any) -> Connect:
    """
    Similar to :func:`connect`, but for connecting to a Unix socket.

    This function calls the event loop's
    :meth:`~asyncio.loop.create_unix_connection` method.

    It is only available on Unix.

    It's mainly useful for debugging servers listening on Unix sockets.

    :param path: file system path to the Unix socket
    :param uri: WebSocket URI

    """
    return connect(uri=uri, path=path, **kwargs)

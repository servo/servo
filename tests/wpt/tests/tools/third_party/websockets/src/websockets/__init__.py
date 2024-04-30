from __future__ import annotations

import typing

from .imports import lazy_import
from .version import version as __version__  # noqa: F401


__all__ = [
    # .client
    "ClientProtocol",
    # .datastructures
    "Headers",
    "HeadersLike",
    "MultipleValuesError",
    # .exceptions
    "AbortHandshake",
    "ConnectionClosed",
    "ConnectionClosedError",
    "ConnectionClosedOK",
    "DuplicateParameter",
    "InvalidHandshake",
    "InvalidHeader",
    "InvalidHeaderFormat",
    "InvalidHeaderValue",
    "InvalidMessage",
    "InvalidOrigin",
    "InvalidParameterName",
    "InvalidParameterValue",
    "InvalidState",
    "InvalidStatus",
    "InvalidStatusCode",
    "InvalidUpgrade",
    "InvalidURI",
    "NegotiationError",
    "PayloadTooBig",
    "ProtocolError",
    "RedirectHandshake",
    "SecurityError",
    "WebSocketException",
    "WebSocketProtocolError",
    # .legacy.auth
    "BasicAuthWebSocketServerProtocol",
    "basic_auth_protocol_factory",
    # .legacy.client
    "WebSocketClientProtocol",
    "connect",
    "unix_connect",
    # .legacy.protocol
    "WebSocketCommonProtocol",
    "broadcast",
    # .legacy.server
    "WebSocketServer",
    "WebSocketServerProtocol",
    "serve",
    "unix_serve",
    # .server
    "ServerProtocol",
    # .typing
    "Data",
    "ExtensionName",
    "ExtensionParameter",
    "LoggerLike",
    "StatusLike",
    "Origin",
    "Subprotocol",
]

# When type checking, import non-deprecated aliases eagerly. Else, import on demand.
if typing.TYPE_CHECKING:
    from .client import ClientProtocol
    from .datastructures import Headers, HeadersLike, MultipleValuesError
    from .exceptions import (
        AbortHandshake,
        ConnectionClosed,
        ConnectionClosedError,
        ConnectionClosedOK,
        DuplicateParameter,
        InvalidHandshake,
        InvalidHeader,
        InvalidHeaderFormat,
        InvalidHeaderValue,
        InvalidMessage,
        InvalidOrigin,
        InvalidParameterName,
        InvalidParameterValue,
        InvalidState,
        InvalidStatus,
        InvalidStatusCode,
        InvalidUpgrade,
        InvalidURI,
        NegotiationError,
        PayloadTooBig,
        ProtocolError,
        RedirectHandshake,
        SecurityError,
        WebSocketException,
        WebSocketProtocolError,
    )
    from .legacy.auth import (
        BasicAuthWebSocketServerProtocol,
        basic_auth_protocol_factory,
    )
    from .legacy.client import WebSocketClientProtocol, connect, unix_connect
    from .legacy.protocol import WebSocketCommonProtocol, broadcast
    from .legacy.server import (
        WebSocketServer,
        WebSocketServerProtocol,
        serve,
        unix_serve,
    )
    from .server import ServerProtocol
    from .typing import (
        Data,
        ExtensionName,
        ExtensionParameter,
        LoggerLike,
        Origin,
        StatusLike,
        Subprotocol,
    )
else:
    lazy_import(
        globals(),
        aliases={
            # .client
            "ClientProtocol": ".client",
            # .datastructures
            "Headers": ".datastructures",
            "HeadersLike": ".datastructures",
            "MultipleValuesError": ".datastructures",
            # .exceptions
            "AbortHandshake": ".exceptions",
            "ConnectionClosed": ".exceptions",
            "ConnectionClosedError": ".exceptions",
            "ConnectionClosedOK": ".exceptions",
            "DuplicateParameter": ".exceptions",
            "InvalidHandshake": ".exceptions",
            "InvalidHeader": ".exceptions",
            "InvalidHeaderFormat": ".exceptions",
            "InvalidHeaderValue": ".exceptions",
            "InvalidMessage": ".exceptions",
            "InvalidOrigin": ".exceptions",
            "InvalidParameterName": ".exceptions",
            "InvalidParameterValue": ".exceptions",
            "InvalidState": ".exceptions",
            "InvalidStatus": ".exceptions",
            "InvalidStatusCode": ".exceptions",
            "InvalidUpgrade": ".exceptions",
            "InvalidURI": ".exceptions",
            "NegotiationError": ".exceptions",
            "PayloadTooBig": ".exceptions",
            "ProtocolError": ".exceptions",
            "RedirectHandshake": ".exceptions",
            "SecurityError": ".exceptions",
            "WebSocketException": ".exceptions",
            "WebSocketProtocolError": ".exceptions",
            # .legacy.auth
            "BasicAuthWebSocketServerProtocol": ".legacy.auth",
            "basic_auth_protocol_factory": ".legacy.auth",
            # .legacy.client
            "WebSocketClientProtocol": ".legacy.client",
            "connect": ".legacy.client",
            "unix_connect": ".legacy.client",
            # .legacy.protocol
            "WebSocketCommonProtocol": ".legacy.protocol",
            "broadcast": ".legacy.protocol",
            # .legacy.server
            "WebSocketServer": ".legacy.server",
            "WebSocketServerProtocol": ".legacy.server",
            "serve": ".legacy.server",
            "unix_serve": ".legacy.server",
            # .server
            "ServerProtocol": ".server",
            # .typing
            "Data": ".typing",
            "ExtensionName": ".typing",
            "ExtensionParameter": ".typing",
            "LoggerLike": ".typing",
            "Origin": ".typing",
            "StatusLike": "typing",
            "Subprotocol": ".typing",
        },
        deprecated_aliases={
            "framing": ".legacy",
            "handshake": ".legacy",
            "parse_uri": ".uri",
            "WebSocketURI": ".uri",
        },
    )

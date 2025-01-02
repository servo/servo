import contextlib
import ssl

from websockets.sync.client import *
from websockets.sync.server import WebSocketServer

from ..utils import CERTIFICATE


__all__ = [
    "CLIENT_CONTEXT",
    "run_client",
    "run_unix_client",
]


CLIENT_CONTEXT = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
CLIENT_CONTEXT.load_verify_locations(CERTIFICATE)


@contextlib.contextmanager
def run_client(wsuri_or_server, secure=None, resource_name="/", **kwargs):
    if isinstance(wsuri_or_server, str):
        wsuri = wsuri_or_server
    else:
        assert isinstance(wsuri_or_server, WebSocketServer)
        if secure is None:
            secure = "ssl_context" in kwargs
        protocol = "wss" if secure else "ws"
        host, port = wsuri_or_server.socket.getsockname()
        wsuri = f"{protocol}://{host}:{port}{resource_name}"
    with connect(wsuri, **kwargs) as client:
        yield client


@contextlib.contextmanager
def run_unix_client(path, **kwargs):
    with unix_connect(path, **kwargs) as client:
        yield client

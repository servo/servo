import asyncio
import ipaddress
import socket
import sys
from typing import AsyncGenerator, Callable, Optional, cast

from ..quic.configuration import QuicConfiguration
from ..quic.connection import QuicConnection
from ..tls import SessionTicketHandler
from .compat import asynccontextmanager
from .protocol import QuicConnectionProtocol, QuicStreamHandler

__all__ = ["connect"]


@asynccontextmanager
async def connect(
    host: str,
    port: int,
    *,
    configuration: Optional[QuicConfiguration] = None,
    create_protocol: Optional[Callable] = QuicConnectionProtocol,
    session_ticket_handler: Optional[SessionTicketHandler] = None,
    stream_handler: Optional[QuicStreamHandler] = None,
    wait_connected: bool = True,
    local_port: int = 0,
) -> AsyncGenerator[QuicConnectionProtocol, None]:
    """
    Connect to a QUIC server at the given `host` and `port`.

    :meth:`connect()` returns an awaitable. Awaiting it yields a
    :class:`~aioquic.asyncio.QuicConnectionProtocol` which can be used to
    create streams.

    :func:`connect` also accepts the following optional arguments:

    * ``configuration`` is a :class:`~aioquic.quic.configuration.QuicConfiguration`
      configuration object.
    * ``create_protocol`` allows customizing the :class:`~asyncio.Protocol` that
      manages the connection. It should be a callable or class accepting the same
      arguments as :class:`~aioquic.asyncio.QuicConnectionProtocol` and returning
      an instance of :class:`~aioquic.asyncio.QuicConnectionProtocol` or a subclass.
    * ``session_ticket_handler`` is a callback which is invoked by the TLS
      engine when a new session ticket is received.
    * ``stream_handler`` is a callback which is invoked whenever a stream is
      created. It must accept two arguments: a :class:`asyncio.StreamReader`
      and a :class:`asyncio.StreamWriter`.
    * ``local_port`` is the UDP port number that this client wants to bind.
    """
    loop = asyncio.get_event_loop()
    local_host = "::"

    # if host is not an IP address, pass it to enable SNI
    try:
        ipaddress.ip_address(host)
        server_name = None
    except ValueError:
        server_name = host

    # lookup remote address
    infos = await loop.getaddrinfo(host, port, type=socket.SOCK_DGRAM)
    addr = infos[0][4]
    if len(addr) == 2:
        # determine behaviour for IPv4
        if sys.platform == "win32":
            # on Windows, we must use an IPv4 socket to reach an IPv4 host
            local_host = "0.0.0.0"
        else:
            # other platforms support dual-stack sockets
            addr = ("::ffff:" + addr[0], addr[1], 0, 0)

    # prepare QUIC connection
    if configuration is None:
        configuration = QuicConfiguration(is_client=True)
    if server_name is not None:
        configuration.server_name = server_name
    connection = QuicConnection(
        configuration=configuration, session_ticket_handler=session_ticket_handler
    )

    # connect
    _, protocol = await loop.create_datagram_endpoint(
        lambda: create_protocol(connection, stream_handler=stream_handler),
        local_addr=(local_host, local_port),
    )
    protocol = cast(QuicConnectionProtocol, protocol)
    protocol.connect(addr)
    if wait_connected:
        await protocol.wait_connected()
    try:
        yield protocol
    finally:
        protocol.close()
    await protocol.wait_closed()

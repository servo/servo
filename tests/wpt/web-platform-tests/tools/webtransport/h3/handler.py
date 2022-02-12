from typing import List, Optional, Tuple

from .webtransport_h3_server import WebTransportSession

# This file exists for documentation purpose.


def connect_received(request_headers: List[Tuple[bytes, bytes]],
                     response_headers: List[Tuple[bytes, bytes]]) -> None:
    """
    Called whenever an extended CONNECT method is received.

    :param request_headers: The request headers received from the peer.
    :param response_headers: The response headers which will be sent to the peer. ``:status`` is set
                             to 200 when it isn't specified.
    """
    pass


def session_established(session: WebTransportSession) -> None:
    """
    Called whenever an WebTransport session is established.

    :param session: A WebTransport session object.
    """


def stream_data_received(session: WebTransportSession, stream_id: int,
                         data: bytes, stream_ended: bool) -> None:
    """
    Called whenever data is received on a WebTransport stream.

    :param session: A WebTransport session object.
    :param stream_id: The ID of the stream.
    :param data: The received data.
    :param stream_ended: Whether the stream is ended.
    """
    pass


def datagram_received(session: WebTransportSession, data: bytes) -> None:
    """
    Called whenever a datagram is received on a WebTransport session.

    :param session: A WebTransport session object.
    :param data: The received data.
    """
    pass


def session_closed(session: WebTransportSession,
                   close_info: Optional[Tuple[int, bytes]],
                   abruptly: bool) -> None:
    """
    Called when a WebTransport session is closed.

    :param session: A WebTransport session.
    :param close_info: The code and reason attached to the
                       CLOSE_WEBTRANSPORT_SESSION capsule.
    :param abruptly: True when the session is closed forcibly
                     (by a CLOSE_CONNECTION QUIC frame for example).
    """
    pass


def stream_reset(session: WebTransportSession,
                 stream_id: int,
                 error_code: int) -> None:
    """
    Called when a stream is reset with RESET_STREAM.

    :param session: A WebTransport session.
    :param stream_id: The ID of the stream.
    :param error_code: The reason of the reset.
    """
    pass

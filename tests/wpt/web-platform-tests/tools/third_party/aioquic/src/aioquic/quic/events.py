from dataclasses import dataclass
from typing import Optional


class QuicEvent:
    """
    Base class for QUIC events.
    """

    pass


@dataclass
class ConnectionIdIssued(QuicEvent):
    connection_id: bytes


@dataclass
class ConnectionIdRetired(QuicEvent):
    connection_id: bytes


@dataclass
class ConnectionTerminated(QuicEvent):
    """
    The ConnectionTerminated event is fired when the QUIC connection is terminated.
    """

    error_code: int
    "The error code which was specified when closing the connection."

    frame_type: Optional[int]
    "The frame type which caused the connection to be closed, or `None`."

    reason_phrase: str
    "The human-readable reason for which the connection was closed."


@dataclass
class DatagramFrameReceived(QuicEvent):
    """
    The DatagramFrameReceived event is fired when a DATAGRAM frame is received.
    """

    data: bytes
    "The data which was received."


@dataclass
class HandshakeCompleted(QuicEvent):
    """
    The HandshakeCompleted event is fired when the TLS handshake completes.
    """

    alpn_protocol: Optional[str]
    "The protocol which was negotiated using ALPN, or `None`."

    early_data_accepted: bool
    "Whether early (0-RTT) data was accepted by the remote peer."

    session_resumed: bool
    "Whether a TLS session was resumed."


@dataclass
class PingAcknowledged(QuicEvent):
    """
    The PingAcknowledged event is fired when a PING frame is acknowledged.
    """

    uid: int
    "The unique ID of the PING."


@dataclass
class ProtocolNegotiated(QuicEvent):
    """
    The ProtocolNegotiated event is fired when ALPN negotiation completes.
    """

    alpn_protocol: Optional[str]
    "The protocol which was negotiated using ALPN, or `None`."


@dataclass
class StreamDataReceived(QuicEvent):
    """
    The StreamDataReceived event is fired whenever data is received on a
    stream.
    """

    data: bytes
    "The data which was received."

    end_stream: bool
    "Whether the STREAM frame had the FIN bit set."

    stream_id: int
    "The ID of the stream the data was received for."


@dataclass
class StreamReset(QuicEvent):
    """
    The StreamReset event is fired when the remote peer resets a stream.
    """

    error_code: int
    "The error code that triggered the reset."

    stream_id: int
    "The ID of the stream that was reset."

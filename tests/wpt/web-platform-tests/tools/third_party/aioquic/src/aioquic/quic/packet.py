import binascii
import ipaddress
import os
from dataclasses import dataclass
from enum import IntEnum
from typing import List, Optional, Tuple

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

from ..buffer import Buffer
from ..tls import pull_block, push_block
from .rangeset import RangeSet

PACKET_LONG_HEADER = 0x80
PACKET_FIXED_BIT = 0x40
PACKET_SPIN_BIT = 0x20

PACKET_TYPE_INITIAL = PACKET_LONG_HEADER | PACKET_FIXED_BIT | 0x00
PACKET_TYPE_ZERO_RTT = PACKET_LONG_HEADER | PACKET_FIXED_BIT | 0x10
PACKET_TYPE_HANDSHAKE = PACKET_LONG_HEADER | PACKET_FIXED_BIT | 0x20
PACKET_TYPE_RETRY = PACKET_LONG_HEADER | PACKET_FIXED_BIT | 0x30
PACKET_TYPE_ONE_RTT = PACKET_FIXED_BIT
PACKET_TYPE_MASK = 0xF0

CONNECTION_ID_MAX_SIZE = 20
PACKET_NUMBER_MAX_SIZE = 4
RETRY_AEAD_KEY = binascii.unhexlify("4d32ecdb2a2133c841e4043df27d4430")
RETRY_AEAD_NONCE = binascii.unhexlify("4d1611d05513a552c587d575")
RETRY_INTEGRITY_TAG_SIZE = 16


class QuicErrorCode(IntEnum):
    NO_ERROR = 0x0
    INTERNAL_ERROR = 0x1
    SERVER_BUSY = 0x2
    FLOW_CONTROL_ERROR = 0x3
    STREAM_LIMIT_ERROR = 0x4
    STREAM_STATE_ERROR = 0x5
    FINAL_SIZE_ERROR = 0x6
    FRAME_ENCODING_ERROR = 0x7
    TRANSPORT_PARAMETER_ERROR = 0x8
    CONNECTION_ID_LIMIT_ERROR = 0x9
    PROTOCOL_VIOLATION = 0xA
    INVALID_TOKEN = 0xB
    CRYPTO_BUFFER_EXCEEDED = 0xD
    CRYPTO_ERROR = 0x100


class QuicProtocolVersion(IntEnum):
    NEGOTIATION = 0
    DRAFT_25 = 0xFF000019
    DRAFT_26 = 0xFF00001A
    DRAFT_27 = 0xFF00001B


@dataclass
class QuicHeader:
    is_long_header: bool
    version: Optional[int]
    packet_type: int
    destination_cid: bytes
    source_cid: bytes
    token: bytes = b""
    integrity_tag: bytes = b""
    rest_length: int = 0


def decode_packet_number(truncated: int, num_bits: int, expected: int) -> int:
    """
    Recover a packet number from a truncated packet number.

    See: Appendix A - Sample Packet Number Decoding Algorithm
    """
    window = 1 << num_bits
    half_window = window // 2
    candidate = (expected & ~(window - 1)) | truncated
    if candidate <= expected - half_window and candidate < (1 << 62) - window:
        return candidate + window
    elif candidate > expected + half_window and candidate >= window:
        return candidate - window
    else:
        return candidate


def get_retry_integrity_tag(
    packet_without_tag: bytes, original_destination_cid: bytes
) -> bytes:
    """
    Calculate the integrity tag for a RETRY packet.
    """
    # build Retry pseudo packet
    buf = Buffer(capacity=1 + len(original_destination_cid) + len(packet_without_tag))
    buf.push_uint8(len(original_destination_cid))
    buf.push_bytes(original_destination_cid)
    buf.push_bytes(packet_without_tag)
    assert buf.eof()

    # run AES-128-GCM
    aead = AESGCM(RETRY_AEAD_KEY)
    integrity_tag = aead.encrypt(RETRY_AEAD_NONCE, b"", buf.data)
    assert len(integrity_tag) == RETRY_INTEGRITY_TAG_SIZE
    return integrity_tag


def get_spin_bit(first_byte: int) -> bool:
    return bool(first_byte & PACKET_SPIN_BIT)


def is_long_header(first_byte: int) -> bool:
    return bool(first_byte & PACKET_LONG_HEADER)


def pull_quic_header(buf: Buffer, host_cid_length: Optional[int] = None) -> QuicHeader:
    first_byte = buf.pull_uint8()

    integrity_tag = b""
    token = b""
    if is_long_header(first_byte):
        # long header packet
        version = buf.pull_uint32()

        destination_cid_length = buf.pull_uint8()
        if destination_cid_length > CONNECTION_ID_MAX_SIZE:
            raise ValueError(
                "Destination CID is too long (%d bytes)" % destination_cid_length
            )
        destination_cid = buf.pull_bytes(destination_cid_length)

        source_cid_length = buf.pull_uint8()
        if source_cid_length > CONNECTION_ID_MAX_SIZE:
            raise ValueError("Source CID is too long (%d bytes)" % source_cid_length)
        source_cid = buf.pull_bytes(source_cid_length)

        if version == QuicProtocolVersion.NEGOTIATION:
            # version negotiation
            packet_type = None
            rest_length = buf.capacity - buf.tell()
        else:
            if not (first_byte & PACKET_FIXED_BIT):
                raise ValueError("Packet fixed bit is zero")

            packet_type = first_byte & PACKET_TYPE_MASK
            if packet_type == PACKET_TYPE_INITIAL:
                token_length = buf.pull_uint_var()
                token = buf.pull_bytes(token_length)
                rest_length = buf.pull_uint_var()
            elif packet_type == PACKET_TYPE_RETRY:
                token_length = buf.capacity - buf.tell() - RETRY_INTEGRITY_TAG_SIZE
                token = buf.pull_bytes(token_length)
                integrity_tag = buf.pull_bytes(RETRY_INTEGRITY_TAG_SIZE)
                rest_length = 0
            else:
                rest_length = buf.pull_uint_var()

        return QuicHeader(
            is_long_header=True,
            version=version,
            packet_type=packet_type,
            destination_cid=destination_cid,
            source_cid=source_cid,
            token=token,
            integrity_tag=integrity_tag,
            rest_length=rest_length,
        )
    else:
        # short header packet
        if not (first_byte & PACKET_FIXED_BIT):
            raise ValueError("Packet fixed bit is zero")

        packet_type = first_byte & PACKET_TYPE_MASK
        destination_cid = buf.pull_bytes(host_cid_length)
        return QuicHeader(
            is_long_header=False,
            version=None,
            packet_type=packet_type,
            destination_cid=destination_cid,
            source_cid=b"",
            token=b"",
            rest_length=buf.capacity - buf.tell(),
        )


def encode_quic_retry(
    version: int,
    source_cid: bytes,
    destination_cid: bytes,
    original_destination_cid: bytes,
    retry_token: bytes,
) -> bytes:
    buf = Buffer(
        capacity=7
        + len(destination_cid)
        + len(source_cid)
        + len(retry_token)
        + RETRY_INTEGRITY_TAG_SIZE
    )
    buf.push_uint8(PACKET_TYPE_RETRY)
    buf.push_uint32(version)
    buf.push_uint8(len(destination_cid))
    buf.push_bytes(destination_cid)
    buf.push_uint8(len(source_cid))
    buf.push_bytes(source_cid)
    buf.push_bytes(retry_token)
    buf.push_bytes(get_retry_integrity_tag(buf.data, original_destination_cid))
    assert buf.eof()
    return buf.data


def encode_quic_version_negotiation(
    source_cid: bytes, destination_cid: bytes, supported_versions: List[int]
) -> bytes:
    buf = Buffer(
        capacity=7
        + len(destination_cid)
        + len(source_cid)
        + 4 * len(supported_versions)
    )
    buf.push_uint8(os.urandom(1)[0] | PACKET_LONG_HEADER)
    buf.push_uint32(QuicProtocolVersion.NEGOTIATION)
    buf.push_uint8(len(destination_cid))
    buf.push_bytes(destination_cid)
    buf.push_uint8(len(source_cid))
    buf.push_bytes(source_cid)
    for version in supported_versions:
        buf.push_uint32(version)
    return buf.data


# TLS EXTENSION


@dataclass
class QuicPreferredAddress:
    ipv4_address: Optional[Tuple[str, int]]
    ipv6_address: Optional[Tuple[str, int]]
    connection_id: bytes
    stateless_reset_token: bytes


@dataclass
class QuicTransportParameters:
    original_connection_id: Optional[bytes] = None
    idle_timeout: Optional[int] = None
    stateless_reset_token: Optional[bytes] = None
    max_packet_size: Optional[int] = None
    initial_max_data: Optional[int] = None
    initial_max_stream_data_bidi_local: Optional[int] = None
    initial_max_stream_data_bidi_remote: Optional[int] = None
    initial_max_stream_data_uni: Optional[int] = None
    initial_max_streams_bidi: Optional[int] = None
    initial_max_streams_uni: Optional[int] = None
    ack_delay_exponent: Optional[int] = None
    max_ack_delay: Optional[int] = None
    disable_active_migration: Optional[bool] = False
    preferred_address: Optional[QuicPreferredAddress] = None
    active_connection_id_limit: Optional[int] = None
    max_datagram_frame_size: Optional[int] = None
    quantum_readiness: Optional[bytes] = None


PARAMS = {
    0: ("original_connection_id", bytes),
    1: ("idle_timeout", int),
    2: ("stateless_reset_token", bytes),
    3: ("max_packet_size", int),
    4: ("initial_max_data", int),
    5: ("initial_max_stream_data_bidi_local", int),
    6: ("initial_max_stream_data_bidi_remote", int),
    7: ("initial_max_stream_data_uni", int),
    8: ("initial_max_streams_bidi", int),
    9: ("initial_max_streams_uni", int),
    10: ("ack_delay_exponent", int),
    11: ("max_ack_delay", int),
    12: ("disable_active_migration", bool),
    13: ("preferred_address", QuicPreferredAddress),
    14: ("active_connection_id_limit", int),
    32: ("max_datagram_frame_size", int),
    3127: ("quantum_readiness", bytes),
}


def pull_quic_preferred_address(buf: Buffer) -> QuicPreferredAddress:
    ipv4_address = None
    ipv4_host = buf.pull_bytes(4)
    ipv4_port = buf.pull_uint16()
    if ipv4_host != bytes(4):
        ipv4_address = (str(ipaddress.IPv4Address(ipv4_host)), ipv4_port)

    ipv6_address = None
    ipv6_host = buf.pull_bytes(16)
    ipv6_port = buf.pull_uint16()
    if ipv6_host != bytes(16):
        ipv6_address = (str(ipaddress.IPv6Address(ipv6_host)), ipv6_port)

    connection_id_length = buf.pull_uint8()
    connection_id = buf.pull_bytes(connection_id_length)
    stateless_reset_token = buf.pull_bytes(16)

    return QuicPreferredAddress(
        ipv4_address=ipv4_address,
        ipv6_address=ipv6_address,
        connection_id=connection_id,
        stateless_reset_token=stateless_reset_token,
    )


def push_quic_preferred_address(
    buf: Buffer, preferred_address: QuicPreferredAddress
) -> None:
    if preferred_address.ipv4_address is not None:
        buf.push_bytes(ipaddress.IPv4Address(preferred_address.ipv4_address[0]).packed)
        buf.push_uint16(preferred_address.ipv4_address[1])
    else:
        buf.push_bytes(bytes(6))

    if preferred_address.ipv6_address is not None:
        buf.push_bytes(ipaddress.IPv6Address(preferred_address.ipv6_address[0]).packed)
        buf.push_uint16(preferred_address.ipv6_address[1])
    else:
        buf.push_bytes(bytes(18))

    buf.push_uint8(len(preferred_address.connection_id))
    buf.push_bytes(preferred_address.connection_id)
    buf.push_bytes(preferred_address.stateless_reset_token)


def pull_quic_transport_parameters(
    buf: Buffer, protocol_version: int
) -> QuicTransportParameters:
    params = QuicTransportParameters()

    if protocol_version < QuicProtocolVersion.DRAFT_27:
        with pull_block(buf, 2) as length:
            end = buf.tell() + length
            while buf.tell() < end:
                param_id = buf.pull_uint16()
                param_len = buf.pull_uint16()
                param_start = buf.tell()
                if param_id in PARAMS:
                    # parse known parameter
                    param_name, param_type = PARAMS[param_id]
                    if param_type == int:
                        setattr(params, param_name, buf.pull_uint_var())
                    elif param_type == bytes:
                        setattr(params, param_name, buf.pull_bytes(param_len))
                    elif param_type == QuicPreferredAddress:
                        setattr(params, param_name, pull_quic_preferred_address(buf))
                    else:
                        setattr(params, param_name, True)
                else:
                    # skip unknown parameter
                    buf.pull_bytes(param_len)
                assert buf.tell() == param_start + param_len
    else:
        while not buf.eof():
            param_id = buf.pull_uint_var()
            param_len = buf.pull_uint_var()
            param_start = buf.tell()
            if param_id in PARAMS:
                # parse known parameter
                param_name, param_type = PARAMS[param_id]
                if param_type == int:
                    setattr(params, param_name, buf.pull_uint_var())
                elif param_type == bytes:
                    setattr(params, param_name, buf.pull_bytes(param_len))
                elif param_type == QuicPreferredAddress:
                    setattr(params, param_name, pull_quic_preferred_address(buf))
                else:
                    setattr(params, param_name, True)
            else:
                # skip unknown parameter
                buf.pull_bytes(param_len)
            assert buf.tell() == param_start + param_len

    return params


def push_quic_transport_parameters(
    buf: Buffer, params: QuicTransportParameters, protocol_version: int
) -> None:
    if protocol_version < QuicProtocolVersion.DRAFT_27:
        with push_block(buf, 2):
            for param_id, (param_name, param_type) in PARAMS.items():
                param_value = getattr(params, param_name)
                if param_value is not None and param_value is not False:
                    buf.push_uint16(param_id)
                    with push_block(buf, 2):
                        if param_type == int:
                            buf.push_uint_var(param_value)
                        elif param_type == bytes:
                            buf.push_bytes(param_value)
                        elif param_type == QuicPreferredAddress:
                            push_quic_preferred_address(buf, param_value)
    else:
        for param_id, (param_name, param_type) in PARAMS.items():
            param_value = getattr(params, param_name)
            if param_value is not None and param_value is not False:
                param_buf = Buffer(capacity=65536)
                if param_type == int:
                    param_buf.push_uint_var(param_value)
                elif param_type == bytes:
                    param_buf.push_bytes(param_value)
                elif param_type == QuicPreferredAddress:
                    push_quic_preferred_address(param_buf, param_value)
                buf.push_uint_var(param_id)
                buf.push_uint_var(param_buf.tell())
                buf.push_bytes(param_buf.data)


# FRAMES


class QuicFrameType(IntEnum):
    PADDING = 0x00
    PING = 0x01
    ACK = 0x02
    ACK_ECN = 0x03
    RESET_STREAM = 0x04
    STOP_SENDING = 0x05
    CRYPTO = 0x06
    NEW_TOKEN = 0x07
    STREAM_BASE = 0x08
    MAX_DATA = 0x10
    MAX_STREAM_DATA = 0x11
    MAX_STREAMS_BIDI = 0x12
    MAX_STREAMS_UNI = 0x13
    DATA_BLOCKED = 0x14
    STREAM_DATA_BLOCKED = 0x15
    STREAMS_BLOCKED_BIDI = 0x16
    STREAMS_BLOCKED_UNI = 0x17
    NEW_CONNECTION_ID = 0x18
    RETIRE_CONNECTION_ID = 0x19
    PATH_CHALLENGE = 0x1A
    PATH_RESPONSE = 0x1B
    TRANSPORT_CLOSE = 0x1C
    APPLICATION_CLOSE = 0x1D
    HANDSHAKE_DONE = 0x1E
    DATAGRAM = 0x30
    DATAGRAM_WITH_LENGTH = 0x31


NON_ACK_ELICITING_FRAME_TYPES = frozenset(
    [
        QuicFrameType.ACK,
        QuicFrameType.ACK_ECN,
        QuicFrameType.PADDING,
        QuicFrameType.TRANSPORT_CLOSE,
        QuicFrameType.APPLICATION_CLOSE,
    ]
)
NON_IN_FLIGHT_FRAME_TYPES = frozenset(
    [
        QuicFrameType.ACK,
        QuicFrameType.ACK_ECN,
        QuicFrameType.TRANSPORT_CLOSE,
        QuicFrameType.APPLICATION_CLOSE,
    ]
)

PROBING_FRAME_TYPES = frozenset(
    [
        QuicFrameType.PATH_CHALLENGE,
        QuicFrameType.PATH_RESPONSE,
        QuicFrameType.PADDING,
        QuicFrameType.NEW_CONNECTION_ID,
    ]
)


@dataclass
class QuicStreamFrame:
    data: bytes = b""
    fin: bool = False
    offset: int = 0


def pull_ack_frame(buf: Buffer) -> Tuple[RangeSet, int]:
    rangeset = RangeSet()
    end = buf.pull_uint_var()  # largest acknowledged
    delay = buf.pull_uint_var()
    ack_range_count = buf.pull_uint_var()
    ack_count = buf.pull_uint_var()  # first ack range
    rangeset.add(end - ack_count, end + 1)
    end -= ack_count
    for _ in range(ack_range_count):
        end -= buf.pull_uint_var() + 2
        ack_count = buf.pull_uint_var()
        rangeset.add(end - ack_count, end + 1)
        end -= ack_count
    return rangeset, delay


def push_ack_frame(buf: Buffer, rangeset: RangeSet, delay: int) -> int:
    ranges = len(rangeset)
    index = ranges - 1
    r = rangeset[index]
    buf.push_uint_var(r.stop - 1)
    buf.push_uint_var(delay)
    buf.push_uint_var(index)
    buf.push_uint_var(r.stop - 1 - r.start)
    start = r.start
    while index > 0:
        index -= 1
        r = rangeset[index]
        buf.push_uint_var(start - r.stop - 1)
        buf.push_uint_var(r.stop - r.start - 1)
        start = r.start
    return ranges

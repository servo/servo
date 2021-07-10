import logging
from enum import Enum, IntEnum
from typing import Dict, List, Optional, Set

import pylsqpack

from aioquic.buffer import Buffer, BufferReadError, encode_uint_var
from aioquic.h3.events import (
    DataReceived,
    H3Event,
    Headers,
    HeadersReceived,
    PushPromiseReceived,
)
from aioquic.h3.exceptions import NoAvailablePushIDError
from aioquic.quic.connection import QuicConnection, stream_is_unidirectional
from aioquic.quic.events import QuicEvent, StreamDataReceived
from aioquic.quic.logger import QuicLoggerTrace

logger = logging.getLogger("http3")

H3_ALPN = ["h3-27", "h3-26", "h3-25"]


class ErrorCode(IntEnum):
    HTTP_NO_ERROR = 0x100
    HTTP_GENERAL_PROTOCOL_ERROR = 0x101
    HTTP_INTERNAL_ERROR = 0x102
    HTTP_STREAM_CREATION_ERROR = 0x103
    HTTP_CLOSED_CRITICAL_STREAM = 0x104
    HTTP_FRAME_UNEXPECTED = 0x105
    HTTP_FRAME_ERROR = 0x106
    HTTP_EXCESSIVE_LOAD = 0x107
    HTTP_ID_ERROR = 0x108
    HTTP_SETTINGS_ERROR = 0x109
    HTTP_MISSING_SETTINGS = 0x10A
    HTTP_REQUEST_REJECTED = 0x10B
    HTTP_REQUEST_CANCELLED = 0x10C
    HTTP_REQUEST_INCOMPLETE = 0x10D
    HTTP_EARLY_RESPONSE = 0x10E
    HTTP_CONNECT_ERROR = 0x10F
    HTTP_VERSION_FALLBACK = 0x110
    HTTP_QPACK_DECOMPRESSION_FAILED = 0x200
    HTTP_QPACK_ENCODER_STREAM_ERROR = 0x201
    HTTP_QPACK_DECODER_STREAM_ERROR = 0x202


class FrameType(IntEnum):
    DATA = 0x0
    HEADERS = 0x1
    PRIORITY = 0x2
    CANCEL_PUSH = 0x3
    SETTINGS = 0x4
    PUSH_PROMISE = 0x5
    GOAWAY = 0x7
    MAX_PUSH_ID = 0xD
    DUPLICATE_PUSH = 0xE


class HeadersState(Enum):
    INITIAL = 0
    AFTER_HEADERS = 1
    AFTER_TRAILERS = 2


class Setting(IntEnum):
    QPACK_MAX_TABLE_CAPACITY = 1
    SETTINGS_MAX_HEADER_LIST_SIZE = 6
    QPACK_BLOCKED_STREAMS = 7
    SETTINGS_NUM_PLACEHOLDERS = 9


class StreamType(IntEnum):
    CONTROL = 0
    PUSH = 1
    QPACK_ENCODER = 2
    QPACK_DECODER = 3


class ProtocolError(Exception):
    """
    Base class for protocol errors.

    These errors are not exposed to the API user, they are handled
    in :meth:`H3Connection.handle_event`.
    """

    error_code = ErrorCode.HTTP_GENERAL_PROTOCOL_ERROR

    def __init__(self, reason_phrase: str = ""):
        self.reason_phrase = reason_phrase


class QpackDecompressionFailed(ProtocolError):
    error_code = ErrorCode.HTTP_QPACK_DECOMPRESSION_FAILED


class QpackDecoderStreamError(ProtocolError):
    error_code = ErrorCode.HTTP_QPACK_DECODER_STREAM_ERROR


class QpackEncoderStreamError(ProtocolError):
    error_code = ErrorCode.HTTP_QPACK_ENCODER_STREAM_ERROR


class StreamCreationError(ProtocolError):
    error_code = ErrorCode.HTTP_STREAM_CREATION_ERROR


class FrameUnexpected(ProtocolError):
    error_code = ErrorCode.HTTP_FRAME_UNEXPECTED


def encode_frame(frame_type: int, frame_data: bytes) -> bytes:
    frame_length = len(frame_data)
    buf = Buffer(capacity=frame_length + 16)
    buf.push_uint_var(frame_type)
    buf.push_uint_var(frame_length)
    buf.push_bytes(frame_data)
    return buf.data


def encode_settings(settings: Dict[int, int]) -> bytes:
    buf = Buffer(capacity=1024)
    for setting, value in settings.items():
        buf.push_uint_var(setting)
        buf.push_uint_var(value)
    return buf.data


def parse_max_push_id(data: bytes) -> int:
    buf = Buffer(data=data)
    max_push_id = buf.pull_uint_var()
    assert buf.eof()
    return max_push_id


def parse_settings(data: bytes) -> Dict[int, int]:
    buf = Buffer(data=data)
    settings = []
    while not buf.eof():
        setting = buf.pull_uint_var()
        value = buf.pull_uint_var()
        settings.append((setting, value))
    return dict(settings)


def qlog_encode_data_frame(byte_length: int, stream_id: int) -> Dict:
    return {
        "byte_length": str(byte_length),
        "frame": {"frame_type": "data"},
        "stream_id": str(stream_id),
    }


def qlog_encode_headers(headers: Headers) -> List[Dict]:
    return [
        {"name": h[0].decode("utf8"), "value": h[1].decode("utf8")} for h in headers
    ]


def qlog_encode_headers_frame(
    byte_length: int, headers: Headers, stream_id: int
) -> Dict:
    return {
        "byte_length": str(byte_length),
        "frame": {"frame_type": "headers", "headers": qlog_encode_headers(headers)},
        "stream_id": str(stream_id),
    }


def qlog_encode_push_promise_frame(
    byte_length: int, headers: Headers, push_id: int, stream_id: int
) -> Dict:
    return {
        "byte_length": str(byte_length),
        "frame": {
            "frame_type": "push_promise",
            "headers": qlog_encode_headers(headers),
            "push_id": str(push_id),
        },
        "stream_id": str(stream_id),
    }


class H3Stream:
    def __init__(self, stream_id: int) -> None:
        self.blocked = False
        self.blocked_frame_size: Optional[int] = None
        self.buffer = b""
        self.ended = False
        self.frame_size: Optional[int] = None
        self.frame_type: Optional[int] = None
        self.headers_recv_state: HeadersState = HeadersState.INITIAL
        self.headers_send_state: HeadersState = HeadersState.INITIAL
        self.push_id: Optional[int] = None
        self.stream_id = stream_id
        self.stream_type: Optional[int] = None


class H3Connection:
    """
    A low-level HTTP/3 connection object.

    :param quic: A :class:`~aioquic.connection.QuicConnection` instance.
    """

    def __init__(self, quic: QuicConnection):
        self._max_table_capacity = 4096
        self._blocked_streams = 16

        self._is_client = quic.configuration.is_client
        self._is_done = False
        self._quic = quic
        self._quic_logger: Optional[QuicLoggerTrace] = quic._quic_logger
        self._decoder = pylsqpack.Decoder(
            self._max_table_capacity, self._blocked_streams
        )
        self._decoder_bytes_received = 0
        self._decoder_bytes_sent = 0
        self._encoder = pylsqpack.Encoder()
        self._encoder_bytes_received = 0
        self._encoder_bytes_sent = 0
        self._stream: Dict[int, H3Stream] = {}

        self._max_push_id: Optional[int] = 8 if self._is_client else None
        self._next_push_id: int = 0

        self._local_control_stream_id: Optional[int] = None
        self._local_decoder_stream_id: Optional[int] = None
        self._local_encoder_stream_id: Optional[int] = None

        self._peer_control_stream_id: Optional[int] = None
        self._peer_decoder_stream_id: Optional[int] = None
        self._peer_encoder_stream_id: Optional[int] = None

        self._init_connection()

    def handle_event(self, event: QuicEvent) -> List[H3Event]:
        """
        Handle a QUIC event and return a list of HTTP events.

        :param event: The QUIC event to handle.
        """
        if isinstance(event, StreamDataReceived) and not self._is_done:
            stream_id = event.stream_id
            stream = self._get_or_create_stream(stream_id)
            try:
                if stream_id % 4 == 0:
                    return self._receive_request_or_push_data(
                        stream, event.data, event.end_stream
                    )
                elif stream_is_unidirectional(stream_id):
                    return self._receive_stream_data_uni(
                        stream, event.data, event.end_stream
                    )
            except ProtocolError as exc:
                self._is_done = True
                self._quic.close(
                    error_code=exc.error_code, reason_phrase=exc.reason_phrase
                )
        return []

    def send_push_promise(self, stream_id: int, headers: Headers) -> int:
        """
        Send a push promise related to the specified stream.

        Returns the stream ID on which headers and data can be sent.

        :param stream_id: The stream ID on which to send the data.
        :param headers: The HTTP request headers for this push.
        """
        assert not self._is_client, "Only servers may send a push promise."
        if self._max_push_id is None or self._next_push_id >= self._max_push_id:
            raise NoAvailablePushIDError

        # send push promise
        push_id = self._next_push_id
        self._next_push_id += 1
        self._quic.send_stream_data(
            stream_id,
            encode_frame(
                FrameType.PUSH_PROMISE,
                encode_uint_var(push_id) + self._encode_headers(stream_id, headers),
            ),
        )

        #  create push stream
        push_stream_id = self._create_uni_stream(StreamType.PUSH)
        self._quic.send_stream_data(push_stream_id, encode_uint_var(push_id))

        return push_stream_id

    def send_data(self, stream_id: int, data: bytes, end_stream: bool) -> None:
        """
        Send data on the given stream.

        To retrieve datagram which need to be sent over the network call the QUIC
        connection's :meth:`~aioquic.connection.QuicConnection.datagrams_to_send`
        method.

        :param stream_id: The stream ID on which to send the data.
        :param data: The data to send.
        :param end_stream: Whether to end the stream.
        """
        # check DATA frame is allowed
        stream = self._get_or_create_stream(stream_id)
        if stream.headers_send_state != HeadersState.AFTER_HEADERS:
            raise FrameUnexpected("DATA frame is not allowed in this state")

        # log frame
        if self._quic_logger is not None:
            self._quic_logger.log_event(
                category="http",
                event="frame_created",
                data=qlog_encode_data_frame(byte_length=len(data), stream_id=stream_id),
            )

        self._quic.send_stream_data(
            stream_id, encode_frame(FrameType.DATA, data), end_stream
        )

    def send_headers(
        self, stream_id: int, headers: Headers, end_stream: bool = False
    ) -> None:
        """
        Send headers on the given stream.

        To retrieve datagram which need to be sent over the network call the QUIC
        connection's :meth:`~aioquic.connection.QuicConnection.datagrams_to_send`
        method.

        :param stream_id: The stream ID on which to send the headers.
        :param headers: The HTTP headers to send.
        :param end_stream: Whether to end the stream.
        """
        # check HEADERS frame is allowed
        stream = self._get_or_create_stream(stream_id)
        if stream.headers_send_state == HeadersState.AFTER_TRAILERS:
            raise FrameUnexpected("HEADERS frame is not allowed in this state")

        frame_data = self._encode_headers(stream_id, headers)

        # log frame
        if self._quic_logger is not None:
            self._quic_logger.log_event(
                category="http",
                event="frame_created",
                data=qlog_encode_headers_frame(
                    byte_length=len(frame_data), headers=headers, stream_id=stream_id
                ),
            )

        # update state and send headers
        if stream.headers_send_state == HeadersState.INITIAL:
            stream.headers_send_state = HeadersState.AFTER_HEADERS
        else:
            stream.headers_send_state = HeadersState.AFTER_TRAILERS
        self._quic.send_stream_data(
            stream_id, encode_frame(FrameType.HEADERS, frame_data), end_stream
        )

    def _create_uni_stream(self, stream_type: int) -> int:
        """
        Create an unidirectional stream of the given type.
        """
        stream_id = self._quic.get_next_available_stream_id(is_unidirectional=True)
        self._quic.send_stream_data(stream_id, encode_uint_var(stream_type))
        return stream_id

    def _decode_headers(self, stream_id: int, frame_data: Optional[bytes]) -> Headers:
        """
        Decode a HEADERS block and send decoder updates on the decoder stream.

        This is called with frame_data=None when a stream becomes unblocked.
        """
        try:
            if frame_data is None:
                decoder, headers = self._decoder.resume_header(stream_id)
            else:
                decoder, headers = self._decoder.feed_header(stream_id, frame_data)
            self._decoder_bytes_sent += len(decoder)
            self._quic.send_stream_data(self._local_decoder_stream_id, decoder)
        except pylsqpack.DecompressionFailed as exc:
            raise QpackDecompressionFailed() from exc

        return headers

    def _encode_headers(self, stream_id: int, headers: Headers) -> bytes:
        """
        Encode a HEADERS block and send encoder updates on the encoder stream.
        """
        encoder, frame_data = self._encoder.encode(stream_id, headers)
        self._encoder_bytes_sent += len(encoder)
        self._quic.send_stream_data(self._local_encoder_stream_id, encoder)
        return frame_data

    def _get_or_create_stream(self, stream_id: int) -> H3Stream:
        if stream_id not in self._stream:
            self._stream[stream_id] = H3Stream(stream_id)
        return self._stream[stream_id]

    def _handle_control_frame(self, frame_type: int, frame_data: bytes) -> None:
        """
        Handle a frame received on the peer's control stream.
        """
        if frame_type == FrameType.SETTINGS:
            settings = parse_settings(frame_data)
            encoder = self._encoder.apply_settings(
                max_table_capacity=settings.get(Setting.QPACK_MAX_TABLE_CAPACITY, 0),
                blocked_streams=settings.get(Setting.QPACK_BLOCKED_STREAMS, 0),
            )
            self._quic.send_stream_data(self._local_encoder_stream_id, encoder)
        elif frame_type == FrameType.MAX_PUSH_ID:
            if self._is_client:
                raise FrameUnexpected("Servers must not send MAX_PUSH_ID")
            self._max_push_id = parse_max_push_id(frame_data)
        elif frame_type in (
            FrameType.DATA,
            FrameType.HEADERS,
            FrameType.PUSH_PROMISE,
            FrameType.DUPLICATE_PUSH,
        ):
            raise FrameUnexpected("Invalid frame type on control stream")

    def _handle_request_or_push_frame(
        self,
        frame_type: int,
        frame_data: Optional[bytes],
        stream: H3Stream,
        stream_ended: bool,
    ) -> List[H3Event]:
        """
        Handle a frame received on a request or push stream.
        """
        http_events: List[H3Event] = []

        if frame_type == FrameType.DATA:
            # check DATA frame is allowed
            if stream.headers_recv_state != HeadersState.AFTER_HEADERS:
                raise FrameUnexpected("DATA frame is not allowed in this state")

            if stream_ended or frame_data:
                http_events.append(
                    DataReceived(
                        data=frame_data,
                        push_id=stream.push_id,
                        stream_ended=stream_ended,
                        stream_id=stream.stream_id,
                    )
                )
        elif frame_type == FrameType.HEADERS:
            # check HEADERS frame is allowed
            if stream.headers_recv_state == HeadersState.AFTER_TRAILERS:
                raise FrameUnexpected("HEADERS frame is not allowed in this state")

            # try to decode HEADERS, may raise pylsqpack.StreamBlocked
            headers = self._decode_headers(stream.stream_id, frame_data)

            # log frame
            if self._quic_logger is not None:
                self._quic_logger.log_event(
                    category="http",
                    event="frame_parsed",
                    data=qlog_encode_headers_frame(
                        byte_length=stream.blocked_frame_size
                        if frame_data is None
                        else len(frame_data),
                        headers=headers,
                        stream_id=stream.stream_id,
                    ),
                )

            # update state and emit headers
            if stream.headers_recv_state == HeadersState.INITIAL:
                stream.headers_recv_state = HeadersState.AFTER_HEADERS
            else:
                stream.headers_recv_state = HeadersState.AFTER_TRAILERS
            http_events.append(
                HeadersReceived(
                    headers=headers,
                    push_id=stream.push_id,
                    stream_id=stream.stream_id,
                    stream_ended=stream_ended,
                )
            )
        elif stream.frame_type == FrameType.PUSH_PROMISE and stream.push_id is None:
            if not self._is_client:
                raise FrameUnexpected("Clients must not send PUSH_PROMISE")
            frame_buf = Buffer(data=frame_data)
            push_id = frame_buf.pull_uint_var()
            headers = self._decode_headers(
                stream.stream_id, frame_data[frame_buf.tell() :]
            )

            # log frame
            if self._quic_logger is not None:
                self._quic_logger.log_event(
                    category="http",
                    event="frame_parsed",
                    data=qlog_encode_push_promise_frame(
                        byte_length=len(frame_data),
                        headers=headers,
                        push_id=push_id,
                        stream_id=stream.stream_id,
                    ),
                )

            # emit event
            http_events.append(
                PushPromiseReceived(
                    headers=headers, push_id=push_id, stream_id=stream.stream_id
                )
            )
        elif frame_type in (
            FrameType.PRIORITY,
            FrameType.CANCEL_PUSH,
            FrameType.SETTINGS,
            FrameType.PUSH_PROMISE,
            FrameType.GOAWAY,
            FrameType.MAX_PUSH_ID,
            FrameType.DUPLICATE_PUSH,
        ):
            raise FrameUnexpected(
                "Invalid frame type on request stream"
                if stream.push_id is None
                else "Invalid frame type on push stream"
            )

        return http_events

    def _init_connection(self) -> None:
        # send our settings
        self._local_control_stream_id = self._create_uni_stream(StreamType.CONTROL)
        self._quic.send_stream_data(
            self._local_control_stream_id,
            encode_frame(
                FrameType.SETTINGS,
                encode_settings(
                    {
                        Setting.QPACK_MAX_TABLE_CAPACITY: self._max_table_capacity,
                        Setting.QPACK_BLOCKED_STREAMS: self._blocked_streams,
                    }
                ),
            ),
        )
        if self._is_client and self._max_push_id is not None:
            self._quic.send_stream_data(
                self._local_control_stream_id,
                encode_frame(FrameType.MAX_PUSH_ID, encode_uint_var(self._max_push_id)),
            )

        # create encoder and decoder streams
        self._local_encoder_stream_id = self._create_uni_stream(
            StreamType.QPACK_ENCODER
        )
        self._local_decoder_stream_id = self._create_uni_stream(
            StreamType.QPACK_DECODER
        )

    def _receive_request_or_push_data(
        self, stream: H3Stream, data: bytes, stream_ended: bool
    ) -> List[H3Event]:
        """
        Handle data received on a request or push stream.
        """
        http_events: List[H3Event] = []

        stream.buffer += data
        if stream_ended:
            stream.ended = True
        if stream.blocked:
            return http_events

        # shortcut for DATA frame fragments
        if (
            stream.frame_type == FrameType.DATA
            and stream.frame_size is not None
            and len(stream.buffer) < stream.frame_size
        ):
            http_events.append(
                DataReceived(
                    data=stream.buffer,
                    push_id=stream.push_id,
                    stream_id=stream.stream_id,
                    stream_ended=False,
                )
            )
            stream.frame_size -= len(stream.buffer)
            stream.buffer = b""
            return http_events

        # handle lone FIN
        if stream_ended and not stream.buffer:
            http_events.append(
                DataReceived(
                    data=b"",
                    push_id=stream.push_id,
                    stream_id=stream.stream_id,
                    stream_ended=True,
                )
            )
            return http_events

        buf = Buffer(data=stream.buffer)
        consumed = 0

        while not buf.eof():
            # fetch next frame header
            if stream.frame_size is None:
                try:
                    stream.frame_type = buf.pull_uint_var()
                    stream.frame_size = buf.pull_uint_var()
                except BufferReadError:
                    break
                consumed = buf.tell()

                # log frame
                if (
                    self._quic_logger is not None
                    and stream.frame_type == FrameType.DATA
                ):
                    self._quic_logger.log_event(
                        category="http",
                        event="frame_parsed",
                        data=qlog_encode_data_frame(
                            byte_length=stream.frame_size, stream_id=stream.stream_id
                        ),
                    )

            # check how much data is available
            chunk_size = min(stream.frame_size, buf.capacity - consumed)
            if stream.frame_type != FrameType.DATA and chunk_size < stream.frame_size:
                break

            # read available data
            frame_data = buf.pull_bytes(chunk_size)
            consumed = buf.tell()

            # detect end of frame
            stream.frame_size -= chunk_size
            if not stream.frame_size:
                stream.frame_size = None

            try:
                http_events.extend(
                    self._handle_request_or_push_frame(
                        frame_type=stream.frame_type,
                        frame_data=frame_data,
                        stream=stream,
                        stream_ended=stream.ended and buf.eof(),
                    )
                )
            except pylsqpack.StreamBlocked:
                stream.blocked = True
                stream.blocked_frame_size = len(frame_data)
                break

        # remove processed data from buffer
        stream.buffer = stream.buffer[consumed:]

        return http_events

    def _receive_stream_data_uni(
        self, stream: H3Stream, data: bytes, stream_ended: bool
    ) -> List[H3Event]:
        http_events: List[H3Event] = []

        stream.buffer += data
        if stream_ended:
            stream.ended = True

        buf = Buffer(data=stream.buffer)
        consumed = 0
        unblocked_streams: Set[int] = set()

        while stream.stream_type == StreamType.PUSH or not buf.eof():
            # fetch stream type for unidirectional streams
            if stream.stream_type is None:
                try:
                    stream.stream_type = buf.pull_uint_var()
                except BufferReadError:
                    break
                consumed = buf.tell()

                # check unicity
                if stream.stream_type == StreamType.CONTROL:
                    if self._peer_control_stream_id is not None:
                        raise StreamCreationError("Only one control stream is allowed")
                    self._peer_control_stream_id = stream.stream_id
                elif stream.stream_type == StreamType.QPACK_DECODER:
                    if self._peer_decoder_stream_id is not None:
                        raise StreamCreationError(
                            "Only one QPACK decoder stream is allowed"
                        )
                    self._peer_decoder_stream_id = stream.stream_id
                elif stream.stream_type == StreamType.QPACK_ENCODER:
                    if self._peer_encoder_stream_id is not None:
                        raise StreamCreationError(
                            "Only one QPACK encoder stream is allowed"
                        )
                    self._peer_encoder_stream_id = stream.stream_id

            if stream.stream_type == StreamType.CONTROL:
                # fetch next frame
                try:
                    frame_type = buf.pull_uint_var()
                    frame_length = buf.pull_uint_var()
                    frame_data = buf.pull_bytes(frame_length)
                except BufferReadError:
                    break
                consumed = buf.tell()

                self._handle_control_frame(frame_type, frame_data)
            elif stream.stream_type == StreamType.PUSH:
                # fetch push id
                if stream.push_id is None:
                    try:
                        stream.push_id = buf.pull_uint_var()
                    except BufferReadError:
                        break
                    consumed = buf.tell()

                # remove processed data from buffer
                stream.buffer = stream.buffer[consumed:]

                return self._receive_request_or_push_data(stream, b"", stream_ended)
            elif stream.stream_type == StreamType.QPACK_DECODER:
                # feed unframed data to decoder
                data = buf.pull_bytes(buf.capacity - buf.tell())
                consumed = buf.tell()
                try:
                    self._encoder.feed_decoder(data)
                except pylsqpack.DecoderStreamError as exc:
                    raise QpackDecoderStreamError() from exc
                self._decoder_bytes_received += len(data)
            elif stream.stream_type == StreamType.QPACK_ENCODER:
                # feed unframed data to encoder
                data = buf.pull_bytes(buf.capacity - buf.tell())
                consumed = buf.tell()
                try:
                    unblocked_streams.update(self._decoder.feed_encoder(data))
                except pylsqpack.EncoderStreamError as exc:
                    raise QpackEncoderStreamError() from exc
                self._encoder_bytes_received += len(data)
            else:
                # unknown stream type, discard data
                buf.seek(buf.capacity)
                consumed = buf.tell()

        # remove processed data from buffer
        stream.buffer = stream.buffer[consumed:]

        # process unblocked streams
        for stream_id in unblocked_streams:
            stream = self._stream[stream_id]

            # resume headers
            http_events.extend(
                self._handle_request_or_push_frame(
                    frame_type=FrameType.HEADERS,
                    frame_data=None,
                    stream=stream,
                    stream_ended=stream.ended and not stream.buffer,
                )
            )
            stream.blocked = False
            stream.blocked_frame_size = None

            # resume processing
            if stream.buffer:
                http_events.extend(
                    self._receive_request_or_push_data(stream, b"", stream.ended)
                )

        return http_events

from __future__ import annotations

import dataclasses
import enum
import io
import secrets
import struct
from typing import Callable, Generator, Optional, Sequence, Tuple

from . import exceptions, extensions
from .typing import Data


try:
    from .speedups import apply_mask
except ImportError:
    from .utils import apply_mask


__all__ = [
    "Opcode",
    "OP_CONT",
    "OP_TEXT",
    "OP_BINARY",
    "OP_CLOSE",
    "OP_PING",
    "OP_PONG",
    "DATA_OPCODES",
    "CTRL_OPCODES",
    "Frame",
    "prepare_data",
    "prepare_ctrl",
    "Close",
]


class Opcode(enum.IntEnum):
    """Opcode values for WebSocket frames."""

    CONT, TEXT, BINARY = 0x00, 0x01, 0x02
    CLOSE, PING, PONG = 0x08, 0x09, 0x0A


OP_CONT = Opcode.CONT
OP_TEXT = Opcode.TEXT
OP_BINARY = Opcode.BINARY
OP_CLOSE = Opcode.CLOSE
OP_PING = Opcode.PING
OP_PONG = Opcode.PONG

DATA_OPCODES = OP_CONT, OP_TEXT, OP_BINARY
CTRL_OPCODES = OP_CLOSE, OP_PING, OP_PONG


class CloseCode(enum.IntEnum):
    """Close code values for WebSocket close frames."""

    NORMAL_CLOSURE = 1000
    GOING_AWAY = 1001
    PROTOCOL_ERROR = 1002
    UNSUPPORTED_DATA = 1003
    # 1004 is reserved
    NO_STATUS_RCVD = 1005
    ABNORMAL_CLOSURE = 1006
    INVALID_DATA = 1007
    POLICY_VIOLATION = 1008
    MESSAGE_TOO_BIG = 1009
    MANDATORY_EXTENSION = 1010
    INTERNAL_ERROR = 1011
    SERVICE_RESTART = 1012
    TRY_AGAIN_LATER = 1013
    BAD_GATEWAY = 1014
    TLS_HANDSHAKE = 1015


# See https://www.iana.org/assignments/websocket/websocket.xhtml
CLOSE_CODE_EXPLANATIONS: dict[int, str] = {
    CloseCode.NORMAL_CLOSURE: "OK",
    CloseCode.GOING_AWAY: "going away",
    CloseCode.PROTOCOL_ERROR: "protocol error",
    CloseCode.UNSUPPORTED_DATA: "unsupported data",
    CloseCode.NO_STATUS_RCVD: "no status received [internal]",
    CloseCode.ABNORMAL_CLOSURE: "abnormal closure [internal]",
    CloseCode.INVALID_DATA: "invalid frame payload data",
    CloseCode.POLICY_VIOLATION: "policy violation",
    CloseCode.MESSAGE_TOO_BIG: "message too big",
    CloseCode.MANDATORY_EXTENSION: "mandatory extension",
    CloseCode.INTERNAL_ERROR: "internal error",
    CloseCode.SERVICE_RESTART: "service restart",
    CloseCode.TRY_AGAIN_LATER: "try again later",
    CloseCode.BAD_GATEWAY: "bad gateway",
    CloseCode.TLS_HANDSHAKE: "TLS handshake failure [internal]",
}


# Close code that are allowed in a close frame.
# Using a set optimizes `code in EXTERNAL_CLOSE_CODES`.
EXTERNAL_CLOSE_CODES = {
    CloseCode.NORMAL_CLOSURE,
    CloseCode.GOING_AWAY,
    CloseCode.PROTOCOL_ERROR,
    CloseCode.UNSUPPORTED_DATA,
    CloseCode.INVALID_DATA,
    CloseCode.POLICY_VIOLATION,
    CloseCode.MESSAGE_TOO_BIG,
    CloseCode.MANDATORY_EXTENSION,
    CloseCode.INTERNAL_ERROR,
    CloseCode.SERVICE_RESTART,
    CloseCode.TRY_AGAIN_LATER,
    CloseCode.BAD_GATEWAY,
}


OK_CLOSE_CODES = {
    CloseCode.NORMAL_CLOSURE,
    CloseCode.GOING_AWAY,
    CloseCode.NO_STATUS_RCVD,
}


BytesLike = bytes, bytearray, memoryview


@dataclasses.dataclass
class Frame:
    """
    WebSocket frame.

    Attributes:
        opcode: Opcode.
        data: Payload data.
        fin: FIN bit.
        rsv1: RSV1 bit.
        rsv2: RSV2 bit.
        rsv3: RSV3 bit.

    Only these fields are needed. The MASK bit, payload length and masking-key
    are handled on the fly when parsing and serializing frames.

    """

    opcode: Opcode
    data: bytes
    fin: bool = True
    rsv1: bool = False
    rsv2: bool = False
    rsv3: bool = False

    def __str__(self) -> str:
        """
        Return a human-readable representation of a frame.

        """
        coding = None
        length = f"{len(self.data)} byte{'' if len(self.data) == 1 else 's'}"
        non_final = "" if self.fin else "continued"

        if self.opcode is OP_TEXT:
            # Decoding only the beginning and the end is needlessly hard.
            # Decode the entire payload then elide later if necessary.
            data = repr(self.data.decode())
        elif self.opcode is OP_BINARY:
            # We'll show at most the first 16 bytes and the last 8 bytes.
            # Encode just what we need, plus two dummy bytes to elide later.
            binary = self.data
            if len(binary) > 25:
                binary = b"".join([binary[:16], b"\x00\x00", binary[-8:]])
            data = " ".join(f"{byte:02x}" for byte in binary)
        elif self.opcode is OP_CLOSE:
            data = str(Close.parse(self.data))
        elif self.data:
            # We don't know if a Continuation frame contains text or binary.
            # Ping and Pong frames could contain UTF-8.
            # Attempt to decode as UTF-8 and display it as text; fallback to
            # binary. If self.data is a memoryview, it has no decode() method,
            # which raises AttributeError.
            try:
                data = repr(self.data.decode())
                coding = "text"
            except (UnicodeDecodeError, AttributeError):
                binary = self.data
                if len(binary) > 25:
                    binary = b"".join([binary[:16], b"\x00\x00", binary[-8:]])
                data = " ".join(f"{byte:02x}" for byte in binary)
                coding = "binary"
        else:
            data = "''"

        if len(data) > 75:
            data = data[:48] + "..." + data[-24:]

        metadata = ", ".join(filter(None, [coding, length, non_final]))

        return f"{self.opcode.name} {data} [{metadata}]"

    @classmethod
    def parse(
        cls,
        read_exact: Callable[[int], Generator[None, None, bytes]],
        *,
        mask: bool,
        max_size: Optional[int] = None,
        extensions: Optional[Sequence[extensions.Extension]] = None,
    ) -> Generator[None, None, Frame]:
        """
        Parse a WebSocket frame.

        This is a generator-based coroutine.

        Args:
            read_exact: generator-based coroutine that reads the requested
                bytes or raises an exception if there isn't enough data.
            mask: whether the frame should be masked i.e. whether the read
                happens on the server side.
            max_size: maximum payload size in bytes.
            extensions: list of extensions, applied in reverse order.

        Raises:
            EOFError: if the connection is closed without a full WebSocket frame.
            UnicodeDecodeError: if the frame contains invalid UTF-8.
            PayloadTooBig: if the frame's payload size exceeds ``max_size``.
            ProtocolError: if the frame contains incorrect values.

        """
        # Read the header.
        data = yield from read_exact(2)
        head1, head2 = struct.unpack("!BB", data)

        # While not Pythonic, this is marginally faster than calling bool().
        fin = True if head1 & 0b10000000 else False
        rsv1 = True if head1 & 0b01000000 else False
        rsv2 = True if head1 & 0b00100000 else False
        rsv3 = True if head1 & 0b00010000 else False

        try:
            opcode = Opcode(head1 & 0b00001111)
        except ValueError as exc:
            raise exceptions.ProtocolError("invalid opcode") from exc

        if (True if head2 & 0b10000000 else False) != mask:
            raise exceptions.ProtocolError("incorrect masking")

        length = head2 & 0b01111111
        if length == 126:
            data = yield from read_exact(2)
            (length,) = struct.unpack("!H", data)
        elif length == 127:
            data = yield from read_exact(8)
            (length,) = struct.unpack("!Q", data)
        if max_size is not None and length > max_size:
            raise exceptions.PayloadTooBig(
                f"over size limit ({length} > {max_size} bytes)"
            )
        if mask:
            mask_bytes = yield from read_exact(4)

        # Read the data.
        data = yield from read_exact(length)
        if mask:
            data = apply_mask(data, mask_bytes)

        frame = cls(opcode, data, fin, rsv1, rsv2, rsv3)

        if extensions is None:
            extensions = []
        for extension in reversed(extensions):
            frame = extension.decode(frame, max_size=max_size)

        frame.check()

        return frame

    def serialize(
        self,
        *,
        mask: bool,
        extensions: Optional[Sequence[extensions.Extension]] = None,
    ) -> bytes:
        """
        Serialize a WebSocket frame.

        Args:
            mask: whether the frame should be masked i.e. whether the write
                happens on the client side.
            extensions: list of extensions, applied in order.

        Raises:
            ProtocolError: if the frame contains incorrect values.

        """
        self.check()

        if extensions is None:
            extensions = []
        for extension in extensions:
            self = extension.encode(self)

        output = io.BytesIO()

        # Prepare the header.
        head1 = (
            (0b10000000 if self.fin else 0)
            | (0b01000000 if self.rsv1 else 0)
            | (0b00100000 if self.rsv2 else 0)
            | (0b00010000 if self.rsv3 else 0)
            | self.opcode
        )

        head2 = 0b10000000 if mask else 0

        length = len(self.data)
        if length < 126:
            output.write(struct.pack("!BB", head1, head2 | length))
        elif length < 65536:
            output.write(struct.pack("!BBH", head1, head2 | 126, length))
        else:
            output.write(struct.pack("!BBQ", head1, head2 | 127, length))

        if mask:
            mask_bytes = secrets.token_bytes(4)
            output.write(mask_bytes)

        # Prepare the data.
        if mask:
            data = apply_mask(self.data, mask_bytes)
        else:
            data = self.data
        output.write(data)

        return output.getvalue()

    def check(self) -> None:
        """
        Check that reserved bits and opcode have acceptable values.

        Raises:
            ProtocolError: if a reserved bit or the opcode is invalid.

        """
        if self.rsv1 or self.rsv2 or self.rsv3:
            raise exceptions.ProtocolError("reserved bits must be 0")

        if self.opcode in CTRL_OPCODES:
            if len(self.data) > 125:
                raise exceptions.ProtocolError("control frame too long")
            if not self.fin:
                raise exceptions.ProtocolError("fragmented control frame")


def prepare_data(data: Data) -> Tuple[int, bytes]:
    """
    Convert a string or byte-like object to an opcode and a bytes-like object.

    This function is designed for data frames.

    If ``data`` is a :class:`str`, return ``OP_TEXT`` and a :class:`bytes`
    object encoding ``data`` in UTF-8.

    If ``data`` is a bytes-like object, return ``OP_BINARY`` and a bytes-like
    object.

    Raises:
        TypeError: if ``data`` doesn't have a supported type.

    """
    if isinstance(data, str):
        return OP_TEXT, data.encode("utf-8")
    elif isinstance(data, BytesLike):
        return OP_BINARY, data
    else:
        raise TypeError("data must be str or bytes-like")


def prepare_ctrl(data: Data) -> bytes:
    """
    Convert a string or byte-like object to bytes.

    This function is designed for ping and pong frames.

    If ``data`` is a :class:`str`, return a :class:`bytes` object encoding
    ``data`` in UTF-8.

    If ``data`` is a bytes-like object, return a :class:`bytes` object.

    Raises:
        TypeError: if ``data`` doesn't have a supported type.

    """
    if isinstance(data, str):
        return data.encode("utf-8")
    elif isinstance(data, BytesLike):
        return bytes(data)
    else:
        raise TypeError("data must be str or bytes-like")


@dataclasses.dataclass
class Close:
    """
    Code and reason for WebSocket close frames.

    Attributes:
        code: Close code.
        reason: Close reason.

    """

    code: int
    reason: str

    def __str__(self) -> str:
        """
        Return a human-readable representation of a close code and reason.

        """
        if 3000 <= self.code < 4000:
            explanation = "registered"
        elif 4000 <= self.code < 5000:
            explanation = "private use"
        else:
            explanation = CLOSE_CODE_EXPLANATIONS.get(self.code, "unknown")
        result = f"{self.code} ({explanation})"

        if self.reason:
            result = f"{result} {self.reason}"

        return result

    @classmethod
    def parse(cls, data: bytes) -> Close:
        """
        Parse the payload of a close frame.

        Args:
            data: payload of the close frame.

        Raises:
            ProtocolError: if data is ill-formed.
            UnicodeDecodeError: if the reason isn't valid UTF-8.

        """
        if len(data) >= 2:
            (code,) = struct.unpack("!H", data[:2])
            reason = data[2:].decode("utf-8")
            close = cls(code, reason)
            close.check()
            return close
        elif len(data) == 0:
            return cls(CloseCode.NO_STATUS_RCVD, "")
        else:
            raise exceptions.ProtocolError("close frame too short")

    def serialize(self) -> bytes:
        """
        Serialize the payload of a close frame.

        """
        self.check()
        return struct.pack("!H", self.code) + self.reason.encode("utf-8")

    def check(self) -> None:
        """
        Check that the close code has a valid value for a close frame.

        Raises:
            ProtocolError: if the close code is invalid.

        """
        if not (self.code in EXTERNAL_CLOSE_CODES or 3000 <= self.code < 5000):
            raise exceptions.ProtocolError("invalid status code")

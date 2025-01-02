# mypy: no-warn-return-any

from enum import IntEnum
from typing import Iterator, Optional

# TODO(bashi): Remove import check suppressions once aioquic dependency is
# resolved.
from aioquic.buffer import UINT_VAR_MAX_SIZE, Buffer, BufferReadError  # type: ignore


class CapsuleType(IntEnum):
    # Defined in
    # https://datatracker.ietf.org/doc/html/draft-ietf-masque-h3-datagram-04#section-8.2
    DATAGRAM_DRAFT04 = 0xff37a0
    REGISTER_DATAGRAM_CONTEXT_DRAFT04 = 0xff37a1
    REGISTER_DATAGRAM_NO_CONTEXT_DRAFT04 = 0xff37a2
    CLOSE_DATAGRAM_CONTEXT_DRAFT04 = 0xff37a3
    # Defined in
    # https://datatracker.ietf.org/doc/html/rfc9297#section-5.4
    DATAGRAM_RFC = 0x00
    # Defined in
    # https://www.ietf.org/archive/id/draft-ietf-webtrans-http3-01.html.
    CLOSE_WEBTRANSPORT_SESSION = 0x2843


class H3Capsule:
    """
    Represents the Capsule concept defined in
    https://ietf-wg-masque.github.io/draft-ietf-masque-h3-datagram/draft-ietf-masque-h3-datagram.html#name-capsules.
    """

    def __init__(self, type: int, data: bytes) -> None:
        """
        :param type the type of this Capsule. We don't use CapsuleType here
                    because this may be a capsule of an unknown type.
        :param data the payload
        """
        self.type = type
        self.data = data

    def encode(self) -> bytes:
        """
        Encodes this H3Capsule and return the bytes.
        """
        buffer = Buffer(capacity=len(self.data) + 2 * UINT_VAR_MAX_SIZE)
        buffer.push_uint_var(self.type)
        buffer.push_uint_var(len(self.data))
        buffer.push_bytes(self.data)
        return buffer.data


class H3CapsuleDecoder:
    """
    A decoder of H3Capsule. This is a streaming decoder and can handle multiple
    decoders.
    """

    def __init__(self) -> None:
        self._buffer: Optional[Buffer] = None
        self._type: Optional[int] = None
        self._length: Optional[int] = None
        self._final: bool = False

    def append(self, data: bytes) -> None:
        """
        Appends the given bytes to this decoder.
        """
        assert not self._final

        if len(data) == 0:
            return
        if self._buffer:
            remaining = self._buffer.pull_bytes(
                self._buffer.capacity - self._buffer.tell())
            self._buffer = Buffer(data=(remaining + data))
        else:
            self._buffer = Buffer(data=data)

    def final(self) -> None:
        """
        Pushes the end-of-stream mark to this decoder. After calling this,
        calling append() will be invalid.
        """
        self._final = True

    def __iter__(self) -> Iterator[H3Capsule]:
        """
        Yields decoded capsules.
        """
        try:
            while self._buffer is not None:
                if self._type is None:
                    self._type = self._buffer.pull_uint_var()
                if self._length is None:
                    self._length = self._buffer.pull_uint_var()
                if self._buffer.capacity - self._buffer.tell() < self._length:
                    if self._final:
                        raise ValueError('insufficient buffer')
                    return
                capsule = H3Capsule(
                    self._type, self._buffer.pull_bytes(self._length))
                self._type = None
                self._length = None
                if self._buffer.tell() == self._buffer.capacity:
                    self._buffer = None
                yield capsule
        except BufferReadError as e:
            if self._final:
                raise e
            if not self._buffer:
                return
            size = self._buffer.capacity - self._buffer.tell()
            if size >= UINT_VAR_MAX_SIZE:
                raise e
            # Ignore the error because there may not be sufficient input.
            return

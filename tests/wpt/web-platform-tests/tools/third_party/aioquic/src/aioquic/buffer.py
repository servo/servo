from ._buffer import Buffer, BufferReadError, BufferWriteError  # noqa

UINT_VAR_MAX = 0x3FFFFFFFFFFFFFFF


def encode_uint_var(value: int) -> bytes:
    """
    Encode a variable-length unsigned integer.
    """
    buf = Buffer(capacity=8)
    buf.push_uint_var(value)
    return buf.data


def size_uint_var(value: int) -> int:
    """
    Return the number of bytes required to encode the given value
    as a QUIC variable-length unsigned integer.
    """
    if value <= 0x3F:
        return 1
    elif value <= 0x3FFF:
        return 2
    elif value <= 0x3FFFFFFF:
        return 4
    elif value <= 0x3FFFFFFFFFFFFFFF:
        return 8
    else:
        raise ValueError("Integer is too big for a variable-length integer")

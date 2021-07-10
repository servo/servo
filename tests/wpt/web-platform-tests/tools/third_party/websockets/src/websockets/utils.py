import itertools


__all__ = ["apply_mask"]


def apply_mask(data: bytes, mask: bytes) -> bytes:
    """
    Apply masking to the data of a WebSocket message.

    :param data: Data to mask
    :param mask: 4-bytes mask

    """
    if len(mask) != 4:
        raise ValueError("mask must contain 4 bytes")

    return bytes(b ^ m for b, m in zip(data, itertools.cycle(mask)))

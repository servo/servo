# -*- coding: utf-8 -*-
"""
hyperframe/exceptions
~~~~~~~~~~~~~~~~~~~~~

Defines the exceptions that can be thrown by hyperframe.
"""


class UnknownFrameError(ValueError):
    """
    An frame of unknown type was received.
    """
    def __init__(self, frame_type, length):
        #: The type byte of the unknown frame that was received.
        self.frame_type = frame_type

        #: The length of the data portion of the unknown frame.
        self.length = length

    def __str__(self):
        return (
            "UnknownFrameError: Unknown frame type 0x%X received, "
            "length %d bytes" % (self.frame_type, self.length)
        )


class InvalidPaddingError(ValueError):
    """
    A frame with invalid padding was received.
    """
    pass


class InvalidFrameError(ValueError):
    """
    Parsing a frame failed because the data was not laid out appropriately.

    .. versionadded:: 3.0.2
    """
    pass

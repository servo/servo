# -*- coding: utf-8 -*-
"""
h2/errors
~~~~~~~~~~~~~~~~~~~

Global error code registry containing the established HTTP/2 error codes.

The current registry is available at:
https://tools.ietf.org/html/rfc7540#section-11.4
"""
import enum


class ErrorCodes(enum.IntEnum):
    """
    All known HTTP/2 error codes.

    .. versionadded:: 2.5.0
    """
    #: Graceful shutdown.
    NO_ERROR = 0x0

    #: Protocol error detected.
    PROTOCOL_ERROR = 0x1

    #: Implementation fault.
    INTERNAL_ERROR = 0x2

    #: Flow-control limits exceeded.
    FLOW_CONTROL_ERROR = 0x3

    #: Settings not acknowledged.
    SETTINGS_TIMEOUT = 0x4

    #: Frame received for closed stream.
    STREAM_CLOSED = 0x5

    #: Frame size incorrect.
    FRAME_SIZE_ERROR = 0x6

    #: Stream not processed.
    REFUSED_STREAM = 0x7

    #: Stream cancelled.
    CANCEL = 0x8

    #: Compression state not updated.
    COMPRESSION_ERROR = 0x9

    #: TCP connection error for CONNECT method.
    CONNECT_ERROR = 0xa

    #: Processing capacity exceeded.
    ENHANCE_YOUR_CALM = 0xb

    #: Negotiated TLS parameters not acceptable.
    INADEQUATE_SECURITY = 0xc

    #: Use HTTP/1.1 for the request.
    HTTP_1_1_REQUIRED = 0xd


def _error_code_from_int(code):
    """
    Given an integer error code, returns either one of :class:`ErrorCodes
    <h2.errors.ErrorCodes>` or, if not present in the known set of codes,
    returns the integer directly.
    """
    try:
        return ErrorCodes(code)
    except ValueError:
        return code


#: Graceful shutdown.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.NO_ERROR
#:    <h2.errors.ErrorCodes.NO_ERROR>`.
NO_ERROR = ErrorCodes.NO_ERROR

#: Protocol error detected.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.PROTOCOL_ERROR
#:    <h2.errors.ErrorCodes.PROTOCOL_ERROR>`.
PROTOCOL_ERROR = ErrorCodes.PROTOCOL_ERROR

#: Implementation fault.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.INTERNAL_ERROR
#:    <h2.errors.ErrorCodes.INTERNAL_ERROR>`.
INTERNAL_ERROR = ErrorCodes.INTERNAL_ERROR

#: Flow-control limits exceeded.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.FLOW_CONTROL_ERROR
#:    <h2.errors.ErrorCodes.FLOW_CONTROL_ERROR>`.
FLOW_CONTROL_ERROR = ErrorCodes.FLOW_CONTROL_ERROR

#: Settings not acknowledged.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.SETTINGS_TIMEOUT
#:    <h2.errors.ErrorCodes.SETTINGS_TIMEOUT>`.
SETTINGS_TIMEOUT = ErrorCodes.SETTINGS_TIMEOUT

#: Frame received for closed stream.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.STREAM_CLOSED
#:    <h2.errors.ErrorCodes.STREAM_CLOSED>`.
STREAM_CLOSED = ErrorCodes.STREAM_CLOSED

#: Frame size incorrect.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.FRAME_SIZE_ERROR
#:    <h2.errors.ErrorCodes.FRAME_SIZE_ERROR>`.
FRAME_SIZE_ERROR = ErrorCodes.FRAME_SIZE_ERROR

#: Stream not processed.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.REFUSED_STREAM
#:    <h2.errors.ErrorCodes.REFUSED_STREAM>`.
REFUSED_STREAM = ErrorCodes.REFUSED_STREAM

#: Stream cancelled.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.CANCEL
#:    <h2.errors.ErrorCodes.CANCEL>`.
CANCEL = ErrorCodes.CANCEL

#: Compression state not updated.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.COMPRESSION_ERROR
#:    <h2.errors.ErrorCodes.COMPRESSION_ERROR>`.
COMPRESSION_ERROR = ErrorCodes.COMPRESSION_ERROR

#: TCP connection error for CONNECT method.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.CONNECT_ERROR
#:    <h2.errors.ErrorCodes.CONNECT_ERROR>`.
CONNECT_ERROR = ErrorCodes.CONNECT_ERROR

#: Processing capacity exceeded.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.ENHANCE_YOUR_CALM
#:    <h2.errors.ErrorCodes.ENHANCE_YOUR_CALM>`.
ENHANCE_YOUR_CALM = ErrorCodes.ENHANCE_YOUR_CALM

#: Negotiated TLS parameters not acceptable.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.INADEQUATE_SECURITY
#:    <h2.errors.ErrorCodes.INADEQUATE_SECURITY>`.
INADEQUATE_SECURITY = ErrorCodes.INADEQUATE_SECURITY

#: Use HTTP/1.1 for the request.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes.HTTP_1_1_REQUIRED
#:    <h2.errors.ErrorCodes.HTTP_1_1_REQUIRED>`.
HTTP_1_1_REQUIRED = ErrorCodes.HTTP_1_1_REQUIRED

#: All known HTTP/2 error codes.
#:
#: .. deprecated:: 2.5.0
#:    Deprecated in favour of :class:`ErrorCodes <h2.errors.ErrorCodes>`.
H2_ERRORS = list(ErrorCodes)

__all__ = ['H2_ERRORS', 'NO_ERROR', 'PROTOCOL_ERROR', 'INTERNAL_ERROR',
           'FLOW_CONTROL_ERROR', 'SETTINGS_TIMEOUT', 'STREAM_CLOSED',
           'FRAME_SIZE_ERROR', 'REFUSED_STREAM', 'CANCEL', 'COMPRESSION_ERROR',
           'CONNECT_ERROR', 'ENHANCE_YOUR_CALM', 'INADEQUATE_SECURITY',
           'HTTP_1_1_REQUIRED', 'ErrorCodes']

# Copyright 2011, Google Inc.
# All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are
# met:
#
#     * Redistributions of source code must retain the above copyright
# notice, this list of conditions and the following disclaimer.
#     * Redistributions in binary form must reproduce the above
# copyright notice, this list of conditions and the following disclaimer
# in the documentation and/or other materials provided with the
# distribution.
#     * Neither the name of Google Inc. nor the names of its
# contributors may be used to endorse or promote products derived from
# this software without specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
# "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
# LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
# A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
# OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
# LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
# DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
# THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
# (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
"""WebSocket utilities."""

from __future__ import absolute_import
import array
import errno
import logging
import os
import re
import six
from six.moves import map
from six.moves import range
import socket
import struct
import zlib

try:
    from mod_pywebsocket import fast_masking
except ImportError:
    pass


def prepend_message_to_exception(message, exc):
    """Prepend message to the exception."""
    exc.args = (message + str(exc), )
    return


def __translate_interp(interp, cygwin_path):
    """Translate interp program path for Win32 python to run cygwin program
    (e.g. perl).  Note that it doesn't support path that contains space,
    which is typically true for Unix, where #!-script is written.
    For Win32 python, cygwin_path is a directory of cygwin binaries.

    Args:
      interp: interp command line
      cygwin_path: directory name of cygwin binary, or None
    Returns:
      translated interp command line.
    """
    if not cygwin_path:
        return interp
    m = re.match('^[^ ]*/([^ ]+)( .*)?', interp)
    if m:
        cmd = os.path.join(cygwin_path, m.group(1))
        return cmd + m.group(2)
    return interp


def get_script_interp(script_path, cygwin_path=None):
    r"""Get #!-interpreter command line from the script.

    It also fixes command path.  When Cygwin Python is used, e.g. in WebKit,
    it could run "/usr/bin/perl -wT hello.pl".
    When Win32 Python is used, e.g. in Chromium, it couldn't.  So, fix
    "/usr/bin/perl" to "<cygwin_path>\perl.exe".

    Args:
      script_path: pathname of the script
      cygwin_path: directory name of cygwin binary, or None
    Returns:
      #!-interpreter command line, or None if it is not #!-script.
    """
    fp = open(script_path)
    line = fp.readline()
    fp.close()
    m = re.match('^#!(.*)', line)
    if m:
        return __translate_interp(m.group(1), cygwin_path)
    return None


def hexify(s):
    return ' '.join(['%02x' % x for x in six.iterbytes(s)])


def get_class_logger(o):
    """Return the logging class information."""
    return logging.getLogger('%s.%s' %
                             (o.__class__.__module__, o.__class__.__name__))


def pack_byte(b):
    """Pack an integer to network-ordered byte"""
    return struct.pack('!B', b)


class NoopMasker(object):
    """A NoOp masking object.

    This has the same interface as RepeatedXorMasker but just returns
    the string passed in without making any change.
    """
    def __init__(self):
        """NoOp."""
        pass

    def mask(self, s):
        """NoOp."""
        return s


class RepeatedXorMasker(object):
    """A masking object that applies XOR on the string.

    Applies XOR on the byte string given to mask method with the masking bytes
    given to the constructor repeatedly. This object remembers the position
    in the masking bytes the last mask method call ended and resumes from
    that point on the next mask method call.
    """
    def __init__(self, masking_key):
        self._masking_key = masking_key
        self._masking_key_index = 0

    def _mask_using_swig(self, s):
        """Perform the mask via SWIG."""
        masked_data = fast_masking.mask(s, self._masking_key,
                                        self._masking_key_index)
        self._masking_key_index = ((self._masking_key_index + len(s)) %
                                   len(self._masking_key))
        return masked_data

    def _mask_using_array(self, s):
        """Perform the mask via python."""
        if isinstance(s, six.text_type):
            raise Exception(
                'Masking Operation should not process unicode strings')

        result = bytearray(s)

        # Use temporary local variables to eliminate the cost to access
        # attributes
        masking_key = [c for c in six.iterbytes(self._masking_key)]
        masking_key_size = len(masking_key)
        masking_key_index = self._masking_key_index

        for i in range(len(result)):
            result[i] ^= masking_key[masking_key_index]
            masking_key_index = (masking_key_index + 1) % masking_key_size

        self._masking_key_index = masking_key_index

        return bytes(result)

    if 'fast_masking' in globals():
        mask = _mask_using_swig
    else:
        mask = _mask_using_array


# By making wbits option negative, we can suppress CMF/FLG (2 octet) and
# ADLER32 (4 octet) fields of zlib so that we can use zlib module just as
# deflate library. DICTID won't be added as far as we don't set dictionary.
# LZ77 window of 32K will be used for both compression and decompression.
# For decompression, we can just use 32K to cover any windows size. For
# compression, we use 32K so receivers must use 32K.
#
# Compression level is Z_DEFAULT_COMPRESSION. We don't have to match level
# to decode.
#
# See zconf.h, deflate.cc, inflate.cc of zlib library, and zlibmodule.c of
# Python. See also RFC1950 (ZLIB 3.3).


class _Deflater(object):
    def __init__(self, window_bits):
        self._logger = get_class_logger(self)

        # Using the smallest window bits of 9 for generating input frames.
        # On WebSocket spec, the smallest window bit is 8. However, zlib does
        # not accept window_bit = 8.
        #
        # Because of a zlib deflate quirk, back-references will not use the
        # entire range of 1 << window_bits, but will instead use a restricted
        # range of (1 << window_bits) - 262. With an increased window_bits = 9,
        # back-references will be within a range of 250. These can still be
        # decompressed with window_bits = 8 and the 256-byte window used there.
        #
        # Similar disscussions can be found in https://crbug.com/691074
        window_bits = max(window_bits, 9)

        self._compress = zlib.compressobj(zlib.Z_DEFAULT_COMPRESSION,
                                          zlib.DEFLATED, -window_bits)

    def compress(self, bytes):
        compressed_bytes = self._compress.compress(bytes)
        self._logger.debug('Compress input %r', bytes)
        self._logger.debug('Compress result %r', compressed_bytes)
        return compressed_bytes

    def compress_and_flush(self, bytes):
        compressed_bytes = self._compress.compress(bytes)
        compressed_bytes += self._compress.flush(zlib.Z_SYNC_FLUSH)
        self._logger.debug('Compress input %r', bytes)
        self._logger.debug('Compress result %r', compressed_bytes)
        return compressed_bytes

    def compress_and_finish(self, bytes):
        compressed_bytes = self._compress.compress(bytes)
        compressed_bytes += self._compress.flush(zlib.Z_FINISH)
        self._logger.debug('Compress input %r', bytes)
        self._logger.debug('Compress result %r', compressed_bytes)
        return compressed_bytes


class _Inflater(object):
    def __init__(self, window_bits):
        self._logger = get_class_logger(self)
        self._window_bits = window_bits

        self._unconsumed = b''

        self.reset()

    def decompress(self, size):
        if not (size == -1 or size > 0):
            raise Exception('size must be -1 or positive')

        data = b''

        while True:
            data += self._decompress.decompress(self._unconsumed,
                                                max(0, size - len(data)))
            self._unconsumed = self._decompress.unconsumed_tail
            if self._decompress.unused_data:
                # Encountered a last block (i.e. a block with BFINAL = 1) and
                # found a new stream (unused_data). We cannot use the same
                # zlib.Decompress object for the new stream. Create a new
                # Decompress object to decompress the new one.
                #
                # It's fine to ignore unconsumed_tail if unused_data is not
                # empty.
                self._unconsumed = self._decompress.unused_data
                self.reset()
                if size >= 0 and len(data) == size:
                    # data is filled. Don't call decompress again.
                    break
                else:
                    # Re-invoke Decompress.decompress to try to decompress all
                    # available bytes before invoking read which blocks until
                    # any new byte is available.
                    continue
            else:
                # Here, since unused_data is empty, even if unconsumed_tail is
                # not empty, bytes of requested length are already in data. We
                # don't have to "continue" here.
                break

        if data:
            self._logger.debug('Decompressed %r', data)
        return data

    def append(self, data):
        self._logger.debug('Appended %r', data)
        self._unconsumed += data

    def reset(self):
        self._logger.debug('Reset')
        self._decompress = zlib.decompressobj(-self._window_bits)


# Compresses/decompresses given octets using the method introduced in RFC1979.


class _RFC1979Deflater(object):
    """A compressor class that applies DEFLATE to given byte sequence and
    flushes using the algorithm described in the RFC1979 section 2.1.
    """
    def __init__(self, window_bits, no_context_takeover):
        self._deflater = None
        if window_bits is None:
            window_bits = zlib.MAX_WBITS
        self._window_bits = window_bits
        self._no_context_takeover = no_context_takeover

    def filter(self, bytes, end=True, bfinal=False):
        if self._deflater is None:
            self._deflater = _Deflater(self._window_bits)

        if bfinal:
            result = self._deflater.compress_and_finish(bytes)
            # Add a padding block with BFINAL = 0 and BTYPE = 0.
            result = result + pack_byte(0)
            self._deflater = None
            return result

        result = self._deflater.compress_and_flush(bytes)
        if end:
            # Strip last 4 octets which is LEN and NLEN field of a
            # non-compressed block added for Z_SYNC_FLUSH.
            result = result[:-4]

        if self._no_context_takeover and end:
            self._deflater = None

        return result


class _RFC1979Inflater(object):
    """A decompressor class a la RFC1979.

    A decompressor class for byte sequence compressed and flushed following
    the algorithm described in the RFC1979 section 2.1.
    """
    def __init__(self, window_bits=zlib.MAX_WBITS):
        self._inflater = _Inflater(window_bits)

    def filter(self, bytes):
        # Restore stripped LEN and NLEN field of a non-compressed block added
        # for Z_SYNC_FLUSH.
        self._inflater.append(bytes + b'\x00\x00\xff\xff')
        return self._inflater.decompress(-1)


class DeflateSocket(object):
    """A wrapper class for socket object to intercept send and recv to perform
    deflate compression and decompression transparently.
    """

    # Size of the buffer passed to recv to receive compressed data.
    _RECV_SIZE = 4096

    def __init__(self, socket):
        self._socket = socket

        self._logger = get_class_logger(self)

        self._deflater = _Deflater(zlib.MAX_WBITS)
        self._inflater = _Inflater(zlib.MAX_WBITS)

    def recv(self, size):
        """Receives data from the socket specified on the construction up
        to the specified size. Once any data is available, returns it even
        if it's smaller than the specified size.
        """

        # TODO(tyoshino): Allow call with size=0. It should block until any
        # decompressed data is available.
        if size <= 0:
            raise Exception('Non-positive size passed')
        while True:
            data = self._inflater.decompress(size)
            if len(data) != 0:
                return data

            read_data = self._socket.recv(DeflateSocket._RECV_SIZE)
            if not read_data:
                return b''
            self._inflater.append(read_data)

    def sendall(self, bytes):
        self.send(bytes)

    def send(self, bytes):
        self._socket.sendall(self._deflater.compress_and_flush(bytes))
        return len(bytes)


# vi:sts=4 sw=4 et

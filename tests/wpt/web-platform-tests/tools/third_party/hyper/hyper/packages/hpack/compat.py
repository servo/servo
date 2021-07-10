# -*- coding: utf-8 -*-
"""
hpack/compat
~~~~~~~~~~~~

Normalizes the Python 2/3 API for internal use.
"""
import sys


_ver = sys.version_info
is_py2 = _ver[0] == 2
is_py3 = _ver[0] == 3

if is_py2:
    def to_byte(char):
        return ord(char)

    def decode_hex(b):
        return b.decode('hex')

    unicode = unicode
    bytes = str

elif is_py3:
    def to_byte(char):
        return char

    def decode_hex(b):
        return bytes.fromhex(b)

    unicode = str
    bytes = bytes

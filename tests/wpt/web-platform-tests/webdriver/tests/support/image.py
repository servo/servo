import base64
import math
import struct

import six

from tests.support.asserts import assert_png


def decodebytes(s):
    if six.PY3:
        return base64.decodebytes(six.ensure_binary(s))
    return base64.decodestring(s)

def png_dimensions(screenshot):
    assert_png(screenshot)
    image = decodebytes(six.ensure_binary(screenshot))
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)

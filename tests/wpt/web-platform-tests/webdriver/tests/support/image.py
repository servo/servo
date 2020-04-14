import base64
import math
import struct

from six import ensure_binary

from tests.support.asserts import assert_png


def png_dimensions(screenshot):
    assert_png(screenshot)
    image = base64.decodestring(ensure_binary(screenshot))
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)

import base64
import math
import struct

from tests.support.asserts import assert_png


def png_dimensions(screenshot):
    assert_png(screenshot)
    image = base64.decodestring(screenshot)
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)

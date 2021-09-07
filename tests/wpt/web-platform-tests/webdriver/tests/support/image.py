import math
import struct
from base64 import decodebytes

from tests.support.asserts import assert_png


def png_dimensions(screenshot):
    assert_png(screenshot)
    image = decodebytes(screenshot.encode())
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)

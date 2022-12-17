import struct

from tests.support.asserts import assert_png


def png_dimensions(screenshot):
    image = assert_png(screenshot)
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)

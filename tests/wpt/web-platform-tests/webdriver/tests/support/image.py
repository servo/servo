import struct

from tests.support.asserts import assert_png


PPI = 96
inch_in_cm = 2.54


def cm_to_px(px):
    return round(px * PPI / inch_in_cm)


def px_to_cm(px):
    return px * inch_in_cm / PPI


def png_dimensions(screenshot):
    image = assert_png(screenshot)
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)


class ImageDifference:
    """Summary of the pixel-level differences between two images.

    :param total_pixels: The total number of pixel differences between the images
    :param max_difference: The maximum difference between any corresponding color channels across
                           all pixels of the image.
    """

    def __init__(self, total_pixels, max_difference):
        self.total_pixels = total_pixels
        self.max_difference = max_difference

    def equal(self):
        return self.total_pixels == 0

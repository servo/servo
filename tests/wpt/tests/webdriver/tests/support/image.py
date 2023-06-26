import struct
from typing import NamedTuple, Tuple

from tests.support.asserts import assert_png


PPI = 96
inch_in_cm = 2.54


def cm_to_px(cm: float) -> float:
    return round(cm * PPI / inch_in_cm)


def px_to_cm(px: float) -> float:
    return px * inch_in_cm / PPI


def png_dimensions(screenshot) -> Tuple[int, int]:
    image = assert_png(screenshot)
    width, height = struct.unpack(">LL", image[16:24])
    return int(width), int(height)


class ImageDifference(NamedTuple):
    """Summary of the pixel-level differences between two images."""

    """The total number of pixel differences between the images"""
    total_pixels: int

    """The maximum difference between any corresponding color channels across all pixels of the image"""
    max_difference: int

    def equal(self) -> bool:
        return self.total_pixels == 0

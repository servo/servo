# Copyright 2026 web-platform-tests contributors
# SPDX-License-Identifier: BSD-3-Clause

"""Generate an original monochrome font for the RTL glyph extents reftest.

Requires fontTools. Each glyph advances 500 units. Arabic U+0627 extends to
the right, U+0628 to the left, and U+062A stays within its advance. Positive
left bearings keep the first and third glyphs contained even if the platform
pads their bounds for antialiasing. Their bounds fit inside the font
ascent/descent. The test uses 20, 40, 60, and 140px to exercise size-dependent
glyph measurement.
"""

from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen


def rectangle(bounds):
    pen = TTGlyphPen(None)
    if bounds:
        x0, y0, x1, y1 = bounds
        pen.moveTo((x0, y0))
        pen.lineTo((x0, y1))
        pen.lineTo((x1, y1))
        pen.lineTo((x1, y0))
        pen.closePath()
    return pen.glyph()


glyph_bounds = {
    ".notdef": None,
    "right": (100, 0, 1000, 800),
    "left": (-500, 0, 500, 800),
    "inside": (100, 0, 500, 800),
}
builder = FontBuilder(1000, isTTF=True)
builder.setupGlyphOrder(list(glyph_bounds))
builder.setupCharacterMap({0x0627: "right", 0x0628: "left", 0x062A: "inside"})
builder.setupGlyf({name: rectangle(bounds) for name, bounds in glyph_bounds.items()})
builder.setupHorizontalMetrics(
    {name: (500, bounds[0] if bounds else 0) for name, bounds in glyph_bounds.items()}
)
builder.setupHorizontalHeader(ascent=1000, descent=-200)
builder.setupNameTable(
    {
        "familyName": "RTL Ink Overflow",
        "styleName": "Regular",
        "uniqueFontIdentifier": "RTLInkOverflow-Regular",
        "fullName": "RTL Ink Overflow Regular",
        "psName": "RTLInkOverflow-Regular",
        "licenseDescription": "BSD 3-Clause License",
        "licenseInfoURL": "https://github.com/web-platform-tests/wpt/blob/master/LICENSE.md",
    }
)
builder.setupOS2(
    sTypoAscender=1000, sTypoDescender=-200, usWinAscent=1000, usWinDescent=200
)
builder.setupPost()
builder.setupMaxp()
builder.font.recalcTimestamp = False
builder.font["head"].created = builder.font["head"].modified = 2082844800
destination = Path(__file__).resolve().parent
builder.font.save(destination / "rtl-ink-overflow.ttf")

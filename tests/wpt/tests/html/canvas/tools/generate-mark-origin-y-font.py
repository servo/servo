#!/usr/bin/env python3
# Generates mark-origin-y-test.ttf, a font whose combining marks are placed by a GPOS
# mark-to-base anchor with a vertical offset, so shaping reports a non-zero y glyph origin.
#
# Usage: pip install fonttools && ./generate-mark-origin-y-font.py
#
# This script is standalone: it is not part of the gentest.py canvas test generation in this directory.
# The output is deterministic (the head table timestamps are pinned), so regenerating it reproduces the
# committed font byte for byte. The font is shared in /fonts/, so any test that needs a mark with a
# vertical glyph offset can use it.
#
# Glyph design (1000 units per em, ascent 800, descent 200, cap height 700):
#
#   'A'    -> bar        advance 1000, ink x [0,1000] y [0,200]
#   U+0301 -> raisedmark advance 0,    drawn at y [0,200], positioned to ink y [700,900]
#   U+0316 -> loweredmark advance 0,   drawn at y [0,200], positioned to ink y [-300,-100]
#
# Both marks are drawn on the baseline, so a measurement that ignores the glyph origin sees
# them inside the base's [0,200] extent. Positioned, the raised mark sticks out 100 units above
# the font ascent and the lowered one 100 units below the font descent.

import io
import os

from fontTools.fontBuilder import FontBuilder
from fontTools.feaLib.builder import addOpenTypeFeatures
from fontTools.pens.ttGlyphPen import TTGlyphPen

EM = 1000
ASCENT = 800
DESCENT = 200
CAP_HEIGHT = 700

FAMILY = "mark-origin-y-test"

FEATURES = """
languagesystem DFLT dflt;
languagesystem latn dflt;

markClass [raisedmark] <anchor 0 0> @RAISED;
markClass [loweredmark] <anchor 0 0> @LOWERED;

feature mark {
    pos base [bar] <anchor 0 700> mark @RAISED;
    pos base [bar] <anchor 0 -300> mark @LOWERED;
} mark;
"""


def bar():
    pen = TTGlyphPen(None)
    pen.moveTo((0, 0))
    pen.lineTo((0, 200))
    pen.lineTo((EM, 200))
    pen.lineTo((EM, 0))
    pen.closePath()
    return pen.glyph()


def empty():
    return TTGlyphPen(None).glyph()


def main():
    glyph_order = [".notdef", "space", "bar", "raisedmark", "loweredmark"]
    glyphs = {
        ".notdef": empty(),
        "space": empty(),
        "bar": bar(),
        "raisedmark": bar(),
        "loweredmark": bar(),
    }
    # (advance, left side bearing). A combining mark must have a zero advance.
    metrics = {
        ".notdef": (EM, 0),
        "space": (EM, 0),
        "bar": (EM, 0),
        "raisedmark": (0, 0),
        "loweredmark": (0, 0),
    }
    cmap = {
        0x0020: "space",
        0x0041: "bar",          # 'A'
        0x0301: "raisedmark",   # COMBINING ACUTE ACCENT
        0x0316: "loweredmark",  # COMBINING GRAVE ACCENT BELOW
    }

    fb = FontBuilder(EM, isTTF=True)
    fb.font["head"].created = fb.font["head"].modified = 0
    fb.setupGlyphOrder(glyph_order)
    fb.setupCharacterMap(cmap)
    fb.setupGlyf(glyphs)
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=ASCENT, descent=-DESCENT)
    fb.setupNameTable({
        "familyName": FAMILY,
        "styleName": "Regular",
        "uniqueFontIdentifier": "%s;Regular;web-platform-tests" % FAMILY,
        "fullName": FAMILY,
        "version": "1.000",
        "psName": "%s-Regular" % FAMILY,
        "manufacturer": "web-platform-tests",
        "licenseDescription":
            "Generated for web-platform-tests by "
            "html/canvas/tools/generate-mark-origin-y-font.py. "
            "Available under the W3C 3-clause BSD license.",
    })
    fb.setupOS2(
        sTypoAscender=ASCENT,
        sTypoDescender=-DESCENT,
        sTypoLineGap=0,
        usWinAscent=ASCENT,
        usWinDescent=DESCENT,
        sxHeight=CAP_HEIGHT,
        sCapHeight=CAP_HEIGHT,
        achVendID="NONE",
        fsType=0,
    )
    fb.setupPost(isFixedPitch=0)

    addOpenTypeFeatures(fb.font, io.StringIO(FEATURES))

    # Lives in the shared /fonts/ directory, referenced as /fonts/mark-origin-y-test.ttf.
    out_dir = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "fonts")
    out = os.path.normpath(os.path.join(out_dir, "%s.ttf" % FAMILY))
    fb.save(out)
    print("Wrote %s" % out)


if __name__ == "__main__":
    main()

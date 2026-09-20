#!/usr/bin/env python3
# Generates letter-spacing/support/mark-anchor-test.ttf, a font that positions a
# combining mark with an OpenType GPOS mark-to-base anchor.
#
# Usage: pip install fonttools && ./generate-mark-anchor-font.py
#
# Why a purpose-built font is needed
# ----------------------------------
# A test for "letter-spacing is not inserted inside a typographic character
# unit" needs a combining mark whose position comes from a *glyph offset* rather
# than from glyph advances, because that is the case where an implementation can
# get the cluster's advance right and the mark's paint position wrong. Whether a
# given system font does that for a given mark is not knowable from within a
# test, so the font is built here instead, with an explicit GPOS MarkBasePos
# anchor and a zero-advance mark glyph. Core Text additionally composes common
# Latin base-plus-mark pairs into a single precomposed glyph, which removes the
# mark entirely, so the mark must be one with no precomposed form.
#
# Glyph design
# ------------
# Marks are stacked *above* the base rather than beside it, and every glyph
# occupies a distinct horizontal band, so a correctly positioned cluster is
# pixel-identical to a single reference glyph without any two outlines sharing,
# crossing, or overlapping an edge:
#
#   'A'    -> bottombar   advance 1000, ink x [0,1000] y [0,200]
#   U+0301 -> midbar      advance 0,    ink x [0,1000] y [250,450]
#   U+0300 -> topbar      advance 0,    ink x [0,1000] y [500,700]
#   'B'    -> twobars     advance 1000, bottombar + midbar
#   'C'    -> threebars   advance 1000, bottombar + midbar + topbar
#   'D'    -> midbaronly  advance 1000, midbar's ink as a *spacing* glyph
#
#   "A" + U+0301            ==  "B"
#   "A" + U+0301 + U+0300   ==  "C"
#   "A" + space + U+0301    ==  "AD"      (see the space anchor below)
#
# The same glyphs are also mapped to Hebrew ALEF, BET, GIMEL and DALET so the RTL
# tests compare the same outlines the LTR ones do. A Hebrew base rather than
# 'unicode-bidi: bidi-override', because an override assigns the mark its level
# directly and so skips bidi rule W1, the rule that keeps a mark with its base.
#
#   U+05D0 -> bottombar    U+05D1 -> twobars
#   U+05D2 -> threebars    U+05D3 -> midbaronly
#
# Both marks anchor to the same point on the base, so each independently proves
# it landed on its base; giving them separate bands is what lets a two-mark
# cluster be compared exactly, since two marks painting the *same* rectangle
# would composite its antialiased edges twice and no longer match a reference
# that paints it once.
#
# 'space' is a mark-attachment base as well as 'A'. A word-spacing test needs a
# mark whose base is a word-separator character, and 'D' gives that test a
# reference: "A" + space + U+0301 paints the bottom bar in the first em and the
# middle bar in the second, which is exactly what "AD" paints.
#
# Deliberately avoided:
#   - No horizontal adjacency between base and marks, so there is no interior
#     vertical edge whose antialiasing could differ from the reference.
#   - No overlapping outlines anywhere, for the compositing reason above.
#
# Every ink coordinate is a multiple of 50 font units, so at a font-size that is
# a multiple of 20px every edge lands on a whole pixel at both 1x and 2x.

import io
import os

from fontTools.fontBuilder import FontBuilder
from fontTools.feaLib.builder import addOpenTypeFeatures
from fontTools.pens.ttGlyphPen import TTGlyphPen

EM = 1000
ASCENT = 800
DESCENT = 200

BOTTOM_BAR = (0, 200)     # y range of the base's ink
MID_BAR = (250, 450)      # y range of U+0301's ink
TOP_BAR = (500, 700)      # y range of U+0300's ink

# The base puts the mark's origin back at the base's own origin: the mark's
# anchor is at x=0, so a base anchor of x=0 means the shaper emits an offset of
# -EM (cancelling the base's advance) and the mark paints over the base.
BASE_ANCHOR_X = 0

FAMILY = "mark-anchor-test"

FEATURES = """
languagesystem DFLT dflt;
languagesystem latn dflt;
languagesystem hebr dflt;

markClass [midbar topbar] <anchor 0 0> @MARKS;

feature mark {
    pos base [bottombar space] <anchor %d 0> mark @MARKS;
} mark;
""" % BASE_ANCHOR_X


def bars(*y_ranges):
    pen = TTGlyphPen(None)
    for y_min, y_max in y_ranges:
        pen.moveTo((0, y_min))
        pen.lineTo((0, y_max))
        pen.lineTo((EM, y_max))
        pen.lineTo((EM, y_min))
        pen.closePath()
    return pen.glyph()


def empty():
    return TTGlyphPen(None).glyph()


def main():
    glyph_order = [".notdef", "space", "bottombar", "midbar", "topbar",
                   "twobars", "threebars", "midbaronly"]

    glyphs = {
        ".notdef": empty(),
        "space": empty(),
        "bottombar": bars(BOTTOM_BAR),
        "midbar": bars(MID_BAR),
        "topbar": bars(TOP_BAR),
        "twobars": bars(BOTTOM_BAR, MID_BAR),
        "threebars": bars(BOTTOM_BAR, MID_BAR, TOP_BAR),
        "midbaronly": bars(MID_BAR),
    }

    # (advance, left side bearing). A combining mark must have a zero advance;
    # that is what makes it a mark rather than a spacing glyph. 'midbaronly' is
    # the same ink with a normal advance, for the word-spacing reference.
    metrics = {
        ".notdef": (EM, 0),
        "space": (EM, 0),
        "bottombar": (EM, 0),
        "midbar": (0, 0),
        "topbar": (0, 0),
        "twobars": (EM, 0),
        "threebars": (EM, 0),
        "midbaronly": (EM, 0),
    }

    cmap = {
        0x0020: "space",
        0x0041: "bottombar",   # 'A'
        0x0042: "twobars",     # 'B'
        0x0043: "threebars",   # 'C'
        0x0044: "midbaronly",  # 'D'
        0x0300: "topbar",      # COMBINING GRAVE ACCENT
        0x0301: "midbar",      # COMBINING ACUTE ACCENT
        # Hebrew bases for the RTL tests; the marks are script-neutral and reused
        # as-is, and neither has a precomposed form on a Hebrew base.
        0x05D0: "bottombar",   # ALEF
        0x05D1: "twobars",     # BET
        0x05D2: "threebars",   # GIMEL
        0x05D3: "midbaronly",  # DALET
    }

    fb = FontBuilder(EM, isTTF=True)
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
        "psName": "mark-anchor-test-Regular",
        "manufacturer": "web-platform-tests",
        "licenseDescription":
            "Generated for web-platform-tests by "
            "css/css-text/tools/generate-mark-anchor-font.py. "
            "Available under the W3C 3-clause BSD license.",
    })
    fb.setupOS2(
        sTypoAscender=ASCENT,
        sTypoDescender=-DESCENT,
        sTypoLineGap=0,
        usWinAscent=ASCENT,
        usWinDescent=DESCENT,
        sxHeight=TOP_BAR[1],
        sCapHeight=TOP_BAR[1],
        achVendID="NONE",
        fsType=0,
    )
    fb.setupPost(isFixedPitch=0)

    addOpenTypeFeatures(fb.font, io.StringIO(FEATURES))

    # Lives in the shared /fonts/ directory because tests in three directories
    # across two specs use it: css-text/letter-spacing, css-text/word-spacing,
    # and css-fonts. Referenced as /fonts/mark-anchor-test.ttf.
    out_dir = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "fonts")
    # Shipped uncompressed so that the font stays readable by standard OpenType
    # tooling: hb-shape has no WOFF support, and being able to run it on the
    # bundled file is worth more here than a few hundred bytes.
    out = os.path.normpath(os.path.join(out_dir, "%s.ttf" % FAMILY))
    fb.save(out)
    print("Wrote %s" % out)


if __name__ == "__main__":
    main()

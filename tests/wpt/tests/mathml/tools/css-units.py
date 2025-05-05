#!/usr/bin/env python3

from utils import mathfont

# Create a font with glyphs for all the font-relative CSS units
# (so that they all have length 1em).
# See https://drafts.csswg.org/css-values-4/#lengths
f = mathfont.create("css-units",
                    "Copyright (c) 2025 Igalia S.L.")

mathfont.drawRectangleGlyph(f.createChar(ord("x")), mathfont.em, mathfont.em // 2) # ex = 0.5em
mathfont.createSquareGlyph(f, ord("O")) # cap = 1em
mathfont.createSquareGlyph(f, ord("0")) # ch = 1em
mathfont.createSquareGlyph(f, ord("水")) # ic = 1em

assert f.capHeight == 1000
assert f.xHeight == 500

mathfont.save(f)

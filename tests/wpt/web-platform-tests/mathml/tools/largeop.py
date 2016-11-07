#!/usr/bin/python

from utils import mathfont
import fontforge

nAryWhiteVerticalBarCodePoint = 0x2AFF
v1 = 5 * mathfont.em
f = mathfont.create("largeop-displayoperatorminheight%d" % v1)
f.math.DisplayOperatorMinHeight = v1
mathfont.createSquareGlyph(f, nAryWhiteVerticalBarCodePoint)
g = f.createChar(-1, "uni2AFF.display")
mathfont.drawRectangleGlyph(g, mathfont.em, v1, 0)
f[nAryWhiteVerticalBarCodePoint].verticalVariants = "uni2AFF uni2AFF.display"
mathfont.save(f)

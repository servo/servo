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

v1 = 2 * mathfont.em
v2 = 3 * mathfont.em
f = mathfont.create("largeop-displayoperatorminheight%d-2AFF-italiccorrection%d" % (v1, v2))
f.copyright = "Copyright (c) 2018 Igalia S.L."
f.math.DisplayOperatorMinHeight = v1
mathfont.createSquareGlyph(f, nAryWhiteVerticalBarCodePoint)
g = f.createChar(-1, "uni2AFF.display")
p = g.glyphPen()
p.moveTo(0, 0)
p.lineTo(v2, v1)
p.lineTo(v2 + mathfont.em, v1)
p.lineTo(mathfont.em, 0)
p.closePath();
g.width = mathfont.em + v2
g.italicCorrection = v2
f[nAryWhiteVerticalBarCodePoint].verticalVariants = "uni2AFF uni2AFF.display"
mathfont.save(f)

#!/usr/bin/python

from utils import mathfont
import fontforge

nAryWhiteVerticalBarCodePoint = 0x2AFF
v1 = 5 * mathfont.em
f = mathfont.create("largeop-displayoperatorminheight%d" % v1,
                    "Copyright (c) 2016 MathML Association")
f.math.DisplayOperatorMinHeight = v1
mathfont.createSquareGlyph(f, nAryWhiteVerticalBarCodePoint)
g = f.createChar(-1, "uni2AFF.display")
mathfont.drawRectangleGlyph(g, mathfont.em, v1, 0)
f[nAryWhiteVerticalBarCodePoint].verticalVariants = "uni2AFF uni2AFF.display"
mathfont.save(f)

v1 = 2 * mathfont.em
v2 = 3 * mathfont.em
f = mathfont.create("largeop-displayoperatorminheight%d-2AFF-italiccorrection%d" % (v1, v2),
                    "Copyright (c) 2018 Igalia S.L.")
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

v1 = 7 * mathfont.em
v2 = 5 * mathfont.em
f = mathfont.create("largeop-displayoperatorminheight%d-2AFF-italiccorrection%d" % (v1, v2),
                    "Copyright (c) 2020 Igalia S.L.")
f.math.DisplayOperatorMinHeight = v1
f.math.MinConnectorOverlap = 0
mathfont.createSquareGlyph(f, nAryWhiteVerticalBarCodePoint)
g = f.createChar(-1, "uni2AFF.bot")
mathfont.drawRectangleGlyph(g,
                            width = 2 * mathfont.em,
                            ascent = mathfont.em)
g = f.createChar(-1, "uni2AFF.ext")
mathfont.drawRectangleGlyph(g,
                            width = mathfont.em,
                            ascent = 2 * mathfont.em,
                            padding_left = mathfont.em)
g = f.createChar(-1, "uni2AFF.top")
mathfont.drawRectangleGlyph(g,
                            width = v2 + mathfont.em,
                            ascent = mathfont.em,
                            padding_left = mathfont.em)
f[nAryWhiteVerticalBarCodePoint].verticalVariants = "uni2AFF"
# Part: (glyphName, isExtender, startConnector, endConnector, fullAdvance)
f[nAryWhiteVerticalBarCodePoint].verticalComponents = \
  (("uni2AFF.bot", False, 0, mathfont.em / 2, mathfont.em),
   ("uni2AFF.ext", True, mathfont.em / 2, mathfont.em / 2, 2 * mathfont.em),
   ("uni2AFF.top", False, mathfont.em / 2, 0, mathfont.em)
  );
f[nAryWhiteVerticalBarCodePoint].verticalComponentItalicCorrection = v2
mathfont.save(f)

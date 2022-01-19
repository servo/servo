#!/usr/bin/env python3

from utils import mathfont
import fontforge

v = mathfont.em / 2
f = mathfont.create("xheight%d" % v,
                    "Copyright (c) 2016 MathML Association")
g = f.createChar(ord('x'))
mathfont.drawRectangleGlyph(g, mathfont.em, v, 0)
assert f.xHeight == v, "Bad x-height value!"
mathfont.save(f)

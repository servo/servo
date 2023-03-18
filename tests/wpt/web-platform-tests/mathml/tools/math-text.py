#!/usr/bin/env python3

import fontforge

font = fontforge.font()
font.em = 1000
lineHeight = 5000
name = "math-text"
font.fontname = name
font.familyname = name
font.fullname = name
font.copyright = "Copyright (c) 2019 Igalia"

glyph = font.createChar(ord(" "), "space")
glyph.width = 1000
glyph = font.createChar(ord("A"))
pen = glyph.glyphPen()
pen.moveTo(0, -500)
pen.lineTo(0, 500)
pen.lineTo(1000, 500)
pen.lineTo(1000, -500)
pen.closePath()

glyph = font.createChar(ord("B"))
pen = glyph.glyphPen()
pen.moveTo(0, 0)
pen.lineTo(0, 1000)
pen.lineTo(1000, 1000)
pen.lineTo(1000, 0)
pen.closePath()

glyph = font.createChar(ord("C"))
pen = glyph.glyphPen()
pen.moveTo(0, -1000)
pen.lineTo(0, 0)
pen.lineTo(1000, 0)
pen.lineTo(1000, -1000)
pen.closePath()

font.os2_typoascent_add = False
font.os2_typoascent = lineHeight // 2
font.os2_typodescent_add = False
font.os2_typodescent = -lineHeight // 2
font.os2_typolinegap = 0
font.hhea_ascent = lineHeight // 2
font.hhea_ascent_add = False
font.hhea_descent = -lineHeight // 2
font.hhea_descent_add = False
font.hhea_linegap = 0
font.os2_winascent = lineHeight // 2
font.os2_winascent_add = False
font.os2_windescent = lineHeight // 2
font.os2_windescent_add = False

font.os2_use_typo_metrics = True

path = "../../fonts/math/math-text.woff"
print("Generating %s..." % path, end="")
font.generate(path)
if font.validate() == 0:
    print(" done.")
else:
    print(" validation error!")
    exit(1)

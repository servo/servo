#!/usr/bin/python

from __future__ import print_function
import fontforge

font = fontforge.font()
font.em = 1000
typoLineHeight = 2300
winHeight = 5000
name = "font-lineheight%d-typolineheight%d" % (winHeight, typoLineHeight)
font.fontname = name
font.familyname = name
font.fullname = name
font.copyright = "Copyright (c) 2016 MathML Association"

glyph = font.createChar(ord(" "), "space")
glyph.width = 1000
glyph = font.createChar(ord("O"))
pen = glyph.glyphPen()
pen.moveTo(0, -200)
pen.lineTo(0, 800)
pen.lineTo(1000, 800)
pen.lineTo(1000, -200)
pen.closePath();

font.os2_typoascent_add = False
font.os2_typoascent = 800
font.os2_typodescent_add = False
font.os2_typodescent = -200
font.os2_typolinegap = typoLineHeight - \
                       (font.os2_typoascent - font.os2_typodescent)

font.hhea_ascent = winHeight / 2
font.hhea_ascent_add = False
font.hhea_descent = -winHeight / 2
font.hhea_descent_add = False
font.hhea_linegap = 0

font.os2_winascent = winHeight / 2
font.os2_winascent_add = False
font.os2_windescent = winHeight / 2
font.os2_windescent_add = False

font.os2_use_typo_metrics = True

path = "../../fonts/math/lineheight%d-typolineheight%d.woff" % (winHeight, typoLineHeight)
print("Generating %s..." % path, end="")
font.generate(path)
if font.validate() == 0:
    print(" done.")
else:
    print(" validation error!")
    exit(1)

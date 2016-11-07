from __future__ import print_function
import fontforge
from misc import MathMLAssociationCopyright

em = 1000

def create(aName):
    print("Generating %s.woff..." % aName, end="")
    mathFont = fontforge.font()
    mathFont.fontname = aName
    mathFont.familyname = aName
    mathFont.fullname = aName
    mathFont.copyright = MathMLAssociationCopyright
    mathFont.encoding = "UnicodeFull"

    # Create a space character. Also force the creation of some MATH subtables
    # so that OTS will not reject the MATH table.
    g = mathFont.createChar(ord(" "), "space")
    g.width = em
    g.italicCorrection = 0
    g.topaccent = 0
    g.mathKern.bottomLeft = tuple([(0,0)])
    g.mathKern.bottomRight = tuple([(0,0)])
    g.mathKern.topLeft = tuple([(0,0)])
    g.mathKern.topRight = tuple([(0,0)])
    mathFont[ord(" ")].horizontalVariants = "space"
    mathFont[ord(" ")].verticalVariants = "space"
    return mathFont

def drawRectangleGlyph(aGlyph, aWidth, aAscent, aDescent):
    p = aGlyph.glyphPen()
    p.moveTo(0, -aDescent)
    p.lineTo(0, aAscent)
    p.lineTo(aWidth, aAscent)
    p.lineTo(aWidth, -aDescent)
    p.closePath();
    aGlyph.width = aWidth

def createSquareGlyph(aFont, aCodePoint):
    g = aFont.createChar(aCodePoint)
    drawRectangleGlyph(g, em, em, 0)

def drawHexaDigit(aGlyph, aX, aValue):
    t = em / 10
    p = aGlyph.glyphPen(replace = False)
    if aValue == 0:
        p.moveTo(aX + t, t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.closePath()
    elif aValue == 1:
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
    elif aValue == 2:
        p.moveTo(aX + t, em - t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + t, em / 2)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
    elif aValue == 3:
        p.moveTo(aX + t, em - t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + t, t)
        p.endPath()
        p.moveTo(aX + t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 4:
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
        p.moveTo(aX + t, em - t)
        p.lineTo(aX + t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 5:
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, em / 2)
        p.lineTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + t, t)
        p.endPath()
    elif aValue == 6:
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 7:
        p.moveTo(aX + t, em - t)
        p.lineTo(aX + em / 2  - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
    elif aValue == 8:
        p.moveTo(aX + t, t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.closePath()
        p.moveTo(aX + 2.5 * t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 9:
        p.moveTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 10: # A
        p.moveTo(aX + t, t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
        p.moveTo(aX + 2.5 * t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 11: # b
        p.moveTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 12: # C
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
    elif aValue == 13: # d
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 14: # E
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
        p.moveTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 15: # F
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.endPath()
        p.moveTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + 2.5 * t, em / 2)
        p.endPath()

def createGlyphFromValue(aFont, aCodePoint):
    g = aFont.createChar(aCodePoint)
    value = aCodePoint
    for i in range(0, 5):
        drawHexaDigit(g, (5 - (i + 1)) * em / 2, value % 16)
        value /= 16
    g.width = 5 * em / 2
    g.stroke("circular", em / 10, "square", "miter", "cleanup")

def save(aFont):
    aFont.em = em
    aFont.ascent = aFont.hhea_ascent = aFont.os2_typoascent = em
    aFont.descent = aFont.hhea_descent = aFont.os2_typodescent = 0
    # aFont.os2_winascent, aFont.os2_windescent should be the maximum of
    # ascent/descent for all glyphs. Does fontforge compute them automatically?
    aFont.hhea_ascent_add = aFont.hhea_descent_add = 0
    aFont.os2_typoascent_add = aFont.os2_typodescent_add = 0
    aFont.os2_winascent_add = aFont.os2_windescent_add = 0
    aFont.os2_use_typo_metrics = True
    aFont.generate("../../fonts/math/%s.woff" % aFont.fontname)
    if aFont.validate() == 0:
        print(" done.")
    else:
        print(" validation error!")
        exit(1)

import fontforge

PUA_startCodePoint = 0xE000
em = 1000


def create(aName, aCopyRight):
    print("Generating %s.woff..." % aName, end="")
    mathFont = fontforge.font()
    mathFont.fontname = aName
    mathFont.familyname = aName
    mathFont.fullname = aName
    mathFont.copyright = aCopyRight
    mathFont.encoding = "UnicodeFull"

    # Create a space character. Also force the creation of some MATH subtables
    # so that OTS will not reject the MATH table.
    g = mathFont.createChar(ord(" "), "space")
    g.width = em
    g.italicCorrection = 0
    g.topaccent = 0
    g.mathKern.bottomLeft = tuple([(0, 0)])
    g.mathKern.bottomRight = tuple([(0, 0)])
    g.mathKern.topLeft = tuple([(0, 0)])
    g.mathKern.topRight = tuple([(0, 0)])
    mathFont[ord(" ")].horizontalVariants = "space"
    mathFont[ord(" ")].verticalVariants = "space"
    return mathFont


def drawRectangleGlyph(glyph, width, ascent, descent=0, padding_left=0):
    p = glyph.glyphPen()
    p.moveTo(padding_left, -descent)
    p.lineTo(padding_left, ascent)
    p.lineTo(padding_left + width, ascent)
    p.lineTo(padding_left + width, -descent)
    p.closePath()
    glyph.width = padding_left + width


def createSquareGlyph(aFont, aCodePoint):
    g = aFont.createChar(aCodePoint)
    drawRectangleGlyph(g, em, em, 0)


def drawHexaDigit(aGlyph, aX, aValue):
    t = em / 10
    p = aGlyph.glyphPen(replace=False)
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
        p.lineTo(aX + em / 2 - t, em - t)
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
    elif aValue == 10:  # A
        p.moveTo(aX + t, t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
        p.moveTo(aX + 2.5 * t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 11:  # b
        p.moveTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 12:  # C
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
    elif aValue == 13:  # d
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + em / 2 - t, t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + t, em / 2)
        p.lineTo(aX + em / 2 - 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 14:  # E
        p.moveTo(aX + em / 2 - t, em - t)
        p.lineTo(aX + t, em - t)
        p.lineTo(aX + t, t)
        p.lineTo(aX + em / 2 - t, t)
        p.endPath()
        p.moveTo(aX + em / 2 - t, em / 2)
        p.lineTo(aX + 2.5 * t, em / 2)
        p.endPath()
    elif aValue == 15:  # F
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
    g.width = 5 * em // 2
    g.stroke("circular", em / 10, "square", "miter", "cleanup")


def createSizeVariants(aFont, aUsePUA=False, aCenterOnBaseline=False):
    if aUsePUA:
        codePoint = PUA_startCodePoint
    else:
        codePoint = -1
    for size in (0, 1, 2, 3):
        g = aFont.createChar(codePoint, "v%d" % size)
        if aCenterOnBaseline:
            drawRectangleGlyph(g, em, (size + 1) * em / 2, (size + 1) * em / 2)
        else:
            drawRectangleGlyph(g, em, (size + 1) * em, 0)
        if aUsePUA:
            codePoint += 1
        g = aFont.createChar(codePoint, "h%d" % size)
        if aCenterOnBaseline:
            drawRectangleGlyph(g, (size + 1) * em, em / 2, em / 2)
        else:
            drawRectangleGlyph(g, (size + 1) * em, em, 0)
        if aUsePUA:
            codePoint += 1


def createStretchy(aFont, codePoint, isHorizontal):
    if isHorizontal:
        aFont[codePoint].horizontalVariants = "h0 h1 h2 h3"
        # Part: (glyphName, isExtender, startConnector, endConnector, fullAdvance)
        aFont[codePoint].horizontalComponents = \
            (("h2", False, 0, em, 3 * em),
             ("h1", True, em, em, 2 * em))
    else:
        aFont[codePoint].verticalVariants = "v0 v1 v2 v3"
        # Part: (glyphName, isExtender, startConnector, endConnector, fullAdvance)
        aFont[codePoint].verticalComponents = \
            (("v2", False, 0, em, 3 * em),
             ("v1", True, em, em, 2 * em))


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

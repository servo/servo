#!/usr/bin/env python3

from utils import mathfont
import fontforge

# Create a WOFF font with glyphs for all the operator strings.
font = mathfont.create("stretchy", "Copyright (c) 2021-2024 Igalia S.L.")

font.math.AxisHeight = 0

# Set parameters for stretchy tests.
font.math.MinConnectorOverlap = mathfont.em // 2

# Make sure that underover parameters don't add extra spacing.
font.math.LowerLimitBaselineDropMin = 0
font.math.LowerLimitGapMin = 0
font.math.StretchStackBottomShiftDown = 0
font.math.StretchStackGapAboveMin = 0
font.math.UnderbarVerticalGap = 0
font.math.UnderbarExtraDescender = 0
font.math.UpperLimitBaselineRiseMin = 0
font.math.UpperLimitGapMin = 0
font.math.StretchStackTopShiftUp = 0
font.math.StretchStackGapBelowMin = 0
font.math.OverbarVerticalGap = 0
font.math.AccentBaseHeight = 0
font.math.OverbarExtraAscender = 0

# These two characters will be stretchable in both directions.
horizontalArrow = 0x295A  # LEFTWARDS HARPOON WITH BARB UP FROM BAR
verticalArrow = 0x295C  # UPWARDS HARPOON WITH BARB RIGHT FROM BAR
upDownArrowWithBase = 0x21A8 # UP DOWN ARROW WITH BASE

mathfont.createSizeVariants(font, aUsePUA=True, aCenterOnBaseline=False)

# Add stretchy vertical and horizontal constructions for the horizontal arrow.
mathfont.createSquareGlyph(font, horizontalArrow)
mathfont.createStretchy(font, horizontalArrow, True)
mathfont.createStretchy(font, horizontalArrow, False)

# Add stretchy vertical and horizontal constructions for the vertical arrow.
mathfont.createSquareGlyph(font, verticalArrow)
mathfont.createStretchy(font, verticalArrow, True)
mathfont.createStretchy(font, verticalArrow, False)

# U+21A8 stretches vertically using two size variants: a base glyph (of height
# half an em) and taller glyphs (of heights 1, 2, 3 and 4 em).
g = font.createChar(upDownArrowWithBase)
mathfont.drawRectangleGlyph(g, mathfont.em, mathfont.em/2, 0)
font[upDownArrowWithBase].verticalVariants = "uni21A8 v0 v1 v2 v3"

mathfont.save(font)


# Create a font to test RTL operators.
font = mathfont.create("stretchy-text-direction-asymetrical", "Copyright (c) 2025 Igalia S.L.")

# Left and right parentheses and brackets, in the following order: { ) } (
codePoints = (0x007B, 0x0029, 0x007D, 0x0028)

for i, point in enumerate(codePoints):
    em = mathfont.em
    width = int(em * (i + 1))
    ascent = em

    glyph = font.createChar(point)
    mathfont.drawRectangleGlyph(glyph, width, ascent)

    for size in (0, 1, 2, 3):
        g = font.createChar(-1, "v%d%d" % (size, i))
        mathfont.drawRectangleGlyph(g, width, (size + 1) * ascent, 0)

    font[point].verticalVariants = "v0%d v1%d v2%d v3%d" % ((i,) * 4)
    font[point].verticalComponents = \
        (("v2%d" % i, False, 0, width, 3 * em),
         ("v1%d" % i, True, em, width, 2 * em))

mathfont.save(font)

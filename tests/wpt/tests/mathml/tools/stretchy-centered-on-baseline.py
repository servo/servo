#!/usr/bin/env python3

from utils import mathfont
import fontforge

# Create a WOFF font with glyphs for all the operator strings.
font = mathfont.create("stretchy-centered-on-baseline",
                       "Copyright (c) 2023 Igalia S.L.")

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

mathfont.createSizeVariants(font, aUsePUA=True, aCenterOnBaseline=True)

# Add stretchy vertical and horizontal constructions for the horizontal arrow.
mathfont.createSquareGlyph(font, horizontalArrow)
mathfont.createStretchy(font, horizontalArrow, True)
mathfont.createStretchy(font, horizontalArrow, False)

# Add stretchy vertical and horizontal constructions for the vertical arrow.
mathfont.createSquareGlyph(font, verticalArrow)
mathfont.createStretchy(font, verticalArrow, True)
mathfont.createStretchy(font, verticalArrow, False)

mathfont.save(font)

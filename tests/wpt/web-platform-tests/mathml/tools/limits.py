#!/usr/bin/env python3

from utils import mathfont
import fontforge

nArySumCodePoint = 0x2211  # largeop operator

v = 3 * mathfont.em
f = mathfont.create("limits-lowerlimitbaselinedropmin%d" % v,
                    "Copyright (c) 2016 MathML Association")
mathfont.createSquareGlyph(f, nArySumCodePoint)
f.math.LowerLimitBaselineDropMin = v
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = 0
f.math.StretchStackBottomShiftDown = 0
f.math.StretchStackGapAboveMin = 0
f.math.StretchStackGapBelowMin = 0
f.math.StretchStackTopShiftUp = 0
f.math.UnderbarExtraDescender = 0
f.math.UnderbarVerticalGap = 0
f.math.UpperLimitBaselineRiseMin = 0
f.math.UpperLimitGapMin = 0
mathfont.save(f)

v = 11 * mathfont.em
f = mathfont.create("limits-lowerlimitgapmin%d" % v,
                    "Copyright (c) 2016 MathML Association")
mathfont.createSquareGlyph(f, nArySumCodePoint)
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = v
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = 0
f.math.StretchStackBottomShiftDown = 0
f.math.StretchStackGapAboveMin = 0
f.math.StretchStackGapBelowMin = 0
f.math.StretchStackTopShiftUp = 0
f.math.UnderbarExtraDescender = 0
f.math.UnderbarVerticalGap = 0
f.math.UpperLimitBaselineRiseMin = 0
f.math.UpperLimitGapMin = 0
mathfont.save(f)

v = 5 * mathfont.em
f = mathfont.create("limits-upperlimitbaselinerisemin%d" % v,
                    "Copyright (c) 2016 MathML Association")
mathfont.createSquareGlyph(f, nArySumCodePoint)
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = 0
f.math.StretchStackBottomShiftDown = 0
f.math.StretchStackGapAboveMin = 0
f.math.StretchStackGapBelowMin = 0
f.math.StretchStackTopShiftUp = 0
f.math.UnderbarExtraDescender = 0
f.math.UnderbarVerticalGap = 0
f.math.UpperLimitBaselineRiseMin = v
f.math.UpperLimitGapMin = 0
mathfont.save(f)

v = 7 * mathfont.em
f = mathfont.create("limits-upperlimitgapmin%d" % v,
                    "Copyright (c) 2016 MathML Association")
mathfont.createSquareGlyph(f, nArySumCodePoint)
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = 0
f.math.StretchStackBottomShiftDown = 0
f.math.StretchStackGapAboveMin = 0
f.math.StretchStackGapBelowMin = 0
f.math.StretchStackTopShiftUp = 0
f.math.UnderbarExtraDescender = 0
f.math.UnderbarVerticalGap = 0
f.math.UpperLimitBaselineRiseMin = 0
f.math.UpperLimitGapMin = v
mathfont.save(f)

#!/usr/bin/python

from utils import mathfont
import fontforge

breveCodePoint = 0x2D8 # accent operator
degreeCodePoint = 0xB0 # nonaccent operator
accentBaseHeight = 4 * mathfont.em

v = 3 * mathfont.em
f = mathfont.create("underover-accentbaseheight%d-overbarextraascender%d" % (accentBaseHeight, v))
mathfont.createSquareGlyph(f, breveCodePoint)
mathfont.createSquareGlyph(f, degreeCodePoint)
f.math.AccentBaseHeight = accentBaseHeight
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = v
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
f = mathfont.create("underover-accentbaseheight%d-overbarverticalgap%d" % (accentBaseHeight, v))
mathfont.createSquareGlyph(f, breveCodePoint)
mathfont.createSquareGlyph(f, degreeCodePoint)
f.math.AccentBaseHeight = accentBaseHeight
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = v
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
f = mathfont.create("underover-accentbaseheight%d-underbarextradescender%d" % (accentBaseHeight, v))
mathfont.createSquareGlyph(f, breveCodePoint)
mathfont.createSquareGlyph(f, degreeCodePoint)
f.math.AccentBaseHeight = accentBaseHeight
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = 0
f.math.StretchStackBottomShiftDown = 0
f.math.StretchStackGapAboveMin = 0
f.math.StretchStackGapBelowMin = 0
f.math.StretchStackTopShiftUp = 0
f.math.UnderbarExtraDescender = v
f.math.UnderbarVerticalGap = 0
f.math.UpperLimitBaselineRiseMin = 0
f.math.UpperLimitGapMin = 0
mathfont.save(f)

v = 7 * mathfont.em
f = mathfont.create("underover-accentbaseheight%d-underbarverticalgap%d" % (accentBaseHeight, v))
mathfont.createSquareGlyph(f, breveCodePoint)
mathfont.createSquareGlyph(f, degreeCodePoint)
f.math.AccentBaseHeight = accentBaseHeight
f.math.LowerLimitBaselineDropMin = 0
f.math.LowerLimitGapMin = 0
f.math.OverbarExtraAscender = 0
f.math.OverbarVerticalGap = 0
f.math.StretchStackBottomShiftDown = 0
f.math.StretchStackGapAboveMin = 0
f.math.StretchStackGapBelowMin = 0
f.math.StretchStackTopShiftUp = 0
f.math.UnderbarExtraDescender = 0
f.math.UnderbarVerticalGap = v
f.math.UpperLimitBaselineRiseMin = 0
f.math.UpperLimitGapMin = 0
mathfont.save(f)

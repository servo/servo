#!/usr/bin/python

from utils import mathfont
import fontforge

v1 = 7 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-axisheight%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = v1
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 5 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-denominatordisplaystylegapmin%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = 0
f.math.FractionDenominatorDisplayStyleGapMin = v1
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 6 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-denominatordisplaystyleshiftdown%d-axisheight%d-rulethickness%d" % (v1, v2, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = v2
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = v1
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 4 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-denominatorgapmin%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = 0
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = v1
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 3 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-denominatorshiftdown%d-axisheight%d-rulethickness%d" % (v1, v2, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = v2
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = v1
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 8 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-numeratordisplaystylegapmin%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = 0
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = v1
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 2 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-numeratordisplaystyleshiftup%d-axisheight%d-rulethickness%d" % (v1, v2, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = v2
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = v1
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 9 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-numeratorgapmin%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = 0
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = v1
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 11 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("fraction-numeratorshiftup%d-axisheight%d-rulethickness%d" % (v1, v2, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = v2
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = v1
f.math.FractionRuleThickness = v2
mathfont.save(f)

v1 = 10 * mathfont.em
f = mathfont.create("fraction-rulethickness%d" % v1,
                    "Copyright (c) 2016 MathML Association")
f.math.AxisHeight = 0
f.math.FractionDenominatorDisplayStyleGapMin = 0
f.math.FractionDenominatorDisplayStyleShiftDown = 0
f.math.FractionDenominatorGapMin = 0
f.math.FractionDenominatorShiftDown = 0
f.math.FractionNumeratorDisplayStyleGapMin = 0
f.math.FractionNumeratorDisplayStyleShiftUp = 0
f.math.FractionNumeratorGapMin = 0
f.math.FractionNumeratorShiftUp = 0
f.math.FractionRuleThickness = v1
mathfont.save(f)

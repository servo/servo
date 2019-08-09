#!/usr/bin/python

from utils import mathfont
import fontforge

v1 = 5 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("stack-bottomdisplaystyleshiftdown%d-axisheight%d" % (v1, v2))
f.math.AxisHeight = v2
f.math.StackBottomDisplayStyleShiftDown = v1
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = 0
mathfont.save(f)

v1 = 6 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("stack-bottomshiftdown%d-axisheight%d" % (v1, v2))
f.math.AxisHeight = v2
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = v1
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = 0
mathfont.save(f)

v = 4 * mathfont.em
f = mathfont.create("stack-displaystylegapmin%d" % v)
f.math.AxisHeight = 0
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = v
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = 0
mathfont.save(f)

v = 8 * mathfont.em
f = mathfont.create("stack-gapmin%d" % v)
f.math.AxisHeight = 0
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = v
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = 0
mathfont.save(f)

v1 = 3 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("stack-topdisplaystyleshiftup%d-axisheight%d" % (v1, v2))
f.math.AxisHeight = v2
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = v1
f.math.StackTopShiftUp = 0
mathfont.save(f)

v1 = 9 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("stack-topshiftup%d-axisheight%d" % (v1, v2))
f.math.AxisHeight = v2
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = v1
mathfont.save(f)

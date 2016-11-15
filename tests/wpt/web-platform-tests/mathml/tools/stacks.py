#!/usr/bin/python

from utils import mathfont
import fontforge

v = 7 * mathfont.em
f = mathfont.create("stack-axisheight%d" % v)
f.math.AxisHeight = v
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = 0
mathfont.save(f)

v = 5 * mathfont.em
f = mathfont.create("stack-bottomdisplaystyleshiftdown%d" % v)
f.math.AxisHeight = 0
f.math.StackBottomDisplayStyleShiftDown = v
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = 0
mathfont.save(f)

v = 6 * mathfont.em
f = mathfont.create("stack-bottomshiftdown%d" % v)
f.math.AxisHeight = 0
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = v
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

v = 3 * mathfont.em
f = mathfont.create("stack-topdisplaystyleshiftup%d" % v)
f.math.AxisHeight = 0
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = v
f.math.StackTopShiftUp = 0
mathfont.save(f)

v = 9 * mathfont.em
f = mathfont.create("stack-topshiftup%d" % v)
f.math.AxisHeight = 0
f.math.StackBottomDisplayStyleShiftDown = 0
f.math.StackBottomShiftDown = 0
f.math.StackDisplayStyleGapMin = 0
f.math.StackGapMin = 0
f.math.StackTopDisplayStyleShiftUp = 0
f.math.StackTopShiftUp = v
mathfont.save(f)

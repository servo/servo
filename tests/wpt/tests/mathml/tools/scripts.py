#!/usr/bin/env python3

from utils import mathfont
import fontforge

v = 3 * mathfont.em
f = mathfont.create("scripts-spaceafterscript%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = v
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 7 * mathfont.em
f = mathfont.create("scripts-superscriptshiftup%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = v
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 5 * mathfont.em
f = mathfont.create("scripts-superscriptshiftupcramped%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = v
mathfont.save(f)

v = 6 * mathfont.em
f = mathfont.create("scripts-subscriptshiftdown%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = v
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 11 * mathfont.em
f = mathfont.create("scripts-subsuperscriptgapmin%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = v
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v1 = 11 * mathfont.em
v2 = 3 * mathfont.em
f = mathfont.create("scripts-subsuperscriptgapmin%d-superscriptbottommaxwithsubscript%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = v1
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = v2
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 4 * mathfont.em
f = mathfont.create("scripts-subscripttopmax%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = v
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 8 * mathfont.em
f = mathfont.create("scripts-superscriptbottommin%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = v
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 9 * mathfont.em
f = mathfont.create("scripts-subscriptbaselinedropmin%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = v
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = 0
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

v = 10 * mathfont.em
f = mathfont.create("scripts-superscriptbaselinedropmax%d" % v,
                    "Copyright (c) 2016 MathML Association")
f.math.SpaceAfterScript = 0
f.math.SubSuperscriptGapMin = 0
f.math.SubscriptBaselineDropMin = 0
f.math.SubscriptShiftDown = 0
f.math.SubscriptTopMax = 0
f.math.SuperscriptBaselineDropMax = v
f.math.SuperscriptBottomMaxWithSubscript = 0
f.math.SuperscriptBottomMin = 0
f.math.SuperscriptShiftUp = 0
f.math.SuperscriptShiftUpCramped = 0
mathfont.save(f)

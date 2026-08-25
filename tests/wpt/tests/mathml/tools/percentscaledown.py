#!/usr/bin/env python3

from utils import mathfont
import fontforge

v1 = 80
v2 = 40
f = mathfont.create("scriptpercentscaledown%d-scriptscriptpercentscaledown%d" % (v1, v2),
                    "Copyright (c) 2019 Igalia S.L.")
f.math.ScriptPercentScaleDown = v1
f.math.ScriptScriptPercentScaleDown = v2
mathfont.save(f)

f = mathfont.create("scriptpercentscaledown0-scriptscriptpercentscaledown%d" % v2,
                    "Copyright (c) 2019 Igalia S.L.")
f.math.ScriptPercentScaleDown = 0
f.math.ScriptScriptPercentScaleDown = v2
mathfont.save(f)

f = mathfont.create("scriptpercentscaledown%d-scriptscriptpercentscaledown0" % v1,
                    "Copyright (c) 2019 Igalia S.L.")
f.math.ScriptPercentScaleDown = v1
f.math.ScriptScriptPercentScaleDown = 0
mathfont.save(f)

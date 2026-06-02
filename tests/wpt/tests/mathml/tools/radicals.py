#!/usr/bin/env python3

from utils import mathfont
import fontforge


radicalCodePoint = 0x221a


def createStretchyRadical(aFont, codePoint=radicalCodePoint, width=mathfont.em, suffix=""):
    g = aFont.createChar(codePoint, "radical%s" % suffix)
    mathfont.drawRectangleGlyph(g, width, mathfont.em, 0)
    for size in (0, 1, 2, 3):
        g = aFont.createChar(-1, "size%d%s" % (size, suffix))
        mathfont.drawRectangleGlyph(g, width, (size + 1) * mathfont.em, 0)
    aFont[codePoint].verticalVariants = "radical%s size1%s size2%s size3%s" % ((suffix,) * 4)
    # Part: (glyphName, isExtender, startConnector, endConnector, fullAdvance)
    aFont.math.MinConnectorOverlap = 0
    aFont[codePoint].verticalComponents = \
        (("size2%s" % suffix, False, 0, width, 3 * mathfont.em),
         ("size1%s" % suffix, True, mathfont.em, width, 2 * mathfont.em))


v1 = 25
v2 = 1 * mathfont.em
f = mathfont.create("radical-degreebottomraisepercent%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = v1
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = 0
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v1 = 7 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-displaystyleverticalgap%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = v1
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = 0
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v1 = 3 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-extraascender%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = v1
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = 0
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v1 = 5 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-kernafterdegreeminus%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = -v1
f.math.RadicalKernBeforeDegree = 0
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v1 = 4 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-kernbeforedegree%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = v1
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v = 8 * mathfont.em
f = mathfont.create("radical-rulethickness%d" % v,
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = 0
f.math.RadicalRuleThickness = v
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v1 = 6 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-verticalgap%d-rulethickness%d" % (v1, v2),
                    "Copyright (c) 2016 MathML Association")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = 0
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = v1
mathfont.save(f)

v1 = 1 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-negativekernbeforedegree%d-rulethickness%d" %
                    (v1, v2), "Copyright (c) 2020 Igalia S.L.")
createStretchyRadical(f)
f.math.RadicalDegreeBottomRaisePercent = 0
f.math.RadicalDisplayStyleVerticalGap = 0
f.math.RadicalExtraAscender = 0
f.math.RadicalKernAfterDegree = 0
f.math.RadicalKernBeforeDegree = -v1
f.math.RadicalRuleThickness = v2
f.math.RadicalVerticalGap = 0
mathfont.save(f)

v1 = 4 * mathfont.em
v2 = 1 * mathfont.em
f = mathfont.create("radical-rtlm", "Copyright (c) 2025 Igalia S.L.")
f.addLookup("gsub", "gsub_single", (), (("rtlm", (("latn", ("dflt")),)),))
f.addLookupSubtable("gsub", "gsub_n")
createStretchyRadical(f, radicalCodePoint, v1)
createStretchyRadical(f, 0xE000, v2, ".rtlm")
f[radicalCodePoint].addPosSub("gsub_n", "radical.rtlm")
mathfont.save(f)

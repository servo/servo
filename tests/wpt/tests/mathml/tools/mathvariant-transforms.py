#!/usr/bin/env python3

from lxml import etree
from utils.misc import downloadWithProgressBar, UnicodeXMLURL
from utils import mathfont

# Retrieve the unicode.xml file if necessary.
unicodeXML = downloadWithProgressBar(UnicodeXMLURL)

# Extract the mathvariants transformation.
xsltTransform = etree.XSLT(etree.XML('''\
<xsl:stylesheet version="1.0"
                       xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:strip-space elements="*"/>
  <xsl:template match="charlist">
    <root><xsl:apply-templates select="character"/></root>
  </xsl:template>
  <xsl:template match="character">
    <xsl:if test="surrogate">
      <entry>
        <xsl:attribute name="mathvariant">
            <xsl:value-of select="surrogate/@mathvariant"/>
        </xsl:attribute>
        <xsl:attribute name="baseChar">
          <xsl:value-of select="surrogate/@ref"/>
        </xsl:attribute>
        <xsl:attribute name="transformedChar">
          <xsl:choose>
            <xsl:when test="bmp">
              <xsl:value-of select="bmp/@ref"/>
            </xsl:when>
            <xsl:otherwise>
               <xsl:value-of select="@id"/>
            </xsl:otherwise>
          </xsl:choose>
        </xsl:attribute>
      </entry>
    </xsl:if>
  </xsl:template>
</xsl:stylesheet>'''))

# Put the mathvariant transforms into a Python structure.
mathvariantTransforms = {}
root = xsltTransform(etree.parse(unicodeXML)).getroot()


def parseCodePoint(aHexaString):
    return int("0x%s" % aHexaString[1:], 16)


for entry in root:
    mathvariant = entry.get("mathvariant")
    baseChar = parseCodePoint(entry.get("baseChar"))
    transformedChar = parseCodePoint(entry.get("transformedChar"))
    if mathvariant not in mathvariantTransforms:
        mathvariantTransforms[mathvariant] = {}
    mathvariantTransforms[mathvariant][baseChar] = transformedChar

# There is no "isolated" mathvariant.
del mathvariantTransforms["isolated"]

# Automatic mathvariant uses the same transform as italic.
# It is handled specially (see below).
mathvariantTransforms["auto"] = mathvariantTransforms["italic"]

# Create a WOFF font for each mathvariant.
for mathvariant in mathvariantTransforms:
    if mathvariant == "auto":
        continue
    font = mathfont.create("mathvariant-%s" % mathvariant,
                           "Copyright (c) 2016 MathML Association")
    for baseChar in mathvariantTransforms[mathvariant]:
        if baseChar not in font:
            mathfont.createGlyphFromValue(font, baseChar)
        transformedChar = mathvariantTransforms[mathvariant][baseChar]
        mathfont.createGlyphFromValue(font, transformedChar)
    mathfont.save(font)

# Common function to generate test for MathML mathvariant / CSS text-transform.


def generateTestFor(mathvariant, mathml):
    assert mathml or mathvariant == "auto", "These tests have been removed!"
    print("Generating tests for %s..." % mathvariant, end="")
    if mathml:
        reftest = open(
            "../relations/css-styling/mathvariant-%s.html" % mathvariant, "w")
        reftestReference = open(
            "../relations/css-styling/mathvariant-%s-ref.html" % mathvariant, "w")
    else:
        reftest = open(
            "../../css/css-text/text-transform/math/text-transform-math-%s-001.html" % mathvariant, "w")
        reftestReference = open(
            "../../css/css-text/text-transform/math/text-transform-math-%s-001-ref.html" % mathvariant, "w")
    source = '\
<!DOCTYPE html>\n\
<html>\n\
<head>\n\
<meta charset="utf-8"/>\n\
<title>%s</title>\n'
    if mathml:
        reftest.write(source % ("mathvariant %s" % mathvariant))
        reftestReference.write(
            source % ("mathvariant %s (reference)" % mathvariant))
    else:
        reftest.write(source % ("text-transform math-%s" % mathvariant))
        reftestReference.write(
            source % ("text-transform math-%s (reference)" % mathvariant))
    if mathvariant == "auto":
        mathAssert = "Verify that a single-char <mi> is equivalent to an <mi> with the transformed italic unicode character."
        mapping = "italic"
    else:
        mathAssert = "Verify that a single-char <mtext> with a %s mathvariant is equivalent to an <mtext> with the transformed unicode character." % mathvariant
        mapping = mathvariant
    if mathml:
        source = '\
<link rel="help" href="https://w3c.github.io/mathml-core/#css-styling">\n\
<link rel="help" href="https://w3c.github.io/mathml-core/#the-mathvariant-attribute">\n\
<link rel="help" href="https://w3c.github.io/mathml-core/#new-text-transform-values">\n\
<link rel="help" href="https://w3c.github.io/mathml-core/#%s-mappings">\n\
<link rel="match" href="mathvariant-%s-ref.html"/>\n\
<meta name="assert" content="%s">\n'
        reftest.write(source % (mapping, mathvariant, mathAssert))
    else:
        source = '\
<link rel="help" href="https://github.com/w3c/csswg-drafts/issues/3745"/>\n\
<link rel="help" href="https://w3c.github.io/mathml-core/#new-text-transform-values">\n\
<link rel="help" href="https://w3c.github.io/mathml-core/#%s-mappings">\n\
<link rel="match" href="text-transform-math-%s-001-ref.html"/>\n\
<meta name="assert" content="Verify that a character with \'text-transform: math-%s\' renders the same as the transformed unicode character.">\n'
        reftest.write(source % (mapping, mathvariant, mathvariant))
    WOFFfont = "mathvariant-%s.woff" % mapping
    source = '\
<style>\n\
  @font-face {\n\
    font-family: TestFont;\n\
    src: url("/fonts/math/%s");\n\
  }\n\
  body > span {\n\
    padding: 10px;\n\
  }\n\
  span > span {\n\
    font-family: monospace;\n\
    font-size: 10px;\n\
  }\n\
  .testfont {\n\
    font-family: TestFont;\n\
    font-size: 10px;\n\
  }\n\
</style>\n\
<body>\n\
  <!-- Generated by mathml/tools/mathvariant.py; DO NOT EDIT. -->\n\
  <p>Test passes if all the equalities below are true.</p>\n' % WOFFfont
    if mathml:
        reftest.write(source)
        reftestReference.write(source)
    else:
        reftest.write(source)
        reftestReference.write(source)
    charIndex = 0
    for baseChar in mathvariantTransforms[mathvariant]:
        transformedChar = mathvariantTransforms[mathvariant][baseChar]
        if mathvariant == "auto":
            tokenTag = '<mi>&#x%0X;</mi>' % baseChar
            tokenTagRef = '<mi>&#x%0X;</mi>' % transformedChar
        else:
            tokenTag = '<mtext mathvariant="%s">&#x%0X;</mtext>' % (
                mathvariant, baseChar)
            tokenTagRef = '<mtext>&#x%0X;</mtext>' % transformedChar
        if mathml:
            reftest.write('  <span><math class="testfont">%s</math>=<span>%05X</span></span>' %
                          (tokenTag, transformedChar))
            reftestReference.write(
                '  <span><math class="testfont">%s</math>=<span>%05X</span></span>' % (tokenTagRef, transformedChar))
        else:
            reftest.write('  <span><span class="testfont" style="text-transform: math-%s">&#x%0X;</span>=<span>%05X</span></span>' %
                          (mathvariant, baseChar, transformedChar))
            reftestReference.write(
                '  <span><span class="testfont">&#x%0X;</span>=<span>%05X</span></span>' % (transformedChar, transformedChar))
        charIndex += 1
        if charIndex % 10 == 0:
            reftest.write('<br/>')
            reftestReference.write('<br/>')
        reftest.write('\n')
        reftestReference.write('\n')
    source = '</body>\n</html>\n'
    reftest.write(source)
    reftestReference.write(source)
    reftest.close()
    reftestReference.close()
    print(" done.")


# Generate css/css-text/text-transform/math/text-transform-math-auto-001.html
generateTestFor(mathvariant="auto", mathml=False)
generateTestFor(mathvariant="auto", mathml=True)

# Generate italic_mapping.js file used by selection tests.
print("Generating italic_mapping.js...", end="")
italic_mapping = open("../../css/css-text/text-transform/math/italic-mapping.js", "w")
italic_mapping.write("// Generated by mathml/tools/mathvariant.py; DO NOT EDIT.\n");
italic_mapping.write("let italic_mapping = {\n");
for baseChar in mathvariantTransforms["italic"]:
    transformedChar = mathvariantTransforms[mathvariant][baseChar]
    italic_mapping.write("  0x%0X: 0x%0X,\n" % (baseChar, transformedChar));
italic_mapping.write("}\n");
italic_mapping.close()
print(" done.")

# Other mathvariant tests can be generated by the following command. They are
# still use internally by browsers implementing full mathvariant support.
# See https://github.com/w3c/mathml-core/issues/182
# for mathvariant in mathvariantTransforms:
#    generateTestFor(mathvariant, mathml=True)

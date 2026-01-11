const xmlString = `
<foo>
  <bar>x</bar>
  <bar>y</bar>
</foo>
`;
const xsltString = `
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <result>
      <xsl:apply-templates select="document('resources/test.xml')//static" />
      <xsl:apply-templates select="foo" />
    </result>
  </xsl:template>
  <xsl:template match="static">
    <failure />
  </xsl:template>
  <xsl:template match="foo">
    <success />
  </xsl:template>
</xsl:stylesheet>
`;
const parser = new DOMParser();

const xmlDoc = parser.parseFromString(xmlString, "application/xml");
const xsltDoc = parser.parseFromString(xsltString, "application/xml");
const xsltProcessor = new XSLTProcessor();

xsltProcessor.importStylesheet(xsltDoc);

test(() => {
  const resultFrag = xsltProcessor.transformToFragment(xmlDoc, document);
  assert_equals(resultFrag.firstChild.localName, "result");
  assert_true(Array.prototype.every.call(resultFrag.firstChild.children,
                                         (e) => e.localName == "success"));
}, `xsl:document function disabled in transformToFragment`);

test(() => {
  const resultDoc = xsltProcessor.transformToDocument(xmlDoc);
  assert_equals(resultDoc.documentElement.localName, "result");
  assert_true(Array.prototype.every.call(resultDoc.documentElement.children,
                                         (e) => e.localName == "success"));
}, `xsl:document function disabled in transformToDocument`);

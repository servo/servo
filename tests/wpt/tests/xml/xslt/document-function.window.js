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
  assert_not_equals(resultFrag.querySelector("success"), null);
  assert_equals(resultFrag.querySelector("failure"), null);
}, `xsl:document function disabled in transformToFragment`);

test(() => {
  const resultDoc = xsltProcessor.transformToDocument(xmlDoc);
  assert_equals(resultDoc.documentElement.localName, "result");
  assert_not_equals(resultDoc.querySelector("success"), null);
  assert_equals(resultDoc.querySelector("failure"), null);
}, `xsl:document function disabled in transformToDocument`);

test(() => {
  // "http://[" is an invalid URI; [ signals the start of an IPv6 literal, but
  // there is no actual address, nor is there a closing ].
  const xmlWithInvalidBase = parser.parseFromString(
      `<foo xml:base="http://[" />`, "application/xml");
  const xsltStringWithInvalidBaseNode = `
  <xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
    <xsl:template match="/">
      <result>
        <count><xsl:value-of select="count(document('', /foo))" /></count>
      </result>
    </xsl:template>
  </xsl:stylesheet>
  `;
  const xsltDocWithInvalidBaseNode = parser.parseFromString(
      xsltStringWithInvalidBaseNode, "application/xml");
  const processor = new XSLTProcessor();
  processor.importStylesheet(xsltDocWithInvalidBaseNode);
  const resultDoc = processor.transformToDocument(xmlWithInvalidBase);
  assert_equals(resultDoc.querySelector("count").textContent, "0");
}, `document() with invalid xml:base target node returns an empty node-set`);


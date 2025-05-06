const xmlString = `
<items>
  <item>Item 1</item>
  <item>Item 2</item>
</items>
`;
const xsltString = `
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="*">
    <xsl:value-of select="name(.)"/>
  </xsl:template>
</xsl:stylesheet>
`;
const parser = new DOMParser();

const xmlDoc = parser.parseFromString(xmlString, "application/xml");
const xsltDoc = parser.parseFromString(xsltString, "application/xml");
const xsltProcessor = new XSLTProcessor();

xsltProcessor.importStylesheet(xsltDoc);

test(() => {
  const resultFragment = xsltProcessor.transformToFragment(xmlDoc.documentElement, document);
  assert_equals(resultFragment.childNodes.length, 1);
  assert_equals(resultFragment.firstChild.nodeValue, "items");
}, `'*' should match the documentElement`);

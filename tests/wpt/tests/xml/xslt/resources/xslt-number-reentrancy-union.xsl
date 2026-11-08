<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="html"/>
  <!-- $w runs xsl:number on the source document, leaving a cached result for
       the second alternative of the count pattern, then on doc2. On doc2 the
       count pattern lazily evaluates $v, which runs the same xsl:number on
       doc3 while the outer one is in progress. -->
  <xsl:variable name="w">
    <xsl:apply-templates select="/r/n" mode="num"/>
    <xsl:apply-templates select="document('xslt-number-reentrancy-doc2.xml')/r/n" mode="num"/>
  </xsl:variable>
  <xsl:variable name="v">
    <xsl:apply-templates select="document('xslt-number-reentrancy-doc3.xml')/r/n" mode="num"/>
    <xsl:text>v</xsl:text>
  </xsl:variable>
  <xsl:template match="n" mode="num">
    <xsl:number level="single" count="never | n[@id = '2'][boolean($v)]"/>
  </xsl:template>
  <xsl:template match="/">
    <html><body><xsl:value-of select="$w"/></body></html>
  </xsl:template>
</xsl:stylesheet>

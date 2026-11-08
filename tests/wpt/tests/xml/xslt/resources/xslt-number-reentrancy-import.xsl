<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <!-- The xsl:number patterns in the imported stylesheets share runtime cache
       slots with the template pattern in the first imported stylesheet. -->
  <xsl:import href="xslt-number-reentrancy-import-prime.xsl"/>
  <xsl:import href="xslt-number-reentrancy-import-outer.xsl"/>
  <xsl:import href="xslt-number-reentrancy-import-inner.xsl"/>
  <xsl:output method="html"/>
  <!-- $a populates the cache on the source document, then runs the outer
       xsl:number on doc2. The outer count pattern lazily evaluates $b, which
       runs the inner xsl:number on doc3 while the outer one is in progress. -->
  <xsl:variable name="a">
    <xsl:apply-templates select="/r/n" mode="prime"/>
    <xsl:for-each select="document('xslt-number-reentrancy-doc2.xml')/r/n">
      <xsl:call-template name="outer"/>
    </xsl:for-each>
  </xsl:variable>
  <xsl:variable name="b">
    <xsl:for-each select="document('xslt-number-reentrancy-doc3.xml')/r/n">
      <xsl:call-template name="inner"/>
    </xsl:for-each>
  </xsl:variable>
  <xsl:template match="/">
    <html><body><xsl:value-of select="$a"/><xsl:value-of select="$b"/></body></html>
  </xsl:template>
</xsl:stylesheet>

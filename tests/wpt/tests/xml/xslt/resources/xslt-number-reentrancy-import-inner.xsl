<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template name="inner">
    <xsl:number count="n[true()][true()]" level="single"/>
  </xsl:template>
</xsl:stylesheet>

<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template name="outer">
    <xsl:number count="n[true()][$b]" level="single"/>
  </xsl:template>
</xsl:stylesheet>

<?xml version="1.0" encoding="ISO-8859-1"?>

<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:param name="static" select="document(//static)"/>
  <xsl:template match="/">
    <xsl:for-each select="$static">
      <html>
        <body>
          <h1>hello world</h1>
        </body>
      </html>
    </xsl:for-each>
  </xsl:template>
</xsl:stylesheet>


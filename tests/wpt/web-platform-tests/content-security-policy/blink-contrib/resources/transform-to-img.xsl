<?xml version="1.0" encoding="ISO-8859-1"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:template match="/">
    <html>
      <head>
        <script>
//<![CDATA[
if (window.testRunner)
    testRunner.dumpAsText();
//]]>
        </script>
      </head>
      <body>
        Here is an image:
        <img src="../resources/abe.png"/>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>

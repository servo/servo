<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
<xsl:output method="html"/>
<xsl:template match="/">
<html>
  <body>
    <script><![CDATA[
    customElements.define("x-foo", class extends HTMLElement {
      static get observedAttributes() { return ["a"]; }
      attributeChangedCallback() {
        document.getElementById("new-container").append(this);
      }
    });
    ]]></script>
    <div id="new-container" />
    <div>
      <x-foo>
        <xsl:attribute name="a">1</xsl:attribute>
        <xsl:if test="count(document('xslt-reparent-custom-element-crash.xml?1')) = 99">
          <b/>
        </xsl:if>
      </x-foo>
    </div>
  </body>
</html>
</xsl:template>
</xsl:stylesheet>

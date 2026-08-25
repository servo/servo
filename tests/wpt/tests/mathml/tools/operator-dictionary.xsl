<!-- -*- Mode: nXML; tab-width: 2; indent-tabs-mode: nil; -*- -->
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">

  <xsl:strip-space elements="*"/>

  <xsl:template match="charlist">
    <root><xsl:apply-templates select="character"/></root>
  </xsl:template>

  <xsl:template match="character">
    <xsl:if test="operator-dictionary">
      <xsl:for-each select="operator-dictionary">
        <entry>

          <xsl:attribute name="unicode">
            <xsl:value-of select="../@id"/>
          </xsl:attribute>

          <xsl:attribute name="form">
            <xsl:value-of select="@form"/>
          </xsl:attribute>

          <!-- begin operator-dictionary -->
          <xsl:if test="@lspace">
            <xsl:attribute name="lspace">
              <xsl:value-of select="@lspace"/>
            </xsl:attribute>
          </xsl:if>
          <xsl:if test="@rspace">
            <xsl:attribute name="rspace">
            <xsl:value-of select="@rspace"/>
            </xsl:attribute>
          </xsl:if>
          <xsl:if test="@minsize">
            <xsl:attribute name="minsize">
              <xsl:value-of select="@minsize"/>
            </xsl:attribute>
          </xsl:if>
          <xsl:if test="@*[.='true']">
            <xsl:attribute name="properties">
              <!-- largeop, movablelimits, stretchy, separator, accent, fence,
                   symmetric -->
              <xsl:for-each select="@*[.='true']">
                <xsl:value-of select="name()"/>
                <xsl:text> </xsl:text>
              </xsl:for-each>
              <xsl:if test="../unicodedata/@mirror = 'Y'">
                <xsl:text>mirrorable </xsl:text>
              </xsl:if>
            </xsl:attribute>
          </xsl:if>
          <xsl:if test="@priority">
            <xsl:attribute name="priority">
              <xsl:value-of select="@priority"/>
            </xsl:attribute>
          </xsl:if>
          <xsl:if test="@linebreakstyle">
            <xsl:attribute name="linebreakstyle">
              <xsl:value-of select="@linebreakstyle"/>
            </xsl:attribute>
          </xsl:if>
          <!-- end operator-dictionary -->

          <xsl:attribute name="description">
            <xsl:value-of select="../description"/>
          </xsl:attribute>

        </entry>
      </xsl:for-each>
    </xsl:if>
  </xsl:template>

</xsl:stylesheet>

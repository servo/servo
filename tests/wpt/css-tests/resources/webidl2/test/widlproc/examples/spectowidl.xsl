<?xml version="1.0" encoding="utf-8"?>
<!--====================================================================
$Id$
Copyright 2009 Aplix Corporation. All rights reserved.
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

XSLT stylesheet to extract Web IDL snippets from Web IDL spec. 
=====================================================================-->
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
<xsl:output method="text" encoding="utf-8"/>

<xsl:template match="code[@class='idl-code']">
    <xsl:value-of select="."/><xsl:text>
</xsl:text>
</xsl:template>

<xsl:template match="text()"/>

</xsl:stylesheet>

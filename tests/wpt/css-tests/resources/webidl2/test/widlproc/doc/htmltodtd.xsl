<?xml version="1.0" encoding="us-ascii"?>
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

Stylesheet to extract DTD for widlprocxml from widlproc.html
=====================================================================-->
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
<xsl:output method="text" encoding="us-ascii" indent="no"/>

<!-- <pre class="dtd"> element -->
<xsl:template match="pre[@class='dtd']">
    <xsl:value-of select="."/>
</xsl:template>

<!--Ignore other text. -->
<xsl:template match="text()" priority="-100"/>

</xsl:stylesheet>

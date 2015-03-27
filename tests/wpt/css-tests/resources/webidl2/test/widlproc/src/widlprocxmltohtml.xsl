<?xml version="1.0" encoding="utf-8"?>
<!--====================================================================
$Id: widlprocxmltohtml.xsl 407 2009-10-26 13:48:48Z tpr $
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

XSLT stylesheet to convert widlprocxml into html documentation.
=====================================================================-->
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
<xsl:output method="html" encoding="utf-8" indent="yes" doctype-public="html"/>

<xsl:param name="date" select="'error: missing date'"/>

<xsl:variable name="title" select="concat('The ',/Definitions/descriptive/name,' Module - Version ',/Definitions/descriptive/version)"/>

<!--Root of document.-->
<xsl:template match="/">
    <html>
        <head>
            <link rel="stylesheet" type="text/css" href="widlhtml.css" media="screen"/>
            <title>
                <xsl:value-of select="$title"/>
            </title>
        </head>
        <body>
            <xsl:apply-templates/>
        </body>
    </html>
</xsl:template>

<!--Root of Definitions.-->
<xsl:template match="Definitions">
    <div class="api" id="{@id}">
        <a href="http://bondi.omtp.org"><img src="http://www.omtp.org/images/BondiSmall.jpg" alt="Bondi logo"/></a>
        <h1><xsl:value-of select="$title"/></h1>
        <h3>12 May 2009</h3>

        <h2>Authors</h2>
        <ul class="authors">
          <xsl:apply-templates select="descriptive/author"/>
        </ul>

        <p class="copyright"><small>© The authors, 2012. All rights reserved.</small></p>

        <hr/>

        <h2>Abstract</h2>

        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="descriptive/description"/>
        <xsl:apply-templates select="descriptive/Code"/>

        <h2>Table of Contents</h2>
        <ul class="toc">
          <li><a href="#intro">Introduction</a>
          <ul>
            <xsl:if test="descriptive/def-api-feature-set">
              <li><a href="#def-api-feature-sets">Feature set</a></li>
            </xsl:if>
            <xsl:if test="descriptive/def-api-feature">
              <li><a href="#def-api-features">Features</a></li>
            </xsl:if>
            <xsl:if test="descriptive/def-device-cap">
              <li><a href="#def-device-caps">Device Capabilities</a></li>
            </xsl:if>
          </ul>
          </li>
          <xsl:if test="Typedef">
            <li><a href="#typedefs">Type Definitions</a>
            <ul class="toc">
              <xsl:for-each select="Typedef[descriptive]">
                <li><a href="#{@id}"><code><xsl:value-of select="@name"/></code></a></li>
              </xsl:for-each>
            </ul>
            </li>
          </xsl:if>
          <xsl:if test="Interface">
	          <li><a href="#interfaces">Interfaces</a>
	          <ul class="toc">
	          <xsl:for-each select="Interface[descriptive]">
	            <li><a href="#{@id}"><code><xsl:value-of select="@name"/></code></a></li>
	          </xsl:for-each>
	          </ul>
	          </li>
          </xsl:if>
          <xsl:if test="Dictionary">
	          <li><a href="#dictionaries">Dictionary types</a>
	          <ul class="toc">
	          <xsl:for-each select="Dictionary[descriptive]">
	            <li><a href="#{@id}"><code><xsl:value-of select="@name"/></code></a></li>
	          </xsl:for-each>
	          </ul>
	          </li>
          </xsl:if>
          <xsl:if test="Callback">
	          <li><a href="#callbacks">Callbacks</a>
	          <ul class="toc">
	          <xsl:for-each select="Callback[descriptive]">
	            <li><a href="#{@id}"><code><xsl:value-of select="@name"/></code></a></li>
	          </xsl:for-each>
	          </ul>
	          </li>
          </xsl:if>
          <xsl:if test="Enum">
	          <li><a href="#enums">Enums</a>
	          <ul class="toc">
	          <xsl:for-each select="Enum[descriptive]">
	            <li><a href="#{@id}"><code><xsl:value-of select="@name"/></code></a></li>
	          </xsl:for-each>
	          </ul>
	          </li>
          </xsl:if>
        </ul>

        <hr/>

        <h2>Summary of Methods</h2>
        <xsl:call-template name="summary"/>
        
        <h2 id="intro">Introduction</h2>

        <xsl:apply-templates select="descriptive/description"/>
        <xsl:apply-templates select="descriptive/Code"/>

        <xsl:if test="descriptive/def-api-feature-set">
            <div id="def-api-feature-sets" class="def-api-feature-sets">
                <h3 id="features">Feature set</h3>
                <p>This is the URI used to declare this API's feature set, for use in bondi.requestFeature. For the URL, the list of features included by the feature set is provided.</p>
                <xsl:apply-templates select="descriptive/def-api-feature-set"/>
            </div>
        </xsl:if>
        <xsl:if test="descriptive/def-api-feature">
            <div id="def-api-features" class="def-api-features">
                <h3 id="features">Features</h3>
                <p>This is the list of URIs used to declare this API's features, for use in bondi.requestFeature. For each URL, the list of functions covered is provided.</p>
                <xsl:apply-templates select="Interface/descriptive/def-instantiated"/>
                <xsl:apply-templates select="descriptive/def-api-feature"/>
            </div>
        </xsl:if>
        <xsl:if test="descriptive/def-device-cap">
            <div class="def-device-caps" id="def-device-caps">
                <h3>Device capabilities</h3>
                <dl>
                  <xsl:apply-templates select="descriptive/def-device-cap"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:if test="Typedef">
            <div class="typedefs" id="typedefs">
                <h2>Type Definitions</h2>
                <xsl:apply-templates select="Typedef[descriptive]"/>
            </div>
        </xsl:if>
        <xsl:if test="Interface">
            <div class="interfaces" id="interfaces">
		        <h2>Interfaces</h2>
       		    <xsl:apply-templates select="Interface"/>
            </div>
        </xsl:if>
        <xsl:if test="Dictionary">
            <div class="dictionaries" id="dictionaries">
        		<h2>Dictionary types</h2>
        		<xsl:apply-templates select="Dictionary"/>
            </div>
        </xsl:if>
        <xsl:if test="Callback">
            <div class="callbacks" id="callbacks">
        		<h2>Callbacks</h2>
        		<xsl:apply-templates select="Callback"/>
            </div>
        </xsl:if>
        <xsl:if test="Enum">
            <div class="enums" id="enums">
        		<h2>Enums</h2>
        		<xsl:apply-templates select="Enum"/>
            </div>
        </xsl:if>
    </div>
</xsl:template>

<!--def-api-feature-set-->
<xsl:template match="def-api-feature-set">
      <dl class="def-api-feature-set">
          <dt><xsl:value-of select="@identifier"/></dt>
          <dd>
            <xsl:apply-templates select="descriptive/brief"/>
            <xsl:apply-templates select="descriptive"/>
            <xsl:apply-templates select="descriptive/Code"/>
            <xsl:if test="descriptive/api-feature">
              <div class="api-features">
                <p>
                  Includes API features:
                </p>
                <ul>
                  <xsl:for-each select="descriptive/api-feature">
                    <li><code><xsl:value-of select="@identifier"/></code></li>
                  </xsl:for-each>
                </ul>
              </div>
            </xsl:if>
          </dd>
      </dl>
</xsl:template>

<!--def-api-feature-->
<xsl:template match="def-api-feature">
      <dl class="def-api-feature">
          <dt><xsl:value-of select="@identifier"/></dt>
          <dd>
            <xsl:apply-templates select="descriptive/brief"/>
            <xsl:apply-templates select="descriptive"/>
            <xsl:apply-templates select="descriptive/Code"/>
            <xsl:if test="descriptive/device-cap">
              <div class="device-caps">
                <p>
                  Device capabilities:
                </p>
                <ul>
                  <xsl:for-each select="descriptive/device-cap">
                    <li><code><xsl:value-of select="@identifier"/></code></li>
                  </xsl:for-each>
                </ul>
              </div>
            </xsl:if>
          </dd>
      </dl>
</xsl:template>

<!--def-device-cap-->
<xsl:template match="def-device-cap">
    <dt class="def-device-cap"><code><xsl:value-of select="@identifier"/></code></dt>
    <dd>
      <xsl:apply-templates select="descriptive/brief"/>
      <xsl:apply-templates select="descriptive"/>
      <xsl:apply-templates select="descriptive/Code"/>
      <xsl:if test="descriptive/param">
        <div class="device-caps">
          <p>Security parameters:</p>
          <ul>
            <xsl:apply-templates select="descriptive/param"/>
          </ul>
        </div>
      </xsl:if>
    </dd>
</xsl:template>

<!--Exception: not implemented-->
<!--Valuetype: not implemented-->
<xsl:template match="Exception|Valuetype|Const">
    <xsl:if test="descriptive">
        <xsl:message terminate="yes">element <xsl:value-of select="name()"/> not supported</xsl:message>
    </xsl:if>
</xsl:template>

<!--Typedef.-->
<xsl:template match="Typedef[descriptive]">
    <div class="typedef" id="{@id}">
        <h3>2.<xsl:number value="position()"/>. <code><xsl:value-of select="@name"/></code></h3>
        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="webidl"/>
        <xsl:apply-templates select="descriptive"/>
        <xsl:apply-templates select="descriptive/Code"/>
    </div>
</xsl:template>

<!--Interface.-->
<xsl:template match="Interface[descriptive]">
    <xsl:variable name="name" select="@name"/>
    <div class="interface" id="{@id}">
        <h3><code><xsl:value-of select="@name"/></code></h3>
        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="webidl"/>
        <xsl:apply-templates select="../Implements[@name2=$name]/webidl"/>
        <xsl:apply-templates select="descriptive"/>
        <xsl:apply-templates select="descriptive/Code"/>
        <xsl:apply-templates select="InterfaceInheritance"/>
        <xsl:if test="Const/descriptive">
            <div class="consts">
                <h4>Constants</h4>
                <dl>
                  <xsl:apply-templates select="Const"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:if test="ExtendedAttributeList/ExtendedAttribute/descriptive">
            <div class="constructors">
                <h4>Constructors</h4>
                <dl>
                  <xsl:apply-templates select="ExtendedAttributeList/ExtendedAttribute"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:if test="Attribute/descriptive">
            <div class="attributes">
                <h4>Attributes</h4>
                <dl>
                  <xsl:apply-templates select="Attribute"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:if test="Operation/descriptive">
            <div class="methods">
                <h4>Methods</h4>
                <dl>
                  <xsl:apply-templates select="Operation"/>
                </dl>
            </div>
        </xsl:if>
    </div>
</xsl:template>
<xsl:template match="Interface[not(descriptive)]">
</xsl:template>

<!--Dictionary.-->
<xsl:template match="Dictionary[descriptive]">
    <xsl:variable name="name" select="@name"/>
    <div class="dictionary" id="{@id}">
        <h3><code><xsl:value-of select="@name"/></code></h3>
        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="webidl"/>
        <xsl:apply-templates select="descriptive"/>
        <xsl:apply-templates select="descriptive/Code"/>
        <xsl:apply-templates select="InterfaceInheritance"/>
        <xsl:if test="Const/descriptive">
            <div class="consts">
                <h4>Constants</h4>
                <dl>
                  <xsl:apply-templates select="Const"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:if test="Attribute/descriptive">
            <div class="attributes">
                <h4>Attributes</h4>
                <dl>
                  <xsl:apply-templates select="Attribute"/>
                </dl>
            </div>
        </xsl:if>
    </div>
</xsl:template>
<xsl:template match="Dictionary[not(descriptive)]">
</xsl:template>

<xsl:template match="InterfaceInheritance/ScopedNameList">
              <p>
                <xsl:text>This interface inherits from: </xsl:text>
                <xsl:for-each select="Name">
                  <code><xsl:value-of select="@name"/></code>
                  <xsl:if test="position!=last()">, </xsl:if>
                </xsl:for-each>
              </p>
</xsl:template>

<!--Attribute-->
<xsl:template match="Attribute">
    <dt class="attribute" id="{@name}">
        <code>
            <xsl:if test="@stringifier">
                stringifier
            </xsl:if>
            <xsl:if test="@readonly">
                readonly
            </xsl:if>
            <xsl:apply-templates select="Type"/>
            <xsl:text> </xsl:text>
            <xsl:value-of select="@name"/>
        </code></dt>
        <dd>
          <xsl:apply-templates select="descriptive/brief"/>
          <xsl:apply-templates select="descriptive"/>
          <xsl:apply-templates select="GetRaises"/>
          <xsl:apply-templates select="SetRaises"/>
          <xsl:apply-templates select="descriptive/Code"/>
        </dd>
</xsl:template>

<!--Const-->
<xsl:template match="Const">
  <dt class="const" id="{@id}">
    <code>
      <xsl:apply-templates select="Type"/>
      <xsl:text> </xsl:text>
      <xsl:value-of select="@name"/>
    </code>
  </dt>
  <dd>
    <xsl:apply-templates select="descriptive/brief"/>
    <xsl:apply-templates select="descriptive"/>
    <xsl:apply-templates select="descriptive/Code"/>
  </dd>
</xsl:template>

<!--ExtendedAttribute name==Constructor || name==NamedConstructor-->
<xsl:template match="ExtendedAttributeList/ExtendedAttribute">
    <dt class="constructor" id="{concat(@name,generate-id(.))}">
        <code>
            <xsl:value-of select="../../@name"/>
             <xsl:text>(</xsl:text>
            <xsl:apply-templates select="ArgumentList">
                <xsl:with-param name="nodesc" select="1"/>
            </xsl:apply-templates>
            <xsl:text>);</xsl:text>
        </code>
    </dt>
    <dd>
        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="descriptive"/>
        <xsl:apply-templates select="ArgumentList"/>
        <xsl:apply-templates select="Raises"/>
        <xsl:if test="descriptive/api-feature">
            <div class="api-features">
                <h6>API features</h6>
                <dl>
                    <xsl:apply-templates select="descriptive/api-feature"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:apply-templates select="webidl"/>
        <xsl:apply-templates select="descriptive/Code"/>
    </dd>
</xsl:template>

<!--Operation-->
<xsl:template match="Operation">
    <dt class="method" id="{concat(@name,generate-id(.))}">
        <code>
            <xsl:if test="@stringifier">
                <xsl:value-of select="concat(@stringifier, ' ')"/>
            </xsl:if>
            <xsl:if test="@omittable">
                <xsl:value-of select="concat(@omittable, ' ')"/>
            </xsl:if>
            <xsl:if test="@getter">
                <xsl:value-of select="concat(@getter, ' ')"/>
            </xsl:if>
            <xsl:if test="@setter">
                <xsl:value-of select="concat(@setter, ' ')"/>
            </xsl:if>
            <xsl:if test="@creator">
            	<xsl:value-of select="concat(@creator, ' ')"/>
            </xsl:if>
            <xsl:if test="@deleter">
                <xsl:value-of select="concat(@deleter, ' ')"/>
            </xsl:if>
            <xsl:if test="@caller">
                <xsl:value-of select="concat(@caller, ' ')"/>
            </xsl:if>
            <xsl:apply-templates select="Type"/>
            <xsl:text> </xsl:text>
            <xsl:value-of select="@name"/>
            <xsl:text>(</xsl:text>
            <xsl:apply-templates select="ArgumentList">
                <xsl:with-param name="nodesc" select="1"/>
            </xsl:apply-templates>
            <xsl:text>);</xsl:text>
        </code>
    </dt>
    <dd>
        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="webidl"/>
        <xsl:apply-templates select="descriptive/Code"/>
        <xsl:apply-templates select="descriptive"/>
        <xsl:apply-templates select="ArgumentList"/>
        <xsl:if test="Type/descriptive">
          <div class="returntype">
            <h5>Return value</h5>
            <xsl:apply-templates select="Type/descriptive"/>
          </div>
        </xsl:if>
        <xsl:apply-templates select="Raises"/>
        <xsl:if test="descriptive/api-feature">
            <div class="api-features">
                <h6>API features</h6>
                <dl>
                    <xsl:apply-templates select="descriptive/api-feature"/>
                </dl>
            </div>
        </xsl:if>
        <xsl:apply-templates select="descriptive/Code"/>
    </dd>
</xsl:template>

<!--Callback-->
<xsl:template match="Callback">
    <xsl:variable name="name" select="@name"/>
    <div class="callback" id="{@id}">
        <h3><code><xsl:value-of select="@name"/></code></h3>

	    <dd>
	        <xsl:apply-templates select="descriptive/brief"/>
	        <xsl:apply-templates select="webidl"/>
	        <xsl:apply-templates select="descriptive"/>
	        <div class="synopsis">
	            <h6>Signature</h6>
	            <pre>
	                <xsl:apply-templates select="Type"/>
	                <xsl:text> </xsl:text>
	                <xsl:value-of select="@name"/>
	                <xsl:text>(</xsl:text>
	                <xsl:apply-templates select="ArgumentList">
	                    <xsl:with-param name="nodesc" select="1"/>
	                </xsl:apply-templates>
	                <xsl:text>);
</xsl:text></pre>
	        </div>
	        <xsl:apply-templates select="descriptive"/>
	        <xsl:apply-templates select="ArgumentList"/>
	        <xsl:if test="Type/descriptive">
	          <div class="returntype">
	            <h5>Return value</h5>
	            <xsl:apply-templates select="Type/descriptive"/>
	          </div>
	        </xsl:if>
	        <xsl:apply-templates select="descriptive/Code"/>
	    </dd>
	</div>
</xsl:template>

<!--ArgumentList. This is passed $nodesc=true to output just the argument
    types and names, and not any documentation for them.-->
<xsl:template match="ArgumentList">
    <xsl:param name="nodesc"/>
    <xsl:choose>
        <xsl:when test="$nodesc">
            <!--$nodesc is true: just output the types and names-->
            <xsl:apply-templates select="Argument[1]">
                <xsl:with-param name="nodesc" select="'nocomma'"/>
            </xsl:apply-templates>
            <xsl:apply-templates select="Argument[position() != 1]">
                <xsl:with-param name="nodesc" select="'comma'"/>
            </xsl:apply-templates>
        </xsl:when>
        <xsl:when test="Argument">
            <!--$nodesc is false: output the documentation-->
            <div class="parameters">
                <h6>Parameters</h6>
                <ul>
                    <xsl:apply-templates/>
                </ul>
            </div>
        </xsl:when>
    </xsl:choose>
</xsl:template>

<!--Argument. This is passed $nodesc=false to output the documentation,
    or $nodesc="nocomma" to output the type and name, or $nodesc="comma"
    to output a comma then the type and name. -->
<xsl:template match="Argument">
    <xsl:param name="nodesc"/>
    <xsl:choose>
        <xsl:when test="$nodesc">
            <!--$nodesc is true: just output the types and names-->
            <xsl:if test="$nodesc = 'comma'">
                <!--Need a comma first.-->
                <xsl:text>, </xsl:text>
            </xsl:if>
            <xsl:if test="@in"><xsl:value-of select="concat(@in, ' ')"/></xsl:if>
            <xsl:if test="@optional"><xsl:value-of select="concat(@optional, ' ')"/></xsl:if>
            <xsl:apply-templates select="Type"/>
            <xsl:if test="@ellipsis"><xsl:text>...</xsl:text></xsl:if>
            <xsl:text> </xsl:text>
            <xsl:value-of select="@name"/>
	    <xsl:if test="@value">
	      <xsl:text>Default value: </xsl:text><xsl:value-of select="@value"/>
	    </xsl:if>
	    <xsl:if test="@stringvalue">
	      <xsl:text>Default value: "</xsl:text><xsl:value-of select="@stringvalue"/><xsl:text>"</xsl:text>
	    </xsl:if>	    
        </xsl:when>
        <xsl:otherwise>
            <!--$nodesc is false: output the documentation-->
            <li class="param">
                <xsl:value-of select="@name"/>:
                <xsl:apply-templates select="descriptive"/>
            </li>
        </xsl:otherwise>
    </xsl:choose>
</xsl:template>

<!--Raises (for an Operation). It is already known that the list
    is not empty.-->
<xsl:template match="Raises">
    <div class="exceptionlist">
        <h5>Exceptions</h5>
        <ul>
            <xsl:apply-templates/>
        </ul>
    </div>
</xsl:template>

<!--RaiseException, the name of an exception in a Raises.-->
<xsl:template match="RaiseException">
    <li class="exception">
        <xsl:value-of select="@name"/>:
        <xsl:apply-templates select="descriptive"/>
    </li>
</xsl:template>

<!--Type.-->
<xsl:template match="Type">
    <xsl:choose>
        <xsl:when test="Type">
            <xsl:text>sequence &lt;</xsl:text>
            <xsl:apply-templates/>
            <xsl:text>></xsl:text>
        </xsl:when>
        <xsl:otherwise>
            <xsl:value-of select="@name"/>
            <xsl:value-of select="@type"/>
            <xsl:if test="@nullable">
                <xsl:text>?</xsl:text>
            </xsl:if>
        </xsl:otherwise>
    </xsl:choose>
</xsl:template>

<!--Enum.-->
<xsl:template match="Enum[descriptive]">
    <xsl:variable name="name" select="@name"/>
    <div class="enum" id="{@id}">
        <h3><code><xsl:value-of select="@name"/></code></h3>
        <xsl:apply-templates select="descriptive/brief"/>
        <xsl:apply-templates select="webidl"/>
        <xsl:apply-templates select="descriptive"/>
        <xsl:apply-templates select="descriptive/Code"/>
        <div class="enumvalues">
            <h4>Values</h4>
            <dl>
              <xsl:apply-templates select="EnumValue"/>
            </dl>
        </div>
    </div>
</xsl:template>
<xsl:template match="Enum[not(descriptive)]">
</xsl:template>

<!--EnumValue-->
<xsl:template match="EnumValue">
  <dt class="enumvalue" id="{@id}">
    <code>
      <xsl:value-of select="@stringvalue"/>
    </code>
  </dt>
  <dd>
    <xsl:apply-templates select="descriptive/brief"/>
    <xsl:apply-templates select="descriptive"/>
    <xsl:apply-templates select="descriptive/Code"/>
  </dd>
</xsl:template>

<xsl:template match="descriptive[not(author)]">
  <xsl:apply-templates select="version"/>
  <xsl:if test="author">
  </xsl:if>
  <xsl:apply-templates select="description"/>
</xsl:template>

<!--brief-->
<xsl:template match="brief">
    <div class="brief">
        <p>
            <xsl:apply-templates/>
        </p>
    </div>
</xsl:template>

<!--description in ReturnType or Argument or ScopedName-->
<xsl:template match="Type/descriptive/description|Argument/descriptive/description|Name/descriptive/description">
    <!--If the description contains just a single <p> then we omit
        the <p> and just do its contents.-->
    <xsl:choose>
        <xsl:when test="p and count(*) = 1">
            <xsl:apply-templates select="p/*|p/text()"/>
        </xsl:when>
        <xsl:otherwise>
            <div class="description">
                <xsl:apply-templates/>
            </div>
        </xsl:otherwise>
    </xsl:choose>
</xsl:template>

<!--Other description-->
<xsl:template match="description">
    <div class="description">
        <xsl:apply-templates/>
    </div>
</xsl:template>

<!--Code-->
<xsl:template match="Code">
    <div class="example">
    	<xsl:choose>
        	<xsl:when test="@lang">
	       		<h5><xsl:value-of select="@lang"/></h5>
        	</xsl:when>
        	<xsl:otherwise>
	       		<h5>Code example</h5>
        	</xsl:otherwise>
    	</xsl:choose>
        <pre class="examplecode"><xsl:apply-templates/></pre>
    </div>
</xsl:template>

<!--webidl : literal Web IDL from input-->
<xsl:template match="webidl">
    <h5>WebIDL</h5>
    <pre class="webidl"><xsl:apply-templates/></pre>
</xsl:template>

<!--author-->
<xsl:template match="author">
    <li class="author"><xsl:apply-templates/></li>
</xsl:template>

<!--version-->
<xsl:template match="version">
    <div class="version">
        <h2>
            Version: <xsl:apply-templates/>
        </h2>
    </div>
</xsl:template>

<!--api-feature-->
<xsl:template match="api-feature">
    <dt>
        <xsl:value-of select="@identifier"/>
    </dt>
    <dd>
        <xsl:apply-templates/>
    </dd>
</xsl:template>

<!--param-->
<xsl:template match="param">
    <li>
        <code><xsl:value-of select="@identifier"/></code>:
        <xsl:apply-templates/>
    </li>
</xsl:template>

<!--def-instantiated.
    This assumes that only one interface in the module has a def-instantiated,
    and that interface contains just one attribute.-->
<xsl:template match="def-instantiated">
    <xsl:variable name="ifacename" select="../../@name"/>
    <p>
        <xsl:choose>
            <xsl:when test="count(descriptive/api-feature)=1">
                When the feature
            </xsl:when>
            <xsl:otherwise>
                When any of the features
            </xsl:otherwise>
        </xsl:choose>
    </p>
    <ul>
        <xsl:for-each select="descriptive/api-feature">
            <li><code>
                <xsl:value-of select="@identifier"/>
            </code></li>
        </xsl:for-each>
    </ul>
    <p>
        is successfully requested, the interface
        <code><xsl:apply-templates select="../../Attribute/Type"/></code>
        is instantiated, and the resulting object appears in the global
        namespace as
        <code><xsl:value-of select="../../../Implements[@name2=$ifacename]/@name1"/>.<xsl:value-of select="../../Attribute/@name"/></code>.
    </p>
</xsl:template>



<!--html elements-->
<xsl:template match="a|b|br|dd|dl|dt|em|li|p|table|td|th|tr|ul">
    <xsl:element name="{name()}"><xsl:for-each select="@*"><xsl:attribute name="{name()}"><xsl:value-of select="."/></xsl:attribute></xsl:for-each><xsl:apply-templates/></xsl:element>
</xsl:template>

<xsl:template name="summary">
  <table class="summary">
    <thead>
      <tr><th>Interface</th><th>Method</th></tr>
    </thead>
    <tbody>
      <xsl:for-each select="Interface[descriptive]">
        <tr><td><a href="#{@id}"><xsl:value-of select="@name"/></a></td>
        <td>
          <xsl:for-each select="Operation">

            <xsl:apply-templates select="Type"/>
            <xsl:text> </xsl:text>
            <a href="#{concat(@name,generate-id(.))}"><xsl:value-of select="@name"/></a>
            <xsl:text>(</xsl:text>
            <xsl:for-each select="ArgumentList/Argument">
              <xsl:variable name="type"><xsl:apply-templates select="Type"/></xsl:variable>
              <xsl:value-of select="concat(normalize-space($type),' ',@name)"/>
              <xsl:if test="position() != last()">, </xsl:if>
            </xsl:for-each>
            <xsl:text>)</xsl:text>
            <xsl:if test="position()!=last()"><br/></xsl:if>
          </xsl:for-each>
        </td>
        </tr>
      </xsl:for-each>
    </tbody>
  </table>
</xsl:template>

<!--<ref> element in literal Web IDL.-->
<xsl:template match="ref[@ref]">
    <a href="{@ref}">
        <xsl:apply-templates/>
    </a>
</xsl:template>

</xsl:stylesheet>


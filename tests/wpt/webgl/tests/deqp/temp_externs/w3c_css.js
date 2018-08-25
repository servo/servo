/*
 * Copyright 2008 The Closure Compiler Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @fileoverview Definitions for W3C's CSS specification
 *  The whole file has been fully type annotated.
 *  http://www.w3.org/TR/DOM-Level-2-Style/css.html
 * @externs
 * @author stevey@google.com (Steve Yegge)
 *
 * TODO(nicksantos): When there are no more occurrences of w3c_range.js and
 * gecko_dom.js being included directly in BUILD files, bug dbeam to split the
 * bottom part of this file into a separate externs.
 */

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet
 */
function StyleSheet() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-type
 */
StyleSheet.prototype.type;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-disabled
 */
StyleSheet.prototype.disabled;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-owner
 */
StyleSheet.prototype.ownerNode;

/**
 * @type {StyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-parentStyleSheet
 */
StyleSheet.prototype.parentStyleSheet;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-href
 */
StyleSheet.prototype.href;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-title
 */
StyleSheet.prototype.title;

/**
 * @type {MediaList}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-media
 */
StyleSheet.prototype.media;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheetList
 */
function StyleSheetList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheetList-length
 */
StyleSheetList.prototype.length;

/**
 * @param {number} index
 * @return {StyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheetList-item
 */
StyleSheetList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-MediaList
 */
function MediaList() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-MediaList-mediaText
 */
MediaList.prototype.mediaText;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-MediaList-length
 */
MediaList.prototype.length;

/**
 * @param {number} index
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-MediaList-item
 */
MediaList.prototype.item = function(index) {};

/**
 * @interface
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-LinkStyle
 */
function LinkStyle() {}

/**
 * @type {StyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-LinkStyle-sheet
 */
LinkStyle.prototype.sheet;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-DocumentStyle
 */
function DocumentStyle() {}

/**
 * @type {StyleSheetList}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/stylesheets.html#StyleSheets-StyleSheet-DocumentStyle-styleSheets
 */
DocumentStyle.prototype.styleSheets;

/**
 * @constructor
 * @extends {StyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleSheet
 */
function CSSStyleSheet() {}

/**
 * @type {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleSheet-ownerRule
 */
CSSStyleSheet.prototype.ownerRule;

/**
 * @type {CSSRuleList}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleSheet-cssRules
 */
CSSStyleSheet.prototype.cssRules;

/**
 * @param {string} rule
 * @param {number} index
 * @return {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleSheet-insertRule
 */
CSSStyleSheet.prototype.insertRule = function(rule, index) {};

/**
 * @param {number} index
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleSheet-deleteRule
 */
CSSStyleSheet.prototype.deleteRule = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRuleList
 */
function CSSRuleList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRuleList-length
 */
CSSRuleList.prototype.length;

/**
 * @param {number} index
 * @return {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRuleList-item
 */
CSSRuleList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule
 */
function CSSRule() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-cssText
 */
CSSRule.prototype.cssText;

/**
 * @type {CSSStyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-sheet
 */
CSSRule.prototype.parentStyleSheet;

/**
 * @type {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-parentRule
 */
CSSRule.prototype.parentRule;

/**
 * @type {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleRule
 */
CSSRule.prototype.style;

/**
 * Indicates that the rule is a {@see CSSUnknownRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.UNKNOWN_RULE = 0;

/**
 * Indicates that the rule is a {@see CSSStyleRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.STYLE_RULE = 1;

/**
 * Indicates that the rule is a {@see CSSCharsetRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.CHARSET_RULE = 2;

/**
 * Indicates that the rule is a {@see CSSImportRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.IMPORT_RULE = 3;

/**
 * Indicates that the rule is a {@see CSSMediaRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.MEDIA_RULE = 4;

/**
 * Indicates that the rule is a {@see CSSFontFaceRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.FONT_FACE_RULE = 5;

/**
 * Indicates that the rule is a {@see CSSPageRule}.
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSRule-ruleType
 */
CSSRule.PAGE_RULE = 6;

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleRule
 */
function CSSStyleRule() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleRule-selectorText
 */
CSSStyleRule.prototype.selectorText;

/**
 * @type {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleRule-style
 */
CSSStyleRule.prototype.style;

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSMediaRule
 */
function CSSMediaRule() {}

/**
 * @type {MediaList}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSMediaRule-mediaTypes
 */
CSSMediaRule.prototype.media;

/**
 * @type {CSSRuleList}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSMediaRule-cssRules
 */
CSSMediaRule.prototype.cssRules;

/**
 * @param {string} rule
 * @param {number} index
 * @return {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSMediaRule-insertRule
 */
CSSMediaRule.prototype.insertRule = function(rule, index) {};

/**
 * @param {number} index
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSMediaRule-deleteRule
 */
CSSMediaRule.prototype.deleteRule = function(index) {};

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSFontFaceRule
 */
function CSSFontFaceRule() {}

/**
 * @type {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSFontFaceRule-style
 */
CSSFontFaceRule.prototype.style;

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPageRule
 */
function CSSPageRule() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPageRule-name
 */
CSSPageRule.prototype.selectorText;

/**
 * @type {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPageRule-style
 */
CSSPageRule.prototype.style;

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSImportRule
 */
function CSSImportRule() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSImportRule-href
 */
CSSImportRule.prototype.href;

/**
 * @type {MediaList}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSImportRule-media
 */
CSSImportRule.prototype.media;

/**
 * @type {CSSStyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSImportRule-styleSheet
 */
CSSImportRule.prototype.styleSheet;

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSCharsetRule
 */
function CSSCharsetRule() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSCharsetRule-encoding
 */
CSSCharsetRule.prototype.encoding;

/**
 * @constructor
 * @extends {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSUnknownRule
 */
function CSSUnknownRule() {}

/**
 * @constructor
 * @extends {CSSProperties}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration
 */
function CSSStyleDeclaration() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-cssText
 */
CSSStyleDeclaration.prototype.cssText;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-length
 */
CSSStyleDeclaration.prototype.length;

/**
 * @type {CSSRule}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-parentRule
 */
CSSStyleDeclaration.prototype.parentRule;

/**
 * @param {string} propertyName
 * @return {CSSValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-getPropertyCSSValue
 */
CSSStyleDeclaration.prototype.getPropertyCSSValue = function(propertyName) {};

/**
 * @param {string} propertyName
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-getPropertyPriority
 */
CSSStyleDeclaration.prototype.getPropertyPriority = function(propertyName) {};

/**
 * @param {string} propertyName
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-getPropertyValue
 */
CSSStyleDeclaration.prototype.getPropertyValue = function(propertyName) {};

/**
 * @param {number} index
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-item
 */
CSSStyleDeclaration.prototype.item = function(index) {};

/**
 * @param {string} propertyName
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-removeProperty
 */
CSSStyleDeclaration.prototype.removeProperty = function(propertyName) {};

/**
 * @param {string} propertyName
 * @param {string} value
 * @param {string=} opt_priority
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSStyleDeclaration-setProperty
 */
CSSStyleDeclaration.prototype.setProperty = function(propertyName, value, opt_priority) {};

// IE-specific

/**
 * @param {string} name
 * @param {number=} opt_flags
 * @return {string|number|boolean|null}
 * @see http://msdn.microsoft.com/en-us/library/ms536429(VS.85).aspx
 */
CSSStyleDeclaration.prototype.getAttribute = function(name, opt_flags) {};

/**
 * @param {string} name
 * @return {string|number|boolean|null}
 * @see http://msdn.microsoft.com/en-us/library/aa358797(VS.85).aspx
 */
CSSStyleDeclaration.prototype.getExpression = function(name) {};

/**
 * @param {string} name
 * @param {number=} opt_flags
 * @return {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms536696(VS.85).aspx
 */
CSSStyleDeclaration.prototype.removeAttribute =
    function(name, opt_flags) {};

/**
 * @param {string} name
 * @return {boolean}
 * @see http://msdn.microsoft.com/en-us/library/aa358798(VS.85).aspx
 */
CSSStyleDeclaration.prototype.removeExpression = function(name) {};

/**
 * @param {string} name
 * @param {*} value
 * @param {number=} opt_flags
 * @see http://msdn.microsoft.com/en-us/library/ms536739(VS.85).aspx
 */
CSSStyleDeclaration.prototype.setAttribute = function(name, value, opt_flags) {};

/**
 * @param {string} name
 * @param {string} expr
 * @param {string=} opt_language
 * @return {undefined}
 * @see http://msdn.microsoft.com/en-us/library/ms531196(VS.85).aspx
 */
CSSStyleDeclaration.prototype.setExpression =
    function(name, expr, opt_language) {};


/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue
 */
function CSSValue() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue-cssText
 */
CSSValue.prototype.cssText;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue-cssValueType
 */
CSSValue.prototype.cssValueType;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue-types
 */
CSSValue.CSS_INHERIT = 0;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue-types
 */
CSSValue.CSS_PRIMITIVE_VALUE = 1;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue-types
 */
CSSValue.CSS_VALUE_LIST = 2;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValue-types
 */
CSSValue.CSS_CUSTOM = 3;

/**
 * @constructor
 * @extends {CSSValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
function CSSPrimitiveValue() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.prototype.primitiveType;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_UNKNOWN = 0;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_NUMBER = 1;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_PERCENTAGE = 2;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_EMS = 3;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_EXS = 4;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_PX = 5;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_CM = 6;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_MM = 7;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_IN = 8;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_PT = 9;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_PC = 10;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_DEG = 11;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_RAD = 12;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_GRAD = 13;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_MS = 14;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_S = 15;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_HZ = 16;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_KHZ = 17;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_DIMENSION = 18;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_STRING = 19;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_URI = 20;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_IDENT = 21;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_ATTR = 22;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_COUNTER = 23;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_RECT = 24;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue
 */
CSSPrimitiveValue.CSS_RGBCOLOR = 25;

/**
 * @return {Counter}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-getCounterValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR}
 */
CSSPrimitiveValue.prototype.getCounterValue = function() {};

/**
 * @param {number} unitType
 * @return {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-getFloatValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR}
 */
CSSPrimitiveValue.prototype.getFloatValue = function(unitType) {};

/**
 * @return {RGBColor}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-getRGBColorValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR}
 */
CSSPrimitiveValue.prototype.getRGBColorValue = function() {};

/**
 * @return {Rect}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-getRectValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR}
 */
CSSPrimitiveValue.prototype.getRectValue = function() {};

/**
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-getStringValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR}
 */
CSSPrimitiveValue.prototype.getStringValue = function() {};

/**
 * @param {number} unitType
 * @param {number} floatValue
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-setFloatValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR},
 *                      {@see DomException.NO_MODIFICATION_ALLOWED_ERR}
 */
CSSPrimitiveValue.prototype.setFloatValue = function(unitType, floatValue) {};

/**
 * @param {number} stringType
 * @param {string} stringValue
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-setStringValue
 * @throws DOMException {@see DomException.INVALID_ACCESS_ERR},
 *                      {@see DomException.NO_MODIFICATION_ALLOWED_ERR}
 */
CSSPrimitiveValue.prototype.setStringValue = function(stringType, stringValue) {};

/**
 * @constructor
 * @extends {CSSValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValueList
 */
function CSSValueList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValueList-length
 */
CSSValueList.prototype.length;

/**
 * @param {number} index
 * @return {CSSValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSValueList-item
 */
CSSValueList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-RGBColor
 */
function RGBColor() {}

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-RGBColor-red
 */
RGBColor.prototype.red;

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-RGBColor-green
 */
RGBColor.prototype.green;

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-RGBColor-blue
 */
RGBColor.prototype.blue;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Rect
 */
function Rect() {}

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Rect-top
 */
Rect.prototype.top;

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Rect-right
 */
Rect.prototype.right;

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Rect-bottom
 */
Rect.prototype.bottom;

/**
 * @type {CSSPrimitiveValue}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Rect-left
 */
Rect.prototype.left;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Counter
 */
function Counter() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Counter-identifier
 */
Counter.prototype.identifier;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Counter-listStyle
 */
Counter.prototype.listStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-Counter-separator
 */
Counter.prototype.separator;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-ViewCSS
 */
function ViewCSS() {}

/**
 * @param {Element} elt
 * @param {?string=} opt_pseudoElt This argument is required according to the
 *     CSS2 specification, but optional in all major browsers. See the note at
 *     https://developer.mozilla.org/en-US/docs/Web/API/Window.getComputedStyle
 * @return {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSview-getComputedStyle
 */
ViewCSS.prototype.getComputedStyle = function(elt, opt_pseudoElt) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-DocumentCSS
 */
function DocumentCSS() {}

/**
 * @param {Element} elt
 * @param {string} pseudoElt
 * @return {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-DocumentCSS-getOverrideStyle
 */
DocumentCSS.prototype.getOverrideStyle = function(elt, pseudoElt) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-DOMImplementationCSS
 */
function DOMImplementationCSS() {}

/**
 * @param {string} title
 * @param {string} media
 * @return {CSSStyleSheet}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-DOMImplementationCSS-createCSSStyleSheet
 * @throws DOMException {@see DomException.SYNTAX_ERR}
 */
DOMImplementationCSS.prototype.createCSSStyleSheet = function(title, media) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-ElementCSSInlineStyle
 */
function ElementCSSInlineStyle() {}

/**
 * @type {CSSStyleDeclaration}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-ElementCSSInlineStyle-style
 */
ElementCSSInlineStyle.prototype.style;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties
 */
function CSSProperties() {}

// CSS 2 properties

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-azimuth
 */
CSSProperties.prototype.azimuth;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-background
 */
CSSProperties.prototype.background;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-backgroundAttachment
 */
CSSProperties.prototype.backgroundAttachment;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-backgroundColor
 */
CSSProperties.prototype.backgroundColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-backgroundImage
 */
CSSProperties.prototype.backgroundImage;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-backgroundPosition
 */
CSSProperties.prototype.backgroundPosition;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-backgroundRepeat
 */
CSSProperties.prototype.backgroundRepeat;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-background/#the-background-size
 */
CSSProperties.prototype.backgroundSize;

/**
 * @implicitCast
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-border
 */
CSSProperties.prototype.border;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderCollapse
 */
CSSProperties.prototype.borderCollapse;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderColor
 */
CSSProperties.prototype.borderColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderSpacing
 */
CSSProperties.prototype.borderSpacing;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSPrimitiveValue-borderStyle
 */
CSSProperties.prototype.borderStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderTop
 */
CSSProperties.prototype.borderTop;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderRight
 */
CSSProperties.prototype.borderRight;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderBottom
 */
CSSProperties.prototype.borderBottom;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderLeft
 */
CSSProperties.prototype.borderLeft;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderTopColor
 */
CSSProperties.prototype.borderTopColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderRightColor
 */
CSSProperties.prototype.borderRightColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderBottomColor
 */
CSSProperties.prototype.borderBottomColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderLeftColor
 */
CSSProperties.prototype.borderLeftColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderTopStyle
 */
CSSProperties.prototype.borderTopStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderRightStyle
 */
CSSProperties.prototype.borderRightStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderBottomStyle
 */
CSSProperties.prototype.borderBottomStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderLeftStyle
 */
CSSProperties.prototype.borderLeftStyle;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderTopWidth
 */
CSSProperties.prototype.borderTopWidth;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderRightWidth
 */
CSSProperties.prototype.borderRightWidth;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderBottomWidth
 */
CSSProperties.prototype.borderBottomWidth;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderLeftWidth
 */
CSSProperties.prototype.borderLeftWidth;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-borderWidth
 */
CSSProperties.prototype.borderWidth;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-background/#the-border-radius
 */
CSSProperties.prototype.borderRadius;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-background/#the-border-radius
 */
CSSProperties.prototype.borderBottomLeftRadius;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-background/#the-border-radius
 */
CSSProperties.prototype.borderBottomRightRadius;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-background/#the-border-radius
 */
CSSProperties.prototype.borderTopLeftRadius;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-background/#the-border-radius
 */
CSSProperties.prototype.borderTopRightRadius;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-bottom
 */
CSSProperties.prototype.bottom;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-captionSide
 */
CSSProperties.prototype.captionSide;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-clear
 */
CSSProperties.prototype.clear;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-clip
 */
CSSProperties.prototype.clip;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-color
 */
CSSProperties.prototype.color;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-content
 */
CSSProperties.prototype.content;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-counterIncrement
 */
CSSProperties.prototype.counterIncrement;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-counterReset
 */
CSSProperties.prototype.counterReset;

/**
 * This is not an official part of the W3C spec. In practice, this is a settable
 * property that works cross-browser. It is used in goog.dom.setProperties() and
 * needs to be extern'd so the --disambiguate_properties JS compiler pass works.
 * @type {string}
 */
CSSProperties.prototype.cssText;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-cue
 */
CSSProperties.prototype.cue;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-cueAfter
 */
CSSProperties.prototype.cueAfter;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-cueBefore
 */
CSSProperties.prototype.cueBefore;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-cursor
 */
CSSProperties.prototype.cursor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-direction
 */
CSSProperties.prototype.direction;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-display
 */
CSSProperties.prototype.display;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-elevation
 */
CSSProperties.prototype.elevation;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-emptyCells
 */
CSSProperties.prototype.emptyCells;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-cssFloat
 */
CSSProperties.prototype.cssFloat;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-font
 */
CSSProperties.prototype.font;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontFamily
 */
CSSProperties.prototype.fontFamily;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontSize
 */
CSSProperties.prototype.fontSize;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontSizeAdjust
 */
CSSProperties.prototype.fontSizeAdjust;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontStretch
 */
CSSProperties.prototype.fontStretch;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontStyle
 */
CSSProperties.prototype.fontStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontVariant
 */
CSSProperties.prototype.fontVariant;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-fontWeight
 */
CSSProperties.prototype.fontWeight;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-height
 */
CSSProperties.prototype.height;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-left
 */
CSSProperties.prototype.left;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-letterSpacing
 */
CSSProperties.prototype.letterSpacing;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-lineHeight
 */
CSSProperties.prototype.lineHeight;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-listStyle
 */
CSSProperties.prototype.listStyle;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-listStyleImage
 */
CSSProperties.prototype.listStyleImage;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-listStylePosition
 */
CSSProperties.prototype.listStylePosition;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-listStyleType
 */
CSSProperties.prototype.listStyleType;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-margin
 */
CSSProperties.prototype.margin;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-marginTop
 */
CSSProperties.prototype.marginTop;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-marginRight
 */
CSSProperties.prototype.marginRight;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-marginBottom
 */
CSSProperties.prototype.marginBottom;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-marginLeft
 */
CSSProperties.prototype.marginLeft;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-markerOffset
 */
CSSProperties.prototype.markerOffset;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-marks
 */
CSSProperties.prototype.marks;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-maxHeight
 */
CSSProperties.prototype.maxHeight;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-maxWidth
 */
CSSProperties.prototype.maxWidth;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-minHeight
 */
CSSProperties.prototype.minHeight;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-minWidth
 */
CSSProperties.prototype.minWidth;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-orphans
 */
CSSProperties.prototype.orphans;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-outline
 */
CSSProperties.prototype.outline;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-outlineColor
 */
CSSProperties.prototype.outlineColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-outlineStyle
 */
CSSProperties.prototype.outlineStyle;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-outlineWidth
 */
CSSProperties.prototype.outlineWidth;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-overflow
 */
CSSProperties.prototype.overflow;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-padding
 */
CSSProperties.prototype.padding;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-paddingTop
 */
CSSProperties.prototype.paddingTop;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-paddingRight
 */
CSSProperties.prototype.paddingRight;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-paddingBottom
 */
CSSProperties.prototype.paddingBottom;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-paddingLeft
 */
CSSProperties.prototype.paddingLeft;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-page
 */
CSSProperties.prototype.page;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pageBreakAfter
 */
CSSProperties.prototype.pageBreakAfter;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pageBreakBefore
 */
CSSProperties.prototype.pageBreakBefore;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pageBreakInside
 */
CSSProperties.prototype.pageBreakInside;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pause
 */
CSSProperties.prototype.pause;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pauseAfter
 */
CSSProperties.prototype.pauseAfter;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pauseBefore
 */
CSSProperties.prototype.pauseBefore;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pitch
 */
CSSProperties.prototype.pitch;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-pitchRange
 */
CSSProperties.prototype.pitchRange;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-playDuring
 */
CSSProperties.prototype.playDuring;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-position
 */
CSSProperties.prototype.position;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-quotes
 */
CSSProperties.prototype.quotes;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-ui/#resize
 */
CSSProperties.prototype.resize;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-richness
 */
CSSProperties.prototype.richness;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-right
 */
CSSProperties.prototype.right;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-size
 */
CSSProperties.prototype.size;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-speak
 */
CSSProperties.prototype.speak;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-speakHeader
 */
CSSProperties.prototype.speakHeader;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-speakNumeral
 */
CSSProperties.prototype.speakNumeral;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-speakPunctuation
 */
CSSProperties.prototype.speakPunctuation;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-speechRate
 */
CSSProperties.prototype.speechRate;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-stress
 */
CSSProperties.prototype.stress;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-tableLayout
 */
CSSProperties.prototype.tableLayout;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-textAlign
 */
CSSProperties.prototype.textAlign;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-textDecoration
 */
CSSProperties.prototype.textDecoration;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-textIndent
 */
CSSProperties.prototype.textIndent;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-textShadow
 */
CSSProperties.prototype.textShadow;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-textTransform
 */
CSSProperties.prototype.textTransform;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-top
 */
CSSProperties.prototype.top;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-unicodeBidi
 */
CSSProperties.prototype.unicodeBidi;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-verticalAlign
 */
CSSProperties.prototype.verticalAlign;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-visibility
 */
CSSProperties.prototype.visibility;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-voiceFamily
 */
CSSProperties.prototype.voiceFamily;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-volume
 */
CSSProperties.prototype.volume;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-whiteSpace
 */
CSSProperties.prototype.whiteSpace;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-widows
 */
CSSProperties.prototype.widows;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-width
 */
CSSProperties.prototype.width;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-wordSpacing
 */
CSSProperties.prototype.wordSpacing;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-wordWrap
 */
CSSProperties.prototype.wordWrap;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/DOM-Level-2-Style/css.html#CSS-CSSProperties-zIndex
 */
CSSProperties.prototype.zIndex;

// CSS 3 properties

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-background/#box-shadow
 */
CSSProperties.prototype.boxShadow;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-ui/#box-sizing
 */
CSSProperties.prototype.boxSizing;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-color/#transparency
 */
CSSProperties.prototype.opacity;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-ui/#text-overflow
 */
CSSProperties.prototype.textOverflow;

// CSS 3 transforms

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-2d-transforms/#backface-visibility-property
 */
CSSProperties.prototype.backfaceVisibility;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-2d-transforms/#perspective
 */
CSSProperties.prototype.perspective;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-2d-transforms/#perspective-origin
 */
CSSProperties.prototype.perspectiveOrigin;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-2d-transforms/#effects
 */
CSSProperties.prototype.transform;

/**
 * @type {string|number}
 * @see http://www.w3.org/TR/css3-2d-transforms/#transform-origin
 */
CSSProperties.prototype.transformOrigin;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-2d-transforms/#transform-style
 */
CSSProperties.prototype.transformStyle;

// CSS 3 transitions

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-transitions/#transition
 */
CSSProperties.prototype.transition;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-transitions/#transition-delay
 */
CSSProperties.prototype.transitionDelay;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-transitions/#transition-duration
 */
CSSProperties.prototype.transitionDuration;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-transitions/#transition-property-property
 */
CSSProperties.prototype.transitionProperty;

/**
 * @type {string}
 * @see http://www.w3.org/TR/css3-transitions/#transition-timing-function
 */
CSSProperties.prototype.transitionTimingFunction;

/**
 * @type {string}
 * @see http://www.w3.org/TR/SVG11/interact.html#PointerEventsProperty
 */
CSSProperties.prototype.pointerEvents;

/**
 * TODO(dbeam): Put this in separate file named w3c_cssom.js.
 * Externs for the CSSOM View Module.
 * @see http://www.w3.org/TR/cssom-view/
 */

// http://www.w3.org/TR/cssom-view/#extensions-to-the-window-interface

/**
 * @param {string} media_query_list
 * @return {MediaQueryList}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-matchmedia
 */
Window.prototype.matchMedia = function(media_query_list) {};

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-innerwidth
 */
Window.prototype.innerWidth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-innerheight
 */
Window.prototype.innerHeight;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-scrollx
 */
Window.prototype.scrollX;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-pagexoffset
 */
Window.prototype.pageXOffset;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-scrolly
 */
Window.prototype.scrollY;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-pageyoffset
 */
Window.prototype.pageYOffset;

/**
 * @param {number} x
 * @param {number} y
 * @see http://www.w3.org/TR/cssom-view/#dom-window-scroll
 */
Window.prototype.scroll = function(x, y) {};

/**
 * @param {number} x
 * @param {number} y
 * @see http://www.w3.org/TR/cssom-view/#dom-window-scrollto
 */
Window.prototype.scrollTo = function(x, y) {};

/**
 * @param {number} x
 * @param {number} y
 * @see http://www.w3.org/TR/cssom-view/#dom-window-scrollby
 */
Window.prototype.scrollBy = function(x, y) {};

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-screenx
 */
Window.prototype.screenX;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-screeny
 */
Window.prototype.screenY;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-outerwidth
 */
Window.prototype.outerWidth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-window-outerheight
 */
Window.prototype.outerHeight;

/**
 * @constructor
 * @see http://www.w3.org/TR/cssom-view/#mediaquerylist
 */
function MediaQueryList() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/cssom-view/#dom-mediaquerylist-media
 */
MediaQueryList.prototype.media;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/cssom-view/#dom-mediaquerylist-matches
 */
MediaQueryList.prototype.matches;

/**
 * @param {MediaQueryListListener} listener
 * @see http://www.w3.org/TR/cssom-view/#dom-mediaquerylist-addlistener
 */
MediaQueryList.prototype.addListener = function(listener) {};

/**
 * @param {MediaQueryListListener} listener
 * @see http://www.w3.org/TR/cssom-view/#dom-mediaquerylist-removelistener
 */
MediaQueryList.prototype.removeListener = function(listener) {};

/**
 * @typedef {(function(!MediaQueryList) : void)}
 * @see http://www.w3.org/TR/cssom-view/#mediaquerylistlistener
 */
var MediaQueryListListener;

/**
 * @constructor
 * @see http://www.w3.org/TR/cssom-view/#screen
 */
function Screen() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-screen-availwidth
 */
Screen.prototype.availWidth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-screen-availheight
 */
Screen.prototype.availHeight;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-screen-width
 */
Screen.prototype.width;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-screen-height
 */
Screen.prototype.height;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-screen-colordepth
 */
Screen.prototype.colorDepth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-screen-pixeldepth
 */
Screen.prototype.pixelDepth;


// http://www.w3.org/TR/cssom-view/#extensions-to-the-document-interface

/**
 * @param {number} x
 * @param {number} y
 * @return {?Element}
 * @see http://www.w3.org/TR/cssom-view/#dom-document-elementfrompoint
 */
Document.prototype.elementFromPoint = function(x, y) {};

/**
 * @param {number} x
 * @param {number} y
 * @return {CaretPosition}
 * @see http://www.w3.org/TR/cssom-view/#dom-document-caretpositionfrompoint
 */
Document.prototype.caretPositionFromPoint = function(x, y) {};


/**
 * @constructor
 * @see http://www.w3.org/TR/cssom-view/#caretposition
 */
function CaretPosition() {}

/**
 * @type {Node}
 * @see http://www.w3.org/TR/cssom-view/#dom-caretposition-offsetnode
 */
CaretPosition.prototype.offsetNode;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-caretposition-offset
 */
CaretPosition.prototype.offset;


// http://www.w3.org/TR/cssom-view/#extensions-to-the-element-interface

/**
 * @return {!ClientRectList}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-getclientrects
 */
Element.prototype.getClientRects = function() {};

/**
 * @return {!ClientRect}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-getboundingclientrect
 */
Element.prototype.getBoundingClientRect = function() {};

/**
 * @param {boolean=} opt_top
 * @see http://www.w3.org/TR/cssom-view/#dom-element-scrollintoview
 */
Element.prototype.scrollIntoView = function(opt_top) {};

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-scrolltop
 */
Element.prototype.scrollTop;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-scrollleft
 */
Element.prototype.scrollLeft;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-scrollwidth
 */
Element.prototype.scrollWidth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-scrollheight
 */
Element.prototype.scrollHeight;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-clienttop
 */
Element.prototype.clientTop;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-clientleft
 */
Element.prototype.clientLeft;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-clientwidth
 */
Element.prototype.clientWidth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-element-clientheight
 */
Element.prototype.clientHeight;

// http://www.w3.org/TR/cssom-view/#extensions-to-the-htmlelement-interface

/**
 * @type {Element}
 * @see http://www.w3.org/TR/cssom-view/#dom-htmlelement-offsetparent
 */
HTMLElement.prototype.offsetParent;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-htmlelement-offsettop
 */
HTMLElement.prototype.offsetTop;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-htmlelement-offsetleft
 */
HTMLElement.prototype.offsetLeft;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-htmlelement-offsetwidth
 */
HTMLElement.prototype.offsetWidth;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-htmlelement-offsetheight
 */
HTMLElement.prototype.offsetHeight;


// http://www.w3.org/TR/cssom-view/#extensions-to-the-range-interface

/**
 * @return {!ClientRectList}
 * @see http://www.w3.org/TR/cssom-view/#dom-range-getclientrects
 */
Range.prototype.getClientRects = function() {};

/**
 * @return {!ClientRect}
 * @see http://www.w3.org/TR/cssom-view/#dom-range-getboundingclientrect
 */
Range.prototype.getBoundingClientRect = function() {};


// http://www.w3.org/TR/cssom-view/#extensions-to-the-mouseevent-interface

// MouseEvent: screen{X,Y} and client{X,Y} are in DOM Level 2/3 Event as well,
// so it seems like a specification issue. I've emailed www-style@w3.org in
// hopes of resolving the conflict, but in the mean time they can live here
// (http://lists.w3.org/Archives/Public/www-style/2012May/0039.html).

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-screenx
 */
//MouseEvent.prototype.screenX;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-screeny
 */
//MouseEvent.prototype.screenY;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-pagex
 */
MouseEvent.prototype.pageX;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-pagey
 */
MouseEvent.prototype.pageY;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-clientx
 */
//MouseEvent.prototype.clientX;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-clienty
 */
//MouseEvent.prototype.clientY;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-x
 */
MouseEvent.prototype.x;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-y
 */
MouseEvent.prototype.y;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-offsetx
 */
MouseEvent.prototype.offsetX;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-mouseevent-offsety
 */
MouseEvent.prototype.offsetY;


// http://www.w3.org/TR/cssom-view/#rectangles

/**
 * @constructor
 * @see http://www.w3.org/TR/cssom-view/#the-clientrectlist-interface
 */
function ClientRectList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrectlist-length
 */
ClientRectList.prototype.length;

/**
 * @param {number} index
 * @return {ClientRect}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrectlist-item
 */
ClientRectList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/cssom-view/#the-clientrect-interface
 */
function ClientRect() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrect-top
 */
ClientRect.prototype.top;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrect-right
 */
ClientRect.prototype.right;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrect-bottom
 */
ClientRect.prototype.bottom;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrect-left
 */
ClientRect.prototype.left;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrect-width
 */
ClientRect.prototype.width;

/**
 * @type {number}
 * @see http://www.w3.org/TR/cssom-view/#dom-clientrect-height
 */
ClientRect.prototype.height;

/**
 * @constructor
 * http://www.w3.org/TR/css3-conditional/#CSS-interface
 */
function CSSInterface() {}

/**
 * @param {string} property
 * @param {string=} opt_value
 * @return {boolean}
 */
CSSInterface.prototype.supports = function(property, opt_value) {};

/**
 * TODO(nicksantos): This suppress tag probably isn't needed, and
 * should be removed.
 * @suppress {duplicate}
 * @type {CSSInterface}
 */
var CSS;

/** @type {CSSInterface} */
Window.prototype.CSS;

// http://dev.w3.org/csswg/css-font-loading/

/**
 * @enum {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#enumdef-fontfaceloadstatus
 */
var FontFaceLoadStatus = {
  ERROR: 'error',
  LOADED: 'loaded',
  LOADING: 'loading',
  UNLOADED: 'unloaded'
};

/**
 * @typedef {{
 *   style: (string|undefined),
 *   weight: (string|undefined),
 *   stretch: (string|undefined),
 *   unicodeRange: (string|undefined),
 *   variant: (string|undefined),
 *   featureSettings: (string|undefined)
 * }}
 * @see http://dev.w3.org/csswg/css-font-loading/#dictdef-fontfacedescriptors
 */
var FontFaceDescriptors;

/**
 * @constructor
 * @param {string} fontFamily
 * @param {string} source
 * @param {!FontFaceDescriptors} descriptors
 * @see http://dev.w3.org/csswg/css-font-loading/#font-face-constructor
 */
function FontFace(fontFamily, source, descriptors) {}

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-family
 */
FontFace.prototype.family;

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-style
 */
FontFace.prototype.style;

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-weight
 */
FontFace.prototype.weight;

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-stretch
 */
FontFace.prototype.stretch;

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-unicoderange
 */
FontFace.prototype.unicodeRange;

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-variant
 */
FontFace.prototype.variant;

/**
 * @type {string}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-featuresettings
 */
FontFace.prototype.featureSettings;

/**
 * @type {FontFaceLoadStatus}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontface-status
 */
FontFace.prototype.status;

/**
 * @return {!Promise.<!FontFace>}
 * @see http://dev.w3.org/csswg/css-font-loading/#font-face-load
 */
FontFace.prototype.load = function() {};

/**
 * @enum
 * @see http://dev.w3.org/csswg/css-font-loading/#enumdef-fontfacesetloadstatus
 */
var FontFaceSetLoadStatus = {
  LOADED: 'loaded',
  LOADING: 'loading'
};

/**
 * @interface
 * @see http://dev.w3.org/csswg/css-font-loading/#FontFaceSet-interface
 */
function FontFaceSet() {}

// Event handlers
// http://dev.w3.org/csswg/css-font-loading/#FontFaceSet-events

/** @type {?function (Event)} */ FontFaceSet.prototype.onloading;
/** @type {?function (Event)} */ FontFaceSet.prototype.onloadingdone;
/** @type {?function (Event)} */ FontFaceSet.prototype.onloadingerror;

/**
 * @param {!FontFace} value
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-add
 */
FontFaceSet.prototype.add = function(value) {};

/**
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-clear
 */
FontFaceSet.prototype.clear = function() {};

/**
 * @param {!FontFace} value
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-delete
 */
FontFaceSet.prototype.delete = function(value) {};

/**
 * @param {!FontFace} font
 * @return {boolean}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-has
 */
FontFaceSet.prototype.has = function(font) {};

/**
 * @param {function(!FontFace, number, !FontFaceSet)} cb
 * @param {Object|undefined=} opt_selfObj
 * see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-foreach
 */
FontFaceSet.prototype.forEach = function(cb, opt_selfObj) {};

/**
 * @param {string} font
 * @param {string=} opt_text
 * @return {!Promise.<!Array.<!FontFace>>}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-load
 */
FontFaceSet.prototype.load = function(font, opt_text) {};

/**
 * @param {string} font
 * @param {string=} opt_text
 * @return {boolean}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-check
 */
FontFaceSet.prototype.check = function(font, opt_text) {};

/**
 * @type {!Promise.<!FontFaceSet>}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-ready
 */
FontFaceSet.prototype.ready;

/**
 * @type {FontFaceSetLoadStatus}
 * @see http://dev.w3.org/csswg/css-font-loading/#dom-fontfaceset-status
 */
FontFaceSet.prototype.status;

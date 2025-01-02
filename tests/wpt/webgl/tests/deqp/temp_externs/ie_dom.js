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
 * @fileoverview Definitions for all the extensions over the
 *  W3C's DOM specification by IE in JScript. This file depends on
 *  w3c_dom2.js. The whole file has NOT been fully type annotated.
 *
 * When a non-standard extension appears in both Gecko and IE, we put
 * it in gecko_dom.js
 *
 * @externs
 * @author stevey@google.com (Steve Yegge)
 */

// TODO(nicksantos): Rewrite all the DOM interfaces as interfaces, instead
// of kludging them as an inheritance hierarchy.

/**
 * @constructor
 * @extends {Document}
 * @see http://msdn.microsoft.com/en-us/library/ms757878(VS.85).aspx
 */
function XMLDOMDocument() {}

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms761398(VS.85).aspx
 */
XMLDOMDocument.prototype.async;

/**
 * @type {!Function}
 * @see http://msdn.microsoft.com/en-us/library/ms762647(VS.85).aspx
 */
XMLDOMDocument.prototype.ondataavailable;

/**
 * @type {!Function}
 * @see http://msdn.microsoft.com/en-us/library/ms764640(VS.85).aspx
 */
XMLDOMDocument.prototype.onreadystatechange;

/**
 * @type {!Function}
 * @see http://msdn.microsoft.com/en-us/library/ms753795(VS.85).aspx
 */
XMLDOMDocument.prototype.ontransformnode;

/**
 * @type {Object}
 * @see http://msdn.microsoft.com/en-us/library/ms756041(VS.85).aspx
 */
XMLDOMDocument.prototype.parseError;

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms761353(VS.85).aspx
 */
XMLDOMDocument.prototype.preserveWhiteSpace;

/**
 * @type {number}
 * @see http://msdn.microsoft.com/en-us/library/ms753702(VS.85).aspx
 */
XMLDOMDocument.prototype.readyState;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms762283(VS.85).aspx
 * @type {boolean}
 */
XMLDOMDocument.prototype.resolveExternals;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms760290(v=vs.85).aspx
 * @param {string} name
 * @param {*} value
 */
XMLDOMDocument.prototype.setProperty = function(name, value) {};

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms767669(VS.85).aspx
 */
XMLDOMDocument.prototype.url;

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms762791(VS.85).aspx
 */
XMLDOMDocument.prototype.validateOnParse;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms763830(VS.85).aspx
 */
XMLDOMDocument.prototype.abort = function() {};

/**
 * @param {*} type
 * @param {string} name
 * @param {string} namespaceURI
 * @return {Node}
 * @see http://msdn.microsoft.com/en-us/library/ms757901(VS.85).aspx
 * @nosideeffects
 */
XMLDOMDocument.prototype.createNode = function(type, name, namespaceURI) {};

/**
 * @param {string} xmlSource
 * @return {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms762722(VS.85).aspx
 * @override
 */
XMLDOMDocument.prototype.load = function(xmlSource) {};

/**
 * @param {string} xmlString
 * @return {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms754585(VS.85).aspx
 * @override
 */
XMLDOMDocument.prototype.loadXML = function(xmlString) {};

/**
 * @param {string} id
 * @return {Node}
 * @see http://msdn.microsoft.com/en-us/library/ms766397(VS.85).aspx
 */
XMLDOMDocument.prototype.nodeFromID = function(id) {};

//==============================================================================
// XMLNode methods and properties
// In a real DOM hierarchy, XMLDOMDocument inherits from XMLNode and Document.
// Since we can't express that in our type system, we put XMLNode properties
// on Node.

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms767570(VS.85).aspx
 */
Node.prototype.baseName;

/**
 * @type {?string}
 * @see http://msdn.microsoft.com/en-us/library/ms762763(VS.85).aspx
 */
Node.prototype.dataType;

/**
 * @type {Node}
 * @see http://msdn.microsoft.com/en-us/library/ms764733(VS.85).aspx
 */
Node.prototype.definition;

/**
 * IE5 used document instead of ownerDocument.
 * Old versions of WebKit used document instead of contentDocument.
 * @type {Document}
 */
Node.prototype.document;


/**
 * Inserts the given HTML text into the element at the location.
 * @param {string} sWhere Where to insert the HTML text, one of 'beforeBegin',
 *     'afterBegin', 'beforeEnd', 'afterEnd'.
 * @param {string} sText HTML text to insert.
 * @see http://msdn.microsoft.com/en-us/library/ms536452(VS.85).aspx
 */
Node.prototype.insertAdjacentHTML = function(sWhere, sText) {};


/**
 * @type {*}
 * @see http://msdn.microsoft.com/en-us/library/ms762308(VS.85).aspx
 */
Node.prototype.nodeTypedValue;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms757895(VS.85).aspx
 */
Node.prototype.nodeTypeString;

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms762237(VS.85).aspx
 */
Node.prototype.parsed;

/**
 * @type {Element}
 * @see http://msdn.microsoft.com/en-us/library/ms534327(VS.85).aspx
 */
Node.prototype.parentElement;

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms753816(VS.85).aspx
 */
Node.prototype.specified;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms762687(VS.85).aspx
 */
Node.prototype.text;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms755989(VS.85).aspx
 */
Node.prototype.xml;

/**
 * @param {string} expression An XPath expression.
 * @return {NodeList}
 * @see http://msdn.microsoft.com/en-us/library/ms754523(VS.85).aspx
 * @nosideeffects
 */
Node.prototype.selectNodes = function(expression) {};

/**
 * @param {string} expression An XPath expression.
 * @return {Node}
 * @see http://msdn.microsoft.com/en-us/library/ms757846(VS.85).aspx
 * @nosideeffects
 */
Node.prototype.selectSingleNode = function(expression) {};

/**
 * @param {Node} stylesheet XSLT stylesheet.
 * @return {string}
 * @see http://msdn.microsoft.com/en-us/library/ms761399(VS.85).aspx
 * @nosideeffects
 */
Node.prototype.transformNode = function(stylesheet) {};

/**
 * @param {Node} stylesheet XSLT stylesheet.
 * @param {Object} outputObject
 * @see http://msdn.microsoft.com/en-us/library/ms766561(VS.85).aspx
 */
Node.prototype.transformNodeToObject =
    function(stylesheet, outputObject) {};

//==============================================================================
// Node methods

/**
 * @param {boolean=} opt_bRemoveChildren Whether to remove the entire sub-tree.
 *    Defaults to false.
 * @return {Node} The object that was removed.
 * @see http://msdn.microsoft.com/en-us/library/ms536708(VS.85).aspx
 */
Node.prototype.removeNode = function(opt_bRemoveChildren) {};

/**
 * @constructor
 */
function ClipboardData() {}

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535220(VS.85).aspx
 * @param {string=} opt_type Type of clipboard data to clear. 'Text' or
 *     'URL' or 'File' or 'HTML' or 'Image'.
 */
ClipboardData.prototype.clearData = function(opt_type) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535220(VS.85).aspx
 * @param {string} type Type of clipboard data to set ('Text' or 'URL').
 * @param {string} data Data to set
 * @return {boolean} Whether the data were set correctly.
 */
ClipboardData.prototype.setData = function(type, data) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535220(VS.85).aspx
 * @param {string} type Type of clipboard data to get ('Text' or 'URL').
 * @return {string} The current data
 */
ClipboardData.prototype.getData = function(type) { };

/**
 * @type {!Window}
 * @see https://developer.mozilla.org/en/DOM/window
 */
var window;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535220(VS.85).aspx
 * @type ClipboardData
 */
Window.prototype.clipboardData;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533724(VS.85).aspx
 */
Window.prototype.dialogHeight;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533725(VS.85).aspx
 */
Window.prototype.dialogLeft;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533726(VS.85).aspx
 */
Window.prototype.dialogTop;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533727(VS.85).aspx
 */
Window.prototype.dialogWidth;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535863(VS.85).aspx
 */
Window.prototype.event;

/**
 * @see http://msdn.microsoft.com/en-us/library/cc197012(VS.85).aspx
 */
Window.prototype.maxConnectionsPer1_0Server;

/**
 * @see http://msdn.microsoft.com/en-us/library/cc197013(VS.85).aspx
 */
Window.prototype.maxConnectionsPerServer;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534198(VS.85).aspx
 */
Window.prototype.offscreenBuffering;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534389(VS.85).aspx
 */
Window.prototype.screenLeft;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534389(VS.85).aspx
 */
Window.prototype.screenTop;

// Functions

/**
 * @param {string} event
 * @param {Function} handler
 * @see http://msdn.microsoft.com/en-us/library/ms536343(VS.85).aspx
 */
Window.prototype.attachEvent;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536392(VS.85).aspx
 */
Window.prototype.createPopup;

/**
 * @param {string} event
 * @param {Function} handler
 * @see http://msdn.microsoft.com/en-us/library/ms536411(VS.85).aspx
 */
Window.prototype.detachEvent;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536420(VS.85).aspx
 */
Window.prototype.execScript;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536425(VS.85).aspx
 */
Window.prototype.focus;

/**
 * @param {number} x
 * @param {number} y
 * @see http://msdn.microsoft.com/en-us/library/ms536618(VS.85).aspx
 */
Window.prototype.moveBy = function(x, y) {};

/**
 * @param {number} x
 * @param {number} y
 * @see http://msdn.microsoft.com/en-us/library/ms536626(VS.85).aspx
 */
Window.prototype.moveTo = function(x, y) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536638(VS.85).aspx
 */
Window.prototype.navigate;

/**
 * @param {*=} opt_url
 * @param {string=} opt_windowName
 * @param {string=} opt_windowFeatures
 * @param {boolean=} opt_replace
 * @return {Window}
 * @see http://msdn.microsoft.com/en-us/library/ms536651(VS.85).aspx
 */
Window.prototype.open = function(opt_url, opt_windowName, opt_windowFeatures,
                                 opt_replace) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536672(VS.85).aspx
 */
Window.prototype.print = function() {};

/**
 * @param {number} width
 * @param {number} height
 * @see http://msdn.microsoft.com/en-us/library/ms536722(VS.85).aspx
 */
Window.prototype.resizeBy = function(width, height) {};

/**
 * @param {number} width
 * @param {number} height
 * @see http://msdn.microsoft.com/en-us/library/ms536723(VS.85).aspx
 */
Window.prototype.resizeTo = function(width, height) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536738(VS.85).aspx
 */
Window.prototype.setActive;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536758(VS.85).aspx
 */
Window.prototype.showHelp;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536761(VS.85).aspx
 */
Window.prototype.showModelessDialog;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535246%28v=vs.85%29.aspx
 * @const {!Object}
 */
Window.prototype.external;

/**
 * @constructor
 */
function History() { };

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535864(VS.85).aspx
 * @param {number|string} delta The number of entries to go back, or
 *     the URL to which to go back. (URL form is supported only in IE)
 */
History.prototype.go = function(delta) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535864(VS.85).aspx
 * @param {number=} opt_distance The number of entries to go back
 *     (Mozilla doesn't support distance -- use #go instead)
 */
History.prototype.back = function(opt_distance) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535864(VS.85).aspx
 * @type {number}
 */
History.prototype.length;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535864(VS.85).aspx
 */
History.prototype.forward = function() {};

/**
 * @type {boolean}
 * @implicitCast
 * @see http://msdn.microsoft.com/en-us/library/ie/ms533072(v=vs.85).aspx
 */
HTMLFrameElement.prototype.allowTransparency;

/**
 * @type {Window}
 * @see http://msdn.microsoft.com/en-us/library/ms533692(VS.85).aspx
 */
HTMLFrameElement.prototype.contentWindow;

/**
 * @type {boolean}
 * @implicitCast
 * @see http://msdn.microsoft.com/en-us/library/ie/ms533072(v=vs.85).aspx
 */
HTMLIFrameElement.prototype.allowTransparency;

/**
 * @type {Window}
 * @see http://msdn.microsoft.com/en-us/library/ms533692(VS.85).aspx
 */
HTMLIFrameElement.prototype.contentWindow;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536385(VS.85).aspx
 */
HTMLBodyElement.prototype.createControlRange;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms534359(VS.85).aspx
 */
HTMLScriptElement.prototype.readyState;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms534359(VS.85).aspx
 */
HTMLIFrameElement.prototype.readyState;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms534359(VS.85).aspx
 */
HTMLImageElement.prototype.readyState;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms534359(VS.85).aspx
 */
HTMLObjectElement.prototype.readyState;


/**
 * @constructor
 */
function ControlRange() {}

ControlRange.prototype.add;
ControlRange.prototype.addElement;
ControlRange.prototype.execCommand;
ControlRange.prototype.item;
ControlRange.prototype.queryCommandEnabled;
ControlRange.prototype.queryCommandIndeterm;
ControlRange.prototype.queryCommandState;
ControlRange.prototype.queryCommandSupported;
ControlRange.prototype.queryCommandValue;
ControlRange.prototype.remove;
ControlRange.prototype.scrollIntoView;
ControlRange.prototype.select;

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/ms535872.aspx
 */
function TextRange() {}

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533538(VS.85).aspx
 */
TextRange.prototype.boundingHeight;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533539(VS.85).aspx
 */
TextRange.prototype.boundingLeft;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533540(VS.85).aspx
 */
TextRange.prototype.boundingTop;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533541(VS.85).aspx
 */
TextRange.prototype.boundingWidth;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533874(VS.85).aspx
 */
TextRange.prototype.htmlText;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534200(VS.85).aspx
 */
TextRange.prototype.offsetLeft;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534303(VS.85).aspx
 */
TextRange.prototype.offsetTop;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534676(VS.85).aspx
 */
TextRange.prototype.text;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536371(VS.85).aspx
 */
TextRange.prototype.collapse;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536373(VS.85).aspx
 */
TextRange.prototype.compareEndPoints;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536416(VS.85).aspx
 */
TextRange.prototype.duplicate;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536419(VS.85).aspx
 */
TextRange.prototype.execCommand;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536421(VS.85).aspx
 */
TextRange.prototype.expand;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536422(VS.85).aspx
 */
TextRange.prototype.findText;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536432(VS.85).aspx
 */
TextRange.prototype.getBookmark;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536433(VS.85).aspx
 */
TextRange.prototype.getBoundingClientRect;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536435(VS.85).aspx
 */
TextRange.prototype.getClientRects;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536450(VS.85).aspx
 */
TextRange.prototype.inRange;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536458(VS.85).aspx
 */
TextRange.prototype.isEqual;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536616(VS.85).aspx
 */
TextRange.prototype.move;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536620(VS.85).aspx
 */
TextRange.prototype.moveEnd;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536623(VS.85).aspx
 */
TextRange.prototype.moveStart;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536628(VS.85).aspx
 */
TextRange.prototype.moveToBookmark;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536630(VS.85).aspx
 */
TextRange.prototype.moveToElementText;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536632(VS.85).aspx
 */
TextRange.prototype.moveToPoint;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536654(VS.85).aspx
 */
TextRange.prototype.parentElement;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536656(VS.85).aspx
 */
TextRange.prototype.pasteHTML;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536676(VS.85).aspx
 */
TextRange.prototype.queryCommandEnabled;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536678(VS.85).aspx
 */
TextRange.prototype.queryCommandIndeterm;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536679(VS.85).aspx
 */
TextRange.prototype.queryCommandState;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536681(VS.85).aspx
 */
TextRange.prototype.queryCommandSupported;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536683(VS.85).aspx
 */
TextRange.prototype.queryCommandValue;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536730(VS.85).aspx
 */
TextRange.prototype.scrollIntoView;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536735(VS.85).aspx
 */
TextRange.prototype.select;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536745(VS.85).aspx
 */
TextRange.prototype.setEndPoint;

/**
 * @return {undefined}
 * @see http://msdn.microsoft.com/en-us/library/ms536418(VS.85).aspx
 */
Selection.prototype.clear = function() {};

/**
 * @return {TextRange|ControlRange}
 * @see http://msdn.microsoft.com/en-us/library/ms536394(VS.85).aspx
 */
Selection.prototype.createRange = function() {};

/**
 * @return {Array.<TextRange>}
 * @see http://msdn.microsoft.com/en-us/library/ms536396(VS.85).aspx
 */
Selection.prototype.createRangeCollection = function() {};

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/ms537447(VS.85).aspx
 */
function controlRange() {}


Document.prototype.loadXML;


// http://msdn.microsoft.com/en-us/library/ms531073(VS.85).aspx

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533065(VS.85).aspx
 */
Document.prototype.activeElement;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533553(VS.85).aspx
 */
Document.prototype.charset;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533693(VS.85).aspx
 */
Document.prototype.cookie;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533714(VS.85).aspx
 */
Document.prototype.defaultCharset;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533731(VS.85).aspx
 */
Document.prototype.dir;

/**
 * @see http://msdn.microsoft.com/en-us/library/cc196988(VS.85).aspx
 */
Document.prototype.documentMode;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533747(VS.85).aspx
 */
Document.prototype.expando;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533750(VS.85).aspx
 */
Document.prototype.fileCreatedDate;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533751(VS.85).aspx
 */
Document.prototype.fileModifiedDate;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533752(VS.85).aspx
 */
Document.prototype.fileSize;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534331(VS.85).aspx
 */
Document.prototype.parentWindow;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534353(VS.85).aspx
 */
Document.prototype.protocol;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms534359(VS.85).aspx
 */
HTMLDocument.prototype.readyState;

/**
 * @type {Selection}
 * @see http://msdn.microsoft.com/en-us/library/ms535869(VS.85).aspx
 */
Document.prototype.selection;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534704(VS.85).aspx
 */
Document.prototype.uniqueID;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534709(VS.85).aspx
 */
Document.prototype.URLUnencoded;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535155(VS.85).aspx
 */
Document.prototype.XMLDocument;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535163(VS.85).aspx
 */
Document.prototype.XSLDocument;

// functions

/**
 * @param {string} event
 * @param {Function} handler
 * @see http://msdn.microsoft.com/en-us/library/ms536343(VS.85).aspx
 */
Document.prototype.attachEvent;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536390(VS.85).aspx
 */
Document.prototype.createEventObject;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms531194(VS.85).aspx
 */
Document.prototype.createStyleSheet;

/**
 * @param {string} event
 * @param {Function} handler
 * @see http://msdn.microsoft.com/en-us/library/ms536411(VS.85).aspx
 */
Document.prototype.detachEvent;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536425(VS.85).aspx
 */
Document.prototype.focus;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536447(VS.85).aspx
 * @return {boolean}
 */
Document.prototype.hasFocus = function() {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536614(VS.85).aspx
 */
Document.prototype.mergeAttributes;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536685(VS.85).aspx
 */
Document.prototype.recalc;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536689(VS.85).aspx
 */
Document.prototype.releaseCapture;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536738(VS.85).aspx
 */
Document.prototype.setActive;


// collections

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537434(VS.85).aspx
 */
Document.prototype.all;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537445(VS.85).aspx
 */
Document.prototype.childNodes;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537459(VS.85).aspx
 */
Document.prototype.frames;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537470(VS.85).aspx
 */
Document.prototype.namespaces;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537487(VS.85).aspx
 */
Document.prototype.scripts;

/**
 * @param {string} sUrl
 * @return {number}
 * @see http://msdn.microsoft.com/en-us/library/ms535922(VS.85).aspx
 */
Element.prototype.addBehavior = function(sUrl) {};

/**
 * @param {string} event
 * @param {Function} handler
 * @see http://msdn.microsoft.com/en-us/library/mm536343(v=vs.85).aspx
 */
Element.prototype.attachEvent;

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms533546(VS.85).aspx
 */
Element.prototype.canHaveChildren;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms533559(v=vs.85).aspx
 */
Element.prototype.classid;

/**
 * @param {number} iCoordX Integer that specifies the client window coordinate
 *     of x.
 * @param {number} iCoordY Integer that specifies the client window coordinate
 *     of y.
 * @return {string} The component of an element located at the specified
 *     coordinates.
 * @see http://msdn.microsoft.com/en-us/library/ms536375(VS.85).aspx
 * @nosideeffects
 */
Element.prototype.componentFromPoint = function(iCoordX, iCoordY) {};


/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms533690(VS.85).aspx
 */
Element.prototype.contentEditable;

/**
 * @return {TextRange}
 * @see http://msdn.microsoft.com/en-us/library/ms536401(VS.85).aspx
 */
Element.prototype.createTextRange;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms535231(VS.85).aspx
 */
Element.prototype.currentStyle;

/**
 * @param {string} event
 * @param {Function} handler
 * @see http://msdn.microsoft.com/en-us/library/ie/ms536411(v=vs.85).aspx
 */
Element.prototype.detachEvent;

/**
 * @param {string=} opt_action
 * @see http://msdn.microsoft.com/en-us/library/ms536414%28VS.85%29.aspx
 */
Element.prototype.doScroll = function(opt_action) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536423(VS.85).aspx
 */
Element.prototype.fireEvent;

/**
 * @type {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms533783(VS.85).aspx
 */
Element.prototype.hideFocus;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533899.aspx
 */
Element.prototype.innerText;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537838(VS.85).aspx
 */
Element.prototype.isContentEditable;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms531395(v=vs.85).aspx
 * NOTE: Left untyped to avoid conflict with subclasses.
 */
Element.prototype.load;

/**
 * @param {number} pointerId Id of the pointer that is assign to the element.
 * @see http://msdn.microsoft.com/en-us/library/ie/hh771882(v=vs.85).aspx
 */
Element.prototype.msSetPointerCapture = function(pointerId) {};

/**
 * @param {number} pointerId
 * @see http://msdn.microsoft.com/en-us/library/ie/hh771880.aspx
 */
Element.prototype.msReleasePointerCapture = function(pointerId) {};

/**
 * @type {?function(Event)}
 * @see http://msdn.microsoft.com/en-us/library/ms536903(v=vs.85).aspx
 */
Element.prototype.onbeforedeactivate;

/**
 * @type {?function(Event)}
 * @see http://msdn.microsoft.com/en-us/library/ms536945(VS.85).aspx
 */
Element.prototype.onmouseenter;

/**
 * @type {?function(Event)}
 * @see http://msdn.microsoft.com/en-us/library/ms536946(VS.85).aspx
 */
Element.prototype.onmouseleave;

/**
 * @type {?function(Event)}
 * @see http://msdn.microsoft.com/en-us/library/ms536969(VS.85).aspx
 */
Element.prototype.onselectstart;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/aa752326(VS.85).aspx
 */
Element.prototype.outerHTML;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536689(VS.85).aspx
 */
Element.prototype.releaseCapture = function() {};

/**
 * @param {number} iID
 * @return {boolean}
 * @see http://msdn.microsoft.com/en-us/library/ms536700(VS.85).aspx
 */
Element.prototype.removeBehavior = function(iID) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/aa703996(VS.85).aspx
 */
Element.prototype.runtimeStyle;

/**
 * @param {string} sStoreName The arbitrary name assigned to a persistent object
 *     in a UserData store.
 * @see http://msdn.microsoft.com/en-us/library/ms531403(v=vs.85).aspx
 */
Element.prototype.save = function(sStoreName) {};

/**
 * @param {boolean=} opt_bContainerCapture Events originating in a container are
 *     captured by the container. Defaults to true.
 * @see http://msdn.microsoft.com/en-us/library/ms536742(VS.85).aspx
 */
Element.prototype.setCapture = function(opt_bContainerCapture) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534635(VS.85).aspx
 */
Element.prototype.sourceIndex;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms537840.aspx
 */
Element.prototype.unselectable;

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/aa752462(v=vs.85).aspx
 */
function HTMLFiltersCollection() {}

/**
 * @see http://msdn.microsoft.com/en-us/library/aa752463(v=vs.85).aspx
 * @type {number}
 */
HTMLFiltersCollection.prototype.length;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms537452(v=vs.85).aspx
 * @type {HTMLFiltersCollection}
 */
Element.prototype.filters;

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/ms532853(v=vs.85).aspx
 */
function HTMLFilter() {}

/**
 * @see http://msdn.microsoft.com/en-us/library/ms532954(v=vs.85).aspx
 */
HTMLFilter.prototype.apply = function() {};

/**
 * @constructor
 * @extends {HTMLFilter}
 * @see http://msdn.microsoft.com/en-us/library/ms532967(v=vs.85).aspx
 */
function AlphaFilter() {}

/**
 * @see http://msdn.microsoft.com/en-us/library/ms532910(v=vs.85).aspx
 * @type {number}
 */
AlphaFilter.prototype.Opacity;

/**
 * @constructor
 * @extends {HTMLFilter}
 * @see http://msdn.microsoft.com/en-us/library/ms532969(v=vs.85).aspx
 */
function AlphaImageLoaderFilter() {}

/**
 * @see http://msdn.microsoft.com/en-us/library/ms532920(v=vs.85).aspx
 * @type {string}
 */
AlphaImageLoaderFilter.prototype.sizingMethod;

/**
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/ms535866(VS.85).aspx
 */
function Location() {}

/**
 * @see http://trac.webkit.org/changeset/113945
 * @type {DOMStringList}
 */
Location.prototype.ancestorOrigins;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533775(VS.85).aspx
 * @type {string}
 */
Location.prototype.hash;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533784(VS.85).aspx
 * @type {string}
 */
Location.prototype.host;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533785(VS.85).aspx
 * @type {string}
 */
Location.prototype.hostname;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms533867(VS.85).aspx
 * @type {string}
 */
Location.prototype.href;

/**
 * @see https://docs.google.com/document/view?id=1r_VTFKApVOaNIkocrg0z-t7lZgzisTuGTXkdzAk4gLU&hl=en
 * @type {string}
 */
Location.prototype.origin;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534332(VS.85).aspx
 * @type {string}
 */
Location.prototype.pathname;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534342(VS.85).aspx
 */
Location.prototype.port;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534353(VS.85).aspx
 * @type {string}
 */
Location.prototype.protocol;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms534620(VS.85).aspx
 * @type {string}
 */
Location.prototype.search;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536342(VS.85).aspx
 * @param {string} url
 */
Location.prototype.assign = function(url) {};

/**
 * @param {boolean=} opt_forceReload If true, reloads the page from
 *     the server. Defaults to false.
 * @see http://msdn.microsoft.com/en-us/library/ms536691(VS.85).aspx
 */
Location.prototype.reload = function(opt_forceReload) {};

/**
 * @param {string} url
 * @see http://msdn.microsoft.com/en-us/library/ms536712(VS.85).aspx
 */
Location.prototype.replace = function(url) {};


// For IE, returns an object representing key-value pairs for all the global
// variables prefixed with str, e.g. test*

/** @param {*=} opt_str */
function RuntimeObject(opt_str) {}


/**
 * @type {StyleSheet}
 * @see http://msdn.microsoft.com/en-us/library/dd347030(VS.85).aspx
 */
HTMLStyleElement.prototype.styleSheet;


/**
 * IE implements Cross Origin Resource Sharing (cross-domain XMLHttpRequests)
 * via the XDomainRequest object.
 *
 * @constructor
 * @see http://msdn.microsoft.com/en-us/library/cc288060(v=vs.85).aspx
 * @see http://www.w3.org/TR/cors/
 */
function XDomainRequest() {}

/**
 * Aborts the request.
 * @see http://msdn.microsoft.com/en-us/library/cc288129(v=vs.85).aspx
 */
XDomainRequest.prototype.abort = function() {};

/**
 * Sets the method and URL for the request.
 * @param {string} bstrMethod Either "GET" or "POST"
 * @param {string} bstrUrl The target URL
 * @see http://msdn.microsoft.com/en-us/library/cc288168(v=vs.85).aspx
 */
XDomainRequest.prototype.open = function(bstrMethod, bstrUrl) {};

/**
 * Sends the request.
 * @param {string=} varBody The POST body to send to the server. If omitted,
 *     the behavior is identical to sending an empty string.
 * @see http://msdn.microsoft.com/en-us/library/cc288207(v=vs.85).aspx
 */
XDomainRequest.prototype.send = function(varBody) {};

/**
 * Called if the request could not be completed. Note that error information is
 * not available.
 * @see http://msdn.microsoft.com/en-us/library/ms536930%28v=VS.85%29.aspx
 * @type {?function()}
 */
XDomainRequest.prototype.onerror;

/**
 * Called when the response has finished.
 * @see http://msdn.microsoft.com/en-us/library/ms536942%28v=VS.85%29.aspx
 * @type {?function()}
 */
XDomainRequest.prototype.onload;

/**
 * Called every time part of the response has been received.
 * @see http://msdn.microsoft.com/en-us/library/cc197058%28v=VS.85%29.aspx
 * @type {?function()}
 */
XDomainRequest.prototype.onprogress;

/**
 * Called if the timeout period has elapsed.
 * @see http://msdn.microsoft.com/en-us/library/cc197061%28v=VS.85%29.aspx
 * @type {?function()}
 */
XDomainRequest.prototype.ontimeout;

/**
 * The current response body.
 * @see http://msdn.microsoft.com/en-us/library/cc287956%28v=VS.85%29.aspx
 * @type {string}
 */
XDomainRequest.prototype.responseText;

/**
 * The timeout (in milliseconds) for the request.
 * @type {number}
 */
XDomainRequest.prototype.timeout;

/**
 * The Content-Type of the response, or an empty string.
 * @type {string}
 */
XDomainRequest.prototype.contentType;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms533542(v=vs.85).aspx
 */
Navigator.prototype.browserLanguage;

/**
 * @type {boolean}
 * @see http://blogs.msdn.com/b/ie/archive/2011/09/20/touch-input-for-ie10-and-metro-style-apps.aspx
 */
Navigator.prototype.msPointerEnabled;

/**
 * @type {number}
 * @see http://msdn.microsoft.com/en-us/library/ms533721(v=vs.85).aspx
 */
Screen.prototype.deviceXDPI;

/**
 * @type {number}
 * @see http://msdn.microsoft.com/en-us/library/ms534128%28v=vs.85%29.aspx
 */
Screen.prototype.logicalXDPI;

/**
 * @type {number}
 * @see http://msdn.microsoft.com/en-us/library/ms534130%28v=vs.85%29.aspx
 */
Screen.prototype.logicalYDPI;

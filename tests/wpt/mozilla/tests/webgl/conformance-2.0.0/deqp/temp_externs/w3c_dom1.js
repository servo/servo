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
 * @fileoverview Definitions for W3C's DOM Level 1 specification.
 *  The whole file has been fully type annotated. Created from
 *  http://www.w3.org/TR/REC-DOM-Level-1/ecma-script-language-binding.html
 *
 * @externs
 * @author stevey@google.com (Steve Yegge)
 */

/**
 * @constructor
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-17189187
 */
function DOMException() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.INDEX_SIZE_ERR = 1;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.DOMSTRING_SIZE_ERR = 2;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.HIERARCHY_REQUEST_ERR = 3;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.WRONG_DOCUMENT_ERR = 4;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.INVALID_CHARACTER_ERR = 5;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.NO_DATA_ALLOWED_ERR = 6;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.NO_MODIFICATION_ALLOWED_ERR = 7;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.NOT_FOUND_ERR = 8;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.NOT_SUPPORTED_ERR = 9;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
DOMException.INUSE_ATTRIBUTE_ERR = 10;

/**
 * @constructor
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-258A00AF
 */
function ExceptionCode() {}

/**
 * @constructor
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-102161490
 */
function DOMImplementation() {}

/**
 * @param {string} feature
 * @param {string} version
 * @return {boolean}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-5CED94D7
 * @nosideeffects
 */
DOMImplementation.prototype.hasFeature = function(feature, version) {};

/**
 * @constructor
 * @implements {EventTarget}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
function Node() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Node.prototype.addEventListener = function(type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Node.prototype.removeEventListener = function(type, listener, opt_useCapture) {};

/** @override */
Node.prototype.dispatchEvent = function(evt) {};

/**
 * @type {NamedNodeMap}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-attributes
 */
Node.prototype.attributes;

/**
 * @type {!NodeList}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-childNodes
 */
Node.prototype.childNodes;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-firstChild
 */
Node.prototype.firstChild;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-lastChild
 */
Node.prototype.lastChild;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-nextSibling
 */
Node.prototype.nextSibling;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-nodeName
 */
Node.prototype.nodeName;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-nodeValue
 */
Node.prototype.nodeValue;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-nodeType
 */
Node.prototype.nodeType;

/**
 * @type {Document}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-ownerDocument
 */
Node.prototype.ownerDocument;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-parentNode
 */
Node.prototype.parentNode;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-previousSibling
 */
Node.prototype.previousSibling;

/**
 * @param {Node} newChild
 * @return {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-appendChild
 */
Node.prototype.appendChild = function(newChild) {};

/**
 * @param {boolean} deep
 * @return {!Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-cloneNode
 * @nosideeffects
 */
Node.prototype.cloneNode = function(deep) {};

/**
 * @return {boolean}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-hasChildNodes
 * @nosideeffects
 */
Node.prototype.hasChildNodes = function() {};

/**
 * @param {Node} newChild
 * @param {Node} refChild
 * @return {!Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-insertBefore
 */
Node.prototype.insertBefore = function(newChild, refChild) {};

/**
 * @param {Node} oldChild
 * @return {!Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-removeChild
 */
Node.prototype.removeChild = function(oldChild) {};

/**
 * @param {Node} newChild
 * @param {Node} oldChild
 * @return {!Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-replaceChild
 */
Node.prototype.replaceChild = function(newChild, oldChild) {};

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.ATTRIBUTE_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.CDATA_SECTION_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.COMMENT_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.DOCUMENT_FRAGMENT_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.DOCUMENT_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.DOCUMENT_TYPE_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.ELEMENT_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.ENTITY_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.ENTITY_REFERENCE_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.PROCESSING_INSTRUCTION_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.TEXT_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.XPATH_NAMESPACE_NODE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1950641247
 */
Node.NOTATION_NODE;

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-B63ED1A3
 */
function DocumentFragment() {}

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#i-Document
 */
function Document() {}

/**
 * @type {DocumentType}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-doctype
 */
Document.prototype.doctype;

/**
 * @type {!Element}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-documentElement
 */
Document.prototype.documentElement;

/**
 * @type {DOMImplementation}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-implementation
 */
Document.prototype.implementation;

/**
 * @param {string} name
 * @return {!Attr}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createAttribute
 * @nosideeffects
 */
Document.prototype.createAttribute = function(name) {};

/**
 * @param {string} data
 * @return {!Comment}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createComment
 * @nosideeffects
 */
Document.prototype.createComment = function(data) {};

/**
 * @param {string} data
 * @return {!CDATASection}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createCDATASection
 * @nosideeffects
 */
Document.prototype.createCDATASection = function(data) {};

/**
 * @return {!DocumentFragment}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createDocumentFragment
 * @nosideeffects
 */
Document.prototype.createDocumentFragment = function() {};

/**
 * Create a DOM element.
 *
 * Web components introduced the second parameter as a way of extending existing
 * tags (e.g. document.createElement('button', 'fancy-button')).
 *
 * @param {string} tagName
 * @param {string=} opt_typeExtension
 * @return {!Element}
 * @nosideeffects
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createElement
 * @see http://w3c.github.io/webcomponents/spec/custom/#extensions-to-document-interface-to-instantiate
 */
Document.prototype.createElement = function(tagName, opt_typeExtension) {};

/**
 * @param {string} name
 * @return {!EntityReference}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createEntityReference
 * @nosideeffects
 */
Document.prototype.createEntityReference = function(name) {};

/**
 * @param {string} target
 * @param {string} data
 * @return {!ProcessingInstruction}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createProcessingInstruction
 * @nosideeffects
 */
Document.prototype.createProcessingInstruction = function(target, data) {};

/**
 * @param {number|string} data
 * @return {!Text}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-createTextNode
 * @nosideeffects
 */
Document.prototype.createTextNode = function(data) {};

/**
 * @param {string} tagname
 * @return {!NodeList}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-A6C9094
 * @nosideeffects
 */
Document.prototype.getElementsByTagName = function(tagname) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-536297177
 */
function NodeList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-203510337
 */
NodeList.prototype.length;

/**
 * @param {number} index
 * @return {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-844377136
 */
NodeList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1780488922
 */
function NamedNodeMap() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-6D0FB19E
 */
NamedNodeMap.prototype.length;

/**
 * @param {string} name
 * @return {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1074577549
 * @nosideeffects
 */
NamedNodeMap.prototype.getNamedItem = function(name) {};

/**
 * @param {number} index
 * @return {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-349467F9
 * @nosideeffects
 */
NamedNodeMap.prototype.item = function(index) {};

/**
 * @param {string} name
 * @return {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-D58B193
 */
NamedNodeMap.prototype.removeNamedItem = function(name) {};

/**
 * @param {Node} arg
 * @return {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1025163788
 */
NamedNodeMap.prototype.setNamedItem = function(arg) {};

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-FF21A306
 */
function CharacterData() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-72AB8359
 */
CharacterData.prototype.data;

/**
 * @type {number}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-7D61178C
 */
CharacterData.prototype.length;

/**
 * @param {string} arg
 * @return {undefined}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-32791A2F
 */
CharacterData.prototype.appendData = function(arg) {};

/**
 * @param {number} offset
 * @param {number} count
 * @return {undefined}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-7C603781
 */
CharacterData.prototype.deleteData = function(offset, count) {};

/**
 * @param {number} offset
 * @param {string} arg
 * @return {undefined}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-3EDB695F
 */
CharacterData.prototype.insertData = function(offset, arg) {};

/**
 * @param {number} offset
 * @param {number} count
 * @param {string} arg
 * @return {undefined}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-E5CBA7FB
 */
CharacterData.prototype.replaceData = function(offset, count, arg) {};

/**
 * @param {number} offset
 * @param {number} count
 * @return {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-6531BCCF
 * @nosideeffects
 */
CharacterData.prototype.substringData = function(offset, count) {};

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-637646024
 */
function Attr() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1112119403
 */
Attr.prototype.name;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-862529273
 */
Attr.prototype.specified;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-221662474
 */
Attr.prototype.value;

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-745549614
 */
function Element() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#attribute-tagName
 */
Element.prototype.tagName;

/**
 * @param {string} name
 * @param {number?=} opt_flags
 * @return {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-getAttribute
 * @see http://msdn.microsoft.com/en-us/library/ms536429(VS.85).aspx
 * @nosideeffects
 */
Element.prototype.getAttribute = function(name, opt_flags) {};

/**
 * @param {string} name
 * @return {Attr}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-getAttributeNode
 * @nosideeffects
 */
Element.prototype.getAttributeNode = function(name) {};

/**
 * @param {string} tagname
 * @return {!NodeList}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1938918D
 * @nosideeffects
 */
Element.prototype.getElementsByTagName = function(tagname) {};

/**
 * @param {string} name
 * @return {undefined}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-removeAttribute
 */
Element.prototype.removeAttribute = function(name) {};

/**
 * @param {Attr} oldAttr
 * @return {?Attr}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-removeAttributeNode
 */
Element.prototype.removeAttributeNode = function(oldAttr) {};

/**
 * @param {string} name
 * @param {string|number|boolean} value Values are converted to strings with
 *     ToString, so we accept number and boolean since both convert easily to
 *     strings.
 * @return {undefined}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-setAttribute
 */
Element.prototype.setAttribute = function(name, value) {};

/**
 * @param {Attr} newAttr
 * @return {?Attr}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#method-setAttributeNode
 */
Element.prototype.setAttributeNode = function(newAttr) {};

// Event handlers
// The DOM level 3 spec has a good index of these
// http://www.w3.org/TR/DOM-Level-3-Events/#event-types

/** @type {?function (Event)} */ Element.prototype.onabort;
/** @type {?function (Event)} */ Element.prototype.onbeforeinput;
/** @type {?function (Event)} */ Element.prototype.onbeforeunload;
/** @type {?function (Event)} */ Element.prototype.onblur;
/** @type {?function (Event)} */ Element.prototype.onchange;
/** @type {?function (Event)} */ Element.prototype.onclick;
/** @type {?function (Event)} */ Element.prototype.oncompositionstart;
/** @type {?function (Event)} */ Element.prototype.oncompositionupdate;
/** @type {?function (Event)} */ Element.prototype.oncompositionend;
/** @type {?function (Event)} */ Element.prototype.oncontextmenu;
/** @type {?function (Event)} */ Element.prototype.oncopy;
/** @type {?function (Event)} */ Element.prototype.oncut;
/** @type {?function (Event)} */ Element.prototype.ondblclick;
/** @type {?function (Event)} */ Element.prototype.onerror;
/** @type {?function (Event)} */ Element.prototype.onfocus;
/** @type {?function (Event)} */ Element.prototype.onfocusin;
/** @type {?function (Event)} */ Element.prototype.onfocusout;
/** @type {?function (Event)} */ Element.prototype.oninput;
/** @type {?function (Event)} */ Element.prototype.onkeydown;
/** @type {?function (Event)} */ Element.prototype.onkeypress;
/** @type {?function (Event)} */ Element.prototype.onkeyup;
/** @type {?function (Event)} */ Element.prototype.onload;
/** @type {?function (Event)} */ Element.prototype.onunload;
/** @type {?function (Event)} */ Element.prototype.onmousedown;
/** @type {?function (Event)} */ Element.prototype.onmousemove;
/** @type {?function (Event)} */ Element.prototype.onmouseout;
/** @type {?function (Event)} */ Element.prototype.onmouseover;
/** @type {?function (Event)} */ Element.prototype.onmouseup;
/** @type {?function (Event)} */ Element.prototype.onmousewheel;
/** @type {?function (Event)} */ Element.prototype.onpaste;
/** @type {?function (Event)} */ Element.prototype.onreset;
/** @type {?function (Event)} */ Element.prototype.onresize;
/** @type {?function (Event)} */ Element.prototype.onscroll;
/** @type {?function (Event)} */ Element.prototype.onselect;
/** @type {?function (Event=)} */ Element.prototype.onsubmit;
/** @type {?function (Event)} */ Element.prototype.ontextinput;
/** @type {?function (Event)} */ Element.prototype.onwheel;

/**
 * @constructor
 * @extends {CharacterData}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1312295772
 */
function Text() {}

/**
 * @param {number} offset
 * @return {Text}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-38853C1D
 */
Text.prototype.splitText = function(offset) {};

/**
 * @constructor
 * @extends {CharacterData}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1728279322
 */
function Comment() {}

/**
 * @constructor
 * @extends {Text}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-667469212
 */
function CDATASection() {}

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-412266927
 */
function DocumentType() {}

/**
 * @type {NamedNodeMap}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1788794630
 */
DocumentType.prototype.entities;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1844763134
 */
DocumentType.prototype.name;

/**
 * @type {NamedNodeMap}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-D46829EF
 */
DocumentType.prototype.notations;

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-5431D1B9
 */
function Notation() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-54F2B4D0
 */
Notation.prototype.publicId;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-E8AAB1D0
 */
Notation.prototype.systemId;

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-527DCFF2
 */
function Entity() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-D7303025
 */
Entity.prototype.publicId;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-D7C29F3E
 */
Entity.prototype.systemId;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-6ABAEB38
 */
Entity.prototype.notationName;

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-11C98490
 */
function EntityReference() {}

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1004215813
 */
function ProcessingInstruction() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-837822393
 */
ProcessingInstruction.prototype.data;

/**
 * @type {string}
 * @see http://www.w3.org/TR/1998/REC-DOM-Level-1-19981001/level-one-core.html#ID-1478689192
 */
ProcessingInstruction.prototype.target;


/**
 * @constructor
 * @implements {EventTarget}
 */
function Window() {}
Window.prototype.Window;

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Window.prototype.addEventListener = function(type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Window.prototype.removeEventListener = function(type, listener, opt_useCapture)
    {};

/** @override */
Window.prototype.dispatchEvent = function(evt) {};

/** @type {?function (Event)} */ Window.prototype.onabort;
/** @type {?function (Event)} */ Window.prototype.onbeforeunload;
/** @type {?function (Event)} */ Window.prototype.onblur;
/** @type {?function (Event)} */ Window.prototype.onchange;
/** @type {?function (Event)} */ Window.prototype.onclick;
/** @type {?function (Event)} */ Window.prototype.onclose;
/** @type {?function (Event)} */ Window.prototype.oncontextmenu;
/** @type {?function (Event)} */ Window.prototype.ondblclick;
/** @type {?function (Event)} */ Window.prototype.ondragdrop;
// onerror has a special signature.
// See https://developer.mozilla.org/en/DOM/window.onerror
// and http://msdn.microsoft.com/en-us/library/cc197053(VS.85).aspx
/** @type {?function (string, string, number)} */
Window.prototype.onerror;
/** @type {?function (Event)} */ Window.prototype.onfocus;
/** @type {?function (Event)} */ Window.prototype.onhashchange;
/** @type {?function (Event)} */ Window.prototype.onkeydown;
/** @type {?function (Event)} */ Window.prototype.onkeypress;
/** @type {?function (Event)} */ Window.prototype.onkeyup;
/** @type {?function (Event)} */ Window.prototype.onload;
/** @type {?function (Event)} */ Window.prototype.onmousedown;
/** @type {?function (Event)} */ Window.prototype.onmousemove;
/** @type {?function (Event)} */ Window.prototype.onmouseout;
/** @type {?function (Event)} */ Window.prototype.onmouseover;
/** @type {?function (Event)} */ Window.prototype.onmouseup;
/** @type {?function (Event)} */ Window.prototype.onmousewheel;
/** @type {?function (Event)} */ Window.prototype.onpaint;
/** @type {?function (Event)} */ Window.prototype.onpopstate;
/** @type {?function (Event)} */ Window.prototype.onreset;
/** @type {?function (Event)} */ Window.prototype.onresize;
/** @type {?function (Event)} */ Window.prototype.onscroll;
/** @type {?function (Event)} */ Window.prototype.onselect;
/** @type {?function (Event=)} */ Window.prototype.onsubmit;
/** @type {?function (Event)} */ Window.prototype.onunload;
/** @type {?function (Event)} */ Window.prototype.onwheel;

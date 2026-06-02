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
 * @fileoverview Definitions for W3C's DOM Level 2 specification.
 *  This file depends on w3c_dom1.js.
 *  The whole file has been fully type annotated.
 *  Created from
 *  http://www.w3.org/TR/REC-DOM-Level-1/ecma-script-language-binding.html
 *
 * @externs
 * @author stevey@google.com (Steve Yegge)
 */
// All the provided definitions have been type annotated.

/**
 * @constructor
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75708506
 */
function HTMLCollection() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40057551
 */
HTMLCollection.prototype.length;

/**
 * @param {number} index
 * @return {Node}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-33262535
 * @nosideeffects
 */
HTMLCollection.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-2-HTML/html.html#HTMLOptionsCollection
 */
function HTMLOptionsCollection() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-HTML/html.html#HTMLOptionsCollection-length
 */
HTMLOptionsCollection.prototype.length;

/**
 * @param {number} index
 * @return {Node}
 * @see http://www.w3.org/TR/DOM-Level-2-HTML/html.html#HTMLOptionsCollection-item
 * @nosideeffects
 */
HTMLOptionsCollection.prototype.item = function(index) {};

/**
 * @constructor
 * @extends {Document}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-26809268
 */
function HTMLDocument() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-18446827
 */
HTMLDocument.prototype.title;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-95229140
 */
HTMLDocument.prototype.referrer;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-2250147
 */
HTMLDocument.prototype.domain;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-46183437
 */
HTMLDocument.prototype.URL;

/**
 * @type {HTMLBodyElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-56360201
 */
HTMLDocument.prototype.body;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-90379117
 */
HTMLDocument.prototype.images;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-85113862
 */
HTMLDocument.prototype.applets;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-7068919
 */
HTMLDocument.prototype.links;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-1689064
 */
HTMLDocument.prototype.forms;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-7577272
 */
HTMLDocument.prototype.anchors;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-8747038
 */
HTMLDocument.prototype.cookie;

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-72161170
 * @override
 */
HTMLDocument.prototype.open = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-98948567
 * @override
 */
HTMLDocument.prototype.close = function() {};

/**
 * @param {string} text
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75233634
 * @override
 */
HTMLDocument.prototype.write = function(text) {};

/**
 * @param {string} text
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-35318390
 * @override
 */
HTMLDocument.prototype.writeln = function(text) {};

/**
 * @param {string} elementName
 * @return {!NodeList}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-71555259
 * @override
 * @nosideeffects
 */
HTMLDocument.prototype.getElementsByName = function(elementName) {};

/**
 * @param {Node} root
 * @param {number=} whatToShow
 * @param {NodeFilter=} filter
 * @param {boolean=} entityReferenceExpansion
 * @return {!NodeIterator}
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-Document
 * @nosideeffects
 */
HTMLDocument.prototype.createNodeIterator = function(
    root, whatToShow, filter, entityReferenceExpansion) {};

/**
 * @param {Node} root
 * @param {number=} whatToShow
 * @param {NodeFilter=} filter
 * @param {boolean=} entityReferenceExpansion
 * @return {!TreeWalker}
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-Document
 * @nosideeffects
 */
HTMLDocument.prototype.createTreeWalker = function(
    root, whatToShow, filter, entityReferenceExpansion) {};


/** @typedef {{
  createNodeIterator: function(Node, number=, NodeFilter=, boolean=) : NodeIterator,
  createTreeWalker: function(Node, number=, NodeFilter=, boolean=) : TreeWalker
}} */
var TraversalDocument;

/**
 * @interface
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-NodeFilter
*/
function NodeFilter() {}

/* Constants for whatToShow */
/** @const {number} */ NodeFilter.SHOW_ALL;
/** @const {number} */ NodeFilter.SHOW_ATTRIBUTE;
/** @const {number} */ NodeFilter.SHOW_CDATA_SECTION;
/** @const {number} */ NodeFilter.SHOW_COMMENT;
/** @const {number} */ NodeFilter.SHOW_DOCUMENT;
/** @const {number} */ NodeFilter.SHOW_DOCUMENT_FRAGMENT;
/** @const {number} */ NodeFilter.SHOW_DOCUMENT_TYPE;
/** @const {number} */ NodeFilter.SHOW_ELEMENT;
/** @const {number} */ NodeFilter.SHOW_ENTITY;
/** @const {number} */ NodeFilter.SHOW_ENTITY_REFERENCE;
/** @const {number} */ NodeFilter.SHOW_NOTATION;
/** @const {number} */ NodeFilter.SHOW_PROCESSING_INSTRUCTION;
/** @const {number} */ NodeFilter.SHOW_TEXT;

/* Consants for acceptNode */
/** @const {number} */ NodeFilter.FILTER_ACCEPT;
/** @const {number} */ NodeFilter.FILTER_REJECT;
/** @const {number} */ NodeFilter.FILTER_SKIP;

/**
 * @param {Node} n
 * @return {number} Any of NodeFilter.FILTER_* constants.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-NodeFilter-acceptNode
*/
NodeFilter.prototype.acceptNode = function(n) {};

/**
 * @interface
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-NodeIterator
*/
function NodeIterator() {}

/**
 * Detach and invalidate the NodeIterator.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-NodeIterator-detach
 */
NodeIterator.prototype.detach = function() {};

/**
 * @return {Node} Next node in the set.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-NodeIterator-nextNode
 */
NodeIterator.prototype.nextNode = function() {};

/**
 * @return {Node} Previous node in the set.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-NodeIterator-previousNode
 */
NodeIterator.prototype.previousNode = function() {};

/**
 * @interface
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker
*/
function TreeWalker() {}

/**
 * @return {?Node} The new Node or null.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-firstChild
 */
TreeWalker.prototype.firstChild = function() {};

/**
 * @return {?Node} The new Node or null..
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-lastChild
 */
TreeWalker.prototype.lastChild = function() {};

/**
 * @return {?Node} The new Node or null.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-nextNode
 */
TreeWalker.prototype.nextNode = function() {};

/**
 * @return {?Node} The new Node or null.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-nextSibling
 */
TreeWalker.prototype.nextSibling = function() {};

/**
 * @return {?Node} The new Node or null.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-parentNode
 */
TreeWalker.prototype.parentNode = function() {};

/**
 * @return {?Node} The new Node or null.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-previousNode
 */
TreeWalker.prototype.previousNode = function() {};

/**
 * @return {?Node} The new Node or null.
 * @see http://www.w3.org/TR/DOM-Level-2-Traversal-Range/traversal.html#Traversal-TreeWalker-previousSibling
 */
TreeWalker.prototype.previousSibling = function() {};

/**
 * @type {Node}
 */
TreeWalker.prototype.root;

/**
 * @type {number}
 */
TreeWalker.prototype.whatToShow;

/**
 * @type {NodeFilter}
 */
TreeWalker.prototype.filter;

/**
 * @type {boolean}
 */
TreeWalker.prototype.expandEntityReference;

/**
 * @type {Node}
 */
TreeWalker.prototype.currentNode;

/**
 * @constructor
 * @extends {Element}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58190037
 */
function HTMLElement() {}

/**
 * @implicitCast
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-63534901
 */
HTMLElement.prototype.id;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-78276800
 */
HTMLElement.prototype.title;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-59132807
 */
HTMLElement.prototype.lang;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-52460740
 */
HTMLElement.prototype.dir;

/**
 * @implicitCast
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-95362176
 */
HTMLElement.prototype.className;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-33759296
 */
function HTMLHtmlElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-9383775
 */
HTMLHtmlElement.prototype.version;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77253168
 */
function HTMLHeadElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96921909
 */
HTMLHeadElement.prototype.profile;

/**
 * @constructor
 * @extends {HTMLElement}
 * @implements {LinkStyle}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-35143001
 */
function HTMLLinkElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87355129
 */
HTMLLinkElement.prototype.disabled;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-63954491
 */
HTMLLinkElement.prototype.charset;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-33532588
 */
HTMLLinkElement.prototype.href;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-85145682
 */
HTMLLinkElement.prototype.hreflang;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75813125
 */
HTMLLinkElement.prototype.media;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-41369587
 */
HTMLLinkElement.prototype.rel;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40715461
 */
HTMLLinkElement.prototype.rev;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-84183095
 */
HTMLLinkElement.prototype.target;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-32498296
 */
HTMLLinkElement.prototype.type;

/** @type {StyleSheet} */
HTMLLinkElement.prototype.sheet;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79243169
 */
function HTMLTitleElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77500413
 */
HTMLTitleElement.prototype.text;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-37041454
 */
function HTMLMetaElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87670826
 */
HTMLMetaElement.prototype.content;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77289449
 */
HTMLMetaElement.prototype.httpEquiv;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-31037081
 */
HTMLMetaElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-35993789
 */
HTMLMetaElement.prototype.scheme;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-73629039
 */
function HTMLBaseElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-65382887
 */
HTMLBaseElement.prototype.href;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-73844298
 */
HTMLBaseElement.prototype.target;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-85283003
 */
function HTMLIsIndexElement() {}

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87069980
 */
HTMLIsIndexElement.prototype.form;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-33589862
 */
HTMLIsIndexElement.prototype.prompt;

/**
 * @constructor
 * @extends {HTMLElement}
 * @implements {LinkStyle}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-16428977
 */
function HTMLStyleElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-51162010
 */
HTMLStyleElement.prototype.disabled;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-76412738
 */
HTMLStyleElement.prototype.media;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-22472002
 */
HTMLStyleElement.prototype.type;

/** @type {StyleSheet} */
HTMLStyleElement.prototype.sheet;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-62018039
 */
function HTMLBodyElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-59424581
 */
HTMLBodyElement.prototype.aLink;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-37574810
 */
HTMLBodyElement.prototype.background;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-24940084
 */
HTMLBodyElement.prototype.bgColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-7662206
 */
HTMLBodyElement.prototype.link;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-73714763
 */
HTMLBodyElement.prototype.text;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-83224305
 */
HTMLBodyElement.prototype.vLink;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40002357
 */
function HTMLFormElement() {}

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-76728479
 */
HTMLFormElement.prototype.elements;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#HTML-HTMLFormElement-length
 */
HTMLFormElement.prototype.length;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-22051454
 */
HTMLFormElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-19661795
 */
HTMLFormElement.prototype.acceptCharset;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-74049184
 */
HTMLFormElement.prototype.action;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-84227810
 */
HTMLFormElement.prototype.enctype;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-82545539
 */
HTMLFormElement.prototype.method;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6512890
 */
HTMLFormElement.prototype.target;

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-76767676
 */
HTMLFormElement.prototype.submit = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-76767677
 */
HTMLFormElement.prototype.reset = function() {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-94282980
 */
function HTMLSelectElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58783172
 */
HTMLSelectElement.prototype.type;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-85676760
 */
HTMLSelectElement.prototype.selectedIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-59351919
 */
HTMLSelectElement.prototype.value;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-5933486
 */
HTMLSelectElement.prototype.length;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-20489458
 */
HTMLSelectElement.prototype.form;

/**
 * @type {!HTMLOptionsCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-30606413
 */
HTMLSelectElement.prototype.options;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79102918
 */
HTMLSelectElement.prototype.disabled;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-13246613
 */
HTMLSelectElement.prototype.multiple;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-41636323
 */
HTMLSelectElement.prototype.name;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-18293826
 */
HTMLSelectElement.prototype.size;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40676705
 */
HTMLSelectElement.prototype.tabIndex;

/**
 * @param {HTMLElement} element
 * @param {HTMLElement} before
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-14493106
 */
HTMLSelectElement.prototype.add = function(element, before) {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-28216144
 * @override
 */
HTMLSelectElement.prototype.blur = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-32130014
 * @override
 */
HTMLSelectElement.prototype.focus = function() {};

/**
 * @param {number} index
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-33404570
 */
HTMLSelectElement.prototype.remove = function(index) {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-38450247
 */
function HTMLOptGroupElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-15518803
 */
HTMLOptGroupElement.prototype.disabled;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-95806054
 */
HTMLOptGroupElement.prototype.label;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70901257
 */
function HTMLOptionElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-37770574
 */
HTMLOptionElement.prototype.defaultSelected;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-23482473
 */
HTMLOptionElement.prototype.disabled;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-17116503
 */
HTMLOptionElement.prototype.form;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-14038413
 */
HTMLOptionElement.prototype.index;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40736115
 */
HTMLOptionElement.prototype.label;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70874476
 */
HTMLOptionElement.prototype.selected;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-48154426
 */
HTMLOptionElement.prototype.text;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6185554
 */
HTMLOptionElement.prototype.value;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6043025
 */
function HTMLInputElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-15328520
 */
HTMLInputElement.prototype.accept;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-59914154
 */
HTMLInputElement.prototype.accessKey;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96991182
 */
HTMLInputElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-92701314
 */
HTMLInputElement.prototype.alt;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-30233917
 */
HTMLInputElement.prototype.checked;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-20509171
 */
HTMLInputElement.prototype.defaultChecked;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-26091157
 */
HTMLInputElement.prototype.defaultValue;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-50886781
 */
HTMLInputElement.prototype.disabled;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-63239895
 */
HTMLInputElement.prototype.form;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-54719353
 */
HTMLInputElement.prototype.maxLength;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-89658498
 */
HTMLInputElement.prototype.name;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-88461592
 */
HTMLInputElement.prototype.readOnly;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79659438
 */
HTMLInputElement.prototype.size;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-97320704
 */
HTMLInputElement.prototype.src;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-62176355
 */
HTMLInputElement.prototype.tabIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-62883744
 */
HTMLInputElement.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-32463706
 */
HTMLInputElement.prototype.useMap;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-49531485
 */
HTMLInputElement.prototype.value;

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-26838235
 * @override
 */
HTMLInputElement.prototype.blur = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-2651361
 * @override
 */
HTMLInputElement.prototype.click = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-65996295
 * @override
 */
HTMLInputElement.prototype.focus = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-34677168
 */
HTMLInputElement.prototype.select = function() {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-24874179
 */
function HTMLTextAreaElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-93102991
 */
HTMLTextAreaElement.prototype.accessKey;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-51387225
 */
HTMLTextAreaElement.prototype.cols;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-36152213
 */
HTMLTextAreaElement.prototype.defaultValue;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-98725443
 */
HTMLTextAreaElement.prototype.disabled;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-18911464
 */
HTMLTextAreaElement.prototype.form;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70715578
 */
HTMLTextAreaElement.prototype.name;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39131423
 */
HTMLTextAreaElement.prototype.readOnly;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-46975887
 */
HTMLTextAreaElement.prototype.rows;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-60363303
 */
HTMLTextAreaElement.prototype.tabIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#HTML-HTMLTextAreaElement-type
 */
HTMLTextAreaElement.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70715579
 */
HTMLTextAreaElement.prototype.value;

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6750689
 * @override
 */
HTMLTextAreaElement.prototype.blur = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39055426
 * @override
 */
HTMLTextAreaElement.prototype.focus = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-48880622
 */
HTMLTextAreaElement.prototype.select = function() {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-34812697
 */
function HTMLButtonElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-73169431
 */
HTMLButtonElement.prototype.accessKey;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-92757155
 */
HTMLButtonElement.prototype.disabled;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-71254493
 */
HTMLButtonElement.prototype.form;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-11029910
 */
HTMLButtonElement.prototype.name;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39190908
 */
HTMLButtonElement.prototype.tabIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-27430092
 */
HTMLButtonElement.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-72856782
 */
HTMLButtonElement.prototype.value;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-13691394
 */
function HTMLLabelElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-43589892
 */
HTMLLabelElement.prototype.accessKey;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-32480901
 */
HTMLLabelElement.prototype.form;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96509813
 */
HTMLLabelElement.prototype.htmlFor;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-7365882
 */
function HTMLFieldSetElement() {}

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75392630
 */
HTMLFieldSetElement.prototype.form;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-21482039
 */
function HTMLLegendElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-11297832
 */
HTMLLegendElement.prototype.accessKey;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79538067
 */
HTMLLegendElement.prototype.align;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-29594519
 */
HTMLLegendElement.prototype.form;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-86834457
 */
function HTMLUListElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39864178
 */
HTMLUListElement.prototype.compact;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96874670
 */
HTMLUListElement.prototype.type;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58056027
 */
function HTMLOListElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-76448506
 */
HTMLOListElement.prototype.compact;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-14793325
 */
HTMLOListElement.prototype.start;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40971103
 */
HTMLOListElement.prototype.type;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-52368974
 */
function HTMLDListElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-21738539
 */
HTMLDListElement.prototype.compact;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-71600284
 */
function HTMLDirectoryElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75317739
 */
HTMLDirectoryElement.prototype.compact;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-72509186
 * @see http://www.w3.org/html/wg/drafts/html/master/interactive-elements.html#the-menu-element
 */
function HTMLMenuElement() {}

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-68436464
 */
HTMLMenuElement.prototype.compact;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-74680021
 */
function HTMLLIElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-52387668
 */
HTMLLIElement.prototype.type;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-45496263
 */
HTMLLIElement.prototype.value;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-22445964
 */
function HTMLDivElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70908791
 */
HTMLDivElement.prototype.align;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-84675076
 */
function HTMLParagraphElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-53465507
 */
HTMLParagraphElement.prototype.align;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-43345119
 */
function HTMLHeadingElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6796462
 */
HTMLHeadingElement.prototype.align;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70319763
 */
function HTMLQuoteElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-53895598
 */
HTMLQuoteElement.prototype.cite;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-11383425
 */
function HTMLPreElement() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-13894083
 */
HTMLPreElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-56836063
 */
function HTMLBRElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-82703081
 */
HTMLBRElement.prototype.clear;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-32774408
 */
function HTMLBaseFontElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87502302
 */
HTMLBaseFontElement.prototype.color;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-88128969
 */
HTMLBaseFontElement.prototype.face;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-38930424
 */
HTMLBaseFontElement.prototype.size;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-43943847
 */
function HTMLFontElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-53532975
 */
HTMLFontElement.prototype.color;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-55715655
 */
HTMLFontElement.prototype.face;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-90127284
 */
HTMLFontElement.prototype.size;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-68228811
 */
function HTMLHRElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-15235012
 */
HTMLHRElement.prototype.align;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79813978
 */
HTMLHRElement.prototype.noShade;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77612587
 */
HTMLHRElement.prototype.size;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87744198
 */
HTMLHRElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79359609
 */
function HTMLModElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75101708
 */
HTMLModElement.prototype.cite;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-88432678
 */
HTMLModElement.prototype.dateTime;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-48250443
 */
function HTMLAnchorElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-89647724
 */
HTMLAnchorElement.prototype.accessKey;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-67619266
 */
HTMLAnchorElement.prototype.charset;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-92079539
 */
HTMLAnchorElement.prototype.coords;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-88517319
 */
HTMLAnchorElement.prototype.href;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87358513
 */
HTMLAnchorElement.prototype.hreflang;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-32783304
 */
HTMLAnchorElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-3815891
 */
HTMLAnchorElement.prototype.rel;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58259771
 */
HTMLAnchorElement.prototype.rev;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-49899808
 */
HTMLAnchorElement.prototype.shape;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-41586466
 */
HTMLAnchorElement.prototype.tabIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6414197
 */
HTMLAnchorElement.prototype.target;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-63938221
 */
HTMLAnchorElement.prototype.type;

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-65068939
 * @override
 */
HTMLAnchorElement.prototype.blur = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-47150313
 * @override
 */
HTMLAnchorElement.prototype.focus = function() {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-17701901
 */
function HTMLImageElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-3211094
 */
HTMLImageElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-95636861
 */
HTMLImageElement.prototype.alt;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-136671
 */
HTMLImageElement.prototype.border;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-91561496
 */
HTMLImageElement.prototype.height;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-53675471
 */
HTMLImageElement.prototype.hspace;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58983880
 */
HTMLImageElement.prototype.isMap;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77376969
 */
HTMLImageElement.prototype.longDesc;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-91256910
 */
HTMLImageElement.prototype.lowSrc;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-47534097
 */
HTMLImageElement.prototype.name;

/**
 * @type {string}
 * @implicitCast
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-87762984
 */
HTMLImageElement.prototype.src;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-35981181
 */
HTMLImageElement.prototype.useMap;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-85374897
 */
HTMLImageElement.prototype.vspace;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-13839076
 */
HTMLImageElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-9893177
 */
function HTMLObjectElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-16962097
 */
HTMLObjectElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-47783837
 */
HTMLObjectElement.prototype.archive;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-82818419
 */
HTMLObjectElement.prototype.border;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75241146
 */
HTMLObjectElement.prototype.code;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-25709136
 */
HTMLObjectElement.prototype.codeBase;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-19945008
 */
HTMLObjectElement.prototype.codeType;

/**
 * @type {Document}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-38538621
 */
HTMLObjectElement.prototype.contentDocument;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-81766986
 */
HTMLObjectElement.prototype.data;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-942770
 */
HTMLObjectElement.prototype.declare;

/**
 * @type {HTMLFormElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-46094773
 */
HTMLObjectElement.prototype.form;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-88925838
 */
HTMLObjectElement.prototype.height;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-17085376
 */
HTMLObjectElement.prototype.hspace;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-20110362
 */
HTMLObjectElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-25039673
 */
HTMLObjectElement.prototype.standby;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-27083787
 */
HTMLObjectElement.prototype.tabIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-91665621
 */
HTMLObjectElement.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6649772
 */
HTMLObjectElement.prototype.useMap;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-8682483
 */
HTMLObjectElement.prototype.vspace;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-38538620
 */
HTMLObjectElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-64077273
 */
function HTMLParamElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-59871447
 */
HTMLParamElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-18179888
 */
HTMLParamElement.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77971357
 */
HTMLParamElement.prototype.value;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-23931872
 */
HTMLParamElement.prototype.valueType;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-31006348
 */
function HTMLAppletElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-8049912
 */
HTMLAppletElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58610064
 */
HTMLAppletElement.prototype.alt;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-14476360
 */
HTMLAppletElement.prototype.archive;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-61509645
 */
HTMLAppletElement.prototype.code;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6581160
 */
HTMLAppletElement.prototype.codeBase;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-90184867
 */
HTMLAppletElement.prototype.height;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-1567197
 */
HTMLAppletElement.prototype.hspace;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39843695
 */
HTMLAppletElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-93681523
 */
HTMLAppletElement.prototype.object;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-22637173
 */
HTMLAppletElement.prototype.vspace;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-16526327
 */
HTMLAppletElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-94109203
 */
function HTMLMapElement() {}

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-71838730
 */
HTMLMapElement.prototype.areas;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-52696514
 */
HTMLMapElement.prototype.name;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-26019118
 */
function HTMLAreaElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-57944457
 */
HTMLAreaElement.prototype.accessKey;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39775416
 */
HTMLAreaElement.prototype.alt;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-66021476
 */
HTMLAreaElement.prototype.coords;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-34672936
 */
HTMLAreaElement.prototype.href;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-61826871
 */
HTMLAreaElement.prototype.noHref;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-85683271
 */
HTMLAreaElement.prototype.shape;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-8722121
 */
HTMLAreaElement.prototype.tabIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-46054682
 */
HTMLAreaElement.prototype.target;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-81598695
 */
function HTMLScriptElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-35305677
 */
HTMLScriptElement.prototype.charset;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-93788534
 */
HTMLScriptElement.prototype.defer;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-56700403
 */
HTMLScriptElement.prototype.event;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-66979266
 */
HTMLScriptElement.prototype.htmlFor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-75147231
 */
HTMLScriptElement.prototype.src;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-46872999
 */
HTMLScriptElement.prototype.text;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-30534818
 */
HTMLScriptElement.prototype.type;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-64060425
 */
function HTMLTableElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-23180977
 */
HTMLTableElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-83532985
 */
HTMLTableElement.prototype.bgColor;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-50969400
 */
HTMLTableElement.prototype.border;

/**
 * @type {HTMLTableCaptionElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-14594520
 */
HTMLTableElement.prototype.caption;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-59162158
 */
HTMLTableElement.prototype.cellPadding;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-68907883
 */
HTMLTableElement.prototype.cellSpacing;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-64808476
 */
HTMLTableElement.prototype.frame;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6156016
 */
HTMLTableElement.prototype.rows;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-26347553
 */
HTMLTableElement.prototype.rules;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-44998528
 */
HTMLTableElement.prototype.summary;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-63206416
 */
HTMLTableElement.prototype.tBodies;

/**
 * @type {HTMLTableSectionElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-64197097
 */
HTMLTableElement.prototype.tFoot;

/**
 * @type {HTMLTableSectionElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-9530944
 */
HTMLTableElement.prototype.tHead;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-77447361
 */
HTMLTableElement.prototype.width;

/**
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96920263
 */
HTMLTableElement.prototype.createCaption = function() {};

/**
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-8453710
 */
HTMLTableElement.prototype.createTFoot = function() {};

/**
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70313345
 */
HTMLTableElement.prototype.createTHead = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-22930071
 */
HTMLTableElement.prototype.deleteCaption = function() {};

/**
 * @param {number} index
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-13114938
 */
HTMLTableElement.prototype.deleteRow = function(index) {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-78363258
 */
HTMLTableElement.prototype.deleteTFoot = function() {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-38310198
 */
HTMLTableElement.prototype.deleteTHead = function() {};

/**
 * @param {number} index
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-39872903
 */
HTMLTableElement.prototype.insertRow = function(index) {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-12035137
 */
function HTMLTableCaptionElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79875068
 */
HTMLTableCaptionElement.prototype.align;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-84150186
 */
function HTMLTableColElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-31128447
 */
HTMLTableColElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-9447412
 */
HTMLTableColElement.prototype.ch;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-57779225
 */
HTMLTableColElement.prototype.chOff;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96511335
 */
HTMLTableColElement.prototype.span;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-83291710
 */
HTMLTableColElement.prototype.vAlign;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-25196799
 */
HTMLTableColElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-67417573
 */
function HTMLTableSectionElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-40530119
 */
HTMLTableSectionElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-83470012
 */
HTMLTableSectionElement.prototype.ch;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-53459732
 */
HTMLTableSectionElement.prototype.chOff;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-52092650
 */
HTMLTableSectionElement.prototype.rows;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-4379116
 */
HTMLTableSectionElement.prototype.vAlign;

/**
 * @param {number} index
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-5625626
 */
HTMLTableSectionElement.prototype.deleteRow = function(index) {};

/**
 * @param {number} index
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-93995626
 */
HTMLTableSectionElement.prototype.insertRow = function(index) {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-6986576
 */
function HTMLTableRowElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-74098257
 */
HTMLTableRowElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-18161327
 */
HTMLTableRowElement.prototype.bgColor;

/**
 * @type {HTMLCollection}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-67349879
 */
HTMLTableRowElement.prototype.cells;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-16230502
 */
HTMLTableRowElement.prototype.ch;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-68207461
 */
HTMLTableRowElement.prototype.chOff;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-67347567
 */
HTMLTableRowElement.prototype.rowIndex;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-79105901
 */
HTMLTableRowElement.prototype.sectionRowIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-90000058
 */
HTMLTableRowElement.prototype.vAlign;

/**
 * @param {number} index
 * @return {undefined}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-11738598
 */
HTMLTableRowElement.prototype.deleteCell = function(index) {};

/**
 * @param {number} index
 * @return {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-68927016
 */
HTMLTableRowElement.prototype.insertCell = function(index) {};

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-82915075
 */
function HTMLTableCellElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-74444037
 */
HTMLTableCellElement.prototype.abbr;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-98433879
 */
HTMLTableCellElement.prototype.align;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-76554418
 */
HTMLTableCellElement.prototype.axis;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-88135431
 */
HTMLTableCellElement.prototype.bgColor;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-80748363
 */
HTMLTableCellElement.prototype.cellIndex;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-30914780
 */
HTMLTableCellElement.prototype.ch;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-20144310
 */
HTMLTableCellElement.prototype.chOff;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-84645244
 */
HTMLTableCellElement.prototype.colSpan;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-89104817
 */
HTMLTableCellElement.prototype.headers;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-83679212
 */
HTMLTableCellElement.prototype.height;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-62922045
 */
HTMLTableCellElement.prototype.noWrap;

/**
 * @type {number}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-48237625
 */
HTMLTableCellElement.prototype.rowSpan;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-36139952
 */
HTMLTableCellElement.prototype.scope;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-58284221
 */
HTMLTableCellElement.prototype.vAlign;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-27480795
 */
HTMLTableCellElement.prototype.width;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-43829095
 */
function HTMLFrameSetElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-98869594
 */
HTMLFrameSetElement.prototype.cols;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-19739247
 */
HTMLFrameSetElement.prototype.rows;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-97790553
 */
function HTMLFrameElement() {}

/**
 * @type {Document}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-78799536
 */
HTMLFrameElement.prototype.contentDocument;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-11858633
 */
HTMLFrameElement.prototype.frameBorder;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-7836998
 */
HTMLFrameElement.prototype.longDesc;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-55569778
 */
HTMLFrameElement.prototype.marginHeight;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-8369969
 */
HTMLFrameElement.prototype.marginWidth;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-91128709
 */
HTMLFrameElement.prototype.name;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-80766578
 */
HTMLFrameElement.prototype.noResize;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-45411424
 */
HTMLFrameElement.prototype.scrolling;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-78799535
 */
HTMLFrameElement.prototype.src;

/**
 * @constructor
 * @extends {HTMLElement}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-50708718
 */
function HTMLIFrameElement() {}

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-11309947
 */
HTMLIFrameElement.prototype.align;

/**
 * @type {Document}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-67133006
 */
HTMLIFrameElement.prototype.contentDocument;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-22463410
 */
HTMLIFrameElement.prototype.frameBorder;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-1678118
 */
HTMLIFrameElement.prototype.height;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-70472105
 */
HTMLIFrameElement.prototype.longDesc;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-91371294
 */
HTMLIFrameElement.prototype.marginHeight;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-66486595
 */
HTMLIFrameElement.prototype.marginWidth;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-96819659
 */
HTMLIFrameElement.prototype.name;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-36369822
 */
HTMLIFrameElement.prototype.scrolling;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-43933957
 */
HTMLIFrameElement.prototype.src;

/**
 * @type {string}
 * @see http://www.w3.org/TR/2000/CR-DOM-Level-2-20000510/html.html#ID-67133005
 */
HTMLIFrameElement.prototype.width;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-258A00AF
 */
DOMException.INVALID_STATE_ERR = 11;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-258A00AF
 */
DOMException.SYNTAX_ERR = 12;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-258A00AF
 */
DOMException.INVALID_MODIFICATION_ERR = 13;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-258A00AF
 */
DOMException.NAMESPACE_ERR = 14;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Core/core.html#ID-258A00AF
 */
DOMException.INVALID_ACCESS_ERR = 15;

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
 * @fileoverview Definitions for all the extensions over
 *  W3C's DOM specification by Gecko. This file depends on
 *  w3c_dom2.js.
 *
 * When a non-standard extension appears in both Gecko and IE, we put
 * it in gecko_dom.js
 *
 * @externs
 */

// TODO: Almost all of it has not been annotated with types.

// Gecko DOM;

/**
 * Mozilla only???
 * @constructor
 * @extends {HTMLElement}
 */
function HTMLSpanElement() {}

/**
 * @see https://developer.mozilla.org/en/Components_object
 */
Window.prototype.Components;

/**
 * @type Window
 * @see https://developer.mozilla.org/en/DOM/window.content
 */
Window.prototype.content;

/**
 * @type {boolean}
 * @see https://developer.mozilla.org/en/DOM/window.closed
 */
Window.prototype.closed;

/** @see https://developer.mozilla.org/en/DOM/window.controllers */
Window.prototype.controllers;

/** @see https://developer.mozilla.org/en/DOM/window.crypto */
Window.prototype.crypto;

/**
 * Gets/sets the status bar text for the given window.
 * @type {string}
 * @see https://developer.mozilla.org/en/DOM/window.defaultStatus
 */
Window.prototype.defaultStatus;

/** @see https://developer.mozilla.org/en/DOM/window.dialogArguments */
Window.prototype.dialogArguments;

/** @see https://developer.mozilla.org/en/DOM/window.directories */
Window.prototype.directories;

/**
 * @type {HTMLObjectElement|HTMLIFrameElement|null}
 * @see https://developer.mozilla.org/en/DOM/window.frameElement
 */
Window.prototype.frameElement;

/**
 * Allows lookup of frames by index or by name.
 * @type {?Object}
 * @see https://developer.mozilla.org/en/DOM/window.frames
 */
Window.prototype.frames;

/**
 * @type {boolean}
 * @see https://developer.mozilla.org/en/DOM/window.fullScreen
 */
Window.prototype.fullScreen;

/**
 * @see https://developer.mozilla.org/en/DOM/Storage#globalStorage
 */
Window.prototype.globalStorage;

/**
 * @type {!History}
 * @see https://developer.mozilla.org/en/DOM/window.history
 */
Window.prototype.history;

/**
 * Returns the number of frames (either frame or iframe elements) in the
 * window.
 *
 * @type {number}
 * @see https://developer.mozilla.org/en/DOM/window.length
 */
Window.prototype.length;

/**
 * @type {!Location}
 * @implicitCast
 * @see https://developer.mozilla.org/en/DOM/window.location
 */
Window.prototype.location;

/**
 * @see https://developer.mozilla.org/en/DOM/window.locationbar
 */
Window.prototype.locationbar;

/**
 * @see https://developer.mozilla.org/en/DOM/window.menubar
 */
Window.prototype.menubar;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/DOM/window.name
 */
Window.prototype.name;

/**
 * @type {Navigator}
 * @see https://developer.mozilla.org/en/DOM/window.navigator
 */
Window.prototype.navigator;

/**
 * @type {?Window}
 * @see https://developer.mozilla.org/en/DOM/window.opener
 */
Window.prototype.opener;

/**
 * @type {!Window}
 * @see https://developer.mozilla.org/en/DOM/window.parent
 */
Window.prototype.parent;

/** @see https://developer.mozilla.org/en/DOM/window.personalbar */
Window.prototype.personalbar;

/** @see https://developer.mozilla.org/en/DOM/window.pkcs11 */
Window.prototype.pkcs11;

/** @see https://developer.mozilla.org/en/DOM/window */
Window.prototype.returnValue;

/** @see https://developer.mozilla.org/en/DOM/window.scrollbars */
Window.prototype.scrollbars;

/**
 * @type {number}
 * @see https://developer.mozilla.org/En/DOM/window.scrollMaxX
 */
Window.prototype.scrollMaxX;

/**
 * @type {number}
 * @see https://developer.mozilla.org/En/DOM/window.scrollMaxY
 */
Window.prototype.scrollMaxY;

/**
 * @type {!Window}
 * @see https://developer.mozilla.org/en/DOM/window.self
 */
Window.prototype.self;

/** @see https://developer.mozilla.org/en/DOM/Storage#sessionStorage */
Window.prototype.sessionStorage;

/** @see https://developer.mozilla.org/en/DOM/window.sidebar */
Window.prototype.sidebar;

/**
 * @type {?string}
 * @see https://developer.mozilla.org/en/DOM/window.status
 */
Window.prototype.status;

/** @see https://developer.mozilla.org/en/DOM/window.statusbar */
Window.prototype.statusbar;

/** @see https://developer.mozilla.org/en/DOM/window.toolbar */
Window.prototype.toolbar;

/**
 * @type {!Window}
 * @see https://developer.mozilla.org/en/DOM/window.self
 */
Window.prototype.top;

/**
 * @type {!Window}
 * @see https://developer.mozilla.org/en/DOM/window.self
 */
Window.prototype.window;

/**
 * @param {*} message
 * @see https://developer.mozilla.org/en/DOM/window.alert
 */
Window.prototype.alert = function(message) {};

/**
 * Decodes a string of data which has been encoded using base-64 encoding.
 *
 * @param {string} encodedData
 * @return {string}
 * @see https://developer.mozilla.org/en/DOM/window.atob
 * @nosideeffects
 */
function atob(encodedData) {}

/** @see https://developer.mozilla.org/en/DOM/window.back */
Window.prototype.back = function() {};

/** @see https://developer.mozilla.org/en/DOM/window.blur */
Window.prototype.blur = function() {};

/**
 * @param {string} stringToEncode
 * @return {string}
 * @see https://developer.mozilla.org/en/DOM/window.btoa
 * @nosideeffects
 */
function btoa(stringToEncode) {}

/** @deprecated */
Window.prototype.captureEvents;

/** @see https://developer.mozilla.org/en/DOM/window.close */
Window.prototype.close = function() {};

/** @see https://developer.mozilla.org/en/DOM/window.find */
Window.prototype.find;

/** @see https://developer.mozilla.org/en/DOM/window.focus */
Window.prototype.focus = function() {};

/** @see https://developer.mozilla.org/en/DOM/window.forward */
Window.prototype.forward = function() {};

/** @see https://developer.mozilla.org/en/DOM/window.getAttention */
Window.prototype.getAttention = function() {};

/**
 * @return {Selection}
 * @see https://developer.mozilla.org/en/DOM/window.getSelection
 * @nosideeffects
 */
Window.prototype.getSelection = function() {};

/** @see https://developer.mozilla.org/en/DOM/window.home */
Window.prototype.home = function() {};

Window.prototype.openDialog;
Window.prototype.releaseEvents;
Window.prototype.scrollByLines;
Window.prototype.scrollByPages;

/**
 * @param {string} uri
 * @param {?=} opt_arguments
 * @param {string=} opt_options
 * @see https://developer.mozilla.org/en/DOM/window.showModalDialog
 */
Window.prototype.showModalDialog;

Window.prototype.sizeToContent;

/**
 * @see http://msdn.microsoft.com/en-us/library/ms536769(VS.85).aspx
 */
Window.prototype.stop = function() {};

Window.prototype.updateCommands;

// properties of Document

/**
 * @see https://developer.mozilla.org/en/DOM/document.alinkColor
 * @type {string}
 */
Document.prototype.alinkColor;

/**
 * @see https://developer.mozilla.org/en/DOM/document.anchors
 * @type {HTMLCollection}
 */
Document.prototype.anchors;

/**
 * @see https://developer.mozilla.org/en/DOM/document.applets
 * @type {HTMLCollection}
 */
Document.prototype.applets;
/** @type {boolean} */ Document.prototype.async;
/** @type {string?} */ Document.prototype.baseURI;
Document.prototype.baseURIObject;

/**
 * @see https://developer.mozilla.org/en/DOM/document.bgColor
 * @type {string}
 */
Document.prototype.bgColor;

/** @type {HTMLBodyElement} */ Document.prototype.body;
Document.prototype.characterSet;

/**
 * @see https://developer.mozilla.org/en/DOM/document.compatMode
 * @type {string}
 */
Document.prototype.compatMode;

Document.prototype.contentType;
/** @type {string} */ Document.prototype.cookie;
Document.prototype.defaultView;

/**
 * @see https://developer.mozilla.org/en/DOM/document.designMode
 * @type {string}
 */
Document.prototype.designMode;

Document.prototype.documentURIObject;

/**
 * @see https://developer.mozilla.org/en/DOM/document.domain
 * @type {string}
 */
Document.prototype.domain;

/**
 * @see https://developer.mozilla.org/en/DOM/document.embeds
 * @type {HTMLCollection}
 */
Document.prototype.embeds;

/**
 * @see https://developer.mozilla.org/en/DOM/document.fgColor
 * @type {string}
 */
Document.prototype.fgColor;

/** @type {Element} */ Document.prototype.firstChild;

/**
 * @see https://developer.mozilla.org/en/DOM/document.forms
 * @type {HTMLCollection}
 */
Document.prototype.forms;

/** @type {number} */ Document.prototype.height;
/** @type {HTMLCollection} */ Document.prototype.images;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/DOM/document.lastModified
 */
Document.prototype.lastModified;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/DOM/document.linkColor
 */
Document.prototype.linkColor;

/**
 * @see https://developer.mozilla.org/en/DOM/document.links
 * @type {HTMLCollection}
 */
Document.prototype.links;

/**
 * @type {!Location}
 * @implicitCast
 */
Document.prototype.location;

Document.prototype.namespaceURI;
Document.prototype.nodePrincipal;
Document.prototype.plugins;
Document.prototype.popupNode;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/DOM/document.referrer
 */
Document.prototype.referrer;

/**
 * @type {StyleSheetList}
 * @see https://developer.mozilla.org/en/DOM/document.styleSheets
 */
Document.prototype.styleSheets;

/** @type {?string} */ Document.prototype.title;
Document.prototype.tooltipNode;
/** @type {string} */ Document.prototype.URL;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/DOM/document.vlinkColor
 */
Document.prototype.vlinkColor;

/** @type {number} */ Document.prototype.width;

// Methods of Document
/**
 * @see https://developer.mozilla.org/en/DOM/document.clear
 */
Document.prototype.clear = function() {};

/**
 * @see https://developer.mozilla.org/en/DOM/document.close
 */
Document.prototype.close;

/**
 * @see https://developer.mozilla.org/en-US/docs/Web/API/document.createElementNS
 * @see http://w3c.github.io/webcomponents/spec/custom/#extensions-to-document-interface-to-instantiate
 * @param {?string} namespaceURI
 * @param {string} qualifiedName
 * @param {string=} opt_typeExtension
 * @return {!Element}
 */
Document.prototype.createElementNS =
    function(namespaceURI, qualifiedName, opt_typeExtension) {};

/**
 * @param {string} type
 * @return {Event}
 */
Document.prototype.createEvent = function(type) {};
Document.prototype.createNSResolver;
/** @return {Range} */ Document.prototype.createRange = function() {};
Document.prototype.createTreeWalker;

Document.prototype.evaluate;

/**
 * @param {string} commandName
 * @param {?boolean=} opt_showUi
 * @param {*=} opt_value
 * @see https://developer.mozilla.org/en/Rich-Text_Editing_in_Mozilla#Executing_Commands
 */
Document.prototype.execCommand;

/**
 * @param {string} s id.
 * @return {HTMLElement}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/DOM/document.getElementById
 */
Document.prototype.getElementById = function(s) {};

/**
 * @param {string} name
 * @return {!NodeList}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/DOM/document.getElementsByClassName
 */
Document.prototype.getElementsByClassName = function(name) {};

/**
 * @param {string} name
 * @return {!NodeList}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/DOM/document.getElementsByName
 */
Document.prototype.getElementsByName = function(name) {};

/**
 * @param {string} namespace
 * @param {string} name
 * @return {!NodeList}
 * @nosideeffects
 * @see https://developer.mozilla.org/en/DOM/document.getElementsByTagNameNS
 */
Document.prototype.getElementsByTagNameNS = function(namespace, name) {};

/**
 * @param {Node} externalNode
 * @param {boolean} deep
 * @return {Node}
 */
Document.prototype.importNode = function(externalNode, deep) {};

/** @param {string} uri */
Document.prototype.load = function(uri) {};
Document.prototype.loadOverlay;

/**
 * @see https://developer.mozilla.org/en/DOM/document.open
 */
Document.prototype.open;

/**
 * @see https://developer.mozilla.org/en/Midas
 * @see http://msdn.microsoft.com/en-us/library/ms536676(VS.85).aspx
 */
Document.prototype.queryCommandEnabled;

/**
 * @see https://developer.mozilla.org/en/Midas
 * @see http://msdn.microsoft.com/en-us/library/ms536678(VS.85).aspx
 */
Document.prototype.queryCommandIndeterm;

/**
 * @see https://developer.mozilla.org/en/Midas
 * @see http://msdn.microsoft.com/en-us/library/ms536679(VS.85).aspx
 */
Document.prototype.queryCommandState;

/**
 * @see https://developer.mozilla.org/en/DOM/document.queryCommandSupported
 * @see http://msdn.microsoft.com/en-us/library/ms536681(VS.85).aspx
 * @param {string} command
 * @return {?} Implementation-specific.
 */
Document.prototype.queryCommandSupported;

/**
 * @see https://developer.mozilla.org/en/Midas
 * @see http://msdn.microsoft.com/en-us/library/ms536683(VS.85).aspx
 */
Document.prototype.queryCommandValue;

/**
 * @see https://developer.mozilla.org/en/DOM/document.write
 * @param {string} text
 */
Document.prototype.write = function(text) {};

/**
 * @see https://developer.mozilla.org/en/DOM/document.writeln
 * @param {string} text
 */
Document.prototype.writeln = function(text) {};

Document.prototype.ononline;
Document.prototype.onoffline;

// XUL
/**
 * @see http://developer.mozilla.org/en/DOM/document.getBoxObjectFor
 * @return {BoxObject}
 * @nosideeffects
 */
Document.prototype.getBoxObjectFor = function(element) {};

// From:
// http://lxr.mozilla.org/mozilla1.8/source/dom/public/idl/range/nsIDOMNSRange.idl

/**
 * @param {string} tag
 * @return {DocumentFragment}
 */
Range.prototype.createContextualFragment;

/**
 * @param {Node} parent
 * @param {number} offset
 * @return {boolean}
 * @nosideeffects
 */
Range.prototype.isPointInRange;

/**
 * @param {Node} parent
 * @param {number} offset
 * @return {number}
 * @nosideeffects
 */
Range.prototype.comparePoint;

/**
 * @param {Node} n
 * @return {boolean}
 * @nosideeffects
 */
Range.prototype.intersectsNode;

/**
 * @param {Node} n
 * @return {number}
 * @nosideeffects
 */
Range.prototype.compareNode;


/** @constructor */
function Selection() {}

/**
 * @type {Node}
 * @see https://developer.mozilla.org/en/DOM/Selection/anchorNode
 */
Selection.prototype.anchorNode;

/**
 * @type {number}
 * @see https://developer.mozilla.org/en/DOM/Selection/anchorOffset
 */
Selection.prototype.anchorOffset;

/**
 * @type {Node}
 * @see https://developer.mozilla.org/en/DOM/Selection/focusNode
 */
Selection.prototype.focusNode;

/**
 * @type {number}
 * @see https://developer.mozilla.org/en/DOM/Selection/focusOffset
 */
Selection.prototype.focusOffset;

/**
 * @type {boolean}
 * @see https://developer.mozilla.org/en/DOM/Selection/isCollapsed
 */
Selection.prototype.isCollapsed;

/**
 * @type {number}
 * @see https://developer.mozilla.org/en/DOM/Selection/rangeCount
 */
Selection.prototype.rangeCount;

/**
 * @param {Range} range
 * @return {undefined}
 * @see https://developer.mozilla.org/en/DOM/Selection/addRange
 */
Selection.prototype.addRange = function(range) {};

/**
 * @param {number} index
 * @return {Range}
 * @see https://developer.mozilla.org/en/DOM/Selection/getRangeAt
 * @nosideeffects
 */
Selection.prototype.getRangeAt = function(index) {};

/**
 * @param {Node} node
 * @param {number} index
 * @return {undefined}
 * @see https://developer.mozilla.org/en/DOM/Selection/collapse
 */
Selection.prototype.collapse = function(node, index) {};

/**
 * @return {undefined}
 * @see https://developer.mozilla.org/en/DOM/Selection/collapseToEnd
 */
Selection.prototype.collapseToEnd = function() {};

/**
 * @return {undefined}
 * @see https://developer.mozilla.org/en/DOM/Selection/collapseToStart
 */
Selection.prototype.collapseToStart = function() {};

/**
 * @param {Node} node
 * @param {boolean} partlyContained
 * @return {boolean}
 * @see https://developer.mozilla.org/en/DOM/Selection/containsNode
 * @nosideeffects
 */
Selection.prototype.containsNode = function(node, partlyContained) {};

/**
 * @see https://developer.mozilla.org/en/DOM/Selection/deleteFromDocument
 */
Selection.prototype.deleteFromDocument = function() {};

/**
 * @param {Node} parentNode
 * @param {number} offset
 * @see https://developer.mozilla.org/en/DOM/Selection/extend
 */
Selection.prototype.extend = function(parentNode, offset) {};

/**
 * @see https://developer.mozilla.org/en/DOM/Selection/removeAllRanges
 */
Selection.prototype.removeAllRanges = function() {};

/**
 * @param {Range} range
 * @see https://developer.mozilla.org/en/DOM/Selection/removeRange
 */
Selection.prototype.removeRange = function(range) {};

/**
 * @param {Node} parentNode
 * @see https://developer.mozilla.org/en/DOM/Selection/selectAllChildren
 */
Selection.prototype.selectAllChildren;

/**
 * @see https://developer.mozilla.org/en/DOM/Selection/selectionLanguageChange
 */
Selection.prototype.selectionLanguageChange;

/** @type {NamedNodeMap} */ Element.prototype.attributes;
Element.prototype.baseURIObject;
/** @type {!NodeList} */ Element.prototype.childNodes;

/**
 * @type {!NodeList}
 * @see https://developer.mozilla.org/en/DOM/element.children
 */
Element.prototype.children;

/**
 * @type {string}
 * @implicitCast
 */
Element.prototype.className;
/** @type {string} */ Element.prototype.dir;

/**
 * Firebug sets this property on elements it is inserting into the DOM.
 * @type {boolean}
 */
Element.prototype.firebugIgnore;

/** @type {Node} */ Element.prototype.firstChild;
/**
 * @type {string}
 * @implicitCast
 */
Element.prototype.id;
/**
 * @type {string}
 * @implicitCast
 */
Element.prototype.innerHTML;
/** @type {string} */ Element.prototype.lang;
/** @type {Node} */ Element.prototype.lastChild;
Element.prototype.localName;
Element.prototype.name;
Element.prototype.namespaceURI;
/** @type {Node} */ Element.prototype.nextSibling;
Element.prototype.nodeName;
Element.prototype.nodePrincipal;
/** @type {number} */ Element.prototype.nodeType;
Element.prototype.nodeValue;
/** @type {Document} */ Element.prototype.ownerDocument;
/** @type {Node} */ Element.prototype.parentNode;
Element.prototype.prefix;
/** @type {Node} */ Element.prototype.previousSibling;
/** @type {!CSSStyleDeclaration} */ Element.prototype.style;
/**
 * @type {number}
 * @implicitCast
 */
Element.prototype.tabIndex;

/**
 * @type {string}
 * @implicitCast
 */
Element.prototype.textContent;
/** @type {string} */ Element.prototype.title;

/**
 * @param {Node} child
 * @return {Node} appendedElement.
 * @override
 */
Element.prototype.appendChild = function(child) {};

/**
 * @override
 * @return {!Element}
 */
Element.prototype.cloneNode = function(deep) {};

/** @override */
Element.prototype.dispatchEvent = function(event) {};

/** @return {undefined} */
Element.prototype.blur = function() {};

/** @return {undefined} */
Element.prototype.click = function() {};

/** @return {undefined} */
Element.prototype.focus = function() {};

/**
 * @return {boolean}
 * @override
 * @nosideeffects
 */
Element.prototype.hasAttributes = function() {};

/**
 * @return {boolean}
 * @override
 * @nosideeffects
 */
Element.prototype.hasChildNodes = function() {};

/** @override */
Element.prototype.insertBefore = function(insertedNode, adjacentNode) {};

/**
 * @return {undefined}
 * @override
 */
Element.prototype.normalize = function() {};

/**
 * @param {Node} removedNode
 * @return {!Node}
 * @override
 */
Element.prototype.removeChild = function(removedNode) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
Element.prototype.removeEventListener = function(type, handler, opt_useCapture)
    {};

/** @override */
Element.prototype.replaceChild = function(insertedNode, replacedNode) {};

/** @type {number} */
HTMLInputElement.prototype.selectionStart;

/** @type {number} */
HTMLInputElement.prototype.selectionEnd;

/**
 * @param {number} selectionStart
 * @param {number} selectionEnd
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/editing.html#dom-textarea/input-setselectionrange
 */
HTMLInputElement.prototype.setSelectionRange =
    function(selectionStart, selectionEnd) {};

/** @type {number} */
HTMLTextAreaElement.prototype.selectionStart;

/** @type {number} */
HTMLTextAreaElement.prototype.selectionEnd;

/**
 * @param {number} selectionStart
 * @param {number} selectionEnd
 * @see http://www.whatwg.org/specs/web-apps/current-work/multipage/editing.html#dom-textarea/input-setselectionrange
 */
HTMLTextAreaElement.prototype.setSelectionRange =
    function(selectionStart, selectionEnd) {};

/** @constructor */
function Navigator() {}

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.appCodeName
 */
Navigator.prototype.appCodeName;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.appVersion
 */
Navigator.prototype.appName;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.appVersion
 */
Navigator.prototype.appVersion;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.buildID
 */
Navigator.prototype.buildID;

/**
 * @type {boolean}
 * @see https://developer.mozilla.org/en/Navigator.cookieEnabled
 */
Navigator.prototype.cookieEnabled;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.language
 */
Navigator.prototype.language;

/**
 * @type {MimeTypeArray}
 * @see https://developer.mozilla.org/en/Navigator.mimeTypes
 */
Navigator.prototype.mimeTypes;

/**
 * @type {boolean}
 * @see https://developer.mozilla.org/en/Navigator.onLine
 */
Navigator.prototype.onLine;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.oscpu
 */
Navigator.prototype.oscpu;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.platform
 */
Navigator.prototype.platform;

/**
 * @type {PluginArray}
 * @see https://developer.mozilla.org/en/Navigator.plugins
 */
Navigator.prototype.plugins;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.product
 */
Navigator.prototype.product;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.productSub
 */
Navigator.prototype.productSub;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.securityPolicy
 */
Navigator.prototype.securityPolicy;

/**
 * @param {string} url
 * @param {ArrayBufferView|Blob|string|FormData=} opt_data
 * @return {boolean}
 * @see https://developer.mozilla.org/en-US/docs/Web/API/navigator.sendBeacon
 */
Navigator.prototype.sendBeacon = function(url, opt_data) {};

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.userAgent
 */
Navigator.prototype.userAgent;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.vendor
 */
Navigator.prototype.vendor;

/**
 * @type {string}
 * @see https://developer.mozilla.org/en/Navigator.vendorSub
 */
Navigator.prototype.vendorSub;

/**
 * @type {function(): boolean}
 * @see https://developer.mozilla.org/en/Navigator.javaEnabled
 * @nosideeffects
 */
Navigator.prototype.javaEnabled = function() {};

/**
 * @constructor
 * @see https://developer.mozilla.org/en/DOM/PluginArray
 */
function PluginArray() {}

/** @type {number} */
PluginArray.prototype.length;

/**
 * @param {number} index
 * @return {Plugin}
 */
PluginArray.prototype.item = function(index) {};

/**
 * @param {string} name
 * @return {Plugin}
 */
PluginArray.prototype.namedItem = function(name) {};

/** @param {boolean=} reloadDocuments */
PluginArray.prototype.refresh = function(reloadDocuments) {};

/** @constructor */
function MimeTypeArray() {}

/**
 * @param {number} index
 * @return {MimeType}
 */
MimeTypeArray.prototype.item = function(index) {};

/**
 * @type {number}
 * @see https://developer.mozilla.org/en/DOM/window.navigator.mimeTypes
 */
MimeTypeArray.prototype.length;

/**
 * @param {string} name
 * @return {MimeType}
 */
MimeTypeArray.prototype.namedItem = function(name) {};

/** @constructor */
function MimeType() {}

/** @type {string} */
MimeType.prototype.description;

/** @type {Plugin} */
MimeType.prototype.enabledPlugin;

/** @type {string} */
MimeType.prototype.suffixes;

/** @type {string} */
MimeType.prototype.type;

/** @constructor */
function Plugin() {}

/** @type {string} */
Plugin.prototype.description;

/** @type {string} */
Plugin.prototype.filename;

/** @type {number} */
Plugin.prototype.length;

/** @type {string} */
Plugin.prototype.name;

/** @constructor */
function BoxObject() {}

/** @type {Element} */
BoxObject.prototype.element;

/** @type {number} */
BoxObject.prototype.screenX;

/** @type {number} */
BoxObject.prototype.screenY;

/** @type {number} */
BoxObject.prototype.x;

/** @type {number} */
BoxObject.prototype.y;

/** @type {number} */
BoxObject.prototype.width;


/**
 * @type {number}
 * @see http://www.google.com/codesearch/p?hl=en#eksvcKKj5Ng/mozilla/dom/public/idl/html/nsIDOMNSHTMLImageElement.idl&q=naturalWidth
 */
HTMLImageElement.prototype.naturalWidth;

/**
 * @type {number}
 * @see http://www.google.com/codesearch/p?hl=en#eksvcKKj5Ng/mozilla/dom/public/idl/html/nsIDOMNSHTMLImageElement.idl&q=naturalHeight
 */
HTMLImageElement.prototype.naturalHeight;


/**
 * @param {Element} element
 * @param {?string=} pseudoElt
 * @return {CSSStyleDeclaration}
 * @nosideeffects
 */
function getComputedStyle(element, pseudoElt) {}

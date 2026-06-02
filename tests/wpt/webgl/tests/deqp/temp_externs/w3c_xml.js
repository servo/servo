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
 * @fileoverview Definitions for W3C's XML related specifications.
 *  This file depends on w3c_dom2.js.
 *  The whole file has been fully type annotated.
 *
 *  Provides the XML standards from W3C.
 *   Includes:
 *    XPath          - Fully type annotated
 *    XMLHttpRequest - Fully type annotated
 *
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html
 * @see http://www.w3.org/TR/XMLHttpRequest/
 * @see http://www.w3.org/TR/XMLHttpRequest2/
 *
 * @externs
 * @author stevey@google.com (Steve Yegge)
 */


/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathException
 */
function XPathException() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#INVALID_EXPRESSION_ERR
 */
XPathException.INVALID_EXPRESSION_ERR = 52;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#TYPE_ERR
 */
XPathException.TYPE_ERR = 52;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#
 */
XPathException.prototype.code;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathEvaluator
 */
function XPathEvaluator() {}

/**
 * @param {string} expr
 * @param {?XPathNSResolver=} opt_resolver
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathEvaluator-createExpression
 * @throws XPathException
 * @throws DOMException
 */
XPathEvaluator.prototype.createExpression = function(expr, opt_resolver) {};

/**
 * @param {Node} nodeResolver
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathEvaluator-createNSResolver
 */
XPathEvaluator.prototype.createNSResolver = function(nodeResolver) {};

/**
 * @param {string} expr
 * @param {Node} contextNode
 * @param {?XPathNSResolver=} opt_resolver
 * @param {?number=} opt_type
 * @param {*=} opt_result
 * @return {XPathResult}
 * @throws XPathException
 * @throws DOMException
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathEvaluator-evaluate
 */
XPathEvaluator.prototype.evaluate = function(expr, contextNode, opt_resolver,
    opt_type, opt_result) {};


/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathExpression
 */
function XPathExpression() {}

/**
 * @param {Node} contextNode
 * @param {number=} opt_type
 * @param {*=} opt_result
 * @return {*}
 * @throws XPathException
 * @throws DOMException
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathExpression-evaluate
 */
XPathExpression.prototype.evaluate = function(contextNode, opt_type,
    opt_result) {};


/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathNSResolver
 */
function XPathNSResolver() {}

/**
 * @param {string} prefix
 * @return {?string}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathNSResolver-lookupNamespaceURI
 */
XPathNSResolver.prototype.lookupNamespaceURI = function(prefix) {};

/**
 * From http://www.w3.org/TR/xpath
 *
 * XPath is a language for addressing parts of an XML document, designed to be
 * used by both XSLT and XPointer.
 *
 * @noalias
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult
 */
function XPathResult() {}

/**
 * @type {boolean} {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-booleanValue
 */
XPathResult.prototype.booleanValue;

/**
 * @type {boolean} {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-invalid-iterator-state
 */
XPathResult.prototype.invalidInteratorState;

/**
 * @type {number}
 * @throws XPathException {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-numberValue
 */
XPathResult.prototype.numberValue;

/**
 * @type {number}
 * @throws XPathException {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-resultType
 */
XPathResult.prototype.resultType;

/**
 * @type {Node}
 * @throws XPathException {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-singleNodeValue
 */
XPathResult.prototype.singleNodeValue;

/**
 * @type {number}
 * @throws XPathException {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-snapshot-length
 */
XPathResult.prototype.snapshotLength;

/**
 * @type {string}
 * @throws XPathException {@see XPathException.TYPE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-stringValue
 */
XPathResult.prototype.stringValue;

/**
 * @return {Node}
 * @throws XPathException {@see XPathException.TYPE_ERR}
 * @throws DOMException {@see DOMException.INVALID_STATE_ERR}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-iterateNext
 */
XPathResult.prototype.iterateNext = function() {};

/**
 * @param {number} index
 * @return {Node}
 * @throws XPathException
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-snapshotItem
 */
XPathResult.prototype.snapshotItem = function(index) {};

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-ANY-TYPE
 */
XPathResult.ANY_TYPE = 0;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-NUMBER-TYPE
 */
XPathResult.NUMBER_TYPE = 1;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-STRING-TYPE
 */
XPathResult.STRING_TYPE = 2;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-BOOLEAN-TYPE
 */
XPathResult.BOOLEAN_TYPE = 3;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-UNORDERED-NODE-ITERATOR-TYPE
 */
XPathResult.UNORDERED_NODE_ITERATOR_TYPE = 4;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-ORDERED-NODE-ITERATOR-TYPE
 */
XPathResult.ORDERED_NODE_ITERATOR_TYPE = 5;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-UNORDERED-NODE-SNAPSHOT-TYPE
 */
XPathResult.UNORDERED_NODE_SNAPSHOT_TYPE = 6;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-ORDERED-NODE-SNAPSHOT-TYPE
 */
XPathResult.ORDERED_NODE_SNAPSHOT_TYPE = 7;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-ANY-UNORDERED-NODE-TYPE
 */
XPathResult.ANY_UNORDERED_NODE_TYPE = 8;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathResult-FIRST-ORDERED-NODE-TYPE
 */
XPathResult.FIRST_ORDERED_NODE_TYPE = 9;

/**
 * @constructor
 * @extends {Node}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathNamespace
 */
function XPathNamespace() {}

/**
 * @type {Element}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPathNamespace-ownerElement
 */
XPathNamespace.prototype.ownerElement;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-XPath/xpath.html#XPATH_NAMESPACE_NODE
 */
XPathNamespace.XPATH_NAMESPACE_NODE = 13;

/**
 * From http://www.w3.org/TR/XMLHttpRequest/
 *
 * (Draft)
 *
 * The XMLHttpRequest Object specification defines an API that provides
 * scripted client functionality for transferring data between a client and a
 * server.
 *
 * @constructor
 * @implements {EventTarget}
 * @see http://www.w3.org/TR/XMLHttpRequest/#xmlhttprequest-object
 */
function XMLHttpRequest() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
XMLHttpRequest.prototype.addEventListener =
    function(type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
XMLHttpRequest.prototype.removeEventListener =
    function(type, listener, opt_useCapture) {};

/** @override */
XMLHttpRequest.prototype.dispatchEvent = function(evt) {};

/**
 * @param {string} method
 * @param {string} url
 * @param {?boolean=} opt_async
 * @param {?string=} opt_user
 * @param {?string=} opt_password
 * @return {undefined}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-open()-method
 */
XMLHttpRequest.prototype.open = function(method, url, opt_async, opt_user,
    opt_password) {};

/**
 * @param {string} header
 * @param {string} value
 * @return {undefined}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-setrequestheader()-method
 */
XMLHttpRequest.prototype.setRequestHeader = function(header, value) {};

/**
 * @param {ArrayBuffer|ArrayBufferView|Blob|Document|FormData|string=} opt_data
 * @return {undefined}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-send()-method
 */
XMLHttpRequest.prototype.send = function(opt_data) {};

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-abort()-method
 */
XMLHttpRequest.prototype.abort = function() {};

/**
 * @return {string}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-getallresponseheaders()-method
 */
XMLHttpRequest.prototype.getAllResponseHeaders = function() {};

/**
 * @param {string} header
 * @return {string}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-getresponseheader()-method
 */
XMLHttpRequest.prototype.getResponseHeader = function(header) {};

/**
 * @type {string}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-responsetext-attribute
 */
XMLHttpRequest.prototype.responseText;

/**
 * @type {Document}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-responsexml-attribute
 */
XMLHttpRequest.prototype.responseXML;

/**
 * @type {number}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-readystate-attribute
 */
XMLHttpRequest.prototype.readyState;

/**
 * @type {number}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-status-attribute
 */
XMLHttpRequest.prototype.status;

/**
 * @type {string}
 * @see http://www.w3.org/TR/XMLHttpRequest/#the-statustext-attribute
 */
XMLHttpRequest.prototype.statusText;

/**
 * @type {Function}
 * @see http://www.w3.org/TR/XMLHttpRequest/#handler-xhr-onreadystatechange
 */
XMLHttpRequest.prototype.onreadystatechange;

/**
 * @type {Function}
 * @see http://www.w3.org/TR/XMLHttpRequest/#handler-xhr-onerror
 */
XMLHttpRequest.prototype.onerror;

/**
 * The FormData object represents an ordered collection of entries. Each entry
 * has a name and value.
 *
 * @param {?Element=} opt_form An optional form to use for constructing the form
 *     data set.
 * @constructor
 * @see http://www.w3.org/TR/XMLHttpRequest2/#the-formdata-interface
 */
function FormData(opt_form) {}

/**
 * @param {string} name
 * @param {Blob|string} value
 */
FormData.prototype.append = function(name, value) {};

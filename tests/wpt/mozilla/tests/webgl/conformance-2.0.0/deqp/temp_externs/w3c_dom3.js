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
 * @fileoverview Definitions for W3C's DOM Level 3 specification.
 *  This file depends on w3c_dom2.js.
 *  The whole file has been fully type annotated.
 *  Created from
 *   http://www.w3.org/TR/DOM-Level-3-Core/ecma-script-binding.html
 *
 * @externs
 * @author stevey@google.com (Steve Yegge)
 */

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-258A00AF
 */
DOMException.prototype.code;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-258A00AF
 */
DOMException.VALIDATION_ERR = 16;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-258A00AF
 */
DOMException.TYPE_MISMATCH_ERR = 17;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMStringList
 */
function DOMStringList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMStringList-length
 */
DOMStringList.prototype.length;

/**
 * @param {string} str
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMStringList-contains
 */
DOMStringList.prototype.contains = function(str) {};

/**
 * @param {number} index
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMStringList-item
 */
DOMStringList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#NameList
 */
function NameList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#NameList-length
 */
NameList.prototype.length;

/**
 * @param {string} str
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#NameList-contains
 * @nosideeffects
 */
NameList.prototype.contains = function(str) {};

/**
 * @param {string} namespaceURI
 * @param {string} name
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#NameList-containsNS
 * @nosideeffects
 */
NameList.prototype.containsNS = function(namespaceURI, name) {};

/**
 * @param {number} index
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#NameList-getName
 * @nosideeffects
 */
NameList.prototype.getName = function(index) {};

/**
 * @param {number} index
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#NameList-getNamespaceURI
 * @nosideeffects
 */
NameList.prototype.getNamespaceURI = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMImplementationList
 */
function DOMImplementationList() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMImplementationList-length
 */
DOMImplementationList.prototype.length;

/**
 * @param {number} index
 * @return {DOMImplementation}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMImplementationList-item
 * @nosideeffects
 */
DOMImplementationList.prototype.item = function(index) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMImplementationSource
 */
function DOMImplementationSource() {}

/**
 * @param {string} namespaceURI
 * @param {string} publicId
 * @param {DocumentType} doctype
 * @return {Document}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Level-2-Core-DOM-createDocument
 * @nosideeffects
 */
DOMImplementation.prototype.createDocument = function(namespaceURI, publicId, doctype) {};

/**
 * @param {string} qualifiedName
 * @param {string} publicId
 * @param {string} systemId
 * @return {DocumentType}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Level-2-Core-DOM-createDocType
 * @nosideeffects
 */
DOMImplementation.prototype.createDocumentType = function(qualifiedName, publicId, systemId) {};

/**
 * @param {string} features
 * @return {DOMImplementation}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-getDOMImpl
 * @nosideeffects
 */
DOMImplementationSource.prototype.getDOMImplementation = function(features) {};

/**
 * @param {string} features
 * @return {DOMImplementationList}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-getDOMImpls
 * @nosideeffects
 */
DOMImplementationSource.prototype.getDOMImplementationList = function(features) {};

/**
 * @param {string} feature
 * @param {string} version
 * @return {Object}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMImplementation3-getFeature
 * @nosideeffects
 */
DOMImplementation.prototype.getFeature = function(feature, version) {};

/**
 * @param {Node} externalNode
 * @return {Node}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-adoptNode
 */
Document.prototype.adoptNode = function(externalNode) {};

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-documentURI
 */
Document.prototype.documentURI;

/**
 * @type {DOMConfiguration}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-domConfig
 */
Document.prototype.domConfig;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-inputEncoding
 */
Document.prototype.inputEncoding;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-strictErrorChecking
 */
Document.prototype.strictErrorChecking;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-encoding
 */
Document.prototype.xmlEncoding;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-standalone
 */
Document.prototype.xmlStandalone;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-version
 */
Document.prototype.xmlVersion;

/**
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-normalizeDocument
 */
Document.prototype.normalizeDocument = function() {};

/**
 * @param {Node} n
 * @param {string} namespaceURI
 * @param {string} qualifiedName
 * @return {Node}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Document3-renameNode
 */
Document.prototype.renameNode = function(n, namespaceURI, qualifiedName) {};

/**
 * @type {?string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-baseURI
 */
Node.prototype.baseURI;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-NodeNSLocalN
 */
Node.prototype.localName;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-NodeNSname
 */
Node.prototype.namespaceURI;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-NodeNSPrefix
 */
Node.prototype.prefix;

/**
 * @type {string}
 * @implicitCast
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-textContent
 */
Node.prototype.textContent;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node-DOCUMENT_POSITION_DISCONNECTED
 */
Node.DOCUMENT_POSITION_DISCONNECTED = 0x01;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node-DOCUMENT_POSITION_PRECEDING
 */
Node.DOCUMENT_POSITION_PRECEDING    = 0x02;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node-DOCUMENT_POSITION_FOLLOWING
 */
Node.DOCUMENT_POSITION_FOLLOWING    = 0x04;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node-DOCUMENT_POSITION_CONTAINS
 */
Node.DOCUMENT_POSITION_CONTAINS     = 0x08;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node-DOCUMENT_POSITION_CONTAINED_BY
 */
Node.DOCUMENT_POSITION_CONTAINED_BY = 0x10;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node-DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
 */
Node.DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC = 0x20;

/**
 * @param {Node} other
 * @return {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-compareDocumentPosition
 * @nosideeffects
 */
Node.prototype.compareDocumentPosition = function(other) {};

/**
 * @param {string} feature
 * @param {string} version
 * @return {Object}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-getFeature
 * @nosideeffects
 */
Node.prototype.getFeature = function(feature, version) {};

/**
 * @param {string} key
 * @return {Object}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-getUserData
 * @nosideeffects
 */
Node.prototype.getUserData = function(key) {};

/**
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-NodeHasAttrs
 * @nosideeffects
 */
Node.prototype.hasAttributes = function() {};

/**
 * @param {string} namespaceURI
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-isDefaultNamespace
 * @nosideeffects
 */
Node.prototype.isDefaultNamespace = function(namespaceURI) {};

/**
 * @param {Node} arg
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-isEqualNode
 * @nosideeffects
 */
Node.prototype.isEqualNode = function(arg) {};

/**
 * @param {Node} other
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-isSameNode
 * @nosideeffects
 */
Node.prototype.isSameNode = function(other) {};

/**
 * @param {string} feature
 * @param {string} version
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Level-2-Core-Node-supports
 * @nosideeffects
 */
Node.prototype.isSupported = function(feature, version) {};

/**
 * @param {string} prefix
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-lookupNamespaceURI
 * @nosideeffects
 */
Node.prototype.lookupNamespaceURI = function(prefix) {};

/**
 * @param {string} namespaceURI
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-lookupNamespacePrefix
 * @nosideeffects
 */
Node.prototype.lookupPrefix = function(namespaceURI) {};

/**
 * @return undefined
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-normalize
 */
Node.prototype.normalize = function() {};

/**
 * @param {Object} key
 * @param {Object} data
 * @param {UserDataHandler} handler
 * @return {Object}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Node3-setUserData'
 */
Node.prototype.setUserData = function(key, data, handler) {};

/**
 * @param {string} query
 * @return {Node}
 * @see http://www.w3.org/TR/selectors-api/#queryselector
 * @nosideeffects
 */
Node.prototype.querySelector = function(query) {};

/**
 * @param {string} query
 * @return {!NodeList}
 * @see http://www.w3.org/TR/selectors-api/#queryselectorall
 * @nosideeffects
 */
Node.prototype.querySelectorAll = function(query) {};

/**
 * @type {Element}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Attr-ownerElement
 */
Attr.prototype.ownerElement;

/**
 * @type {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Attr-isId
 */
Attr.prototype.isId;

/**
 * @type {TypeInfo}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Attr-schemaTypeInfo
 */
Attr.prototype.schemaTypeInfo;

/**
 * @type {TypeInfo}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Element-schemaTypeInfo
 */
Element.prototype.schemaTypeInfo;

/**
 * @param {string} namespaceURI
 * @param {string} localName
 * @return {Attr}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElGetAtNodeNS
 * @nosideeffects
 */
Element.prototype.getAttributeNodeNS = function(namespaceURI, localName) {};

/**
 * @param {string} namespaceURI
 * @param {string} localName
 * @return {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElGetAttrNS
 * @nosideeffects
 */
Element.prototype.getAttributeNS = function(namespaceURI, localName) {};

/**
 * @param {string} namespaceURI
 * @param {string} localName
 * @return {!NodeList}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-A6C90942
 * @nosideeffects
 */
Element.prototype.getElementsByTagNameNS = function(namespaceURI, localName) {};

/**
 * @param {string} name
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElHasAttr
 * @nosideeffects
 */
Element.prototype.hasAttribute = function(name) {};

/**
 * @param {string} namespaceURI
 * @param {string} localName
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElHasAttrNS
 * @nosideeffects
 */
Element.prototype.hasAttributeNS = function(namespaceURI, localName) {};

/**
 * @param {string} namespaceURI
 * @param {string} localName
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElRemAtNS
 */
Element.prototype.removeAttributeNS = function(namespaceURI, localName) {};

/**
 * @param {Attr} newAttr
 * @return {Attr}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElSetAtNodeNS
 */
Element.prototype.setAttributeNodeNS = function(newAttr) {};

/**
 * @param {string} namespaceURI
 * @param {string} qualifiedName
 * @param {string|number|boolean} value Values are converted to strings with
 *     ToString, so we accept number and boolean since both convert easily to
 *     strings.
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElSetAttrNS
 */
Element.prototype.setAttributeNS = function(namespaceURI, qualifiedName, value) {};

/**
 * @param {string} name
 * @param {boolean} isId
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElSetIdAttr
 */
Element.prototype.setIdAttribute = function(name, isId) {};

/**
 * @param {Attr} idAttr
 * @param {boolean} isId
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElSetIdAttrNode
 */
Element.prototype.setIdAttributeNode = function(idAttr, isId) {};

/**
 * @param {string} namespaceURI
 * @param {string} localName
 * @param {boolean} isId
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ElSetIdAttrNS
 */
Element.prototype.setIdAttributeNS = function(namespaceURI, localName, isId) {};

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Text3-wholeText
 */
Text.prototype.wholeText;

/**
 * @param {string} newText
 * @return {Text}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Text3-replaceWholeText
 */
Text.prototype.replaceWholeText = function(newText) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo
 */
function TypeInfo() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-DERIVATION_EXTENSION
 */
TypeInfo.prototype.DERIVATION_EXTENSION;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-DERIVATION_LIST
 */
TypeInfo.prototype.DERIVATION_LIST;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-DERIVATION_RESTRICTION
 */
TypeInfo.prototype.DERIVATION_RESTRICTION;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-DERIVATION_UNION
 */
TypeInfo.prototype.DERIVATION_UNION;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-typeName
 */
TypeInfo.prototype.typeName;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-typeNamespace
 */
TypeInfo.prototype.typeNamespace;

/**
 * @param {string} typeNamespaceArg
 * @param {string} typeNameArg
 * @param {number} derivationMethod
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#TypeInfo-isDerivedFrom
 * @nosideeffects
 */
TypeInfo.prototype.isDerivedFrom = function(typeNamespaceArg, typeNameArg, derivationMethod) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#UserDataHandler
 */
function UserDataHandler() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#UserDataHandler-CLONED
 */
UserDataHandler.prototype.NODE_CLONED = 1;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#UserDataHandler-IMPORTED
 */
UserDataHandler.prototype.NODE_IMPORTED = 2;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#UserDataHandler-DELETED
 */
UserDataHandler.prototype.NODE_DELETED = 3;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#UserDataHandler-RENAMED
 */
UserDataHandler.prototype.NODE_RENAMED = 4;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#UserDataHandler-ADOPTED
 */
UserDataHandler.prototype.NODE_ADOPTED = 5;

/**
 * @param {number} operation
 * @param {string} key
 * @param {*=} opt_data
 * @param {?Node=} opt_src
 * @param {?Node=} opt_dst
 * @return {undefined}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-handleUserDataEvent
 */
UserDataHandler.prototype.handle = function(operation, key, opt_data,
  opt_src, opt_dst) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-Interfaces-DOMError
 */
function DOMError() {}

/**
 * @type {DOMLocator}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-location
 */
DOMError.prototype.location;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-message
 */
DOMError.prototype.message;

/**
 * @type {Object}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-relatedData
 */
DOMError.prototype.relatedData;

/**
 * @type {Object}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-relatedException
 */
DOMError.prototype.relatedException;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-severity-warning
 */
DOMError.SEVERITY_WARNING = 1;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-severity-error
 */
DOMError.SEVERITY_ERROR = 2;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-severity-fatal-error
 */
DOMError.SEVERITY_FATAL_ERROR = 3;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-severity
 */
DOMError.prototype.severity;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-DOMError-type
 */
DOMError.prototype.type;

/**
 * @type {string}
 * @see http://www.w3.org/TR/dom/#domerror
 */
DOMError.prototype.name;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ERROR-Interfaces-DOMErrorHandler
 */
function DOMErrorHandler() {}

/**
 * @param {DOMError} error
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-ERRORS-DOMErrorHandler-handleError
 */
DOMErrorHandler.prototype.handleError = function(error) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Interfaces-DOMLocator
 */
function DOMLocator() {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMLocator-byteOffset
 */
DOMLocator.prototype.byteOffset;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMLocator-column-number
 */
DOMLocator.prototype.columnNumber;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMLocator-line-number
 */
DOMLocator.prototype.lineNumber;

/**
 * @type {Node}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMLocator-node
 */
DOMLocator.prototype.relatedNode;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMLocator-uri
 */
DOMLocator.prototype.uri;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMLocator-utf16Offset
 */
DOMLocator.prototype.utf16Offset;

/**
 * @constructor
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMConfiguration
 */
function DOMConfiguration() {}

/**
 * @type {DOMStringList}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMConfiguration-parameterNames
 */
DOMConfiguration.prototype.parameterNames;

/**
 * @param {string} name
 * @return {boolean}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMConfiguration-canSetParameter
 * @nosideeffects
 */
DOMConfiguration.prototype.canSetParameter = function(name) {};

/**
 * @param {string} name
 * @return {*}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMConfiguration-getParameter
 * @nosideeffects
 */
DOMConfiguration.prototype.getParameter = function(name) {};

/**
 * @param {string} name
 * @param {*} value
 * @return {*}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#DOMConfiguration-property
 */
DOMConfiguration.prototype.setParameter = function(name, value) {};

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-Core-DocType-internalSubset
 */
DocumentType.prototype.internalSubset;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-Core-DocType-publicId
 */
DocumentType.prototype.publicId;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#ID-Core-DocType-systemId
 */
DocumentType.prototype.systemId;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Entity3-inputEncoding
 */
Entity.prototype.inputEncoding;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Entity3-encoding
 */
Entity.prototype.xmlEncoding;

/**
 * @type {string}
 * @see http://www.w3.org/TR/DOM-Level-3-Core/core.html#Entity3-version
 */
Entity.prototype.xmlVersion;

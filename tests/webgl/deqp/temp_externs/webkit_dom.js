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
 * @fileoverview Definitions for all the extensions over W3C's DOM
 *  specification by WebKit. This file depends on w3c_dom2.js.
 *  All the provided definitions has been type annotated
 *
 * @externs
 */


/**
 * @param {boolean=} opt_center
 * @see https://bugzilla.mozilla.org/show_bug.cgi?id=403510
 */
Element.prototype.scrollIntoViewIfNeeded = function(opt_center) {};

/**
 * @constructor
 * @see http://trac.webkit.org/browser/trunk/Source/WebCore/page/MemoryInfo.idl
 * @see http://trac.webkit.org/browser/trunk/Source/WebCore/page/MemoryInfo.cpp
 */
function MemoryInfo() {};

/** @type {number} */
MemoryInfo.prototype.totalJSHeapSize;

/** @type {number} */
MemoryInfo.prototype.usedJSHeapSize;

/** @type {number} */
MemoryInfo.prototype.jsHeapSizeLimit;

/**
 * @constructor
 * @see http://trac.webkit.org/browser/trunk/Source/WebCore/inspector/ScriptProfileNode.idl
 */
function ScriptProfileNode() {};

/** @type {string} */
ScriptProfileNode.prototype.functionName;

/** @type {string} */
ScriptProfileNode.prototype.url;

/** @type {number} */
ScriptProfileNode.prototype.lineNumber;

/** @type {number} */
ScriptProfileNode.prototype.totalTime;

/** @type {number} */
ScriptProfileNode.prototype.selfTime;

/** @type {number} */
ScriptProfileNode.prototype.numberOfCalls;

/** @type {Array.<ScriptProfileNode>} */
ScriptProfileNode.prototype.children;

/** @type {boolean} */
ScriptProfileNode.prototype.visible;

/** @type {number} */
ScriptProfileNode.prototype.callUID;

/**
 * @constructor
 * @see http://trac.webkit.org/browser/trunk/Source/WebCore/inspector/ScriptProfile.idl
 */
function ScriptProfile() {};

/** @type {string} */
ScriptProfile.prototype.title;

/** @type {number} */
ScriptProfile.prototype.uid;

/** @type {ScriptProfileNode} */
ScriptProfile.prototype.head;

/**
 * @constructor
 * @see http://trac.webkit.org/browser/trunk/Source/WebCore/page/Console.idl
 * @see http://trac.webkit.org/browser/trunk/Source/WebCore/page/Console.cpp
 */
function Console() {};

/**
 * @param {*} condition
 * @param {...*} var_args
 */
Console.prototype.assert = function(condition, var_args) {};

/**
 * @param {...*} var_args
 */
Console.prototype.error = function(var_args) {};

/**
 * @param {...*} var_args
 */
Console.prototype.info = function(var_args) {};

/**
 * @param {...*} var_args
 */
Console.prototype.log = function(var_args) {};

/**
 * @param {...*} var_args
 */
Console.prototype.warn = function(var_args) {};

/**
 * @param {...*} var_args
 */
Console.prototype.debug = function(var_args) {};

/**
 * @param {*} value
 */
Console.prototype.dir = function(value) {};

/**
 * @param {...*} var_args
 */
Console.prototype.dirxml = function(var_args) {};

/**
 * @param {!Object} data
 * @param {*=} opt_columns
 */
Console.prototype.table = function(data, opt_columns) {};

/**
 * @return {undefined}
 */
Console.prototype.trace = function() {};

/**
 * @param {*} value
 */
Console.prototype.count = function(value) {};

/**
 * @param {*} value
 */
Console.prototype.markTimeline = function(value) {};

/**
 * @param {string=} opt_title
 */
Console.prototype.profile = function(opt_title) {};

/** @type {Array.<ScriptProfile>} */
Console.prototype.profiles;

/**
 * @param {string=} opt_title
 */
Console.prototype.profileEnd = function(opt_title) {};

/**
 * @param {string} name
 */
Console.prototype.time = function(name) {};

/**
 * @param {string} name
 */
Console.prototype.timeEnd = function(name) {};

/**
 * @param {*} value
 */
Console.prototype.timeStamp = function(value) {};

/**
 * @param {...*} var_args
 */
Console.prototype.group = function(var_args) {};

/**
 * @param {...*} var_args
 */
Console.prototype.groupCollapsed = function(var_args) {};

Console.prototype.groupEnd = function() {};

Console.prototype.clear = function() {};

/** @type {MemoryInfo} */
Console.prototype.memory;

/** @type {!Console} */
Window.prototype.console;

/**
 * @type {!Console}
 * @suppress {duplicate}
 */
var console;

/**
 * @type {number}
 * @see http://developer.android.com/reference/android/webkit/WebView.html
 */
Window.prototype.devicePixelRatio;

/** @type {Node} */
Selection.prototype.baseNode;

/** @type {number} */
Selection.prototype.baseOffset;

/** @type {Node} */
Selection.prototype.extentNode;

/** @type {number} */
Selection.prototype.extentOffset;

/** @type {string} */
Selection.prototype.type;

/**
 * @return {undefined}
 */
Selection.prototype.empty = function() {};

/**
 * @param {Node} baseNode
 * @param {number} baseOffset
 * @param {Node} extentNode
 * @param {number} extentOffset
 * @return {undefined}
 */
Selection.prototype.setBaseAndExtent =
 function(baseNode, baseOffset, extentNode, extentOffset) {};

/**
 * @param {string} alter
 * @param {string} direction
 * @param {string} granularity
 * @return {undefined}
 */
Selection.prototype.modify = function(alter, direction, granularity) {};

/**
 * @param {Element} element
 * @param {string} pseudoElement
 * @param {boolean=} opt_authorOnly
 * @return {CSSRuleList}
 * @nosideeffects
 */
ViewCSS.prototype.getMatchedCSSRules =
    function(element, pseudoElement, opt_authorOnly) {};

/**
 * @param {string} contextId
 * @param {string} name
 * @param {number} width
 * @param {number} height
 * @nosideeffects
 */
Document.prototype.getCSSCanvasContext =
    function(contextId, name, width, height) {};

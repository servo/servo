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
 * @fileoverview Definitions for all the extensions over some of the
 *  W3C's XML specifications by Gecko. This file depends on
 *  w3c_xml.js. The whole file has been fully type annotated.
 *
 * @externs
 */

/**
 * XMLSerializer can be used to convert DOM subtree or DOM document into text.
 * XMLSerializer is available to unprivileged scripts.
 *
 * XMLSerializer is mainly useful for applications and extensions based on
 * Mozilla platform. While it's available to web pages, it's not part of any
 * standard and level of support in other browsers is unknown.
 *
 * @constructor
 */
function XMLSerializer() {}

/**
 * Returns the serialized subtree in the form of a string
 * @param {Node} subtree
 * @return {string}
 */
XMLSerializer.prototype.serializeToString = function(subtree) {};

/**
 * The subtree rooted by the specified element is serialized to a byte stream
 * using the character set specified.
 *
 * @param {Node} subtree
 * @return {Object}
 */
XMLSerializer.prototype.serializeToStream = function(subtree) {};

/**
 * DOMParser is mainly useful for applications and extensions based on Mozilla
 * platform. While it's available to web pages, it's not part of any standard and
 * level of support in other browsers is unknown.
 *
 * @constructor
 */
function DOMParser() {}

/**
 * The string passed in is parsed into a DOM document.
 *
 * Example:
 *  var parser = new DOMParser();
 *  var doc = parser.parseFromString(aStr, "text/xml");
 *
 * @param {string} src The UTF16 string to be parsed.
 * @param {string} type The content type of the string.
 * @return {Document}
 */
DOMParser.prototype.parseFromString = function(src, type) {};

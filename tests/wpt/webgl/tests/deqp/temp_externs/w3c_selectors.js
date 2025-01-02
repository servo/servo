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
 * @fileoverview Definitions for W3C's Selectors API.
 *  This file depends on w3c_dom1.js.
 *  @see http://www.w3.org/TR/selectors-api2/
 *
 * @externs
 */

/**
 * @param {string} selectors
 * @return {Element}
 * @override
 */
Document.prototype.querySelector = function(selectors) {};

/**
 * @param {string} selectors
 * @return {!NodeList}
 * @override
 */
Document.prototype.querySelectorAll = function(selectors) {};

/**
 * @param {string} selectors
 * @return {Element}
 * @override
 */
Element.prototype.querySelector = function(selectors) {};

/**
 * @param {string} selectors
 * @return {!NodeList}
 * @override
 */
Element.prototype.querySelectorAll = function(selectors) {};

/**
 * https://dom.spec.whatwg.org/#dom-element-matches
 * https://developer.mozilla.org/en-US/docs/Web/API/Element.matches
 * @param {string} selectors
 * @return {boolean}
 */
Element.prototype.matches = function(selectors) {};

/**
 * @param {string} selectors
 * @param {(Node|NodeList)=} refNodes
 * @return {boolean}
 */
Element.prototype.matchesSelector = function(selectors, refNodes) {};

/**
 * @see https://developer.mozilla.org/en/DOM/Node.mozMatchesSelector
 * @param {string} selectors
 * @return {boolean}
 */
Element.prototype.mozMatchesSelector = function(selectors) {};

/**
 * @see http://developer.apple.com/library/safari/documentation/WebKit/Reference/ElementClassRef/Element/Element.html
 * @param {string} selectors
 * @return {boolean}
 */
Element.prototype.webkitMatchesSelector = function(selectors) {};

/**
 * @see http://msdn.microsoft.com/en-us/library/ff975201.aspx
 * @param {string} selectors
 * @return {boolean}
 */
Element.prototype.msMatchesSelector = function(selectors) {};

/**
 * @see http://www.opera.com/docs/changelogs/windows/1150/
 * @param {string} selectors
 * @return {boolean}
 */
Element.prototype.oMatchesSelector = function(selectors) {};

/*
 * Copyright 2011 The Closure Compiler Authors
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
 * @fileoverview Definitions for timing control for script base animations. The
 *  whole file has been fully type annotated.
 *
 * @see http://www.w3.org/TR/animation-timing/
 * @see http://webstuff.nfshost.com/anim-timing/Overview.html
 * @externs
 */

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element In early versions of this API, the callback
 *     was invoked only if the element was visible.
 * @return {number}
 */
function requestAnimationFrame(callback, opt_element) {};

/**
 * @param {number} handle
 */
function cancelRequestAnimationFrame(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
function webkitRequestAnimationFrame(callback, opt_element) {};

/**
 * @param {number} handle
 */
function webkitCancelRequestAnimationFrame(handle) {};

/**
 * @param {number} handle
 */
function webkitCancelAnimationFrame(handle) {};

/**
 * @param {?function(number)} callback It's legitimate to pass a null
 *     callback and listen on the MozBeforePaint event instead.
 * @param {Element=} opt_element
 * @return {number}
 */
function mozRequestAnimationFrame(callback, opt_element) {};

/**
 * @param {number} handle
 */
function mozCancelRequestAnimationFrame(handle) {};

/**
 * @param {number} handle
 */
function mozCancelAnimationFrame(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
function msRequestAnimationFrame(callback, opt_element) {};

/**
 * @param {number} handle
 */
function msCancelRequestAnimationFrame(handle) {};

/**
 * @param {number} handle
 */
function msCancelAnimationFrame(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
function oRequestAnimationFrame(callback, opt_element) {};

/**
 * @param {number} handle
 */
function oCancelRequestAnimationFrame(handle) {};

/**
 * @param {number} handle
 */
function oCancelAnimationFrame(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
Window.prototype.requestAnimationFrame = function(callback, opt_element) {};

/**
 * @param {number} handle
 */
Window.prototype.cancelRequestAnimationFrame = function(handle) {};

/**
 * @param {number} handle
 */
Window.prototype.cancelAnimationFrame = function(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
Window.prototype.webkitRequestAnimationFrame = function(callback, opt_element) {};

/**
 * @param {number} handle
 */
Window.prototype.webkitCancelRequestAnimationFrame = function(handle) {};

/**
 * @param {number} handle
 */
Window.prototype.webkitCancelAnimationFrame = function(handle) {};

/**
 * @param {?function(number)} callback It's legitimate to pass a null
 *     callback and listen on the MozBeforePaint event instead.
 * @param {Element=} opt_element
 * @return {number}
 */
Window.prototype.mozRequestAnimationFrame = function(callback, opt_element) {};

/**
 * @param {number} handle
 */
Window.prototype.mozCancelRequestAnimationFrame = function(handle) {};

/**
 * @param {number} handle
 */
Window.prototype.mozCancelAnimationFrame = function(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
Window.prototype.msRequestAnimationFrame = function(callback, opt_element) {};

/**
 * @param {number} handle
 */
Window.prototype.msCancelRequestAnimationFrame = function(handle) {};

/**
 * @param {number} handle
 */
Window.prototype.msCancelAnimationFrame = function(handle) {};

/**
 * @param {function(number)} callback
 * @param {Element=} opt_element
 * @return {number}
 */
Window.prototype.oRequestAnimationFrame = function(callback, opt_element) {};

/**
 * @param {number} handle
 */
Window.prototype.oCancelRequestAnimationFrame = function(handle) {};

/**
 * @param {number} handle
 */
Window.prototype.oCancelAnimationFrame = function(handle) {};

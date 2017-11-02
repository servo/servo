/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('framework.common.tcuPixelFormat');

goog.scope(function() {

var tcuPixelFormat = framework.common.tcuPixelFormat;

/**
 * @constructor
 * @param {number=} r
 * @param {number=} g
 * @param {number=} b
 * @param {number=} a
 */
tcuPixelFormat.PixelFormat = function(r, g, b, a) {
    this.redBits = r || 0;
    this.greenBits = g || 0;
    this.blueBits = b || 0;
    this.alphaBits = a || 0;
};

/**
 * @param {WebGL2RenderingContext} context
 * @return {tcuPixelFormat.PixelFormat}
 */
tcuPixelFormat.PixelFormatFromContext = function(context) {
    var r = /** @type {number} */ (context.getParameter(gl.RED_BITS));
    var g = /** @type {number} */ (context.getParameter(gl.GREEN_BITS));
    var b = /** @type {number} */ (context.getParameter(gl.BLUE_BITS));
    var a = /** @type {number} */ (context.getParameter(gl.ALPHA_BITS));

    return new tcuPixelFormat.PixelFormat(r, g, b, a);
};

/**
 * @param {number} r
 * @param {number} g
 * @param {number} b
 * @param {number} a
 * @return {boolean}
 */
tcuPixelFormat.PixelFormat.prototype.equals = function(r, g, b, a) {
    return this.redBits === r &&
            this.greenBits === g &&
            this.blueBits === b &&
            this.alphaBits === a;
};

/**
 * @return {Array<number>}
 */
tcuPixelFormat.PixelFormat.prototype.getColorThreshold = function() {
    return [1 << (8 - this.redBits),
            1 << (8 - this.greenBits),
            1 << (8 - this.blueBits),
            (this.alphaBits > 0) ? (1 << (8 - this.alphaBits)) : 0];
};

});

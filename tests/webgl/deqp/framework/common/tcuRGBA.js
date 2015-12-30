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
goog.provide('framework.common.tcuRGBA');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var tcuRGBA = framework.common.tcuRGBA;
var deMath = framework.delibs.debase.deMath;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * class tcuRGBA.RGBA
     * @constructor
     * @struct
     * @param {goog.NumberArray=} value
     */
    tcuRGBA.RGBA = function(value) {
        /** @type {goog.NumberArray} */ this.m_value = value || null;

    };

    /**
     * @enum
     * In JS, these are not shift values, but positions in a typed array
     */
    tcuRGBA.RGBA.Shift = {
        RED: 0,
        GREEN: 1,
        BLUE: 2,
        ALPHA: 3
    };

    /**
     * @enum
     * Mask used as flags
     * Hopefully will not use typed arrays
     */
    tcuRGBA.RGBA.Mask = function() {
        return {
            RED: false,
            GREEN: false,
            BLUE: false,
            ALPHA: false
        };
    };

    /**
     * Builds an tcuRGBA.RGBA object from color components
     * @param {number} r
     * @param {number} g
     * @param {number} b
     * @param {number} a
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.newRGBAComponents = function(r, g, b, a) {
        DE_ASSERT(deMath.deInRange32(r, 0, 255));
        DE_ASSERT(deMath.deInRange32(g, 0, 255));
        DE_ASSERT(deMath.deInRange32(b, 0, 255));
        DE_ASSERT(deMath.deInRange32(a, 0, 255));

        return new tcuRGBA.RGBA([r, g, b, a]);
    };

    /**
     * Builds an tcuRGBA.RGBA object from a number array
     * @param {goog.NumberArray} v
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.newRGBAFromArray = function(v) {
        return new tcuRGBA.RGBA(v.slice(0, 4));
    };

    /**
     * @param {number} value
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.newRGBAFromValue = function(value) {
        var rgba = new tcuRGBA.RGBA();
        var array32 = new Uint32Array([value]);
        rgba.m_value = (new Uint8Array(array32.buffer));
        return rgba;
    };

    /**
     * @param {number} v
     */
    tcuRGBA.RGBA.prototype.setRed = function(v) { DE_ASSERT(deMath.deInRange32(v, 0, 255)); this.m_value[tcuRGBA.RGBA.Shift.RED] = v; };

    /**
     * @param {number} v
     */
    tcuRGBA.RGBA.prototype.setGreen = function(v) { DE_ASSERT(deMath.deInRange32(v, 0, 255)); this.m_value[tcuRGBA.RGBA.Shift.GREEN] = v; };

    /**
     * @param {number} v
     */
    tcuRGBA.RGBA.prototype.setBlue = function(v) { DE_ASSERT(deMath.deInRange32(v, 0, 255)); this.m_value[tcuRGBA.RGBA.Shift.BLUE] = v; };

    /**
     * @param {number} v
     */
    tcuRGBA.RGBA.prototype.setAlpha = function(v) { DE_ASSERT(deMath.deInRange32(v, 0, 255)); this.m_value[tcuRGBA.RGBA.Shift.ALPHA] = v; };

    /**
     * @return {number}
     */
    tcuRGBA.RGBA.prototype.getRed = function() { return this.m_value[tcuRGBA.RGBA.Shift.RED]; };

    /**
     * @return {number}
     */
    tcuRGBA.RGBA.prototype.getGreen = function() { return this.m_value[tcuRGBA.RGBA.Shift.GREEN]; };

    /**
     * @return {number}
     */
    tcuRGBA.RGBA.prototype.getBlue = function() { return this.m_value[tcuRGBA.RGBA.Shift.BLUE]; };

    /**
     * @return {number}
     */
    tcuRGBA.RGBA.prototype.getAlpha = function() { return this.m_value[tcuRGBA.RGBA.Shift.ALPHA]; };

    /**
     * @param {tcuRGBA.RGBA} thr
     * @return {boolean}
     */
    tcuRGBA.RGBA.prototype.isBelowThreshold = function(thr) { return (this.getRed() <= thr.getRed()) && (this.getGreen() <= thr.getGreen()) && (this.getBlue() <= thr.getBlue()) && (this.getAlpha() <= thr.getAlpha()); };

    /**
     * @param {Uint8Array} bytes
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.RGBA.fromBytes = function(bytes) { return tcuRGBA.newRGBAFromArray(bytes); };

    /**
     * @param {Uint8Array} bytes
     */
    tcuRGBA.RGBA.prototype.toBytes = function(bytes) { var result = new Uint8Array(this.m_value); bytes[0] = result[0]; bytes[1] = result[1]; bytes[2] = result[2]; bytes[3] = result[3]; };

    /**
     * @return {Array<number>}
     */
    tcuRGBA.RGBA.prototype.toVec = function() {
        return [
            this.getRed() / 255.0,
            this.getGreen() / 255.0,
            this.getBlue() / 255.0,
            this.getAlpha() / 255.0
        ];
    };

    /**
     * @return {Array<number>}
     */
    tcuRGBA.RGBA.prototype.toIVec = function() {
        return [
            this.getRed(),
            this.getGreen(),
            this.getBlue(),
            this.getAlpha()
        ];
    };

    /**
     * @param {tcuRGBA.RGBA} v
     * @return {boolean}
     */
    tcuRGBA.RGBA.prototype.equals = function(v) {
        return (
            this.m_value[0] == v.m_value[0] &&
            this.m_value[1] == v.m_value[1] &&
            this.m_value[2] == v.m_value[2] &&
            this.m_value[3] == v.m_value[3]
        );
    };

    /**
     * @param {tcuRGBA.RGBA} a
     * @param {tcuRGBA.RGBA} b
     * @param {tcuRGBA.RGBA} threshold
     * @return {boolean}
     */
    tcuRGBA.compareThreshold = function(a, b, threshold) {
        if (a.equals(b)) return true; // Quick-accept
        return tcuRGBA.computeAbsDiff(a, b).isBelowThreshold(threshold);
    };

    /**
     * @param {tcuRGBA.RGBA} a
     * @param {tcuRGBA.RGBA} b
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.computeAbsDiff = function(a, b) {
        return tcuRGBA.newRGBAComponents(
            Math.abs(a.getRed() - b.getRed()),
            Math.abs(a.getGreen() - b.getGreen()),
            Math.abs(a.getBlue() - b.getBlue()),
            Math.abs(a.getAlpha() - b.getAlpha())
        );
    };

    /**
     * @param {tcuRGBA.RGBA} a
     * @param {number} b
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.multiply = function(a, b) {
        return tcuRGBA.newRGBAComponents(
            deMath.clamp(a.getRed() * b, 0, 255),
            deMath.clamp(a.getGreen() * b, 0, 255),
            deMath.clamp(a.getBlue() * b, 0, 255),
            deMath.clamp(a.getAlpha() * b, 0, 255));
    };

    /**
     * @param {tcuRGBA.RGBA} a
     * @param {tcuRGBA.RGBA} b
     * @return {tcuRGBA.RGBA}
     */
    tcuRGBA.max = function(a, b) {
        return tcuRGBA.newRGBAComponents(
            Math.max(a.getRed(), b.getRed()),
            Math.max(a.getGreen(), b.getGreen()),
            Math.max(a.getBlue(), b.getBlue()),
            Math.max(a.getAlpha(), b.getAlpha())
        );
    };

    tcuRGBA.RGBA.prototype.toString = function() {
        return '[' + this.m_value[0] + ',' + this.m_value[1] + ',' + this.m_value[2] + ',' + this.m_value[3] + ']';
    };

    // Color constants
    tcuRGBA.RGBA.red = tcuRGBA.newRGBAComponents(0xFF, 0, 0, 0xFF);
    tcuRGBA.RGBA.green = tcuRGBA.newRGBAComponents(0, 0xFF, 0, 0xFF);
    tcuRGBA.RGBA.blue = tcuRGBA.newRGBAComponents(0, 0, 0xFF, 0xFF);
    tcuRGBA.RGBA.gray = tcuRGBA.newRGBAComponents(0x80, 0x80, 0x80, 0xFF);
    tcuRGBA.RGBA.white = tcuRGBA.newRGBAComponents(0xFF, 0xFF, 0xFF, 0xFF);
    tcuRGBA.RGBA.black = tcuRGBA.newRGBAComponents(0, 0, 0, 0xFF);

});

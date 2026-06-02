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
goog.provide('framework.common.tcuTexLookupVerifier');
goog.require('framework.common.tcuTexVerifierUtil');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

    var tcuTexLookupVerifier = framework.common.tcuTexLookupVerifier;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var tcuTexVerifierUtil = framework.common.tcuTexVerifierUtil;
    var deMath = framework.delibs.debase.deMath;

    /** @typedef {(tcuTexLookupVerifier.LookupPrecision|{tcuTexLookupVerifier.LookupPrecision})} */
    tcuTexLookupVerifier.PrecType;

    /**
     * Generic lookup precision parameters
     * @constructor
     * @struct
     * @param {Array<number>=} coordBits
     * @param {Array<number>=} uvwBits
     * @param {Array<number>=} colorThreshold
     * @param {Array<boolean>=} colorMask
     */
    tcuTexLookupVerifier.LookupPrecision = function(coordBits, uvwBits, colorThreshold, colorMask) {
        /** @type {Array<number>} */ this.coordBits = coordBits || [22, 22, 22];
        /** @type {Array<number>} */ this.uvwBits = uvwBits || [16, 16, 16];
        /** @type {Array<number>} */ this.colorThreshold = colorThreshold || [0, 0, 0, 0];
        /** @type {Array<boolean>} */ this.colorMask = colorMask || [true, true, true, true];
    };

    /**
     * Lod computation precision parameters
     * @constructor
     * @struct
     * @param {number=} derivateBits
     * @param {number=} lodBits
     */
    tcuTexLookupVerifier.LodPrecision = function(derivateBits, lodBits) {
        /** @type {number} */ this.derivateBits = derivateBits === undefined ? 22 : derivateBits;
        /** @type {number} */ this.lodBits = lodBits === undefined ? 16 : lodBits;
    };

    /**
     * @enum {number}
     */
    tcuTexLookupVerifier.TexLookupScaleMode = {
        MINIFY: 0,
        MAGNIFY: 1
    };

    // Generic utilities

    /**
     * @param {tcuTexture.Sampler} sampler
     * @return {boolean}
     */
    tcuTexLookupVerifier.isSamplerSupported = function(sampler) {
        return sampler.compare == tcuTexture.CompareMode.COMPAREMODE_NONE &&
            tcuTexVerifierUtil.isWrapModeSupported(sampler.wrapS) &&
            tcuTexVerifierUtil.isWrapModeSupported(sampler.wrapT) &&
            tcuTexVerifierUtil.isWrapModeSupported(sampler.wrapR);
    };

    // Color read & compare utilities

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @return {boolean}
     */
    tcuTexLookupVerifier.coordsInBounds = function(access, x, y, z) {
        return deMath.deInBounds32(x, 0, access.getWidth()) && deMath.deInBounds32(y, 0, access.getHeight()) && deMath.deInBounds32(z, 0, access.getDepth());
    };

    /**
     * @param {tcuTexture.TextureFormat} format
     * @return {boolean}
     */
    tcuTexLookupVerifier.isSRGB = function(format) {
        return format.order == tcuTexture.ChannelOrder.sRGB || format.order == tcuTexture.ChannelOrder.sRGBA;
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {tcuTexture.Sampler} sampler
     * @param {number} i
     * @param {number} j
     * @param {number} k
     * @return {Array<number>}
     */
    tcuTexLookupVerifier.lookupScalar = function(access, sampler, i, j, k) {
        if (tcuTexLookupVerifier.coordsInBounds(access, i, j, k))
            return access.getPixel(i, j, k);
        else
            return deMath.toIVec(sampler.borderColor);
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {tcuTexture.Sampler} sampler
     * @param {number} i
     * @param {number} j
     * @param {number} k
     * @return {Array<number>}
     */
    tcuTexLookupVerifier.lookupFloat = function(access, sampler, i, j, k) {
        // Specialization for float lookups: sRGB conversion is performed as specified in format.
        if (tcuTexLookupVerifier.coordsInBounds(access, i, j, k)) {
            /** @type {Array<number>} */ var p = access.getPixel(i, j, k);
            return tcuTexLookupVerifier.isSRGB(access.getFormat()) ? tcuTextureUtil.sRGBToLinear(p) : p;
        } else
            return sampler.borderColor;
    };
    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} ref
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isColorValid = function(prec, ref, result) {
        return deMath.boolAll(
            deMath.logicalOrBool(
                deMath.lessThanEqual(deMath.absDiff(ref, result), prec.colorThreshold),
                deMath.logicalNotBool(prec.colorMask)));
    };

    /**
     * @constructor
     * @struct
     * @param {Array<number>=} p00
     * @param {Array<number>=} p01
     * @param {Array<number>=} p10
     * @param {Array<number>=} p11
     */
    tcuTexLookupVerifier.ColorQuad = function(p00, p01, p10, p11) {
        /** @type {Array<number>} */ this.p00 = p00 || null; //!< (0, 0)
        /** @type {Array<number>} */ this.p01 = p01 || null; //!< (1, 0)
        /** @type {Array<number>} */ this.p10 = p10 || null; //!< (0, 1)
        /** @type {Array<number>} */ this.p11 = p11 || null; //!< (1, 1)
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} level
     * @param {tcuTexture.Sampler} sampler
     * @param {number} x0
     * @param {number} x1
     * @param {number} y0
     * @param {number} y1
     * @param {number} z
     * @return {tcuTexLookupVerifier.ColorQuad}
     */
    tcuTexLookupVerifier.lookupQuad = function(level, sampler, x0, x1, y0, y1, z) {
        var p00 = tcuTexLookupVerifier.lookupFloat(level, sampler, x0, y0, z);
        var p10 = tcuTexLookupVerifier.lookupFloat(level, sampler, x1, y0, z);
        var p01 = tcuTexLookupVerifier.lookupFloat(level, sampler, x0, y1, z);
        var p11 = tcuTexLookupVerifier.lookupFloat(level, sampler, x1, y1, z);
        return new tcuTexLookupVerifier.ColorQuad(p00, p01, p10, p11);
    };

    /**
     * @constructor
     * @struct
     * @param {Array<number>=} p0
     * @param {Array<number>=} p1
     */
    tcuTexLookupVerifier.ColorLine = function(p0, p1) {
        /** @type {Array<number>} */ this.p0 = p0 || null; //!< 0
        /** @type {Array<number>} */ this.p1 = p1 || null; //!< 1
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} level
     * @param {tcuTexture.Sampler} sampler
     * @param {number} x0
     * @param {number} x1
     * @param {number} y
     * @return {tcuTexLookupVerifier.ColorLine}
     */
    tcuTexLookupVerifier.lookupLine = function(level, sampler, x0, x1, y) {
        return new tcuTexLookupVerifier.ColorLine(
            tcuTexLookupVerifier.lookupFloat(level, sampler, x0, y, 0),
            tcuTexLookupVerifier.lookupFloat(level, sampler, x1, y, 0)
        );
    };

    /**
     * @param {Array<number>} vec
     * @return {number}
     */
    tcuTexLookupVerifier.minComp = function(vec) {
        /** @type {number} */ var minVal = vec[0];
        for (var ndx = 1; ndx < vec.length; ndx++)
            minVal = Math.min(minVal, vec[ndx]);
        return minVal;
    };

    /**
     * @param {Array<number>} vec
     * @return {number}
     */
    tcuTexLookupVerifier.maxComp = function(vec) {
        /** @type {number} */ var maxVal = vec[0];
        for (var ndx = 1; ndx < vec.length; ndx++)
            maxVal = Math.max(maxVal, vec[ndx]);
        return maxVal;
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexLookupVerifier.ColorLine} line
     * @return {number}
     */
    tcuTexLookupVerifier.computeBilinearSearchStepFromFloatLine = function(prec, line) {
        assertMsgOptions(deMath.boolAll(deMath.greaterThan(prec.colorThreshold, [0, 0, 0, 0])), 'Threshold not greater than 0.', false, true);

        /** @type {number} */ var maxSteps = 1 << 16;
        /** @type {Array<number>} */ var d = deMath.absDiff(line.p1, line.p0);
        /** @type {Array<number>} */ var stepCount = deMath.divide([d, d, d, d], prec.colorThreshold);
        /** @type {Array<number>} */
        var minStep = deMath.divide([1, 1, 1, 1], deMath.add(stepCount, [1, 1, 1, 1]));
        /** @type {number} */ var step = Math.max(tcuTexLookupVerifier.minComp(minStep), 1 / maxSteps);

        return step;
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexLookupVerifier.ColorQuad} quad
     * @return {number}
     */
    tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad = function(prec, quad) {
        assertMsgOptions(deMath.boolAll(deMath.greaterThan(prec.colorThreshold, [0, 0, 0, 0])), 'Threshold not greater than 0.', false, true);

        /** @type {number} */ var maxSteps = 1 << 16;
        /** @type {Array<number>} */ var d0 = deMath.absDiff(quad.p10, quad.p00);
        /** @type {Array<number>} */ var d1 = deMath.absDiff(quad.p01, quad.p00);
        /** @type {Array<number>} */ var d2 = deMath.absDiff(quad.p11, quad.p10);
        /** @type {Array<number>} */ var d3 = deMath.absDiff(quad.p11, quad.p01);
        /** @type {Array<number>} */ var maxD = deMath.max(d0, deMath.max(d1, deMath.max(d2, d3)));
        /** @type {Array<number>} */ var stepCount = deMath.divide(maxD, prec.colorThreshold);
        /** @type {Array<number>} */ var minStep = deMath.divide([1, 1, 1, 1], deMath.add(stepCount, [1, 1, 1, 1]));
        /** @type {number} */ var step = Math.max(tcuTexLookupVerifier.minComp(minStep), 1 / maxSteps);

        return step;
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @return {number}
     */
    tcuTexLookupVerifier.computeBilinearSearchStepForUnorm = function(prec) {
        assertMsgOptions(deMath.boolAll(deMath.greaterThan(prec.colorThreshold, [0, 0, 0, 0])), 'Threshold not greater than 0.', false, true);

        /** @type {Array<number>} */ var stepCount = deMath.divide([1, 1, 1, 1], prec.colorThreshold);
        /** @type {Array<number>} */ var minStep = deMath.divide([1, 1, 1, 1], (deMath.add(stepCount, [1, 1, 1, 1])));
        /** @type {number} */ var step = tcuTexLookupVerifier.minComp(minStep);

        return step;
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @return {number}
     */
    tcuTexLookupVerifier.computeBilinearSearchStepForSnorm = function(prec) {
        assertMsgOptions(deMath.boolAll(deMath.greaterThan(prec.colorThreshold, [0, 0, 0, 0])), 'Threshold not greater than 0.', false, true);

        /** @type {Array<number>} */ var stepCount = deMath.divide([2.0, 2.0, 2.0, 2.0], prec.colorThreshold);
        /** @type {Array<number>} */ var minStep = deMath.divide([1, 1, 1, 1], deMath.add(stepCount, [1, 1, 1, 1]));
        /** @type {number} */ var step = tcuTexLookupVerifier.minComp(minStep);

        return step;
    };

    /**
     * @param {tcuTexLookupVerifier.ColorLine} line
     * @return {Array<number>}
     */
    tcuTexLookupVerifier.minLine = function(line) {
        return deMath.min(line.p0, line.p1);
    };

    /**
    * @param {tcuTexLookupVerifier.ColorLine} line
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.maxLine = function(line) {
        var max = deMath.max;
        return max(line.p0, line.p1);
    };

    /**
    * @param {tcuTexLookupVerifier.ColorQuad} quad
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.minQuad = function(quad) {
        var min = deMath.min;
        return min(quad.p00, min(quad.p10, min(quad.p01, quad.p11)));
    };

    /**
    * @param {tcuTexLookupVerifier.ColorQuad} quad
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.maxQuad = function(quad) {
        var max = deMath.max;
        return max(quad.p00, max(quad.p10, max(quad.p01, quad.p11)));
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexLookupVerifier.ColorQuad} quad
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isInColorBounds_1Quad = function(prec, quad, result) {
        var quadMin = tcuTexLookupVerifier.minQuad;
        var quadMax = tcuTexLookupVerifier.maxQuad;
        /** @type {Array<number>} */ var minVal = deMath.subtract(quadMin(quad), prec.colorThreshold);
        /** @type {Array<number>} */ var maxVal = deMath.add(quadMax(quad), prec.colorThreshold);
        return deMath.boolAll(
            deMath.logicalOrBool(
                deMath.logicalAndBool(
                    deMath.greaterThanEqual(result, minVal),
                    deMath.lessThanEqual(result, maxVal)),
                deMath.logicalNotBool(prec.colorMask)));
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexLookupVerifier.ColorQuad} quad0
     * @param {tcuTexLookupVerifier.ColorQuad} quad1
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isInColorBounds_2Quad = function(prec, quad0, quad1, result) {
        var min = deMath.min;
        var max = deMath.max;
        var quadMin = tcuTexLookupVerifier.minQuad;
        var quadMax = tcuTexLookupVerifier.maxQuad;
        /** @type {Array<number>} */ var minVal = deMath.subtract(min(quadMin(quad0), quadMin(quad1)), prec.colorThreshold);
        /** @type {Array<number>} */ var maxVal = deMath.add(max(quadMax(quad0), quadMax(quad1)), prec.colorThreshold);
        return deMath.boolAll(
            deMath.logicalOrBool(
                deMath.logicalAndBool(
                    deMath.greaterThanEqual(result, minVal),
                    deMath.lessThanEqual(result, maxVal)),
                deMath.logicalNotBool(prec.colorMask)));
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexLookupVerifier.ColorLine} line0
     * @param {tcuTexLookupVerifier.ColorLine} line1
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isInColorBounds_2Line = function(prec, line0, line1, result) {
        var min = deMath.min;
        var max = deMath.max;
        var lineMin = tcuTexLookupVerifier.minLine;
        var lineMax = tcuTexLookupVerifier.maxLine;
        /** @type {Array<number>} */ var minVal = deMath.subtract(min(lineMin(line0), lineMin(line1)), prec.colorThreshold);
        /** @type {Array<number>} */ var maxVal = deMath.add(max(lineMax(line0), lineMax(line1)), prec.colorThreshold);
        return deMath.boolAll(
            deMath.logicalOrBool(
                deMath.logicalAndBool(
                    deMath.greaterThanEqual(result, minVal),
                    deMath.lessThanEqual(result, maxVal)),
                deMath.logicalNotBool(prec.colorMask)));
    };

    /**
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {tcuTexLookupVerifier.ColorQuad} quad00
    * @param {tcuTexLookupVerifier.ColorQuad} quad01
    * @param {tcuTexLookupVerifier.ColorQuad} quad10
    * @param {tcuTexLookupVerifier.ColorQuad} quad11
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isInColorBounds_4Quad = function(prec, quad00, quad01, quad10, quad11, result) {
        var min = deMath.min;
        var max = deMath.max;
        var quadMin = tcuTexLookupVerifier.minQuad;
        var quadMax = tcuTexLookupVerifier.maxQuad;
        /** @type {Array<number>} */ var minVal = deMath.subtract(min(quadMin(quad00), min(quadMin(quad01), min(quadMin(quad10), quadMin(quad11)))), prec.colorThreshold);
        /** @type {Array<number>} */ var maxVal = deMath.add(max(quadMax(quad00), max(quadMax(quad01), max(quadMax(quad10), quadMax(quad11)))), prec.colorThreshold);
        return deMath.boolAll(
            deMath.logicalOrBool(
                deMath.logicalAndBool(
                    deMath.greaterThanEqual(result, minVal),
                    deMath.lessThanEqual(result, maxVal)),
                deMath.logicalNotBool(prec.colorMask)));
    };

    // Range search utilities

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} c0
     * @param {Array<number>} c1
     * @param {Array<number>} fBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLinearRangeValid = function(prec, c0, c1, fBounds, result) {
        // This is basically line segment - AABB test. Valid interpolation line is checked
        // against result AABB constructed by applying threshold.

        /** @type {Array<number>} */ var rMin = deMath.subtract(result, prec.colorThreshold);
        /** @type {Array<number>} */ var rMax = deMath.add(result, prec.colorThreshold);

        // Algorithm: For each component check whether segment endpoints are inside, or intersect with slab.
        // If all intersect or are inside, line segment intersects the whole 4D AABB.
        for (var compNdx = 0; compNdx < 4; compNdx++) {
            if (!prec.colorMask[compNdx])
                continue;

            /** @type {number} */ var i0 = c0[compNdx] * (1 - fBounds[0]) + c1[compNdx] * fBounds[0];
            /** @type {number} */ var i1 = c0[compNdx] * (1 - fBounds[1]) + c1[compNdx] * fBounds[1];
            if ((i0 > rMax[compNdx] && i1 > rMax[compNdx]) ||
                (i0 < rMin[compNdx] && i1 < rMin[compNdx])) {
                return false;
            }
        }

        return true;
    };

    /**
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexLookupVerifier.ColorQuad} quad
     * @param {Array<number>} xBounds
     * @param {Array<number>} yBounds
     * @param {number} searchStep
     * @param {Array<number>} result
     * @return {boolean}
    */
    tcuTexLookupVerifier.isBilinearRangeValid = function(prec, quad, xBounds, yBounds, searchStep, result) {
        assertMsgOptions(xBounds[0] <= xBounds[1], 'Out of bounds: X direction.', false, true);
        assertMsgOptions(yBounds[0] <= yBounds[1], 'Out of bounds: Y direction.', false, true);

        if (!tcuTexLookupVerifier.isInColorBounds_1Quad(prec, quad, result))
            return false;

        for (var x = xBounds[0]; x < xBounds[1] + searchStep; x += searchStep) {
            /** @type {number} */ var a = Math.min(x, xBounds[1]);
            /** @type {Array<number>} */ var c0 = deMath.add(deMath.scale(quad.p00, (1 - a)), deMath.scale(quad.p10, a));
            /** @type {Array<number>} */ var c1 = deMath.add(deMath.scale(quad.p01, (1 - a)), deMath.scale(quad.p11, a));

            if (tcuTexLookupVerifier.isLinearRangeValid(prec, c0, c1, yBounds, result))
                return true;
        }

        return false;
    };

    /**
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {tcuTexLookupVerifier.ColorQuad} quad0
    * @param {tcuTexLookupVerifier.ColorQuad} quad1
    * @param {Array<number>} xBounds
    * @param {Array<number>} yBounds
    * @param {Array<number>} zBounds
    * @param {number} searchStep
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isTrilinearRangeValid = function(prec, quad0, quad1, xBounds, yBounds, zBounds, searchStep, result) {
        assertMsgOptions(xBounds[0] <= xBounds[1], 'Out of bounds: X direction.', false, true);
        assertMsgOptions(yBounds[0] <= yBounds[1], 'Out of bounds: Y direction.', false, true);
        assertMsgOptions(zBounds[0] <= zBounds[1], 'Out of bounds: Z direction.', false, true);

        if (!tcuTexLookupVerifier.isInColorBounds_2Quad(prec, quad0, quad1, result))
            return false;

        for (var x = xBounds[0]; x < xBounds[1] + searchStep; x += searchStep) {
            for (var y = yBounds[0]; y < yBounds[1] + searchStep; y += searchStep) {
                /** @type {number} */ var a = Math.min(x, xBounds[1]);
                /** @type {number} */ var b = Math.min(y, yBounds[1]);
                /** @type {Array<number>} */
                var c0 = deMath.add(
                            deMath.add(
                                deMath.add(
                                    deMath.scale(quad0.p00, (1 - a) * (1 - b)),
                                    deMath.scale(quad0.p10, a * (1 - b))),
                                deMath.scale(quad0.p01, (1 - a) * b)),
                            deMath.scale(quad0.p11, a * b));
                /** @type {Array<number>} */
                var c1 = deMath.add(
                            deMath.add(
                                deMath.add(
                                    deMath.scale(quad1.p00, (1 - a) * (1 - b)),
                                    deMath.scale(quad1.p10, a * (1 - b))),
                                deMath.scale(quad1.p01, (1 - a) * b)),
                            deMath.scale(quad1.p11, a * b));

                if (tcuTexLookupVerifier.isLinearRangeValid(prec, c0, c1, zBounds, result))
                    return true;
            }
        }

        return false;
    };

    /**
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {tcuTexLookupVerifier.ColorQuad} quad0
    * @param {tcuTexLookupVerifier.ColorQuad} quad1
    * @param {Array<number>} xBounds0
    * @param {Array<number>} yBounds0
    * @param {Array<number>} xBounds1
    * @param {Array<number>} yBounds1
    * @param {Array<number>} zBounds
    * @param {number} searchStep
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.is2DTrilinearFilterResultValid = function(prec, quad0, quad1, xBounds0, yBounds0, xBounds1, yBounds1, zBounds, searchStep, result) {
        assertMsgOptions(xBounds0[0] <= xBounds0[1], 'Out of bounds: X direction.', false, true);
        assertMsgOptions(yBounds0[0] <= yBounds0[1], 'Out of bounds: Y direction.', false, true);
        assertMsgOptions(xBounds1[0] <= xBounds1[1], 'Out of bounds: X direction.', false, true);
        assertMsgOptions(yBounds1[0] <= yBounds1[1], 'Out of bounds: Y direction.', false, true);

        if (!tcuTexLookupVerifier.isInColorBounds_2Quad(prec, quad0, quad1, result))
            return false;

        for (var x0 = xBounds0[0]; x0 < xBounds0[1] + searchStep; x0 += searchStep) {
            for (var y0 = yBounds0[0]; y0 < yBounds0[1] + searchStep; y0 += searchStep) {
                /** @type {number} */ var a0 = Math.min(x0, xBounds0[1]);
                /** @type {number} */ var b0 = Math.min(y0, yBounds0[1]);
                /** @type {Array<number>} */
                var c0 = deMath.add(
                            deMath.add(
                                deMath.add(
                                    deMath.scale(quad0.p00, (1 - a0) * (1 - b0)),
                                    deMath.scale(quad0.p10, a0 * (1 - b0))),
                                deMath.scale(quad0.p01, (1 - a0) * b0)),
                            deMath.scale(quad0.p11, a0 * b0));

                for (var x1 = xBounds1[0]; x1 <= xBounds1[1]; x1 += searchStep) {
                    for (var y1 = yBounds1[0]; y1 <= yBounds1[1]; y1 += searchStep) {
                        /** @type {number} */ var a1 = Math.min(x1, xBounds1[1]);
                        /** @type {number} */ var b1 = Math.min(y1, yBounds1[1]);
                        /** @type {Array<number>} */
                        var c1 = deMath.add(
                                    deMath.add(
                                        deMath.add(
                                            deMath.scale(quad1.p00, (1 - a1) * (1 - b1)),
                                            deMath.scale(quad1.p10, a1 * (1 - b1))),
                                        deMath.scale(quad1.p01, (1 - a1) * b1)),
                                    deMath.scale(quad1.p11, a1 * b1));

                        if (tcuTexLookupVerifier.isLinearRangeValid(prec, c0, c1, zBounds, result))
                            return true;
                    }
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {tcuTexLookupVerifier.ColorQuad} quad00
    * @param {tcuTexLookupVerifier.ColorQuad} quad01
    * @param {tcuTexLookupVerifier.ColorQuad} quad10
    * @param {tcuTexLookupVerifier.ColorQuad} quad11
    * @param {Array<number>} xBounds0
    * @param {Array<number>} yBounds0
    * @param {Array<number>} zBounds0
    * @param {Array<number>} xBounds1
    * @param {Array<number>} yBounds1
    * @param {Array<number>} zBounds1
    * @param {Array<number>} wBounds
    * @param {number} searchStep
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.is3DTrilinearFilterResultValid = function(prec, quad00, quad01, quad10, quad11, xBounds0, yBounds0, zBounds0, xBounds1, yBounds1, zBounds1, wBounds, searchStep, result) {
        assertMsgOptions(xBounds0[0] <= xBounds0[1], 'Out of bounds: X direction.', false, true);
        assertMsgOptions(yBounds0[0] <= yBounds0[1], 'Out of bounds: Y direction.', false, true);
        assertMsgOptions(zBounds0[0] <= zBounds0[1], 'Out of bounds: Z direction.', false, true);
        assertMsgOptions(xBounds1[0] <= xBounds1[1], 'Out of bounds: X direction.', false, true);
        assertMsgOptions(yBounds1[0] <= yBounds1[1], 'Out of bounds: Y direction.', false, true);
        assertMsgOptions(zBounds1[0] <= zBounds1[1], 'Out of bounds: Z direction.', false, true);

        if (!tcuTexLookupVerifier.isInColorBounds_4Quad(prec, quad00, quad01, quad10, quad11, result))
            return false;

        function biInterp(result, p00, p01, p10, p11, s00, s01, s10, s11) {
            for (var ii = 0; ii < 4; ++ii) {
                result[ii] = p00[ii] * s00 + p10[ii] * s10 + p01[ii] * s01 + p11[ii] * s11;
            }
        }

        function interp(result, p0, p1, s) {
            for (var ii = 0; ii < 4; ++ii) {
                result[ii] = p0[ii] * (1 - s) + p1[ii] * s;
            }
        }

        /** @type {Array<number>} */ var c00 = [0, 0, 0, 0];
        /** @type {Array<number>} */ var c01 = [0, 0, 0, 0];
        /** @type {Array<number>} */ var c10 = [0, 0, 0, 0];
        /** @type {Array<number>} */ var c11 = [0, 0, 0, 0];
        /** @type {Array<number>} */ var cz0 = [0, 0, 0, 0];
        /** @type {Array<number>} */ var cz1 = [0, 0, 0, 0];

        for (var x0 = xBounds0[0]; x0 < xBounds0[1] + searchStep; x0 += searchStep) {
            for (var y0 = yBounds0[0]; y0 < yBounds0[1] + searchStep; y0 += searchStep) {
                /** @type {number} */ var a0 = Math.min(x0, xBounds0[1]);
                /** @type {number} */ var b0 = Math.min(y0, yBounds0[1]);

                /** @type {number} */ var s00 = (1 - a0) * (1 - b0);
                /** @type {number} */ var s01 = (1 - a0) * b0;
                /** @type {number} */ var s10 = a0 * (1 - b0);
                /** @type {number} */ var s11 = a0 * b0;

                biInterp(c00, quad00.p00, quad00.p01, quad00.p10, quad00.p11, s00, s01, s10, s11);
                biInterp(c01, quad01.p00, quad01.p01, quad01.p10, quad01.p11, s00, s01, s10, s11);

                for (var z0 = zBounds0[0]; z0 < zBounds0[1] + searchStep; z0 += searchStep) {
                    /** @type {number} */ var c0 = Math.min(z0, zBounds0[1]);
                    interp(cz0, c00, c01, c0);

                    for (var x1 = xBounds1[0]; x1 < xBounds1[1] + searchStep; x1 += searchStep) {
                        for (var y1 = yBounds1[0]; y1 < yBounds1[1] + searchStep; y1 += searchStep) {
                            /** @type {number} */ var a1 = Math.min(x1, xBounds1[1]);
                            /** @type {number} */ var b1 = Math.min(y1, yBounds1[1]);

                            /** @type {number} */ var t00 = (1 - a1) * (1 - b1);
                            /** @type {number} */ var t01 = (1 - a1) * b1;
                            /** @type {number} */ var t10 = a1 * (1 - b1);
                            /** @type {number} */ var t11 = a1 * b1;

                            biInterp(c10, quad10.p00, quad10.p01, quad10.p10, quad10.p11, t00, t01, t10, t11);
                            biInterp(c11, quad11.p00, quad11.p01, quad11.p10, quad11.p11, t00, t01, t10, t11);

                            for (var z1 = zBounds1[0]; z1 < zBounds1[1] + searchStep; z1 += searchStep) {
                                /** @type {number} */ var c1 = Math.min(z1, zBounds1[1]);
                                interp(cz1, c10, c11, c1);

                                if (tcuTexLookupVerifier.isLinearRangeValid(prec, cz0, cz1, wBounds, result))
                                    return true;
                            }
                        }
                    }
                }
            }
        }

        return false;
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} level
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {number} coordX
     * @param {number} coordY
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isNearestSampleResultValid_CoordXYAsNumber = function(level, sampler, prec, coordX, coordY, result) {
        assertMsgOptions(level.getDepth() == 1, 'Depth must be 1.', false, true);

        /** @type {Array<number>} */
        var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getWidth(), coordX, prec.coordBits[0], prec.uvwBits[0]);

        /** @type {number} */ var minI = Math.floor(uBounds[0]);
        /** @type {number} */ var maxI = Math.floor(uBounds[1]);

        for (var i = minI; i <= maxI; i++) {
            /** @type {number} */ var x = tcuTexVerifierUtil.wrap(sampler.wrapS, i, level.getWidth());
            /** @type {Array<number>} */ var color;
            if (tcuTexLookupVerifier.isSRGB(level.getFormat())) {
                color = tcuTexLookupVerifier.lookupFloat(level, sampler, x, coordY, 0);
            } else {
                color = tcuTexLookupVerifier.lookupScalar(level, sampler, x, coordY, 0);
            }

            if (tcuTexLookupVerifier.isColorValid(prec, color, result))
                return true;
        }

        return false;
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} level
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord vec2
     * @param {number} coordZ int
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec2AndInt = function(level, sampler, prec, coord, coordZ, result) {
        /** @type {Array<number>} */
        var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getWidth(), coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getHeight(), coord[1], prec.coordBits[1], prec.uvwBits[1]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI = Math.floor(uBounds[0]);
        /** @type {number} */ var maxI = Math.floor(uBounds[1]);
        /** @type {number} */ var minJ = Math.floor(vBounds[0]);
        /** @type {number} */ var maxJ = Math.floor(vBounds[1]);

        // \todo [2013-07-03 pyry] This could be optimized by first computing ranges based on wrap mode.

        for (var j = minJ; j <= maxJ; j++)
        for (var i = minI; i <= maxI; i++) {
            /** @type {number} */ var x = tcuTexVerifierUtil.wrap(sampler.wrapS, i, level.getWidth());
            /** @type {number} */ var y = tcuTexVerifierUtil.wrap(sampler.wrapT, j, level.getHeight());
            /** @type {Array<number>} */ var color;
            if (tcuTexLookupVerifier.isSRGB(level.getFormat())) {
                color = tcuTexLookupVerifier.lookupFloat(level, sampler, x, y, coordZ);
            } else {
                color = tcuTexLookupVerifier.lookupScalar(level, sampler, x, y, coordZ);
            }

            if (tcuTexLookupVerifier.isColorValid(prec, color, result))
                return true;
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord vec3
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec3 = function(level, sampler, prec, coord, result) {
        /** @type {Array<number>} */
        var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getWidth(), coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getHeight(), coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var wBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getDepth(), coord[2], prec.coordBits[2], prec.uvwBits[2]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI = Math.floor(uBounds[0]);
        /** @type {number} */ var maxI = Math.floor(uBounds[1]);
        /** @type {number} */ var minJ = Math.floor(vBounds[0]);
        /** @type {number} */ var maxJ = Math.floor(vBounds[1]);
        /** @type {number} */ var minK = Math.floor(wBounds[0]);
        /** @type {number} */ var maxK = Math.floor(wBounds[1]);

        // \todo [2013-07-03 pyry] This could be optimized by first computing ranges based on wrap mode.

        for (var k = minK; k <= maxK; k++) {
            for (var j = minJ; j <= maxJ; j++) {
                for (var i = minI; i <= maxI; i++) {
                    /** @type {number} */ var x = tcuTexVerifierUtil.wrap(sampler.wrapS, i, level.getWidth());
                    /** @type {number} */ var y = tcuTexVerifierUtil.wrap(sampler.wrapT, j, level.getHeight());
                    /** @type {number} */ var z = tcuTexVerifierUtil.wrap(sampler.wrapR, k, level.getDepth());
                    /** @type {Array<number>} */ var color;
                    if (tcuTexLookupVerifier.isSRGB(level.getFormat())) {
                        color = tcuTexLookupVerifier.lookupFloat(level, sampler, x, y, z);
                    } else {
                        color = tcuTexLookupVerifier.lookupScalar(level, sampler, x, y, z);
                    }

                    if (tcuTexLookupVerifier.isColorValid(prec, color, result))
                        return true;
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {number} coordX
    * @param {number} coordY
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLinearSampleResultValid_CoordXYAsNumber = function(level, sampler, prec, coordX, coordY, result) {
        /** @type {Array<number>} */ var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getWidth(), coordX, prec.coordBits[0], prec.uvwBits[0]);

        /** @type {number} */ var minI = Math.floor(uBounds[0] - 0.5);
        /** @type {number} */ var maxI = Math.floor(uBounds[1] - 0.5);

        /** @type {number} */ var w = level.getWidth();

        for (var i = minI; i <= maxI; i++) {
            // Wrapped coordinates
            /** @type {number} */ var x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i, w);
            /** @type {number} */ var x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i + 1, w);

            // Bounds for filtering factors
            /** @type {number} */ var minA = deMath.clamp((uBounds[0] - 0.5) - i, 0, 1);
            /** @type {number} */ var maxA = deMath.clamp((uBounds[1] - 0.5) - i, 0, 1);

            /** @type {Array<number>} */ var colorA = tcuTexLookupVerifier.lookupFloat(level, sampler, x0, coordY, 0);
            /** @type {Array<number>} */ var colorB = tcuTexLookupVerifier.lookupFloat(level, sampler, x1, coordY, 0);

            if (tcuTexLookupVerifier.isLinearRangeValid(prec, colorA, colorB, [minA, maxA], result))
                return true;
        }

        return false;
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} level
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord vec2
     * @param {number} coordZ int
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLinearSampleResultValid_CoordAsVec2AndInt = function(level, sampler, prec, coord, coordZ, result) {
        /** @type {Array<number>} */ var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getWidth(), coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */ var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getHeight(), coord[1], prec.coordBits[1], prec.uvwBits[1]);

        // Integer coordinate bounds for (x0,y0) - without wrap mode
        /** @type {number} */ var minI = Math.floor(uBounds[0] - 0.5);
        /** @type {number} */ var maxI = Math.floor(uBounds[1] - 0.5);
        /** @type {number} */ var minJ = Math.floor(vBounds[0] - 0.5);
        /** @type {number} */ var maxJ = Math.floor(vBounds[1] - 0.5);

        /** @type {number} */ var w = level.getWidth();
        /** @type {number} */ var h = level.getHeight();

        /** @type {tcuTexture.TextureChannelClass} */
        var texClass = tcuTexture.getTextureChannelClass(level.getFormat().type);

        /** @type {number} */
        var searchStep = (texClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForUnorm(prec) :
              (texClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForSnorm(prec) :
              0; // Step is computed for floating-point quads based on texel values.

        // \todo [2013-07-03 pyry] This could be optimized by first computing ranges based on wrap mode.

        for (var j = minJ; j <= maxJ; j++)
        for (var i = minI; i <= maxI; i++) {
            // Wrapped coordinates
            /** @type {number} */ var x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i, w);
            /** @type {number} */ var x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i + 1, w);
            /** @type {number} */ var y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j, h);
            /** @type {number} */ var y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j + 1, h);

            // Bounds for filtering factors
            /** @type {number} */ var minA = deMath.clamp((uBounds[0] - 0.5) - i, 0, 1);
            /** @type {number} */ var maxA = deMath.clamp((uBounds[1] - 0.5) - i, 0, 1);
            /** @type {number} */ var minB = deMath.clamp((vBounds[0] - 0.5) - j, 0, 1);
            /** @type {number} */ var maxB = deMath.clamp((vBounds[1] - 0.5) - j, 0, 1);

            /** @type {tcuTexLookupVerifier.ColorQuad} */
            var quad = tcuTexLookupVerifier.lookupQuad(level, sampler, x0, x1, y0, y1, coordZ);

            if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                searchStep = tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad);

            if (tcuTexLookupVerifier.isBilinearRangeValid(prec, quad, [minA, maxA], [minB, maxB], searchStep, result))
                return true;
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord vec3
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLinearSampleResultValid_CoordAsVec3 = function(level, sampler, prec, coord, result) {
        /** @type {Array<number>} */
        var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getWidth(), coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getHeight(), coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var wBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, level.getDepth(), coord[2], prec.coordBits[2], prec.uvwBits[2]);

        // Integer coordinate bounds for (x0,y0) - without wrap mode
        /** @type {number} */ var minI = Math.floor(uBounds[0] - 0.5);
        /** @type {number} */ var maxI = Math.floor(uBounds[1] - 0.5);
        /** @type {number} */ var minJ = Math.floor(vBounds[0] - 0.5);
        /** @type {number} */ var maxJ = Math.floor(vBounds[1] - 0.5);
        /** @type {number} */ var minK = Math.floor(wBounds[0] - 0.5);
        /** @type {number} */ var maxK = Math.floor(wBounds[1] - 0.5);

        /** @type {number} */ var w = level.getWidth();
        /** @type {number} */ var h = level.getHeight();
        /** @type {number} */ var d = level.getDepth();

        /** @type {tcuTexture.TextureChannelClass} */
        var texClass = tcuTexture.getTextureChannelClass(level.getFormat().type);
        /** @type {number} */
        var searchStep = (texClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForUnorm(prec) :
                         (texClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForSnorm(prec) :
                         0; // Step is computed for floating-point quads based on texel values.

        // \todo [2013-07-03 pyry] This could be optimized by first computing ranges based on wrap mode.

        for (var k = minK; k <= maxK; k++) {
            for (var j = minJ; j <= maxJ; j++) {
                for (var i = minI; i <= maxI; i++) {
                    // Wrapped coordinates
                    /** @type {number} */ var x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i, w);
                    /** @type {number} */ var x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i + 1, w);
                    /** @type {number} */ var y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j, h);
                    /** @type {number} */ var y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j + 1, h);
                    /** @type {number} */ var z0 = tcuTexVerifierUtil.wrap(sampler.wrapR, k, d);
                    /** @type {number} */ var z1 = tcuTexVerifierUtil.wrap(sampler.wrapR, k + 1, d);

                    // Bounds for filtering factors
                    /** @type {number} */ var minA = deMath.clamp((uBounds[0] - 0.5) - i, 0, 1);
                    /** @type {number} */ var maxA = deMath.clamp((uBounds[1] - 0.5) - i, 0, 1);
                    /** @type {number} */ var minB = deMath.clamp((vBounds[0] - 0.5) - j, 0, 1);
                    /** @type {number} */ var maxB = deMath.clamp((vBounds[1] - 0.5) - j, 0, 1);
                    /** @type {number} */ var minC = deMath.clamp((wBounds[0] - 0.5) - k, 0, 1);
                    /** @type {number} */ var maxC = deMath.clamp((wBounds[1] - 0.5) - k, 0, 1);

                    /** @type {tcuTexLookupVerifier.ColorQuad} */
                    var quad0 = tcuTexLookupVerifier.lookupQuad(level, sampler, x0, x1, y0, y1, z0);
                    /** @type {tcuTexLookupVerifier.ColorQuad} */
                    var quad1 = tcuTexLookupVerifier.lookupQuad(level, sampler, x0, x1, y0, y1, z1);

                    if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                        searchStep = Math.min(tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad0), tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad1));

                    if (tcuTexLookupVerifier.isTrilinearRangeValid(prec, quad0, quad1, [minA, maxA], [minB, maxB], [minC, maxC], searchStep, result))
                        return true;
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {number} coord
    * @param {number} coordY
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isNearestMipmapLinearSampleResultValid_CoordXYAsNumber = function(level0, level1, sampler, prec, coord, coordY, fBounds, result) {
        /** @type {number} */ var w0 = level0.getWidth();
        /** @type {number} */ var w1 = level1.getWidth();

        /** @type {Array<number>} */
        var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w0, coord, prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w1, coord, prec.coordBits[0], prec.uvwBits[0]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI0 = Math.floor(uBounds0[0]);
        /** @type {number} */ var maxI0 = Math.floor(uBounds0[1]);
        /** @type {number} */ var minI1 = Math.floor(uBounds1[0]);
        /** @type {number} */ var maxI1 = Math.floor(uBounds1[1]);

        for (var i0 = minI0; i0 <= maxI0; i0++) {
            for (var i1 = minI1; i1 <= maxI1; i1++) {
                /** @type {Array<number>} */
                var c0 = tcuTexLookupVerifier.lookupFloat(level0, sampler, tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0), coordY, 0);
                /** @type {Array<number>} */
                var c1 = tcuTexLookupVerifier.lookupFloat(level1, sampler, tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1), coordY, 0);

                if (tcuTexLookupVerifier.isLinearRangeValid(prec, c0, c1, fBounds, result))
                    return true;
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {number} coordZ
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isNearestMipmapLinearSampleResultValid_CoordAsVec2AndInt = function(level0, level1, sampler, prec, coord, coordZ, fBounds, result) {
        /** @type {number} */ var w0 = level0.getWidth();
        /** @type {number} */ var w1 = level1.getWidth();
        /** @type {number} */ var h0 = level0.getHeight();
        /** @type {number} */ var h1 = level1.getHeight();

        /** @type {Array<number>} */
        var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w0, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w1, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h0, coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h1, coord[1], prec.coordBits[1], prec.uvwBits[1]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI0 = Math.floor(uBounds0[0]);
        /** @type {number} */ var maxI0 = Math.floor(uBounds0[1]);
        /** @type {number} */ var minI1 = Math.floor(uBounds1[0]);
        /** @type {number} */ var maxI1 = Math.floor(uBounds1[1]);
        /** @type {number} */ var minJ0 = Math.floor(vBounds0[0]);
        /** @type {number} */ var maxJ0 = Math.floor(vBounds0[1]);
        /** @type {number} */ var minJ1 = Math.floor(vBounds1[0]);
        /** @type {number} */ var maxJ1 = Math.floor(vBounds1[1]);

        for (var j0 = minJ0; j0 <= maxJ0; j0++) {
            for (var i0 = minI0; i0 <= maxI0; i0++) {
                for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                    for (var i1 = minI1; i1 <= maxI1; i1++) {
                        /** @type {Array<number>} */ var c0 = tcuTexLookupVerifier.lookupFloat(level0, sampler, tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0), tcuTexVerifierUtil.wrap(sampler.wrapT, j0, h0), coordZ);
                        /** @type {Array<number>} */ var c1 = tcuTexLookupVerifier.lookupFloat(level1, sampler, tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1), tcuTexVerifierUtil.wrap(sampler.wrapT, j1, h1), coordZ);

                        if (tcuTexLookupVerifier.isLinearRangeValid(prec, c0, c1, fBounds, result))
                            return true;
                    }
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isNearestMipmapLinearSampleResultValid_CoordAsVec3 = function(level0, level1, sampler, prec, coord, fBounds, result) {
        /** @type {number} */ var w0 = level0.getWidth();
        /** @type {number} */ var w1 = level1.getWidth();
        /** @type {number} */ var h0 = level0.getHeight();
        /** @type {number} */ var h1 = level1.getHeight();
        /** @type {number} */ var d0 = level0.getDepth();
        /** @type {number} */ var d1 = level1.getDepth();

        /** @type {Array<number>} */
        var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w0, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w1, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h0, coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h1, coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var wBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, d0, coord[2], prec.coordBits[2], prec.uvwBits[2]);
        /** @type {Array<number>} */
        var wBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, d1, coord[2], prec.coordBits[2], prec.uvwBits[2]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI0 = Math.floor(uBounds0[0]);
        /** @type {number} */ var maxI0 = Math.floor(uBounds0[1]);
        /** @type {number} */ var minI1 = Math.floor(uBounds1[0]);
        /** @type {number} */ var maxI1 = Math.floor(uBounds1[1]);
        /** @type {number} */ var minJ0 = Math.floor(vBounds0[0]);
        /** @type {number} */ var maxJ0 = Math.floor(vBounds0[1]);
        /** @type {number} */ var minJ1 = Math.floor(vBounds1[0]);
        /** @type {number} */ var maxJ1 = Math.floor(vBounds1[1]);
        /** @type {number} */ var minK0 = Math.floor(wBounds0[0]);
        /** @type {number} */ var maxK0 = Math.floor(wBounds0[1]);
        /** @type {number} */ var minK1 = Math.floor(wBounds1[0]);
        /** @type {number} */ var maxK1 = Math.floor(wBounds1[1]);

        for (var k0 = minK0; k0 <= maxK0; k0++) {
            for (var j0 = minJ0; j0 <= maxJ0; j0++) {
                for (var i0 = minI0; i0 <= maxI0; i0++) {
                    for (var k1 = minK1; k1 <= maxK1; k1++) {
                        for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                            for (var i1 = minI1; i1 <= maxI1; i1++) {
                                /** @type {Array<number>} */ var c0 = tcuTexLookupVerifier.lookupFloat(level0, sampler, tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0), tcuTexVerifierUtil.wrap(sampler.wrapT, j0, h0), tcuTexVerifierUtil.wrap(sampler.wrapR, k0, d0));
                                /** @type {Array<number>} */ var c1 = tcuTexLookupVerifier.lookupFloat(level1, sampler, tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1), tcuTexVerifierUtil.wrap(sampler.wrapT, j1, h1), tcuTexVerifierUtil.wrap(sampler.wrapR, k1, d1));

                                if (tcuTexLookupVerifier.isLinearRangeValid(prec, c0, c1, fBounds, result))
                                    return true;
                            }
                        }
                    }
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {number} coordZ
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLinearMipmapLinearSampleResultValid_CoordAsVec2AndInt = function(level0, level1, sampler, prec, coord, coordZ, fBounds, result) {
        // \todo [2013-07-04 pyry] This is strictly not correct as coordinates between levels should be dependent.
        //                           Right now this allows pairing any two valid bilinear quads.

        /** @type {number} */ var w0 = level0.getWidth();
        /** @type {number} */ var w1 = level1.getWidth();
        /** @type {number} */ var h0 = level0.getHeight();
        /** @type {number} */ var h1 = level1.getHeight();

        /** @type {Array<number>} */
        var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w0, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w1, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h0, coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h1, coord[1], prec.coordBits[1], prec.uvwBits[1]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI0 = Math.floor(uBounds0[0] - 0.5);
        /** @type {number} */ var maxI0 = Math.floor(uBounds0[1] - 0.5);
        /** @type {number} */ var minI1 = Math.floor(uBounds1[0] - 0.5);
        /** @type {number} */ var maxI1 = Math.floor(uBounds1[1] - 0.5);
        /** @type {number} */ var minJ0 = Math.floor(vBounds0[0] - 0.5);
        /** @type {number} */ var maxJ0 = Math.floor(vBounds0[1] - 0.5);
        /** @type {number} */ var minJ1 = Math.floor(vBounds1[0] - 0.5);
        /** @type {number} */ var maxJ1 = Math.floor(vBounds1[1] - 0.5);

        /** @type {tcuTexture.TextureChannelClass} */
        var texClass = tcuTexture.getTextureChannelClass(level0.getFormat().type);
        /** @type {number} */ var cSearchStep = (texClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForUnorm(prec) :
                                                (texClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForSnorm(prec) :
                                                0; // Step is computed for floating-point quads based on texel values.

        /** @type {number} */ var x0;
        /** @type {number} */ var x1;
        /** @type {number} */ var y0;
        /** @type {number} */ var y1;

        for (var j0 = minJ0; j0 <= maxJ0; j0++) {
            for (var i0 = minI0; i0 <= maxI0; i0++) {
                /** @type {number} */ var searchStep0;

                x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0);
                x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0 + 1, w0);
                y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0, h0);
                y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0 + 1, h0);

                /** @type {tcuTexLookupVerifier.ColorQuad} */
                var quad0 = tcuTexLookupVerifier.lookupQuad(level0, sampler, x0, x1, y0, y1, coordZ);

                if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                    searchStep0 = tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad0);
                else
                    searchStep0 = cSearchStep;

                /** @type {number} */ var minA0 = deMath.clamp((uBounds0[0] - 0.5) - i0, 0, 1);
                /** @type {number} */ var maxA0 = deMath.clamp((uBounds0[1] - 0.5) - i0, 0, 1);
                /** @type {number} */ var minB0 = deMath.clamp((vBounds0[0] - 0.5) - j0, 0, 1);
                /** @type {number} */ var maxB0 = deMath.clamp((vBounds0[1] - 0.5) - j0, 0, 1);

                for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                    for (var i1 = minI1; i1 <= maxI1; i1++) {
                        /** @type {number} */ var searchStep1;

                        x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1);
                        x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1 + 1, w1);
                        y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1, h1);
                        y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1 + 1, h1);

                        /** @type {tcuTexLookupVerifier.ColorQuad} */
                        var quad1 = tcuTexLookupVerifier.lookupQuad(level1, sampler, x0, x1, y0, y1, coordZ);

                        if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                            searchStep1 = tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad1);
                        else
                            searchStep1 = cSearchStep;

                        /** @type {number} */ var minA1 = deMath.clamp((uBounds1[0] - 0.5) - i1, 0, 1);
                        /** @type {number} */ var maxA1 = deMath.clamp((uBounds1[1] - 0.5) - i1, 0, 1);
                        /** @type {number} */ var minB1 = deMath.clamp((vBounds1[0] - 0.5) - j1, 0, 1);
                        /** @type {number} */ var maxB1 = deMath.clamp((vBounds1[1] - 0.5) - j1, 0, 1);

                        if (tcuTexLookupVerifier.is2DTrilinearFilterResultValid(prec, quad0, quad1, [minA0, maxA0], [minB0, maxB0], [minA1, maxA1], [minB1, maxB1],
                                                           fBounds, Math.min(searchStep0, searchStep1), result))
                            return true;
                    }
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLinearMipmapLinearSampleResultValid_CoordAsVec3 = function(level0, level1, sampler, prec, coord, fBounds, result) {
        // \todo [2013-07-04 pyry] This is strictly not correct as coordinates between levels should be dependent.
        //                           Right now this allows pairing any two valid bilinear quads.

        /** @type {number} */ var w0 = level0.getWidth();
        /** @type {number} */ var w1 = level1.getWidth();
        /** @type {number} */ var h0 = level0.getHeight();
        /** @type {number} */ var h1 = level1.getHeight();
        /** @type {number} */ var d0 = level0.getDepth();
        /** @type {number} */ var d1 = level1.getDepth();

        /** @type {Array<number>} */
        var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w0, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, w1, coord[0], prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */
        var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h0, coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, h1, coord[1], prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */
        var wBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, d0, coord[2], prec.coordBits[2], prec.uvwBits[2]);
        /** @type {Array<number>} */
        var wBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(
            sampler.normalizedCoords, d1, coord[2], prec.coordBits[2], prec.uvwBits[2]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI0 = Math.floor(uBounds0[0] - 0.5);
        /** @type {number} */ var maxI0 = Math.floor(uBounds0[1] - 0.5);
        /** @type {number} */ var minI1 = Math.floor(uBounds1[0] - 0.5);
        /** @type {number} */ var maxI1 = Math.floor(uBounds1[1] - 0.5);
        /** @type {number} */ var minJ0 = Math.floor(vBounds0[0] - 0.5);
        /** @type {number} */ var maxJ0 = Math.floor(vBounds0[1] - 0.5);
        /** @type {number} */ var minJ1 = Math.floor(vBounds1[0] - 0.5);
        /** @type {number} */ var maxJ1 = Math.floor(vBounds1[1] - 0.5);
        /** @type {number} */ var minK0 = Math.floor(wBounds0[0] - 0.5);
        /** @type {number} */ var maxK0 = Math.floor(wBounds0[1] - 0.5);
        /** @type {number} */ var minK1 = Math.floor(wBounds1[0] - 0.5);
        /** @type {number} */ var maxK1 = Math.floor(wBounds1[1] - 0.5);

        /** @type {tcuTexture.TextureChannelClass} */
        var texClass = tcuTexture.getTextureChannelClass(level0.getFormat().type);
        /** @type {number} */ var cSearchStep = texClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT ? tcuTexLookupVerifier.computeBilinearSearchStepForUnorm(prec) :
                                                texClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT ? tcuTexLookupVerifier.computeBilinearSearchStepForSnorm(prec) :
                                                0; // Step is computed for floating-point quads based on texel values.

        /** @type {number} */ var x0;
        /** @type {number} */ var x1;
        /** @type {number} */ var y0;
        /** @type {number} */ var y1;
        /** @type {number} */ var z0;
        /** @type {number} */ var z1;

        for (var k0 = minK0; k0 <= maxK0; k0++) {
            for (var j0 = minJ0; j0 <= maxJ0; j0++) {
                for (var i0 = minI0; i0 <= maxI0; i0++) {
                    /** @type {number} */ var searchStep0;

                    x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0);
                    x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0 + 1, w0);
                    y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0, h0);
                    y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0 + 1, h0);
                    z0 = tcuTexVerifierUtil.wrap(sampler.wrapR, k0, d0);
                    z1 = tcuTexVerifierUtil.wrap(sampler.wrapR, k0 + 1, d0);
                    /** @type {tcuTexLookupVerifier.ColorQuad} */
                    var quad00 = tcuTexLookupVerifier.lookupQuad(level0, sampler, x0, x1, y0, y1, z0);
                    /** @type {tcuTexLookupVerifier.ColorQuad} */
                    var quad01 = tcuTexLookupVerifier.lookupQuad(level0, sampler, x0, x1, y0, y1, z1);

                    if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                        searchStep0 = Math.min(tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad00), tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad01));
                    else
                        searchStep0 = cSearchStep;

                    /** @type {number} */ var minA0 = deMath.clamp((uBounds0[0] - 0.5) - i0, 0, 1);
                    /** @type {number} */ var maxA0 = deMath.clamp((uBounds0[1] - 0.5) - i0, 0, 1);
                    /** @type {number} */ var minB0 = deMath.clamp((vBounds0[0] - 0.5) - j0, 0, 1);
                    /** @type {number} */ var maxB0 = deMath.clamp((vBounds0[1] - 0.5) - j0, 0, 1);
                    /** @type {number} */ var minC0 = deMath.clamp((wBounds0[0] - 0.5) - k0, 0, 1);
                    /** @type {number} */ var maxC0 = deMath.clamp((wBounds0[1] - 0.5) - k0, 0, 1);

                    for (var k1 = minK1; k1 <= maxK1; k1++) {
                        for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                            for (var i1 = minI1; i1 <= maxI1; i1++) {

                                /** @type {number} */ var searchStep1;

                                x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1);
                                x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1 + 1, w1);
                                y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1, h1);
                                y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1 + 1, h1);
                                z0 = tcuTexVerifierUtil.wrap(sampler.wrapR, k1, d1);
                                z1 = tcuTexVerifierUtil.wrap(sampler.wrapR, k1 + 1, d1);
                                /** @type {tcuTexLookupVerifier.ColorQuad} */
                                var quad10 = tcuTexLookupVerifier.lookupQuad(level1, sampler, x0, x1, y0, y1, z0);
                                /** @type {tcuTexLookupVerifier.ColorQuad} */
                                var quad11 = tcuTexLookupVerifier.lookupQuad(level1, sampler, x0, x1, y0, y1, z1);

                                if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                                    searchStep1 = Math.min(tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad10), tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad11));
                                else
                                    searchStep1 = cSearchStep;

                                /** @type {number} */ var minA1 = deMath.clamp((uBounds1[0] - 0.5) - i1, 0, 1);
                                /** @type {number} */ var maxA1 = deMath.clamp((uBounds1[1] - 0.5) - i1, 0, 1);
                                /** @type {number} */ var minB1 = deMath.clamp((vBounds1[0] - 0.5) - j1, 0, 1);
                                /** @type {number} */ var maxB1 = deMath.clamp((vBounds1[1] - 0.5) - j1, 0, 1);
                                /** @type {number} */ var minC1 = deMath.clamp((wBounds1[0] - 0.5) - k1, 0, 1);
                                /** @type {number} */ var maxC1 = deMath.clamp((wBounds1[1] - 0.5) - k1, 0, 1);

                                if (tcuTexLookupVerifier.is3DTrilinearFilterResultValid(
                                    prec, quad00, quad01, quad10, quad11,
                                    [minA0, maxA0], [minB0, maxB0], [minC0, maxC0],
                                    [minA1, maxA1], [minB1, maxB1], [minC1, maxC1],
                                    fBounds, Math.min(searchStep0, searchStep1), result))
                                    return true;
                            }
                        }
                    }
                }
            }
        }

        return false;
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexture.FilterMode} filterMode
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {number} coordX
    * @param {number} coordY
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLevelSampleResultValid_CoordXYAsNumber = function(level, sampler, filterMode, prec, coordX, coordY, result) {
        if (filterMode == tcuTexture.FilterMode.LINEAR)
            return tcuTexLookupVerifier.isLinearSampleResultValid_CoordXYAsNumber(level, sampler, prec, coordX, coordY, result);
        else
            return tcuTexLookupVerifier.isNearestSampleResultValid_CoordXYAsNumber(level, sampler, prec, coordX, coordY, result);
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexture.FilterMode} filterMode
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {number} coordZ
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt = function(level, sampler, filterMode, prec, coord, coordZ, result) {
        if (filterMode == tcuTexture.FilterMode.LINEAR)
            return tcuTexLookupVerifier.isLinearSampleResultValid_CoordAsVec2AndInt(level, sampler, prec, coord, coordZ, result);
        else
            return tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec2AndInt(level, sampler, prec, coord, coordZ, result);
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexture.FilterMode} filterMode
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec3 = function(level, sampler, filterMode, prec, coord, result) {
        if (filterMode == tcuTexture.FilterMode.LINEAR)
            return tcuTexLookupVerifier.isLinearSampleResultValid_CoordAsVec3(level, sampler, prec, coord, result);
        else
            return tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec3(level, sampler, prec, coord, result);
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexture.FilterMode} levelFilter
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {number} coordZ
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isMipmapLinearSampleResultValid_CoordAsVec2AndInt = function(level0, level1, sampler, levelFilter, prec, coord, coordZ, fBounds, result) {
        if (levelFilter == tcuTexture.FilterMode.LINEAR)
            return tcuTexLookupVerifier.isLinearMipmapLinearSampleResultValid_CoordAsVec2AndInt(level0, level1, sampler, prec, coord, coordZ, fBounds, result);
        else
            return tcuTexLookupVerifier.isNearestMipmapLinearSampleResultValid_CoordAsVec2AndInt(level0, level1, sampler, prec, coord, coordZ, fBounds, result);
    };

    /**
    * @param {tcuTexture.ConstPixelBufferAccess} level0
    * @param {tcuTexture.ConstPixelBufferAccess} level1
    * @param {tcuTexture.Sampler} sampler
    * @param {tcuTexture.FilterMode} levelFilter
    * @param {tcuTexLookupVerifier.LookupPrecision} prec
    * @param {Array<number>} coord
    * @param {Array<number>} fBounds
    * @param {Array<number>} result
    * @return {boolean}
    */
    tcuTexLookupVerifier.isMipmapLinearSampleResultValid_CoordAsVec3 = function(level0, level1, sampler, levelFilter, prec, coord, fBounds, result) {
        if (levelFilter == tcuTexture.FilterMode.LINEAR)
            return tcuTexLookupVerifier.isLinearMipmapLinearSampleResultValid_CoordAsVec3(level0, level1, sampler, prec, coord, fBounds, result);
        else
            return tcuTexLookupVerifier.isNearestMipmapLinearSampleResultValid_CoordAsVec3(level0, level1, sampler, prec, coord, fBounds, result);
    };

    /**
     * @param {tcuTexture.Texture2DView} texture
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {Array<number>} lodBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLookupResultValid_Texture2DView = function(texture, sampler, prec, coord, lodBounds, result) {
        /** @type {number} */ var minLod = lodBounds[0];
        /** @type {number} */ var maxLod = lodBounds[1];
        /** @type {boolean} */ var canBeMagnified = minLod <= sampler.lodThreshold;
        /** @type {boolean} */ var canBeMinified = maxLod > sampler.lodThreshold;

        assertMsgOptions(tcuTexLookupVerifier.isSamplerSupported(sampler), 'Sampler not supported.', false, true);

        /** @type {number} */ var minLevel;
        /** @type {number} */ var maxLevel;

        if (canBeMagnified)
            if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(texture.getLevel(0), sampler, sampler.magFilter, prec, coord, 0, result))
                return true;

        if (canBeMinified) {
            /** @type {boolean} */ var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
            /** @type {boolean} */ var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
            /** @type {number} */ var minTexLevel = 0;
            /** @type {number} */ var maxTexLevel = texture.getNumLevels() - 1;

            assertMsgOptions(minTexLevel <= maxTexLevel, 'minTexLevel > maxTexLevel', false, true);

            if (isLinearMipmap && minTexLevel < maxTexLevel) {
                minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
                maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

                assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    /** @type {number} */ var minF = deMath.clamp(minLod - level, 0, 1);
                    /** @type {number} */ var maxF = deMath.clamp(maxLod - level, 0, 1);

                    if (tcuTexLookupVerifier.isMipmapLinearSampleResultValid_CoordAsVec2AndInt(texture.getLevel(level), texture.getLevel(level + 1), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coord, 0, [minF, maxF], result))
                        return true;
                }
            } else if (isNearestMipmap) {
                // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
                //         decision to allow floor(lod + 0.5) as well.
                minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
                maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

                assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(texture.getLevel(level), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coord, 0, result))
                        return true;
                }
            } else {
                if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(texture.getLevel(0), sampler, sampler.minFilter, prec, coord, 0, result))
                    return true;
            }
        }

        return false;
    };

    /**
     * @param {tcuTexture.TextureCubeView} texture
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {Array<number>} lodBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLookupResultValid_TextureCubeView = function(texture, sampler, prec, coord, lodBounds, result) {
        /** @type {number} */ var numPossibleFaces = 0;

        assertMsgOptions(tcuTexLookupVerifier.isSamplerSupported(sampler), 'Sampler not supported.', false, true);

        /** @type {Array<tcuTexture.CubeFace>} */ var possibleFaces = tcuTexVerifierUtil.getPossibleCubeFaces(coord, prec.coordBits);

        /** @type {number} */ var minLevel;
        /** @type {number} */ var maxLevel;

        if (!possibleFaces)
            return true; // Result is undefined.

        for (var tryFaceNdx = 0; tryFaceNdx < possibleFaces.length; tryFaceNdx++) {
            /** @type {tcuTexture.CubeFaceCoords} */
            var faceCoords = new tcuTexture.CubeFaceCoords(possibleFaces[tryFaceNdx], tcuTexture.projectToFace(possibleFaces[tryFaceNdx], coord));
            /** @type {number} */ var minLod = lodBounds[0];
            /** @type {number} */ var maxLod = lodBounds[1];
            /** @type {boolean} */ var canBeMagnified = minLod <= sampler.lodThreshold;
            /** @type {boolean} */ var canBeMinified = maxLod > sampler.lodThreshold;

            /** @type {Array<tcuTexture.ConstPixelBufferAccess>} */ var faces = [];
            if (canBeMagnified) {
                tcuTexLookupVerifier.getCubeLevelFaces(texture, 0, faces);

                if (tcuTexLookupVerifier.isCubeLevelSampleResultValid(faces, sampler, sampler.magFilter, prec, faceCoords, result))
                    return true;
            }

            if (canBeMinified) {
                /** @type {boolean} */ var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
                /** @type {boolean} */ var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
                /** @type {number} */ var minTexLevel = 0;
                /** @type {number} */ var maxTexLevel = texture.getNumLevels() - 1;

                assertMsgOptions(minTexLevel <= maxTexLevel, 'minTexLevel > maxTexLevel', false, true);

                if (isLinearMipmap && minTexLevel < maxTexLevel) {
                    minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
                    maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

                    assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                    for (var levelNdx = minLevel; levelNdx <= maxLevel; levelNdx++) {
                        /** @type {number} */ var minF = deMath.clamp(minLod - levelNdx, 0, 1);
                        /** @type {number} */ var maxF = deMath.clamp(maxLod - levelNdx, 0, 1);

                        /** @type {Array<tcuTexture.ConstPixelBufferAccess>} */ var faces0 = [];
                        /** @type {Array<tcuTexture.ConstPixelBufferAccess>} */ var faces1 = [];

                        tcuTexLookupVerifier.getCubeLevelFaces(texture, levelNdx, faces0);
                        tcuTexLookupVerifier.getCubeLevelFaces(texture, levelNdx + 1, faces1);

                        if (tcuTexLookupVerifier.isCubeMipmapLinearSampleResultValid(faces0, faces1, sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, faceCoords, [minF, maxF], result))
                            return true;
                    }
                } else if (isNearestMipmap) {
                    // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
                    //         decision to allow floor(lod + 0.5) as well.
                    minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
                    maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

                    assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                    for (var levelNdx = minLevel; levelNdx <= maxLevel; levelNdx++) {
                        tcuTexLookupVerifier.getCubeLevelFaces(texture, levelNdx, faces);

                        if (tcuTexLookupVerifier.isCubeLevelSampleResultValid(faces, sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, faceCoords, result))
                            return true;
                    }
                } else {
                    tcuTexLookupVerifier.getCubeLevelFaces(texture, 0, faces);

                    if (tcuTexLookupVerifier.isCubeLevelSampleResultValid(faces, sampler, sampler.minFilter, prec, faceCoords, result))
                        return true;
                }
            }
        }

        return false;
    };

    /**
     * @param {tcuTexture.Texture2DArrayView} texture
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {Array<number>} lodBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLookupResultValid_Texture2DArrayView = function(texture, sampler, prec, coord, lodBounds, result) {
        /** @type {Array<number>} */ var layerRange = tcuTexLookupVerifier.computeLayerRange(texture.getNumLayers(), prec.coordBits[2], coord[2]);
        /** @type {Array<number>} */ var coordXY = deMath.swizzle(coord, [0, 1]);
        /** @type {number} */ var minLod = lodBounds[0];
        /** @type {number} */ var maxLod = lodBounds[1];
        /** @type {boolean} */ var canBeMagnified = minLod <= sampler.lodThreshold;
        /** @type {boolean} */ var canBeMinified = maxLod > sampler.lodThreshold;

        assertMsgOptions(tcuTexLookupVerifier.isSamplerSupported(sampler), 'Sampler not supported.', false, true);
        /** @type {number} */ var minLevel;
        /** @type {number} */ var maxLevel;

        for (var layer = layerRange[0]; layer <= layerRange[1]; layer++) {
            if (canBeMagnified) {
                if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(texture.getLevel(0), sampler, sampler.magFilter, prec, coordXY, layer, result))
                    return true;
            }

            if (canBeMinified) {
                /** @type {boolean} */ var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
                /** @type {boolean} */ var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
                /** @type {number} */ var minTexLevel = 0;
                /** @type {number} */ var maxTexLevel = texture.getNumLevels() - 1;

                assertMsgOptions(minTexLevel <= maxTexLevel, 'minTexLevel > maxTexLevel', false, true);

                if (isLinearMipmap && minTexLevel < maxTexLevel) {
                    minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
                    maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

                    assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                    for (var level = minLevel; level <= maxLevel; level++) {
                        /** @type {number} */ var minF = deMath.clamp(minLod - level, 0, 1);
                        /** @type {number} */ var maxF = deMath.clamp(maxLod - level, 0, 1);

                        if (tcuTexLookupVerifier.isMipmapLinearSampleResultValid_CoordAsVec2AndInt(texture.getLevel(level), texture.getLevel(level + 1), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coordXY, layer, [minF, maxF], result))
                            return true;
                    }
                } else if (isNearestMipmap) {
                    // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
                    //         decision to allow floor(lod + 0.5) as well.
                    minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
                    maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

                    assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                    for (var level = minLevel; level <= maxLevel; level++) {
                        if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(texture.getLevel(level), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coordXY, layer, result))
                            return true;
                    }
                } else {
                    if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(texture.getLevel(0), sampler, sampler.minFilter, prec, coordXY, layer, result))
                        return true;
                }
            }
        }

        return false;
    };

    /**
     * @param {tcuTexture.Texture3DView} texture
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {Array<number>} lodBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLookupResultValid = function(texture, sampler, prec, coord, lodBounds, result) {
        /** @type {number} */ var minLod = lodBounds[0];
        /** @type {number} */ var maxLod = lodBounds[1];
        /** @type {boolean} */ var canBeMagnified = minLod <= sampler.lodThreshold;
        /** @type {boolean} */ var canBeMinified = maxLod > sampler.lodThreshold;

        assertMsgOptions(tcuTexLookupVerifier.isSamplerSupported(sampler), 'Sampler not supported.', false, true);

        /** @type {number} */ var minLevel;
        /** @type {number} */ var maxLevel;

        if (canBeMagnified)
            if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec3(texture.getLevel(0), sampler, sampler.magFilter, prec, coord, result))
                return true;

        if (canBeMinified) {
            /** @type {boolean} */ var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
            /** @type {boolean} */ var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
            /** @type {number} */ var minTexLevel = 0;
            /** @type {number} */ var maxTexLevel = texture.getNumLevels() - 1;

            assertMsgOptions(minTexLevel <= maxTexLevel, 'minTexLevel > maxTexLevel', false, true);

            if (isLinearMipmap && minTexLevel < maxTexLevel) {
                minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
                maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

                assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    /** @type {number} */ var minF = deMath.clamp(minLod - level, 0, 1);
                    /** @type {number} */ var maxF = deMath.clamp(maxLod - level, 0, 1);

                    if (tcuTexLookupVerifier.isMipmapLinearSampleResultValid_CoordAsVec3(texture.getLevel(level), texture.getLevel(level + 1), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coord, [minF, maxF], result))
                        return true;
                }
            } else if (isNearestMipmap) {
                // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
                //         decision to allow floor(lod + 0.5) as well.
                minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
                maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

                assertMsgOptions(minLevel <= maxLevel, 'minLevel > maxLevel', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec3(texture.getLevel(level), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coord, result))
                        return true;
                }
            } else {
                if (tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec3(texture.getLevel(0), sampler, sampler.minFilter, prec, coord, result))
                    return true;
            }
        }

        return false;
    };

    /**
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} faces (&faces)[CUBEFACE_LAST]
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexture.CubeFaceCoords} coords
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isSeamlessLinearSampleResultValid = function(faces, sampler, prec, coords, result) {
        /** @type {number} */ var size = faces[coords.face].getWidth();

        /** @type {Array<number>} */ var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size, coords.s, prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */ var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size, coords.t, prec.coordBits[1], prec.uvwBits[1]);

        // Integer coordinate bounds for (x0,y0) - without wrap mode
        /** @type {number} */ var minI = Math.floor(uBounds[0] - 0.5);
        /** @type {number} */ var maxI = Math.floor(uBounds[1] - 0.5);
        /** @type {number} */ var minJ = Math.floor(vBounds[0] - 0.5);
        /** @type {number} */ var maxJ = Math.floor(vBounds[1] - 0.5);

        /** @type {tcuTexture.TextureChannelClass} */ var texClass = tcuTexture.getTextureChannelClass(faces[coords.face].getFormat().type);
        /** @type {number} */ var searchStep = (texClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForUnorm(prec) :
                                               (texClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForSnorm(prec) :
                                               0; // Step is computed for floating-point quads based on texel values.

        for (var j = minJ; j <= maxJ; j++) {
            for (var i = minI; i <= maxI; i++) {
                /** @type {tcuTexture.CubeFaceCoords} */ var c00 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 0, j + 0]), size);
                /** @type {tcuTexture.CubeFaceCoords} */ var c10 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 1, j + 0]), size);
                /** @type {tcuTexture.CubeFaceCoords} */ var c01 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 0, j + 1]), size);
                /** @type {tcuTexture.CubeFaceCoords} */ var c11 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 1, j + 1]), size);

                // If any of samples is out of both edges, implementations can do pretty much anything according to spec.
                // \todo [2013-07-08 pyry] Test the special case where all corner pixels have exactly the same color.
                if (c00 == null || c01 == null || c10 == null || c11 == null ||
                    c00.face == null || c01.face == null || c10.face == null || c11.face == null)
                    return true;

                // Bounds for filtering factors
                /** @type {number} */ var minA = deMath.clamp((uBounds[0] - 0.5) - i, 0, 1);
                /** @type {number} */ var maxA = deMath.clamp((uBounds[1] - 0.5) - i, 0, 1);
                /** @type {number} */ var minB = deMath.clamp((vBounds[0] - 0.5) - j, 0, 1);
                /** @type {number} */ var maxB = deMath.clamp((vBounds[1] - 0.5) - j, 0, 1);

                /** @type {tcuTexLookupVerifier.ColorQuad} */
                var quad = new tcuTexLookupVerifier.ColorQuad([], [], [], []);
                quad.p00 = tcuTexLookupVerifier.lookupFloat(faces[c00.face], sampler, c00.s, c00.t, 0);
                quad.p10 = tcuTexLookupVerifier.lookupFloat(faces[c10.face], sampler, c10.s, c10.t, 0);
                quad.p01 = tcuTexLookupVerifier.lookupFloat(faces[c01.face], sampler, c01.s, c01.t, 0);
                quad.p11 = tcuTexLookupVerifier.lookupFloat(faces[c11.face], sampler, c11.s, c11.t, 0);

                if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                    searchStep = tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad);

                if (tcuTexLookupVerifier.isBilinearRangeValid(prec, quad, [minA, maxA], [minB, maxB], searchStep, result))
                    return true;
            }
        }

        return false;
    };

    /**
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} faces0 (&faces0)[CUBEFACE_LAST]
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} faces1 (&faces1)[CUBEFACE_LAST]
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexture.CubeFaceCoords} coords
     * @param {Array<number>} fBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isSeamplessLinearMipmapLinearSampleResultValid = function(faces0, faces1, sampler, prec, coords, fBounds, result) {
        // \todo [2013-07-04 pyry] This is strictly not correct as coordinates between levels should be dependent.
        //                           Right now this allows pairing any two valid bilinear quads.
        /** @type {number} */ var size0 = faces0[coords.face].getWidth();
        /** @type {number} */ var size1 = faces1[coords.face].getWidth();

        /** @type {Array<number>} */ var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size0, coords.s, prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */ var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size1, coords.s, prec.coordBits[0], prec.uvwBits[0]);
        /** @type {Array<number>} */ var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size0, coords.t, prec.coordBits[1], prec.uvwBits[1]);
        /** @type {Array<number>} */ var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size1, coords.t, prec.coordBits[1], prec.uvwBits[1]);

        // Integer coordinates - without wrap mode
        /** @type {number} */ var minI0 = Math.floor(uBounds0[0] - 0.5);
        /** @type {number} */ var maxI0 = Math.floor(uBounds0[1] - 0.5);
        /** @type {number} */ var minI1 = Math.floor(uBounds1[0] - 0.5);
        /** @type {number} */ var maxI1 = Math.floor(uBounds1[1] - 0.5);
        /** @type {number} */ var minJ0 = Math.floor(vBounds0[0] - 0.5);
        /** @type {number} */ var maxJ0 = Math.floor(vBounds0[1] - 0.5);
        /** @type {number} */ var minJ1 = Math.floor(vBounds1[0] - 0.5);
        /** @type {number} */ var maxJ1 = Math.floor(vBounds1[1] - 0.5);

        /** @type {tcuTexture.TextureChannelClass} */ var texClass = tcuTexture.getTextureChannelClass(faces0[coords.face].getFormat().type);
        /** @type {number} */ var cSearchStep = (texClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForUnorm(prec) :
                                                (texClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT) ? tcuTexLookupVerifier.computeBilinearSearchStepForSnorm(prec) :
                                                0; // Step is computed for floating-point quads based on texel values.

        /** @type {tcuTexture.CubeFaceCoords} */ var c00;
        /** @type {tcuTexture.CubeFaceCoords} */ var c10;
        /** @type {tcuTexture.CubeFaceCoords} */ var c01;
        /** @type {tcuTexture.CubeFaceCoords} */ var c11;

        for (var j0 = minJ0; j0 <= maxJ0; j0++) {
            for (var i0 = minI0; i0 <= maxI0; i0++) {
                /** @type {tcuTexLookupVerifier.ColorQuad} */
                var quad0 = new tcuTexLookupVerifier.ColorQuad([], [], [], []);
                /** @type {number} */ var searchStep0;

                c00 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 0, j0 + 0]), size0);
                c10 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 1, j0 + 0]), size0);
                c01 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 0, j0 + 1]), size0);
                c11 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 1, j0 + 1]), size0);

                // If any of samples is out of both edges, implementations can do pretty much anything according to spec.
                // \todo [2013-07-08 pyry] Test the special case where all corner pixels have exactly the same color.
                if (c00 == null || c01 == null || c10 == null || c11 == null ||
                    c00.face == null || c01.face == null || c10.face == null || c11.face == null)
                    return true;

                quad0.p00 = tcuTexLookupVerifier.lookupFloat(faces0[c00.face], sampler, c00.s, c00.t, 0);
                quad0.p10 = tcuTexLookupVerifier.lookupFloat(faces0[c10.face], sampler, c10.s, c10.t, 0);
                quad0.p01 = tcuTexLookupVerifier.lookupFloat(faces0[c01.face], sampler, c01.s, c01.t, 0);
                quad0.p11 = tcuTexLookupVerifier.lookupFloat(faces0[c11.face], sampler, c11.s, c11.t, 0);

                if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                    searchStep0 = tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad0);
                else
                    searchStep0 = cSearchStep;

                /** @type {number} */ var minA0 = deMath.clamp((uBounds0[0] - 0.5) - i0, 0, 1);
                /** @type {number} */ var maxA0 = deMath.clamp((uBounds0[1] - 0.5) - i0, 0, 1);
                /** @type {number} */ var minB0 = deMath.clamp((vBounds0[0] - 0.5) - j0, 0, 1);
                /** @type {number} */ var maxB0 = deMath.clamp((vBounds0[1] - 0.5) - j0, 0, 1);

                for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                    for (var i1 = minI1; i1 <= maxI1; i1++) {
                        /** @type {tcuTexLookupVerifier.ColorQuad} */
                        var quad1 = new tcuTexLookupVerifier.ColorQuad([], [], [], []);
                        /** @type {number} */ var searchStep1;

                        c00 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 0, j1 + 0]), size1);
                        c10 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 1, j1 + 0]), size1);
                        c01 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 0, j1 + 1]), size1);
                        c11 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 1, j1 + 1]), size1);

                        if (c00 == null || c01 == null || c10 == null || c11 == null ||
                            c00.face == null || c01.face == null || c10.face == null || c11.face == null)
                            return true;

                        quad1.p00 = tcuTexLookupVerifier.lookupFloat(faces1[c00.face], sampler, c00.s, c00.t, 0);
                        quad1.p10 = tcuTexLookupVerifier.lookupFloat(faces1[c10.face], sampler, c10.s, c10.t, 0);
                        quad1.p01 = tcuTexLookupVerifier.lookupFloat(faces1[c01.face], sampler, c01.s, c01.t, 0);
                        quad1.p11 = tcuTexLookupVerifier.lookupFloat(faces1[c11.face], sampler, c11.s, c11.t, 0);

                        if (texClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
                            searchStep1 = tcuTexLookupVerifier.computeBilinearSearchStepFromFloatQuad(prec, quad1);
                        else
                            searchStep1 = cSearchStep;

                        /** @type {number} */ var minA1 = deMath.clamp((uBounds1[0] - 0.5) - i1, 0, 1);
                        /** @type {number} */ var maxA1 = deMath.clamp((uBounds1[1] - 0.5) - i1, 0, 1);
                        /** @type {number} */ var minB1 = deMath.clamp((vBounds1[0] - 0.5) - j1, 0, 1);
                        /** @type {number} */ var maxB1 = deMath.clamp((vBounds1[1] - 0.5) - j1, 0, 1);

                        if (tcuTexLookupVerifier.is2DTrilinearFilterResultValid(prec, quad0, quad1, [minA0, maxA0], [minB0, maxB0], [minA1, maxA1], [minB1, maxB1],
                                                           fBounds, Math.min(searchStep0, searchStep1), result))
                            return true;
                    }
                }
            }
        }

        return false;
    };

    /**
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} level (&level)[CUBEFACE_LAST]
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexture.FilterMode} filterMode
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexture.CubeFaceCoords} coords
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isCubeLevelSampleResultValid = function(level, sampler, filterMode, prec, coords, result) {
        if (filterMode == tcuTexture.FilterMode.LINEAR) {
            if (sampler.seamlessCubeMap)
                return tcuTexLookupVerifier.isSeamlessLinearSampleResultValid(level, sampler, prec, coords, result);
            else
                return tcuTexLookupVerifier.isLinearSampleResultValid_CoordAsVec2AndInt(level[coords.face], sampler, prec, [coords.s, coords.t], 0, result);
        } else
            return tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec2AndInt(level[coords.face], sampler, prec, [coords.s, coords.t], 0, result);
    };

    /**
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} faces0 (&faces0)[CUBEFACE_LAST]
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} faces1 (&faces1)[CUBEFACE_LAST]
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexture.FilterMode} levelFilter
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {tcuTexture.CubeFaceCoords} coords
     * @param {Array<number>} fBounds
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isCubeMipmapLinearSampleResultValid = function(faces0, faces1, sampler, levelFilter, prec, coords, fBounds, result) {
        if (levelFilter == tcuTexture.FilterMode.LINEAR) {
            if (sampler.seamlessCubeMap)
                return tcuTexLookupVerifier.isSeamplessLinearMipmapLinearSampleResultValid(faces0, faces1, sampler, prec, coords, fBounds, result);
            else
                return tcuTexLookupVerifier.isLinearMipmapLinearSampleResultValid_CoordAsVec2AndInt(faces0[coords.face], faces1[coords.face], sampler, prec, [coords.s, coords.t], 0, fBounds, result);
        } else
            return tcuTexLookupVerifier.isNearestMipmapLinearSampleResultValid_CoordAsVec2AndInt(faces0[coords.face], faces1[coords.face], sampler, prec, [coords.s, coords.t], 0, fBounds, result);
    };

    /**
     * @param {tcuTexture.TextureCubeView} texture
     * @param {number} levelNdx
     * @param {Array<tcuTexture.ConstPixelBufferAccess>} out (&out)[CUBEFACE_LAST]
     */
    tcuTexLookupVerifier.getCubeLevelFaces = function(texture, levelNdx, out) {
        for (var faceNdx = 0; faceNdx < 6; faceNdx++)
            out[faceNdx] = texture.getLevelFace(levelNdx, /** @type {tcuTexture.CubeFace} */ (faceNdx));
    };

    /**
     * @param {number} numLayers
     * @param {number} numCoordBits
     * @param {number} layerCoord
     * @return {Array<number>}
     */
    tcuTexLookupVerifier.computeLayerRange = function(numLayers, numCoordBits, layerCoord) {
        /** @type {number} */ var err = tcuTexVerifierUtil.computeFloatingPointError(layerCoord, numCoordBits);
        /** @type {number} */ var minL = Math.floor(layerCoord - err + 0.5); // Round down
        /** @type {number} */ var maxL = Math.ceil(layerCoord + err + 0.5) - 1; // Round up

        assertMsgOptions(minL <= maxL, 'minL > maxL', false, true);

        return [deMath.clamp(minL, 0, numLayers - 1), deMath.clamp(maxL, 0, numLayers - 1)];
    };

    /**
    * @param {Array<number>} bits
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.computeFixedPointThreshold = function(bits) {
        return tcuTexVerifierUtil.computeFixedPointError_Vector(bits);
    };

    /**
    * @param {Array<number>} bits
    * @param {Array<number>} value
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.computeFloatingPointThreshold = function(bits, value) {
        return tcuTexVerifierUtil.computeFloatingPointError_Vector(value, bits);
    };

    /**
    * @param {number} dudx
    * @param {number} dvdx
    * @param {number} dwdx
    * @param {number} dudy
    * @param {number} dvdy
    * @param {number} dwdy
    * @param {tcuTexLookupVerifier.LodPrecision} prec
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.computeLodBoundsFromDerivates = function(dudx, dvdx, dwdx, dudy, dvdy, dwdy, prec) {
        /** @type {number} */ var mu = Math.max(Math.abs(dudx), Math.abs(dudy));
        /** @type {number} */ var mv = Math.max(Math.abs(dvdx), Math.abs(dvdy));
        /** @type {number} */ var mw = Math.max(Math.abs(dwdx), Math.abs(dwdy));
        /** @type {number} */ var minDBound = Math.max(Math.max(mu, mv), mw);
        /** @type {number} */ var maxDBound = mu + mv + mw;
        /** @type {number} */ var minDErr = tcuTexVerifierUtil.computeFloatingPointError(minDBound, prec.derivateBits);
        /** @type {number} */ var maxDErr = tcuTexVerifierUtil.computeFloatingPointError(maxDBound, prec.derivateBits);
        /** @type {number} */ var minLod = Math.log2(minDBound - minDErr);
        /** @type {number} */ var maxLod = Math.log2(maxDBound + maxDErr);
        /** @type {number} */ var lodErr = tcuTexVerifierUtil.computeFixedPointError(prec.lodBits);

        assertMsgOptions(minLod <= maxLod, 'Error: minLod > maxLod', false, true);
        return [minLod - lodErr, maxLod + lodErr];
    };

    /**
    * @param {number} dudx
    * @param {number} dvdx
    * @param {number} dudy
    * @param {number} dvdy
    * @param {tcuTexLookupVerifier.LodPrecision} prec
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV = function(dudx, dvdx, dudy, dvdy, prec) {
        return tcuTexLookupVerifier.computeLodBoundsFromDerivates(dudx, dvdx, 0, dudy, dvdy, 0, prec);
    };

    /**
    * @param {number} dudx
    * @param {number} dudy
    * @param {tcuTexLookupVerifier.LodPrecision} prec
    * @return {Array<number>}
    */
    tcuTexLookupVerifier.computeLodBoundsFromDerivatesU = function(dudx, dudy, prec) {
        return tcuTexLookupVerifier.computeLodBoundsFromDerivates(dudx, 0, 0, dudy, 0, 0, prec);
    };

    /**
     * @param {Array<number>} coord
     * @param {Array<number>} coordDx
     * @param {Array<number>} coordDy
     * @param {number} faceSize
     * @param {tcuTexLookupVerifier.LodPrecision} prec
     * @return {Array<number>}
     */
    tcuTexLookupVerifier.computeCubeLodBoundsFromDerivates = function(coord, coordDx, coordDy, faceSize, prec) {
        /** @type {boolean} */ var allowBrokenEdgeDerivate = false;
        /** @type {tcuTexture.CubeFace} */ var face = tcuTexture.selectCubeFace(coord);
        /** @type {number} */ var maNdx = 0;
        /** @type {number} */ var sNdx = 0;
        /** @type {number} */ var tNdx = 0;

        // \note Derivate signs don't matter when computing lod
        switch (face) {
            case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X:
            case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X: maNdx = 0; sNdx = 2; tNdx = 1; break;
            case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y:
            case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y: maNdx = 1; sNdx = 0; tNdx = 2; break;
            case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z:
            case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z: maNdx = 2; sNdx = 0; tNdx = 1; break;
            default:
                throw new Error('Invalid CubeFace.');
        }

        /** @type {number} */ var sc = coord[sNdx];
        /** @type {number} */ var tc = coord[tNdx];
        /** @type {number} */ var ma = Math.abs(coord[maNdx]);
        /** @type {number} */ var scdx = coordDx[sNdx];
        /** @type {number} */ var tcdx = coordDx[tNdx];
        /** @type {number} */ var madx = Math.abs(coordDx[maNdx]);
        /** @type {number} */ var scdy = coordDy[sNdx];
        /** @type {number} */ var tcdy = coordDy[tNdx];
        /** @type {number} */ var mady = Math.abs(coordDy[maNdx]);
        /** @type {number} */ var dudx = faceSize * 0.5 * (scdx * ma - sc * madx) / (ma * ma);
        /** @type {number} */ var dvdx = faceSize * 0.5 * (tcdx * ma - tc * madx) / (ma * ma);
        /** @type {number} */ var dudy = faceSize * 0.5 * (scdy * ma - sc * mady) / (ma * ma);
        /** @type {number} */ var dvdy = faceSize * 0.5 * (tcdy * ma - tc * mady) / (ma * ma);
        /** @type {Array<number>} */ var bounds = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(dudx, dvdx, dudy, dvdy, prec);

        // Implementations may compute derivate from projected (s, t) resulting in incorrect values at edges.
        if (allowBrokenEdgeDerivate) {
            /** @type {Array<number>} */ var dxErr = tcuTexVerifierUtil.computeFloatingPointError_Vector(coordDx, [prec.derivateBits, prec.derivateBits, prec.derivateBits]);
            /** @type {Array<number>} */ var dyErr = tcuTexVerifierUtil.computeFloatingPointError_Vector(coordDy, [prec.derivateBits, prec.derivateBits, prec.derivateBits]);
            /** @type {Array<number>} */ var xoffs = deMath.add(deMath.abs(coordDx), dxErr);
            /** @type {Array<number>} */ var yoffs = deMath.add(deMath.abs(coordDy), dyErr);

            if (tcuTexture.selectCubeFace(deMath.add(coord, xoffs)) != face ||
                tcuTexture.selectCubeFace(deMath.subtract(coord, xoffs)) != face ||
                tcuTexture.selectCubeFace(deMath.add(coord, yoffs)) != face ||
                tcuTexture.selectCubeFace(deMath.subtract(coord, yoffs)) != face) {
                return [bounds[0], 1000];
            }
        }

        return bounds;
    };

    /**
     * @param {Array<number>} lodBounds
     * @param {Array<number>} lodMinMax
     * @param {tcuTexLookupVerifier.LodPrecision} prec
     * @return {Array<number>}
     */
    tcuTexLookupVerifier.clampLodBounds = function(lodBounds, lodMinMax, prec) {
        /** @type {number} */ var lodErr = tcuTexVerifierUtil.computeFixedPointError(prec.lodBits);
        /** @type {number} */ var a = lodMinMax[0];
        /** @type {number} */ var b = lodMinMax[1];
        return [deMath.clamp(lodBounds[0], a - lodErr, b - lodErr), deMath.clamp(lodBounds[1], a + lodErr, b + lodErr)];
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.TexLookupScaleMode} scaleMode
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {number} coordZ
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLevel2DLookupResultValid = function(access, sampler, scaleMode, prec, coord, coordZ, result) {
        /** @type {tcuTexture.FilterMode} */
        var filterMode = (scaleMode == tcuTexLookupVerifier.TexLookupScaleMode.MAGNIFY) ? sampler.magFilter : sampler.minFilter;
        return tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec2AndInt(access, sampler, filterMode, prec, coord, coordZ, result);
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.TexLookupScaleMode} scaleMode
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {number} coordZ
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLevel2DLookupResultValid_Int = function(access, sampler, scaleMode, prec, coord, coordZ, result) {
        assertMsgOptions(sampler.minFilter == tcuTexture.FilterMode.NEAREST && sampler.magFilter == tcuTexture.FilterMode.NEAREST, 'minFilter and magFilter must be NEAREST', false, true);
        return tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec2AndInt(access, sampler, prec, coord, coordZ, result);
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.TexLookupScaleMode} scaleMode
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLevel3DLookupResultValid = function(access, sampler, scaleMode, prec, coord, result) {
        /** @type {tcuTexture.FilterMode} */
        var filterMode = (scaleMode == tcuTexLookupVerifier.TexLookupScaleMode.MAGNIFY) ? sampler.magFilter : sampler.minFilter;
        return tcuTexLookupVerifier.isLevelSampleResultValid_CoordAsVec3(access, sampler, filterMode, prec, coord, result);
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} access
     * @param {tcuTexture.Sampler} sampler
     * @param {tcuTexLookupVerifier.TexLookupScaleMode} scaleMode
     * @param {tcuTexLookupVerifier.LookupPrecision} prec
     * @param {Array<number>} coord
     * @param {Array<number>} result
     * @return {boolean}
     */
    tcuTexLookupVerifier.isLevel3DLookupResultValid_Int = function(access, sampler, scaleMode, prec, coord, result) {
        assertMsgOptions(sampler.minFilter == tcuTexture.FilterMode.NEAREST && sampler.magFilter == tcuTexture.FilterMode.NEAREST, 'minFilter and magFilter must be NEAREST', false, true);
        return tcuTexLookupVerifier.isNearestSampleResultValid_CoordAsVec3(access, sampler, prec, coord, result);
    };

});

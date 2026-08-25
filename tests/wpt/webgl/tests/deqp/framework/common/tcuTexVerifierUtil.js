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
goog.provide('framework.common.tcuTexVerifierUtil');
goog.require('framework.common.tcuFloat');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deUtil');

goog.scope(function() {

    var tcuTexVerifierUtil = framework.common.tcuTexVerifierUtil;
    var deMath = framework.delibs.debase.deMath;
    var deUtil = framework.delibs.debase.deUtil;
    var tcuFloat = framework.common.tcuFloat;
    var tcuTexture = framework.common.tcuTexture;

    /**
     * @param {number} value
     * @param {number} numAccurateBits
     * @return {number}
     */
    tcuTexVerifierUtil.computeFloatingPointError = function(value, numAccurateBits) {
        /** @type {number} */ var numGarbageBits = 23 - numAccurateBits;
        /** @type {number} */ var mask = (1 << numGarbageBits) - 1;
        /** @type {number} */ var exp = tcuFloat.newFloat32(value).exponent();

        /** @type {tcuFloat.deFloat} */ var v1 = new tcuFloat.deFloat();
        /** @type {tcuFloat.deFloat} */ var v2 = new tcuFloat.deFloat();
        return v1.construct(1, exp, 1 << 23 | mask).getValue() - v2.construct(1, exp, 1 << 23).getValue();
    };

    /**
     * @param {number} numAccurateBits
     * @return {number}
     */
    tcuTexVerifierUtil.computeFixedPointError = function(numAccurateBits) {
        return tcuTexVerifierUtil.computeFloatingPointError(1.0, numAccurateBits);
    };

    /**
     * @param {Array<number>} numAccurateBits
     * @return {Array<number>}
     */
    tcuTexVerifierUtil.computeFixedPointError_Vector = function(numAccurateBits) {
        /** @type {Array<number>} */ var res = [];
        for (var ndx = 0; ndx < numAccurateBits.length; ndx++)
            res[ndx] = tcuTexVerifierUtil.computeFixedPointError(numAccurateBits[ndx]);
        return res;
    };

    /**
     * @param {Array<number>} value
     * @param {Array<number>} numAccurateBits
     * @return {Array<number>}
     */
    tcuTexVerifierUtil.computeFloatingPointError_Vector = function(value, numAccurateBits) {
        assertMsgOptions(value.length === numAccurateBits.length, '', false, true);
        /** @type {Array<number>} */ var res = [];
        for (var ndx = 0; ndx < value.length; ndx++)
            res[ndx] = tcuTexVerifierUtil.computeFloatingPointError(value[ndx], numAccurateBits[ndx]);
        return res;
    };

    // Sampler introspection

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {boolean}
     */
    tcuTexVerifierUtil.isNearestMipmapFilter = function(mode) {
        return mode == tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST || mode == tcuTexture.FilterMode.LINEAR_MIPMAP_NEAREST;
    };

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {boolean}
     */
    tcuTexVerifierUtil.isLinearMipmapFilter = function(mode) {
        return mode == tcuTexture.FilterMode.NEAREST_MIPMAP_LINEAR || mode == tcuTexture.FilterMode.LINEAR_MIPMAP_LINEAR;
    };

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {boolean}
     */
    tcuTexVerifierUtil.isMipmapFilter = function(mode) {
        return tcuTexVerifierUtil.isNearestMipmapFilter(mode) || tcuTexVerifierUtil.isLinearMipmapFilter(mode);
    };

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {boolean}
     */
    tcuTexVerifierUtil.isLinearFilter = function(mode) {
        return mode == tcuTexture.FilterMode.LINEAR || mode == tcuTexture.FilterMode.LINEAR_MIPMAP_NEAREST || mode == tcuTexture.FilterMode.LINEAR_MIPMAP_LINEAR;
    };

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {boolean}
     */
    tcuTexVerifierUtil.isNearestFilter = function(mode) {
        return !tcuTexVerifierUtil.isLinearFilter(mode);
    };

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {tcuTexture.FilterMode}
     */
     tcuTexVerifierUtil.getLevelFilter = function(mode) {
        return tcuTexVerifierUtil.isLinearFilter(mode) ? tcuTexture.FilterMode.LINEAR : tcuTexture.FilterMode.NEAREST;
    };

    /**
     * @param {tcuTexture.WrapMode} mode
     * @return {boolean}
     */
    tcuTexVerifierUtil.isWrapModeSupported = function(mode) {
        return mode != tcuTexture.WrapMode.MIRRORED_REPEAT_CL && mode != tcuTexture.WrapMode.REPEAT_CL;
    };

    /**
     *
     * @param {boolean} normalizedCoords
     * @param {number} dim
     * @param {number} coord
     * @param {number} coordBits
     * @param {number} uvBits
     * @return {Array<number>}
     */
    tcuTexVerifierUtil.computeNonNormalizedCoordBounds = function(normalizedCoords, dim, coord, coordBits, uvBits) {
        /** @type {number} */ var coordErr = tcuTexVerifierUtil.computeFloatingPointError(coord, coordBits);
        /** @type {number} */ var minN = coord - coordErr;
        /** @type {number} */ var maxN = coord + coordErr;
        /** @type {number} */ var minA = normalizedCoords ? minN * dim : minN;
        /** @type {number} */ var maxA = normalizedCoords ? maxN * dim : maxN;
        /** @type {number} */ var minC = minA - tcuTexVerifierUtil.computeFixedPointError(uvBits);
        /** @type {number} */ var maxC = maxA + tcuTexVerifierUtil.computeFixedPointError(uvBits);
        assertMsgOptions(minC <= maxC, '', false, true);
        return [minC, maxC];
    };

    /**
     * @param {Array<number>} coord
     * @param {Array<number>} bits
     * @return {?Array<tcuTexture.CubeFace>}
     */
     tcuTexVerifierUtil.getPossibleCubeFaces = function(coord, bits) {

        /** @type {Array<tcuTexture.CubeFace>} */ var faces = [];

        /** @type {number} */ var x = coord[0];
        /** @type {number} */ var y = coord[1];
        /** @type {number} */ var z = coord[2];
        /** @type {number} */ var ax = Math.abs(x);
        /** @type {number} */ var ay = Math.abs(y);
        /** @type {number} */ var az = Math.abs(z);
        /** @type {number} */ var ex = tcuTexVerifierUtil.computeFloatingPointError(x, bits[0]);
        /** @type {number} */ var ey = tcuTexVerifierUtil.computeFloatingPointError(y, bits[1]);
        /** @type {number} */ var ez = tcuTexVerifierUtil.computeFloatingPointError(z, bits[2]);
        /** @type {number} */ var numFaces = 0;

        if (ay + ey < ax - ex && az + ez < ax - ex) {
            if (x >= ex) faces.push(tcuTexture.CubeFace.CUBEFACE_POSITIVE_X);
            if (x <= ex) faces.push(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X);
        } else if (ax + ex < ay - ey && az + ez < ay - ey) {
            if (y >= ey) faces.push(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y);
            if (y <= ey) faces.push(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y);
        } else if (ax + ex < az - ez && ay + ey < az - ez) {
            if (z >= ez) faces.push(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z);
            if (z <= ez) faces.push(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z);
        } else {
            // One or more components are equal (or within error bounds). Allow all faces where major axis is not zero.
            if (ax > ex) {
                faces.push(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X);
                faces.push(tcuTexture.CubeFace.CUBEFACE_POSITIVE_X);
            }

            if (ay > ey) {
                faces.push(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y);
                faces.push(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y);
            }

            if (az > ez) {
                faces.push(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z);
                faces.push(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z);
            }
        }

        return faces.length == 0 ? null : faces;
    };

    /**
     * @param {tcuTexture.Sampler} sampler
     * @return {tcuTexture.Sampler}
     */
    tcuTexVerifierUtil.getUnnormalizedCoordSampler = function(sampler) {
        var copy = /** @type {tcuTexture.Sampler} */ (deUtil.clone(sampler));
        copy.normalizedCoords = false;
        return copy;
    };

    /**
     * @param {number} a
     * @param {number} b
     * @return {number}
     */
    tcuTexVerifierUtil.imod = function(a, b) {
        return deMath.imod(a, b);
    };

    /**
     * @param {number} a
     * @return {number}
     */
    tcuTexVerifierUtil.mirror = function(a) {
        return deMath.mirror(a);
    };

    /**
     * @param {tcuTexture.WrapMode} mode
     * @param {number} c
     * @param {number} size
     * @return {number}
     */
    tcuTexVerifierUtil.wrap = function(mode, c, size) {
        switch (mode) {
            // \note CL and GL modes are handled identically here, as verification process accounts for
            //         accuracy differences caused by different methods (wrapping vs. denormalizing first).
            case tcuTexture.WrapMode.CLAMP_TO_EDGE:
                return deMath.clamp(c, 0, size - 1);

            case tcuTexture.WrapMode.REPEAT_GL:
            case tcuTexture.WrapMode.REPEAT_CL:
                return deMath.imod(c, size);

            case tcuTexture.WrapMode.MIRRORED_REPEAT_GL:
            case tcuTexture.WrapMode.MIRRORED_REPEAT_CL:
                return (size - 1) - deMath.mirror(deMath.imod(c, 2 * size) - size);

            default:
                throw new Error('Wrap mode not supported.');
        }
    };

});

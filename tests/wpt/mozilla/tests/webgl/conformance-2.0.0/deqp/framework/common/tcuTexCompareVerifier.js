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
goog.provide('framework.common.tcuTexCompareVerifier');
goog.require('framework.common.tcuTexVerifierUtil');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var tcuTexCompareVerifier = framework.common.tcuTexCompareVerifier;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuTexVerifierUtil = framework.common.tcuTexVerifierUtil;

/**
 * \brief Texture compare (shadow) lookup precision parameters.
 * @constructor
 * @struct
 * @param {Array<number>=} coordBits
 * @param {Array<number>=} uvwBits
 * @param {number=} pcfBits
 * @param {number=} referenceBits
 * @param {number=} resultBits
 */
tcuTexCompareVerifier.TexComparePrecision = function(coordBits, uvwBits, pcfBits, referenceBits, resultBits) {
    this.coordBits = coordBits === undefined ? [22, 22, 22] : coordBits;
    this.uvwBits = uvwBits === undefined ? [22, 22, 22] : uvwBits;
    this.pcfBits = pcfBits === undefined ? 16 : pcfBits;
    this.referenceBits = referenceBits === undefined ? 16 : referenceBits;
    this.resultBits = resultBits === undefined ? 16 : resultBits;
};

/**
 * @constructor
 * @struct
 */
tcuTexCompareVerifier.CmpResultSet = function() {
    this.isTrue = false;
    this.isFalse = false;
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {number} cmpValue_
 * @param {number} cmpReference_
 * @param {number} referenceBits
 * @param {boolean} isFixedPoint
 * @return {tcuTexCompareVerifier.CmpResultSet}
 */
tcuTexCompareVerifier.execCompare = function(compareMode,
                                 cmpValue_,
                                 cmpReference_,
                                 referenceBits,
                                 isFixedPoint) {
    var clampValues = isFixedPoint; // if comparing against a floating point texture, ref (and value) is not clamped
    var cmpValue = (clampValues) ? (deMath.clamp(cmpValue_, 0, 1)) : (cmpValue_);
    var cmpReference = (clampValues) ? (deMath.clamp(cmpReference_, 0, 1)) : (cmpReference_);
    var err = tcuTexVerifierUtil.computeFixedPointError(referenceBits);
    var res = new tcuTexCompareVerifier.CmpResultSet();

    switch (compareMode) {
        case tcuTexture.CompareMode.COMPAREMODE_LESS:
            res.isTrue = cmpReference - err < cmpValue;
            res.isFalse = cmpReference + err >= cmpValue;
            break;

        case tcuTexture.CompareMode.COMPAREMODE_LESS_OR_EQUAL:
            res.isTrue = cmpReference - err <= cmpValue;
            res.isFalse = cmpReference + err > cmpValue;
            break;

        case tcuTexture.CompareMode.COMPAREMODE_GREATER:
            res.isTrue = cmpReference + err > cmpValue;
            res.isFalse = cmpReference - err <= cmpValue;
            break;

        case tcuTexture.CompareMode.COMPAREMODE_GREATER_OR_EQUAL:
            res.isTrue = cmpReference + err >= cmpValue;
            res.isFalse = cmpReference - err < cmpValue;
            break;

        case tcuTexture.CompareMode.COMPAREMODE_EQUAL:
            res.isTrue = deMath.deInRange32(cmpValue, cmpReference - err, cmpReference + err);
            res.isFalse = err != 0 || cmpValue != cmpReference;
            break;

        case tcuTexture.CompareMode.COMPAREMODE_NOT_EQUAL:
            res.isTrue = err != 0 || cmpValue != cmpReference;
            res.isFalse = deMath.deInRange32(cmpValue, cmpReference - err, cmpReference + err);
            break;

        case tcuTexture.CompareMode.COMPAREMODE_ALWAYS:
            res.isTrue = true;
            break;

        case tcuTexture.CompareMode.COMPAREMODE_NEVER:
            res.isFalse = true;
            break;

        default:
            throw new Error('Invalid compare mode:' + compareMode);
    }

    assertMsgOptions(res.isTrue || res.isFalse, 'Both tests failed!', false, true);
    return res;
};

/**
 * @param {tcuTexture.TextureFormat} format
 * @return {boolean}
 */
tcuTexCompareVerifier.isFixedPointDepthTextureFormat = function(format) {
    var channelClass = tcuTexture.getTextureChannelClass(format.type);

    if (format.order == tcuTexture.ChannelOrder.D) {
        // depth internal formats cannot be non-normalized integers
        return channelClass != tcuTexture.TextureChannelClass.FLOATING_POINT;
    } else if (format.order == tcuTexture.ChannelOrder.DS) {
        // combined formats have no single channel class, detect format manually
        switch (format.type) {
            case tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV: return false;
            case tcuTexture.ChannelType.UNSIGNED_INT_24_8: return true;

            default:
                throw new Error('Invalid texture format: ' + format);
        }
    }

    return false;
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths
 * @param {Array<number>} fBounds
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isLinearCompareValid = function(compareMode, prec, depths, fBounds, cmpReference, result, isFixedPointDepth) {
    assertMsgOptions(fBounds[0] >= 0 && fBounds[0] <= fBounds[1] && fBounds[1] <= 1, 'Invalid fBounds', false, true);

    var d0 = depths[0];
    var d1 = depths[1];

    var cmp0 = tcuTexCompareVerifier.execCompare(compareMode, d0, cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp1 = tcuTexCompareVerifier.execCompare(compareMode, d1, cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp = [cmp0, cmp1];

    var isTrue = getMask(cmp, function(x) {return x.isTrue;});
    var isFalse = getMask(cmp, function(x) {return x.isFalse;});

    var f0 = fBounds[0];
    var f1 = fBounds[1];

    var pcfErr = tcuTexVerifierUtil.computeFixedPointError(prec.pcfBits);
    var resErr = tcuTexVerifierUtil.computeFixedPointError(prec.resultBits);
    var totalErr = pcfErr + resErr;

    for (var comb = 0; comb < 4; comb++) {
        if (((comb & isTrue) | (~comb & isFalse )) != 3)
            continue;

        var cmp0True = ((comb >> 0) & 1) != 0;
        var cmp1True = ((comb >> 1) & 1) != 0;

        var ref0 = cmp0True ? 1 : 0;
        var ref1 = cmp1True ? 1 : 0;

        var v0 = ref0 * (1 - f0) + ref1 * f0;
        var v1 = ref0 * (1 - f1) + ref1 * f1;
        var minV = Math.min(v0, v1);
        var maxV = Math.max(v0, v1);
        var minR = minV - totalErr;
        var maxR = maxV + totalErr;

        if (deMath.deInRange32(result, minR, maxR))
            return true;
    }
    return false;
};

/**
 * @param {number} val
 * @param {number} offset
 * @return {Array<boolean>}
 */
tcuTexCompareVerifier.extractBVec4 = function(val, offset) {
    return [
        ((val >> (offset + 0)) & 1) != 0,
        ((val >> (offset + 1)) & 1) != 0,
        ((val >> (offset + 2)) & 1) != 0,
        ((val >> (offset + 3)) & 1) != 0];
};

/**
 * Values are in order (0,0), (1,0), (0,1), (1,1)
 * @param {Array<number>} values
 * @param {number} x
 * @param {number} y
 * @return {number}
 */
tcuTexCompareVerifier.bilinearInterpolate = function(values, x, y) {
    var v00 = values[0];
    var v10 = values[1];
    var v01 = values[2];
    var v11 = values[3];
    var res = v00 * (1 - x) * (1 - y) + v10 * x * (1 - y) + v01 * (1 - x) * y + v11 * x * y;
    return res;
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths vec4
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isBilinearAnyCompareValid = function(compareMode,
                                    prec,
                                    depths,
                                    cmpReference,
                                    result,
                                    isFixedPointDepth) {
    assertMsgOptions(prec.pcfBits === 0, 'PCF bits must be 0', false, true);

    var d0 = depths[0];
    var d1 = depths[1];
    var d2 = depths[2];
    var d3 = depths[3];

    var cmp0 = tcuTexCompareVerifier.execCompare(compareMode, d0, cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp1 = tcuTexCompareVerifier.execCompare(compareMode, d1, cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp2 = tcuTexCompareVerifier.execCompare(compareMode, d2, cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp3 = tcuTexCompareVerifier.execCompare(compareMode, d3, cmpReference, prec.referenceBits, isFixedPointDepth);

    var canBeTrue = cmp0.isTrue || cmp1.isTrue || cmp2.isTrue || cmp3.isTrue;
    var canBeFalse = cmp0.isFalse || cmp1.isFalse || cmp2.isFalse || cmp3.isFalse;

    var resErr = tcuTexVerifierUtil.computeFixedPointError(prec.resultBits);

    var minBound = canBeFalse ? 0 : 1;
    var maxBound = canBeTrue ? 1 : 0;

    return deMath.deInRange32(result, minBound - resErr, maxBound + resErr);
};

/**
 * @param {Array<tcuTexCompareVerifier.CmpResultSet>} arr
 * @param {function(tcuTexCompareVerifier.CmpResultSet): boolean} getValue
 * @return {number}
 */
var getMask = function(arr, getValue) {
    var mask = 0;
    for (var i = 0; i < arr.length; i++) {
        var val = getValue(arr[i]);
        if (val)
            mask |= 1 << i;
    }
    return mask;
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths vec4
 * @param {Array<number>} xBounds vec2
 * @param {Array<number>} yBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isBilinearPCFCompareValid = function(compareMode,
                                    prec,
                                    depths,
                                    xBounds,
                                    yBounds,
                                    cmpReference,
                                    result,
                                    isFixedPointDepth) {
    assertMsgOptions(0.0 <= xBounds[0] && xBounds[0] <= xBounds[1] && xBounds[1] <= 1.0, 'x coordinate out of bounds', false, true);
    assertMsgOptions(0.0 <= yBounds[0] && yBounds[0] <= yBounds[1] && yBounds[1] <= 1.0, 'y coordinate out of bounds', false, true);
    assertMsgOptions(prec.pcfBits > 0, 'PCF bits must be > 0', false, true);

    var d0 = depths[0];
    var d1 = depths[1];
    var d2 = depths[2];
    var d3 = depths[3];

    /** @type {Array<tcuTexCompareVerifier.CmpResultSet>} */ var cmp = [];
    cmp[0] = tcuTexCompareVerifier.execCompare(compareMode, d0, cmpReference, prec.referenceBits, isFixedPointDepth);
    cmp[1] = tcuTexCompareVerifier.execCompare(compareMode, d1, cmpReference, prec.referenceBits, isFixedPointDepth);
    cmp[2] = tcuTexCompareVerifier.execCompare(compareMode, d2, cmpReference, prec.referenceBits, isFixedPointDepth);
    cmp[3] = tcuTexCompareVerifier.execCompare(compareMode, d3, cmpReference, prec.referenceBits, isFixedPointDepth);

    var isTrue = getMask(cmp, function(x) {return x.isTrue});
    var isFalse = getMask(cmp, function(x) {return x.isFalse});

    // Interpolation parameters
    var x0 = xBounds[0];
    var x1 = xBounds[1];
    var y0 = yBounds[0];
    var y1 = yBounds[1];

    // Error parameters
    var pcfErr = tcuTexVerifierUtil.computeFixedPointError(prec.pcfBits);
    var resErr = tcuTexVerifierUtil.computeFixedPointError(prec.resultBits);
    var totalErr = pcfErr + resErr;

    // Iterate over all valid combinations.
    // \note It is not enough to compute minmax over all possible result sets, as ranges may
    //       not necessarily overlap, i.e. there are gaps between valid ranges.
    for (var comb = 0; comb < (1 << 4); comb++) {
        // Filter out invalid combinations:
        //  1) True bit is set in comb but not in isTrue => sample can not be true
        //  2) True bit is NOT set in comb and not in isFalse => sample can not be false
        if (((comb & isTrue) | (~comb & isFalse)) != (1 << 4) - 1)
            continue;

        var cmpTrue = tcuTexCompareVerifier.extractBVec4(comb, 0);
        var refVal = tcuTextureUtil.select([1, 1, 1, 1], [0, 0, 0, 0], cmpTrue);

        var v0 = tcuTexCompareVerifier.bilinearInterpolate(refVal, x0, y0);
        var v1 = tcuTexCompareVerifier.bilinearInterpolate(refVal, x1, y0);
        var v2 = tcuTexCompareVerifier.bilinearInterpolate(refVal, x0, y1);
        var v3 = tcuTexCompareVerifier.bilinearInterpolate(refVal, x1, y1);
        var minV = Math.min(v0, v1, v2, v3);
        var maxV = Math.max(v0, v1, v2, v3);
        var minR = minV - totalErr;
        var maxR = maxV + totalErr;

        if (deMath.deInRange32(result, minR, maxR))
            return true;
    }

    return false;
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths vec4
 * @param {Array<number>} xBounds vec2
 * @param {Array<number>} yBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isBilinearCompareValid = function(compareMode,
                                    prec,
                                    depths,
                                    xBounds,
                                    yBounds,
                                    cmpReference,
                                    result,
                                    isFixedPointDepth) {
    if (prec.pcfBits > 0)
        return tcuTexCompareVerifier.isBilinearPCFCompareValid(compareMode, prec, depths, xBounds, yBounds, cmpReference, result, isFixedPointDepth);
    else
        return tcuTexCompareVerifier.isBilinearAnyCompareValid(compareMode, prec, depths, cmpReference, result, isFixedPointDepth);
};
/**
 * @param {tcuTexture.ConstPixelBufferAccess} level
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {number} coordZ
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isLinearCompareResultValid = function(level,
                                       sampler,
                                       prec,
                                       coord,
                                       coordZ,
                                       cmpReference,
                                       result) {
    var isFixedPointDepth = tcuTexCompareVerifier.isFixedPointDepthTextureFormat(level.getFormat());
    var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getWidth(), coord[0], prec.coordBits[0], prec.uvwBits[0]);
    var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getHeight(), coord[1], prec.coordBits[1], prec.uvwBits[1]);

    // Integer coordinate bounds for (x0,y0) - without wrap mode
    var minI = Math.floor(uBounds[0] - 0.5);
    var maxI = Math.floor(uBounds[1] - 0.5);
    var minJ = Math.floor(vBounds[0] - 0.5);
    var maxJ = Math.floor(vBounds[1] - 0.5);

    var w = level.getWidth();
    var h = level.getHeight();

    // \todo [2013-07-03 pyry] This could be optimized by first computing ranges based on wrap mode.

    for (var j = minJ; j <= maxJ; j++) {
        for (var i = minI; i <= maxI; i++) {
            // Wrapped coordinates
            var x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i, w);
            var x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i + 1, w);
            var y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j, h);
            var y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j + 1, h);

            // Bounds for filtering factors
            var minA = deMath.clamp((uBounds[0] - 0.5) - i, 0, 1);
            var maxA = deMath.clamp((uBounds[1] - 0.5) - i, 0, 1);
            var minB = deMath.clamp((vBounds[0] - 0.5) - j, 0, 1);
            var maxB = deMath.clamp((vBounds[1] - 0.5) - j, 0, 1);

            var depths = [
                level.getPixDepth(x0, y0, coordZ),
                level.getPixDepth(x1, y0, coordZ),
                level.getPixDepth(x0, y1, coordZ),
                level.getPixDepth(x1, y1, coordZ)
                ];

            if (tcuTexCompareVerifier.isBilinearCompareValid(sampler.compare, prec, depths, [minA, maxA], [minB, maxB], cmpReference, result, isFixedPointDepth))
                return true;
        }
    }

    return false;
};

/**
 * @param {tcuTexCompareVerifier.CmpResultSet} resultSet
 * @param {number} result
 * @param {number} resultBits
 */
tcuTexCompareVerifier.isResultInSet = function(resultSet, result, resultBits) {
    var err = tcuTexVerifierUtil.computeFixedPointError(resultBits);
    var minR = result - err;
    var maxR = result + err;

    return (resultSet.isTrue && deMath.deInRange32(1, minR, maxR)) ||
           (resultSet.isFalse && deMath.deInRange32(0, minR, maxR));
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} level
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {number} coordZ
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isNearestCompareResultValid = function(level,
                                       sampler,
                                       prec,
                                       coord,
                                       coordZ,
                                       cmpReference,
                                       result) {
    var isFixedPointDepth = tcuTexCompareVerifier.isFixedPointDepthTextureFormat(level.getFormat());
    var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getWidth(), coord[0], prec.coordBits[0], prec.uvwBits[0]);
    var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, level.getHeight(), coord[1], prec.coordBits[1], prec.uvwBits[1]);

    // Integer coordinates - without wrap mode
    var minI = Math.floor(uBounds[0]);
    var maxI = Math.floor(uBounds[1]);
    var minJ = Math.floor(vBounds[0]);
    var maxJ = Math.floor(vBounds[1]);

    for (var j = minJ; j <= maxJ; j++) {
        for (var i = minI; i <= maxI; i++) {
            var x = tcuTexVerifierUtil.wrap(sampler.wrapS, i, level.getWidth());
            var y = tcuTexVerifierUtil.wrap(sampler.wrapT, j, level.getHeight());
            var depth = level.getPixDepth(x, y, coordZ);
            var resSet = tcuTexCompareVerifier.execCompare(sampler.compare, depth, cmpReference, prec.referenceBits, isFixedPointDepth);

            if (tcuTexCompareVerifier.isResultInSet(resSet, result, prec.resultBits))
                return true;
        }
    }

    return false;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} level
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexture.FilterMode} filterMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {number} coordZ
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isLevelCompareResultValid = function(level,
                                       sampler,
                                       filterMode,
                                       prec,
                                       coord,
                                       coordZ,
                                       cmpReference,
                                       result) {
    if (filterMode == tcuTexture.FilterMode.LINEAR)
        return tcuTexCompareVerifier.isLinearCompareResultValid(level, sampler, prec, coord, coordZ, cmpReference, result);
    else
        return tcuTexCompareVerifier.isNearestCompareResultValid(level, sampler, prec, coord, coordZ, cmpReference, result);
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths0 vec4
 * @param {Array<number>} depths1 vec4
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isTrilinearAnyCompareValid = function(compareMode,
                                     prec,
                                     depths0,
                                     depths1,
                                     cmpReference,
                                     result,
                                     isFixedPointDepth) {
    assertMsgOptions(prec.pcfBits === 0, 'PCF bits must be 0', false, true);

    var cmp00 = tcuTexCompareVerifier.execCompare(compareMode, depths0[0], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp01 = tcuTexCompareVerifier.execCompare(compareMode, depths0[1], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp02 = tcuTexCompareVerifier.execCompare(compareMode, depths0[2], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp03 = tcuTexCompareVerifier.execCompare(compareMode, depths0[3], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp10 = tcuTexCompareVerifier.execCompare(compareMode, depths1[0], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp11 = tcuTexCompareVerifier.execCompare(compareMode, depths1[1], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp12 = tcuTexCompareVerifier.execCompare(compareMode, depths1[2], cmpReference, prec.referenceBits, isFixedPointDepth);
    var cmp13 = tcuTexCompareVerifier.execCompare(compareMode, depths1[3], cmpReference, prec.referenceBits, isFixedPointDepth);

    var canBeTrue = cmp00.isTrue ||
                                      cmp01.isTrue ||
                                      cmp02.isTrue ||
                                      cmp03.isTrue ||
                                      cmp10.isTrue ||
                                      cmp11.isTrue ||
                                      cmp12.isTrue ||
                                      cmp13.isTrue;
    var canBeFalse = cmp00.isFalse ||
                                      cmp01.isFalse ||
                                      cmp02.isFalse ||
                                      cmp03.isFalse ||
                                      cmp10.isFalse ||
                                      cmp11.isFalse ||
                                      cmp12.isFalse ||
                                      cmp13.isFalse;

    var resErr = tcuTexVerifierUtil.computeFixedPointError(prec.resultBits);

    var minBound = canBeFalse ? 0 : 1;
    var maxBound = canBeTrue ? 1 : 0;

    return deMath.deInRange32(result, minBound - resErr, maxBound + resErr);
};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths0 vec4
 * @param {Array<number>} depths1 vec4
 * @param {Array<number>} xBounds0
 * @param {Array<number>} yBounds0
 * @param {Array<number>} xBounds1
 * @param {Array<number>} yBounds1
 * @param {Array<number>} fBounds
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isTrilinearPCFCompareValid = function(compareMode,
                                     prec,
                                     depths0,
                                     depths1,
                                     xBounds0,
                                     yBounds0,
                                     xBounds1,
                                     yBounds1,
                                     fBounds,
                                     cmpReference,
                                     result,
                                     isFixedPointDepth) {
    assertMsgOptions(0.0 <= xBounds0[0] && xBounds0[0] <= xBounds0[1] && xBounds0[1] <= 1.0, 'x0 coordinate out of bounds', false, true);
    assertMsgOptions(0.0 <= yBounds0[0] && yBounds0[0] <= yBounds0[1] && yBounds0[1] <= 1.0, 'y0 coordinate out of bounds', false, true);
    assertMsgOptions(0.0 <= xBounds1[0] && xBounds1[0] <= xBounds1[1] && xBounds1[1] <= 1.0, 'x1 coordinate out of bounds', false, true);
    assertMsgOptions(0.0 <= yBounds1[0] && yBounds1[0] <= yBounds1[1] && yBounds1[1] <= 1.0, 'y1 coordinate out of bounds', false, true);
    assertMsgOptions(0.0 <= fBounds[0] && fBounds[0] <= fBounds[1] && fBounds[1] <= 1.0, 'linear factor out of bounds', false, true);
    assertMsgOptions(prec.pcfBits > 0, 'PCF bits must be > 0', false, true);

    /** @type {Array<tcuTexCompareVerifier.CmpResultSet>} */ var cmp = [];
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths0[0], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths0[1], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths0[2], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths0[3], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths1[0], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths1[1], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths1[2], cmpReference, prec.referenceBits, isFixedPointDepth));
    cmp.push(tcuTexCompareVerifier.execCompare(compareMode, depths1[3], cmpReference, prec.referenceBits, isFixedPointDepth));

    var isTrue = getMask(cmp, function(x) {return x.isTrue});
    var isFalse = getMask(cmp, function(x) {return x.isFalse});

    // Error parameters
    var pcfErr = tcuTexVerifierUtil.computeFixedPointError(prec.pcfBits);
    var resErr = tcuTexVerifierUtil.computeFixedPointError(prec.resultBits);
    var totalErr = pcfErr + resErr;

    // Iterate over all valid combinations.
    for (var comb = 0; comb < (1 << 8); comb++) {
        // Filter out invalid combinations.
        if (((comb & isTrue) | (~comb & isFalse)) != (1 << 8) - 1)
            continue;

        var cmpTrue0 = tcuTexCompareVerifier.extractBVec4(comb, 0);
        var cmpTrue1 = tcuTexCompareVerifier.extractBVec4(comb, 4);
        var refVal0 = tcuTextureUtil.select([1, 1, 1, 1], [0, 0, 0, 0], cmpTrue0);
        var refVal1 = tcuTextureUtil.select([1, 1, 1, 1], [0, 0, 0, 0], cmpTrue1);

        // Bilinear interpolation within levels.
        var v00 = tcuTexCompareVerifier.bilinearInterpolate(refVal0, xBounds0[0], yBounds0[0]);
        var v01 = tcuTexCompareVerifier.bilinearInterpolate(refVal0, xBounds0[1], yBounds0[0]);
        var v02 = tcuTexCompareVerifier.bilinearInterpolate(refVal0, xBounds0[0], yBounds0[1]);
        var v03 = tcuTexCompareVerifier.bilinearInterpolate(refVal0, xBounds0[1], yBounds0[1]);
        var minV0 = Math.min(v00, v01, v02, v03);
        var maxV0 = Math.max(v00, v01, v02, v03);

        var v10 = tcuTexCompareVerifier.bilinearInterpolate(refVal1, xBounds1[0], yBounds1[0]);
        var v11 = tcuTexCompareVerifier.bilinearInterpolate(refVal1, xBounds1[1], yBounds1[0]);
        var v12 = tcuTexCompareVerifier.bilinearInterpolate(refVal1, xBounds1[0], yBounds1[1]);
        var v13 = tcuTexCompareVerifier.bilinearInterpolate(refVal1, xBounds1[1], yBounds1[1]);
        var minV1 = Math.min(v10, v11, v12, v13);
        var maxV1 = Math.max(v10, v11, v12, v13);

        // Compute min-max bounds by filtering between minimum bounds and maximum bounds between levels.
        // HW can end up choosing pretty much any of samples between levels, and thus interpolating
        // between minimums should yield lower bound for range, and same for upper bound.
        // \todo [2013-07-17 pyry] This seems separable? Can this be optimized? At least ranges could be pre-computed and later combined.
        var minF0 = minV0 * (1 - fBounds[0]) + minV1 * fBounds[0];
        var minF1 = minV0 * (1 - fBounds[1]) + minV1 * fBounds[1];
        var maxF0 = maxV0 * (1 - fBounds[0]) + maxV1 * fBounds[0];
        var maxF1 = maxV0 * (1 - fBounds[1]) + maxV1 * fBounds[1];

        var minF = Math.min(minF0, minF1);
        var maxF = Math.max(maxF0, maxF1);

        var minR = minF - totalErr;
        var maxR = maxF + totalErr;

        if (deMath.deInRange32(result, minR, maxR))
            return true;
    }

    return false;

};

/**
 * @param {tcuTexture.CompareMode} compareMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} depths0 vec4
 * @param {Array<number>} depths1 vec4
 * @param {Array<number>} xBounds0
 * @param {Array<number>} yBounds0
 * @param {Array<number>} xBounds1
 * @param {Array<number>} yBounds1
 * @param {Array<number>} fBounds
 * @param {number} cmpReference
 * @param {number} result
 * @param {boolean} isFixedPointDepth
 * @return {boolean}
 */
tcuTexCompareVerifier.isTrilinearCompareValid = function(compareMode,
                                     prec,
                                     depths0,
                                     depths1,
                                     xBounds0,
                                     yBounds0,
                                     xBounds1,
                                     yBounds1,
                                     fBounds,
                                     cmpReference,
                                     result,
                                     isFixedPointDepth) {
    if (prec.pcfBits > 0)
        return tcuTexCompareVerifier.isTrilinearPCFCompareValid(compareMode, prec, depths0, depths1, xBounds0, yBounds0, xBounds1, yBounds1, fBounds, cmpReference, result, isFixedPointDepth);
    else
        return tcuTexCompareVerifier.isTrilinearAnyCompareValid(compareMode, prec, depths0, depths1, cmpReference, result, isFixedPointDepth);
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} level0
 * @param {tcuTexture.ConstPixelBufferAccess} level1
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {number} coordZ
 * @param {Array<number>} fBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isLinearMipmapLinearCompareResultValid = function(level0,
                                              level1,
                                              sampler,
                                              prec,
                                              coord,
                                              coordZ,
                                              fBounds,
                                              cmpReference,
                                              result) {
    var isFixedPointDepth = tcuTexCompareVerifier.isFixedPointDepthTextureFormat(level0.getFormat());

    // \todo [2013-07-04 pyry] This is strictly not correct as coordinates between levels should be dependent.
    //                         Right now this allows pairing any two valid bilinear quads.

    var w0 = level0.getWidth();
    var w1 = level1.getWidth();
    var h0 = level0.getHeight();
    var h1 = level1.getHeight();

    var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, w0, coord[0], prec.coordBits[0], prec.uvwBits[0]);
    var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, w1, coord[0], prec.coordBits[0], prec.uvwBits[0]);
    var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, h0, coord[1], prec.coordBits[1], prec.uvwBits[1]);
    var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, h1, coord[1], prec.coordBits[1], prec.uvwBits[1]);

    // Integer coordinates - without wrap mode
    var minI0 = Math.floor(uBounds0[0] - 0.5);
    var maxI0 = Math.floor(uBounds0[1] - 0.5);
    var minI1 = Math.floor(uBounds1[0] - 0.5);
    var maxI1 = Math.floor(uBounds1[1] - 0.5);
    var minJ0 = Math.floor(vBounds0[0] - 0.5);
    var maxJ0 = Math.floor(vBounds0[1] - 0.5);
    var minJ1 = Math.floor(vBounds1[0] - 0.5);
    var maxJ1 = Math.floor(vBounds1[1] - 0.5);

    for (var j0 = minJ0; j0 <= maxJ0; j0++) {
        for (var i0 = minI0; i0 <= maxI0; i0++) {
            var minA0 = deMath.clamp((uBounds0[0] - 0.5) - i0, 0, 1);
            var maxA0 = deMath.clamp((uBounds0[1] - 0.5) - i0, 0, 1);
            var minB0 = deMath.clamp((vBounds0[0] - 0.5) - j0, 0, 1);
            var maxB0 = deMath.clamp((vBounds0[1] - 0.5) - j0, 0, 1);
            var depths0 = [];

            var x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0);
            var x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0 + 1, w0);
            var y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0, h0);
            var y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0 + 1, h0);

            depths0[0] = level0.getPixDepth(x0, y0, coordZ);
            depths0[1] = level0.getPixDepth(x1, y0, coordZ);
            depths0[2] = level0.getPixDepth(x0, y1, coordZ);
            depths0[3] = level0.getPixDepth(x1, y1, coordZ);

            for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                for (var i1 = minI1; i1 <= maxI1; i1++) {
                    var minA1 = deMath.clamp((uBounds1[0] - 0.5) - i1, 0, 1);
                    var maxA1 = deMath.clamp((uBounds1[1] - 0.5) - i1, 0, 1);
                    var minB1 = deMath.clamp((vBounds1[0] - 0.5) - j1, 0, 1);
                    var maxB1 = deMath.clamp((vBounds1[1] - 0.5) - j1, 0, 1);
                    var depths1 = [];

                    x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1);
                    x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1 + 1, w1);
                    y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1, h1);
                    y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1 + 1, h1);

                    depths1[0] = level1.getPixDepth(x0, y0, coordZ);
                    depths1[1] = level1.getPixDepth(x1, y0, coordZ);
                    depths1[2] = level1.getPixDepth(x0, y1, coordZ);
                    depths1[3] = level1.getPixDepth(x1, y1, coordZ);

                    if (tcuTexCompareVerifier.isTrilinearCompareValid(sampler.compare, prec, depths0, depths1,
                                                [minA0, maxA0], [minB0, maxB0],
                                                [minA1, maxA1], [minB1, maxB1],
                                                fBounds, cmpReference, result, isFixedPointDepth))
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
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {number} coordZ
 * @param {Array<number>} fBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isNearestMipmapLinearCompareResultValid = function(level0,
                                              level1,
                                              sampler,
                                              prec,
                                              coord,
                                              coordZ,
                                              fBounds,
                                              cmpReference,
                                              result) {
    var isFixedPointDepth = tcuTexCompareVerifier.isFixedPointDepthTextureFormat(level0.getFormat());

    var w0 = level0.getWidth();
    var w1 = level1.getWidth();
    var h0 = level0.getHeight();
    var h1 = level1.getHeight();

    var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, w0, coord[0], prec.coordBits[0], prec.uvwBits[0]);
    var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, w1, coord[0], prec.coordBits[0], prec.uvwBits[0]);
    var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, h0, coord[1], prec.coordBits[1], prec.uvwBits[1]);
    var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, h1, coord[1], prec.coordBits[1], prec.uvwBits[1]);

    var minI0 = Math.floor(uBounds0[0]);
    var maxI0 = Math.floor(uBounds0[1]);
    var minI1 = Math.floor(uBounds1[0]);
    var maxI1 = Math.floor(uBounds1[1]);
    var minJ0 = Math.floor(vBounds0[0]);
    var maxJ0 = Math.floor(vBounds0[1]);
    var minJ1 = Math.floor(vBounds1[0]);
    var maxJ1 = Math.floor(vBounds1[1]);

    for (var j0 = minJ0; j0 <= maxJ0; j0++) {
        for (var i0 = minI0; i0 <= maxI0; i0++) {
            var x0 = tcuTexVerifierUtil.wrap(sampler.wrapS, i0, w0);
            var y0 = tcuTexVerifierUtil.wrap(sampler.wrapT, j0, h0);

            // Derivated from C++ dEQP function lookupDepth()
            // Since x0 and y0 are wrapped, here lookupDepth() returns the same result as getPixDepth()
            assertMsgOptions(deMath.deInBounds32(x0, 0, level0.getWidth()) && deMath.deInBounds32(y0, 0, level0.getHeight()) && deMath.deInBounds32(coordZ, 0, level0.getDepth()), 'x0, y0 or coordZ out of bound.', false, true);
            var depth0 = level0.getPixDepth(x0, y0, coordZ);

            for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                for (var i1 = minI1; i1 <= maxI1; i1++) {
                    var x1 = tcuTexVerifierUtil.wrap(sampler.wrapS, i1, w1);
                    var y1 = tcuTexVerifierUtil.wrap(sampler.wrapT, j1, h1);

                    // Derivated from C++ dEQP function lookupDepth()
                    // Since x1 and y1 are wrapped, here lookupDepth() returns the same result as getPixDepth()
                    assertMsgOptions(deMath.deInBounds32(x1, 0, level1.getWidth()) && deMath.deInBounds32(y1, 0, level1.getHeight()), 'x1 or y1 out of bound.', false, true);
                    var depth1 = level1.getPixDepth(x1, y1, coordZ);

                    if (tcuTexCompareVerifier.isLinearCompareValid(sampler.compare, prec, [depth0, depth1], fBounds, cmpReference, result, isFixedPointDepth))
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
 * @param {tcuTexture.FilterMode} levelFilter
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {number} coordZ
 * @param {Array<number>} fBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isMipmapLinearCompareResultValid = function(level0,
                                              level1,
                                              sampler,
                                              levelFilter,
                                              prec,
                                              coord,
                                              coordZ,
                                              fBounds,
                                              cmpReference,
                                              result) {
    if (levelFilter == tcuTexture.FilterMode.LINEAR)
        return tcuTexCompareVerifier.isLinearMipmapLinearCompareResultValid(level0, level1, sampler, prec, coord, coordZ, fBounds, cmpReference, result);
    else
        return tcuTexCompareVerifier.isNearestMipmapLinearCompareResultValid(level0, level1, sampler, prec, coord, coordZ, fBounds, cmpReference, result);
};

/**
 * @param {tcuTexture.Texture2DView} texture
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {Array<number>} lodBounds vec2 level-of-detail bounds
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isTexCompareResultValid2D = function(texture, sampler, prec, coord, lodBounds, cmpReference, result) {
    var minLod = lodBounds[0];
    var maxLod = lodBounds[1];
    var canBeMagnified = minLod <= sampler.lodThreshold;
    var canBeMinified = maxLod > sampler.lodThreshold;

    if (canBeMagnified) {
        if (tcuTexCompareVerifier.isLevelCompareResultValid(texture.getLevel(0), sampler, sampler.magFilter, prec, coord, 0, cmpReference, result))
            return true;
    }

    if (canBeMinified) {
        var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
        var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
        var minTexLevel = 0;
        var maxTexLevel = texture.getNumLevels() - 1;

        assertMsgOptions(minTexLevel < maxTexLevel, 'Invalid texture levels.', false, true);

        if (isLinearMipmap) {
            var minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
            var maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

            assertMsgOptions(minLevel <= maxLevel, 'Invalid texture levels.', false, true);

            for (var level = minLevel; level <= maxLevel; level++) {
                var minF = deMath.clamp(minLod - level, 0, 1);
                var maxF = deMath.clamp(maxLod - level, 0, 1);

                if (tcuTexCompareVerifier.isMipmapLinearCompareResultValid(texture.getLevel(level), texture.getLevel(level + 1), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coord, 0, [minF, maxF], cmpReference, result))
                    return true;
            }
        } else if (isNearestMipmap) {
            // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
            //       decision to allow floor(lod + 0.5) as well.
            var minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
            var maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

            assertMsgOptions(minLevel <= maxLevel, 'Invalid texture levels.', false, true);

            for (var level = minLevel; level <= maxLevel; level++) {
                if (tcuTexCompareVerifier.isLevelCompareResultValid(texture.getLevel(level), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, coord, 0, cmpReference, result))
                    return true;
            }
        } else {
            if (tcuTexCompareVerifier.isLevelCompareResultValid(texture.getLevel(0), sampler, sampler.minFilter, prec, coord, 0, cmpReference, result))
                return true;
        }
    }

    return false;
};

/**
 * @param {tcuTexture.TextureCubeView} texture
 * @param {number} baseLevelNdx
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {tcuTexture.CubeFaceCoords} coords
 * @param {Array<number>} fBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isSeamplessLinearMipmapLinearCompareResultValid = function(texture,
                                                             baseLevelNdx,
                                                             sampler,
                                                             prec,
                                                             coords,
                                                             fBounds,
                                                             cmpReference,
                                                             result) {
    var isFixedPointDepth = tcuTexCompareVerifier.isFixedPointDepthTextureFormat(texture.getLevelFace(baseLevelNdx, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X).getFormat());
    var size0 = texture.getLevelFace(baseLevelNdx, coords.face).getWidth();
    var size1 = texture.getLevelFace(baseLevelNdx + 1, coords.face).getWidth();

    var uBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size0, coords.s, prec.coordBits[0], prec.uvwBits[0]);
    var uBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size1, coords.s, prec.coordBits[0], prec.uvwBits[0]);
    var vBounds0 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size0, coords.t, prec.coordBits[1], prec.uvwBits[1]);
    var vBounds1 = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size1, coords.t, prec.coordBits[1], prec.uvwBits[1]);

    // Integer coordinates - without wrap mode
    var minI0 = Math.floor(uBounds0[0] - 0.5);
    var maxI0 = Math.floor(uBounds0[1] - 0.5);
    var minI1 = Math.floor(uBounds1[0] - 0.5);
    var maxI1 = Math.floor(uBounds1[1] - 0.5);
    var minJ0 = Math.floor(vBounds0[0] - 0.5);
    var maxJ0 = Math.floor(vBounds0[1] - 0.5);
    var minJ1 = Math.floor(vBounds1[0] - 0.5);
    var maxJ1 = Math.floor(vBounds1[1] - 0.5);

    /** @type {Array<tcuTexture.ConstPixelBufferAccess>} */ var faces0 = [];
    /** @type {Array<tcuTexture.ConstPixelBufferAccess>} */ var faces1 = [];

    for (var key in tcuTexture.CubeFace) {
        var face = tcuTexture.CubeFace[key];
        faces0[face] = texture.getLevelFace(baseLevelNdx, face);
        faces1[face] = texture.getLevelFace(baseLevelNdx + 1, face);
    }

    for (var j0 = minJ0; j0 <= maxJ0; j0++) {
        for (var i0 = minI0; i0 <= maxI0; i0++) {
            var minA0 = deMath.clamp((uBounds0[0] - 0.5) - i0, 0, 1);
            var maxA0 = deMath.clamp((uBounds0[1] - 0.5) - i0, 0, 1);
            var minB0 = deMath.clamp((vBounds0[0] - 0.5) - j0, 0, 1);
            var maxB0 = deMath.clamp((vBounds0[1] - 0.5) - j0, 0, 1);
            var depths0 = [];

            var c00 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 0, j0 + 0]), size0);
            var c10 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 1, j0 + 0]), size0);
            var c01 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 0, j0 + 1]), size0);
            var c11 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i0 + 1, j0 + 1]), size0);

            // If any of samples is out of both edges, implementations can do pretty much anything according to spec.
            // \todo [2013-07-08 pyry] Test the special case where all corner pixels have exactly the same color.
            if (c00 == null || c01 == null || c10 == null || c11 == null)
                return true;

            depths0[0] = faces0[c00.face].getPixDepth(c00.s, c00.t);
            depths0[1] = faces0[c10.face].getPixDepth(c10.s, c10.t);
            depths0[2] = faces0[c01.face].getPixDepth(c01.s, c01.t);
            depths0[3] = faces0[c11.face].getPixDepth(c11.s, c11.t);

            for (var j1 = minJ1; j1 <= maxJ1; j1++) {
                for (var i1 = minI1; i1 <= maxI1; i1++) {
                    var minA1 = deMath.clamp((uBounds1[0] - 0.5) - i1, 0, 1);
                    var maxA1 = deMath.clamp((uBounds1[1] - 0.5) - i1, 0, 1);
                    var minB1 = deMath.clamp((vBounds1[0] - 0.5) - j1, 0, 1);
                    var maxB1 = deMath.clamp((vBounds1[1] - 0.5) - j1, 0, 1);
                    var depths1 = [];

                    c00 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 0, j1 + 0]), size1);
                    c10 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 1, j1 + 0]), size1);
                    c01 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 0, j1 + 1]), size1);
                    c11 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i1 + 1, j1 + 1]), size1);

                    if (c00 == null || c01 == null || c10 == null || c11 == null)
                        return true;

                    depths1[0] = faces1[c00.face].getPixDepth(c00.s, c00.t);
                    depths1[1] = faces1[c10.face].getPixDepth(c10.s, c10.t);
                    depths1[2] = faces1[c01.face].getPixDepth(c01.s, c01.t);
                    depths1[3] = faces1[c11.face].getPixDepth(c11.s, c11.t);

                    if (tcuTexCompareVerifier.isTrilinearCompareValid(sampler.compare, prec, depths0, depths1,
                                                [minA0, maxA0], [minB0, maxB0],
                                                [minA1, maxA1], [minB1, maxB1],
                                                fBounds, cmpReference, result, isFixedPointDepth))
                        return true;
                }
            }
        }
    }

    return false;
};

/**
 * @param {tcuTexture.TextureCubeView} texture
 * @param {number} levelNdx
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {tcuTexture.CubeFaceCoords} coords
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */

tcuTexCompareVerifier.isSeamlessLinearCompareResultValid = function(texture,
                                                levelNdx,
                                                sampler,
                                                prec,
                                                coords,
                                                cmpReference,
                                                result) {
    var isFixedPointDepth = tcuTexCompareVerifier.isFixedPointDepthTextureFormat(texture.getLevelFace(levelNdx, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X).getFormat());
    var size = texture.getLevelFace(levelNdx, coords.face).getWidth();

    var uBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size, coords.s, prec.coordBits[0], prec.uvwBits[0]);
    var vBounds = tcuTexVerifierUtil.computeNonNormalizedCoordBounds(sampler.normalizedCoords, size, coords.t, prec.coordBits[1], prec.uvwBits[1]);

    // Integer coordinate bounds for (x0,y0) - without wrap mode
    var minI = Math.floor(uBounds[0] - 0.5);
    var maxI = Math.floor(uBounds[1] - 0.5);
    var minJ = Math.floor(vBounds[0] - 0.5);
    var maxJ = Math.floor(vBounds[1] - 0.5);

    // Face accesses
    /** @type {Array<tcuTexture.ConstPixelBufferAccess>} */ var faces = [];

    for (var key in tcuTexture.CubeFace) {
        var face = tcuTexture.CubeFace[key];
        faces[face] = texture.getLevelFace(levelNdx, face);
    }

    for (var j = minJ; j <= maxJ; j++) {
        for (var i = minI; i <= maxI; i++) {
            var c00 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 0, j + 0]), size);
            var c10 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 1, j + 0]), size);
            var c01 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 0, j + 1]), size);
            var c11 = tcuTexture.remapCubeEdgeCoords(new tcuTexture.CubeFaceCoords(coords.face, [i + 1, j + 1]), size);

            // If any of samples is out of both edges, implementations can do pretty much anything according to spec.
            // \todo [2013-07-08 pyry] Test the special case where all corner pixels have exactly the same color.
            if (!c00 || !c01 || !c10 || !c11)
                return true;

            // Bounds for filtering factors
            var minA = deMath.clamp((uBounds[0] - 0.5) - i, 0, 1);
            var maxA = deMath.clamp((uBounds[1] - 0.5) - i, 0, 1);
            var minB = deMath.clamp((vBounds[0] - 0.5) - j, 0, 1);
            var maxB = deMath.clamp((vBounds[1] - 0.5) - j, 0, 1);

            var depths = [];
            depths[0] = faces[c00.face].getPixDepth(c00.s, c00.t);
            depths[1] = faces[c10.face].getPixDepth(c10.s, c10.t);
            depths[2] = faces[c01.face].getPixDepth(c01.s, c01.t);
            depths[3] = faces[c11.face].getPixDepth(c11.s, c11.t);

            if (tcuTexCompareVerifier.isBilinearCompareValid(sampler.compare, prec, depths, [minA, maxA], [minB, maxB], cmpReference, result, isFixedPointDepth))
                return true;
        }
    }

    return false;
};

/**
 * @param {tcuTexture.TextureCubeView} texture
 * @param {number} levelNdx
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexture.FilterMode} filterMode
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {tcuTexture.CubeFaceCoords} coords
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isCubeLevelCompareResultValid = function(texture,
                                           levelNdx,
                                           sampler,
                                           filterMode,
                                           prec,
                                           coords,
                                           cmpReference,
                                           result) {
    if (filterMode == tcuTexture.FilterMode.LINEAR) {
        if (sampler.seamlessCubeMap)
            return tcuTexCompareVerifier.isSeamlessLinearCompareResultValid(texture, levelNdx, sampler, prec, coords, cmpReference, result);
        else
            return tcuTexCompareVerifier.isLinearCompareResultValid(texture.getLevelFace(levelNdx, coords.face), sampler, prec, [coords.s, coords.t], 0, cmpReference, result);
    } else
        return tcuTexCompareVerifier.isNearestCompareResultValid(texture.getLevelFace(levelNdx, coords.face), sampler, prec, [coords.s, coords.t], 0, cmpReference, result);
};

/**
 * @param {tcuTexture.TextureCubeView} texture
 * @param {number} baseLevelNdx
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexture.FilterMode} levelFilter
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {tcuTexture.CubeFaceCoords} coords
 * @param {Array<number>} fBounds vec2
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isCubeMipmapLinearCompareResultValid = function(texture,
                                                  baseLevelNdx,
                                                  sampler,
                                                  levelFilter,
                                                  prec,
                                                  coords,
                                                  fBounds,
                                                  cmpReference,
                                                  result) {
    if (levelFilter == tcuTexture.FilterMode.LINEAR) {
        if (sampler.seamlessCubeMap)
            return tcuTexCompareVerifier.isSeamplessLinearMipmapLinearCompareResultValid(texture, baseLevelNdx, sampler, prec, coords, fBounds, cmpReference, result);
        else
            return tcuTexCompareVerifier.isLinearMipmapLinearCompareResultValid(texture.getLevelFace(baseLevelNdx, coords.face),
                                                          texture.getLevelFace(baseLevelNdx + 1, coords.face),
                                                          sampler, prec, [coords.s, coords.t], 0, fBounds, cmpReference, result);
    } else
        return tcuTexCompareVerifier.isNearestMipmapLinearCompareResultValid(texture.getLevelFace(baseLevelNdx, coords.face),
                                                       texture.getLevelFace(baseLevelNdx + 1, coords.face),
                                                       sampler, prec, [coords.s, coords.t], 0, fBounds, cmpReference, result);
};

/**
 * @param {tcuTexture.TextureCubeView} texture
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec2 texture coordinates
 * @param {Array<number>} lodBounds vec2 level-of-detail bounds
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isTexCompareResultValidCube = function(texture, sampler, prec, coord, lodBounds, cmpReference, result) {
    /** @type {Array<tcuTexture.CubeFace>} */var possibleFaces = tcuTexVerifierUtil.getPossibleCubeFaces(coord, prec.coordBits);

    if (!possibleFaces)
        return true; // Result is undefined.

    for (var tryFaceNdx = 0; tryFaceNdx < possibleFaces.length; tryFaceNdx++) {
        var face = possibleFaces[tryFaceNdx];
        var faceCoords = new tcuTexture.CubeFaceCoords(face, tcuTexture.projectToFace(face, coord));
        var minLod = lodBounds[0];
        var maxLod = lodBounds[1];
        var canBeMagnified = minLod <= sampler.lodThreshold;
        var canBeMinified = maxLod > sampler.lodThreshold;

        if (canBeMagnified) {
            if (tcuTexCompareVerifier.isCubeLevelCompareResultValid(texture, 0, sampler, sampler.magFilter, prec, faceCoords, cmpReference, result))
                return true;
        }

        if (canBeMinified) {
            var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
            var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
            var minTexLevel = 0;
            var maxTexLevel = texture.getNumLevels() - 1;

            assertMsgOptions(minTexLevel < maxTexLevel, 'Invalid texture levels.', false, true);

            if (isLinearMipmap) {
                var minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
                var maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

                assertMsgOptions(minLevel <= maxLevel, 'Invalid texture levels.', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    var minF = deMath.clamp(minLod - level, 0, 1);
                    var maxF = deMath.clamp(maxLod - level, 0, 1);

                    if (tcuTexCompareVerifier.isCubeMipmapLinearCompareResultValid(texture, level, sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, faceCoords, [minF, maxF], cmpReference, result))
                        return true;
                }
            } else if (isNearestMipmap) {
                // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
                //       decision to allow floor(lod + 0.5) as well.
                var minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
                var maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

                assertMsgOptions(minLevel <= maxLevel, 'Invalid texture levels.', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    if (tcuTexCompareVerifier.isCubeLevelCompareResultValid(texture, level, sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, faceCoords, cmpReference, result))
                        return true;
                }
            } else {
                if (tcuTexCompareVerifier.isCubeLevelCompareResultValid(texture, 0, sampler, sampler.minFilter, prec, faceCoords, cmpReference, result))
                    return true;
            }
        }
    }

    return false;
};

/**
 * @param {tcuTexture.Texture2DArrayView} texture
 * @param {tcuTexture.Sampler} sampler
 * @param {tcuTexCompareVerifier.TexComparePrecision} prec
 * @param {Array<number>} coord vec3 texture coordinates
 * @param {Array<number>} lodBounds vec2 level-of-detail bounds
 * @param {number} cmpReference
 * @param {number} result
 * @return {boolean}
 */
tcuTexCompareVerifier.isTexCompareResultValid2DArray = function(texture, sampler, prec, coord, lodBounds, cmpReference, result) {
    var depthErr = tcuTexVerifierUtil.computeFloatingPointError(coord[2], prec.coordBits[2]) + tcuTexVerifierUtil.computeFixedPointError(prec.uvwBits[2]);
    var minZ = coord[2] - depthErr;
    var maxZ = coord[2] + depthErr;
    var minLayer = deMath.clamp(Math.floor(minZ + 0.5), 0, texture.getNumLayers() - 1);
    var maxLayer = deMath.clamp(Math.floor(maxZ + 0.5), 0, texture.getNumLayers() - 1);

    for (var layer = minLayer; layer <= maxLayer; layer++) {
        var minLod = lodBounds[0];
        var maxLod = lodBounds[1];
        var canBeMagnified = minLod <= sampler.lodThreshold;
        var canBeMinified = maxLod > sampler.lodThreshold;

        if (canBeMagnified) {
            if (tcuTexCompareVerifier.isLevelCompareResultValid(texture.getLevel(0), sampler, sampler.magFilter, prec, deMath.swizzle(coord, [0, 1]), layer, cmpReference, result))
                return true;
        }

        if (canBeMinified) {
            var isNearestMipmap = tcuTexVerifierUtil.isNearestMipmapFilter(sampler.minFilter);
            var isLinearMipmap = tcuTexVerifierUtil.isLinearMipmapFilter(sampler.minFilter);
            var minTexLevel = 0;
            var maxTexLevel = texture.getNumLevels() - 1;

            assertMsgOptions(minTexLevel < maxTexLevel, 'Invalid texture levels.', false, true);

            if (isLinearMipmap) {
                var minLevel = deMath.clamp(Math.floor(minLod), minTexLevel, maxTexLevel - 1);
                var maxLevel = deMath.clamp(Math.floor(maxLod), minTexLevel, maxTexLevel - 1);

                assertMsgOptions(minLevel <= maxLevel, 'Invalid texture levels.', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    var minF = deMath.clamp(minLod - level, 0, 1);
                    var maxF = deMath.clamp(maxLod - level, 0, 1);

                    if (tcuTexCompareVerifier.isMipmapLinearCompareResultValid(texture.getLevel(level), texture.getLevel(level + 1), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, deMath.swizzle(coord, [0, 1]), layer, [minF, maxF], cmpReference, result))
                        return true;
                }
            } else if (isNearestMipmap) {
                // \note The accurate formula for nearest mipmapping is level = ceil(lod + 0.5) - 1 but Khronos has made
                //       decision to allow floor(lod + 0.5) as well.
                var minLevel = deMath.clamp(Math.ceil(minLod + 0.5) - 1, minTexLevel, maxTexLevel);
                var maxLevel = deMath.clamp(Math.floor(maxLod + 0.5), minTexLevel, maxTexLevel);

                assertMsgOptions(minLevel <= maxLevel, 'Invalid texture levels.', false, true);

                for (var level = minLevel; level <= maxLevel; level++) {
                    if (tcuTexCompareVerifier.isLevelCompareResultValid(texture.getLevel(level), sampler, tcuTexVerifierUtil.getLevelFilter(sampler.minFilter), prec, deMath.swizzle(coord, [0, 1]), layer, cmpReference, result))
                        return true;
                }
            } else {
                if (tcuTexCompareVerifier.isLevelCompareResultValid(texture.getLevel(0), sampler, sampler.minFilter, prec, deMath.swizzle(coord, [0, 1]), layer, cmpReference, result))
                    return true;
            }
        }
    }

    return false;
};

});

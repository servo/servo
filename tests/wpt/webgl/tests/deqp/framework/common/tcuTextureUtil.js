/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a tcuTextureUtil.copy of the License at
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
goog.provide('framework.common.tcuTextureUtil');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');

goog.scope(function() {

var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var deRandom = framework.delibs.debase.deRandom;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

/**
 * @param {number} t
 * @param {number} minVal
 * @param {number} maxVal
 * @return {number}
 */
tcuTextureUtil.linearInterpolate = function(t, minVal, maxVal) {
    return minVal + (maxVal - minVal) * t;
};

/** tcuTextureUtil.linearChannelToSRGB
 * @param {number} cl
 * @return {number}
 */
tcuTextureUtil.linearChannelToSRGB = function(cl) {
    if (cl <= 0.0)
        return 0.0;
    else if (cl < 0.0031308)
        return 12.92 * cl;
    else if (cl < 1.0)
        return 1.055 * Math.pow(cl, 0.41666) - 0.055;
    else
        return 1.0;
};

/**
 * Convert sRGB to linear colorspace
 * @param {Array<number>} cs
 * @return {Array<number>}
 */
tcuTextureUtil.sRGBToLinear = function(cs) {
    return [tcuTextureUtil.sRGBChannelToLinear(cs[0]),
            tcuTextureUtil.sRGBChannelToLinear(cs[1]),
            tcuTextureUtil.sRGBChannelToLinear(cs[2]),
            cs[3]];
};

/**
 * @param {number} cs
 * @return {number}
 */
 tcuTextureUtil.sRGBChannelToLinear = function(cs) {
    if (cs <= 0.04045)
        return cs / 12.92;
    else
        return Math.pow((cs + 0.055) / 1.055, 2.4);
};

/** tcuTextureUtil.linearToSRGB
 * @param {Array<number>} cl
 * @return {Array<number>}
 */
tcuTextureUtil.linearToSRGB = function(cl) {
    return [tcuTextureUtil.linearChannelToSRGB(cl[0]),
            tcuTextureUtil.linearChannelToSRGB(cl[1]),
            tcuTextureUtil.linearChannelToSRGB(cl[2]),
            cl[3]
            ];
};

/**
 * tcuTextureUtil.getSubregion
 * @param {tcuTexture.PixelBufferAccess} access
 * @param {number} x
 * @param {number} y
 * @param {number} z
 * @param {number} width
 * @param {number} height
 * @param {number} depth
 * @return {tcuTexture.PixelBufferAccess}
 */
tcuTextureUtil.getSubregion = function(access, x, y, z, width, height, depth) {

    DE_ASSERT(deMath.deInBounds32(x, 0, access.getWidth()) && deMath.deInRange32(x + width, x, access.getWidth()));
    DE_ASSERT(deMath.deInBounds32(y, 0, access.getHeight()) && deMath.deInRange32(y + height, y, access.getHeight()));
    DE_ASSERT(deMath.deInBounds32(z, 0, access.getDepth()) && deMath.deInRange32(z + depth, z, access.getDepth()));

    return new tcuTexture.PixelBufferAccess({
        format: access.getFormat(),
        width: width,
        height: height,
        depth: depth,
        rowPitch: access.getRowPitch(),
        slicePitch: access.getSlicePitch(),
        offset: access.m_offset + access.getFormat().getPixelSize() * x + access.getRowPitch() * y + access.getSlicePitch() * z,
        data: access.getBuffer()
        });
};

/**
 * @param {tcuTexture.PixelBufferAccess} access
 * @param {Array<number>} minVal
 * @param {Array<number>} maxVal
 */
tcuTextureUtil.fillWithComponentGradients1D = function(access, minVal, maxVal) {
    DE_ASSERT(access.getHeight() == 1);
    for (var x = 0; x < access.getWidth(); x++) {
        var s = (x + 0.5) / access.getWidth();

        var r = tcuTextureUtil.linearInterpolate(s, minVal[0], maxVal[0]);
        var g = tcuTextureUtil.linearInterpolate(s, minVal[1], maxVal[1]);
        var b = tcuTextureUtil.linearInterpolate(s, minVal[2], maxVal[2]);
        var a = tcuTextureUtil.linearInterpolate(s, minVal[3], maxVal[3]);

        access.setPixel([r, g, b, a], x, 0);
    }
};

/**
 * @param {tcuTexture.PixelBufferAccess} access
 * @param {Array<number>} minVal
 * @param {Array<number>} maxVal
 */
tcuTextureUtil.fillWithComponentGradients2D = function(access, minVal, maxVal) {
    for (var y = 0; y < access.getHeight(); y++) {
        var t = (y + 0.5) / access.getHeight();
        for (var x = 0; x < access.getWidth(); x++) {
            var s = (x + 0.5) / access.getWidth();

            var r = tcuTextureUtil.linearInterpolate((s + t) * 0.5, minVal[0], maxVal[0]);
            var g = tcuTextureUtil.linearInterpolate((s + (1 - t)) * 0.5, minVal[1], maxVal[1]);
            var b = tcuTextureUtil.linearInterpolate(((1 - s) + t) * 0.5, minVal[2], maxVal[2]);
            var a = tcuTextureUtil.linearInterpolate(((1 - s) + (1 - t)) * 0.5, minVal[3], maxVal[3]);

            access.setPixel([r, g, b, a], x, y);
        }
    }
};

/**
 * @param {tcuTexture.PixelBufferAccess} dst
 * @param {Array<number>} minVal
 * @param {Array<number>} maxVal
 */
tcuTextureUtil.fillWithComponentGradients3D = function(dst, minVal, maxVal) {
    for (var z = 0; z < dst.getDepth(); z++) {
        var p = (z + 0.5) / dst.getDepth();
        var b = tcuTextureUtil.linearInterpolate(p, minVal[2], maxVal[2]);
        for (var y = 0; y < dst.getHeight(); y++) {
            var t = (y + 0.5) / dst.getHeight();
            var g = tcuTextureUtil.linearInterpolate(t, minVal[1], maxVal[1]);
            for (var x = 0; x < dst.getWidth(); x++) {
                var s = (x + 0.5) / dst.getWidth();
                var r = tcuTextureUtil.linearInterpolate(s, minVal[0], maxVal[0]);
                var a = tcuTextureUtil.linearInterpolate(1 - (s + t + p) / 3, minVal[3], maxVal[3]);
                dst.setPixel([r, g, b, a], x, y, z);
            }
        }
    }
};

/**
 * @param {tcuTexture.PixelBufferAccess} access
 * @param {Array<number>} minVal
 * @param {Array<number>} maxVal
 */
tcuTextureUtil.fillWithComponentGradients = function(access, minVal, maxVal) {
    if (access.getHeight() == 1 && access.getDepth() == 1)
        tcuTextureUtil.fillWithComponentGradients1D(access, minVal, maxVal);
    else if (access.getDepth() == 1)
        tcuTextureUtil.fillWithComponentGradients2D(access, minVal, maxVal);
    else
        tcuTextureUtil.fillWithComponentGradients3D(access, minVal, maxVal);
};

/**
 * @param {tcuTexture.PixelBufferAccess} dst
 */
tcuTextureUtil.fillWithRGBAQuads = function(dst) {
    checkMessage(dst.getDepth() == 1, 'Depth must be 1');
    var width = dst.getWidth();
    var height = dst.getHeight();
    var left = width / 2;
    var top = height / 2;

    tcuTextureUtil.getSubregion(dst, 0, 0, 0, left, top, 1).clear([1.0, 0.0, 0.0, 1.0]);
    tcuTextureUtil.getSubregion(dst, left, 0, 0, width - left, top, 1).clear([0.0, 1.0, 0.0, 1.0]);
    tcuTextureUtil.getSubregion(dst, 0, top, 0, left, height - top, 1).clear([0.0, 0.0, 1.0, 0.0]);
    tcuTextureUtil.getSubregion(dst, left, top, 0, width - left, height - top, 1).clear([0.5, 0.5, 0.5, 1.0]);
};

// \todo [2012-11-13 pyry] There is much better metaballs code in CL SIR value generators.
/**
 * @param {tcuTexture.PixelBufferAccess} dst
 * @param {number} numBalls
 * @param {number} seed
 */
tcuTextureUtil.fillWithMetaballs = function(dst, numBalls, seed) {
    checkMessage(dst.getDepth() == 1, 'Depth must be 1');
    var points = [];
    var rnd = new deRandom.Random(seed);

    for (var i = 0; i < numBalls; i++) {
        var x = rnd.getFloat();
        var y = rnd.getFloat();
        points[i] = [x, y];
    }

    for (var y = 0; y < dst.getHeight(); y++)
    for (var x = 0; x < dst.getWidth(); x++) {
        var p = [x / dst.getWidth(), y / dst.getHeight()];

        var sum = 0.0;
        for (var pointNdx = 0; pointNdx < points.length; pointNdx++) {
            var d = deMath.subtract(p, points[pointNdx]);
            var f = 0.01 / (d[0] * d[0] + d[1] * d[1]);

            sum += f;
        }

        dst.setPixel([sum, sum, sum, sum], x, y);
    }
};

/**
 * Create tcuTextureUtil.TextureFormatInfo.
 * @constructor
 * @param {Array<number>} valueMin
 * @param {Array<number>} valueMax
 * @param {Array<number>} lookupScale
 * @param {Array<number>} lookupBias
 */
tcuTextureUtil.TextureFormatInfo = function(valueMin, valueMax, lookupScale, lookupBias) {
    /** @type {Array<number>} */ this.valueMin = valueMin;
    /** @type {Array<number>} */ this.valueMax = valueMax;
    /** @type {Array<number>} */ this.lookupScale = lookupScale;
    /** @type {Array<number>} */ this.lookupBias = lookupBias;
};

/**
 * @param {?tcuTexture.ChannelType} channelType
 * @return {Array<number>}
 */
tcuTextureUtil.getChannelValueRange = function(channelType) {
    var cMin = 0;
    var cMax = 0;

    switch (channelType) {
        // Signed normalized formats.
        case tcuTexture.ChannelType.SNORM_INT8:
        case tcuTexture.ChannelType.SNORM_INT16: cMin = -1; cMax = 1; break;

        // Unsigned normalized formats.
        case tcuTexture.ChannelType.UNORM_INT8:
        case tcuTexture.ChannelType.UNORM_INT16:
        case tcuTexture.ChannelType.UNORM_SHORT_565:
        case tcuTexture.ChannelType.UNORM_SHORT_4444:
        case tcuTexture.ChannelType.UNORM_INT_101010:
        case tcuTexture.ChannelType.UNORM_INT_1010102_REV: cMin = 0; cMax = 1; break;

        // Misc formats.
        case tcuTexture.ChannelType.SIGNED_INT8: cMin = -128; cMax = 127; break;
        case tcuTexture.ChannelType.SIGNED_INT16: cMin = -32768; cMax = 32767; break;
        case tcuTexture.ChannelType.SIGNED_INT32: cMin = -2147483648; cMax = 2147483647; break;
        case tcuTexture.ChannelType.UNSIGNED_INT8: cMin = 0; cMax = 255; break;
        case tcuTexture.ChannelType.UNSIGNED_INT16: cMin = 0; cMax = 65535; break;
        case tcuTexture.ChannelType.UNSIGNED_INT32: cMin = 0; cMax = 4294967295; break;
        case tcuTexture.ChannelType.HALF_FLOAT: cMin = -1e3; cMax = 1e3; break;
        case tcuTexture.ChannelType.FLOAT: cMin = -1e5; cMax = 1e5; break;
        case tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV: cMin = 0; cMax = 1e4; break;
        case tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV: cMin = 0; cMax = 1e5; break;

        default:
            DE_ASSERT(false);
    }

    return [cMin, cMax];
};

/**
 * Creates an array by choosing between 'a' and 'b' based on 'cond' array.
 * @param {Array | number} a
 * @param {Array | number} b
 * @param {Array<boolean>} cond Condtions
 * @return {Array}
 */
tcuTextureUtil.select = function(a, b, cond) {

    /*DE_ASSERT(!(a.length && !b.length)
                || !(!a.length && b.length)
                || !((a.length && b.length) && ((a.length != b.length) || (b.length != cond.length) || (a.length != cond.length))));*/

    if (a.length && !b.length) throw new Error('second input parameter is not a vector');
    if (!a.length && b.length) throw new Error('first input parameter is not a vector');
    if ((a.length && b.length) && ((a.length != b.length) || (b.length != cond.length) || (a.length != cond.length))) throw new Error('different size vectors');

    var dst = [];
    for (var i = 0; i < cond.length; i++)
        if (cond[i]) {
            if (a.length) dst.push(a[i]);
            else dst.push(a);
        } else {
            if (b.length) dst.push(b[i]);
            else dst.push(b);
        }
    return dst;
};

/**
 * Get standard parameters for testing texture format
 *
 * Returns tcuTextureUtil.TextureFormatInfo that describes good parameters for exercising
 * given TextureFormat. Parameters include value ranges per channel and
 * suitable lookup scaling and bias in order to reduce result back to
 * 0..1 range.
 *
 * @param {tcuTexture.TextureFormat} format
 * @return {tcuTextureUtil.TextureFormatInfo}
 */
tcuTextureUtil.getTextureFormatInfo = function(format) {
    // Special cases.
    if (format.isEqual(new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV)))
        return new tcuTextureUtil.TextureFormatInfo([0, 0, 0, 0],
                                 [1023, 1023, 1023, 3],
                                 [1 / 1023, 1 / 1023, 1 / 1023, 1 / 3],
                                 [0, 0, 0, 0]);
    else if (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS)
        return new tcuTextureUtil.TextureFormatInfo([0, 0, 0, 0],
                                 [1, 1, 1, 0],
                                 [1, 1, 1, 1],
                                 [0, 0, 0, 0]); // Depth / stencil formats.
    else if (format.isEqual(new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_5551)))
        return new tcuTextureUtil.TextureFormatInfo([0, 0, 0, 0.5],
                                 [1, 1, 1, 1.5],
                                 [1, 1, 1, 1],
                                 [0, 0, 0, 0]);

    var cRange = tcuTextureUtil.getChannelValueRange(format.type);
    var chnMask = null;

    switch (format.order) {
        case tcuTexture.ChannelOrder.R: chnMask = [true, false, false, false]; break;
        case tcuTexture.ChannelOrder.A: chnMask = [false, false, false, true]; break;
        case tcuTexture.ChannelOrder.L: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.LA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.RG: chnMask = [true, true, false, false]; break;
        case tcuTexture.ChannelOrder.RGB: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.RGBA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.sRGB: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.sRGBA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.D: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.DS: chnMask = [true, true, true, true]; break;
        default:
            DE_ASSERT(false);
    }

    var scale = 1 / (cRange[1] - cRange[0]);
    var bias = -cRange[0] * scale;

    return new tcuTextureUtil.TextureFormatInfo(tcuTextureUtil.select(cRange[0], 0, chnMask),
                             tcuTextureUtil.select(cRange[1], 0, chnMask),
                             tcuTextureUtil.select(scale, 1, chnMask),
                             tcuTextureUtil.select(bias, 0, chnMask));
};

/** tcuTextureUtil.getChannelBitDepth
 * @param {?tcuTexture.ChannelType} channelType
 * @return {Array<number>}
 */
tcuTextureUtil.getChannelBitDepth = function(channelType) {

    switch (channelType) {
        case tcuTexture.ChannelType.SNORM_INT8: return [8, 8, 8, 8];
        case tcuTexture.ChannelType.SNORM_INT16: return [16, 16, 16, 16];
        case tcuTexture.ChannelType.SNORM_INT32: return [32, 32, 32, 32];
        case tcuTexture.ChannelType.UNORM_INT8: return [8, 8, 8, 8];
        case tcuTexture.ChannelType.UNORM_INT16: return [16, 16, 16, 16];
        case tcuTexture.ChannelType.UNORM_INT32: return [32, 32, 32, 32];
        case tcuTexture.ChannelType.UNORM_SHORT_565: return [5, 6, 5, 0];
        case tcuTexture.ChannelType.UNORM_SHORT_4444: return [4, 4, 4, 4];
        case tcuTexture.ChannelType.UNORM_SHORT_555: return [5, 5, 5, 0];
        case tcuTexture.ChannelType.UNORM_SHORT_5551: return [5, 5, 5, 1];
        case tcuTexture.ChannelType.UNORM_INT_101010: return [10, 10, 10, 0];
        case tcuTexture.ChannelType.UNORM_INT_1010102_REV: return [10, 10, 10, 2];
        case tcuTexture.ChannelType.SIGNED_INT8: return [8, 8, 8, 8];
        case tcuTexture.ChannelType.SIGNED_INT16: return [16, 16, 16, 16];
        case tcuTexture.ChannelType.SIGNED_INT32: return [32, 32, 32, 32];
        case tcuTexture.ChannelType.UNSIGNED_INT8: return [8, 8, 8, 8];
        case tcuTexture.ChannelType.UNSIGNED_INT16: return [16, 16, 16, 16];
        case tcuTexture.ChannelType.UNSIGNED_INT32: return [32, 32, 32, 32];
        case tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV: return [10, 10, 10, 2];
        case tcuTexture.ChannelType.UNSIGNED_INT_24_8: return [24, 0, 0, 8];
        case tcuTexture.ChannelType.HALF_FLOAT: return [16, 16, 16, 16];
        case tcuTexture.ChannelType.FLOAT: return [32, 32, 32, 32];
        case tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV: return [11, 11, 10, 0];
        case tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV: return [9, 9, 9, 0];
        case tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV: return [32, 0, 0, 8];
        default:
            DE_ASSERT(false);
            return [0, 0, 0, 0];
    }
};

/** tcuTextureUtil.getTextureFormatBitDepth
 * @param {tcuTexture.TextureFormat} format
 * @return {Array<number>}
 */
tcuTextureUtil.getTextureFormatBitDepth = function(format) {

    /** @type {Array<number>} */ var chnBits = tcuTextureUtil.getChannelBitDepth(format.type); // IVec4
    /** @type {Array<boolean>} */ var chnMask = [false, false, false, false]; // BVec4
    /** @type {Array<number>} */ var chnSwz = [0, 1, 2, 3]; // IVec4

    switch (format.order) {
        case tcuTexture.ChannelOrder.R: chnMask = [true, false, false, false]; break;
        case tcuTexture.ChannelOrder.A: chnMask = [false, false, false, true]; break;
        case tcuTexture.ChannelOrder.RA: chnMask = [true, false, false, true]; break;
        case tcuTexture.ChannelOrder.L: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.I: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.LA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.RG: chnMask = [true, true, false, false]; break;
        case tcuTexture.ChannelOrder.RGB: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.RGBA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.BGRA: chnMask = [true, true, true, true]; chnSwz = [2, 1, 0, 3]; break;
        case tcuTexture.ChannelOrder.ARGB: chnMask = [true, true, true, true]; chnSwz = [1, 2, 3, 0]; break;
        case tcuTexture.ChannelOrder.sRGB: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.sRGBA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.D: chnMask = [true, false, false, false]; break;
        case tcuTexture.ChannelOrder.DS: chnMask = [true, false, false, true]; break;
        case tcuTexture.ChannelOrder.S: chnMask = [false, false, false, true]; break;
        default:
            DE_ASSERT(false);
    }

    return tcuTextureUtil.select(deMath.swizzle(chnBits, [chnSwz[0], chnSwz[1], chnSwz[2], chnSwz[3]]), [0, 0, 0, 0], chnMask);

};

/** tcuTextureUtil.fillWithGrid
 * @const @param {tcuTexture.PixelBufferAccess} access
 * @param {number} cellSize
 * @param {Array<number>} colorA
 * @param {Array<number>} colorB
 */
tcuTextureUtil.fillWithGrid = function(access, cellSize, colorA, colorB) {
    if (access.getHeight() == 1 && access.getDepth() == 1)
        tcuTextureUtil.fillWithGrid1D(access, cellSize, colorA, colorB);
    else if (access.getDepth() == 1)
        tcuTextureUtil.fillWithGrid2D(access, cellSize, colorA, colorB);
    else
        tcuTextureUtil.fillWithGrid3D(access, cellSize, colorA, colorB);
};

/** tcuTextureUtil.fillWithGrid1D
 * @const @param {tcuTexture.PixelBufferAccess} access
 * @param {number} cellSize
 * @param {Array<number>} colorA
 * @param {Array<number>} colorB
 */
tcuTextureUtil.fillWithGrid1D = function(access, cellSize, colorA, colorB) {
    for (var x = 0; x < access.getWidth(); x++) {
        var mx = Math.floor(x / cellSize) % 2;

        if (mx)
            access.setPixel(colorB, x, 0);
        else
            access.setPixel(colorA, x, 0);
    }
};

/** tcuTextureUtil.fillWithGrid2D
 * @const @param {tcuTexture.PixelBufferAccess} access
 * @param {number} cellSize
 * @param {Array<number>} colorA
 * @param {Array<number>} colorB
 */
tcuTextureUtil.fillWithGrid2D = function(access, cellSize, colorA, colorB) {
    for (var y = 0; y < access.getHeight(); y++)
        for (var x = 0; x < access.getWidth(); x++) {
            var mx = Math.floor(x / cellSize) % 2;
            var my = Math.floor(y / cellSize) % 2;

            if (mx ^ my)
                access.setPixel(colorB, x, y);
            else
                access.setPixel(colorA, x, y);
        }
};

/** tcuTextureUtil.fillWithGrid3D
 * @const @param {tcuTexture.PixelBufferAccess} access
 * @param {number} cellSize
 * @param {Array<number>} colorA
 * @param {Array<number>} colorB
 */
tcuTextureUtil.fillWithGrid3D = function(access, cellSize, colorA, colorB) {
    for (var z = 0; z < access.getDepth(); z++)
        for (var y = 0; y < access.getHeight(); y++)
            for (var x = 0; x < access.getWidth(); x++) {
                var mx = Math.floor(x / cellSize) % 2;
                var my = Math.floor(y / cellSize) % 2;
                var mz = Math.floor(z / cellSize) % 2;

                if (mx ^ my ^ mz)
                    access.setPixel(colorB, x, y, z);
                else
                    access.setPixel(colorA, x, y, z);
            }
};

/**
 * @const @param {tcuTexture.TextureFormat} format
 * @return {Array<number>}
 */
tcuTextureUtil.getTextureFormatMantissaBitDepth = function(format) {
    /** @type {Array<number>} */ var chnBits = tcuTextureUtil.getChannelMantissaBitDepth(format.type);
    /** @type {Array<boolean>} */ var chnMask = [false, false, false, false];
    /** @type {Array<number>} */ var chnSwz = [0, 1, 2, 3];

    switch (format.order) {
        case tcuTexture.ChannelOrder.R: chnMask = [true, false, false, false]; break;
        case tcuTexture.ChannelOrder.A: chnMask = [false, false, false, true]; break;
        case tcuTexture.ChannelOrder.RA: chnMask = [true, false, false, true]; break;
        case tcuTexture.ChannelOrder.L: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.I: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.LA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.RG: chnMask = [true, true, false, false]; break;
        case tcuTexture.ChannelOrder.RGB: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.RGBA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.BGRA: chnMask = [true, true, true, true]; chnSwz = [2, 1, 0, 3]; break;
        case tcuTexture.ChannelOrder.ARGB: chnMask = [true, true, true, true]; chnSwz = [1, 2, 3, 0]; break;
        case tcuTexture.ChannelOrder.sRGB: chnMask = [true, true, true, false]; break;
        case tcuTexture.ChannelOrder.sRGBA: chnMask = [true, true, true, true]; break;
        case tcuTexture.ChannelOrder.D: chnMask = [true, false, false, false]; break;
        case tcuTexture.ChannelOrder.DS: chnMask = [true, false, false, true]; break;
        case tcuTexture.ChannelOrder.S: chnMask = [false, false, false, true]; break;
        default:
            DE_ASSERT(false);
    }
    return tcuTextureUtil.select(deMath.swizzle(chnBits, [chnSwz[0], chnSwz[1], chnSwz[2], chnSwz[3]]), [0, 0, 0, 0], chnMask);
};

/**
 * @param {?tcuTexture.ChannelType} channelType
 * @return {Array<number>}
 */
tcuTextureUtil.getChannelMantissaBitDepth = function(channelType) {
    switch (channelType) {
        case tcuTexture.ChannelType.SNORM_INT8:
        case tcuTexture.ChannelType.SNORM_INT16:
        case tcuTexture.ChannelType.SNORM_INT32:
        case tcuTexture.ChannelType.UNORM_INT8:
        case tcuTexture.ChannelType.UNORM_INT16:
        case tcuTexture.ChannelType.UNORM_INT32:
        case tcuTexture.ChannelType.UNORM_SHORT_565:
        case tcuTexture.ChannelType.UNORM_SHORT_4444:
        case tcuTexture.ChannelType.UNORM_SHORT_555:
        case tcuTexture.ChannelType.UNORM_SHORT_5551:
        case tcuTexture.ChannelType.UNORM_INT_101010:
        case tcuTexture.ChannelType.UNORM_INT_1010102_REV:
        case tcuTexture.ChannelType.SIGNED_INT8:
        case tcuTexture.ChannelType.SIGNED_INT16:
        case tcuTexture.ChannelType.SIGNED_INT32:
        case tcuTexture.ChannelType.UNSIGNED_INT8:
        case tcuTexture.ChannelType.UNSIGNED_INT16:
        case tcuTexture.ChannelType.UNSIGNED_INT32:
        case tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV:
        case tcuTexture.ChannelType.UNSIGNED_INT_24_8:
        case tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV:
            return tcuTextureUtil.getChannelBitDepth(channelType);
        case tcuTexture.ChannelType.HALF_FLOAT: return [10, 10, 10, 10];
        case tcuTexture.ChannelType.FLOAT: return [23, 23, 23, 23];
        case tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV: return [6, 6, 5, 0];
        case tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV: return [23, 0, 0, 8];
        default:
            throw new Error('Invalid channelType: ' + channelType);
    }
};

/**
 * @param {tcuTexture.PixelBufferAccess} dst
 * @param {tcuTexture.ConstPixelBufferAccess} src
 */
tcuTextureUtil.copy = function(dst, src) {
    var width = dst.getWidth();
    var height = dst.getHeight();
    var depth = dst.getDepth();

    DE_ASSERT(src.getWidth() == width && src.getHeight() == height && src.getDepth() == depth);

    if (src.getFormat().isEqual(dst.getFormat())) {
        var srcData = src.getDataPtr();
        var dstData = dst.getDataPtr();

        if (srcData.length == dstData.length) {
            dstData.set(srcData);
            return;
        }
    }
    var srcClass = tcuTexture.getTextureChannelClass(src.getFormat().type);
    var dstClass = tcuTexture.getTextureChannelClass(dst.getFormat().type);
    var srcIsInt = srcClass == tcuTexture.TextureChannelClass.SIGNED_INTEGER || srcClass == tcuTexture.TextureChannelClass.UNSIGNED_INTEGER;
    var dstIsInt = dstClass == tcuTexture.TextureChannelClass.SIGNED_INTEGER || dstClass == tcuTexture.TextureChannelClass.UNSIGNED_INTEGER;

    if (srcIsInt && dstIsInt) {
        for (var z = 0; z < depth; z++)
        for (var y = 0; y < height; y++)
        for (var x = 0; x < width; x++)
            dst.setPixelInt(src.getPixelInt(x, y, z), x, y, z);
    } else {
        for (var z = 0; z < depth; z++)
        for (var y = 0; y < height; y++)
        for (var x = 0; x < width; x++)
            dst.setPixel(src.getPixel(x, y, z), x, y, z);
    }
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} access
 */
tcuTextureUtil.estimatePixelValueRange = function(access) {
    var format = access.getFormat();

    switch (format.type) {
        case tcuTexture.ChannelType.UNORM_INT8:
        case tcuTexture.ChannelType.UNORM_INT16:
            // Normalized unsigned formats.
            return [
                [0, 0, 0, 0],
                [1, 1, 1, 1]
            ];

        case tcuTexture.ChannelType.SNORM_INT8:
        case tcuTexture.ChannelType.SNORM_INT16:
            // Normalized signed formats.
            return [
                [-1, -1, -1, -1],
                [1, 1, 1, 1]
            ];

        default:
            // \note Samples every 4/8th pixel.
            var minVal = [Infinity, Infinity, Infinity, Infinity];
            var maxVal = [-Infinity, -Infinity, -Infinity, -Infinity];

            for (var z = 0; z < access.getDepth(); z += 2) {
                for (var y = 0; y < access.getHeight(); y += 2) {
                    for (var x = 0; x < access.getWidth(); x += 2) {
                        var p = access.getPixel(x, y, z);

                        minVal[0] = Math.min(minVal[0], p[0]);
                        minVal[1] = Math.min(minVal[1], p[1]);
                        minVal[2] = Math.min(minVal[2], p[2]);
                        minVal[3] = Math.min(minVal[3], p[3]);

                        maxVal[0] = Math.max(maxVal[0], p[0]);
                        maxVal[1] = Math.max(maxVal[1], p[1]);
                        maxVal[2] = Math.max(maxVal[2], p[2]);
                        maxVal[3] = Math.max(maxVal[3], p[3]);
                    }
                }
            }
            return [minVal, maxVal];
    }
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} access
 * @return {{scale: Array<number>, bias: Array<number>}}
 */
tcuTextureUtil.computePixelScaleBias = function(access) {
    var limits = tcuTextureUtil.estimatePixelValueRange(access);
    var minVal = limits[0];
    var maxVal = limits[1];

    var scale = [1, 1, 1, 1];
    var bias = [0, 0, 0, 0];

    var eps = 0.0001;

    for (var c = 0; c < 4; c++) {
        if (maxVal[c] - minVal[c] < eps) {
            scale[c] = (maxVal[c] < eps) ? 1 : (1 / maxVal[c]);
            bias[c] = (c == 3) ? (1 - maxVal[c] * scale[c]) : (0 - minVal[c] * scale[c]);
        } else {
            scale[c] = 1 / (maxVal[c] - minVal[c]);
            bias[c] = 0 - minVal[c] * scale[c];
        }
    }

    return {
        scale: scale,
        bias: bias
    };
};

});

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
goog.provide('framework.common.tcuFuzzyImageCompare');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');

goog.scope(function() {

var tcuFuzzyImageCompare = framework.common.tcuFuzzyImageCompare;
var deMath = framework.delibs.debase.deMath;
var deRandom = framework.delibs.debase.deRandom;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * tcuFuzzyImageCompare.FuzzyCompareParams struct
     * @constructor
     * @param {number=} maxSampleSkip_
     * @param {number=} minErrThreshold_
     * @param {number=} errExp_
     */
    tcuFuzzyImageCompare.FuzzyCompareParams = function(maxSampleSkip_, minErrThreshold_, errExp_) {
        /** @type {number} */ this.maxSampleSkip = maxSampleSkip_ === undefined ? 8 : maxSampleSkip_;
        /** @type {number} */ this.minErrThreshold = minErrThreshold_ === undefined ? 4 : minErrThreshold_;
        /** @type {number} */ this.errExp = errExp_ === undefined ? 4.0 : errExp_;
    };

    /**
     * @param {Array<number>} v
     * @return {Array<number>}
     */
    tcuFuzzyImageCompare.roundArray4ToUint8Sat = function(v) {
        return [
            deMath.clamp(Math.trunc(v[0] + 0.5), 0, 255),
            deMath.clamp(Math.trunc(v[1] + 0.5), 0, 255),
            deMath.clamp(Math.trunc(v[2] + 0.5), 0, 255),
            deMath.clamp(Math.trunc(v[3] + 0.5), 0, 255)
        ];
    };

    /**
     * @param {Array<number>} pa
     * @param {Array<number>} pb
     * @param {number} minErrThreshold
     * @return {number}
     */
    tcuFuzzyImageCompare.compareColors = function(pa, pb, minErrThreshold) {
        /** @type {number}*/ var r = Math.max(Math.abs(pa[0] - pb[0]) - minErrThreshold, 0);
        /** @type {number}*/ var g = Math.max(Math.abs(pa[1] - pb[1]) - minErrThreshold, 0);
        /** @type {number}*/ var b = Math.max(Math.abs(pa[2] - pb[2]) - minErrThreshold, 0);
        /** @type {number}*/ var a = Math.max(Math.abs(pa[3] - pb[3]) - minErrThreshold, 0);

        /** @type {number}*/ var scale = 1.0 / (255 - minErrThreshold);
        /** @type {number}*/ var sqSum = (r * r + g * g + b * b + a * a) * (scale * scale);

        return Math.sqrt(sqSum);
    };

    /**
     * @param {tcuTexture.RGBA8View} src
     * @param {number} u
     * @param {number} v
     * @param {number} NumChannels
     * @return {Array<number>}
     */
    tcuFuzzyImageCompare.bilinearSample = function(src, u, v, NumChannels) {
        /** @type {number}*/ var w = src.width;
        /** @type {number}*/ var h = src.height;

        /** @type {number}*/ var x0 = Math.floor(u - 0.5);
        /** @type {number}*/ var x1 = x0 + 1;
        /** @type {number}*/ var y0 = Math.floor(v - 0.5);
        /** @type {number}*/ var y1 = y0 + 1;

        /** @type {number}*/ var i0 = deMath.clamp(x0, 0, w - 1);
        /** @type {number}*/ var i1 = deMath.clamp(x1, 0, w - 1);
        /** @type {number}*/ var j0 = deMath.clamp(y0, 0, h - 1);
        /** @type {number}*/ var j1 = deMath.clamp(y1, 0, h - 1);

        /** @type {number}*/ var a = (u - 0.5) - Math.floor(u - 0.5);
        /** @type {number}*/ var b = (u - 0.5) - Math.floor(u - 0.5);

        /** @type {Array<number>} */ var p00 = src.read(i0, j0, NumChannels);
        /** @type {Array<number>} */ var p10 = src.read(i1, j0, NumChannels);
        /** @type {Array<number>} */ var p01 = src.read(i0, j1, NumChannels);
        /** @type {Array<number>} */ var p11 = src.read(i1, j1, NumChannels);
        /** @type {number} */ var dst = 0;

        // Interpolate.
        /** @type {Array<number>}*/ var f = [];
        for (var c = 0; c < NumChannels; c++) {
                f[c] = p00[c] * (1.0 - a) * (1.0 - b) +
                (p10[c] * a * (1.0 - b)) +
                (p01[c] * (1.0 - a) * b) +
                (p11[c] * a * b);
        }

        return tcuFuzzyImageCompare.roundArray4ToUint8Sat(f);
    };

    /**
     * @param {tcuTexture.RGBA8View} dst
     * @param {tcuTexture.RGBA8View} src
     * @param {number} shiftX
     * @param {number} shiftY
     * @param {Array<number>} kernelX
     * @param {Array<number>} kernelY
     * @param {number} DstChannels
     * @param {number} SrcChannels
     */
    tcuFuzzyImageCompare.separableConvolve = function(dst, src, shiftX, shiftY, kernelX, kernelY, DstChannels, SrcChannels) {
        DE_ASSERT(dst.width == src.width && dst.height == src.height);

        /** @type {tcuTexture.TextureLevel} */ var tmp = new tcuTexture.TextureLevel(dst.getFormat(), dst.height, dst.width);
        var tmpView = new tcuTexture.RGBA8View(tmp.getAccess());

        /** @type {number} */ var kw = kernelX.length;
        /** @type {number} */ var kh = kernelY.length;

        /** @type {Array<number>} */ var sum = [];
        /** @type {number} */ var f;
        /** @type {Array<number>} */ var p;

        // Horizontal pass
        // \note Temporary surface is written in column-wise order
        for (var j = 0; j < src.height; j++) {
            for (var i = 0; i < src.width; i++) {
                sum[0] = sum[1] = sum[2] = sum[3] = 0;
                for (var kx = 0; kx < kw; kx++) {
                    f = kernelX[kw - kx - 1];
                    p = src.read(deMath.clamp(i + kx - shiftX, 0, src.width - 1), j, SrcChannels);
                    sum = deMath.add(sum, deMath.scale(p, f));
                }

                sum = tcuFuzzyImageCompare.roundArray4ToUint8Sat(sum);
                tmpView.write(j, i, sum, DstChannels);
            }
        }

        // Vertical pass
        for (var j = 0; j < src.height; j++) {
            for (var i = 0; i < src.width; i++) {
                sum[0] = sum[1] = sum[2] = sum[3] = 0;
                for (var ky = 0; ky < kh; ky++) {
                    f = kernelY[kh - ky - 1];
                    p = tmpView.read(deMath.clamp(j + ky - shiftY, 0, tmpView.width - 1), i, DstChannels);
                    sum = deMath.add(sum, deMath.scale(p, f));
                }

                sum = tcuFuzzyImageCompare.roundArray4ToUint8Sat(sum);
                dst.write(i, j, sum, DstChannels);
            }
        }
    };

    /**
     * @param {tcuFuzzyImageCompare.FuzzyCompareParams} params
     * @param {deRandom.Random} rnd
     * @param {Array<number>} pixel
     * @param {tcuTexture.RGBA8View} surface
     * @param {number} x
     * @param {number} y
     * @param {number} NumChannels
     * @return {number}
     */
    tcuFuzzyImageCompare.compareToNeighbor = function(params, rnd, pixel, surface, x, y, NumChannels) {
        /** @type {number} */ var minErr = 100;

        // (x, y) + (0, 0)
        minErr = Math.min(minErr, tcuFuzzyImageCompare.compareColors(pixel, surface.read(x, y, NumChannels), params.minErrThreshold));
        if (minErr == 0.0)
            return minErr;

        // Area around (x, y)
        /** @type {Array<Array.<number>>} */ var s_coords =
        [
            [-1, -1],
            [0, -1],
            [1, -1],
            [-1, 0],
            [1, 0],
            [-1, 1],
            [0, 1],
            [1, 1]
        ];

        /** @type {number} */ var dx;
        /** @type {number} */ var dy;

        for (var d = 0; d < s_coords.length; d++) {
            dx = x + s_coords[d][0];
            dy = y + s_coords[d][1];

            if (!deMath.deInBounds32(dx, 0, surface.width) || !deMath.deInBounds32(dy, 0, surface.height))
                continue;

            minErr = Math.min(minErr, tcuFuzzyImageCompare.compareColors(pixel, surface.read(dx, dy, NumChannels), params.minErrThreshold));
            if (minErr == 0.0)
                return minErr;
        }

        // Random bilinear-interpolated samples around (x, y)
        for (var s = 0; s < 32; s++) {
            dx = x + rnd.getFloat() * 2.0 - 0.5;
            dy = y + rnd.getFloat() * 2.0 - 0.5;

            /** @type {Array<number>} */ var sample = tcuFuzzyImageCompare.bilinearSample(surface, dx, dy, NumChannels);

            minErr = Math.min(minErr, tcuFuzzyImageCompare.compareColors(pixel, sample, params.minErrThreshold));
            if (minErr == 0.0)
                return minErr;
        }

        return minErr;
    };

    /**
     * @param {Array<number>} c
     * @return {number}
     */
    tcuFuzzyImageCompare.toGrayscale = function(c) {
        return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    };

    /**
     * @param {tcuTexture.TextureFormat} format
     * @return {boolean}
     */
    tcuFuzzyImageCompare.isFormatSupported = function(format) {
        return format.type == tcuTexture.ChannelType.UNORM_INT8 && (format.order == tcuTexture.ChannelOrder.RGB || format.order == tcuTexture.ChannelOrder.RGBA);
    };

    /**
     * @param {tcuFuzzyImageCompare.FuzzyCompareParams} params
     * @param {tcuTexture.ConstPixelBufferAccess} ref
     * @param {tcuTexture.ConstPixelBufferAccess} cmp
     * @param {tcuTexture.PixelBufferAccess} errorMask
     * @return {number}
     */
    tcuFuzzyImageCompare.fuzzyCompare = function(params, ref, cmp, errorMask) {
        assertMsgOptions(ref.getWidth() == cmp.getWidth() && ref.getHeight() == cmp.getHeight(),
            'Reference and result images have different dimensions', false, true);

        assertMsgOptions(ref.getWidth() == errorMask.getWidth() && ref.getHeight() == errorMask.getHeight(),
            'Reference and error mask images have different dimensions', false, true);

        if (!tcuFuzzyImageCompare.isFormatSupported(ref.getFormat()) || !tcuFuzzyImageCompare.isFormatSupported(cmp.getFormat()))
            throw new Error('Unsupported format in fuzzy comparison');

        /** @type {number} */ var width = ref.getWidth();
        /** @type {number} */ var height = ref.getHeight();
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(667);

        // Filtered
        /** @type {tcuTexture.TextureLevel} */ var refFiltered = new tcuTexture.TextureLevel(new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8), width, height);
        /** @type {tcuTexture.TextureLevel} */ var cmpFiltered = new tcuTexture.TextureLevel(new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8), width, height);

        var refView = new tcuTexture.RGBA8View(ref);
        var cmpView = new tcuTexture.RGBA8View(cmp);
        var refFilteredView = new tcuTexture.RGBA8View(tcuTexture.PixelBufferAccess.newFromTextureLevel(refFiltered));
        var cmpFilteredView = new tcuTexture.RGBA8View(tcuTexture.PixelBufferAccess.newFromTextureLevel(cmpFiltered));

        // Kernel = {0.15, 0.7, 0.15}
        /** @type {Array<number>} */ var kernel = [0.1, 0.8, 0.1];
        /** @type {number} */ var shift = Math.floor((kernel.length - 1) / 2);

        switch (ref.getFormat().order) {
            case tcuTexture.ChannelOrder.RGBA: tcuFuzzyImageCompare.separableConvolve(refFilteredView, refView, shift, shift, kernel, kernel, 4, 4); break;
            case tcuTexture.ChannelOrder.RGB: tcuFuzzyImageCompare.separableConvolve(refFilteredView, refView, shift, shift, kernel, kernel, 4, 3); break;
            default:
                throw new Error('tcuFuzzyImageCompare.fuzzyCompare - Invalid ChannelOrder');
        }

        switch (cmp.getFormat().order) {
            case tcuTexture.ChannelOrder.RGBA: tcuFuzzyImageCompare.separableConvolve(cmpFilteredView, cmpView, shift, shift, kernel, kernel, 4, 4); break;
            case tcuTexture.ChannelOrder.RGB: tcuFuzzyImageCompare.separableConvolve(cmpFilteredView, cmpView, shift, shift, kernel, kernel, 4, 3); break;
            default:
                throw new Error('tcuFuzzyImageCompare.fuzzyCompare - Invalid ChannelOrder');
        }

        /** @type {number} */ var numSamples = 0;
        /** @type {number} */ var errSum = 0.0;

        // Clear error mask to green.
        errorMask.clear([0.0, 1.0, 0.0, 1.0]);

        for (var y = 1; y < height - 1; y++) {
            for (var x = 1; x < width - 1; x += params.maxSampleSkip > 0 ? rnd.getInt(0, params.maxSampleSkip) : 1) {
                /** @type {number} */ var err = Math.min(tcuFuzzyImageCompare.compareToNeighbor(params, rnd, refFilteredView.read(x, y, 4), cmpFilteredView, x, y, 4),
                                       tcuFuzzyImageCompare.compareToNeighbor(params, rnd, cmpFilteredView.read(x, y, 4), refFilteredView, x, y, 4));

                err = Math.pow(err, params.errExp);

                errSum += err;
                numSamples += 1;

                // Build error image.
                /** @type {number} */ var red = err * 500.0;
                /** @type {number} */ var luma = tcuFuzzyImageCompare.toGrayscale(cmp.getPixel(x, y));
                /** @type {number} */ var rF = 0.7 + 0.3 * luma;
                errorMask.setPixel([red * rF, (1.0 - red) * rF, 0.0, 1.0], x, y);

            }
        }

        // Scale error sum based on number of samples taken
        errSum *= ((width - 2) * (height - 2)) / numSamples;

        return errSum;
    };

});

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
goog.provide('framework.referencerenderer.rrMultisamplePixelBufferAccess');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var rrMultisamplePixelBufferAccess = framework.referencerenderer.rrMultisamplePixelBufferAccess;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var tcuTextureUtil = framework.common.tcuTextureUtil;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

/**
 * \brief Read-write pixel data access to multisampled buffers.
 *
 * Multisampled data access follows the multisampled indexing convention.
 *
 * Prevents accidental usage of non-multisampled buffer as multisampled
 * with PixelBufferAccess.
 * @constructor
 * @param {tcuTexture.PixelBufferAccess=} rawAccess
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess = function(rawAccess) {
    this.m_access = rawAccess || new tcuTexture.PixelBufferAccess({
                                            width: 0,
                                            height: 0});
};

/**
 * @return {tcuTexture.PixelBufferAccess}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.raw = function() { return this.m_access; };

/**
 * @return {boolean}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.isEmpty = function() { return this.m_access.isEmpty(); };

/**
 * @return {number}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.getNumSamples = function() { return this.raw().getWidth(); };

/**
 * @return {tcuTexture.PixelBufferAccess}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.toSinglesampleAccess = function() {
    DE_ASSERT(this.getNumSamples() == 1);

    return new tcuTexture.PixelBufferAccess({
                                  format: this.m_access.getFormat(),
                                  width: this.m_access.getHeight(),
                                  height: this.m_access.getDepth(),
                                  depth: 1,
                                  rowPitch: this.m_access.getSlicePitch(),
                                  slicePitch: this.m_access.getSlicePitch() * this.m_access.getDepth(),
                                  data: this.m_access.m_data,
                                  offset: this.m_access.m_offset});
};

/**
 * @param {tcuTexture.PixelBufferAccess} original
 * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess = function(original) {
    return new rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess(
                new tcuTexture.PixelBufferAccess({
                                format: original.getFormat(),
                                width: 1,
                                height: original.getWidth(),
                                depth: original.getHeight(),
                                rowPitch: original.getFormat().getPixelSize(),
                                slicePitch: original.getRowPitch(),
                                data: original.m_data,
                                offset: original.m_offset}));
};

/**
 * @param {tcuTexture.PixelBufferAccess} multisampledAccess
 * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromMultisampleAccess = function(multisampledAccess) {
    return new rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess(multisampledAccess);
};

/**
 * @param {Array<number>} region
 * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.getSubregion = function(region) {
    var x = region[0];
    var y = region[1];
    var width = region[2];
    var height = region[3];

    return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromMultisampleAccess(tcuTextureUtil.getSubregion(this.raw(), 0, x, y, this.getNumSamples(), width, height));
};

/**
 * @return {Array<number>} [x, y, width, height]
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.getBufferSize = function() {
    return [0, 0, this.raw().getHeight(), this.raw().getDepth()];
};

/**
 * @param {tcuTexture.PixelBufferAccess} dst
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.resolveMultisampleColorBuffer = function(dst) {
    var src = this;
    DE_ASSERT(dst.getWidth() == src.raw().getHeight());
    DE_ASSERT(dst.getHeight() == src.raw().getDepth());

    var numSamples = src.getNumSamples();
    var sum = [0, 0, 0, 0];
    for (var y = 0; y < dst.getHeight(); y++) {
        for (var x = 0; x < dst.getWidth(); x++) {
            sum[0] = 0;
            sum[1] = 0;
            sum[2] = 0;
            sum[3] = 0;

            for (var s = 0; s < src.raw().getWidth(); s++) {
                var pixel = src.raw().getPixel(s, x, y);
                sum[0] += pixel[0];
                sum[1] += pixel[1];
                sum[2] += pixel[2];
                sum[3] += pixel[3];
            }

            sum[0] /= numSamples;
            sum[1] /= numSamples;
            sum[2] /= numSamples;
            sum[3] /= numSamples;

            dst.setPixel(sum, x, y);
        }
    }
};

/**
 * @param {number} x
 * @param {number} y
 * @return {Array<number>}
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.resolveMultisamplePixel = function(x, y) {
    var sum = [0, 0, 0, 0];
    for (var s = 0; s < this.getNumSamples(); s++)
        sum = deMath.add(sum, this.raw().getPixel(s, x, y));

    for (var i = 0; i < sum.length; i++)
        sum[i] = sum[i] / this.getNumSamples();

    return sum;
};

/**
 * @param {Array<number>} color
 */
rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.prototype.clear = function(color) {
    this.raw().clear(color);
};

});

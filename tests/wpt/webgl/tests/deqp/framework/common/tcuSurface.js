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
goog.provide('framework.common.tcuSurface');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

var tcuSurface = framework.common.tcuSurface;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var gluTextureUtil = framework.opengl.gluTextureUtil;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

/**
 * \brief RGBA8888 surface
 *
 * tcuSurface.Surface provides basic pixel storage functionality. Only single format
 * (RGBA8888) is supported.
 *
 * PixelBufferAccess (see tcuTexture.h) provides much more flexible API
 * for handling various pixel formats. This is mainly a convinience class.
 * @constructor
 * @param {number=} width
 * @param {number=} height
 */
tcuSurface.Surface = function(width, height) {
    width = width || 0;
    height = height || 0;
    this.setSize(width, height);
};

/**
 * @param {number} width
 * @param {number} height
 */
tcuSurface.Surface.prototype.setSize = function(width, height) {
    this.m_width = width;
    this.m_height = height;
    if (width * height > 0) {
        this.m_data = new ArrayBuffer(4 * width * height);
        this.m_pixels = new Uint8Array(this.m_data);
    } else {
        this.m_data = null;
        this.m_pixels = null;
    }
};

/**
 * @return {number}
 */
tcuSurface.Surface.prototype.getWidth = function() { return this.m_width; };

/**
 * @return {number}
 */
tcuSurface.Surface.prototype.getHeight = function() { return this.m_height; };

/**
 * @param {number} x
 * @param {number} y
 * @param {Array<number>} color Vec4 color
 */
tcuSurface.Surface.prototype.setPixel = function(x, y, color) {
    DE_ASSERT(deMath.deInBounds32(x, 0, this.m_width));
    DE_ASSERT(deMath.deInBounds32(y, 0, this.m_height));

    var offset = 4 * (x + y * this.m_width);
    for (var i = 0; i < 4; i++)
        this.m_pixels[offset + i] = color[i];
};

/**
 * @param {number} x
 * @param {number} y
 * @return {Array<number>}
 */
tcuSurface.Surface.prototype.getPixel = function(x, y) {
    DE_ASSERT(deMath.deInBounds32(x, 0, this.m_width));
    DE_ASSERT(deMath.deInBounds32(y, 0, this.m_height));

    var color = [];
    color.length = 4;

    var offset = 4 * (x + y * this.m_width);
    for (var i = 0; i < 4; i++)
        color[i] = this.m_pixels[offset + i];

    return color;
};

/**
 * @param {number} x
 * @param {number} y
 * @return {number}
 */
tcuSurface.Surface.prototype.getPixelUintRGB8 = function(x, y) {
    DE_ASSERT(deMath.deInBounds32(x, 0, this.m_width));
    DE_ASSERT(deMath.deInBounds32(y, 0, this.m_height));

    var offset = 4 * (x + y * this.m_width);
    return (this.m_pixels[offset] << 16) +
        (this.m_pixels[offset + 1] << 8) +
        this.m_pixels[offset + 2];
};

/**
 * Read the viewport contents into this surface
 * @param {*=} ctx WebGL or ref context
 * @param {(Array<number>|{x: number, y: number, width: number, height: number})=} view
 */
tcuSurface.Surface.prototype.readViewport = function(ctx, view) {
    ctx = ctx || gl;
    var v = view || /** @type {Array<number>} */ (ctx.getParameter(gl.VIEWPORT));
    /** @type {number} */ var x;
    /** @type {number} */ var y;
    /** @type {number} */ var width;
    /** @type {number} */ var height;
    if (v instanceof Array || ArrayBuffer.isView(v)) {
        x = v[0];
        y = v[1];
        width = v[2];
        height = v[3];
    } else {
        x = v.x;
        y = v.y;
        width = v.width;
        height = v.height;
    }
    if (width != this.m_width || height != this.m_height)
        this.setSize(width, height);
    ctx.readPixels(x, y, width, height, gl.RGBA, gl.UNSIGNED_BYTE, this.m_pixels);
};

/**
 * @return {Uint8Array}
 */
tcuSurface.Surface.prototype.getPixels = function() { return this.m_pixels || null; };

/**
 * @return {tcuTexture.PixelBufferAccess} Pixel Buffer Access object
 */
tcuSurface.Surface.prototype.getAccess = function() {
    return new tcuTexture.PixelBufferAccess({
                    format: new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8),
                    width: this.m_width,
                    height: this.m_height,
                    data: this.m_data
                });

};

tcuSurface.Surface.prototype.getSubAccess = function(x, y, width, height) {
    /* TODO: Implement. the deqp getSubAccess() looks broken. It will probably fail if
     * x != 0 or width != m_width
     */
     throw new Error('Unimplemented');
};

});

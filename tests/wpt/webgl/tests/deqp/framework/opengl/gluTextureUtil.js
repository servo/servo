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

/*--------------------------------------------------------------------*//*!
 * \brief Map tcuTexture.TextureFormat to GL pixel transfer format.
 *
 * Maps generic texture format description to GL pixel transfer format.
 * If no mapping is found, throws tcu::InternalError.
 *
 * \param texFormat Generic texture format.
 * \return GL pixel transfer format.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('framework.opengl.gluTextureUtil');
goog.require('framework.common.tcuCompressedTexture');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {

var gluTextureUtil = framework.opengl.gluTextureUtil;
var deString = framework.delibs.debase.deString;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuCompressedTexture = framework.common.tcuCompressedTexture;
var gluShaderUtil = framework.opengl.gluShaderUtil;

/**
 * @param {number} format
 * @param {number} dataType
 * @constructor
 */
gluTextureUtil.TransferFormat = function(format, dataType) {
    this.format = format; //!< Pixel format.
    this.dataType = dataType; //!< Data type.
};

/**
 * Map tcuTexture.TextureFormat to GL pixel transfer format.
 *
 * Maps generic texture format description to GL pixel transfer format.
 * If no mapping is found, throws tcu::InternalError.
 *
 * @param {tcuTexture.TextureFormat} texFormat Generic texture format.
 * @return {gluTextureUtil.TransferFormat} GL pixel transfer format.
 * @throws {Error}
 */
gluTextureUtil.getTransferFormat = function(/* tcuTexture.TextureFormat */ texFormat) {
    var format = gl.NONE;
    var type = gl.NONE;
    /*boolean*/ var isInt = false;

    switch (texFormat.type) {
        case tcuTexture.ChannelType.SIGNED_INT8:
        case tcuTexture.ChannelType.SIGNED_INT16:
        case tcuTexture.ChannelType.SIGNED_INT32:
        case tcuTexture.ChannelType.UNSIGNED_INT8:
        case tcuTexture.ChannelType.UNSIGNED_INT16:
        case tcuTexture.ChannelType.UNSIGNED_INT32:
        case tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV:
            isInt = true;
            break;

        default:
            isInt = false;
            break;
    }

    switch (texFormat.order) {
        case tcuTexture.ChannelOrder.A: format = gl.ALPHA; break;
        case tcuTexture.ChannelOrder.L: format = gl.LUMINANCE; break;
        case tcuTexture.ChannelOrder.LA: format = gl.LUMINANCE_ALPHA; break;
        case tcuTexture.ChannelOrder.R: format = isInt ? gl.RED_INTEGER : gl.RED; break;
        case tcuTexture.ChannelOrder.RG: format = isInt ? gl.RG_INTEGER : gl.RG; break;
        case tcuTexture.ChannelOrder.RGB: format = isInt ? gl.RGB_INTEGER : gl.RGB; break;
        case tcuTexture.ChannelOrder.RGBA: format = isInt ? gl.RGBA_INTEGER : gl.RGBA; break;
        case tcuTexture.ChannelOrder.sRGB: format = gl.RGB; break;
        case tcuTexture.ChannelOrder.sRGBA: format = gl.RGBA; break;
        case tcuTexture.ChannelOrder.D: format = gl.DEPTH_COMPONENT; break;
        case tcuTexture.ChannelOrder.DS: format = gl.DEPTH_STENCIL; break;
        case tcuTexture.ChannelOrder.S: format = gl.STENCIL_INDEX; break;

        default:
            throw new Error('Unknown ChannelOrder ' + texFormat.order);
    }

    switch (texFormat.type) {
        case tcuTexture.ChannelType.SNORM_INT8: type = gl.BYTE; break;
        case tcuTexture.ChannelType.SNORM_INT16: type = gl.SHORT; break;
        case tcuTexture.ChannelType.UNORM_INT8: type = gl.UNSIGNED_BYTE; break;
        case tcuTexture.ChannelType.UNORM_INT16: type = gl.UNSIGNED_SHORT; break;
        case tcuTexture.ChannelType.UNORM_SHORT_565: type = gl.UNSIGNED_SHORT_5_6_5; break;
        case tcuTexture.ChannelType.UNORM_SHORT_4444: type = gl.UNSIGNED_SHORT_4_4_4_4; break;
        case tcuTexture.ChannelType.UNORM_SHORT_5551: type = gl.UNSIGNED_SHORT_5_5_5_1; break;
        case tcuTexture.ChannelType.SIGNED_INT8: type = gl.BYTE; break;
        case tcuTexture.ChannelType.SIGNED_INT16: type = gl.SHORT; break;
        case tcuTexture.ChannelType.SIGNED_INT32: type = gl.INT; break;
        case tcuTexture.ChannelType.UNSIGNED_INT8: type = gl.UNSIGNED_BYTE; break;
        case tcuTexture.ChannelType.UNSIGNED_INT16: type = gl.UNSIGNED_SHORT; break;
        case tcuTexture.ChannelType.UNSIGNED_INT32: type = gl.UNSIGNED_INT; break;
        case tcuTexture.ChannelType.FLOAT: type = gl.FLOAT; break;
        case tcuTexture.ChannelType.UNORM_INT_101010: type = gl.UNSIGNED_INT_2_10_10_10_REV; break;
        case tcuTexture.ChannelType.UNORM_INT_1010102_REV: type = gl.UNSIGNED_INT_2_10_10_10_REV; break;
        case tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV: type = gl.UNSIGNED_INT_2_10_10_10_REV; break;
        case tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV: type = gl.UNSIGNED_INT_10F_11F_11F_REV; break;
        case tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV: type = gl.UNSIGNED_INT_5_9_9_9_REV; break;
        case tcuTexture.ChannelType.HALF_FLOAT: type = gl.HALF_FLOAT; break;
        case tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV: type = gl.FLOAT_32_UNSIGNED_INT_24_8_REV; break;
        case tcuTexture.ChannelType.UNSIGNED_INT_24_8: type = texFormat.order == tcuTexture.ChannelOrder.D ?
                                                                 gl.UNSIGNED_INT : gl.UNSIGNED_INT_24_8; break;

        default:
            throw new Error("Can't map texture format to GL transfer format " + texFormat.type);
    }

    return new gluTextureUtil.TransferFormat(format, type);
};

/**
 * Map tcuTexture.TextureFormat to GL internal sized format.
 *
 * Maps generic texture format description to GL internal format.
 * If no mapping is found, throws Error.
 *
 * @param {tcuTexture.TextureFormat} texFormat Generic texture format.
 * @return {number} GL texture format.
 * @throws {Error}
 */
gluTextureUtil.getInternalFormat = function(texFormat) {

    var stringify = function(order, type) {
        return '' + order + ' ' + type;
    };

    switch (stringify(texFormat.order, texFormat.type)) {
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_5551): return gl.RGB5_A1;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_4444): return gl.RGBA4;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_SHORT_565): return gl.RGB565;
        case stringify(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNORM_INT16): return gl.DEPTH_COMPONENT16;
        case stringify(tcuTexture.ChannelOrder.S, tcuTexture.ChannelType.UNSIGNED_INT8): return gl.STENCIL_INDEX8;

        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.FLOAT): return gl.RGBA32F;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT32): return gl.RGBA32I;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT32): return gl.RGBA32UI;
        // TODO: Check which ones are valid in WebGL 2 - case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT16): return gl.RGBA16;
        //case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SNORM_INT16): return gl.RGBA16_SNORM;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.HALF_FLOAT): return gl.RGBA16F;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT16): return gl.RGBA16I;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT16): return gl.RGBA16UI;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8): return gl.RGBA8;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT8): return gl.RGBA8I;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT8): return gl.RGBA8UI;
        case stringify(tcuTexture.ChannelOrder.sRGBA, tcuTexture.ChannelType.UNORM_INT8): return gl.SRGB8_ALPHA8;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT_1010102_REV): return gl.RGB10_A2;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV): return gl.RGB10_A2UI;
        case stringify(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SNORM_INT8): return gl.RGBA8_SNORM;

        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT8): return gl.RGB8;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV): return gl.R11F_G11F_B10F;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.FLOAT): return gl.RGB32F;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SIGNED_INT32): return gl.RGB32I;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT32): return gl.RGB32UI;
        //case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT16): return gl.RGB16;
        //case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SNORM_INT16): return gl.RGB16_SNORM;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.HALF_FLOAT): return gl.RGB16F;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SIGNED_INT16): return gl.RGB16I;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT16): return gl.RGB16UI;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SNORM_INT8): return gl.RGB8_SNORM;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SIGNED_INT8): return gl.RGB8I;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT8): return gl.RGB8UI;
        case stringify(tcuTexture.ChannelOrder.sRGB, tcuTexture.ChannelType.UNORM_INT8): return gl.SRGB8;
        case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV): return gl.RGB9_E5;
        //case stringify(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT_1010102_REV): return gl.RGB10;

        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.FLOAT): return gl.RG32F;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SIGNED_INT32): return gl.RG32I;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNSIGNED_INT32): return gl.RG32UI;
        //case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNORM_INT16): return gl.RG16;
        //case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SNORM_INT16): return gl.RG16_SNORM;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.HALF_FLOAT): return gl.RG16F;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SIGNED_INT16): return gl.RG16I;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNSIGNED_INT16): return gl.RG16UI;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNORM_INT8): return gl.RG8;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SIGNED_INT8): return gl.RG8I;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNSIGNED_INT8): return gl.RG8UI;
        case stringify(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SNORM_INT8): return gl.RG8_SNORM;

        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.FLOAT): return gl.R32F;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SIGNED_INT32): return gl.R32I;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNSIGNED_INT32): return gl.R32UI;
        //case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNORM_INT16): return gl.R16;
        //case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SNORM_INT16): return gl.R16_SNORM;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.HALF_FLOAT): return gl.R16F;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SIGNED_INT16): return gl.R16I;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNSIGNED_INT16): return gl.R16UI;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNORM_INT8): return gl.R8;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SIGNED_INT8): return gl.R8I;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNSIGNED_INT8): return gl.R8UI;
        case stringify(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SNORM_INT8): return gl.R8_SNORM;

        case stringify(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.FLOAT): return gl.DEPTH_COMPONENT32F;
        case stringify(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNSIGNED_INT_24_8): return gl.DEPTH_COMPONENT24;
        //case stringify(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNSIGNED_INT32): return gl.DEPTH_COMPONENT32;
        case stringify(tcuTexture.ChannelOrder.DS, tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV): return gl.DEPTH32F_STENCIL8;
        case stringify(tcuTexture.ChannelOrder.DS, tcuTexture.ChannelType.UNSIGNED_INT_24_8): return gl.DEPTH24_STENCIL8;

        default:
            throw new Error("Can't map texture format to GL internal format");
    }
};

/**
 * Enable WEBGL_compressed_texture_etc support if available, by merging it
 * into the WebGL2RenderingContext.
 *
 * This function may be called many times.
 *
 * @return {boolean} True if enabled.
 */
gluTextureUtil.enableCompressedTextureETC = (function() {
    var enabled = undefined;
    return function() {
        if (enabled === undefined) {
            enabled = false;

            var WEBGL_compressed_texture_etc = gl.getExtension("WEBGL_compressed_texture_etc");
            if (WEBGL_compressed_texture_etc) {
                // Extend gl with enums from WEBGL_compressed_texture_etc
                // (if it doesn't already have the etc texture formats).
                var proto = Object.getPrototypeOf(WEBGL_compressed_texture_etc);
                for (var prop in proto) {
                    if (proto.hasOwnProperty(prop)) {
                        gl[prop] = proto[prop];
                    }
                }
                enabled = true;
            }
        }
        return enabled;
    };
})();

/**
 * Map generic compressed format to GL compressed format enum.
 *
 * Maps generic compressed format to GL compressed format enum value.
 * If no mapping is found, throws Error.

 * @param {tcuCompressedTexture.Format} format Generic compressed format.
 * @return {number} GL compressed texture format.
 * @throws {Error}
 */
gluTextureUtil.getGLFormat = function(/* tcuCompressedTexture.Format */ format) {
    switch (format) {
        // TODO: check which are available in WebGL 2 - case tcuCompressedTexture.Format.ETC1_RGB8: return gl.ETC1_RGB8_OES;
        case tcuCompressedTexture.Format.EAC_R11: return gl.COMPRESSED_R11_EAC;
        case tcuCompressedTexture.Format.EAC_SIGNED_R11: return gl.COMPRESSED_SIGNED_R11_EAC;
        case tcuCompressedTexture.Format.EAC_RG11: return gl.COMPRESSED_RG11_EAC;
        case tcuCompressedTexture.Format.EAC_SIGNED_RG11: return gl.COMPRESSED_SIGNED_RG11_EAC;
        case tcuCompressedTexture.Format.ETC2_RGB8: return gl.COMPRESSED_RGB8_ETC2;
        case tcuCompressedTexture.Format.ETC2_SRGB8: return gl.COMPRESSED_SRGB8_ETC2;
        case tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1: return gl.COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2;
        case tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1: return gl.COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2;
        case tcuCompressedTexture.Format.ETC2_EAC_RGBA8: return gl.COMPRESSED_RGBA8_ETC2_EAC;
        case tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC;

        /*case tcuCompressedTexture.Format.ASTC_4x4_RGBA: return gl.COMPRESSED_RGBA_ASTC_4x4_KHR;
        case tcuCompressedTexture.Format.ASTC_5x4_RGBA: return gl.COMPRESSED_RGBA_ASTC_5x4_KHR;
        case tcuCompressedTexture.Format.ASTC_5x5_RGBA: return gl.COMPRESSED_RGBA_ASTC_5x5_KHR;
        case tcuCompressedTexture.Format.ASTC_6x5_RGBA: return gl.COMPRESSED_RGBA_ASTC_6x5_KHR;
        case tcuCompressedTexture.Format.ASTC_6x6_RGBA: return gl.COMPRESSED_RGBA_ASTC_6x6_KHR;
        case tcuCompressedTexture.Format.ASTC_8x5_RGBA: return gl.COMPRESSED_RGBA_ASTC_8x5_KHR;
        case tcuCompressedTexture.Format.ASTC_8x6_RGBA: return gl.COMPRESSED_RGBA_ASTC_8x6_KHR;
        case tcuCompressedTexture.Format.ASTC_8x8_RGBA: return gl.COMPRESSED_RGBA_ASTC_8x8_KHR;
        case tcuCompressedTexture.Format.ASTC_10x5_RGBA: return gl.COMPRESSED_RGBA_ASTC_10x5_KHR;
        case tcuCompressedTexture.Format.ASTC_10x6_RGBA: return gl.COMPRESSED_RGBA_ASTC_10x6_KHR;
        case tcuCompressedTexture.Format.ASTC_10x8_RGBA: return gl.COMPRESSED_RGBA_ASTC_10x8_KHR;
        case tcuCompressedTexture.Format.ASTC_10x10_RGBA: return gl.COMPRESSED_RGBA_ASTC_10x10_KHR;
        case tcuCompressedTexture.Format.ASTC_12x10_RGBA: return gl.COMPRESSED_RGBA_ASTC_12x10_KHR;
        case tcuCompressedTexture.Format.ASTC_12x12_RGBA: return gl.COMPRESSED_RGBA_ASTC_12x12_KHR;
        case tcuCompressedTexture.Format.ASTC_4x4_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_4x4_KHR;
        case tcuCompressedTexture.Format.ASTC_5x4_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_5x4_KHR;
        case tcuCompressedTexture.Format.ASTC_5x5_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_5x5_KHR;
        case tcuCompressedTexture.Format.ASTC_6x5_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_6x5_KHR;
        case tcuCompressedTexture.Format.ASTC_6x6_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_6x6_KHR;
        case tcuCompressedTexture.Format.ASTC_8x5_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_8x5_KHR;
        case tcuCompressedTexture.Format.ASTC_8x6_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_8x6_KHR;
        case tcuCompressedTexture.Format.ASTC_8x8_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_8x8_KHR;
        case tcuCompressedTexture.Format.ASTC_10x5_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_10x5_KHR;
        case tcuCompressedTexture.Format.ASTC_10x6_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_10x6_KHR;
        case tcuCompressedTexture.Format.ASTC_10x8_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_10x8_KHR;
        case tcuCompressedTexture.Format.ASTC_10x10_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_10x10_KHR;
        case tcuCompressedTexture.Format.ASTC_12x10_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_12x10_KHR;
        case tcuCompressedTexture.Format.ASTC_12x12_SRGB8_ALPHA8: return gl.COMPRESSED_SRGB8_ALPHA8_ASTC_12x12_KHR;*/

        default:
            throw new Error("Can't map compressed format to GL format");
    }
};

/**
 * @param {number} dataType
 * @param {boolean} normalized
 * @return {tcuTexture.ChannelType}
 * @throws {Error}
 */
gluTextureUtil.mapGLChannelType = function(/* deMath.deUint32 */ dataType, /*boolean*/ normalized) {
    // \note Normalized bit is ignored where it doesn't apply.

    switch (dataType) {
        case gl.UNSIGNED_BYTE: return normalized ? tcuTexture.ChannelType.UNORM_INT8 : tcuTexture.ChannelType.UNSIGNED_INT8;
        case gl.BYTE: return normalized ? tcuTexture.ChannelType.SNORM_INT8 : tcuTexture.ChannelType.SIGNED_INT8;
        case gl.UNSIGNED_SHORT: return normalized ? tcuTexture.ChannelType.UNORM_INT16 : tcuTexture.ChannelType.UNSIGNED_INT16;
        case gl.SHORT: return normalized ? tcuTexture.ChannelType.SNORM_INT16 : tcuTexture.ChannelType.SIGNED_INT16;
        case gl.UNSIGNED_INT: return normalized ? tcuTexture.ChannelType.UNORM_INT32 : tcuTexture.ChannelType.UNSIGNED_INT32;
        case gl.INT: return normalized ? tcuTexture.ChannelType.SNORM_INT32 : tcuTexture.ChannelType.SIGNED_INT32;
        case gl.FLOAT: return tcuTexture.ChannelType.FLOAT;
        case gl.UNSIGNED_SHORT_4_4_4_4: return tcuTexture.ChannelType.UNORM_SHORT_4444;
        case gl.UNSIGNED_SHORT_5_5_5_1: return tcuTexture.ChannelType.UNORM_SHORT_5551;
        case gl.UNSIGNED_SHORT_5_6_5: return tcuTexture.ChannelType.UNORM_SHORT_565;
        case gl.HALF_FLOAT: return tcuTexture.ChannelType.HALF_FLOAT;
        case gl.UNSIGNED_INT_2_10_10_10_REV: return normalized ? tcuTexture.ChannelType.UNORM_INT_1010102_REV : tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV;
        case gl.UNSIGNED_INT_10F_11F_11F_REV: return tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV;
        case gl.UNSIGNED_INT_24_8: return tcuTexture.ChannelType.UNSIGNED_INT_24_8;
        case gl.FLOAT_32_UNSIGNED_INT_24_8_REV: return tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV;
        case gl.UNSIGNED_INT_5_9_9_9_REV: return tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV;

        default:
            throw new Error('Unsupported dataType ' + dataType);
    }
};

/**
 * @param {number} format Generic compressed format.
 * @param {number} dataType
 * @return {tcuTexture.TextureFormat} GL texture format.
 * @throws {Error}
 */
gluTextureUtil.mapGLTransferFormat = function(format, dataType) {
    switch (format) {
        case gl.ALPHA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.A, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.LUMINANCE: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.L, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.LUMINANCE_ALPHA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.LA, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.RGB: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.RGBA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, gluTextureUtil.mapGLChannelType(dataType, true));
        //case gl.BGRA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.BGRA, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.RG: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.RED: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.RGBA_INTEGER: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, gluTextureUtil.mapGLChannelType(dataType, false));
        case gl.RGB_INTEGER: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, gluTextureUtil.mapGLChannelType(dataType, false));
        case gl.RG_INTEGER: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, gluTextureUtil.mapGLChannelType(dataType, false));
        case gl.RED_INTEGER: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, gluTextureUtil.mapGLChannelType(dataType, false));

        case gl.DEPTH_COMPONENT: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, gluTextureUtil.mapGLChannelType(dataType, true));
        case gl.DEPTH_STENCIL: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.DS, gluTextureUtil.mapGLChannelType(dataType, true));

        default:
            throw new Error("Can't map GL pixel format (" + deString.enumToString(gl, format) + ', ' + deString.enumToString(gl, dataType) + ') to texture format');
    }
};

 /**
 * Map GL internal texture format to tcuTexture.TextureFormat.
 *
 * If no mapping is found, throws Error.
 * @param {number} internalFormat
 * @return {tcuTexture.TextureFormat} GL texture format.
 * @throws {Error}
 */
gluTextureUtil.mapGLInternalFormat = function(/*deMath.deUint32*/ internalFormat) {
    if (internalFormat === undefined)
        throw new Error('internalformat is undefined');

    switch (internalFormat) {
        case gl.RGB5_A1: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_5551);
        case gl.RGBA4: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_4444);
        case gl.RGB565: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_SHORT_565);
        case gl.DEPTH_COMPONENT16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNORM_INT16);
        case gl.STENCIL_INDEX8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.S, tcuTexture.ChannelType.UNSIGNED_INT8);

        case gl.RGBA32F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.FLOAT);
        case gl.RGBA32I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT32);
        case gl.RGBA32UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT32);
        //TODO: Check which are available in WebGL 2 case gl.RGBA16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT16);
        //case gl.RGBA16_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SNORM_INT16);
        case gl.RGBA16F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.HALF_FLOAT);
        case gl.RGBA16I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT16);
        case gl.RGBA16UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT16);
        case gl.RGBA8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);
        case gl.RGBA8I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT8);
        case gl.RGBA8UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT8);
        case gl.SRGB8_ALPHA8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.sRGBA, tcuTexture.ChannelType.UNORM_INT8);
        case gl.RGB10_A2: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT_1010102_REV);
        case gl.RGB10_A2UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV);
        case gl.RGBA8_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SNORM_INT8);

        case gl.RGB8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT8);
        case gl.R11F_G11F_B10F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV);
        case gl.RGB32F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.FLOAT);
        case gl.RGB32I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SIGNED_INT32);
        case gl.RGB32UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT32);
        //case gl.RGB16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT16);
        //case gl.RGB16_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SNORM_INT16);
        case gl.RGB16F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.HALF_FLOAT);
        case gl.RGB16I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SIGNED_INT16);
        case gl.RGB16UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT16);
        case gl.RGB8_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SNORM_INT8);
        case gl.RGB8I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.SIGNED_INT8);
        case gl.RGB8UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT8);
        case gl.SRGB8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.sRGB, tcuTexture.ChannelType.UNORM_INT8);
        case gl.RGB9_E5: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNSIGNED_INT_999_E5_REV);
        //case gl.RGB10: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT_1010102_REV);

        case gl.RG32F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.FLOAT);
        case gl.RG32I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SIGNED_INT32);
        case gl.RG32UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNSIGNED_INT32);
        //case gl.RG16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNORM_INT16);
        //case gl.RG16_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SNORM_INT16);
        case gl.RG16F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.HALF_FLOAT);
        case gl.RG16I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SIGNED_INT16);
        case gl.RG16UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNSIGNED_INT16);
        case gl.RG8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNORM_INT8);
        case gl.RG8I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SIGNED_INT8);
        case gl.RG8UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.UNSIGNED_INT8);
        case gl.RG8_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RG, tcuTexture.ChannelType.SNORM_INT8);

        case gl.R32F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.FLOAT);
        case gl.R32I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SIGNED_INT32);
        case gl.R32UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNSIGNED_INT32);
        //case gl.R16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNORM_INT16);
        //case gl.R16_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SNORM_INT16);
        case gl.R16F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.HALF_FLOAT);
        case gl.R16I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SIGNED_INT16);
        case gl.R16UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNSIGNED_INT16);
        case gl.R8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNORM_INT8);
        case gl.R8I: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SIGNED_INT8);
        case gl.R8UI: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.UNSIGNED_INT8);
        case gl.R8_SNORM: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.R, tcuTexture.ChannelType.SNORM_INT8);

        case gl.DEPTH_COMPONENT32F: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.FLOAT);
        case gl.DEPTH_COMPONENT24: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNSIGNED_INT_24_8);
        //case gl.DEPTH_COMPONENT32: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNSIGNED_INT32);
        case gl.DEPTH32F_STENCIL8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.DS, tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV);
        case gl.DEPTH24_STENCIL8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.DS, tcuTexture.ChannelType.UNSIGNED_INT_24_8);

        default:
            throw new Error("Can't map GL sized internal format (" + internalFormat.toString(16) + ') to texture format');
    }
};

/**
 * @param {number} format
 * @return {boolean}
 */
gluTextureUtil.isGLInternalColorFormatFilterable = function(format) {
    switch (format) {
        case gl.R8:
        case gl.R8_SNORM:
        case gl.RG8:
        case gl.RG8_SNORM:
        case gl.RGB8:
        case gl.RGB8_SNORM:
        case gl.RGB565:
        case gl.RGBA4:
        case gl.RGB5_A1:
        case gl.RGBA8:
        case gl.RGBA8_SNORM:
        case gl.RGB10_A2:
        case gl.SRGB8:
        case gl.SRGB8_ALPHA8:
        case gl.R16F:
        case gl.RG16F:
        case gl.RGB16F:
        case gl.RGBA16F:
        case gl.R11F_G11F_B10F:
        case gl.RGB9_E5:
            return true;

        case gl.RGB10_A2UI:
        case gl.R32F:
        case gl.RG32F:
        case gl.RGB32F:
        case gl.RGBA32F:
        case gl.R8I:
        case gl.R8UI:
        case gl.R16I:
        case gl.R16UI:
        case gl.R32I:
        case gl.R32UI:
        case gl.RG8I:
        case gl.RG8UI:
        case gl.RG16I:
        case gl.RG16UI:
        case gl.RG32I:
        case gl.RG32UI:
        case gl.RGB8I:
        case gl.RGB8UI:
        case gl.RGB16I:
        case gl.RGB16UI:
        case gl.RGB32I:
        case gl.RGB32UI:
        case gl.RGBA8I:
        case gl.RGBA8UI:
        case gl.RGBA16I:
        case gl.RGBA16UI:
        case gl.RGBA32I:
        case gl.RGBA32UI:
            return false;

        default:
            throw new Error('Unrecognized format ' + format);
    }
};

/**
 * @param {number} wrapMode
 * @return {tcuTexture.WrapMode}
 */
gluTextureUtil.mapGLWrapMode = function(wrapMode) {
    switch (wrapMode) {
        case gl.CLAMP_TO_EDGE: return tcuTexture.WrapMode.CLAMP_TO_EDGE;
        case gl.REPEAT: return tcuTexture.WrapMode.REPEAT_GL;
        case gl.MIRRORED_REPEAT: return tcuTexture.WrapMode.MIRRORED_REPEAT_GL;
        default:
            throw new Error("Can't map GL wrap mode " + deString.enumToString(gl, wrapMode));
    }
};

/**
 * @param {number} filterMode
 * @return {tcuTexture.FilterMode}
 * @throws {Error}
 */
gluTextureUtil.mapGLFilterMode = function(filterMode) {
    switch (filterMode) {
        case gl.NEAREST: return tcuTexture.FilterMode.NEAREST;
        case gl.LINEAR: return tcuTexture.FilterMode.LINEAR;
        case gl.NEAREST_MIPMAP_NEAREST: return tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST;
        case gl.NEAREST_MIPMAP_LINEAR: return tcuTexture.FilterMode.NEAREST_MIPMAP_LINEAR;
        case gl.LINEAR_MIPMAP_NEAREST: return tcuTexture.FilterMode.LINEAR_MIPMAP_NEAREST;
        case gl.LINEAR_MIPMAP_LINEAR: return tcuTexture.FilterMode.LINEAR_MIPMAP_LINEAR;
        default:
            throw new Error("Can't map GL filter mode" + filterMode);
    }
};

/* TODO: Port the code below */
// /*--------------------------------------------------------------------*//*!
//  * \brief Map GL sampler parameters to tcu::Sampler.
//  *
//  * If no mapping is found, throws tcu::InternalError.
//  *
//  * \param wrapS S-component wrap mode
//  * \param minFilter Minification filter mode
//  * \param magFilter Magnification filter mode
//  * \return Sampler description.
//  *//*--------------------------------------------------------------------*/
// /*tcu::Sampler mapGLSamplerWrapS (deUint32 wrapS, deUint32 minFilter, deUint32 magFilter)
// {
//     return mapGLSampler(wrapS, wrapS, wrapS, minFilter, magFilter);
// }
// */

/**
 * Map GL sampler parameters to tcu::Sampler.
 *
 * If no mapping is found, throws tcu::InternalError.
 *
 * @param {number} wrapS S-component wrap mode
 * @param {number} wrapT T-component wrap mode
 * @param {number} minFilter Minification filter mode
 * @param {number} magFilter Magnification filter mode
 * @return {tcuTexture.Sampler}
 */
gluTextureUtil.mapGLSamplerWrapST = function(wrapS, wrapT, minFilter, magFilter) {
    return gluTextureUtil.mapGLSampler(wrapS, wrapT, wrapS, minFilter, magFilter);
};

/**
 * Map GL sampler parameters to tcu::Sampler.
 *
 * If no mapping is found, throws tcu::InternalError.
 * @param {number} wrapS S-component wrap mode
 * @param {number} wrapT T-component wrap mode
 * @param {number} wrapR R-component wrap mode
 * @param {number} minFilter Minification filter mode
 * @param {number} magFilter Magnification filter mode
 * @return {tcuTexture.Sampler}
 */
gluTextureUtil.mapGLSampler = function(wrapS, wrapT, wrapR, minFilter, magFilter) {
    return new tcuTexture.Sampler(
        gluTextureUtil.mapGLWrapMode(wrapS),
        gluTextureUtil.mapGLWrapMode(wrapT),
        gluTextureUtil.mapGLWrapMode(wrapR),
        gluTextureUtil.mapGLFilterMode(minFilter),
        gluTextureUtil.mapGLFilterMode(magFilter),
        0.0,
        true,
        tcuTexture.CompareMode.COMPAREMODE_NONE,
        0,
        [0.0, 0.0, 0.0, 0.0]);
};

// /*--------------------------------------------------------------------*//*!
//  * \brief Map GL compare function to tcu::Sampler::CompareMode.
//  *
//  * If no mapping is found, throws tcu::InternalError.
//  *
//  * \param mode GL compare mode
//  * \return Compare mode
//  *//*--------------------------------------------------------------------*/
/**
 * @param {number} mode
 */
gluTextureUtil.mapGLCompareFunc = function(mode) {
     switch (mode) {
     case gl.LESS: return tcuTexture.CompareMode.COMPAREMODE_LESS;
         case gl.LEQUAL: return tcuTexture.CompareMode.COMPAREMODE_LESS_OR_EQUAL;
         case gl.GREATER: return tcuTexture.CompareMode.COMPAREMODE_GREATER;
         case gl.GEQUAL: return tcuTexture.CompareMode.COMPAREMODE_GREATER_OR_EQUAL;
         case gl.EQUAL: return tcuTexture.CompareMode.COMPAREMODE_EQUAL;
         case gl.NOTEQUAL: return tcuTexture.CompareMode.COMPAREMODE_NOT_EQUAL;
         case gl.ALWAYS: return tcuTexture.CompareMode.COMPAREMODE_ALWAYS;
         case gl.NEVER: return tcuTexture.CompareMode.COMPAREMODE_NEVER;
         default:
             throw new Error("Can't map GL compare mode " + mode);
     }
};

/**
 * Get GL wrap mode.
 *
 * If no mapping is found, throws tcu::InternalError.
 *
 * @param {tcuTexture.WrapMode} wrapMode
 * @return {number} GL wrap mode
 */
gluTextureUtil.getGLWrapMode = function(wrapMode) {
    switch (wrapMode) {
        case tcuTexture.WrapMode.CLAMP_TO_EDGE: return gl.CLAMP_TO_EDGE;
        case tcuTexture.WrapMode.REPEAT_GL: return gl.REPEAT;
        case tcuTexture.WrapMode.MIRRORED_REPEAT_GL: return gl.MIRRORED_REPEAT;
        default:
            throw new Error("Can't map wrap mode");
    }
};

/**
 * Get GL filter mode.
 *
 * If no mapping is found, throws tcu::InternalError.
 *
 * @param {tcuTexture.FilterMode} filterMode Filter mode
 * @return {number} GL filter mode
 */
gluTextureUtil.getGLFilterMode = function(filterMode) {
    switch (filterMode) {
        case tcuTexture.FilterMode.NEAREST: return gl.NEAREST;
        case tcuTexture.FilterMode.LINEAR: return gl.LINEAR;
        case tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST: return gl.NEAREST_MIPMAP_NEAREST;
        case tcuTexture.FilterMode.NEAREST_MIPMAP_LINEAR: return gl.NEAREST_MIPMAP_LINEAR;
        case tcuTexture.FilterMode.LINEAR_MIPMAP_NEAREST: return gl.LINEAR_MIPMAP_NEAREST;
        case tcuTexture.FilterMode.LINEAR_MIPMAP_LINEAR: return gl.LINEAR_MIPMAP_LINEAR;
        default:
            throw new Error("Can't map filter mode");
    }
};

/**
 * Get GL compare mode.
 *
 * If no mapping is found, throws tcu::InternalError.
 *
 * @param {tcuTexture.CompareMode} compareMode Compare mode
 * @return {number} GL compare mode
 */
gluTextureUtil.getGLCompareFunc = function(compareMode) {
    switch (compareMode) {
        case tcuTexture.CompareMode.COMPAREMODE_NONE: return gl.NONE;
        case tcuTexture.CompareMode.COMPAREMODE_LESS: return gl.LESS;
        case tcuTexture.CompareMode.COMPAREMODE_LESS_OR_EQUAL: return gl.LEQUAL;
        case tcuTexture.CompareMode.COMPAREMODE_GREATER: return gl.GREATER;
        case tcuTexture.CompareMode.COMPAREMODE_GREATER_OR_EQUAL: return gl.GEQUAL;
        case tcuTexture.CompareMode.COMPAREMODE_EQUAL: return gl.EQUAL;
        case tcuTexture.CompareMode.COMPAREMODE_NOT_EQUAL: return gl.NOTEQUAL;
        case tcuTexture.CompareMode.COMPAREMODE_ALWAYS: return gl.ALWAYS;
        case tcuTexture.CompareMode.COMPAREMODE_NEVER: return gl.NEVER;
        default:
            throw new Error("Can't map compare mode");
    }
};

/**
 * Get GL cube face.
 *
 * If no mapping is found, throws tcu::InternalError.
 *
 * @param {tcuTexture.CubeFace} face Cube face
 * @return {number} GL cube face
 */
gluTextureUtil.getGLCubeFace = function(face) {
    switch (face) {
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X:
            return gl.TEXTURE_CUBE_MAP_NEGATIVE_X;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X:
            return gl.TEXTURE_CUBE_MAP_POSITIVE_X;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y:
            return gl.TEXTURE_CUBE_MAP_NEGATIVE_Y;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y:
            return gl.TEXTURE_CUBE_MAP_POSITIVE_Y;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z:
            return gl.TEXTURE_CUBE_MAP_NEGATIVE_Z;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z:
            return gl.TEXTURE_CUBE_MAP_POSITIVE_Z;
        default:
            throw Error("Can't map cube face");
    }
};

// /*--------------------------------------------------------------------*//*!
//  * \brief Get GLSL sampler type for texture format.
//  *
//  * If no mapping is found, glu::TYPE_LAST is returned.
//  *
//  * \param format Texture format
//  * \return GLSL 1D sampler type for format
//  *//*--------------------------------------------------------------------*/
// DataType getSampler1DType (tcu::TextureFormat format)
// {
//     using tcu::TextureFormat;

//     if (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS)
//         return TYPE_SAMPLER_1D;

//     if (format.order == tcuTexture.ChannelOrder.S)
//         return TYPE_LAST;

//     switch (tcu::getTextureChannelClass(format.type))
//     {
//         case tcu::TEXTURECHANNELCLASS_FLOATING_POINT:
//         case tcu::TEXTURECHANNELCLASS_SIGNED_FIXED_POINT:
//         case tcu::TEXTURECHANNELCLASS_UNSIGNED_FIXED_POINT:
//             return glu::TYPE_SAMPLER_1D;

//         case tcu::TEXTURECHANNELCLASS_SIGNED_INTEGER:
//             return glu::TYPE_INT_SAMPLER_1D;

//         case tcu::TEXTURECHANNELCLASS_UNSIGNED_INTEGER:
//             return glu::TYPE_UINT_SAMPLER_1D;

//         default:
//             return glu::TYPE_LAST;
//     }
// }

/**
 * Get GLSL sampler type for texture format.
 * If no mapping is found, glu::TYPE_LAST is returned.
 *
 * @param {tcuTexture.TextureFormat} format
 * @return {gluShaderUtil.DataType} GLSL 2D sampler type for format
 */
gluTextureUtil.getSampler2DType = function(format) {
    if (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS)
    return gluShaderUtil.DataType.SAMPLER_2D;

    if (format.order == tcuTexture.ChannelOrder.S)
    return /** @type {gluShaderUtil.DataType} */ (Object.keys(gluShaderUtil.DataType).length);

    switch (tcuTexture.getTextureChannelClass(format.type)) {
        case tcuTexture.TextureChannelClass.FLOATING_POINT:
        case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
        case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
            return gluShaderUtil.DataType.SAMPLER_2D;

        case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
            return gluShaderUtil.DataType.INT_SAMPLER_2D;

        case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
            return gluShaderUtil.DataType.UINT_SAMPLER_2D;

        default:
            return /** @type {gluShaderUtil.DataType} */ (Object.keys(gluShaderUtil.DataType).length);
    }
};

/**
 *
 * @param {tcuTexture.TextureFormat} format
 * @return {gluShaderUtil.DataType} GLSL 2D sampler type for format
 */
gluTextureUtil.getSampler3DType = function(format) {
    if (format.order === tcuTexture.ChannelOrder.D || format.order === tcuTexture.ChannelOrder.DS)
        return gluShaderUtil.DataType.SAMPLER_3D;

    if (format.order === tcuTexture.ChannelOrder.S)
        return /** @type {gluShaderUtil.DataType} */ (Object.keys(gluShaderUtil.DataType).length); // shouldn't we throw an error instead?

    switch (tcuTexture.getTextureChannelClass(format.type)) {
        case tcuTexture.TextureChannelClass.FLOATING_POINT:
        case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
        case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
            return gluShaderUtil.DataType.SAMPLER_3D;

        case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
            return gluShaderUtil.DataType.INT_SAMPLER_3D;

        case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
            return gluShaderUtil.DataType.UINT_SAMPLER_3D;

        default:
            return /** @type {gluShaderUtil.DataType} */ (Object.keys(gluShaderUtil.DataType).length);
    }
};

/**
 * \brief Get GLSL sampler type for texture format.
 *
 * @param {tcuTexture.TextureFormat} format
 * @return {gluShaderUtil.DataType} GLSL 2D sampler type for format
 */
gluTextureUtil.getSamplerCubeType = function(format) {
    if (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS)
        return gluShaderUtil.DataType.SAMPLER_CUBE;

    if (format.order == tcuTexture.ChannelOrder.S)
        throw new Error('No cube sampler');

    switch (tcuTexture.getTextureChannelClass(format.type)) {
        case tcuTexture.TextureChannelClass.FLOATING_POINT:
        case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
        case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
            return gluShaderUtil.DataType.SAMPLER_CUBE;

        case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
            return gluShaderUtil.DataType.INT_SAMPLER_CUBE;

        case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
            return gluShaderUtil.DataType.UINT_SAMPLER_CUBE;

        default:
            throw new Error('No cube sampler');
    }
};

/**
 * \brief Get GLSL sampler type for texture format.
 *
 * If no mapping is found, glu::TYPE_LAST is returned.
 *
 * @param {tcuTexture.TextureFormat} format
 * @return {gluShaderUtil.DataType} GLSL 2D sampler type for format
 */
gluTextureUtil.getSampler2DArrayType = function(format) {

    if (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS)
        return gluShaderUtil.DataType.SAMPLER_2D_ARRAY;

    if (format.order == tcuTexture.ChannelOrder.S)
        throw new Error('No 2d array sampler');

    switch (tcuTexture.getTextureChannelClass(format.type)) {
        case tcuTexture.TextureChannelClass.FLOATING_POINT:
        case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
        case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
            return gluShaderUtil.DataType.SAMPLER_2D_ARRAY;

        case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
            return gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY;

        case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
            return gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY;

        default:
            throw new Error('No 2d array sampler');
    }
};

/**
 * \brief Get GLSL sampler type for texture format.
 *
 * If no mapping is found, glu::TYPE_LAST is returned.
 *
 * @param {tcuTexture.TextureFormat} format
 * @return {gluShaderUtil.DataType} GLSL 2D sampler type for format
 */
gluTextureUtil.getSampler3D = function(format) {
    if (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS)
        return gluShaderUtil.DataType.SAMPLER_3D;

    if (format.order == tcuTexture.ChannelOrder.S)
        throw new Error('No 3d sampler');

    switch (tcuTexture.getTextureChannelClass(format.type)) {
        case tcuTexture.TextureChannelClass.FLOATING_POINT:
        case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
        case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
            return gluShaderUtil.DataType.SAMPLER_3D;

        case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
            return gluShaderUtil.DataType.INT_SAMPLER_3D;

        case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
            return gluShaderUtil.DataType.UINT_SAMPLER_3D;

        default:
            throw new Error('No 3d sampler');
    }
};

gluTextureUtil.RenderableType = {
    RENDERABLE_COLOR: (1<<0),
    RENDERABLE_DEPTH: (1<<1),
    RENDERABLE_STENCIL: (1<<2)
};

/**
 * \brief Get renderable bits.
 * \note Works currently only on ES3 context.
 *
 * @param {number} internalFormat
 * @return {gluTextureUtil.RenderableType}
 */
gluTextureUtil.getRenderableBitsES3 = function(internalFormat)
{
   switch (internalFormat)
   {
       // Color-renderable formats
       case gl.RGBA32I:
       case gl.RGBA32UI:
       case gl.RGBA16I:
       case gl.RGBA16UI:
       case gl.RGBA8:
       case gl.RGBA8I:
       case gl.RGBA8UI:
       case gl.SRGB8_ALPHA8:
       case gl.RGB10_A2:
       case gl.RGB10_A2UI:
       case gl.RGBA4:
       case gl.RGB5_A1:
       case gl.RGB8:
       case gl.RGB565:
       case gl.RG32I:
       case gl.RG32UI:
       case gl.RG16I:
       case gl.RG16UI:
       case gl.RG8:
       case gl.RG8I:
       case gl.RG8UI:
       case gl.R32I:
       case gl.R32UI:
       case gl.R16I:
       case gl.R16UI:
       case gl.R8:
       case gl.R8I:
       case gl.R8UI:
           return gluTextureUtil.RenderableType.RENDERABLE_COLOR;

       // EXT_color_buffer_float
       case gl.RGBA32F:
       case gl.R11F_G11F_B10F:
       case gl.RG32F:
       case gl.R32F:
       case gl.RGBA16F:
       case gl.RG16F:
       case gl.R16F:
           if (gl.getExtension("EXT_color_buffer_float"))
               return gluTextureUtil.RenderableType.RENDERABLE_COLOR;
           else
               return 0;

       // Depth formats
       case gl.DEPTH_COMPONENT32F:
       case gl.DEPTH_COMPONENT24:
       case gl.DEPTH_COMPONENT16:
           return gluTextureUtil.RenderableType.RENDERABLE_DEPTH;

       // Depth+stencil formats
       case gl.DEPTH32F_STENCIL8:
       case gl.DEPTH24_STENCIL8:
           return gluTextureUtil.RenderableType.RENDERABLE_DEPTH | gluTextureUtil.RenderableType.RENDERABLE_STENCIL;

       // Stencil formats
       case gl.STENCIL_INDEX8:
           return gluTextureUtil.RenderableType.RENDERABLE_STENCIL;

       default:
           return 0;
   }
}

/**
 * \brief Check if sized internal format is color-renderable.
 * \note Works currently only on ES3 context.
 *
 * @param {number} sizedFormat
 * @return {boolean}
 */
gluTextureUtil.isSizedFormatColorRenderable = function(sizedFormat)
{
    var renderable = 0;
    renderable = gluTextureUtil.getRenderableBitsES3(sizedFormat);
    return (renderable & gluTextureUtil.RenderableType.RENDERABLE_COLOR) != 0;
}

});

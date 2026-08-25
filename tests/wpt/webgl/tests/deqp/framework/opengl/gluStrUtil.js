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
goog.provide('framework.opengl.gluStrUtil');

goog.scope(function() {

var gluStrUtil = framework.opengl.gluStrUtil;

gluStrUtil.getPixelFormatName = function(value) {
    switch (value) {
        case gl.LUMINANCE: return 'gl.LUMINANCE';
        case gl.LUMINANCE_ALPHA: return 'gl.LUMINANCE_ALPHA';
        case gl.ALPHA: return 'gl.ALPHA';
        case gl.RGB: return 'gl.RGB';
        case gl.RGBA: return 'gl.RGBA';
        case gl.RGBA4: return 'gl.RGBA4';
        case gl.RGB5_A1: return 'gl.RGB5_A1';
        case gl.RGB565: return 'gl.RGB565';
        case gl.DEPTH_COMPONENT16: return 'gl.DEPTH_COMPONENT16';
        case gl.STENCIL_INDEX8: return 'gl.STENCIL_INDEX8';
        case gl.RG: return 'gl.RG';
        case gl.RED: return 'gl.RED';
        case gl.RGBA_INTEGER: return 'gl.RGBA_INTEGER';
        case gl.RGB_INTEGER: return 'gl.RGB_INTEGER';
        case gl.RG_INTEGER: return 'gl.RG_INTEGER';
        case gl.RED_INTEGER: return 'gl.RED_INTEGER';
        case gl.DEPTH_COMPONENT: return 'gl.DEPTH_COMPONENT';
        case gl.DEPTH_STENCIL: return 'gl.DEPTH_STENCIL';
        case gl.RGBA32F: return 'gl.RGBA32F';
        case gl.RGBA32I: return 'gl.RGBA32I';
        case gl.RGBA32UI: return 'gl.RGBA32UI';
        // case gl.RGBA16: return 'gl.RGBA16';
        // case gl.RGBA16_SNORM: return 'gl.RGBA16_SNORM';
        case gl.RGBA16F: return 'gl.RGBA16F';
        case gl.RGBA16I: return 'gl.RGBA16I';
        case gl.RGBA16UI: return 'gl.RGBA16UI';
        case gl.RGBA8: return 'gl.RGBA8';
        case gl.RGBA8I: return 'gl.RGBA8I';
        case gl.RGBA8UI: return 'gl.RGBA8UI';
        case gl.SRGB8_ALPHA8: return 'gl.SRGB8_ALPHA8';
        case gl.RGB10_A2: return 'gl.RGB10_A2';
        case gl.RGB10_A2UI: return 'gl.RGB10_A2UI';
        case gl.RGBA8_SNORM: return 'gl.RGBA8_SNORM';
        case gl.RGB8: return 'gl.RGB8';
        case gl.R11F_G11F_B10F: return 'gl.R11F_G11F_B10F';
        case gl.RGB32F: return 'gl.RGB32F';
        case gl.RGB32I: return 'gl.RGB32I';
        case gl.RGB32UI: return 'gl.RGB32UI';
        // case gl.RGB16: return 'gl.RGB16';
        // case gl.RGB16_SNORM: return 'gl.RGB16_SNORM';
        case gl.RGB16F: return 'gl.RGB16F';
        case gl.RGB16I: return 'gl.RGB16I';
        case gl.RGB16UI: return 'gl.RGB16UI';
        case gl.RGB8_SNORM: return 'gl.RGB8_SNORM';
        case gl.RGB8I: return 'gl.RGB8I';
        case gl.RGB8UI: return 'gl.RGB8UI';
        case gl.SRGB8: return 'gl.SRGB8';
        case gl.RGB9_E5: return 'gl.RGB9_E5';
        case gl.RG32F: return 'gl.RG32F';
        case gl.RG32I: return 'gl.RG32I';
        case gl.RG32UI: return 'gl.RG32UI';
        // case gl.RG16: return 'gl.RG16';
        // case gl.RG16_SNORM: return 'gl.RG16_SNORM';
        case gl.RG16F: return 'gl.RG16F';
        case gl.RG16I: return 'gl.RG16I';
        case gl.RG16UI: return 'gl.RG16UI';
        case gl.RG8: return 'gl.RG8';
        case gl.RG8I: return 'gl.RG8I';
        case gl.RG8UI: return 'gl.RG8UI';
        case gl.RG8_SNORM: return 'gl.RG8_SNORM';
        case gl.R32F: return 'gl.R32F';
        case gl.R32I: return 'gl.R32I';
        case gl.R32UI: return 'gl.R32UI';
        // case gl.R16: return 'gl.R16';
        // case gl.R16_SNORM: return 'gl.R16_SNORM';
        case gl.R16F: return 'gl.R16F';
        case gl.R16I: return 'gl.R16I';
        case gl.R16UI: return 'gl.R16UI';
        case gl.R8: return 'gl.R8';
        case gl.R8I: return 'gl.R8I';
        case gl.R8UI: return 'gl.R8UI';
        case gl.R8_SNORM: return 'gl.R8_SNORM';
        case gl.DEPTH_COMPONENT32F: return 'gl.DEPTH_COMPONENT32F';
        case gl.DEPTH_COMPONENT24: return 'gl.DEPTH_COMPONENT24';
        case gl.DEPTH32F_STENCIL8: return 'gl.DEPTH32F_STENCIL8';
        case gl.DEPTH24_STENCIL8: return 'gl.DEPTH24_STENCIL8';
        // case gl.RGB10: return 'gl.RGB10';
        // case gl.DEPTH_COMPONENT32: return 'gl.DEPTH_COMPONENT32';
        case gl.SRGB: return 'gl.SRGB';
        // case gl.SRGB_ALPHA: return 'gl.SRGB_ALPHA';
        default: return '';
    }
};

gluStrUtil.getTypeName = function(value) {
    switch (value) {
        case gl.BYTE: return 'gl.BYTE';
        case gl.UNSIGNED_BYTE: return 'gl.UNSIGNED_BYTE';
        case gl.SHORT: return 'gl.SHORT';
        case gl.UNSIGNED_SHORT: return 'gl.UNSIGNED_SHORT';
        case gl.INT: return 'gl.INT';
        case gl.UNSIGNED_INT: return 'gl.UNSIGNED_INT';
        case gl.FLOAT: return 'gl.FLOAT';
        // case gl.FIXED: return 'gl.FIXED';
        case gl.UNSIGNED_SHORT_5_6_5: return 'gl.UNSIGNED_SHORT_5_6_5';
        case gl.UNSIGNED_SHORT_4_4_4_4: return 'gl.UNSIGNED_SHORT_4_4_4_4';
        case gl.UNSIGNED_SHORT_5_5_5_1: return 'gl.UNSIGNED_SHORT_5_5_5_1';
        case gl.HALF_FLOAT: return 'gl.HALF_FLOAT';
        case gl.INT_2_10_10_10_REV: return 'gl.INT_2_10_10_10_REV';
        case gl.UNSIGNED_INT_2_10_10_10_REV: return 'gl.UNSIGNED_INT_2_10_10_10_REV';
        case gl.UNSIGNED_INT_10F_11F_11F_REV: return 'gl.UNSIGNED_INT_10F_11F_11F_REV';
        case gl.UNSIGNED_INT_5_9_9_9_REV: return 'gl.UNSIGNED_INT_5_9_9_9_REV';
        case gl.UNSIGNED_INT_24_8: return 'gl.UNSIGNED_INT_24_8';
        case gl.FLOAT_32_UNSIGNED_INT_24_8_REV: return 'gl.FLOAT_32_UNSIGNED_INT_24_8_REV';
        case gl.SIGNED_NORMALIZED: return 'gl.SIGNED_NORMALIZED';
        case gl.UNSIGNED_NORMALIZED: return 'gl.UNSIGNED_NORMALIZED';
        // case gl.HALF_FLOAT_OES: return 'gl.HALF_FLOAT_OES';
        default: return '';
    }
};

gluStrUtil.getErrorName = function(value) {
    switch (value) {
        case gl.NO_ERROR: return 'gl.NO_ERROR';
        case gl.INVALID_ENUM: return 'gl.INVALID_ENUM';
        case gl.INVALID_VALUE: return 'gl.INVALID_VALUE';
        case gl.INVALID_OPERATION: return 'gl.INVALID_OPERATION';
        case gl.OUT_OF_MEMORY: return 'gl.OUT_OF_MEMORY';
        // case gl.INVALID_FRAMEBUFFER_OPERATION: return 'gl.INVALID_FRAMEBUFFER_OPERATION';
        default: return '';
    }
};

gluStrUtil.getFramebufferStatusName = function(value) {
    switch (value) {
        case gl.FRAMEBUFFER_COMPLETE: return 'gl.FRAMEBUFFER_COMPLETE';
        case gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT: return 'gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT';
        case gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: return 'gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT';
        case gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS: return 'gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS';
        case gl.FRAMEBUFFER_UNSUPPORTED: return 'gl.FRAMEBUFFER_UNSUPPORTED';
        case gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE: return 'gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE';
    //    case: gl.FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS: return 'gl.FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS';
        default: return '';
    }
};

});

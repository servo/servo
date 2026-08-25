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
goog.provide('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.referencerenderer.rrRenderState');

goog.scope(function() {

var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
var deMath = framework.delibs.debase.deMath;
var rrRenderState = framework.referencerenderer.rrRenderState;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;

/** Return oldValue with the bits indicated by mask replaced by corresponding bits of newValue.
 * @param {number} oldValue
 * @param {number} newValue
 * @param {number} mask
 * @return {number}
 */
rrFragmentOperations.maskedBitReplace = function(oldValue, newValue, mask) {
    return (oldValue & ~mask) | (newValue & mask);
};

/**
 * @param {Array<number>} point
 * @param {?} rect
 * @return {boolean}
 */
rrFragmentOperations.isInsideRect = function(point, rect) {
    return deMath.deInBounds32(point[0], rect.left, rect.left + rect.width) &&
           deMath.deInBounds32(point[1], rect.bottom, rect.bottom + rect.height);
};

/**
 * @constructor
 * @param {Array<number>} coefficents
 * @param {Array<number>} coords
 * @param {number} depth
 */
rrFragmentOperations.Fragment = function(coefficents, coords, depth) {
    /** @type {Array<number>} */ this.barycentric = coefficents;
    /** @type {Array<number>} */ this.pixelCoord = coords;
    /** @type {boolean} */ this.isAlive = true;
    /** @type {boolean} */ this.stencilPassed = true;
    /** @type {boolean} */ this.depthPassed = true;
    /** @type {Array<number>} */ this.sampleDepths = [depth];
    /** @type {Array<number>} */ this.clampedBlendSrcColor = [];
    /** @type {Array<number>} */ this.clampedBlendSrc1Color = [];
    /** @type {Array<number>} */ this.clampedBlendDstColor = [];
    /** @type {Array<number>} */ this.blendSrcFactorRGB = [];
    /** @type {number} */ this.blendSrcFactorA = NaN;
    /** @type {Array<number>} */ this.blendDstFactorRGB = [];
    /** @type {number} */ this.blendDstFactorA = NaN;
    /** @type {Array<number>} */ this.blendedRGB = [];
    /** @type {number} */ this.blendedA = NaN;
    /** @type {Array<number>} */ this.signedValue = []; //!< integer targets
    /** @type {Array<number>} */ this.unsignedValue = []; //!< unsigned integer targets
    /** @type {Array<number>} */ this.value = []; /*TODO: what type should it be? */
    /** @type {Array<number>} */ this.value1 = []; /*TODO: what type should it be? */
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {rrRenderState.WindowRectangle} scissorRect
 */
rrFragmentOperations.executeScissorTest = function(inputFragments, scissorRect) {
    for (var i = 0; i < inputFragments.length; i++) {
        var frag = inputFragments[i];
        if (frag.isAlive) {
            if (!rrFragmentOperations.isInsideRect(frag.pixelCoord, scissorRect))
                frag.isAlive = false;
        }
    }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {rrRenderState.StencilState} stencilState
 * @param {number} numStencilBits
 * @param {tcuTexture.PixelBufferAccess} stencilBuffer
 */
rrFragmentOperations.executeStencilCompare = function(inputFragments, stencilState, numStencilBits, stencilBuffer) {
    var clampedStencilRef = deMath.clamp(stencilState.ref, 0, (1 << numStencilBits) - 1);

    /**
     * @param {function(number=,number=):boolean} expression
     */
    var sample_register_stencil_compare = function(expression) {
        for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var fragSampleNdx = 0;
                var stencilBufferValue = stencilBuffer.getPixStencil(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                var maskedRef = stencilState.compMask & clampedStencilRef;
                var maskedBuf = stencilState.compMask & stencilBufferValue;
                frag.stencilPassed = expression(maskedRef, maskedBuf);
            }
        }
    };

    switch (stencilState.func) {
        case rrRenderState.TestFunc.NEVER: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return false;}); break;
        case rrRenderState.TestFunc.ALWAYS: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return true;}); break;
        case rrRenderState.TestFunc.LESS: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return maskedRef < maskedBuf;}); break;
        case rrRenderState.TestFunc.LEQUAL: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return maskedRef <= maskedBuf;}); break;
        case rrRenderState.TestFunc.GREATER: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return maskedRef > maskedBuf;}); break;
        case rrRenderState.TestFunc.GEQUAL: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return maskedRef >= maskedBuf;}); break;
        case rrRenderState.TestFunc.EQUAL: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return maskedRef == maskedBuf;}); break;
        case rrRenderState.TestFunc.NOTEQUAL: sample_register_stencil_compare(function(maskedRef, maskedBuf) { return maskedRef != maskedBuf;}); break;
        default:
            throw new Error('Unrecognized stencil test function:' + stencilState.func);
    }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {rrRenderState.StencilState} stencilState
 * @param {number} numStencilBits
 * @param {tcuTexture.PixelBufferAccess} stencilBuffer
 */
rrFragmentOperations.executeStencilSFail = function(inputFragments, stencilState, numStencilBits, stencilBuffer) {
    var clampedStencilRef = deMath.clamp(stencilState.ref, 0, (1 << numStencilBits) - 1);
    /**
     * @param {function(number,number):number} expression
     */
    var sample_register_sfail = function(expression) {
       for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive && !frag.stencilPassed) {
                var fragSampleNdx = 0;
                var stencilBufferValue = stencilBuffer.getPixStencil(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                stencilBuffer.setPixStencil(rrFragmentOperations.maskedBitReplace(stencilBufferValue, expression(stencilBufferValue, numStencilBits), stencilState.writeMask), fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                frag.isAlive = false;
            }
        }
    };

    switch (stencilState.sFail) {
        case rrRenderState.StencilOp.KEEP:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return stencilBufferValue;}); break;
        case rrRenderState.StencilOp.ZERO:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return 0;}); break;
        case rrRenderState.StencilOp.REPLACE:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return clampedStencilRef;}); break;
        case rrRenderState.StencilOp.INCR:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return deMath.clamp(stencilBufferValue + 1, 0, (1 << numStencilBits) - 1);}); break;
        case rrRenderState.StencilOp.DECR:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return deMath.clamp(stencilBufferValue - 1, 0, (1 << numStencilBits) - 1);}); break;
        case rrRenderState.StencilOp.INCR_WRAP:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return (stencilBufferValue + 1) & ((1 << numStencilBits) - 1);}); break;
        case rrRenderState.StencilOp.DECR_WRAP:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return (stencilBufferValue - 1) & ((1 << numStencilBits) - 1);}); break;
        case rrRenderState.StencilOp.INVERT:
            sample_register_sfail(function(stencilBufferValue, numStencilBits) { return (~stencilBufferValue) & ((1 << numStencilBits) - 1);}); break;
        default:
            throw new Error('Unrecognized stencil op:' + stencilState.sFail);
    }

};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {rrRenderState.TestFunc} depthFunc
 * @param {tcuTexture.PixelBufferAccess} depthBuffer
 */
rrFragmentOperations.executeDepthCompare = function(inputFragments, depthFunc, depthBuffer) {
    /**
     * @param {function(number=,number=):boolean} expression
     */
    var convertToDepthBuffer = false;

    var access;
    if (depthBuffer.getFormat().type != tcuTexture.ChannelType.FLOAT &&
        depthBuffer.getFormat().type != tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV) {
        access = new tcuTexture.PixelBufferAccess({
            format: depthBuffer.getFormat(),
            width: 1,
            height: 1,
            depth: 1,
            data: new ArrayBuffer(8)
        });
        convertToDepthBuffer = true;
    }

    var sample_register_depth_compare = function(expression) {
      for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var fragSampleNdx = 0;
                var depthBufferValue = depthBuffer.getPixDepth(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                var sampleDepthFloat = frag.sampleDepths[fragSampleNdx];

                var sampleDepth;
                if (convertToDepthBuffer) {
                    /* convert input float to target buffer format for comparison */
                    access.setPixDepth(sampleDepthFloat, 0, 0, 0);
                    sampleDepth = access.getPixDepth(0, 0, 0);
                } else {
                    sampleDepth = deMath.clamp(sampleDepthFloat, 0.0, 1.0);
                }

                frag.depthPassed = expression(sampleDepth, depthBufferValue);
            }
        }
    };

    switch (depthFunc) {
        case rrRenderState.TestFunc.NEVER: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return false;}); break;
        case rrRenderState.TestFunc.ALWAYS: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return true;}); break;
        case rrRenderState.TestFunc.LESS: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return sampleDepth < depthBufferValue;}); break;
        case rrRenderState.TestFunc.LEQUAL: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return sampleDepth <= depthBufferValue;}); break;
        case rrRenderState.TestFunc.GREATER: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return sampleDepth > depthBufferValue;}); break;
        case rrRenderState.TestFunc.GEQUAL: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return sampleDepth >= depthBufferValue;}); break;
        case rrRenderState.TestFunc.EQUAL: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return sampleDepth == depthBufferValue;}); break;
        case rrRenderState.TestFunc.NOTEQUAL: sample_register_depth_compare(function(sampleDepth, depthBufferValue) { return sampleDepth != depthBufferValue;}); break;
        default:
            throw new Error('Unrecognized depth function:' + depthFunc);
    }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {tcuTexture.PixelBufferAccess} depthBuffer
 */
rrFragmentOperations.executeDepthWrite = function(inputFragments, depthBuffer) {
      for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive && frag.depthPassed) {
                var fragSampleNdx = 0;
                var clampedDepth = deMath.clamp(frag.sampleDepths[fragSampleNdx], 0.0, 1.0);
                depthBuffer.setPixDepth(clampedDepth, fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
            }
        }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {rrRenderState.StencilState} stencilState
 * @param {number} numStencilBits
 * @param {tcuTexture.PixelBufferAccess} stencilBuffer
 */
rrFragmentOperations.executeStencilDpFailAndPass = function(inputFragments, stencilState, numStencilBits, stencilBuffer) {
   var clampedStencilRef = deMath.clamp(stencilState.ref, 0, (1 << numStencilBits) - 1);

    /**
     * @param {function(boolean):boolean} condition
     * @param {function(number,number):number} expression
     */
    var sample_register_dpfail_or_dppass = function(condition, expression) {
          for (var i = 0; i < inputFragments.length; i++) {
                var frag = inputFragments[i];
                if (frag.isAlive && condition(frag.depthPassed)) {
                    var fragSampleNdx = 0;
                    var stencilBufferValue = stencilBuffer.getPixStencil(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                    stencilBuffer.setPixStencil(rrFragmentOperations.maskedBitReplace(stencilBufferValue, expression(stencilBufferValue, numStencilBits), stencilState.writeMask), fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                }
        }
    };

    var switch_dpfail_or_dppass = function(op_name, condition) {
        switch (stencilState[op_name]) {
            case rrRenderState.StencilOp.KEEP: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return stencilBufferValue;}); break;
            case rrRenderState.StencilOp.ZERO: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return 0;}); break;
            case rrRenderState.StencilOp.REPLACE: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return clampedStencilRef;}); break;
            case rrRenderState.StencilOp.INCR: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return deMath.clamp(stencilBufferValue + 1, 0, (1 << numStencilBits) - 1);}); break;
            case rrRenderState.StencilOp.DECR: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return deMath.clamp(stencilBufferValue - 1, 0, (1 << numStencilBits) - 1);}); break;
            case rrRenderState.StencilOp.INCR_WRAP: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return (stencilBufferValue + 1) & ((1 << numStencilBits) - 1);}); break;
            case rrRenderState.StencilOp.DECR_WRAP: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return (stencilBufferValue - 1) & ((1 << numStencilBits) - 1);}); break;
            case rrRenderState.StencilOp.INVERT: sample_register_dpfail_or_dppass(condition, function(stencilBufferValue, numStencilBits) { return (~stencilBufferValue) & ((1 << numStencilBits) - 1);}); break;
            default:
                throw new Error('Unrecognized stencil operation:' + op_name);
        }
    };

    var passed = function(depthPassed) { return depthPassed;};
    var failed = function(depthPassed) { return !depthPassed;};

    switch_dpfail_or_dppass('dpFail', failed);
    switch_dpfail_or_dppass('dpPass', passed);
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {Array<number>} blendColor
 * @param {rrRenderState.BlendState} blendRGBState
 */
rrFragmentOperations.executeBlendFactorComputeRGB = function(inputFragments, blendColor, blendRGBState) {
    /**
     * @param {string} factor_name
     * @param {function(Array<number>, Array<number>, Array<number>):Array<number>} expression
     */
    var sample_register_blend_factor = function(factor_name, expression) {
       for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var src = frag.clampedBlendSrcColor;
                var src1 = frag.clampedBlendSrc1Color;
                var dst = frag.clampedBlendDstColor;
                frag[factor_name] = deMath.clampVector(expression(src, src1, dst), 0, 1);
            }
        }
    };

    var switch_src_or_dst_factor_rgb = function(func_name, factor_name) {
        switch (blendRGBState[func_name]) {
            case rrRenderState.BlendFunc.ZERO:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [0, 0, 0];}); break;
            case rrRenderState.BlendFunc.ONE:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [1, 1, 1];}); break;
            case rrRenderState.BlendFunc.SRC_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.swizzle(src, [0, 1, 2]);}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.subtract([1, 1, 1], deMath.swizzle(src, [0, 1, 2]));}); break;
            case rrRenderState.BlendFunc.DST_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.swizzle(dst, [0, 1, 2]);}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_DST_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.subtract([1, 1, 1], deMath.swizzle(dst, [0, 1, 2]));}); break;
            case rrRenderState.BlendFunc.SRC_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [src[3], src[3], src[3]];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [1.0 - src[3], 1.0 - src[3], 1.0 - src[3]];}); break;
            case rrRenderState.BlendFunc.DST_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [dst[3], dst[3], dst[3]];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_DST_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [1.0 - dst[3], 1.0 - dst[3], 1.0 - dst[3]];}); break;
            case rrRenderState.BlendFunc.CONSTANT_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.swizzle(blendColor, [0, 1, 2]);}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_CONSTANT_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.subtract([1, 1, 1], deMath.swizzle(blendColor, [0, 1, 2]));}); break;
            case rrRenderState.BlendFunc.CONSTANT_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [blendColor[3], blendColor[3], blendColor[3]];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_CONSTANT_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [1.0 - blendColor[3], 1.0 - blendColor[3], 1.0 - blendColor[3]];}); break;
            case rrRenderState.BlendFunc.SRC_ALPHA_SATURATE:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [Math.min(src[3], 1.0 - dst[3]), Math.min(src[3], 1.0 - dst[3]), Math.min(src[3], 1.0 - dst[3])];}); break;
            case rrRenderState.BlendFunc.SRC1_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.swizzle(src1, [0, 1, 2]);}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC1_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return deMath.subtract([1, 1, 1], deMath.swizzle(src1, [0, 1, 2]));}); break;
            case rrRenderState.BlendFunc.SRC1_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [src1[3], src1[3], src1[3]];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC1_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return [1.0 - src1[3], 1.0 - src1[3], 1.0 - src1[3]];}); break;
            default:
                throw new Error('Unrecognized blend function:' + func_name);
            }
    };

    switch_src_or_dst_factor_rgb('srcFunc', 'blendSrcFactorRGB');
    switch_src_or_dst_factor_rgb('dstFunc', 'blendDstFactorRGB');

};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {Array<number>} blendColor
 * @param {rrRenderState.BlendState} blendAState
 */
rrFragmentOperations.executeBlendFactorComputeA = function(inputFragments, blendColor, blendAState) {
    /**
     * @param {string} factor_name
     * @param {function(Array<number>, Array<number>, Array<number>):number} expression
     */
    var sample_register_blend_factor = function(factor_name, expression) {
       for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var src = frag.clampedBlendSrcColor;
                var src1 = frag.clampedBlendSrc1Color;
                var dst = frag.clampedBlendDstColor;
                frag[factor_name] = deMath.clamp(expression(src, src1, dst), 0, 1);
            }
        }
    };

    var swictch_src_or_dst_factor_a = function(func_name, factor_name) {
        switch (blendAState[func_name]) {
            case rrRenderState.BlendFunc.ZERO:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 0.0;}); break;
            case rrRenderState.BlendFunc.ONE:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0;}); break;
            case rrRenderState.BlendFunc.SRC_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return src[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - src[3];}); break;
            case rrRenderState.BlendFunc.DST_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return dst[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_DST_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - dst[3];}); break;
            case rrRenderState.BlendFunc.SRC_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return src[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - src[3];}); break;
            case rrRenderState.BlendFunc.DST_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return dst[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_DST_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - dst[3];}); break;
            case rrRenderState.BlendFunc.CONSTANT_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return blendColor[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_CONSTANT_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - blendColor[3];}); break;
            case rrRenderState.BlendFunc.CONSTANT_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return blendColor[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_CONSTANT_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - blendColor[3];}); break;
            case rrRenderState.BlendFunc.SRC_ALPHA_SATURATE:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0;}); break;
            case rrRenderState.BlendFunc.SRC1_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return src1[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC1_COLOR:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - src1[3];}); break;
            case rrRenderState.BlendFunc.SRC1_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return src1[3];}); break;
            case rrRenderState.BlendFunc.ONE_MINUS_SRC1_ALPHA:
                sample_register_blend_factor(factor_name, function(src, src1, dst) { return 1.0 - src1[3];}); break;
            default:
                throw new Error('Unrecognized blend function:' + func_name);
        }
    };

    swictch_src_or_dst_factor_a('srcFunc', 'blendSrcFactorA');
    swictch_src_or_dst_factor_a('dstFunc', 'blendDstFactorA');
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments Fragments to write
 * @param {rrRenderState.BlendState} blendRGBState
 * @param {rrRenderState.BlendState} blendAState
 */
rrFragmentOperations.executeBlend = function(inputFragments, blendRGBState, blendAState) {
    var sample_register_blended_color = function(color_name, expression) {
       for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var src = frag.clampedBlendSrcColor;
                var dst = frag.clampedBlendDstColor;
                frag[color_name] = expression(src, dst, frag);
            }
        }
    };

    switch (blendRGBState.equation) {
        case rrRenderState.BlendEquation.ADD:
            sample_register_blended_color('blendedRGB', function(src, dst, frag) { return deMath.add(deMath.multiply(deMath.swizzle(src, [0, 1, 2]), frag.blendSrcFactorRGB), deMath.multiply(deMath.swizzle(dst, [0, 1, 2]), frag.blendDstFactorRGB));}); break;
        case rrRenderState.BlendEquation.SUBTRACT:
            sample_register_blended_color('blendedRGB', function(src, dst, frag) { return deMath.subtract(deMath.multiply(deMath.swizzle(src, [0, 1, 2]), frag.blendSrcFactorRGB), deMath.multiply(deMath.swizzle(dst, [0, 1, 2]), frag.blendDstFactorRGB));}); break;
        case rrRenderState.BlendEquation.REVERSE_SUBTRACT:
            sample_register_blended_color('blendedRGB', function(src, dst, frag) { return deMath.subtract(deMath.multiply(deMath.swizzle(dst, [0, 1, 2]), frag.blendDstFactorRGB), deMath.multiply(deMath.swizzle(src, [0, 1, 2]), frag.blendSrcFactorRGB));}); break;
        case rrRenderState.BlendEquation.MIN:
            sample_register_blended_color('blendedRGB', function(src, dst, frag) { return deMath.min(deMath.swizzle(src, [0, 1, 2]), deMath.swizzle(dst, [0, 1, 2]));}); break;
        case rrRenderState.BlendEquation.MAX:
            sample_register_blended_color('blendedRGB', function(src, dst, frag) { return deMath.max(deMath.swizzle(src, [0, 1, 2]), deMath.swizzle(dst, [0, 1, 2]));}); break;
        default:
            throw new Error('Unrecognized blend equation:' + blendRGBState.equation);
    }

    switch (blendAState.equation) {
        case rrRenderState.BlendEquation.ADD:
            sample_register_blended_color('blendedA', function(src, dst, frag) { return src[3] * frag.blendSrcFactorA + dst[3] * frag.blendDstFactorA;}); break;
        case rrRenderState.BlendEquation.SUBTRACT:
            sample_register_blended_color('blendedA', function(src, dst, frag) { return src[3] * frag.blendSrcFactorA - dst[3] * frag.blendDstFactorA;}); break;
        case rrRenderState.BlendEquation.REVERSE_SUBTRACT:
            sample_register_blended_color('blendedA', function(src, dst, frag) { return dst[3] * frag.blendDstFactorA - src[3] * frag.blendSrcFactorA;}); break;
        case rrRenderState.BlendEquation.MIN:
            sample_register_blended_color('blendedA', function(src, dst, frag) { return Math.min(src[3], dst[3]);}); break;
        case rrRenderState.BlendEquation.MAX:
            sample_register_blended_color('blendedA', function(src, dst, frag) { return Math.max(src[3], dst[3]);}); break;
        default:
            throw new Error('Unrecognized blend equation:' + blendAState.equation);
    }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments
 * @param {boolean} isSRGB
 * @param {tcuTexture.PixelBufferAccess} colorBuffer
 */
rrFragmentOperations.executeColorWrite = function(inputFragments, isSRGB, colorBuffer) {
       for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var combinedColor = frag.blendedRGB.slice();
                combinedColor[3] = frag.blendedA;
                if (isSRGB)
                    combinedColor = tcuTextureUtil.linearToSRGB(combinedColor);

                colorBuffer.setPixel(combinedColor, 0, frag.pixelCoord[0], frag.pixelCoord[1]);
        }
    }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments
 * @param {Array<boolean>} colorMaskFactor
 * @param {Array<boolean>} colorMaskNegationFactor
 * @param {boolean} isSRGB
 * @param {tcuTexture.PixelBufferAccess} colorBuffer
 */
rrFragmentOperations.executeMaskedColorWrite = function(inputFragments, colorMaskFactor, colorMaskNegationFactor, isSRGB, colorBuffer) {
       for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var fragSampleNdx = 0;
                var originalColor = colorBuffer.getPixel(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                var newColor = frag.blendedRGB.slice();
                newColor[3] = frag.blendedA;

                if (isSRGB)
                    newColor = tcuTextureUtil.linearToSRGB(newColor);

                newColor = deMath.add(deMath.multiply(colorMaskFactor, newColor), deMath.multiply(colorMaskNegationFactor, originalColor));

                colorBuffer.setPixel(newColor, fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
            }
        }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments
 * @param {Array<boolean>} colorMask
 * @param {tcuTexture.PixelBufferAccess} colorBuffer
 */
rrFragmentOperations.executeSignedValueWrite = function(inputFragments, colorMask, colorBuffer) {
      for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var fragSampleNdx = 0;
                var originalValue = colorBuffer.getPixelInt(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                var newValue = tcuTextureUtil.select(frag.signedValue, originalValue, colorMask);

                colorBuffer.setPixelInt(newValue, fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
            }
        }
};

/**
 * @param {Array<rrFragmentOperations.Fragment>} inputFragments
 * @param {Array<boolean>} colorMask
 * @param {tcuTexture.PixelBufferAccess} colorBuffer
 */
rrFragmentOperations.executeUnsignedValueWrite = function(inputFragments, colorMask, colorBuffer) {
      for (var i = 0; i < inputFragments.length; i++) {
            var frag = inputFragments[i];
            if (frag.isAlive) {
                var fragSampleNdx = 0;
                var originalValue = colorBuffer.getPixelInt(fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
                var newValue = tcuTextureUtil.select(frag.unsignedValue, originalValue, colorMask);

                colorBuffer.setPixelInt(newValue, fragSampleNdx, frag.pixelCoord[0], frag.pixelCoord[1]);
            }
        }
};

/**
 * @constructor
 */
rrFragmentOperations.FragmentProcessor = function() {
    /* TODO: implement */
};

});

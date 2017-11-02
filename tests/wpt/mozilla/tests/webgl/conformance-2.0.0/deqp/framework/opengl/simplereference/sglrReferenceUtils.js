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
 *//*!
 * \file
 * \brief Reference context utils
 *//*--------------------------------------------------------------------*/

'use strict';
goog.provide('framework.opengl.simplereference.sglrReferenceUtils');
goog.require('framework.common.tcuFloat');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrRenderState');
goog.require('framework.referencerenderer.rrRenderer');
goog.require('framework.referencerenderer.rrShaders');
goog.require('framework.referencerenderer.rrVertexAttrib');

goog.scope(function() {

    var sglrReferenceUtils = framework.opengl.simplereference.sglrReferenceUtils;
    var deMath = framework.delibs.debase.deMath;
    var tcuFloat = framework.common.tcuFloat;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
    var rrRenderer = framework.referencerenderer.rrRenderer;
    var rrDefs = framework.referencerenderer.rrDefs;
    var rrShaders = framework.referencerenderer.rrShaders;
    var rrRenderState = framework.referencerenderer.rrRenderState;

    /**
    * @param {number} type (32-bit, unsigend)
    * @return {rrVertexAttrib.VertexAttribType}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLPureIntegerVertexAttributeType = function(type) {
        switch (type) {
            case gl.UNSIGNED_BYTE: return rrVertexAttrib.VertexAttribType.PURE_UINT8;
            case gl.UNSIGNED_SHORT: return rrVertexAttrib.VertexAttribType.PURE_UINT16;
            case gl.UNSIGNED_INT: return rrVertexAttrib.VertexAttribType.PURE_UINT32;
            case gl.BYTE: return rrVertexAttrib.VertexAttribType.PURE_INT8;
            case gl.SHORT: return rrVertexAttrib.VertexAttribType.PURE_INT16;
            case gl.INT: return rrVertexAttrib.VertexAttribType.PURE_INT32;
            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} type (32-bit, unsigend)
    * @param {boolean} normalizedInteger
    * @param {number} size
    * @return {rrVertexAttrib.VertexAttribType} converted value from type to VertexAttribType
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLFloatVertexAttributeType = function(type, normalizedInteger, size) {

        /** @type {boolean} */ var useClampingNormalization = true;

        switch (type) {
            case gl.FLOAT:
                return rrVertexAttrib.VertexAttribType.FLOAT;
            case gl.HALF_FLOAT:
                return rrVertexAttrib.VertexAttribType.HALF;
            /* Not supported in WebGL 1/2 case gl.FIXED:
                return rrVertexAttrib.VertexAttribType.FIXED;
            case gl.DOUBLE:
                return rrVertexAttrib.VertexAttribType.DOUBLE; */
            case gl.UNSIGNED_BYTE:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_UINT8;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_UNORM8;

            case gl.UNSIGNED_SHORT:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_UINT16;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_UNORM16;

            case gl.UNSIGNED_INT:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_UINT32;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_UNORM32;

            case gl.UNSIGNED_INT_2_10_10_10_REV:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_UINT_2_10_10_10_REV;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV;

            case gl.BYTE:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_INT8;
                else if (useClampingNormalization)
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM8_CLAMP;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM8_SCALE;

            case gl.SHORT:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_INT16;
                else if (useClampingNormalization)
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM16_CLAMP;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM16_SCALE;

            case gl.INT:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_INT32;
                else if (useClampingNormalization)
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM32_CLAMP;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM32_SCALE;

            case gl.INT_2_10_10_10_REV:
                if (!normalizedInteger)
                    return rrVertexAttrib.VertexAttribType.NONPURE_INT_2_10_10_10_REV;
                else if (useClampingNormalization)
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP;
                else
                    return rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE;

            default:
                throw new Error('Value to do mapping not compatible');

        }

    };

    /**
    * @param {number} size
    * @return {number}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLSize = function(size) {
        switch (size) {
            case 1: return 1;
            case 2: return 2;
            case 3: return 3;
            case 4: return 4;
            /* NOT in GL
            case gl.BGRA: return 4;
            */

            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} type (32-bit, unsigned)
    * @return {rrRenderer.PrimitiveType}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLPrimitiveType = function(type) {
        switch (type) {
            case gl.TRIANGLES: return rrRenderer.PrimitiveType.TRIANGLES;
            case gl.TRIANGLE_STRIP: return rrRenderer.PrimitiveType.TRIANGLE_STRIP;
            case gl.TRIANGLE_FAN: return rrRenderer.PrimitiveType.TRIANGLE_FAN;
            case gl.LINES: return rrRenderer.PrimitiveType.LINES;
            case gl.LINE_STRIP: return rrRenderer.PrimitiveType.LINE_STRIP;
            case gl.LINE_LOOP: return rrRenderer.PrimitiveType.LINE_LOOP;
            case gl.POINTS: return rrRenderer.PrimitiveType.POINTS;

            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} type (32-bit, unsigned)
    * @return {rrDefs.IndexType}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLIndexType = function(type) {
        switch (type) {
            case gl.UNSIGNED_BYTE: return rrDefs.IndexType.INDEXTYPE_UINT8;
            case gl.UNSIGNED_SHORT: return rrDefs.IndexType.INDEXTYPE_UINT16;
            case gl.UNSIGNED_INT: return rrDefs.IndexType.INDEXTYPE_UINT32;
            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} func (deUint32)
    * @return {rrRenderState.TestFunc}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLTestFunc = function(func) {
        switch (func) {
            case gl.ALWAYS: return rrRenderState.TestFunc.ALWAYS;
            case gl.EQUAL: return rrRenderState.TestFunc.EQUAL;
            case gl.GEQUAL: return rrRenderState.TestFunc.GEQUAL;
            case gl.GREATER: return rrRenderState.TestFunc.GREATER;
            case gl.LEQUAL: return rrRenderState.TestFunc.LEQUAL;
            case gl.LESS: return rrRenderState.TestFunc.LESS;
            case gl.NEVER: return rrRenderState.TestFunc.NEVER;
            case gl.NOTEQUAL: return rrRenderState.TestFunc.NOTEQUAL;
            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} op (deUint32)
    * @return {rrRenderState.StencilOp}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLStencilOp = function(op) {
        switch (op) {
            case gl.KEEP: return rrRenderState.StencilOp.KEEP;
            case gl.ZERO: return rrRenderState.StencilOp.ZERO;
            case gl.REPLACE: return rrRenderState.StencilOp.REPLACE;
            case gl.INCR: return rrRenderState.StencilOp.INCR;
            case gl.DECR: return rrRenderState.StencilOp.DECR;
            case gl.INCR_WRAP: return rrRenderState.StencilOp.INCR_WRAP;
            case gl.DECR_WRAP: return rrRenderState.StencilOp.DECR_WRAP;
            case gl.INVERT: return rrRenderState.StencilOp.INVERT;
            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} equation (deUint32)
    * @return {rrRenderState.BlendEquation}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLBlendEquation = function(equation) {
        switch (equation) {
            case gl.FUNC_ADD: return rrRenderState.BlendEquation.ADD;
            case gl.FUNC_SUBTRACT: return rrRenderState.BlendEquation.SUBTRACT;
            case gl.FUNC_REVERSE_SUBTRACT: return rrRenderState.BlendEquation.REVERSE_SUBTRACT;
            case gl.MIN: return rrRenderState.BlendEquation.MIN;
            case gl.MAX: return rrRenderState.BlendEquation.MAX;
            default:
                throw new Error('Value to do mapping not compatible');
        }
    };

    /**
    * @param {number} equation (deUint32)
    * @return {rrRenderState.BlendEquationAdvanced}
    * @throws {Error}
    */
    /*sglrReferenceUtils.mapGLBlendEquationAdvanced = function(equation) {
        switch (equation) {
            case gl.MULTIPLY_KHR: return rrRenderState.BlendEquationAdvanced.MULTIPLY;
            case gl.SCREEN_KHR: return rrRenderState.BlendEquationAdvanced.SCREEN;
            case gl.OVERLAY_KHR: return rrRenderState.BlendEquationAdvanced.OVERLAY;
            case gl.DARKEN_KHR: return rrRenderState.BlendEquationAdvanced.DARKEN;
            case gl.LIGHTEN_KHR: return rrRenderState.BlendEquationAdvanced.LIGHTEN;
            case gl.COLORDODGE_KHR: return rrRenderState.BlendEquationAdvanced.COLORDODGE;
            case gl.COLORBURN_KHR: return rrRenderState.BlendEquationAdvanced.COLORBURN;
            case gl.HARDLIGHT_KHR: return rrRenderState.BlendEquationAdvanced.HARDLIGHT;
            case gl.SOFTLIGHT_KHR: return rrRenderState.BlendEquationAdvanced.SOFTLIGHT;
            case gl.DIFFERENCE_KHR: return rrRenderState.BlendEquationAdvanced.DIFFERENCE;
            case gl.EXCLUSION_KHR: return rrRenderState.BlendEquationAdvanced.EXCLUSION;
            case gl.HSL_HUE_KHR: return rrRenderState.BlendEquationAdvanced.HSL_HUE;
            case gl.HSL_SATURATION_KHR: return rrRenderState.BlendEquationAdvanced.HSL_SATURATION;
            case gl.HSL_COLOR_KHR: return rrRenderState.BlendEquationAdvanced.HSL_COLOR;
            case gl.HSL_LUMINOSITY_KHR: return rrRenderState.BlendEquationAdvanced.HSL_LUMINOSITY;
            default:
                throw new Error("Value to do mapping not compatible");
        }
    };*/

    /**
    * @param {number} func (deUint32)
    * @return {rrRenderState.BlendFunc}
    * @throws {Error}
    */
    sglrReferenceUtils.mapGLBlendFunc = function(func) {
        switch (func) {
            case gl.ZERO: return rrRenderState.BlendFunc.ZERO;
            case gl.ONE: return rrRenderState.BlendFunc.ONE;
            case gl.SRC_COLOR: return rrRenderState.BlendFunc.SRC_COLOR;
            case gl.ONE_MINUS_SRC_COLOR: return rrRenderState.BlendFunc.ONE_MINUS_SRC_COLOR;
            case gl.DST_COLOR: return rrRenderState.BlendFunc.DST_COLOR;
            case gl.ONE_MINUS_DST_COLOR: return rrRenderState.BlendFunc.ONE_MINUS_DST_COLOR;
            case gl.SRC_ALPHA: return rrRenderState.BlendFunc.SRC_ALPHA;
            case gl.ONE_MINUS_SRC_ALPHA: return rrRenderState.BlendFunc.ONE_MINUS_SRC_ALPHA;
            case gl.DST_ALPHA: return rrRenderState.BlendFunc.DST_ALPHA;
            case gl.ONE_MINUS_DST_ALPHA: return rrRenderState.BlendFunc.ONE_MINUS_DST_ALPHA;
            case gl.CONSTANT_COLOR: return rrRenderState.BlendFunc.CONSTANT_COLOR;
            case gl.ONE_MINUS_CONSTANT_COLOR: return rrRenderState.BlendFunc.ONE_MINUS_CONSTANT_COLOR;
            case gl.CONSTANT_ALPHA: return rrRenderState.BlendFunc.CONSTANT_ALPHA;
            case gl.ONE_MINUS_CONSTANT_ALPHA: return rrRenderState.BlendFunc.ONE_MINUS_CONSTANT_ALPHA;
            case gl.SRC_ALPHA_SATURATE: return rrRenderState.BlendFunc.SRC_ALPHA_SATURATE;
    //         case gl.SRC1_COLOR: return rrRenderState.BlendFunc.SRC1_COLOR;
    //         case gl.ONE_MINUS_SRC1_COLOR: return rrRenderState.BlendFunc.ONE_MINUS_SRC1_COLOR;
    //         case gl.SRC1_ALPHA: return rrRenderState.BlendFunc.SRC1_ALPHA;
    //         case gl.ONE_MINUS_SRC1_ALPHA: return rrRenderState.BlendFunc.ONE_MINUS_SRC1_ALPHA;
            default:
                throw new Error('Value to do mapping not compatible');
        }
    };
});

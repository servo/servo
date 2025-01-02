/*-------------------------------------------------------------------------
 * drawElements Quality Program Reference Renderer
 * -----------------------------------------------
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
 * \brief Reference renderer render state.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('framework.referencerenderer.rrRenderState');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrMultisamplePixelBufferAccess');

goog.scope(function() {

var rrRenderState = framework.referencerenderer.rrRenderState;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var rrMultisamplePixelBufferAccess = framework.referencerenderer.rrMultisamplePixelBufferAccess;
var rrDefs = framework.referencerenderer.rrDefs;

/**
 * Enum for rrRenderState.HorizontalFill values.
 * @enum {number}
 */
rrRenderState.HorizontalFill = {
    LEFT: 0,
    RIGHT: 1
};

/**
 * Enum for rrRenderState.VerticalFill values.
 * @enum {number}
 */
rrRenderState.VerticalFill = {
    TOP: 0,
    BOTTOM: 1
};

/**
 * Enum for rrRenderState.Winding values.
 * @enum {number}
 */
rrRenderState.Winding = {
    CCW: 0,
    CC: 1
};

/**
 * Enum for rrRenderState.CullMode values.
 * @enum {number}
 */
rrRenderState.CullMode = {
    NONE: 0,
    BACK: 1,
    FRONT: 2
};

/**rrRenderState.Winding : rrRenderState.Winding,

 * @constructor
 */
rrRenderState.RasterizationState = function() {
    /** @type {number} */ this.winding = rrRenderState.Winding.CCW;
    /** @type {number} */ this.horizontalFill = rrRenderState.HorizontalFill.LEFT;
    /** @type {number} */ this.verticalFill = rrRenderState.VerticalFill.BOTTOM;
};

/**
 * Enum for rrRenderState.TestFunc values.
 * @enum {number}
 */
rrRenderState.TestFunc = {
    NEVER: 0,
    ALWAYS: 1,
    LESS: 2,
    LEQUAL: 3,
    GREATER: 4,
    GEQUAL: 5,
    EQUAL: 6,
    NOTEQUAL: 7
};

/**
 * Enum for rrRenderState.StencilOp values.
 * @enum {number}
 */
rrRenderState.StencilOp = {
    KEEP: 0,
    ZERO: 1,
    REPLACE: 2,
    INCR: 3, //!< Increment with saturation.
    DECR: 4, //!< Decrement with saturation.
    INCR_WRAP: 5,
    DECR_WRAP: 6,
    INVERT: 7
};

/**
 * Enum for rrRenderState.BlendMode values.
 * @enum {number}
 */
rrRenderState.BlendMode = {
    NONE: 0, //!< No blending.
    STANDARD: 1 //!< Standard blending.
// Advanced blending is not supported
//    ADVANCED : 2 //!< Advanced blending mode, as defined in gl.KHR_blend_equation_advanced.
};

/**
 * Enum for rrRenderState.BlendEquation values.
 * @enum {number}
 */
rrRenderState.BlendEquation = {
    ADD: 0,
    SUBTRACT: 1,
    REVERSE_SUBTRACT: 2,
    MIN: 3,
    MAX: 4
};

// /**
//  * Enum for rrRenderState.BlendEquationAdvanced values.
//  * @enum {number}
//  */
// rrRenderState.BlendEquationAdvanced = {
//     MULTIPLY : 0,
//     SCREEN : 1,
//     OVERLAY : 2,
//     DARKEN : 3,
//     LIGHTEN : 4,
//     COLORDODGE : 5,
//     COLORBURN : 6,
//     HARDLIGHT : 7,
//     SOFTLIGHT : 8,
//     DIFFERENCE : 9,
//     EXCLUSION : 10,
//     HSL_HUE : 11,
//     HSL_SATURATION : 12,
//     HSL_COLOR : 13,
//     HSL_LUMINOSITY : 14
// };

/**
 * Enum for rrRenderState.BlendFunc values.
 * @enum {number}
 */
rrRenderState.BlendFunc = {
    ZERO: 0,
    ONE: 1,
    SRC_COLOR: 2,
    ONE_MINUS_SRC_COLOR: 3,
    DST_COLOR: 4,
    ONE_MINUS_DST_COLOR: 5,
    SRC_ALPHA: 6,
    ONE_MINUS_SRC_ALPHA: 7,
    DST_ALPHA: 8,
    ONE_MINUS_DST_ALPHA: 9,
    CONSTANT_COLOR: 10,
    ONE_MINUS_CONSTANT_COLOR: 11,
    CONSTANT_ALPHA: 12,
    ONE_MINUS_CONSTANT_ALPHA: 13,
    SRC_ALPHA_SATURATE: 14,
    SRC1_COLOR: 15,
    ONE_MINUS_SRC1_COLOR: 16,
    SRC1_ALPHA: 17,
    ONE_MINUS_SRC1_ALPHA: 18
};

/**
 * @constructor
 */
rrRenderState.StencilState = function() {
    /** @type {number} */ this.func = rrRenderState.TestFunc.ALWAYS;
    /** @type {number} */ this.ref = 0;
    /** @type {number} */ this.compMask = ~0;
    /** @type {number} */ this.sFail = rrRenderState.StencilOp.KEEP;
    /** @type {number} */ this.dpFail = rrRenderState.StencilOp.KEEP;
    /** @type {number} */ this.dpPass = rrRenderState.StencilOp.KEEP;
    /** @type {number} */ this.writeMask = ~0;
};

/**
 * @constructor
 */
rrRenderState.BlendState = function() {
    /** @type {number} */ this.equation = rrRenderState.BlendEquation.ADD;
    /** @type {number} */ this.srcFunc = rrRenderState.BlendFunc.ONE;
    /** @type {number} */ this.dstFunc = rrRenderState.BlendFunc.ZERO;
};

/**
 * @param {(Array<number>|number)} left_
 * @param {number=} bottom_
 * @param {number=} width_
 * @param {number=} height_
 * @constructor
 */
rrRenderState.WindowRectangle = function(left_, bottom_, width_, height_) {
    // Is first parameter an array? Use it
    if (left_.length && left_.length == 4) {
        this.left = left_[0];
        this.bottom = left_[1];
        this.width = left_[2];
        this.height = left_[3];
    } else {
        this.left = left_;
        this.bottom = bottom_;
        this.width = width_;
        this.height = height_;
    }
};

/**
 * @constructor
 */
rrRenderState.FragmentOperationState = function() {
    /** @type {boolean} */ this.scissorTestEnabled = false;
    /** @type {rrRenderState.WindowRectangle} */ this.scissorRectangle = new rrRenderState.WindowRectangle(0, 0, 1, 1);

    /** @type {boolean} */ this.stencilTestEnabled = false;

    /** @type {Array<rrRenderState.StencilState>} */ this.stencilStates = [];
    for (var type in rrDefs.FaceType)
        this.stencilStates[rrDefs.FaceType[type]] = new rrRenderState.StencilState();

    /** @type {boolean} */ this.depthTestEnabled = false;
    /** @type {rrRenderState.TestFunc} */ this.depthFunc = rrRenderState.TestFunc.LESS;
    /** @type {boolean} */ this.depthMask = true;

    /** @type {rrRenderState.BlendMode} */ this.blendMode = rrRenderState.BlendMode.NONE;
    /** @type {rrRenderState.BlendState} */ this.blendRGBState = new rrRenderState.BlendState();
    /** @type {rrRenderState.BlendState} */ this.blendAState = new rrRenderState.BlendState();
    /** @type {Array<number>} */ this.blendColor = [0.0, 0.0, 0.0, 0.0];
//    /** @type {rrRenderState.BlendEquationAdvanced} */ this.blendEquationAdvanced = null;

    /** @type {boolean} */ this.sRGBEnabled = true;

    /** @type {boolean} */ this.depthClampEnabled = false;

    /** @type {boolean} */ this.polygonOffsetEnabled = false;
    /** @type {number} */ this.polygonOffsetFactor = 0.0;
    /** @type {number} */ this.polygonOffsetUnits = 0.0;

    /** @type {Array<boolean>} */ this.colorMask = [true, true, true, true];

    /** @type {number} */ this.numStencilBits = 8;
};

/**
 * @constructor
 */
rrRenderState.PointState = function() {
    /** @type {number} */ this.pointSize = 1.0;
};

/**
 * @constructor
 */
rrRenderState.LineState = function() {
    /** @type {number} */ this.lineWidth = 1.0;
};

/**
 * Constructor checks if the parameter has a "raw" member to detect if the instance is
 * of type rrRenderState.WindowRectangle or MultisamplePixelBufferAccess.
 * @param {rrRenderState.WindowRectangle|rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} rect_
 * @constructor
 */
rrRenderState.ViewportState = function(rect_) {
    /** @type {number} */ this.zn = 0.0;
    /** @type {number} */ this.zf = 1.0;
    /** @type {rrRenderState.WindowRectangle} */ this.rect;

    if (rect_.raw) {
        this.rect = new rrRenderState.WindowRectangle(0, 0, rect_.raw().getHeight(),
            rect_.raw().getDepth());
    } else {
        this.rect = /** @type {rrRenderState.WindowRectangle} */ (rect_);
    }
};

/**
 * @constructor
 */
rrRenderState.RestartState = function() {
    /** @type {boolean} */ this.enabled = false;
    /** @type {number} */ this.restartIndex = 0xFFFFFFFF;
};

/**
 * @constructor
 * @param {rrRenderState.ViewportState} viewport_
 */
rrRenderState.RenderState = function(viewport_) {
    /** @type {rrRenderState.CullMode} */ this.cullMode = rrRenderState.CullMode.NONE;
    /** @type {number} */ this.provokingVertexConvention;
    /** @type {rrRenderState.ViewportState} */ this.viewport = viewport_;

    /** @type {rrRenderState.RasterizationState} */ this.rasterization = new rrRenderState.RasterizationState();
    /** @type {rrRenderState.FragmentOperationState} */ this.fragOps = new rrRenderState.FragmentOperationState();
    /** @type {rrRenderState.PointState} */ this.point = new rrRenderState.PointState();
    /** @type {rrRenderState.LineState} */ this.line = new rrRenderState.LineState();
    /** @type {rrRenderState.RestartState} */ this.restart = new rrRenderState.RestartState();
};

});

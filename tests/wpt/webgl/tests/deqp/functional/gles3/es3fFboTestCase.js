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
goog.provide('functional.gles3.es3fFboTestCase');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.referencerenderer.rrRenderer');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {

var es3fFboTestCase = functional.gles3.es3fFboTestCase;
var tcuTestCase = framework.common.tcuTestCase;
var deMath = framework.delibs.debase.deMath;
var tcuSurface = framework.common.tcuSurface;
var tcuTexture = framework.common.tcuTexture;
var rrRenderer = framework.referencerenderer.rrRenderer;
var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;
var tcuPixelFormat = framework.common.tcuPixelFormat;
var tcuImageCompare = framework.common.tcuImageCompare;
var deString = framework.delibs.debase.deString;
var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var deRandom = framework.delibs.debase.deRandom;

/** @typedef {(sglrGLContext.GLContext | WebGL2RenderingContext | sglrReferenceContext.ReferenceContext)} */
es3fFboTestCase.Context;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

    /**
    * es3fFboTestCase.FboTestCase class, inherits from TestCase and sglrContextWrapper
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    * @param {string} name
    * @param {string} description
    * @param {boolean=} useScreenSizedViewport
    */
    es3fFboTestCase.FboTestCase = function(name, description, useScreenSizedViewport /*= false */) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {number} */ this.m_viewportWidth = useScreenSizedViewport === undefined ? gl.drawingBufferWidth : 128;
        /** @type {number} */ this.m_viewportHeight = useScreenSizedViewport === undefined ? gl.drawingBufferHeight : 128;
        /** @type {es3fFboTestCase.Context} */ this.m_curCtx = null;
    };

    es3fFboTestCase.FboTestCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fFboTestCase.FboTestCase.prototype.constructor = es3fFboTestCase.FboTestCase;

    es3fFboTestCase.FboTestCase.prototype.getWidth = function() {
        return Math.min(gl.drawingBufferWidth, this.m_viewportWidth);
    };

    es3fFboTestCase.FboTestCase.prototype.getHeight = function() {
        return Math.min(gl.drawingBufferHeight, this.m_viewportHeight);
    };

    /**
     * Sets the current context (inherited from sglrContextWrapper)
     * @param {es3fFboTestCase.Context} context
     */
    es3fFboTestCase.FboTestCase.prototype.setContext = function(context) {
        this.m_curCtx = context;
    };

    /**
     * Gets the current context (inherited from sglrContextWrapper)
     * @return {es3fFboTestCase.Context}
     */
    es3fFboTestCase.FboTestCase.prototype.getCurrentContext = function() {
        return this.m_curCtx;
    };

    /**
     * @param {tcuSurface.Surface} reference
     * @param {tcuSurface.Surface} result
     */
    es3fFboTestCase.FboTestCase.prototype.compare = function(reference, result) {
        return tcuImageCompare.fuzzyCompare('Result', 'Image comparison result', reference.getAccess(), result.getAccess(), 0.05, tcuImageCompare.CompareLogMode.RESULT);
    };

    /**
     * @param {number} sizedFormat
     */
    es3fFboTestCase.FboTestCase.prototype.checkFormatSupport = function(sizedFormat) {
        /** @const @type {boolean} */ var isCoreFormat = es3fFboTestCase.isRequiredFormat(sizedFormat);
        /** @const @type {Array<string>} */ var requiredExts = (!isCoreFormat) ? es3fFboTestCase.getEnablingExtensions(sizedFormat) : [];

        // Check that we don't try to use invalid formats.
        DE_ASSERT(isCoreFormat || requiredExts);
        if (requiredExts.length > 0 && !es3fFboTestCase.isAnyExtensionSupported(gl, requiredExts)) {
            var msg = 'SKIP: Format ' + WebGLTestUtils.glEnumToString(gl, sizedFormat) + ' not supported';
            debug(msg);
            throw new TestFailedException(msg);
        }
    };

    /**
    * @param {number} sizedFormat deUint32
    * @param {number} numSamples
    */
    es3fFboTestCase.FboTestCase.prototype.checkSampleCount = function(sizedFormat, numSamples) {
        /** @const @type {number} */ var minSampleCount = es3fFboTestCase.getMinimumSampleCount(sizedFormat);

        if (numSamples > minSampleCount) {
            // Exceeds spec-mandated minimum - need to check.
            /** @const @type {goog.NumberArray} */ var supportedSampleCounts = es3fFboTestCase.querySampleCounts(sizedFormat);
            var supported = Array.prototype.slice.call(supportedSampleCounts);
            if (supported.indexOf(numSamples) == -1) {
                if (minSampleCount == 0 || numSamples > gl.getParameter(gl.MAX_SAMPLES)) {
                    checkMessage(false, "Sample count not supported, but it is allowed.");
                    return false;
                } else {
                    throw new Error('Sample count not supported');
                }
            }
            return true;
        }
        return true;
    };

    /**
    * @param {tcuSurface.Surface} dst
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    * @param {tcuTexture.TextureFormat} format
    * @param {Array<number>} scale Vec4
    * @param {Array<number>} bias Vec4
    */
    es3fFboTestCase.FboTestCase.prototype.readPixelsUsingFormat = function(dst, x, y, width, height, format, scale, bias) {
        dst.setSize(width, height);
        es3fFboTestUtil.readPixels(this.getCurrentContext(), dst, x, y, width, height, format, scale, bias);
    };

    /**
    * @param {tcuSurface.Surface} dst
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    */
    es3fFboTestCase.FboTestCase.prototype.readPixels = function(dst, x, y, width, height) {
        dst.readViewport(this.getCurrentContext(), [x, y, width, height]);
    };

    /**
    * @param {number} target
    */
    es3fFboTestCase.FboTestCase.prototype.checkFramebufferStatus = function(target) {
        /** @type {number} */ var status = this.getCurrentContext().checkFramebufferStatus(target);
        if (status != gl.FRAMEBUFFER_COMPLETE)
            throw new Error('Framebuffer Status: ' + WebGLTestUtils.glEnumToString(gl, status));
    };

    es3fFboTestCase.FboTestCase.prototype.checkError = function() {
        /** @type {number} */ var err = this.getCurrentContext().getError();
            if (err != gl.NO_ERROR)
                throw new Error('glError: ' + WebGLTestUtils.glEnumToString(gl, err));
    };

    /**
    * @param {tcuTexture.TextureFormat} format
    * @param {Array<number>=} value Vec4
    */
    es3fFboTestCase.FboTestCase.prototype.clearColorBuffer = function(format, value) {
        if (value === undefined) value = [0.0, 0.0, 0.0, 0.0];
        es3fFboTestUtil.clearColorBuffer(this.getCurrentContext(), format, value);
    };

    es3fFboTestCase.FboTestCase.prototype.iterate = function() {
        // Viewport.
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));
        /** @type {number} */ var width = Math.min(gl.drawingBufferWidth, this.m_viewportWidth);
        /** @type {number} */ var height = Math.min(gl.drawingBufferHeight, this.m_viewportHeight);
        /** @type {number} */ var x = rnd.getInt(0, gl.drawingBufferWidth - width);
        /** @type {number} */ var y = rnd.getInt(0, gl.drawingBufferHeight - height);

        // Surface format and storage is choosen by render().
        /** @type {tcuSurface.Surface} */ var reference = new tcuSurface.Surface(width, height);
        /** @type {tcuSurface.Surface} */ var result = new tcuSurface.Surface(width, height);

        // Call preCheck() that can throw exception if some requirement is not met.
        if (this.preCheck && !this.preCheck())
            return tcuTestCase.IterateResult.STOP;

        // Render using GLES3.
        try {
            /** @type {sglrGLContext.GLContext} */ var context = new sglrGLContext.GLContext(
                                                             gl,
                                                             [x, y, width, height]);

             this.setContext(context);
             this.render(result);

             // Check error.
             /** @type {number} */ var err = context.getError();
             if (err != gl.NO_ERROR)
                 throw new Error('glError: ' + context);

             this.setContext(null);
         } catch (e) {
             if (e instanceof es3fFboTestUtil.FboIncompleteException)
                 if (e.getReason() == gl.FRAMEBUFFER_UNSUPPORTED) {
                     // log << e;
                     // m_testCtx.setTestResult(QP_TEST_RESULT_NOT_SUPPORTED, 'Not supported');
                     assertMsgOptions(false, 'Not supported', true, false);
                     return tcuTestCase.IterateResult.STOP;
                 }
             throw e;
         }

        // Render reference.
        var alphaBits = /** @type {number} */ (gl.getParameter(gl.ALPHA_BITS));
        /** @type {sglrReferenceContext.ReferenceContextBuffers} */
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(new tcuPixelFormat.PixelFormat(
                                                                            8,
                                                                            8,
                                                                            8,
                                                                            alphaBits > 0 ? 8 : 0),
                                                                       /** @type {number} */ (gl.getParameter(gl.DEPTH_BITS)),
                                                                       /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS)),
                                                                       width,
                                                                       height);
        /** @type {sglrReferenceContext.ReferenceContext} */
        var refContext = new sglrReferenceContext.ReferenceContext(new sglrReferenceContext.ReferenceContextLimits(gl),
                                                                buffers.getColorbuffer(),
                                                                buffers.getDepthbuffer(),
                                                                buffers.getStencilbuffer());
        refContext.getError();
        this.setContext(refContext);
        this.render(reference);
        this.setContext(null);

        /** @type {boolean} */ var isOk = this.compare(reference, result);

        assertMsgOptions(isOk, '', true, false);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
    * Deinit. Clear some GL state variables
    */
    es3fFboTestCase.FboTestCase.prototype.deinit = function () {
        // Pixel operations
        {
            gl.disable(gl.SCISSOR_TEST);

            gl.disable(gl.STENCIL_TEST);
            gl.stencilFunc(gl.ALWAYS, 0, 0xffff);
            gl.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP);

            gl.disable(gl.DEPTH_TEST);
            gl.depthFunc(gl.LESS);

            gl.disable(gl.BLEND);
            gl.blendFunc(gl.ONE, gl.ZERO);
            gl.blendEquation(gl.FUNC_ADD);
            gl.blendColor(0.0, 0.0, 0.0, 0.0);

            gl.enable(gl.DITHER);
        }

        // Framebuffer control
        {
            gl.colorMask(true, true, true, true);
            gl.depthMask(true);
            gl.stencilMask(0xffff);

            gl.clearColor(0.0, 0.0, 0.0, 0.0);
            gl.clearDepth(1.0);
            gl.clearStencil(0.0);
            // Do not call clear() here because it might generate an INVALID_OPERATION if
            // some color buffers are of integer formats due to WebGL2 specific constraint.
            // The tests do not rely on clear() here.
        }
    };

    /**
    * @param {number} format
    * @return {boolean}
    */
    es3fFboTestCase.isRequiredFormat = function(format) {
        switch (format) {
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
                return true;

            // Depth formats
            case gl.DEPTH_COMPONENT32F:
            case gl.DEPTH_COMPONENT24:
            case gl.DEPTH_COMPONENT16:
                return true;

            // Depth+stencil formats
            case gl.DEPTH32F_STENCIL8:
            case gl.DEPTH24_STENCIL8:
                return true;

            // Stencil formats
            case gl.STENCIL_INDEX8:
                return true;

            default:
                return false;
        }
    };

    /**
    * @param {number} format deUint32
    * @return {Array<string>}
    */
    es3fFboTestCase.getEnablingExtensions = function(format) {
        /** @return {Array<string>} */ var out = [];

        DE_ASSERT(!es3fFboTestCase.isRequiredFormat(format));

        switch (format) {
            case gl.RGBA16F:
            case gl.RG16F:
            case gl.R16F:
            case gl.RGBA32F:
            case gl.RGB32F:
            case gl.R11F_G11F_B10F:
            case gl.RG32F:
            case gl.R32F:
                out.push('EXT_color_buffer_float');
                break;
            case gl.RGB16F:
                // EXT_color_buffer_half_float is not exposed in WebGL 2.0.
                break;
            default:
                break;
        }

        return out;
    };

    /**
    * @param {es3fFboTestCase.Context} context
    * @param {Array<string>} requiredExts
    * @return {boolean}
    */
    es3fFboTestCase.isAnyExtensionSupported = function(context, requiredExts) {
        for (var iter in requiredExts) {
            /** @const @type {string} */ var extension = requiredExts[iter];

            if (sglrGLContext.isExtensionSupported(gl, extension)) {
                // enable the extension
                gl.getExtension(extension);
                return true;
            }
        }

        return false;
    };

/**
 * @param {number} format GL format
 * @return {number}
 */
es3fFboTestCase.getMinimumSampleCount = function(format) {
    switch (format) {
        // Core formats
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
        case gl.DEPTH_COMPONENT32F:
        case gl.DEPTH_COMPONENT24:
        case gl.DEPTH_COMPONENT16:
        case gl.DEPTH32F_STENCIL8:
        case gl.DEPTH24_STENCIL8:
        case gl.STENCIL_INDEX8:
            return 4;

        // gl.EXT_color_buffer_float
        case gl.R11F_G11F_B10F:
        case gl.RG16F:
        case gl.R16F:
            return 4;

        case gl.RGBA32F:
        case gl.RGBA16F:
        case gl.RG32F:
        case gl.R32F:
            return 0;

        // gl.EXT_color_buffer_half_float
        case gl.RGB16F:
            return 0;

        default:
            throw new Error('Unknown format:' + format);
    }
};

es3fFboTestCase.querySampleCounts = function(format) {
    return gl.getInternalformatParameter(gl.RENDERBUFFER, format, gl.SAMPLES);
};

});

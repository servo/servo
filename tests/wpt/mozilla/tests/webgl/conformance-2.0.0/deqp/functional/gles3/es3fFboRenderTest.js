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
goog.provide('functional.gles3.es3fFboRenderTest');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {

    var es3fFboRenderTest = functional.gles3.es3fFboRenderTest;
    var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuLogImage = framework.common.tcuLogImage;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;
    var deString = framework.delibs.debase.deString;
    var deUtil = framework.delibs.debase.deUtil;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var sglrReferenceContext =
        framework.opengl.simplereference.sglrReferenceContext;
    var rrUtil = framework.referencerenderer.rrUtil;

    /** @type {WebGL2RenderingContext} */ var gl;

    /**
     * @constructor
     * @param {number=} buffers_
     * @param {number=} colorType_
     * @param {number=} colorFormat_
     * @param {number=} depthStencilType_
     * @param {number=} depthStencilFormat_
     * @param {number=} width_
     * @param {number=} height_
     * @param {number=} samples_
     */
    es3fFboRenderTest.FboConfig = function(
        buffers_, colorType_, colorFormat_, depthStencilType_,
        depthStencilFormat_, width_, height_, samples_
    ) {
        // Buffer bit mask (gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT|...)
        this.buffers = buffers_ ? buffers_ : 0;
        // gl.TEXTURE_2D, gl.TEXTURE_CUBE_MAP, gl.RENDERBUFFER
        this.colorType = colorType_ ? colorType_ : gl.NONE;
        // Internal format for color buffer texture or renderbuffer
        this.colorFormat = colorFormat_ ? colorFormat_ : gl.NONE;
        this.depthStencilType = depthStencilType_?
            depthStencilType_ : gl.NONE;
        this.depthStencilFormat = depthStencilFormat_ ?
            depthStencilFormat_ : gl.NONE;
        this.width = width_ ? width_ : 0;
        this.height = height_ ? height_ : 0;
        this.samples = samples_? samples_ : 0;
    };

    /**
     * @param {number} type
     * @return {string}
     */
     es3fFboRenderTest.getTypeName = function(type) {
        switch (type) {
            case gl.TEXTURE_2D: return 'tex2d';
            case gl.RENDERBUFFER: return 'rbo';
            default:
                testFailed('Unknown type');
        }
        return 'Should not get to this point';
    };

    /**
     * @return {string}
     */
    es3fFboRenderTest.FboConfig.prototype.getName = function() {
        var name = '';

        assertMsgOptions((this.buffers & gl.COLOR_BUFFER_BIT) != 0,
            'Color buffer is not specified', false, true);

        name += es3fFboRenderTest.getTypeName(this.colorType) + '_' +
                es3fFboTestUtil.getFormatName(this.colorFormat);

        if (this.buffers & gl.DEPTH_BUFFER_BIT)
            name += '_depth';
        if (this.buffers & gl.STENCIL_BUFFER_BIT)
            name += '_stencil';

        if (this.buffers & (gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT))
            name += '_' + es3fFboRenderTest.getTypeName(this.depthStencilType) +
                   '_' + es3fFboTestUtil.getFormatName(this.depthStencilFormat);

        return name;
    };

    /**
     * @param {number} format
     * @return {Array<string>}
     */
    es3fFboRenderTest.getEnablingExtensions = function(format) {
        /** @type {Array<string>} */ var out = [];

        switch (format) {
            case gl.RGB16F:
                assertMsgOptions(false, "Not part of the tested formats", false, true);
                break;

            case gl.RGBA16F:
            case gl.RG16F:
            case gl.R16F:
            case gl.RGBA32F:
            case gl.RGB32F:
            case gl.R11F_G11F_B10F:
            case gl.RG32F:
            case gl.R32F:
                out.push('EXT_color_buffer_float');

            default:
                break;
        }

        return out;
    };

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {string} name
     * @return {*}
     */
    es3fFboRenderTest.isExtensionSupported = function(context, name) {
        return context.getExtension(name);
    };

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {Array<string>} requiredExts
     * @return {boolean}
     */
    es3fFboRenderTest.isAnyExtensionSupported = function(
        context, requiredExts) {

        if (!requiredExts || requiredExts.length == 0)
            return true;

        for (var extNdx = 0; extNdx < requiredExts.length; extNdx++) {
            var extension = requiredExts[extNdx];

            if (es3fFboRenderTest.isExtensionSupported(context, extension))
                return true;
        }

        return false;
    };

    /**
     * @param {Array} list
     * @param {string} sep
     * @return {string}
     */
    es3fFboRenderTest.join = function(list, sep) {
        var out = '';

        for (var elemNdx = 0; elemNdx < list.length; elemNdx++) {
            if (elemNdx != 0)
                out += sep;
            out += list[elemNdx];
        }

        return out;
    };

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {number} sizedFormat
     */
    es3fFboRenderTest.checkColorFormatSupport = function(context, sizedFormat) {
        /** @type {Array<string>} */ var requiredExts =
            es3fFboRenderTest.getEnablingExtensions(sizedFormat);

        if (!es3fFboRenderTest.isAnyExtensionSupported(context, requiredExts)) {
            var errMsg = 'Format not supported, requires ' + (
                (requiredExts.length == 1) ? requiredExts[0] :
                ' one of the following: ' +
                requiredExts.join(', ')
            );
            checkMessage(false, errMsg);

            throw new TestFailedException(errMsg);
        }
    };

    /**
     * @constructor
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {es3fFboRenderTest.FboConfig} config
     * @param {number} width
     * @param {number} height
     * @param {sglrReferenceContext.AnyFramebuffer=} fbo
     * @param {sglrReferenceContext.AnyRenderbuffer=} colorBufferName
     * @param {sglrReferenceContext.AnyRenderbuffer=} depthStencilBufferName
     */
    es3fFboRenderTest.Framebuffer = function(
        context, config, width, height, fbo,
        colorBufferName, depthStencilBufferName) {

        this.m_config = config;
        this.m_context = context;
        this.m_framebuffer = fbo ? fbo : null;
        this.m_colorBuffer = colorBufferName ? colorBufferName : null;
        this.m_depthStencilBuffer = depthStencilBufferName ?
            depthStencilBufferName : null;

        // Verify that color format is supported
        es3fFboRenderTest.checkColorFormatSupport(context, config.colorFormat);

        if (!this.m_framebuffer)
            this.m_framebuffer = context.createFramebuffer();
        context.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);

        if (this.m_config.buffers & (gl.COLOR_BUFFER_BIT)) {
            switch (this.m_config.colorType) {
                case gl.TEXTURE_2D:
                    this.m_colorBuffer = this.createTex2D(
                        /** @type {WebGLTexture} */ (colorBufferName),
                        this.m_config.colorFormat, width, height
                    );

                    context.framebufferTexture2D(
                        gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                        gl.TEXTURE_2D, this.m_colorBuffer, 0
                    );

                    break;

                case gl.RENDERBUFFER:
                    this.m_colorBuffer = this.createRbo(
                        /** @type {WebGLRenderbuffer} */ (colorBufferName),
                        this.m_config.colorFormat, width, height
                    );

                    context.framebufferRenderbuffer(
                        gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                        gl.RENDERBUFFER, this.m_colorBuffer
                    );

                    break;

                default:
                    testFailed('Unsupported type');
            }
        }

        if (this.m_config.buffers &
            (gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT)) {

            switch (this.m_config.depthStencilType) {
                case gl.TEXTURE_2D:
                    this.m_depthStencilBuffer = this.createTex2D(
                        /** @type {WebGLTexture} */
                        (depthStencilBufferName),
                        this.m_config.depthStencilFormat, width, height
                    );
                    break;
                case gl.RENDERBUFFER:
                    this.m_depthStencilBuffer = this.createRbo(
                        /** @type {WebGLRenderbuffer} */
                        (depthStencilBufferName),
                        this.m_config.depthStencilFormat, width, height
                    );
                    break;

                default:
                    testFailed('Unsupported type');
            }
        }

        for (var ndx = 0; ndx < 2; ndx++) {
            var bit = ndx ? gl.STENCIL_BUFFER_BIT : gl.DEPTH_BUFFER_BIT;
            var point = ndx ? gl.STENCIL_ATTACHMENT : gl.DEPTH_ATTACHMENT;

            if ((this.m_config.buffers & bit) == 0)
                continue; /* Not used. */

            switch (this.m_config.depthStencilType) {
                case gl.TEXTURE_2D:
                    context.framebufferTexture2D(
                        gl.FRAMEBUFFER, point, gl.TEXTURE_2D,
                        this.m_depthStencilBuffer, 0
                    );
                    break;
                case gl.RENDERBUFFER:
                    context.framebufferRenderbuffer(
                        gl.FRAMEBUFFER, point,
                        gl.RENDERBUFFER, this.m_depthStencilBuffer
                    );
                    break;
                default:
                    throw new Error('Invalid depth stencil type');
            }
        }

        context.bindFramebuffer(gl.FRAMEBUFFER, null);
    };

    /**
     * @return {es3fFboRenderTest.FboConfig}
     */
    es3fFboRenderTest.Framebuffer.prototype.getConfig = function() {
        return this.m_config;
    };

    /**
     * @return {?sglrReferenceContext.AnyFramebuffer}
     */
    es3fFboRenderTest.Framebuffer.prototype.getFramebuffer = function() {
        return this.m_framebuffer;
    };

    /**
     * @return {?sglrReferenceContext.AnyRenderbuffer}
     */
    es3fFboRenderTest.Framebuffer.prototype.getColorBuffer = function() {
        return this.m_colorBuffer;
    };

    /**
     * @return {?sglrReferenceContext.AnyRenderbuffer}
     */
    es3fFboRenderTest.Framebuffer.prototype.getDepthStencilBuffer = function() {
        return this.m_depthStencilBuffer;
    };

    /**
     * deinit
     */
    es3fFboRenderTest.Framebuffer.prototype.deinit = function() {
        this.m_context.deleteFramebuffer(
            /** @type {WebGLFramebuffer} */ (this.m_framebuffer)
        );
        this.destroyBuffer(this.m_colorBuffer, this.m_config.colorType);
        this.destroyBuffer(
            this.m_depthStencilBuffer, this.m_config.depthStencilType
        );
    };

    /**
     * checkCompleteness
     */
    es3fFboRenderTest.Framebuffer.prototype.checkCompleteness = function() {
        this.m_context.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);
        var status = this.m_context.checkFramebufferStatus(gl.FRAMEBUFFER);
        this.m_context.bindFramebuffer(gl.FRAMEBUFFER, null);
        if (status != gl.FRAMEBUFFER_COMPLETE)
            throw new es3fFboTestUtil.FboIncompleteException(status);
    };

    /**
     * @param {?WebGLTexture|sglrReferenceContext.TextureContainer} name
     * @param {number} format
     * @param {number} width
     * @param {number} height
     * @return {?WebGLTexture|sglrReferenceContext.TextureContainer}
     */
    es3fFboRenderTest.Framebuffer.prototype.createTex2D = function(
        name, format, width, height) {

        if (!name)
            name = this.m_context.createTexture();

        this.m_context.bindTexture(gl.TEXTURE_2D, name);
        this.m_context.texImage2DDelegate(
            gl.TEXTURE_2D, 0, format, width, height
        );

        if (!deMath.deIsPowerOfTwo32(width) ||
            !deMath.deIsPowerOfTwo32(height)) {

            // Set wrap mode to clamp for NPOT FBOs
            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE
            );
        }

        this.m_context.texParameteri(
            gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST
        );
        this.m_context.texParameteri(
            gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST
        );

        return name;
    };

    /**
     * @param {?WebGLRenderbuffer|sglrReferenceContext.Renderbuffer} name
     * @param {number} format
     * @param {number} width
     * @param {number} height
     * @return {?WebGLRenderbuffer|sglrReferenceContext.Renderbuffer}
     */
    es3fFboRenderTest.Framebuffer.prototype.createRbo = function(
        name, format, width, height) {

        if (!name)
            name = this.m_context.createRenderbuffer();

        this.m_context.bindRenderbuffer(gl.RENDERBUFFER, name);
        this.m_context.renderbufferStorage(
            gl.RENDERBUFFER, format, width, height
        );

        return name;
    };

    /**
     * @param {?sglrReferenceContext.AnyRenderbuffer} name
     * @param {number} type
     */
    es3fFboRenderTest.Framebuffer.prototype.destroyBuffer = function(
        name, type) {

        if (type == gl.TEXTURE_2D || type == gl.TEXTURE_CUBE_MAP)
            this.m_context.deleteTexture(/** @type {?WebGLTexture} */ (name));
        else if (type == gl.RENDERBUFFER)
            this.m_context.deleteRenderbuffer(
                /** @type {?WebGLRenderbuffer} */ (name)
            );
        else
            assertMsgOptions(
                type == gl.NONE, 'Invalid buffer type', false, true
            );
    };

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {WebGLTexture|sglrReferenceContext.TextureContainer} name
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fFboRenderTest.createMetaballsTex2D = function(
        context, name, format, dataType, width, height) {

        /** @type {tcuTexture.TextureFormat} */ var texFormat =
            gluTextureUtil.mapGLTransferFormat(format, dataType);
        /** @type {tcuTexture.TextureLevel} */ var level =
            new tcuTexture.TextureLevel(texFormat, width, height);

        tcuTextureUtil.fillWithMetaballs(
            level.getAccess(), 5, /*name ^*/ width ^ height
        );

        context.bindTexture(gl.TEXTURE_2D, name);
        context.texImage2D(
            gl.TEXTURE_2D, 0, format, width, height, 0, format,
            dataType, level.getAccess().getDataPtr()
        );
        context.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    };

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {WebGLTexture|sglrReferenceContext.TextureContainer} name
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fFboRenderTest.createQuadsTex2D = function(
        context, name, format, dataType, width, height) {

        /** @type {tcuTexture.TextureFormat} */
        var texFormat = gluTextureUtil.mapGLTransferFormat(format, dataType);
        /** @type {tcuTexture.TextureLevel} */
        var level = new tcuTexture.TextureLevel(texFormat, width, height);

        tcuTextureUtil.fillWithRGBAQuads(level.getAccess());

        context.bindTexture(gl.TEXTURE_2D, name);
        context.texImage2D(
            gl.TEXTURE_2D, 0, format, width, height, 0,
            format, dataType, level.getAccess().getDataPtr()
        );
        context.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     * @param {es3fFboRenderTest.FboConfig} config
     */
    es3fFboRenderTest.FboRenderCase = function(name, description, config) {
        tcuTestCase.DeqpTest.call(this, name, description);
        this.m_config = config;
    };

    es3fFboRenderTest.FboRenderCase.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);

    es3fFboRenderTest.FboRenderCase.prototype.constructor =
        es3fFboRenderTest.FboRenderCase;

    /**
     * Must be overridden
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     *      fboContext
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.FboRenderCase.prototype.render = function(
        fboContext, dst) {
            throw new Error('Must override');
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fFboRenderTest.FboRenderCase.prototype.iterate = function() {
        var clearColor = [0.125, 0.25, 0.5, 1.0];
        /** @type {?string} */ var failReason = "";

        // Position & size for context
        var rnd = new deRandom.deRandom();
        deRandom.deRandom_init(rnd, deString.deStringHash(this.fullName()));

        var width = Math.min(gl.canvas.width, 128);
        var height = Math.min(gl.canvas.height, 128);
        var xMax = gl.canvas.width - width + 1;
        var yMax = gl.canvas.height - height + 1;
        var x = Math.abs(deRandom.deRandom_getInt(rnd)) % xMax;
        var y = Math.abs(deRandom.deRandom_getInt(rnd)) % yMax;

        /** @type {tcuSurface.Surface} */
        var gles3Frame = new tcuSurface.Surface(width, height);
        /** @type {tcuSurface.Surface} */
        var refFrame = new tcuSurface.Surface(width, height);

        /** @type {number} */ var gles3Error = 0;
        /** @type {number} */ var refError = 0;

        // Render using GLES3
        /**
         * @type {sglrGLContext.GLContext|
         * sglrReferenceContext.ReferenceContext}
         */
        var context;

        try {
            context = new sglrGLContext.GLContext(gl, [x, y, width, height]);

            context.clearColor(
                clearColor[0], clearColor[1], clearColor[2], clearColor[3]
            );

            context.clear(
                gl.COLOR_BUFFER_BIT |
                gl.DEPTH_BUFFER_BIT |
                gl.STENCIL_BUFFER_BIT
            );

            this.render(context, gles3Frame); // Call actual render func
            gles3Error = context.getError();
        }
        catch (e) {
            if (e instanceof es3fFboTestUtil.FboIncompleteException) {
                e.message = WebGLTestUtils.glEnumToString(gl, e.getReason());
                if(e.getReason() == gl.FRAMEBUFFER_UNSUPPORTED) {
                    // Mark test case as unsupported
                    bufferedLogToConsole(e + ': ' + e.message);
                    testFailed('Not supported');
                    return tcuTestCase.IterateResult.STOP;
                }
            }

            // Propagate error
            throw e;
        }

        // Render reference image

        /** @type {sglrReferenceContext.ReferenceContextBuffers} */
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(
            new tcuPixelFormat.PixelFormat(
                8, 8, 8,
                gl.getParameter(gl.ALPHA_BITS) ? 8 : 0
            ),
            /** @type {number} */ (gl.getParameter(gl.DEPTH_BITS)),
            /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS)),
            width,
            height
        );
        context = new sglrReferenceContext.ReferenceContext(
            new sglrReferenceContext.ReferenceContextLimits(gl),
            buffers.getColorbuffer(),
            buffers.getDepthbuffer(),
            buffers.getStencilbuffer()
        );

        context.clearColor(
            clearColor[0], clearColor[1], clearColor[2], clearColor[3]
        );

        context.clear(
            gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT
        );

        this.render(context, refFrame);
        refError = context.getError();

        // Compare error codes
        var errorCodesOk = (gles3Error == refError);

        if (!errorCodesOk) {
            bufferedLogToConsole (
                'Error code mismatch: got ' +
                WebGLTestUtils.glEnumToString(gl, gles3Error) + ', expected ' +
                WebGLTestUtils.glEnumToString(gl, refError)
            );
            failReason = 'Got unexpected error';
        }

        // Compare images
        var imagesOk = this.compare(refFrame, gles3Frame);

        if (!imagesOk && !failReason)
            failReason = 'Image comparison failed';

        // Store test result
        var isOk = errorCodesOk && imagesOk;
        assertMsgOptions(isOk, failReason, true, true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @param {tcuSurface.Surface} reference
     * @param {tcuSurface.Surface} result
     * @return {boolean}
     */
    es3fFboRenderTest.FboRenderCase.prototype.compare = function(
        reference, result) {

        var threshold = new tcuRGBA.RGBA(
            /* TODO: tcu::max(getFormatThreshold(this.m_config.colorFormat),*/
            [12, 12, 12, 12]
        );

        return tcuImageCompare.bilinearCompare(
            'ComparisonResult', 'Image comparison result',
            reference.getAccess(), result.getAccess(),
            threshold, tcuImageCompare.CompareLogMode.RESULT
        );
    };

    /**
     * deinit
     */
    es3fFboRenderTest.FboRenderCase.prototype.deinit = function() {
        gl.clearColor(0.0, 0.0, 0.0, 0.0);
        gl.clearDepth(1.0);
        gl.clearStencil(0);

        gl.disable(gl.STENCIL_TEST);
        gl.disable(gl.DEPTH_TEST);
        gl.disable(gl.BLEND);

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        gl.bindRenderbuffer(gl.RENDERBUFFER, null);
    };

    // FboCases

    /**
     * @constructor
     * @extends {es3fFboRenderTest.FboRenderCase}
     * @param {es3fFboRenderTest.FboConfig} config
     */
    es3fFboRenderTest.StencilClearsTest = function(config) {
        es3fFboRenderTest.FboRenderCase.call(
            this, config.getName(), 'Stencil clears', config
        );
    };

    es3fFboRenderTest.StencilClearsTest.prototype =
        Object.create(es3fFboRenderTest.FboRenderCase.prototype);

    es3fFboRenderTest.StencilClearsTest.prototype.constructor =
        es3fFboRenderTest.StencilClearsTest;

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.StencilClearsTest.prototype.render = function(
        context, dst) {

        /** @type {tcuTexture.TextureFormat} */
        var colorFormat = gluTextureUtil.mapGLInternalFormat(
            this.m_config.colorFormat
        );

        /** @type {gluShaderUtil.DataType} */
        var fboSamplerType = /** @type {gluShaderUtil.DataType} */ (
            gluTextureUtil.getSampler2DType(colorFormat)
        );

        /** @type {gluShaderUtil.DataType} */
        var fboOutputType = es3fFboTestUtil.getFragmentOutputType(colorFormat);

        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fboRangeInfo = tcuTextureUtil.getTextureFormatInfo(colorFormat);

        var fboOutScale = deMath.subtract(
            fboRangeInfo.valueMax, fboRangeInfo.valueMin
        );

        var fboOutBias = fboRangeInfo.valueMin;

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFboShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], fboOutputType
        );

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texFromFboShader = new es3fFboTestUtil.Texture2DShader(
            [fboSamplerType], gluShaderUtil.DataType.FLOAT_VEC4);

        /** @type {number} */ var texToFboShaderID =
            context.createProgram(texToFboShader);

        /** @type {number} */ var texFromFboShaderID =
            context.createProgram(texFromFboShader);

        /** @type {?WebGLTexture|sglrReferenceContext.TextureContainer} */
        var metaballsTex = context.createTexture();

        /** @type {?WebGLTexture|sglrReferenceContext.TextureContainer} */
        var quadsTex = context.createTexture();

        /** @type {number} */ var width = 128;
        /** @type {number} */ var height = 128;

        texToFboShader.setOutScaleBias(fboOutScale, fboOutBias);
        texFromFboShader.setTexScaleBias(
            0, fboRangeInfo.lookupScale, fboRangeInfo.lookupBias
        );

        es3fFboRenderTest.createQuadsTex2D(
            context, quadsTex, gl.RGBA, gl.UNSIGNED_BYTE, width, height
        );

        es3fFboRenderTest.createMetaballsTex2D(
            context, metaballsTex, gl.RGBA, gl.UNSIGNED_BYTE, width, height
        );

        /** @type {es3fFboRenderTest.Framebuffer} */
        var fbo = new es3fFboRenderTest.Framebuffer(
            context, this.m_config, width, height
        );
        fbo.checkCompleteness();

        // Bind framebuffer and clear
        context.bindFramebuffer(gl.FRAMEBUFFER, fbo.getFramebuffer());
        context.viewport(0, 0, width, height);
        context.clearColor(0.0, 0.0, 0.0, 1.0);
        context.clear(
            gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT
        );

        // Do stencil clears
        context.enable(gl.SCISSOR_TEST);
        context.scissor(10, 16, 32, 120);
        context.clearStencil(1);
        context.clear(gl.STENCIL_BUFFER_BIT);
        context.scissor(16, 32, 100, 64);
        context.clearStencil(2);
        context.clear(gl.STENCIL_BUFFER_BIT);
        context.disable(gl.SCISSOR_TEST);

        // Draw 2 textures with stecil tests
        context.enable(gl.STENCIL_TEST);

        context.bindTexture(gl.TEXTURE_2D, quadsTex);
        context.stencilFunc(gl.EQUAL, 1, 0xff);

        texToFboShader.setUniforms(context, texToFboShaderID);
        rrUtil.drawQuad(
            context, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        context.bindTexture(gl.TEXTURE_2D, metaballsTex);
        context.stencilFunc(gl.EQUAL, 2, 0xff);

        texToFboShader.setUniforms(context, texToFboShaderID);
        rrUtil.drawQuad(
            context, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        context.disable(gl.STENCIL_TEST);

        if (fbo.getConfig().colorType == gl.TEXTURE_2D) {
            context.bindFramebuffer(gl.FRAMEBUFFER, null);
            context.bindTexture(gl.TEXTURE_2D, fbo.getColorBuffer());
            context.viewport(0, 0, context.getWidth(), context.getHeight());

            texFromFboShader.setUniforms(context, texFromFboShaderID);
            rrUtil.drawQuad(
                context, texFromFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );

            dst.readViewport(
                context, [0, 0, context.getWidth(), context.getHeight()]
            );
        } else
            es3fFboTestUtil.readPixels(
                context, dst, 0, 0, width, height, colorFormat,
                fboRangeInfo.lookupScale, fboRangeInfo.lookupBias
            );
    };

    /**
     * @constructor
     * @extends {es3fFboRenderTest.FboRenderCase}
     * @param {es3fFboRenderTest.FboConfig} config
     */
    es3fFboRenderTest.SharedColorbufferTest = function(config) {
        es3fFboRenderTest.FboRenderCase.call(
            this, config.getName(), 'Shared colorbuffer', config
        );
    };

    es3fFboRenderTest.SharedColorbufferTest.prototype =
        Object.create(es3fFboRenderTest.FboRenderCase.prototype);

    es3fFboRenderTest.SharedColorbufferTest.prototype.constructor =
        es3fFboRenderTest.SharedColorbufferTest;

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.SharedColorbufferTest.prototype.render = function(
        context, dst) {

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D],
            gluShaderUtil.DataType.FLOAT_VEC4
        );

        /** @type {es3fFboTestUtil.FlatColorShader} */
        var flatShader = new es3fFboTestUtil.FlatColorShader(
            gluShaderUtil.DataType.FLOAT_VEC4
        );

        /** @type {number} */
        var texShaderID = context.createProgram(texShader);
        /** @type {number} */
        var flatShaderID = context.createProgram(flatShader);

        /** @type {number} */ var width = 128;
        /** @type {number} */ var height = 128;

        /** @type {?WebGLTexture|sglrReferenceContext.TextureContainer} */
        var quadsTex = context.createTexture();

        /** @type {?WebGLTexture|sglrReferenceContext.TextureContainer} */
        var metaballsTex = context.createTexture();

        /** @type {boolean} */ var stencil =
            (this.m_config.buffers & gl.STENCIL_BUFFER_BIT) != 0;

        context.disable(gl.DITHER);

        // Textures
        es3fFboRenderTest.createQuadsTex2D(
            context, quadsTex, gl.RGB, gl.UNSIGNED_BYTE, 64, 64
        );
        es3fFboRenderTest.createMetaballsTex2D(
            context, metaballsTex, gl.RGBA, gl.UNSIGNED_BYTE, 64, 64
        );

        context.viewport(0, 0, width, height);

        // Fbo A
        /** @type {es3fFboRenderTest.Framebuffer} */
        var fboA = new es3fFboRenderTest.Framebuffer(
                context, this.m_config, width, height
        );
        fboA.checkCompleteness();

        // Fbo B - don't create colorbuffer

        /** @type {es3fFboRenderTest.FboConfig} */
        var cfg = /** @type {es3fFboRenderTest.FboConfig} */
            (deUtil.clone(this.m_config));

        cfg.buffers = deMath.binaryOp(
            cfg.buffers,
            deMath.binaryNot(gl.COLOR_BUFFER_BIT),
            deMath.BinaryOp.AND
        );
        cfg.colorType = gl.NONE;
        cfg.colorFormat = gl.NONE;

        /** @type {es3fFboRenderTest.Framebuffer} */
        var fboB = new es3fFboRenderTest.Framebuffer(
            context, cfg, width, height
        );

        // Attach color buffer from fbo A
        context.bindFramebuffer(gl.FRAMEBUFFER, fboB.getFramebuffer());
        switch (this.m_config.colorType) {
            case gl.TEXTURE_2D:
                context.framebufferTexture2D(
                    gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                    gl.TEXTURE_2D, fboA.getColorBuffer(), 0
                );
                break;

            case gl.RENDERBUFFER:
                context.framebufferRenderbuffer(
                    gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                    gl.RENDERBUFFER, fboA.getColorBuffer()
                );
                break;

            default:
                throw new Error('Invalid color type');
        }

        // Clear depth and stencil in fbo B
        context.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        // Render quads to fbo 1, with depth 0.0
        context.bindFramebuffer(gl.FRAMEBUFFER, fboA.getFramebuffer());
        context.bindTexture(gl.TEXTURE_2D, quadsTex);
        context.clearColor(0.0, 0.0, 0.0, 1.0);
        context.clear(
            gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT
        );

        if (stencil) {
            // Stencil to 1 in fbo A
            context.clearStencil(1);
            context.clear(gl.STENCIL_BUFFER_BIT);
        }

        texShader.setUniforms(context, texShaderID);

        context.enable(gl.DEPTH_TEST);
        rrUtil.drawQuad(
            context, texShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );
        context.disable(gl.DEPTH_TEST);

        // Blend metaballs to fbo 2
        context.bindFramebuffer(gl.FRAMEBUFFER, fboB.getFramebuffer());
        context.bindTexture(gl.TEXTURE_2D, metaballsTex);
        context.enable(gl.BLEND);
        context.blendFuncSeparate(
            gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ZERO, gl.ONE
        );
        rrUtil.drawQuad(
            context, texShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        // Render small quad that is only visible if depth buffer
        // is not shared with fbo A - or there is no depth bits
        context.bindTexture(gl.TEXTURE_2D, quadsTex);
        context.enable(gl.DEPTH_TEST);
        rrUtil.drawQuad(context, texShaderID, [0.5, 0.5, 0.5], [1.0, 1.0, 0.5]);
        context.disable(gl.DEPTH_TEST);

        if (stencil) {
            flatShader.setColor(context, flatShaderID, [0.0, 1.0, 0.0, 1.0]);

            // Clear subset of stencil buffer to 1
            context.enable(gl.SCISSOR_TEST);
            context.scissor(10, 10, 12, 25);
            context.clearStencil(1);
            context.clear(gl.STENCIL_BUFFER_BIT);
            context.disable(gl.SCISSOR_TEST);

            // Render quad with stencil mask == 1
            context.enable(gl.STENCIL_TEST);
            context.stencilFunc(gl.EQUAL, 1, 0xff);
            rrUtil.drawQuad(
                context, flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );
            context.disable(gl.STENCIL_TEST);
        }

        // Get results
        if (fboA.getConfig().colorType == gl.TEXTURE_2D) {
            texShader.setUniforms(context, texShaderID);

            context.bindFramebuffer(gl.FRAMEBUFFER, null);
            context.bindTexture(gl.TEXTURE_2D, fboA.getColorBuffer());
            context.viewport(0, 0, context.getWidth(), context.getHeight());
            rrUtil.drawQuad(
                context, texShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );
            dst.readViewport(
                context, [0, 0, context.getWidth(), context.getHeight()]
            );
        } else
            es3fFboTestUtil.readPixels(
                context, dst, 0, 0, width, height,
                gluTextureUtil.mapGLInternalFormat(
                    fboA.getConfig().colorFormat
                ), [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]
            );
    };

    /**
     * @constructor
     * @extends {es3fFboRenderTest.FboRenderCase}
     * @param {es3fFboRenderTest.FboConfig} config
     */
    es3fFboRenderTest.SharedColorbufferClearsTest = function(config) {
        es3fFboRenderTest.FboRenderCase.call(
            this, config.getName(), 'Shared colorbuffer clears', config
        );
    };

    es3fFboRenderTest.SharedColorbufferClearsTest.prototype =
        Object.create(es3fFboRenderTest.FboRenderCase.prototype);

    es3fFboRenderTest.SharedColorbufferClearsTest.prototype.constructor =
        es3fFboRenderTest.SharedColorbufferClearsTest;

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.SharedColorbufferClearsTest.prototype.render = function(
        context, dst) {

        /** @type {tcuTexture.TextureFormat} */
        var colorFormat = gluTextureUtil.mapGLInternalFormat(
            this.m_config.colorFormat
        );

        /** @type {gluShaderUtil.DataType} */
        var fboSamplerType = gluTextureUtil.getSampler2DType(colorFormat);

        var width = 128;
        var height = 128;
        var colorbuffer = this.m_config.colorType == gl.TEXTURE_2D?
            context.createTexture() :
            context.createRenderbuffer();

        // Check for format support.
        es3fFboRenderTest.checkColorFormatSupport(
            context, this.m_config.colorFormat
        );

        // Single colorbuffer
        if (this.m_config.colorType == gl.TEXTURE_2D) {
            context.bindTexture(gl.TEXTURE_2D, colorbuffer);
            context.texImage2DDelegate(
                gl.TEXTURE_2D, 0, this.m_config.colorFormat, width, height
            );
            context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST
            );
            context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST
            );
        } else {
            assertMsgOptions(
                this.m_config.colorType == gl.RENDERBUFFER,
                'Not a render buffer type', false, true
            );
            context.bindRenderbuffer(gl.RENDERBUFFER, colorbuffer);
            context.renderbufferStorage(
                gl.RENDERBUFFER, this.m_config.colorFormat, width, height
            );
        }

        // Multiple framebuffers sharing the colorbuffer
        var fbo = [
            context.createFramebuffer(),
            context.createFramebuffer(),
            context.createFramebuffer()
        ];

        for (var fboi = 0; fboi < fbo.length; fboi++) {
            context.bindFramebuffer(gl.FRAMEBUFFER, fbo[fboi]);

            if (this.m_config.colorType == gl.TEXTURE_2D)
                context.framebufferTexture2D(
                    gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                    gl.TEXTURE_2D, colorbuffer, 0
                );
            else
                context.framebufferRenderbuffer(
                    gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                    gl.RENDERBUFFER, colorbuffer
                );
        }

        context.bindFramebuffer(gl.FRAMEBUFFER, fbo[0]);

        // Check completeness

        var status = context.checkFramebufferStatus(gl.FRAMEBUFFER);
        if (status != gl.FRAMEBUFFER_COMPLETE)
            throw new es3fFboTestUtil.FboIncompleteException(status);

        // Render to them
        context.viewport(0, 0, width, height);
        context.clearColor(0.0, 0.0, 1.0, 1.0);
        context.clear(gl.COLOR_BUFFER_BIT);

        context.enable(gl.SCISSOR_TEST);

        context.bindFramebuffer(gl.FRAMEBUFFER, fbo[1]);
        context.clearColor(0.6, 0.0, 0.0, 1.0);
        context.scissor(10, 10, 64, 64);
        context.clear(gl.COLOR_BUFFER_BIT);
        context.clearColor(0.0, 0.6, 0.0, 1.0);
        context.scissor(60, 60, 40, 20);
        context.clear(gl.COLOR_BUFFER_BIT);

        context.bindFramebuffer(gl.FRAMEBUFFER, fbo[2]);
        context.clearColor(0.0, 0.0, 0.6, 1.0);
        context.scissor(20, 20, 100, 10);
        context.clear(gl.COLOR_BUFFER_BIT);

        context.bindFramebuffer(gl.FRAMEBUFFER, fbo[0]);
        context.clearColor(0.6, 0.0, 0.6, 1.0);
        context.scissor(20, 20, 5, 100);
        context.clear(gl.COLOR_BUFFER_BIT);

        context.disable(gl.SCISSOR_TEST);

        if (this.m_config.colorType == gl.TEXTURE_2D) {
            /** @type {es3fFboTestUtil.Texture2DShader} */
            var shader = new es3fFboTestUtil.Texture2DShader(
                [fboSamplerType], gluShaderUtil.DataType.FLOAT_VEC4
            );
            var shaderID = context.createProgram(shader);

            shader.setUniforms(context, shaderID);

            context.bindFramebuffer(gl.FRAMEBUFFER, null);
            context.viewport(0, 0, context.getWidth(), context.getHeight());
            rrUtil.drawQuad(
                context, shaderID, [-0.9, -0.9, 0.0], [0.9, 0.9, 0.0]
            );
            dst.readViewport(
                context, [0, 0, context.getWidth(), context.getHeight()]
            );
        } else
            es3fFboTestUtil.readPixels(
                context, dst, 0, 0, width, height, colorFormat,
                [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]
            );

        //delete FBOs
        for (fboi = 0; fboi < fbo.length; fboi++)
            context.deleteFramebuffer(fbo[fboi]);

        //delete Texture/Renderbuffer
        if (this.m_config.colorType == gl.TEXTURE_2D)
            context.deleteTexture(colorbuffer);
        else
            context.deleteRenderbuffer(colorbuffer);
    };

    /**
     * @constructor
     * @extends {es3fFboRenderTest.FboRenderCase}
     * @param {es3fFboRenderTest.FboConfig} config
     */
    es3fFboRenderTest.SharedDepthStencilTest = function(config) {
        es3fFboRenderTest.FboRenderCase.call(
            this, config.getName(), 'Shared depth/stencilbuffer', config
        );
    };

    es3fFboRenderTest.SharedDepthStencilTest.prototype =
        Object.create(es3fFboRenderTest.FboRenderCase.prototype);

    es3fFboRenderTest.SharedDepthStencilTest.prototype.constructor =
        es3fFboRenderTest.SharedDepthStencilTest;

    /**
     * @param {es3fFboRenderTest.FboConfig} config
     * @return {boolean}
     */
    es3fFboRenderTest.SharedDepthStencilTest.prototype.isConfigSupported =
    function(config) {
        return deMath.binaryOp(
            config.buffers,
            deMath.binaryOp(
                gl.DEPTH_BUFFER_BIT, gl.STENCIL_BUFFER_BIT, deMath.BinaryOp.OR
            ), deMath.BinaryOp.AND
        ) != 0;
    };

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.SharedDepthStencilTest.prototype.render = function(
        context, dst) {

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D],
            gluShaderUtil.DataType.FLOAT_VEC4
        );

        /** @type {es3fFboTestUtil.FlatColorShader} */
        var flatShader = new es3fFboTestUtil.FlatColorShader(
            gluShaderUtil.DataType.FLOAT_VEC4
        );

        var texShaderID = context.createProgram(texShader);
        var flatShaderID = context.createProgram(flatShader);
        var width = 128;
        var height = 128;
        // bool depth = (this.m_config.buffers & gl.DEPTH_BUFFER_BIT) != 0;
        /**@type {boolean} */ var stencil =
            (this.m_config.buffers & gl.STENCIL_BUFFER_BIT) != 0;

        // Textures
        var metaballsTex = context.createTexture();
        var quadsTex = context.createTexture();
        es3fFboRenderTest.createMetaballsTex2D(
            context, metaballsTex, gl.RGB, gl.UNSIGNED_BYTE, 64, 64
        );
        es3fFboRenderTest.createQuadsTex2D(
            context, quadsTex, gl.RGB, gl.UNSIGNED_BYTE, 64, 64
        );

        context.viewport(0, 0, width, height);

        // Fbo A
        /** @type {es3fFboRenderTest.Framebuffer} */
        var fboA = new es3fFboRenderTest.Framebuffer(
            context, this.m_config, width, height
        );

        fboA.checkCompleteness();

        // Fbo B
        /** @type {es3fFboRenderTest.FboConfig} */
        var cfg = /** @type {es3fFboRenderTest.FboConfig} */
            (deUtil.clone(this.m_config));

        cfg.buffers = deMath.binaryOp(
            cfg.buffers,
            deMath.binaryNot(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT),
            deMath.BinaryOp.AND
        );
        cfg.depthStencilType = gl.NONE;
        cfg.depthStencilFormat = gl.NONE;

        /** @type {es3fFboRenderTest.Framebuffer} */
        var fboB = new es3fFboRenderTest.Framebuffer(
            context, cfg, width, height
        );

        // Bind depth/stencil buffers from fbo A to fbo B
        context.bindFramebuffer(gl.FRAMEBUFFER, fboB.getFramebuffer());
        for (var ndx = 0; ndx < 2; ndx++) {
            var bit = ndx ? gl.STENCIL_BUFFER_BIT : gl.DEPTH_BUFFER_BIT;
            var point = ndx ? gl.STENCIL_ATTACHMENT : gl.DEPTH_ATTACHMENT;

            if (
                deMath.binaryOp(
                    this.m_config.buffers, bit, deMath.BinaryOp.AND
                ) == 0
            )
                continue;

            switch (this.m_config.depthStencilType) {
                case gl.TEXTURE_2D:
                    context.framebufferTexture2D(
                        gl.FRAMEBUFFER, point, gl.TEXTURE_2D,
                        fboA.getDepthStencilBuffer(), 0
                    );
                    break;
                case gl.RENDERBUFFER:
                    context.framebufferRenderbuffer(
                        gl.FRAMEBUFFER, point, gl.RENDERBUFFER,
                        fboA.getDepthStencilBuffer()
                    );
                    break;
                default:
                    testFailed('Not implemented');
            }
        }

        // Setup uniforms
        texShader.setUniforms(context, texShaderID);

        // Clear color to red and stencil to 1 in fbo B.
        context.clearColor(1.0, 0.0, 0.0, 1.0);
        context.clearStencil(1);
        context.clear(
            gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT
        );

        context.enable(gl.DEPTH_TEST);

        // Render quad to fbo A
        context.bindFramebuffer(gl.FRAMEBUFFER, fboA.getFramebuffer());
        context.bindTexture(gl.TEXTURE_2D, quadsTex);
        rrUtil.drawQuad(
            context, texShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        if (stencil) {
            // Clear subset of stencil buffer to 0 in fbo A
            context.enable(gl.SCISSOR_TEST);
            context.scissor(10, 10, 12, 25);
            context.clearStencil(0);
            context.clear(gl.STENCIL_BUFFER_BIT);
            context.disable(gl.SCISSOR_TEST);
        }

        // Render metaballs to fbo B
        context.bindFramebuffer(gl.FRAMEBUFFER, fboB.getFramebuffer());
        context.bindTexture(gl.TEXTURE_2D, metaballsTex);
        rrUtil.drawQuad(
            context, texShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]
        );

        context.disable(gl.DEPTH_TEST);

        if (stencil) {
            // Render quad with stencil mask == 0
            context.enable(gl.STENCIL_TEST);
            context.stencilFunc(gl.EQUAL, 0, 0xff);
            context.useProgram(flatShaderID);
            flatShader.setColor(context, flatShaderID, [0.0, 1.0, 0.0, 1.0]);
            rrUtil.drawQuad(
                context, flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );
            context.disable(gl.STENCIL_TEST);
        }

        if (this.m_config.colorType == gl.TEXTURE_2D) {
            // Render both to screen
            context.bindFramebuffer(gl.FRAMEBUFFER, null);
            context.viewport(0, 0, context.getWidth(), context.getHeight());
            context.bindTexture(gl.TEXTURE_2D, fboA.getColorBuffer());
            rrUtil.drawQuad(
                context, texShaderID, [-1.0, -1.0, 0.0], [0.0, 1.0, 0.0]
            );
            context.bindTexture(gl.TEXTURE_2D, fboB.getColorBuffer());
            rrUtil.drawQuad(
                context, texShaderID, [0.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );

            dst.readViewport(
                context, [0, 0, context.getWidth(), context.getHeight()]
            );
        } else {
            // Read results from fbo B
            es3fFboTestUtil.readPixels(
                context, dst, 0, 0, width, height,
                gluTextureUtil.mapGLInternalFormat(this.m_config.colorFormat),
                [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]
            );
        }
    };

    /**
     * @constructor
     * @extends {es3fFboRenderTest.FboRenderCase}
     * @param {es3fFboRenderTest.FboConfig} config
     */
    es3fFboRenderTest.ResizeTest = function(config) {
        es3fFboRenderTest.FboRenderCase.call(
            this, config.getName(), 'Resize framebuffer', config
        );
    };

    es3fFboRenderTest.ResizeTest.prototype =
        Object.create(es3fFboRenderTest.FboRenderCase.prototype);

    es3fFboRenderTest.ResizeTest.prototype.constructor =
        es3fFboRenderTest.ResizeTest;

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * context
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.ResizeTest.prototype.render = function(context, dst) {
        /** @type {tcuTexture.TextureFormat} */
        var colorFormat = gluTextureUtil.mapGLInternalFormat(
            this.m_config.colorFormat
        );
        /** @type {gluShaderUtil.DataType} */
        var fboSamplerType = gluTextureUtil.getSampler2DType(colorFormat);
        /** @type {gluShaderUtil.DataType} */
        var fboOutputType = es3fFboTestUtil.getFragmentOutputType(colorFormat);
        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fboRangeInfo = tcuTextureUtil.getTextureFormatInfo(colorFormat);
        var fboOutScale = deMath.subtract(
            fboRangeInfo.valueMax, fboRangeInfo.valueMin
        );
        var fboOutBias = fboRangeInfo.valueMin;

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFboShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], fboOutputType
        );

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texFromFboShader = new es3fFboTestUtil.Texture2DShader(
            [fboSamplerType], gluShaderUtil.DataType.FLOAT_VEC4
        );

        /** @type {es3fFboTestUtil.FlatColorShader} */
        var flatShader = new es3fFboTestUtil.FlatColorShader(fboOutputType);
        /** @type {WebGLProgram} */
        var texToFboShaderID = context.createProgram(texToFboShader);
        /** @type {WebGLProgram} */
        var texFromFboShaderID = context.createProgram(texFromFboShader);
        /** @type {WebGLProgram} */
        var flatShaderID = context.createProgram(flatShader);

        var quadsTex = context.createTexture();
        var metaballsTex = context.createTexture();

        var depth = deMath.binaryOp(
            this.m_config.buffers, gl.DEPTH_BUFFER_BIT, deMath.BinaryOp.AND
        ) != 0;
        var stencil = deMath.binaryOp(
            this.m_config.buffers, gl.STENCIL_BUFFER_BIT, deMath.BinaryOp.AND
        ) != 0;

        var initialWidth = 128;
        var initialHeight = 128;
        var newWidth = 64;
        var newHeight = 32;

        texToFboShader.setOutScaleBias(fboOutScale, fboOutBias);
        texFromFboShader.setTexScaleBias(
            0, fboRangeInfo.lookupScale, fboRangeInfo.lookupBias
        );

        es3fFboRenderTest.createQuadsTex2D(
            context, quadsTex, gl.RGB, gl.UNSIGNED_BYTE, 64, 64
        );
        es3fFboRenderTest.createMetaballsTex2D(
            context, metaballsTex, gl.RGB, gl.UNSIGNED_BYTE, 32, 32
        );

        /** @type {es3fFboRenderTest.Framebuffer} */
        var fbo = new es3fFboRenderTest.Framebuffer(
            context, this.m_config, initialWidth, initialHeight
        );
        fbo.checkCompleteness();

        // Setup shaders
        texToFboShader.setUniforms(context, texToFboShaderID);
        texFromFboShader.setUniforms(context, texFromFboShaderID);
        flatShader.setColor(
            context, flatShaderID, deMath.add(
                deMath.multiply([0.0, 1.0, 0.0, 1.0], fboOutScale), fboOutBias
            )
        );

        // Render quads
        context.bindFramebuffer(gl.FRAMEBUFFER, fbo.getFramebuffer());
        context.viewport(0, 0, initialWidth, initialHeight);
        es3fFboTestUtil.clearColorBuffer(
            context, colorFormat, [0.0, 0.0, 0.0, 1.0]
        );
        context.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        context.bindTexture(gl.TEXTURE_2D, quadsTex);
        rrUtil.drawQuad(
            context, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        if (fbo.getConfig().colorType == gl.TEXTURE_2D) {
            // Render fbo to screen
            context.bindFramebuffer(gl.FRAMEBUFFER, null);
            context.viewport(0, 0, context.getWidth(), context.getHeight());
            context.bindTexture(gl.TEXTURE_2D, fbo.getColorBuffer());
            rrUtil.drawQuad(
                context, texFromFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );
            // Restore binding
            context.bindFramebuffer(gl.FRAMEBUFFER, fbo.getFramebuffer());
        }

        // Resize buffers
        switch (fbo.getConfig().colorType) {
            case gl.TEXTURE_2D:
                context.bindTexture(gl.TEXTURE_2D, fbo.getColorBuffer());
                context.texImage2DDelegate(
                    gl.TEXTURE_2D, 0, fbo.getConfig().colorFormat,
                    newWidth, newHeight
                );
                break;

            case gl.RENDERBUFFER:
                context.bindRenderbuffer(gl.RENDERBUFFER, fbo.getColorBuffer());
                context.renderbufferStorage(
                    gl.RENDERBUFFER, fbo.getConfig().colorFormat,
                    newWidth, newHeight
                );
                break;

            default:
                throw new Error('Color type unsupported');
        }

        if (depth || stencil) {
            switch (fbo.getConfig().depthStencilType) {
                case gl.TEXTURE_2D:
                    context.bindTexture(
                        gl.TEXTURE_2D, fbo.getDepthStencilBuffer()
                    );
                    context.texImage2DDelegate(
                        gl.TEXTURE_2D, 0, fbo.getConfig().depthStencilFormat,
                        newWidth, newHeight
                    );
                    break;

                case gl.RENDERBUFFER:
                    context.bindRenderbuffer(
                        gl.RENDERBUFFER, fbo.getDepthStencilBuffer()
                    );
                    context.renderbufferStorage(
                        gl.RENDERBUFFER, fbo.getConfig().depthStencilFormat,
                        newWidth, newHeight
                    );
                    break;

                default:
                    throw new Error('Depth / stencil type unsupported');
            }
        }

        // Render to resized fbo
        context.viewport(0, 0, newWidth, newHeight);
        es3fFboTestUtil.clearColorBuffer(
            context, colorFormat, [1.0, 0.0, 0.0, 1.0]
        );
        context.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        context.enable(gl.DEPTH_TEST);

        context.bindTexture(gl.TEXTURE_2D, metaballsTex);
        rrUtil.drawQuad(
            context, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        context.bindTexture(gl.TEXTURE_2D, quadsTex);
        rrUtil.drawQuad(
            context, texToFboShaderID, [0.0, 0.0, -1.0], [1.0, 1.0, 1.0]
        );

        context.disable(gl.DEPTH_TEST);

        if (stencil) {
            context.enable(gl.SCISSOR_TEST);
            context.clearStencil(1);
            context.scissor(10, 10, 5, 15);
            context.clear(gl.STENCIL_BUFFER_BIT);
            context.disable(gl.SCISSOR_TEST);

            context.enable(gl.STENCIL_TEST);
            context.stencilFunc(gl.EQUAL, 1, 0xff);
            rrUtil.drawQuad(
                context, flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );
            context.disable(gl.STENCIL_TEST);
        }

        if (this.m_config.colorType == gl.TEXTURE_2D) {
            context.bindFramebuffer(gl.FRAMEBUFFER, null);
            context.viewport(0, 0, context.getWidth(), context.getHeight());
            context.bindTexture(gl.TEXTURE_2D, fbo.getColorBuffer());
            rrUtil.drawQuad(
                context, texFromFboShaderID, [-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]
            );
            dst.readViewport(
                context, [0, 0, context.getWidth(), context.getHeight()]
            );
        } else
            es3fFboTestUtil.readPixels(
                context, dst, 0, 0, newWidth, newHeight, colorFormat,
                fboRangeInfo.lookupScale, fboRangeInfo.lookupBias
            );
    };

    /**
     * @constructor
     * @extends {es3fFboRenderTest.FboRenderCase}
     * @param {es3fFboRenderTest.FboConfig} config
     * @param {number} buffers
     * @param {boolean} rebind
     */
    es3fFboRenderTest.RecreateBuffersTest = function(config, buffers, rebind) {
        es3fFboRenderTest.FboRenderCase.call(
            this, config.getName() +
            (rebind ? '' : '_no_rebind'),
            'Recreate buffers', config
        );
        this.m_buffers = buffers;
        this.m_rebind = rebind;
    };

    es3fFboRenderTest.RecreateBuffersTest.prototype =
        Object.create(es3fFboRenderTest.FboRenderCase.prototype);

    es3fFboRenderTest.RecreateBuffersTest.prototype.construtor =
        es3fFboRenderTest.RecreateBuffersTest;

    /**
     * @param {?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext}
     * ctx
     * @param {tcuSurface.Surface} dst
     */
    es3fFboRenderTest.RecreateBuffersTest.prototype.render = function(
        ctx, dst) {

        /** @type {tcuTexture.TextureFormat} */
        var colorFormat = gluTextureUtil.mapGLInternalFormat(
            this.m_config.colorFormat
        );
        /** @type {gluShaderUtil.DataType} */
        var fboSamplerType = gluTextureUtil.getSampler2DType(colorFormat);
        /** @type {gluShaderUtil.DataType} */
        var fboOutputType = es3fFboTestUtil.getFragmentOutputType(colorFormat);
        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fboRangeInfo = tcuTextureUtil.getTextureFormatInfo(colorFormat);
        var fboOutScale = deMath.subtract(
            fboRangeInfo.valueMax, fboRangeInfo.valueMin
        );
        var fboOutBias = fboRangeInfo.valueMin;

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFboShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], fboOutputType
        );
        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texFromFboShader = new es3fFboTestUtil.Texture2DShader(
            [fboSamplerType], gluShaderUtil.DataType.FLOAT_VEC4
        );

        /** @type {es3fFboTestUtil.FlatColorShader} */
        var flatShader = new es3fFboTestUtil.FlatColorShader(fboOutputType);
        /** @type {number} */
        var texToFboShaderID = ctx.createProgram(texToFboShader);
        /** @type {number} */
        var texFromFboShaderID = ctx.createProgram(texFromFboShader);
        /** @type {number} */
        var flatShaderID = ctx.createProgram(flatShader);

        var width = 128;
        var height = 128;
        var metaballsTex = ctx.createTexture();
        var quadsTex = ctx.createTexture();
        var stencil = deMath.binaryOp(
            this.m_config.buffers, gl.STENCIL_BUFFER_BIT, deMath.BinaryOp.AND
        ) != 0;

        es3fFboRenderTest.createQuadsTex2D(
            ctx, quadsTex, gl.RGB, gl.UNSIGNED_BYTE, 64, 64
        );
        es3fFboRenderTest.createMetaballsTex2D(
            ctx, metaballsTex, gl.RGB, gl.UNSIGNED_BYTE, 64, 64
        );

        /** @type {es3fFboRenderTest.Framebuffer} */
        var fbo = new es3fFboRenderTest.Framebuffer(
            ctx, this.m_config, width, height
        );
        fbo.checkCompleteness();

        // Setup shaders
        texToFboShader.setOutScaleBias(fboOutScale, fboOutBias);
        texFromFboShader.setTexScaleBias(
            0, fboRangeInfo.lookupScale, fboRangeInfo.lookupBias
        );
        texToFboShader.setUniforms(ctx, texToFboShaderID);
        texFromFboShader.setUniforms(ctx, texFromFboShaderID);
        flatShader.setColor(
            ctx, flatShaderID, deMath.add(
                deMath.multiply([0.0, 0.0, 1.0, 1.0], fboOutScale
            ), fboOutBias)
        );

        // Draw scene
        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo.getFramebuffer());
        ctx.viewport(0, 0, width, height);
        es3fFboTestUtil.clearColorBuffer(
            ctx, colorFormat, [1.0, 0.0, 0.0, 1.0]
        );
        ctx.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        ctx.enable(gl.DEPTH_TEST);

        ctx.bindTexture(gl.TEXTURE_2D, quadsTex);
        rrUtil.drawQuad(
            ctx, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        ctx.disable(gl.DEPTH_TEST);

        if (stencil) {
            ctx.enable(gl.SCISSOR_TEST);
            ctx.scissor(
                Math.floor(width / 4), Math.floor(height / 4),
                Math.floor(width / 2), Math.floor(height / 2)
            );
            ctx.clearStencil(1);
            ctx.clear(gl.STENCIL_BUFFER_BIT);
            ctx.disable(gl.SCISSOR_TEST);
        }

        // Recreate buffers
        if (!this.m_rebind)
            ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

        assertMsgOptions(
            deMath.binaryOp(
                this.m_buffers, deMath.binaryOp(
                    gl.DEPTH_BUFFER_BIT,
                    gl.STENCIL_BUFFER_BIT,
                    deMath.BinaryOp.OR
                ), deMath.BinaryOp.AND
            ) == 0 || deMath.binaryOp(
                this.m_buffers, deMath.binaryOp(
                    gl.DEPTH_BUFFER_BIT,
                    gl.STENCIL_BUFFER_BIT,
                    deMath.BinaryOp.OR
                ), deMath.BinaryOp.AND
            ) == deMath.binaryOp(
                    this.m_config.buffers, deMath.binaryOp(
                        gl.DEPTH_BUFFER_BIT,
                        gl.STENCIL_BUFFER_BIT,
                        deMath.BinaryOp.OR
                    ), deMath.BinaryOp.AND
            ), 'Depth/stencil buffers are not disabled or not ' +
            'equal to the config\'s depth/stencil buffer state',
            false, true
        );

        // Recreate.
        for (var ndx = 0; ndx < 2; ndx++) {
            var bit = ndx == 0 ? gl.COLOR_BUFFER_BIT :
                (gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
            var type = ndx == 0 ? fbo.getConfig().colorType :
                fbo.getConfig().depthStencilType;
            var format = ndx == 0 ? fbo.getConfig().colorFormat :
                fbo.getConfig().depthStencilFormat;
            var buf = ndx == 0 ? fbo.getColorBuffer() :
                fbo.getDepthStencilBuffer();

            if (deMath.binaryOp(this.m_buffers, bit, deMath.BinaryOp.AND) == 0)
                continue;

            switch (type) {
                case gl.TEXTURE_2D:
                    ctx.deleteTexture(/** @type {WebGLTexture} */ (buf));
                    buf = ctx.createTexture();
                    ctx.bindTexture(gl.TEXTURE_2D, buf);
                    ctx.texImage2DDelegate(
                        gl.TEXTURE_2D, 0, format, width, height
                    );
                    ctx.texParameteri(
                        gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST
                    );
                    ctx.texParameteri(
                        gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST
                    );
                    break;

                case gl.RENDERBUFFER:
                    ctx.deleteRenderbuffer(
                        /** @type {WebGLRenderbuffer} */ (buf)
                    );
                    buf = ctx.createRenderbuffer();
                    ctx.bindRenderbuffer(gl.RENDERBUFFER, buf);
                    ctx.renderbufferStorage(
                        gl.RENDERBUFFER, format, width, height
                    );
                    break;

                default:
                    throw new Error('Unsupported buffer type');
            }

            if (ndx == 0) {
                fbo.m_colorBuffer = buf;
            } else {
                fbo.m_depthStencilBuffer = buf;
            }
        }

        // Rebind.
        if (this.m_rebind) {
            for (var ndx = 0; ndx < 3; ndx++) {
                var bit = ndx == 0 ? gl.COLOR_BUFFER_BIT :
                    ndx == 1 ? gl.DEPTH_BUFFER_BIT :
                    ndx == 2 ? gl.STENCIL_BUFFER_BIT : 0;
                var point = ndx == 0 ? gl.COLOR_ATTACHMENT0 :
                    ndx == 1 ? gl.DEPTH_ATTACHMENT :
                    ndx == 2 ? gl.STENCIL_ATTACHMENT : 0;
                var type = ndx == 0 ? fbo.getConfig().colorType :
                    fbo.getConfig().depthStencilType;
                var buf = ndx == 0 ? fbo.getColorBuffer() :
                    fbo.getDepthStencilBuffer();

                if (deMath.binaryOp(
                        this.m_buffers, bit, deMath.BinaryOp.AND) == 0)
                    continue;

                switch (type) {
                    case gl.TEXTURE_2D:
                        ctx.framebufferTexture2D(
                            gl.FRAMEBUFFER, point, gl.TEXTURE_2D, buf, 0
                        );
                        break;

                    case gl.RENDERBUFFER:
                        ctx.framebufferRenderbuffer(
                            gl.FRAMEBUFFER, point, gl.RENDERBUFFER, buf
                        );
                        break;

                    default:
                        throw new Error('Invalid buffer type');
                }
            }
        }

        if (!this.m_rebind)
            ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo.getFramebuffer());

        ctx.clearStencil(0);

        // \note Clear only buffers that were re-created
        ctx.clear(
            deMath.binaryOp(
                this.m_buffers,
                deMath.binaryOp(
                    gl.DEPTH_BUFFER_BIT,
                    gl.STENCIL_BUFFER_BIT,
                    deMath.BinaryOp.OR
                ), deMath.BinaryOp.AND
            )
        );

        if (deMath.binaryOp(
            this.m_buffers, gl.COLOR_BUFFER_BIT, deMath.BinaryOp.AND)) {
            // Clearing of integer buffers is undefined
            // so do clearing by rendering flat color.
            rrUtil.drawQuad(
                ctx, flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );
        }

        ctx.enable(gl.DEPTH_TEST);

        if (stencil) {
            // \note Stencil test enabled only if we have stencil buffer
            ctx.enable(gl.STENCIL_TEST);
            ctx.stencilFunc(gl.EQUAL, 0, 0xff);
        }
        ctx.bindTexture(gl.TEXTURE_2D, metaballsTex);
        rrUtil.drawQuad(
            ctx, texToFboShaderID, [-1.0, -1.0, 1.0], [1.0, 1.0, -1.0]
        );
        if (stencil)
            ctx.disable(gl.STENCIL_TEST);

        ctx.disable(gl.DEPTH_TEST);

        if (fbo.getConfig().colorType == gl.TEXTURE_2D) {
            // Unbind fbo
            ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

            // Draw to screen
            ctx.bindTexture(gl.TEXTURE_2D, fbo.getColorBuffer());
            ctx.viewport(0, 0, ctx.getWidth(), ctx.getHeight());
            rrUtil.drawQuad(
                ctx, texFromFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
            );

            // Read from screen
            dst.readViewport(ctx, [0, 0, ctx.getWidth(), ctx.getHeight()]);
        } else {
            // Read from fbo
            es3fFboTestUtil.readPixels(
                ctx, dst, 0, 0, width, height, colorFormat,
                fboRangeInfo.lookupScale, fboRangeInfo.lookupBias
            );
        }
    };

    // FboGroups

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fFboRenderTest.FboRenderTestGroup = function() {
        tcuTestCase.DeqpTest.call(this, 'render', 'Rendering Tests');
    };

    es3fFboRenderTest.FboRenderTestGroup.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);

    es3fFboRenderTest.FboRenderTestGroup.prototype.constructor =
        es3fFboRenderTest.FboRenderTestGroup;

    /**
     * @enum {number}
     */
    var FormatType = {
        FLOAT: 0,
        INT: 1,
        UINT: 2
    };

    // Required by specification.
    /**
     * @typedef {{format: number, type: FormatType}}
     */
    var ColorFormatStruct;

    /**
     * @typedef {{format: number, depth: boolean, stencil: boolean}}
     */
    var DepthStencilFormatStruct;

    /**
     * init
     */
    es3fFboRenderTest.FboRenderTestGroup.prototype.init = function() {
        var objectTypes = [
            gl.TEXTURE_2D,
            gl.RENDERBUFFER
        ];

        /** @type {Array<ColorFormatStruct>} */ var colorFormats = [{
                format: gl.RGBA32F, type: FormatType.FLOAT
            },{
                format: gl.RGBA32I, type: FormatType.INT
            },{
                format: gl.RGBA32UI, type: FormatType.UINT
            },{
                format: gl.RGBA16F, type: FormatType.FLOAT
            },{
                format: gl.RGBA16I, type: FormatType.INT
            },{
                format: gl.RGBA16UI, type: FormatType.UINT
            },/*{
                // RGB16F isn't made color-renderable through WebGL's EXT_color_buffer_float
                format: gl.RGB16F, type: FormatType.FLOAT
            },*/{
                format: gl.RGBA8I, type: FormatType.INT
            },{
                format: gl.RGBA8UI, type: FormatType.UINT
            },{
                format: gl.RGB10_A2UI, type: FormatType.UINT
            },{
                format: gl.R11F_G11F_B10F, type: FormatType.FLOAT
            },{
                format: gl.RG32F, type: FormatType.FLOAT
            },{
                format: gl.RG32I, type: FormatType.INT
            },{
                format: gl.RG32UI, type: FormatType.UINT
            },{
                format: gl.RG16F, type: FormatType.FLOAT
            },{
                format: gl.RG16I, type: FormatType.INT
            },{
                format: gl.RG16UI, type: FormatType.UINT
            },{
                format: gl.RG8, type: FormatType.FLOAT
            },{
                format: gl.RG8I, type: FormatType.INT
            },{
                format: gl.RG8UI, type: FormatType.UINT
            },{
                format: gl.R32F, type: FormatType.FLOAT
            },{
                format: gl.R32I, type: FormatType.INT
            },{
                format: gl.R32UI, type: FormatType.UINT
            },{
                format: gl.R16F, type: FormatType.FLOAT
            },{
                format: gl.R16I, type: FormatType.INT
            },{
                format: gl.R16UI, type: FormatType.UINT
            },{
                format: gl.R8, type: FormatType.FLOAT
            },{
                format: gl.R8I, type: FormatType.INT
            },{
                format: gl.R8UI, type: FormatType.UINT
        }];

        /** @type {Array<DepthStencilFormatStruct>} */
        var depthStencilFormats = [{
                format: gl.DEPTH_COMPONENT32F, depth: true, stencil: false
            },{
                format: gl.DEPTH_COMPONENT24, depth: true, stencil: false
            },{
                format: gl.DEPTH_COMPONENT16, depth: true, stencil: false
            },{
                format: gl.DEPTH32F_STENCIL8, depth: true, stencil: true
            },{
                format: gl.DEPTH24_STENCIL8, depth: true, stencil: true
            },{
                format: gl.STENCIL_INDEX8, depth: false, stencil: true
        }];

        /** @type {es3fFboRenderTest.FboConfig} */ var config;
        var colorType;
        var stencilType;
        var colorFmt;
        var depth;
        var stencil;
        var depthStencilType;
        var depthStencilFormat;

        // .stencil_clear
        /** @type {tcuTestCase.DeqpTest} */
        var stencilClearGroup = new tcuTestCase.DeqpTest(
            'stencil_clear', 'Stencil buffer clears'
        );

        this.addChild(stencilClearGroup);

        for (var fmtNdx = 0; fmtNdx < depthStencilFormats.length; fmtNdx++) {
            colorType = gl.TEXTURE_2D;
            stencilType = gl.RENDERBUFFER;
            colorFmt = gl.RGBA8;

            if (!depthStencilFormats[fmtNdx].stencil)
                continue;

            config = new es3fFboRenderTest.FboConfig(
                gl.COLOR_BUFFER_BIT | gl.STENCIL_BUFFER_BIT,
                colorType, colorFmt, stencilType,
                depthStencilFormats[fmtNdx].format
            );
            stencilClearGroup.addChild(
                new es3fFboRenderTest.StencilClearsTest(config)
            );
        }

        // .shared_colorbuffer_clear
        /** @type {tcuTestCase.DeqpTest} */
        var sharedColorbufferClearGroup = new tcuTestCase.DeqpTest(
            'shared_colorbuffer_clear', 'Shader colorbuffer clears'
        );

        this.addChild(sharedColorbufferClearGroup);

        for (var colorFmtNdx = 0;
            colorFmtNdx < colorFormats.length;
            colorFmtNdx++) {

            // Clearing of integer buffers is undefined.
            if (colorFormats[colorFmtNdx].type == FormatType.INT ||
                colorFormats[colorFmtNdx].type == FormatType.UINT)
                continue;

            for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {
                config = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT, objectTypes[typeNdx],
                    colorFormats[colorFmtNdx].format, gl.NONE, gl.NONE
                );
                sharedColorbufferClearGroup.addChild(
                    new es3fFboRenderTest.SharedColorbufferClearsTest(config)
                );
            }
        }

        // .shared_colorbuffer
        /** @type {Array<tcuTestCase.DeqpTest>} */ var sharedColorbufferGroup = [];
        var numSharedColorbufferGroups = 3;
        for (var ii = 0; ii < numSharedColorbufferGroups; ++ii) {
            sharedColorbufferGroup[ii] = new tcuTestCase.DeqpTest(
                'shared_colorbuffer', 'Shared colorbuffer tests'
            );
            this.addChild(sharedColorbufferGroup[ii]);
        }

        for (var colorFmtNdx = 0; colorFmtNdx < colorFormats.length; colorFmtNdx++) {

            depthStencilType = gl.RENDERBUFFER;
            depthStencilFormat = gl.DEPTH24_STENCIL8;

            // Blending with integer buffers and fp32 targets is not supported.
            if (colorFormats[colorFmtNdx].type == FormatType.INT ||
                colorFormats[colorFmtNdx].type == FormatType.UINT ||
                colorFormats[colorFmtNdx].format == gl.RGBA32F ||
                colorFormats[colorFmtNdx].format == gl.RGB32F ||
                colorFormats[colorFmtNdx].format == gl.RG32F ||
                colorFormats[colorFmtNdx].format == gl.R32F)
                continue;

            for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {
                /** @type {es3fFboRenderTest.FboConfig} */
                var colorOnlyConfig = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT, objectTypes[typeNdx],
                    colorFormats[colorFmtNdx].format, gl.NONE, gl.NONE
                );
                /** @type {es3fFboRenderTest.FboConfig} */
                var colorDepthConfig = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT,
                    objectTypes[typeNdx], colorFormats[colorFmtNdx].format,
                    depthStencilType, depthStencilFormat
                );
                /** @type {es3fFboRenderTest.FboConfig} */
                var colorDepthStencilConfig =
                    new es3fFboRenderTest.FboConfig(
                        gl.COLOR_BUFFER_BIT |
                        gl.DEPTH_BUFFER_BIT |
                        gl.STENCIL_BUFFER_BIT,
                        objectTypes[typeNdx], colorFormats[colorFmtNdx].format,
                        depthStencilType, depthStencilFormat
                );

                sharedColorbufferGroup[0].addChild(
                    new es3fFboRenderTest.SharedColorbufferTest(colorOnlyConfig)
                );

                sharedColorbufferGroup[1].addChild(
                    new es3fFboRenderTest.SharedColorbufferTest(
                        colorDepthConfig
                    )
                );

                sharedColorbufferGroup[2].addChild(
                    new es3fFboRenderTest.SharedColorbufferTest(
                        colorDepthStencilConfig
                    )
                );
            }
        }

        // .shared_depth_stencil
        /** @type {tcuTestCase.DeqpTest} */
        var sharedDepthStencilGroup = new tcuTestCase.DeqpTest(
            'shared_depth_stencil', 'Shared depth and stencil buffers'
        );

        this.addChild(sharedDepthStencilGroup);

        for (var fmtNdx = 0; fmtNdx < depthStencilFormats.length; fmtNdx++) {
            colorType = gl.TEXTURE_2D;
            colorFmt = gl.RGBA8;
            depth = depthStencilFormats[fmtNdx].depth;
            stencil = depthStencilFormats[fmtNdx].stencil;

            if (!depth)
                continue; // Not verified.

            // Depth and stencil: both rbo and textures
            for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {
                config = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT |
                    (depth ? gl.DEPTH_BUFFER_BIT : 0) |
                    (stencil ? gl.STENCIL_BUFFER_BIT : 0),
                    colorType, colorFmt, objectTypes[typeNdx],
                    depthStencilFormats[fmtNdx].format
                );

                sharedDepthStencilGroup.addChild(
                    new es3fFboRenderTest.SharedDepthStencilTest(config)
                );
            }
        }

        // .resize
        /** @type {Array<tcuTestCase.DeqpTest>} */ var resizeGroup = [];
        var numResizeGroups = 4;
        for (var ii = 0; ii < numResizeGroups; ++ii) {
            resizeGroup[ii] = new tcuTestCase.DeqpTest('resize', 'FBO resize tests');
            this.addChild(resizeGroup[ii]);
        }

        for (var colorFmtNdx = 0; colorFmtNdx < colorFormats.length; colorFmtNdx++) {

            var colorFormat = colorFormats[colorFmtNdx].format;

            // Color-only.
            for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {
                config = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT, objectTypes[typeNdx],
                    colorFormat, gl.NONE, gl.NONE
                );
                resizeGroup[colorFmtNdx % numResizeGroups].addChild(new es3fFboRenderTest.ResizeTest(config));
            }

            // For selected color formats tests depth & stencil variants.
            if (colorFormat == gl.RGBA8 || colorFormat == gl.RGBA16F) {
                for (var depthStencilFmtNdx = 0; depthStencilFmtNdx < depthStencilFormats.length; depthStencilFmtNdx++) {

                    colorType = gl.TEXTURE_2D;
                    depth = depthStencilFormats[depthStencilFmtNdx].depth;
                    stencil = depthStencilFormats[depthStencilFmtNdx].stencil;

                    // Depth and stencil: both rbo and textures
                    for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {

                        if (!depth && objectTypes[typeNdx] != gl.RENDERBUFFER)
                            continue; // Not supported.

                        config = new es3fFboRenderTest.FboConfig(
                            gl.COLOR_BUFFER_BIT |
                            (depth ? gl.DEPTH_BUFFER_BIT : 0) |
                            (stencil ? gl.STENCIL_BUFFER_BIT : 0),
                            colorType, colorFormat, objectTypes[typeNdx],
                            depthStencilFormats[depthStencilFmtNdx].format
                        );

                        resizeGroup[colorFmtNdx % numResizeGroups].addChild(
                            new es3fFboRenderTest.ResizeTest(config)
                        );
                    }
                }
            }
        }

        // .recreate_color
        /** @type {Array<tcuTestCase.DeqpTest>} */ var recreateColorGroup = [];
        var numRecreateColorGroups = 7;
        for (var ii = 0; ii < numRecreateColorGroups; ++ii) {
            recreateColorGroup[ii] = new tcuTestCase.DeqpTest('recreate_color', 'Recreate colorbuffer tests');
            this.addChild(recreateColorGroup[ii]);
        }

        for (var colorFmtNdx = 0; colorFmtNdx < colorFormats.length; colorFmtNdx++) {

            colorFormat = colorFormats[colorFmtNdx].format;
            depthStencilFormat = gl.DEPTH24_STENCIL8;
            depthStencilType = gl.RENDERBUFFER;

            // Color-only.
            for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {
                config = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT |
                    gl.DEPTH_BUFFER_BIT |
                    gl.STENCIL_BUFFER_BIT,
                    objectTypes[typeNdx], colorFormat,
                    depthStencilType, depthStencilFormat
                );

                recreateColorGroup[colorFmtNdx % numRecreateColorGroups].addChild(
                    new es3fFboRenderTest.RecreateBuffersTest(
                        config, gl.COLOR_BUFFER_BIT, true /* rebind */
                    )
                );
            }
        }

        // .recreate_depth_stencil
        /** @type {tcuTestCase.DeqpTest} */
        var recreateDepthStencilGroup = new tcuTestCase.DeqpTest(
            'recreate_depth_stencil', 'Recreate depth and stencil buffers'
        );

        this.addChild(recreateDepthStencilGroup);

        for (var fmtNdx = 0; fmtNdx < depthStencilFormats.length; fmtNdx++) {
            colorType = gl.TEXTURE_2D;
            colorFmt = gl.RGBA8;
            depth = depthStencilFormats[fmtNdx].depth;
            stencil = depthStencilFormats[fmtNdx].stencil;

            // Depth and stencil: both rbo and textures
            for (var typeNdx = 0; typeNdx < objectTypes.length; typeNdx++) {
                if (!depth && objectTypes[typeNdx] != gl.RENDERBUFFER)
                    continue;

                config = new es3fFboRenderTest.FboConfig(
                    gl.COLOR_BUFFER_BIT |
                    (depth ? gl.DEPTH_BUFFER_BIT : 0) |
                    (stencil ? gl.STENCIL_BUFFER_BIT : 0),
                    colorType, colorFmt, objectTypes[typeNdx],
                    depthStencilFormats[fmtNdx].format
                );

                recreateDepthStencilGroup.addChild(
                    new es3fFboRenderTest.RecreateBuffersTest(
                        config,
                        (depth ? gl.DEPTH_BUFFER_BIT : 0) |
                        (stencil ? gl.STENCIL_BUFFER_BIT : 0),
                        true /* rebind */
                    )
                );
            }
        }
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     */
    es3fFboRenderTest.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;

        state.setRoot(new es3fFboRenderTest.FboRenderTestGroup());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to run tests', false);
            tcuTestCase.runner.terminate();
        }
    };
});

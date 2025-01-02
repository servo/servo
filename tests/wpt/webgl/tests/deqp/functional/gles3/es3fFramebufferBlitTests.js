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
goog.provide('functional.gles3.es3fFramebufferBlitTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestCase');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {

    var es3fFramebufferBlitTests = functional.gles3.es3fFramebufferBlitTests;
    var es3fFboTestCase = functional.gles3.es3fFboTestCase;
    var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var deMath = framework.delibs.debase.deMath;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var rrUtil = framework.referencerenderer.rrUtil;
    var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /** @type {WebGL2RenderingContext} */ var gl;
    /**
    * es3fFramebufferBlitTests.BlitRectCase class, inherits from FboTestCase
    * @constructor
    * @extends {es3fFboTestCase.FboTestCase}
    * @param {string} name
    * @param {string} desc
    * @param {number} filter deUint32
    * @param {Array<number>} srcSize
    * @param {Array<number>} srcRect
    * @param {Array<number>} dstSize
    * @param {Array<number>} dstRect
    * @param {number=} cellSize
    */
    es3fFramebufferBlitTests.BlitRectCase = function(name, desc, filter, srcSize, srcRect, dstSize, dstRect, cellSize) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @const {number} */ this.m_filter = filter;
        /** @const {Array<number>} */ this.m_srcSize = srcSize;
        /** @const {Array<number>} */ this.m_srcRect = srcRect;
        /** @const {Array<number>} */ this.m_dstSize = dstSize;
        /** @const {Array<number>} */ this.m_dstRect = dstRect;
        /** @const {number} */ this.m_cellSize = cellSize === undefined ? 8 : cellSize;
        /** @const {Array<number>} */ this.m_gridCellColorA = [0.2, 0.7, 0.1, 1.0];
        /** @const {Array<number>} */ this.m_gridCellColorB = [0.7, 0.1, 0.5, 0.8];
    };

    es3fFramebufferBlitTests.BlitRectCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFramebufferBlitTests.BlitRectCase.prototype.constructor = es3fFramebufferBlitTests.BlitRectCase;

    /**
    * @param {tcuSurface.Surface} dst
    */
    es3fFramebufferBlitTests.BlitRectCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        /** @type {number} */ var colorFormat = gl.RGBA8;

        /** @type {es3fFboTestUtil.GradientShader} */
        var gradShader = new es3fFboTestUtil.GradientShader(
            gluShaderUtil.DataType.FLOAT_VEC4);
        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D],
            gluShaderUtil.DataType.FLOAT_VEC4);

        var gradShaderID = ctx.createProgram(gradShader);
        var texShaderID = ctx.createProgram(texShader);

        var srcFbo;
        var dstFbo;
        var srcRbo;
        var dstRbo;

        // Setup shaders
        gradShader.setGradient(ctx, gradShaderID, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        texShader.setUniforms(ctx, texShaderID);

        // Create framebuffers.

        /** @type {Array<number>} */ var size;

        // source framebuffers
        srcFbo = ctx.createFramebuffer();
        srcRbo = ctx.createRenderbuffer();
        size = this.m_srcSize;

        ctx.bindRenderbuffer(gl.RENDERBUFFER, srcRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, size[0], size[1]);
        ctx.bindFramebuffer(gl.FRAMEBUFFER, srcFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, srcRbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        // destination framebuffers
        dstFbo = ctx.createFramebuffer();
        dstRbo = ctx.createRenderbuffer();
        size = this.m_dstSize;

        ctx.bindRenderbuffer(gl.RENDERBUFFER, dstRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, size[0], size[1]);
        ctx.bindFramebuffer(gl.FRAMEBUFFER, dstFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, dstRbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        // Fill destination with gradient.
        ctx.bindFramebuffer(gl.FRAMEBUFFER, dstFbo);
        ctx.viewport(0, 0, this.m_dstSize[0], this.m_dstSize[1]);

        rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, 0], [1, 1, 0]);

        // Fill source with grid pattern.
        /** @const {number} */ var format = gl.RGBA;
        /** @const {number} */ var dataType = gl.UNSIGNED_BYTE;
        /** @const {number} */ var texW = this.m_srcSize[0];
        /** @const {number} */ var texH = this.m_srcSize[1];
        var gridTex;
        /** @type {tcuTexture.TextureLevel} */ var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

        tcuTextureUtil.fillWithGrid(data.getAccess(), this.m_cellSize, this.m_gridCellColorA, this.m_gridCellColorB);

        gridTex = ctx.createTexture();
        ctx.bindTexture(gl.TEXTURE_2D, gridTex);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

        ctx.bindFramebuffer(gl.FRAMEBUFFER, srcFbo);
        ctx.viewport(0, 0, this.m_srcSize[0], this.m_srcSize[1]);

        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);

        // Perform copy.
        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, srcFbo);
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, dstFbo);
        ctx.blitFramebuffer(this.m_srcRect[0], this.m_srcRect[1], this.m_srcRect[2], this.m_srcRect[3],
                           this.m_dstRect[0], this.m_dstRect[1], this.m_dstRect[2], this.m_dstRect[3],
                           gl.COLOR_BUFFER_BIT, this.m_filter);

        // Read back results.
        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, dstFbo);

        this.readPixelsUsingFormat(dst, 0, 0, this.m_dstSize[0], this.m_dstSize[1],
                                          gluTextureUtil.mapGLInternalFormat(colorFormat),
                                          [1.0, 1.0, 1.0, 1.0],
                                          [0.0, 0.0, 0.0, 0.0]);
    };

    /**
    * @param {tcuSurface.Surface} reference
    * @param {tcuSurface.Surface} result
    * @return {boolean}
    */
    es3fFramebufferBlitTests.BlitRectCase.prototype.compare = function(reference, result) {
        // Use pixel-threshold compare for rect cases since 1px off will mean failure.
        var threshold = [7, 7, 7, 7];
        return tcuImageCompare.pixelThresholdCompare('Result', 'Image comparison result', reference, result, threshold);
    };

    /**
    * es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase class
    * @constructor
    * @extends {es3fFramebufferBlitTests.BlitRectCase}
    * @param {string} name
    * @param {string} desc
    * @param {Array<number>} srcSize
    * @param {Array<number>} srcRect
    * @param {Array<number>} dstSize
    * @param {Array<number>} dstRect
    */
    es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase = function(name, desc, srcSize, srcRect, dstSize, dstRect) {
        es3fFramebufferBlitTests.BlitRectCase.call(this, name, desc, gl.NEAREST, srcSize, srcRect, dstSize, dstRect, 1);
    };

    es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase.prototype = Object.create(es3fFramebufferBlitTests.BlitRectCase.prototype);
    es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase.prototype.constructor = es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase;

    /**
    * @param {tcuSurface.Surface} reference
    * @param {tcuSurface.Surface} result
    * @return {boolean}
    */
    es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase.prototype.compare = function(reference, result) {
        assertMsgOptions(result.getWidth() == reference.getWidth() && result.getHeight() == reference.getHeight(),
            'Reference and result images have different dimensions', false, true);

        // Image origin must be visible (for baseColor)
        DE_ASSERT(Math.min(this.m_dstRect[0], this.m_dstRect[2]) >= 0);
        DE_ASSERT(Math.min(this.m_dstRect[1], this.m_dstRect[3]) >= 0);
        /** @const {tcuRGBA.RGBA} */ var cellColorA = tcuRGBA.newRGBAFromVec(this.m_gridCellColorA);
        /** @const {tcuRGBA.RGBA} */ var cellColorB = tcuRGBA.newRGBAFromVec(this.m_gridCellColorB);
        // TODO: implement
        // const tcu::RGBA threshold = this.m_context.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(7,7,7,7);
        /** @type {tcuRGBA.RGBA} */ var threshold = tcuRGBA.newRGBAComponents(7, 7, 7, 7);
        /** @const {Array<number>} */  //IVec4.xyzw
        var destinationArea = [
            deMath.clamp(Math.min(this.m_dstRect[0], this.m_dstRect[2]), 0, result.getWidth()),
            deMath.clamp(Math.min(this.m_dstRect[1], this.m_dstRect[3]), 0, result.getHeight()),
            deMath.clamp(Math.max(this.m_dstRect[0], this.m_dstRect[2]), 0, result.getWidth()),
            deMath.clamp(Math.max(this.m_dstRect[1], this.m_dstRect[3]), 0, result.getHeight())];

        /** @const {tcuRGBA.RGBA} */ var baseColor = new tcuRGBA.RGBA(result.getPixel(destinationArea[0], destinationArea[1]));

        /** @const {boolean} */ var signConfig = tcuRGBA.compareThreshold(baseColor, cellColorA, threshold);

        /** @type {boolean} */ var error = false;
        /** @type {tcuSurface.Surface} */ var errorMask = new tcuSurface.Surface(result.getWidth(), result.getHeight());
        /** @type {Array<boolean>} */ var horisontalSign = [];
        /** @type {Array<boolean>} */ var verticalSign = [];

        errorMask.getAccess().clear([0.0, 1.0, 0.0, 1.0]);

        // Checking only area in our destination rect

        // m_testCtx.getLog()
        //     << tcu::TestLog::Message
        //     << 'Verifying consistency of NEAREST filtering. Verifying rect ' << m_dstRect << '.\n'
        //     << 'Rounding direction of the NEAREST filter at the horisontal texel edge (x = n + 0.5) should not depend on the y-coordinate.\n'
        //     << 'Rounding direction of the NEAREST filter at the vertical texel edge (y = n + 0.5) should not depend on the x-coordinate.\n'
        //     << 'Blitting a grid (with uniform sized cells) should result in a grid (with non-uniform sized cells).'
        //     << tcu::TestLog::EndMessage;

        // Verify that destination only contains valid colors

        /** @type {tcuRGBA.RGBA} */ var color;

        for (var dy = 0; dy < destinationArea[3] - destinationArea[1]; ++dy) {
            for (var dx = 0; dx < destinationArea[2] - destinationArea[0]; ++dx) {
                color = new tcuRGBA.RGBA(result.getPixel(destinationArea[0] + dx, destinationArea[1] + dy));

                /** @const {boolean} */
                var isValidColor =
                            tcuRGBA.compareThreshold(color, cellColorA, threshold) ||
                            tcuRGBA.compareThreshold(color, cellColorB, threshold);

                if (!isValidColor) {
                    errorMask.setPixel(destinationArea[0] + dx, destinationArea[1] + dy, tcuRGBA.RGBA.red.toVec());
                    error = true;
                }
            }
        }

        if (error) {
            // m_testCtx.getLog()
            //     << tcu::TestLog::Message
            //     << 'Image verification failed, destination rect contains unexpected values. '
            //     << 'Expected either ' << cellColorA << ' or ' << cellColorB << '.'
            //     << tcu::TestLog::EndMessage
            //     << tcu::TestLog::ImageSet('Result', 'Image verification result')
            //     << tcu::TestLog::Image('Result', 'Result', result)
            //     << tcu::TestLog::Image('ErrorMask', 'Error mask', errorMask)
            //     << tcu::TestLog::EndImageSet;
            return false;
        }

        // Detect result edges by reading the first row and first column of the blitted area.
        // Blitting a grid should result in a grid-like image. ('sign changes' should be consistent)

        for (var dx = 0; dx < destinationArea[2] - destinationArea[0]; ++dx) {
            color = new tcuRGBA.RGBA(result.getPixel(destinationArea[0] + dx, destinationArea[1]));
            if (tcuRGBA.compareThreshold(color, cellColorA, threshold))
                horisontalSign[dx] = true;
            else if (tcuRGBA.compareThreshold(color, cellColorB, threshold))
                horisontalSign[dx] = false;
            else
                DE_ASSERT(false);
        }
        for (var dy = 0; dy < destinationArea[3] - destinationArea[1]; ++dy) {
            color = new tcuRGBA.RGBA(result.getPixel(destinationArea[0], destinationArea[1] + dy));

            if (tcuRGBA.compareThreshold(color, cellColorA, threshold))
                verticalSign[dy] = true;
            else if (tcuRGBA.compareThreshold(color, cellColorB, threshold))
                verticalSign[dy] = false;
            else
                DE_ASSERT(false);
        }

        // Verify grid-like image

        for (var dy = 0; dy < destinationArea[3] - destinationArea[1]; ++dy) {
            for (var dx = 0; dx < destinationArea[2] - destinationArea[0]; ++dx) {
                color = new tcuRGBA.RGBA(result.getPixel(destinationArea[0] + dx, destinationArea[1] + dy));
                /** @const {boolean} */ var resultSign = tcuRGBA.compareThreshold(cellColorA, color, threshold);
                /** @const {boolean} */ var correctSign = (horisontalSign[dx] == verticalSign[dy]) == signConfig;

                if (resultSign != correctSign) {
                    errorMask.setPixel(destinationArea[0] + dx, destinationArea[1] + dy, tcuRGBA.RGBA.red.toVec());
                    error = true;
                }
            }
        }
        // Report result

        // if (error)
        // {
        //     m_testCtx.getLog()
        //         << tcu::TestLog::Message
        //         << 'Image verification failed, nearest filter is not consistent.'
        //         << tcu::TestLog::EndMessage
        //         << tcu::TestLog::ImageSet('Result', 'Image verification result')
        //         << tcu::TestLog::Image('Result', 'Result', result)
        //         << tcu::TestLog::Image('ErrorMask', 'Error mask', errorMask)
        //         << tcu::TestLog::EndImageSet;
        // }
        // else
        // {
        //     m_testCtx.getLog()
        //         << tcu::TestLog::Message
        //         << 'Image verification passed.'
        //         << tcu::TestLog::EndMessage
        //         << tcu::TestLog::ImageSet('Result', 'Image verification result')
        //         << tcu::TestLog::Image('Result', 'Result', result)
        //         << tcu::TestLog::EndImageSet;
        // }

        return !error;
    };

    /**
    * es3fFramebufferBlitTests.FramebufferBlitTests class, inherits from TestCase
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fFramebufferBlitTests.FramebufferBlitTests = function() {
        tcuTestCase.DeqpTest.call(this, 'blit', 'Framebuffer blit tests');
    };

    es3fFramebufferBlitTests.FramebufferBlitTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fFramebufferBlitTests.FramebufferBlitTests.prototype.constructor = es3fFramebufferBlitTests.FramebufferBlitTests;

    es3fFramebufferBlitTests.FramebufferBlitTests.prototype.init = function() {
        /** @const {Array<number>} */ var colorFormats = [
            // RGBA formats
            gl.RGBA32I,
            gl.RGBA32UI,
            gl.RGBA16I,
            gl.RGBA16UI,
            gl.RGBA8,
            gl.RGBA8I,
            gl.RGBA8UI,
            gl.SRGB8_ALPHA8,
            gl.RGB10_A2,
            gl.RGB10_A2UI,
            gl.RGBA4,
            gl.RGB5_A1,

            // RGB formats
            gl.RGB8,
            gl.RGB565,

            // RG formats
            gl.RG32I,
            gl.RG32UI,
            gl.RG16I,
            gl.RG16UI,
            gl.RG8,
            gl.RG8I,
            gl.RG8UI,

            // R formats
            gl.R32I,
            gl.R32UI,
            gl.R16I,
            gl.R16UI,
            gl.R8,
            gl.R8I,
            gl.R8UI,

            // gl.EXT_color_buffer_float
            gl.RGBA32F,
            gl.RGBA16F,
            gl.R11F_G11F_B10F,
            gl.RG32F,
            gl.RG16F,
            gl.R32F,
            gl.R16F
        ];

        /** @const {Array<number>} */ var depthStencilFormats = [
            gl.DEPTH_COMPONENT32F,
            gl.DEPTH_COMPONENT24,
            gl.DEPTH_COMPONENT16,
            gl.DEPTH32F_STENCIL8,
            gl.DEPTH24_STENCIL8,
            gl.STENCIL_INDEX8
        ];

        // .rect
        /** @constructor
         * @param {string} name
         * @param {Array<number>} srcRect
         * @param {Array<number>} dstRect
         */
        var CopyRect = function(name, srcRect, dstRect) {
            /** @const {string} */ this.name = name;
            /** @type {Array<number>} */ this.srcRect = srcRect;
            /** @type {Array<number>} */ this.dstRect = dstRect;
        };

        /** @const {Array<CopyRect>} */ var copyRects = [
            new CopyRect('basic', [10, 20, 65, 100], [45, 5, 100, 85]),
            new CopyRect('scale', [10, 20, 65, 100], [25, 30, 125, 94]),
            new CopyRect('out_of_bounds', [-10, -15, 100, 63], [50, 30, 136, 144])
        ];

        /** @const {Array<CopyRect>} */ var filterConsistencyRects = [

            new CopyRect('mag', [20, 10, 74, 88], [10, 10, 91, 101]),
            new CopyRect('min', [10, 20, 78, 100], [20, 20, 71, 80]),
            new CopyRect('out_of_bounds_mag', [21, 10, 73, 82], [11, 43, 141, 151]),
            new CopyRect('out_of_bounds_min', [11, 21, 77, 97], [80, 82, 135, 139])
        ];

        /** @constructor
         * @param {?string} name
         * @param {Array<number>} srcSwizzle
         * @param {Array<number>} dstSwizzle
         */
        var Swizzle = function(name, srcSwizzle, dstSwizzle) {
            /** @const {?string} */ this.name = name;
            /** @type {Array<number>} */ this.srcSwizzle = srcSwizzle;
            /** @type {Array<number>} */ this.dstSwizzle = dstSwizzle;
        };

        /** @const {Array<Swizzle>} */ var swizzles = [
            new Swizzle(null, [0, 1, 2, 3], [0, 1, 2, 3]),
            new Swizzle('reverse_src_x', [2, 1, 0, 3], [0, 1, 2, 3]),
            new Swizzle('reverse_src_y', [0, 3, 2, 1], [0, 1, 2, 3]),
            new Swizzle('reverse_dst_x', [0, 1, 2, 3], [2, 1, 0, 3]),
            new Swizzle('reverse_dst_y', [0, 1, 2, 3], [0, 3, 2, 1]),
            new Swizzle('reverse_src_dst_x', [2, 1, 0, 3], [2, 1, 0, 3]),
            new Swizzle('reverse_src_dst_y', [0, 3, 2, 1], [0, 3, 2, 1])
        ];

        /** @const {Array<number>} */ var srcSize = [127, 119];
        /** @const {Array<number>} */ var dstSize = [132, 128];

        // Blit rectangle tests.
        for (var rectNdx = 0; rectNdx < copyRects.length; rectNdx++) {
            /** @type {tcuTestCase.DeqpTest} */ var rectGroup = tcuTestCase.newTest('rect', 'Blit rectangle tests');
            this.addChild(rectGroup);

            for (var swzNdx = 0; swzNdx < swizzles.length; swzNdx++) {
                /** @type {string} */ var name = copyRects[rectNdx].name + (swizzles[swzNdx].name ? ('_' + swizzles[swzNdx].name) : '');
                /** @type {Array<number>} */ var srcSwz = swizzles[swzNdx].srcSwizzle;
                /** @type {Array<number>} */ var dstSwz = swizzles[swzNdx].dstSwizzle;
                /** @type {Array<number>} */ var srcRect = deMath.swizzle(copyRects[rectNdx].srcRect, srcSwz);
                /** @type {Array<number>} */ var dstRect = deMath.swizzle(copyRects[rectNdx].dstRect, dstSwz);

                rectGroup.addChild(new es3fFramebufferBlitTests.BlitRectCase((name + '_nearest'), '', gl.NEAREST, srcSize, srcRect, dstSize, dstRect));
                rectGroup.addChild(new es3fFramebufferBlitTests.BlitRectCase((name + '_linear'), '', gl.LINEAR, srcSize, srcRect, dstSize, dstRect));
            }
        }

        // Nearest filter tests
        for (var rectNdx = 0; rectNdx < filterConsistencyRects.length; rectNdx++) {
            /** @type {tcuTestCase.DeqpTest} */ var rectGroup = tcuTestCase.newTest('rect', 'Blit rectangle tests');
            this.addChild(rectGroup);
            for (var swzNdx = 0; swzNdx < swizzles.length; swzNdx++) {
                var name = 'nearest_consistency_' + filterConsistencyRects[rectNdx].name + (swizzles[swzNdx].name ? ('_' + swizzles[swzNdx].name) : '');
                var srcSwz = swizzles[swzNdx].srcSwizzle;
                var dstSwz = swizzles[swzNdx].dstSwizzle;
                var srcRect = deMath.swizzle(filterConsistencyRects[rectNdx].srcRect, srcSwz);
                var dstRect = deMath.swizzle(filterConsistencyRects[rectNdx].dstRect, dstSwz);

                rectGroup.addChild(new es3fFramebufferBlitTests.BlitNearestFilterConsistencyCase(name, 'Test consistency of the nearest filter', srcSize, srcRect, dstSize, dstRect));
            }
        }

        // .conversion
        for (var srcFmtNdx = 0; srcFmtNdx < colorFormats.length; srcFmtNdx++) {
            /** @type {tcuTestCase.DeqpTest} */ var conversionGroup = tcuTestCase.newTest('conversion', 'Color conversion tests');
            this.addChild(conversionGroup);
            for (var dstFmtNdx = 0; dstFmtNdx < colorFormats.length; dstFmtNdx++) {
                /** @type {number} */ var srcFormat = colorFormats[srcFmtNdx];
                /** @type {tcuTexture.TextureFormat} */ var srcTexFmt = gluTextureUtil.mapGLInternalFormat(srcFormat);
                /** @type {tcuTexture.TextureChannelClass} */ var srcType = tcuTexture.getTextureChannelClass(srcTexFmt.type);
                /** @type {number} */ var dstFormat = colorFormats[dstFmtNdx];
                /** @type {tcuTexture.TextureFormat} */ var dstTexFmt = gluTextureUtil.mapGLInternalFormat(dstFormat);
                /** @type {tcuTexture.TextureChannelClass} */ var dstType = tcuTexture.getTextureChannelClass(dstTexFmt.type);

                if (((srcType == tcuTexture.TextureChannelClass.FLOATING_POINT || srcType == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT) !=
                     (dstType == tcuTexture.TextureChannelClass.FLOATING_POINT || dstType == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT)) ||
                    ((srcType == tcuTexture.TextureChannelClass.SIGNED_INTEGER) != (dstType == tcuTexture.TextureChannelClass.SIGNED_INTEGER)) ||
                    ((srcType == tcuTexture.TextureChannelClass.UNSIGNED_INTEGER) != (dstType == tcuTexture.TextureChannelClass.UNSIGNED_INTEGER)))
                    continue; // Conversion not supported.

                var name = es3fFboTestUtil.getFormatName(srcFormat) + '_to_' + es3fFboTestUtil.getFormatName(dstFormat);

                conversionGroup.addChild(new es3fFramebufferBlitTests.BlitColorConversionCase(name, '', srcFormat, dstFormat, [127, 113]));
            }
        }

        // .depth_stencil
        /** @type {tcuTestCase.DeqpTest} */ var depthStencilGroup = tcuTestCase.newTest('depth_stencil', 'Depth and stencil blits');
        this.addChild(depthStencilGroup);

        for (var fmtNdx = 0; fmtNdx < depthStencilFormats.length; fmtNdx++) {
            /** @type {number} */ var format = depthStencilFormats[fmtNdx];
            /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(format);
            /** @type {string} */ var fmtName = es3fFboTestUtil.getFormatName(format);
            /** @type {boolean} */ var depth = texFmt.order == tcuTexture.ChannelOrder.D || texFmt.order == tcuTexture.ChannelOrder.DS;
            /** @type {boolean} */ var stencil = texFmt.order == tcuTexture.ChannelOrder.S || texFmt.order == tcuTexture.ChannelOrder.DS;
            /** @type {number} */ var buffers = (depth ? gl.DEPTH_BUFFER_BIT : 0) | (stencil ? gl.STENCIL_BUFFER_BIT : 0);

            depthStencilGroup.addChild(new es3fFramebufferBlitTests.BlitDepthStencilCase((fmtName + '_basic'), '', format, buffers, [128, 128], [0, 0, 128, 128], buffers, [128, 128], [0, 0, 128, 128], buffers));
            depthStencilGroup.addChild(new es3fFramebufferBlitTests.BlitDepthStencilCase((fmtName + '_scale'), '', format, buffers, [127, 119], [10, 30, 100, 70], buffers, [111, 130], [20, 5, 80, 130], buffers));

            if (depth && stencil) {
                depthStencilGroup.addChild(new es3fFramebufferBlitTests.BlitDepthStencilCase((fmtName + '_depth_only'), '', format, buffers, [128, 128], [0, 0, 128, 128], buffers, [128, 128], [0, 0, 128, 128], gl.DEPTH_BUFFER_BIT));
                depthStencilGroup.addChild(new es3fFramebufferBlitTests.BlitDepthStencilCase((fmtName + '_stencil_only'), '', format, buffers, [128, 128], [0, 0, 128, 128], buffers, [128, 128], [0, 0, 128, 128], gl.STENCIL_BUFFER_BIT));
            }
        }

        // .default_framebuffer
        /**
         * @constructor
         * @param {string} name
         * @param {es3fFramebufferBlitTests.BlitArea} area
         */
        var Area = function(name, area) {
            /** @type {string} name */ this.name = name;
            /** @type {es3fFramebufferBlitTests.BlitArea} area */ this.area = area;
        };

        /** @type {Array<Area>} */ var areas = [
            new Area('scale', es3fFramebufferBlitTests.BlitArea.AREA_SCALE),
            new Area('out_of_bounds', es3fFramebufferBlitTests.BlitArea.AREA_OUT_OF_BOUNDS)
        ];

        var numDefaultFbSubGroups = 7;
        /** @type {Array<tcuTestCase.DeqpTest>} */ var defaultFbGroup = [];
        for (var ii = 0; ii < numDefaultFbSubGroups; ++ii) {
            defaultFbGroup[ii] = tcuTestCase.newTest('default_framebuffer', 'Blits with default framebuffer');
            this.addChild(defaultFbGroup[ii]);
        }
        for (var fmtNdx = 0; fmtNdx < colorFormats.length; fmtNdx++) {
            var format = colorFormats[fmtNdx];
            var texFmt = gluTextureUtil.mapGLInternalFormat(format);
            var fmtClass = tcuTexture.getTextureChannelClass(texFmt.type);
            var filter = gluTextureUtil.isGLInternalColorFormatFilterable(format) ? gl.LINEAR : gl.NEAREST;
            var filterable = gluTextureUtil.isGLInternalColorFormatFilterable(format);

            if (fmtClass != tcuTexture.TextureChannelClass.FLOATING_POINT &&
                fmtClass != tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT &&
                fmtClass != tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT)
                continue; // Conversion not supported.

            defaultFbGroup[fmtNdx % numDefaultFbSubGroups].addChild(new es3fFramebufferBlitTests.BlitDefaultFramebufferCase(es3fFboTestUtil.getFormatName(format), '', format, filter));

            for (var areaNdx = 0; areaNdx < areas.length; areaNdx++) {
                var name = areas[areaNdx].name;
                var addLinear = filterable;
                var addNearest = !addLinear || (areas[areaNdx].area != es3fFramebufferBlitTests.BlitArea.AREA_OUT_OF_BOUNDS); // No need to check out-of-bounds with different filtering

                if (addNearest) {

                    defaultFbGroup[fmtNdx % numDefaultFbSubGroups].addChild(new es3fFramebufferBlitTests.DefaultFramebufferBlitCase((es3fFboTestUtil.getFormatName(format) + '_nearest_' + name + '_blit_from_default'), '', format, gl.NEAREST, es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET, areas[areaNdx].area));
                    defaultFbGroup[fmtNdx % numDefaultFbSubGroups].addChild(new es3fFramebufferBlitTests.DefaultFramebufferBlitCase((es3fFboTestUtil.getFormatName(format) + '_nearest_' + name + '_blit_to_default'), '', format, gl.NEAREST, es3fFramebufferBlitTests.BlitDirection.BLIT_TO_DEFAULT_FROM_TARGET, areas[areaNdx].area));
                }

                if (addLinear) {
                    defaultFbGroup[fmtNdx % numDefaultFbSubGroups].addChild(new es3fFramebufferBlitTests.DefaultFramebufferBlitCase((es3fFboTestUtil.getFormatName(format) + '_linear_' + name + '_blit_from_default'), '', format, gl.LINEAR, es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET, areas[areaNdx].area));
                    defaultFbGroup[fmtNdx % numDefaultFbSubGroups].addChild(new es3fFramebufferBlitTests.DefaultFramebufferBlitCase((es3fFboTestUtil.getFormatName(format) + '_linear_' + name + '_blit_to_default'), '', format, gl.LINEAR, es3fFramebufferBlitTests.BlitDirection.BLIT_TO_DEFAULT_FROM_TARGET, areas[areaNdx].area));
                }
            }
        }
    };

    /**
     * @param {?tcuTexture.ChannelOrder} order
     * @return {Array<boolean>}
     */
    es3fFramebufferBlitTests.getChannelMask = function(order) {
        switch (order) {
            case tcuTexture.ChannelOrder.R: return [true, false, false, false];
            case tcuTexture.ChannelOrder.RG: return [true, true, false, false];
            case tcuTexture.ChannelOrder.RGB: return [true, true, true, false];
            case tcuTexture.ChannelOrder.RGBA: return [true, true, true, true];
            case tcuTexture.ChannelOrder.sRGB: return [true, true, true, false];
            case tcuTexture.ChannelOrder.sRGBA: return [true, true, true, true];
            default:
                DE_ASSERT(false);
                return [false, false, false, false];
        }
    };

    /**
     * es3fFramebufferBlitTests.BlitColorConversionCase class, inherits from FboTestCase
     * @constructor
     * @extends {es3fFboTestCase.FboTestCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} srcFormat
     * @param {number} dstFormat
     * @param {Array<number>} size
     */
    es3fFramebufferBlitTests.BlitColorConversionCase = function(name, desc, srcFormat, dstFormat, size) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @type {number} */ this.m_srcFormat = srcFormat;
        /** @type {number} */ this.m_dstFormat = dstFormat;
        /** @type {Array<number>} */ this.m_size = size;
    };

    es3fFramebufferBlitTests.BlitColorConversionCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFramebufferBlitTests.BlitColorConversionCase.prototype.constructor = es3fFramebufferBlitTests.BlitColorConversionCase;

    es3fFramebufferBlitTests.BlitColorConversionCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_srcFormat);
        this.checkFormatSupport(this.m_dstFormat);
        return true; // No exception thrown
    };

    /**
     * @param {tcuSurface.Surface} dst
     */
    es3fFramebufferBlitTests.BlitColorConversionCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        /** @type {tcuTexture.TextureFormat} */ var srcFormat = gluTextureUtil.mapGLInternalFormat(this.m_srcFormat);
        /** @type {tcuTexture.TextureFormat} */ var dstFormat = gluTextureUtil.mapGLInternalFormat(this.m_dstFormat);

        /** @type {gluShaderUtil.DataType} */ var srcOutputType = es3fFboTestUtil.getFragmentOutputType(srcFormat);
        /** @type {gluShaderUtil.DataType} */ var dstOutputType = es3fFboTestUtil.getFragmentOutputType(dstFormat);

        // Compute ranges \note Doesn't handle case where src or dest is not subset of the another!
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var srcFmtRangeInfo = tcuTextureUtil.getTextureFormatInfo(srcFormat);
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var dstFmtRangeInfo = tcuTextureUtil.getTextureFormatInfo(dstFormat);

        /** @type {Array<boolean>} */ var copyMask = deMath.logicalAndBool(es3fFramebufferBlitTests.getChannelMask(srcFormat.order), es3fFramebufferBlitTests.getChannelMask(dstFormat.order));
        /** @type {Array<boolean>} */ var srcIsGreater = deMath.greaterThan(deMath.subtract(srcFmtRangeInfo.valueMax, srcFmtRangeInfo.valueMin), deMath.subtract(dstFmtRangeInfo.valueMax, dstFmtRangeInfo.valueMin));

        /** @type {tcuTextureUtil.TextureFormatInfo} */ var srcRangeInfo = new tcuTextureUtil.TextureFormatInfo(
                                                     tcuTextureUtil.select(dstFmtRangeInfo.valueMin, srcFmtRangeInfo.valueMin, deMath.logicalAndBool(copyMask, srcIsGreater)),
                                                     tcuTextureUtil.select(dstFmtRangeInfo.valueMax, srcFmtRangeInfo.valueMax, deMath.logicalAndBool(copyMask, srcIsGreater)),
                                                     tcuTextureUtil.select(dstFmtRangeInfo.lookupScale, srcFmtRangeInfo.lookupScale, deMath.logicalAndBool(copyMask, srcIsGreater)),
                                                     tcuTextureUtil.select(dstFmtRangeInfo.lookupBias, srcFmtRangeInfo.lookupBias, deMath.logicalAndBool(copyMask, srcIsGreater)));
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var dstRangeInfo = new tcuTextureUtil.TextureFormatInfo(
                                                     tcuTextureUtil.select(dstFmtRangeInfo.valueMin, srcFmtRangeInfo.valueMin, deMath.logicalOrBool(deMath.logicalNotBool(copyMask), srcIsGreater)),
                                                     tcuTextureUtil.select(dstFmtRangeInfo.valueMax, srcFmtRangeInfo.valueMax, deMath.logicalOrBool(deMath.logicalNotBool(copyMask), srcIsGreater)),
                                                     tcuTextureUtil.select(dstFmtRangeInfo.lookupScale, srcFmtRangeInfo.lookupScale, deMath.logicalOrBool(deMath.logicalNotBool(copyMask), srcIsGreater)),
                                                     tcuTextureUtil.select(dstFmtRangeInfo.lookupBias, srcFmtRangeInfo.lookupBias, deMath.logicalOrBool(deMath.logicalNotBool(copyMask), srcIsGreater)));

        // Shaders.
        /** @type {es3fFboTestUtil.GradientShader} */
        var gradientToSrcShader = new es3fFboTestUtil.GradientShader(srcOutputType);
        /** @type {es3fFboTestUtil.GradientShader} */
        var gradientToDstShader = new es3fFboTestUtil.GradientShader(dstOutputType);

        var gradShaderSrcID = ctx.createProgram(gradientToSrcShader);
        var gradShaderDstID = ctx.createProgram(gradientToDstShader);

        var srcFbo;
        var dstFbo;
        var srcRbo;
        var dstRbo;

        // Create framebuffers.
        // Source framebuffers
        srcFbo = ctx.createFramebuffer();
        srcRbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, srcRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_srcFormat, this.m_size[0], this.m_size[1]);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, srcFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, srcRbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        // Destination framebuffers
        dstFbo = ctx.createFramebuffer();
        dstRbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, dstRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_dstFormat, this.m_size[0], this.m_size[1]);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, dstFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, dstRbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_size[0], this.m_size[1]);

        // Render gradients.
        for (var ndx = 0; ndx < 2; ndx++) {
            ctx.bindFramebuffer(gl.FRAMEBUFFER, ndx ? dstFbo : srcFbo);
            if (ndx) {
                gradientToDstShader.setGradient(ctx, gradShaderDstID, dstRangeInfo.valueMax, dstRangeInfo.valueMin);
                rrUtil.drawQuad(ctx, gradShaderDstID, [-1, -1, 0], [1, 1, 0]);
            } else {
                gradientToSrcShader.setGradient(ctx, gradShaderSrcID, srcRangeInfo.valueMin, srcRangeInfo.valueMax);
                rrUtil.drawQuad(ctx, gradShaderSrcID, [-1, -1, 0], [1, 1, 0]);
            }
        }

        // Execute copy.
        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, srcFbo);
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, dstFbo);
        ctx.blitFramebuffer(0, 0, this.m_size[0], this.m_size[1], 0, 0, this.m_size[0], this.m_size[1], gl.COLOR_BUFFER_BIT, gl.NEAREST);
        this.checkError();

        // Read results.
        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, dstFbo);
        this.readPixelsUsingFormat(dst, 0, 0, this.m_size[0], this.m_size[1], dstFormat, dstRangeInfo.lookupScale, dstRangeInfo.lookupBias);

    };

    /**
     * @param {tcuSurface.Surface} reference
     * @param {tcuSurface.Surface} result
     */
    es3fFramebufferBlitTests.BlitColorConversionCase.prototype.compare = function(reference, result) {
        /** @const {tcuTexture.TextureFormat} */ var srcFormat = gluTextureUtil.mapGLInternalFormat(this.m_srcFormat);
        /** @const {tcuTexture.TextureFormat} */ var dstFormat = gluTextureUtil.mapGLInternalFormat(this.m_dstFormat);
        /** @const {boolean} */ var srcIsSRGB = (srcFormat.order == tcuTexture.ChannelOrder.sRGBA);
        /** @const {boolean} */ var dstIsSRGB = (dstFormat.order == tcuTexture.ChannelOrder.sRGBA);
        /** @type {tcuRGBA.RGBA} */ var threshold = new tcuRGBA.RGBA();

        if (dstIsSRGB)
            threshold = es3fFboTestUtil.getToSRGBConversionThreshold(srcFormat, dstFormat);
        else {
            /** @type {tcuRGBA.RGBA} */ var srcMaxDiff = es3fFboTestUtil.getThresholdFromTextureFormat(srcFormat);
            /** @type {tcuRGBA.RGBA} */ var dstMaxDiff = es3fFboTestUtil.getThresholdFromTextureFormat(dstFormat);
            if (srcIsSRGB)
                srcMaxDiff = tcuRGBA.multiply(srcMaxDiff, 2);

            threshold = tcuRGBA.max(srcMaxDiff, dstMaxDiff);
        }

        // m_testCtx.getLog() << tcu::TestLog::Message << 'threshold = ' << threshold << tcu::TestLog::EndMessage;
        return tcuImageCompare.pixelThresholdCompare('Result', 'Image comparison result', reference, result, threshold.toIVec());
    };

    /**
     * @constructor
     * @extends {es3fFboTestCase.FboTestCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format deUint32
     * @param {number} srcBuffers deUint32
     * @param {Array<number>} srcSize IVec2
     * @param {Array<number>} srcRect IVec4
     * @param {number} dstBuffers deUint32
     * @param {Array<number>} dstSize IVec2
     * @param {Array<number>} dstRect IVec4
     * @param {number} copyBuffers deUint32
     */
    es3fFramebufferBlitTests.BlitDepthStencilCase = function(name, desc, format, srcBuffers, srcSize, srcRect, dstBuffers, dstSize, dstRect, copyBuffers) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @type {number} */ this.m_format = format;
        /** @type {number} */ this.m_srcBuffers = srcBuffers;
        /** @type {Array<number>} */ this.m_srcSize = srcSize;
        /** @type {Array<number>} */ this.m_srcRect = srcRect;
        /** @type {number} */ this.m_dstBuffers = dstBuffers;
        /** @type {Array<number>} */ this.m_dstSize = dstSize;
        /** @type {Array<number>} */ this.m_dstRect = dstRect;
        /** @type {number} */ this.m_copyBuffers = copyBuffers;
    };

    es3fFramebufferBlitTests.BlitDepthStencilCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFramebufferBlitTests.BlitDepthStencilCase.prototype.constructor = es3fFramebufferBlitTests.BlitDepthStencilCase;

    /**
     * @protected
     */
    es3fFramebufferBlitTests.BlitDepthStencilCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

    /**
     * @protected
     * @param {tcuSurface.Surface} dst
     */
    es3fFramebufferBlitTests.BlitDepthStencilCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        /** @const {number} */ var colorFormat = gl.RGBA8;
        var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D] ,
            gluShaderUtil.DataType.FLOAT_VEC4);
        var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);

        var flatShaderID = ctx.createProgram(flatShader);
        var texShaderID = ctx.createProgram(texShader);
        var gradShaderID = ctx.createProgram(gradShader);

        var srcFbo;
        var dstFbo;
        var srcColorRbo;
        var dstColorRbo;
        var srcDepthStencilRbo;
        var dstDepthStencilRbo;

        // setup shaders
        gradShader.setGradient(ctx, gradShaderID, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        texShader.setUniforms(ctx, texShaderID);

        // Create framebuffers
        // Source framebuffers
        srcFbo = ctx.createFramebuffer();
        srcColorRbo = ctx.createRenderbuffer();
        srcDepthStencilRbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, srcColorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.m_srcSize[0], this.m_srcSize[1]);

        ctx.bindRenderbuffer(gl.RENDERBUFFER, srcDepthStencilRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_format, this.m_srcSize[0], this.m_srcSize[1]);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, srcFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, srcColorRbo);

        if (this.m_srcBuffers & gl.DEPTH_BUFFER_BIT)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, srcDepthStencilRbo);
        if (this.m_srcBuffers & gl.STENCIL_BUFFER_BIT)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, srcDepthStencilRbo);

        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        // Clear depth to 1 and stencil to 0.
        ctx.clearBufferfi(gl.DEPTH_STENCIL, 0, 1.0, 0);

        // Destination framebuffers
        dstFbo = ctx.createFramebuffer();
        dstColorRbo = ctx.createRenderbuffer();
        dstDepthStencilRbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, dstColorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.m_dstSize[0], this.m_dstSize[1]);

        ctx.bindRenderbuffer(gl.RENDERBUFFER, dstDepthStencilRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_format, this.m_dstSize[0], this.m_dstSize[1]);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, dstFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, dstColorRbo);

        if (this.m_dstBuffers & gl.DEPTH_BUFFER_BIT)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, dstDepthStencilRbo);
        if (this.m_dstBuffers & gl.STENCIL_BUFFER_BIT)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, dstDepthStencilRbo);

        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        // Clear depth to 1 and stencil to 0.
        ctx.clearBufferfi(gl.DEPTH_STENCIL, 0, 1.0, 0);

        // Fill source with gradient, depth = [-1..1], stencil = 7
        ctx.bindFramebuffer(gl.FRAMEBUFFER, srcFbo);
        ctx.viewport(0, 0, this.m_srcSize[0], this.m_srcSize[1]);
        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
        ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
        ctx.stencilFunc(gl.ALWAYS, 7, 0xff);

        rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, -1], [1, 1, 1]);

        // Fill destination with grid pattern, depth = 0 and stencil = 1
        /** @const {number} */ var format = gl.RGBA;
        /** @const {number} */ var dataType = gl.UNSIGNED_BYTE;
        /** @const {number} */ var texW = this.m_srcSize[0];
        /** @const {number} */ var texH = this.m_srcSize[1];
        /** @type {WebGLTexture|sglrReferenceContext.TextureContainer} */ var gridTex = null;
        /** @type {tcuTexture.TextureLevel} */ var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

        tcuTextureUtil.fillWithGrid(data.getAccess(), 8, [0.2, 0.7, 0.1, 1.0], [0.7, 0.1, 0.5, 0.8]);

        gridTex = ctx.createTexture();
        ctx.bindTexture(gl.TEXTURE_2D, gridTex);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

        ctx.bindFramebuffer(gl.FRAMEBUFFER, dstFbo);
        ctx.viewport(0, 0, this.m_dstSize[0], this.m_dstSize[1]);
        ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);

        // Perform copy.
        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, srcFbo);
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, dstFbo);
        ctx.blitFramebuffer(this.m_srcRect[0], this.m_srcRect[1], this.m_srcRect[2], this.m_srcRect[3], this.m_dstRect[0], this.m_dstRect[1], this.m_dstRect[2], this.m_dstRect[3], this.m_copyBuffers, gl.NEAREST);

        // Render blue color where depth < 0, decrement on depth failure.
        ctx.bindFramebuffer(gl.FRAMEBUFFER, dstFbo);
        ctx.viewport(0, 0, this.m_dstSize[0], this.m_dstSize[1]);
        ctx.stencilOp(gl.KEEP, gl.DECR, gl.KEEP);
        ctx.stencilFunc(gl.ALWAYS, 0, 0xff);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [0.0, 0.0, 1.0, 1.0]);

        rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

        if (this.m_dstBuffers & gl.STENCIL_BUFFER_BIT) {
            // Render green color where stencil == 6.
            ctx.disable(gl.DEPTH_TEST);
            ctx.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP);
            ctx.stencilFunc(gl.EQUAL, 6, 0xff);

            flatShader.setColor(this.getCurrentContext(), flatShaderID, [0.0, 1.0, 0.0, 1.0]);

            rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

        }
        this.readPixelsUsingFormat(dst, 0, 0, this.m_dstSize[0], this.m_dstSize[1], gluTextureUtil.mapGLInternalFormat(colorFormat), [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]);

    };

    /**
     * @constructor
     * @extends {es3fFboTestCase.FboTestCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} filter
     */
    es3fFramebufferBlitTests.BlitDefaultFramebufferCase = function(name, desc, format, filter) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @const {number} */ this.m_format = format;
        /** @const {number} */ this.m_filter = filter;
    };

    es3fFramebufferBlitTests.BlitDefaultFramebufferCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFramebufferBlitTests.BlitDefaultFramebufferCase.prototype.constructor = es3fFramebufferBlitTests.BlitDefaultFramebufferCase;

    /**
     * @protected
     */
    es3fFramebufferBlitTests.BlitDefaultFramebufferCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

    /**
     * @protected
     * @param {tcuSurface.Surface} dst
     */
    es3fFramebufferBlitTests.BlitDefaultFramebufferCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        /** @type {tcuTexture.TextureFormat} */ var colorFormat = gluTextureUtil.mapGLInternalFormat(this.m_format);
        /** @type {gluTextureUtil.TransferFormat} */ var transferFmt = gluTextureUtil.getTransferFormat(colorFormat);

        /** @type {es3fFboTestUtil.GradientShader} */ var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
        /** @type {es3fFboTestUtil.Texture2DShader} */ var texShader = new es3fFboTestUtil.Texture2DShader([gluTextureUtil.getSampler2DType(colorFormat)], gluShaderUtil.DataType.FLOAT_VEC4);

        var gradShaderID = ctx.createProgram(gradShader);
        var texShaderID = ctx.createProgram(texShader);
        var fbo;
        var tex;
        /** @const {number} */ var texW = 128;
        /** @const {number} */ var texH = 128;

        // Setup shaders
        gradShader.setGradient(ctx, gradShaderID, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        texShader.setUniforms(ctx, texShaderID);

        // FBO
        fbo = ctx.createFramebuffer();
        tex = ctx.createTexture();

        ctx.bindTexture(gl.TEXTURE_2D, tex);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, this.m_filter);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, this.m_filter);
        ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_format, texW, texH, 0, transferFmt.format, transferFmt.dataType, null);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        // Render gradient to screen.
        ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

        rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, 0], [1, 1, 0]);

        // Blit gradient from screen to fbo.
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, fbo);
        ctx.blitFramebuffer(0, 0, ctx.getWidth(), ctx.getHeight(), 0, 0, texW, texH, gl.COLOR_BUFFER_BIT, this.m_filter);

        // Fill left half of viewport with quad that uses texture.
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, null);
        ctx.clearBufferfv(gl.COLOR, 0, [1.0, 0.0, 0.0, 1.0]);

        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);

        // Blit fbo to right half.
        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, fbo);
        ctx.blitFramebuffer(0, 0, texW, texH, Math.floor(ctx.getWidth() / 2), 0, ctx.getWidth(), ctx.getHeight(), gl.COLOR_BUFFER_BIT, this.m_filter);

        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, null);
        this.readPixels(dst, 0, 0, ctx.getWidth(), ctx.getHeight());

    };

    /**
     * @protected
     * @param {tcuSurface.Surface} reference
     * @param {tcuSurface.Surface} result
     */
    es3fFramebufferBlitTests.BlitDefaultFramebufferCase.prototype.compare = function(reference, result) {
        /** @const {tcuRGBA.RGBA} */
        var threshold = tcuRGBA.max(es3fFboTestUtil.getFormatThreshold(this.m_format), tcuRGBA.newRGBAComponents(12, 12, 12, 12));

        //m_testCtx.getLog() << TestLog::Message << 'Comparing images, threshold: ' << threshold << TestLog::EndMessage;

        return tcuImageCompare.bilinearCompare('Result', 'Image comparison result', reference.getAccess(), result.getAccess(), threshold);
    };

    /** @enum */
    es3fFramebufferBlitTests.BlitDirection = {
        BLIT_DEFAULT_TO_TARGET: 0,
        BLIT_TO_DEFAULT_FROM_TARGET: 1
    };

    /** @enum */
    es3fFramebufferBlitTests.BlitArea = {
        AREA_SCALE: 0,
        AREA_OUT_OF_BOUNDS: 1
    };

    /**
     * @constructor
     * @extends {es3fFramebufferBlitTests.BlitDefaultFramebufferCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} filter
     * @param {es3fFramebufferBlitTests.BlitDirection} dir
     * @param {es3fFramebufferBlitTests.BlitArea} area
     */
    es3fFramebufferBlitTests.DefaultFramebufferBlitCase = function(name, desc, format, filter, dir, area) {
        es3fFramebufferBlitTests.BlitDefaultFramebufferCase.call(this, name, desc, format, filter);
        /** @const {es3fFramebufferBlitTests.BlitDirection} */ this.m_blitDir = dir;
        /** @const {es3fFramebufferBlitTests.BlitArea} */ this.m_blitArea = area;
        /** @type {Array<number>} */ this.m_srcRect = [-1, -1, -1, -1];
        /** @type {Array<number>} */ this.m_dstRect = [-1, -1, -1, -1];
        /** @type {Array<number>} */ this.m_interestingArea = [-1, -1, -1, -1];
    };

    es3fFramebufferBlitTests.DefaultFramebufferBlitCase.prototype = Object.create(es3fFramebufferBlitTests.BlitDefaultFramebufferCase.prototype);
    es3fFramebufferBlitTests.DefaultFramebufferBlitCase.prototype.constructor = es3fFramebufferBlitTests.DefaultFramebufferBlitCase;

    es3fFramebufferBlitTests.DefaultFramebufferBlitCase.prototype.init = function() {
        // requirements
        /** @const {number} */ var minViewportSize = 128;
        if (gl.drawingBufferWidth < minViewportSize ||
            gl.drawingBufferHeight < minViewportSize)
            throw new Error('Viewport size ' + minViewportSize + 'x' + minViewportSize + ' required');

        // prevent viewport randoming
        this.m_viewportWidth = gl.drawingBufferWidth;
        this.m_viewportHeight = gl.drawingBufferHeight;

        // set proper areas
        if (this.m_blitArea == es3fFramebufferBlitTests.BlitArea.AREA_SCALE) {
            this.m_srcRect = [10, 20, 65, 100];
            this.m_dstRect = [25, 30, 125, 94];
            this.m_interestingArea = [0, 0, 128, 128];
        } else if (this.m_blitArea == es3fFramebufferBlitTests.BlitArea.AREA_OUT_OF_BOUNDS) {
            /** @const {Array<number>} */
            var ubound = (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET) ?
                         ([128, 128]) :
                         ([gl.drawingBufferWidth, gl.drawingBufferHeight]);

            this.m_srcRect = [-10, -15, 100, 63];
            this.m_dstRect = deMath.add(deMath.swizzle(ubound, [0, 1, 0, 1]), [-75, -99, 8, 16]);
            this.m_interestingArea = [ubound[0] - 128, ubound[1] - 128, ubound[0], ubound[1]];
        }
    };

    /**
     * @param {tcuSurface.Surface} dst
     */
    es3fFramebufferBlitTests.DefaultFramebufferBlitCase.prototype.render = function(dst) {
        /** @type {es3fFboTestCase.Context} */
        var ctx = this.getCurrentContext();
        // TOOD: implement
        /** @type {tcuTexture.TextureFormat} */ var colorFormat = gluTextureUtil.mapGLInternalFormat(this.m_format);
        /** @type {gluTextureUtil.TransferFormat} */ var transferFmt = gluTextureUtil.getTransferFormat(colorFormat);
        /** @const {tcuTexture.TextureChannelClass} */
        var targetClass = (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET) ?
            (tcuTexture.getTextureChannelClass(colorFormat.type)) :
            (tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT);

        var fbo;
        var fboTex;
        /** @const {number} */ var fboTexW = 128;
        /** @const {number} */ var fboTexH = 128;
        /** @const {number} */ var sourceWidth = (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET) ? (ctx.getWidth()) : (fboTexW);
        /** @const {number} */ var sourceHeight = (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET) ? (ctx.getHeight()) : (fboTexH);
        /** @const {number} */ var gridRenderWidth = Math.min(256, sourceWidth);
        /** @const {number} */ var gridRenderHeight = Math.min(256, sourceHeight);

        var targetFbo;
        var sourceFbo;

        // FBO
        fbo = ctx.createFramebuffer();
        fboTex = ctx.createTexture();

        ctx.bindTexture(gl.TEXTURE_2D, fboTex);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, this.m_filter);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, this.m_filter);
        ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_format, fboTexW, fboTexH, 0, transferFmt.format, transferFmt.dataType, null);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, fboTex, 0);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        targetFbo = (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET) ? (fbo) : (null);
        sourceFbo = (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_DEFAULT_TO_TARGET) ? (null) : (fbo);

        // Render grid to source framebuffer
        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D],
            gluShaderUtil.DataType.FLOAT_VEC4);
        var texShaderID = this.getCurrentContext().createProgram(texShader);
        /** @const {number} */ var internalFormat = gl.RGBA8;
        /** @const {number} */ var format = gl.RGBA;
        /** @const {number} */ var dataType = gl.UNSIGNED_BYTE;
        /** @const {number} */ var gridTexW = 128;
        /** @const {number} */ var gridTexH = 128;
        /** @type {WebGLTexture|framework.opengl.simplereference.sglrReferenceContext.TextureContainer|null} */
        var gridTex = null;
        /** @type {tcuTexture.TextureLevel} */ var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), gridTexW, gridTexH, 1);

        tcuTextureUtil.fillWithGrid(data.getAccess(), 9, [0.9, 0.5, 0.1, 0.9], [0.2, 0.8, 0.2, 0.7]);

        gridTex = ctx.createTexture();
        ctx.bindTexture(gl.TEXTURE_2D, gridTex);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        ctx.texImage2D(gl.TEXTURE_2D, 0, internalFormat, gridTexW, gridTexH, 0, format, dataType, data.getAccess().getDataPtr());

        ctx.bindFramebuffer(gl.FRAMEBUFFER, sourceFbo);
        ctx.viewport(0, 0, gridRenderWidth, gridRenderHeight);
        ctx.clearBufferfv(gl.COLOR, 0, [1.0, 0.0, 0.0, 1.0]);

        texShader.setUniforms(this.getCurrentContext(), texShaderID);

        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);

        ctx.useProgram(null);

        // Blit source framebuffer to destination

        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, sourceFbo);
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, targetFbo);
        this.checkError();

        if (targetClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT ||
            targetClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT ||
            targetClass == tcuTexture.TextureChannelClass.FLOATING_POINT)
            ctx.clearBufferfv(gl.COLOR, 0, [1.0, 1.0, 0.0, 1.0]);
        else if (targetClass == tcuTexture.TextureChannelClass.SIGNED_INTEGER)
            ctx.clearBufferiv(gl.COLOR, 0, [0, 0, 0, 0]);
        else if (targetClass == tcuTexture.TextureChannelClass.UNSIGNED_INTEGER)
            ctx.clearBufferuiv(gl.COLOR, 0, [0, 0, 0, 0]);
        else
            DE_ASSERT(false);

        ctx.blitFramebuffer(this.m_srcRect[0], this.m_srcRect[1], this.m_srcRect[2], this.m_srcRect[3], this.m_dstRect[0], this.m_dstRect[1], this.m_dstRect[2], this.m_dstRect[3], gl.COLOR_BUFFER_BIT, this.m_filter);
        this.checkError();

        // Read target

        ctx.bindFramebuffer(gl.FRAMEBUFFER, targetFbo);

        if (this.m_blitDir == es3fFramebufferBlitTests.BlitDirection.BLIT_TO_DEFAULT_FROM_TARGET)
            this.readPixels(dst, this.m_interestingArea[0], this.m_interestingArea[1], this.m_interestingArea[2] - this.m_interestingArea[0], this.m_interestingArea[3] - this.m_interestingArea[1]);
        else
            this.readPixelsUsingFormat(dst, this.m_interestingArea[0], this.m_interestingArea[1], this.m_interestingArea[2] - this.m_interestingArea[0], this.m_interestingArea[3] - this.m_interestingArea[1], colorFormat, [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]);

        this.checkError();
    };

    es3fFramebufferBlitTests.run = function(context, range) {
        gl = context;
        //Set up root Test
        var state = tcuTestCase.runner;

        var test = new es3fFramebufferBlitTests.FramebufferBlitTests();
        var testName = test.fullName();
        var testDescription = test.getDescription() || '';

        state.testName = testName;
        state.setRoot(test);
        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            test.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            bufferedLogToConsole(err);
            testFailedOptions('Failed to es3fFramebufferBlitTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

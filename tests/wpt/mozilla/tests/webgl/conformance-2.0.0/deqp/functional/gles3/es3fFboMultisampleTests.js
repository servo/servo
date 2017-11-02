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
goog.provide('functional.gles3.es3fFboMultisampleTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestCase');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {

var es3fFboMultisampleTests = functional.gles3.es3fFboMultisampleTests;
var es3fFboTestCase = functional.gles3.es3fFboTestCase;
var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var tcuTestCase = framework.common.tcuTestCase;
var tcuSurface = framework.common.tcuSurface;
var tcuRGBA = framework.common.tcuRGBA;
var tcuImageCompare = framework.common.tcuImageCompare;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var deRandom = framework.delibs.debase.deRandom;
var deMath = framework.delibs.debase.deMath;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var rrUtil = framework.referencerenderer.rrUtil;

/** @type {WebGL2RenderingContext} */ var gl;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

    /**
     * @constructor
     * @extends {es3fFboTestCase.FboTestCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} colorFormat
     * @param {number} depthStencilFormat
     * @param {Array<number>} size
     * @param {number} numSamples
     */
    es3fFboMultisampleTests.BasicFboMultisampleCase = function(name, desc, colorFormat, depthStencilFormat, size, numSamples) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @type {number} */ this.m_colorFormat = colorFormat;
        /** @type {number} */ this.m_depthStencilFormat = depthStencilFormat;
        /** @type {Array<number>} */ this.m_size = size;
        /** @type {number} */ this.m_numSamples = numSamples;
    };

    es3fFboMultisampleTests.BasicFboMultisampleCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFboMultisampleTests.BasicFboMultisampleCase.prototype.constructor = es3fFboMultisampleTests.BasicFboMultisampleCase;

    es3fFboMultisampleTests.BasicFboMultisampleCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_colorFormat);
        if (!this.checkSampleCount(this.m_colorFormat, this.m_numSamples))
            return false;

        if (this.m_depthStencilFormat != gl.NONE) {
            this.checkFormatSupport(this.m_depthStencilFormat);
            if (!this.checkSampleCount(this.m_depthStencilFormat, this.m_numSamples))
                return false;
        }
        return true; // No exception thrown
    };

    /**
     * @param {tcuSurface.Surface} dst
     */
    es3fFboMultisampleTests.BasicFboMultisampleCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        /** @type {tcuTexture.TextureFormat} */ var colorFmt = gluTextureUtil.mapGLInternalFormat(this.m_colorFormat);
        /** @type {tcuTexture.TextureFormat} */ var depthStencilFmt = this.m_depthStencilFormat != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFormat) : new tcuTexture.TextureFormat(null, null);
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var colorFmtInfo = tcuTextureUtil.getTextureFormatInfo(colorFmt);
        /** @type {boolean} */ var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
        /** @type {boolean} */ var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
        /** @type {es3fFboTestUtil.GradientShader} */ var gradShader = new es3fFboTestUtil.GradientShader(es3fFboTestUtil.getFragmentOutputType(colorFmt));
        /** @type {es3fFboTestUtil.FlatColorShader} */ var flatShader = new es3fFboTestUtil.FlatColorShader(es3fFboTestUtil.getFragmentOutputType(colorFmt));
        var gradShaderID = this.getCurrentContext().createProgram(gradShader);
        var flatShaderID = this.getCurrentContext().createProgram(flatShader);
        var msaaFbo = null;
        var resolveFbo = null;
        var msaaColorRbo = null;
        var resolveColorRbo = null;
        var msaaDepthStencilRbo = null;
        var resolveDepthStencilRbo = null;

        // Create framebuffers.
        msaaColorRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, msaaColorRbo);
        ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, this.m_colorFormat, this.m_size[0], this.m_size[1]);

        if (depth || stencil) {
            msaaDepthStencilRbo = ctx.createRenderbuffer();
            ctx.bindRenderbuffer(gl.RENDERBUFFER, msaaDepthStencilRbo);
            ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, this.m_depthStencilFormat, this.m_size[0], this.m_size[1]);
        }

        msaaFbo = ctx.createFramebuffer();
        ctx.bindFramebuffer(gl.FRAMEBUFFER, msaaFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, msaaColorRbo);
        if (depth)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, msaaDepthStencilRbo);
        if (stencil)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, msaaDepthStencilRbo);

        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        resolveColorRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, resolveColorRbo);
        ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, 0, this.m_colorFormat, this.m_size[0], this.m_size[1]);

        if (depth || stencil) {
            resolveDepthStencilRbo = ctx.createRenderbuffer();
            ctx.bindRenderbuffer(gl.RENDERBUFFER, resolveDepthStencilRbo);
            ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, 0, this.m_depthStencilFormat, this.m_size[0], this.m_size[1]);
        }

        resolveFbo = ctx.createFramebuffer();
        ctx.bindFramebuffer(gl.FRAMEBUFFER, resolveFbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, resolveColorRbo);
        if (depth)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, resolveDepthStencilRbo);
        if (stencil)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, resolveDepthStencilRbo);

        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, msaaFbo);
        ctx.viewport(0, 0, this.m_size[0], this.m_size[1]);

        // Clear depth and stencil buffers.
        ctx.clearBufferfi(gl.DEPTH_STENCIL, 0, 1.0, 0);

        // Fill MSAA fbo with gradient, depth = [-1..1]
        ctx.enable(gl.DEPTH_TEST);
        gradShader.setGradient(this.getCurrentContext(), gradShaderID, colorFmtInfo.valueMin, colorFmtInfo.valueMax);

        rrUtil.drawQuad(this.getCurrentContext(), gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        // Render random-colored quads.
        /** @const {number} */ var numQuads = 8;

        // The choice of random seed affects the correctness of the tests,
        // because there are some boundary conditions which aren't handled
        // correctly even in the C++ dEQP tests.
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(7);

        ctx.depthFunc(gl.ALWAYS);
        ctx.enable(gl.STENCIL_TEST);
        ctx.stencilFunc(gl.ALWAYS, 0, 0xff);
        ctx.stencilOp(gl.KEEP, gl.KEEP, gl.INCR);

        for (var ndx = 0; ndx < numQuads; ndx++) {
            /** @type {number} */ var r = rnd.getFloat();
            /** @type {number} */ var g = rnd.getFloat();
            /** @type {number} */ var b = rnd.getFloat();
            /** @type {number} */ var a = rnd.getFloat();
            /** @type {number} */ var x0 = rnd.getFloat(-1.0, 1.0);
            /** @type {number} */ var y0 = rnd.getFloat(-1.0, 1.0);
            /** @type {number} */ var z0 = rnd.getFloat(-1.0, 1.0);
            /** @type {number} */ var x1 = rnd.getFloat(-1.0, 1.0);
            /** @type {number} */ var y1 = rnd.getFloat(-1.0, 1.0);
            /** @type {number} */ var z1 = rnd.getFloat(-1.0, 1.0);

            flatShader.setColor(this.getCurrentContext(), flatShaderID, deMath.add(deMath.multiply([r, g, b, a], deMath.subtract(colorFmtInfo.valueMax, colorFmtInfo.valueMin)), colorFmtInfo.valueMin));
            rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [x0, y0, z0], [x1, y1, z1]);
        }

        ctx.disable(gl.DEPTH_TEST);
        ctx.disable(gl.STENCIL_TEST);
        this.checkError();

        // Resolve using glBlitFramebuffer().
        ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, resolveFbo);
        ctx.blitFramebuffer(0, 0, this.m_size[0], this.m_size[1], 0, 0, this.m_size[0], this.m_size[1], gl.COLOR_BUFFER_BIT | (depth ? gl.DEPTH_BUFFER_BIT : 0) | (stencil ? gl.STENCIL_BUFFER_BIT : 0), gl.NEAREST);

        ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, resolveFbo);

        /** @type {number} */ var numSteps;
        /** @type {number} */ var step;
        /** @type {number} */ var d;
        /** @type {number} */ var c;
        /** @type {number} */ var s;
        if (depth) {
            // Visualize depth.
            numSteps = 8;
            step = 2.0 / numSteps;
            ctx.enable(gl.DEPTH_TEST);
            ctx.depthFunc(gl.LESS);
            ctx.depthMask(false);
            ctx.colorMask(false, false, true, false);

            for (var ndx = 0; ndx < numSteps; ndx++) {
                d = -1.0 + step * ndx;
                c = ndx / (numSteps - 1);

                flatShader.setColor(this.getCurrentContext(), flatShaderID, deMath.add(deMath.multiply([0.0, 0.0, c, 1.0], deMath.subtract(colorFmtInfo.valueMax, colorFmtInfo.valueMin)), colorFmtInfo.valueMin));
                rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-1.0, -1.0, d], [1.0, 1.0, d]);
            }

            ctx.disable(gl.DEPTH_TEST);
        }

        if (stencil) {
            // Visualize stencil.
            numSteps = 4;
            step = 1;

            ctx.enable(gl.STENCIL_TEST);
            ctx.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP);
            ctx.colorMask(false, true, false, false);

            for (var ndx = 0; ndx < numSteps; ndx++) {
                s = step * ndx;
                c = ndx / (numSteps - 1);

                ctx.stencilFunc(gl.EQUAL, s, 0xff);

                flatShader.setColor(this.getCurrentContext(), flatShaderID, deMath.add(deMath.multiply([0.0, c, 0.0, 1.0], deMath.subtract(colorFmtInfo.valueMax, colorFmtInfo.valueMin)), colorFmtInfo.valueMin));
                rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
            }

            ctx.disable(gl.STENCIL_TEST);
        }

        this.readPixelsUsingFormat(dst, 0, 0, this.m_size[0], this.m_size[1], colorFmt, colorFmtInfo.lookupScale, colorFmtInfo.lookupBias);
    };

    /**
     * @param {tcuSurface.Surface} reference
     * @param {tcuSurface.Surface} result
     * @return {boolean}
     */
    es3fFboMultisampleTests.BasicFboMultisampleCase.prototype.colorCompare = function(reference, result) {
        /** @const {tcuRGBA.RGBA} */ var threshold = tcuRGBA.max(es3fFboTestUtil.getFormatThreshold(this.m_colorFormat), tcuRGBA.newRGBAComponents(12, 12, 12, 12));
        return tcuImageCompare.bilinearCompare('Result', 'Image comparison result', reference.getAccess(), result.getAccess(), threshold, tcuImageCompare.CompareLogMode.RESULT);
    };

    /**
     * @param {tcuSurface.Surface} reference
     * @param {tcuSurface.Surface} result
     * @return {boolean}
     */
    es3fFboMultisampleTests.BasicFboMultisampleCase.prototype.compare = function(reference, result) {
        if (this.m_depthStencilFormat != gl.NONE)
            return es3fFboTestCase.FboTestCase.prototype.compare(reference, result); // FboTestCase.compare
        else
            return this.colorCompare(reference, result);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fFboMultisampleTests.FboMultisampleTests = function() {
        tcuTestCase.DeqpTest.call(this, 'msaa', 'Multisample FBO tests');
    };

    es3fFboMultisampleTests.FboMultisampleTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fFboMultisampleTests.FboMultisampleTests.prototype.constructor = es3fFboMultisampleTests.FboMultisampleTests;

    es3fFboMultisampleTests.FboMultisampleTests.prototype.init = function() {
        /** @const {Array<number>} */ var colorFormats = [
            // RGBA formats
            gl.RGBA8,
            gl.SRGB8_ALPHA8,
            gl.RGB10_A2,
            gl.RGBA4,
            gl.RGB5_A1,

            // RGB formats
            gl.RGB8,
            gl.RGB565,

            // RG formats
            gl.RG8,

            // R formats
            gl.R8,

            // gl.EXT_color_buffer_float
            // Multi-sample floating-point color buffers can be optional supported, see https://www.khronos.org/registry/webgl/extensions/EXT_color_buffer_float/
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

        /** @const {Array<number>} */ var sampleCounts = [2, 4, 8];

        for (var sampleCntNdx in sampleCounts) {
            /** @type {number} */ var samples = sampleCounts[sampleCntNdx];
            /** @type {tcuTestCase.DeqpTest} */
            var sampleCountGroup = tcuTestCase.newTest(samples + '_samples', '');
            this.addChild(sampleCountGroup);

            // Color formats.
            for (var fmtNdx in colorFormats)
                sampleCountGroup.addChild(new es3fFboMultisampleTests.BasicFboMultisampleCase(es3fFboTestUtil.getFormatName(colorFormats[fmtNdx]), '', colorFormats[fmtNdx], gl.NONE, [119, 131], samples));

            // Depth/stencil formats.
            for (var fmtNdx in depthStencilFormats)
                sampleCountGroup.addChild(new es3fFboMultisampleTests.BasicFboMultisampleCase(es3fFboTestUtil.getFormatName(depthStencilFormats[fmtNdx]), '', gl.RGBA8, depthStencilFormats[fmtNdx], [119, 131], samples));
        }
    };

    es3fFboMultisampleTests.run = function(context, range) {
        gl = context;
        //Set up root Test
        var state = tcuTestCase.runner;

        var test = new es3fFboMultisampleTests.FboMultisampleTests();
        var testName = test.fullName();
        var testDescription = test.getDescription();

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
            testFailedOptions('Failed to es3fFboMultisampleTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

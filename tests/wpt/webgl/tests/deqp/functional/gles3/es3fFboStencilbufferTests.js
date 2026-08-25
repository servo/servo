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
goog.provide('functional.gles3.es3fFboStencilbufferTests');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestCase');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {

    var es3fFboStencilbufferTests = functional.gles3.es3fFboStencilbufferTests;
    var es3fFboTestCase = functional.gles3.es3fFboTestCase;
    var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var rrUtil = framework.referencerenderer.rrUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;

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
     * @param {number} format
     * @param {Array<number>} size
     * @param {boolean} useDepth
     */
    es3fFboStencilbufferTests.BasicFboStencilCase = function(name, desc, format, size, useDepth) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @type {number} */ this.m_format = format;
        /** @type {Array<number>} */ this.m_size = size;
        /** @type {boolean} */ this.m_useDepth = useDepth;
    };

    es3fFboStencilbufferTests.BasicFboStencilCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFboStencilbufferTests.BasicFboStencilCase.prototype.constructor = es3fFboStencilbufferTests.BasicFboStencilCase;

    es3fFboStencilbufferTests.BasicFboStencilCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

    /**
     * @param {tcuSurface.Surface} dst
     */
    es3fFboStencilbufferTests.BasicFboStencilCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        /** @const {number} */ var colorFormat = gl.RGBA8;

        /** @type {es3fFboTestUtil.GradientShader} */ var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
        /** @type {es3fFboTestUtil.FlatColorShader} */ var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
        var flatShaderID = this.getCurrentContext().createProgram(flatShader);
        var gradShaderID = this.getCurrentContext().createProgram(gradShader);

        var fbo = 0;
        var colorRbo = 0;
        var depthStencilRbo = 0;

        // Colorbuffer.
        colorRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.m_size[0], this.m_size[1]);

        // Stencil (and depth) buffer.
        depthStencilRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_format, this.m_size[0], this.m_size[1]);

        // Framebuffer.
        fbo = ctx.createFramebuffer();
        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);
        if (this.m_useDepth)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_size[0], this.m_size[1]);

        // Clear framebuffer.
        ctx.clearBufferfv(gl.COLOR, 0, [0.0, 0.0, 0.0, 0.0]);
        ctx.clearBufferfi(gl.DEPTH_STENCIL, 0, 1.0, 0);

        // Render intersecting quads - increment stencil on depth pass
        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
        ctx.stencilFunc(gl.ALWAYS, 0, 0xff);
        ctx.stencilOp(gl.KEEP, gl.KEEP, gl.INCR);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [1.0, 0.0, 0.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        gradShader.setGradient(this.getCurrentContext(), gradShaderID, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        ctx.disable(gl.DEPTH_TEST);

        // Draw quad with stencil test (stencil == 1 or 2 depending on depth) - decrement on stencil failure
        ctx.stencilFunc(gl.EQUAL, this.m_useDepth ? 2 : 1, 0xff);
        ctx.stencilOp(gl.DECR, gl.KEEP, gl.KEEP);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [0.0, 1.0, 0.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]);

        // Draw quad with stencil test where stencil > 1 or 2 depending on depth buffer
        ctx.stencilFunc(gl.GREATER, this.m_useDepth ? 1 : 2, 0xff);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [0.0, 0.0, 1.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        this.readPixelsUsingFormat(dst, 0, 0, this.m_size[0], this.m_size[1], gluTextureUtil.mapGLInternalFormat(colorFormat), [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]);
    };

    /**
     * @constructor
     * @extends {es3fFboTestCase.FboTestCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} attachDepth
     */
    es3fFboStencilbufferTests.DepthStencilAttachCase = function(name, desc, attachDepth, attachStencil) {
        es3fFboTestCase.FboTestCase.call(this, name, desc);
        /** @type {number} */ this.m_attachDepth = attachDepth;
        /** @type {number} */ this.m_attachStencil = attachStencil;
        DE_ASSERT(this.m_attachDepth == gl.DEPTH_ATTACHMENT || this.m_attachDepth == gl.DEPTH_STENCIL_ATTACHMENT || this.m_attachDepth == gl.NONE);
        DE_ASSERT(this.m_attachStencil == gl.STENCIL_ATTACHMENT || this.m_attachStencil == gl.NONE);
        DE_ASSERT(this.m_attachDepth != gl.DEPTH_STENCIL || this.m_attachStencil == gl.NONE);
    };

    es3fFboStencilbufferTests.DepthStencilAttachCase.prototype = Object.create(es3fFboTestCase.FboTestCase.prototype);
    es3fFboStencilbufferTests.DepthStencilAttachCase.prototype.constructor = es3fFboStencilbufferTests.DepthStencilAttachCase;

    /**
     * @param {tcuSurface.Surface} dst
     */
    es3fFboStencilbufferTests.DepthStencilAttachCase.prototype.render = function(dst) {

        var ctx = this.getCurrentContext();
        /** @const {number} */ var colorFormat = gl.RGBA8;
        /** @const {number} */ var depthStencilFormat = gl.DEPTH24_STENCIL8;
        /** @const {number} */ var width = 128;
        /** @const {number} */ var height = 128;
        /** @const {boolean} */ var hasDepth = this.m_attachDepth == gl.DEPTH_STENCIL || this.m_attachDepth == gl.DEPTH_ATTACHMENT;
        // /** @const {boolean} */ var hasStencil = this.m_attachDepth == gl.DEPTH_STENCIL || this.m_attachStencil == gl.DEPTH_STENCIL_ATTACHMENT); // commented out in original code

        /** @type {es3fFboTestUtil.GradientShader} */ var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
        /** @type {es3fFboTestUtil.FlatColorShader} */ var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
        var flatShaderID = this.getCurrentContext().createProgram(flatShader);
        var gradShaderID = this.getCurrentContext().createProgram(gradShader);

        var fbo = 0;
        var colorRbo = 0;
        var depthStencilRbo = 0;

        // Colorbuffer.
        colorRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, width, height);

        // Depth-stencil buffer.
        depthStencilRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, depthStencilFormat, width, height);

        // Framebuffer.
        fbo = ctx.createFramebuffer();
        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);

        if (this.m_attachDepth != gl.NONE)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, this.m_attachDepth, gl.RENDERBUFFER, depthStencilRbo);
        if (this.m_attachStencil != gl.NONE)
            ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, this.m_attachStencil, gl.RENDERBUFFER, depthStencilRbo);

        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, width, height);

        // Clear framebuffer.
        ctx.clearBufferfv(gl.COLOR, 0, [0.0, 0.0, 0.0, 0.0]);
        ctx.clearBufferfi(gl.DEPTH_STENCIL, 0, 1.0, 0);

        // Render intersecting quads - increment stencil on depth pass
        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
        ctx.stencilFunc(gl.ALWAYS, 0, 0xff);
        ctx.stencilOp(gl.KEEP, gl.KEEP, gl.INCR);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [1.0, 0.0, 0.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        gradShader.setGradient(this.getCurrentContext(), gradShaderID, [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        ctx.disable(gl.DEPTH_TEST);

        // Draw quad with stencil test (stencil == 1 or 2 depending on depth) - decrement on stencil failure
        ctx.stencilFunc(gl.EQUAL, hasDepth ? 2 : 1, 0xff);
        ctx.stencilOp(gl.DECR, gl.KEEP, gl.KEEP);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [0.0, 1.0, 0.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]);

        // Draw quad with stencil test where stencil > 1 or 2 depending on depth buffer
        ctx.stencilFunc(gl.GREATER, hasDepth ? 1 : 2, 0xff);

        flatShader.setColor(this.getCurrentContext(), flatShaderID, [0.0, 0.0, 1.0, 1.0]);

        rrUtil.drawQuad(this.getCurrentContext(), flatShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        this.readPixelsUsingFormat(dst, 0, 0, width, height, gluTextureUtil.mapGLInternalFormat(colorFormat), [1.0, 1.0, 1.0, 1.0], [0.0, 0.0, 0.0, 0.0]);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fFboStencilbufferTests.FboStencilTests = function() {
        tcuTestCase.DeqpTest.call(this, 'stencil', 'FBO Stencilbuffer tests');
    };

    es3fFboStencilbufferTests.FboStencilTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fFboStencilbufferTests.FboStencilTests.prototype.constructor = es3fFboStencilbufferTests.FboStencilTests;

    es3fFboStencilbufferTests.FboStencilTests.prototype.init = function() {
        /** @const {Array<number>} */ var stencilFormats = [
            gl.DEPTH32F_STENCIL8,
            gl.DEPTH24_STENCIL8,
            gl.STENCIL_INDEX8
        ];

        // .basic
        /** @type {tcuTestCase.DeqpTest} */
        var basicGroup = tcuTestCase.newTest('basic', 'Basic stencil tests');
        this.addChild(basicGroup);

        for (var fmtNdx = 0; fmtNdx < stencilFormats.length; fmtNdx++) {
            /** @type {number} */ var format = stencilFormats[fmtNdx];
            /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(format);

            basicGroup.addChild(new es3fFboStencilbufferTests.BasicFboStencilCase(es3fFboTestUtil.getFormatName(format), '', format, [111, 132], false));

            if (texFmt.order == tcuTexture.ChannelOrder.DS)
                basicGroup.addChild(new es3fFboStencilbufferTests.BasicFboStencilCase(es3fFboTestUtil.getFormatName(format) + '_depth', '', format, [111, 132], true));
        }

        // .attach
        /** @type {tcuTestCase.DeqpTest} */
        var attachGroup = tcuTestCase.newTest('attach', 'Attaching depth stencil');
        this.addChild(attachGroup);

        attachGroup.addChild(new es3fFboStencilbufferTests.DepthStencilAttachCase('depth_only', 'Only depth part of depth-stencil RBO attached', gl.DEPTH_ATTACHMENT, gl.NONE));
        attachGroup.addChild(new es3fFboStencilbufferTests.DepthStencilAttachCase('stencil_only', 'Only stencil part of depth-stencil RBO attached', gl.NONE, gl.STENCIL_ATTACHMENT));
        attachGroup.addChild(new es3fFboStencilbufferTests.DepthStencilAttachCase('depth_stencil_separate', 'Depth and stencil attached separately', gl.DEPTH_ATTACHMENT, gl.STENCIL_ATTACHMENT));
        attachGroup.addChild(new es3fFboStencilbufferTests.DepthStencilAttachCase('depth_stencil_attachment', 'Depth and stencil attached with DEPTH_STENCIL_ATTACHMENT', gl.DEPTH_STENCIL_ATTACHMENT, gl.NONE));
    };

    es3fFboStencilbufferTests.run = function(context) {
        gl = context;
        //Set up root Test
        var state = tcuTestCase.runner;

        var test = new es3fFboStencilbufferTests.FboStencilTests();
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
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fFboStencilbufferTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

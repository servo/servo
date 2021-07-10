/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the 'License');
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an 'AS IS' BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fFboDepthbufferTests');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.opengl.gluTextureUtil');
goog.require('functional.gles3.es3fFboTestCase');
goog.require('functional.gles3.es3fFboTestUtil');
goog.require('framework.common.tcuRGBA');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.referencerenderer.rrUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {
var es3fFboDepthbufferTests = functional.gles3.es3fFboDepthbufferTests;
var es3fFboTestCase = functional.gles3.es3fFboTestCase;
var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var tcuTestCase = framework.common.tcuTestCase;
var tcuSurface = framework.common.tcuSurface;
var tcuTexture = framework.common.tcuTexture;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var tcuRGBA = framework.common.tcuRGBA;
var deRandom = framework.delibs.debase.deRandom;
var tcuImageCompare = framework.common.tcuImageCompare;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var rrUtil = framework.referencerenderer.rrUtil;
var deMath = framework.delibs.debase.deMath;
var gluShaderUtil = framework.opengl.gluShaderUtil;

/** @type {WebGL2RenderingContext} */ var gl;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} format
 * @param {number} width
 * @param {number} height
 */
es3fFboDepthbufferTests.BasicFboDepthCase = function(name, desc, format, width, height) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_format = format;
    this.m_width = width;
    this.m_height = height;
};

setParentClass(es3fFboDepthbufferTests.BasicFboDepthCase, es3fFboTestCase.FboTestCase);

es3fFboDepthbufferTests.BasicFboDepthCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

es3fFboDepthbufferTests.BasicFboDepthCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var colorFormat = gl.RGBA8;
        /** @type {es3fFboTestUtil.GradientShader} */
        var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], gluShaderUtil.DataType.FLOAT_VEC4);
        var texShaderID = ctx.createProgram(texShader);
        var gradShaderID = ctx.createProgram(gradShader);
        var clearDepth = 1;

        // Setup shaders
        gradShader.setGradient(ctx, gradShaderID, [0, 0, 0, 0], [1, 1, 1, 1]);
        texShader.setUniforms(ctx, texShaderID);

        // Setup FBO

        var fbo = ctx.createFramebuffer();
        var colorRbo = ctx.createRenderbuffer();
        var depthRbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.m_width, this.m_height);

        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_format, this.m_width, this.m_height);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthRbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_width, this.m_height);

        // Clear depth to 1
        ctx.clearBufferfv(gl.DEPTH, 0, [clearDepth]);

        // Render gradient with depth = [-1..1]
        ctx.enable(gl.DEPTH_TEST);
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        // Render grid pattern with depth = 0
        var format = gl.RGBA;
        var dataType = gl.UNSIGNED_BYTE;
        var texW = 128;
        var texH = 128;
        var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

        tcuTextureUtil.fillWithGrid(data.getAccess(), 8, [0.2, 0.7, 0.1, 1.0], [0.7, 0.1, 0.5, 0.8]);

        var gridTex = ctx.createTexture();
        ctx.bindTexture(gl.TEXTURE_2D, gridTex);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

        rrUtil.drawQuad(ctx, texShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
        this.checkError();

        // Read results.
        this.readPixelsUsingFormat(dst, 0, 0, this.m_width, this.m_height,
            gluTextureUtil.mapGLInternalFormat(colorFormat),
             [1, 1, 1, 1], [0, 0, 0, 0]);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} format
 * @param {number} width
 * @param {number} height
 */
es3fFboDepthbufferTests.DepthWriteClampCase = function(name, desc, format, width, height) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_format = format;
    this.m_width = width;
    this.m_height = height;
};

setParentClass(es3fFboDepthbufferTests.DepthWriteClampCase, es3fFboTestCase.FboTestCase);

es3fFboDepthbufferTests.DepthWriteClampCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

es3fFboDepthbufferTests.DepthWriteClampCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var colorFormat = gl.RGBA8;
        var transferFmt = gluTextureUtil.getTransferFormat(gluTextureUtil.mapGLInternalFormat(this.m_format));
        /** @type {es3fFboTestUtil.DepthGradientShader} */
        var gradShader = new es3fFboTestUtil.DepthGradientShader(gluShaderUtil.DataType.FLOAT_VEC4);

        var gradShaderID = ctx.createProgram(gradShader);
        var clearDepth = 1;
        var red = [1, 0, 0, 1];
        var green = [0, 1, 0, 1];

        // Setup FBO

        var fbo = ctx.createFramebuffer();
        var colorRbo = ctx.createRenderbuffer();
        var depthTexture = ctx.createTexture();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.m_width, this.m_height);

        ctx.bindTexture(gl.TEXTURE_2D, depthTexture);
        ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_format, this.m_width, this.m_height, 0, transferFmt.format, transferFmt.dataType, null);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, depthTexture, 0);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_width, this.m_height);

        // Clear depth to 1
        ctx.clearBufferfv(gl.DEPTH, 0, [clearDepth]);

        // Render gradient with depth = [-1..1]
        ctx.enable(gl.DEPTH_TEST);
        ctx.depthFunc(gl.ALWAYS);
        gradShader.setUniforms(ctx, gradShaderID, -1, 2, green);
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        ctx.depthMask(false);

        // Test if any fragment has greater depth than 1; there should be none
        ctx.depthFunc(gl.LESS); // (1 < depth) ?
        gradShader.setUniforms(ctx, gradShaderID, 1, 1, red);
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        // Test if any fragment has smaller depth than 0; there should be none
        ctx.depthFunc(gl.GREATER); // (0 > depth) ?
        gradShader.setUniforms(ctx, gradShaderID, 0, 0, red);
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        // Read results.
        this.readPixelsUsingFormat(dst, 0, 0, this.m_width, this.m_height,
            gluTextureUtil.mapGLInternalFormat(colorFormat),
             [1, 1, 1, 1], [0, 0, 0, 0]);

        ctx.depthMask(true);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} format
 * @param {number} width
 * @param {number} height
 */
es3fFboDepthbufferTests.DepthTestClampCase = function(name, desc, format, width, height) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_format = format;
    this.m_width = width;
    this.m_height = height;
};

setParentClass(es3fFboDepthbufferTests.DepthTestClampCase, es3fFboTestCase.FboTestCase);

es3fFboDepthbufferTests.DepthTestClampCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

es3fFboDepthbufferTests.DepthTestClampCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var colorFormat = gl.RGBA8;
        var transferFmt = gluTextureUtil.getTransferFormat(gluTextureUtil.mapGLInternalFormat(this.m_format));
        /** @type {es3fFboTestUtil.DepthGradientShader} */
        var gradShader = new es3fFboTestUtil.DepthGradientShader(gluShaderUtil.DataType.FLOAT_VEC4);

        var gradShaderID = ctx.createProgram(gradShader);
        var clearDepth = 1;
        var yellow = [1, 1, 0, 1];
        var green = [0, 1, 0, 1];

        // Setup FBO

        var fbo = ctx.createFramebuffer();
        var colorRbo = ctx.createRenderbuffer();
        var depthTexture = ctx.createTexture();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.m_width, this.m_height);

        ctx.bindTexture(gl.TEXTURE_2D, depthTexture);
        ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_format, this.m_width, this.m_height, 0, transferFmt.format, transferFmt.dataType, null);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, depthTexture, 0);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_width, this.m_height);

        // Clear depth to 1
        ctx.clearBufferfv(gl.DEPTH, 0, [clearDepth]);

        // Test values used in depth test are clamped

        // Render green quad, depth gradient = [-1..2]
        ctx.enable(gl.DEPTH_TEST);
        ctx.depthFunc(gl.ALWAYS);

        gradShader.setUniforms(ctx, gradShaderID, -1, 2, green);
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        // Render yellow quad, depth gradient = [-0.5..3]. Gradients have equal values only outside [0, 1] range due to clamping
        ctx.depthFunc(gl.EQUAL);

        gradShader.setUniforms(ctx, gradShaderID, -0.5, 3, yellow);
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);

        // Read results.
        this.readPixelsUsingFormat(dst, 0, 0, this.m_width, this.m_height,
            gluTextureUtil.mapGLInternalFormat(colorFormat),
             [1, 1, 1, 1], [0, 0, 0, 0]);
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fFboDepthbufferTests.FboDepthbufferTests = function() {
    tcuTestCase.DeqpTest.call(this, 'depth', 'depth tests');
};

setParentClass(es3fFboDepthbufferTests.FboDepthbufferTests, tcuTestCase.DeqpTest);

es3fFboDepthbufferTests.FboDepthbufferTests.prototype.init = function() {
    var depthFormats = [
        gl.DEPTH_COMPONENT32F,
        gl.DEPTH_COMPONENT24,
        gl.DEPTH_COMPONENT16,
        gl.DEPTH32F_STENCIL8,
        gl.DEPTH24_STENCIL8
    ];

    // .basic
    var basicGroup = tcuTestCase.newTest('basic', 'Basic depth tests');
    this.addChild(basicGroup);

    for (var ndx = 0; ndx < depthFormats.length; ndx++)
        basicGroup.addChild(new es3fFboDepthbufferTests.BasicFboDepthCase(es3fFboTestUtil.getFormatName(depthFormats[ndx]), '', depthFormats[ndx], 119, 127));

    // .depth_write_clamp
    var depthClampGroup = tcuTestCase.newTest('depth_write_clamp', 'Depth write clamping tests');
    this.addChild(depthClampGroup);

    for (var ndx = 0; ndx < depthFormats.length; ndx++)
        depthClampGroup.addChild(new es3fFboDepthbufferTests.DepthWriteClampCase(es3fFboTestUtil.getFormatName(depthFormats[ndx]), '', depthFormats[ndx], 119, 127));

    // .depth_test_clamp
    var depthTestGroup = tcuTestCase.newTest('depth_test_clamp', 'Depth test value clamping tests');
    this.addChild(depthTestGroup);

    for (var ndx = 0; ndx < depthFormats.length; ndx++)
        depthTestGroup.addChild(new es3fFboDepthbufferTests.DepthTestClampCase(es3fFboTestUtil.getFormatName(depthFormats[ndx]), '', depthFormats[ndx], 119, 127));

};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fFboDepthbufferTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fFboDepthbufferTests.FboDepthbufferTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fFboDepthbufferTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

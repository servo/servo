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
goog.provide('functional.gles3.es3fFboInvalidateTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestCase');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {
var es3fFboInvalidateTests = functional.gles3.es3fFboInvalidateTests;
var tcuTestCase = framework.common.tcuTestCase;
var es3fFboTestCase = functional.gles3.es3fFboTestCase;
var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var rrUtil = framework.referencerenderer.rrUtil;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var deMath = framework.delibs.debase.deMath;
var tcuRGBA = framework.common.tcuRGBA;
var tcuImageCompare = framework.common.tcuImageCompare;

/** @type {WebGL2RenderingContext} */ var gl;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

var getDefaultFBDiscardAttachments = function(discardBufferBits) {
    var attachments = [];

    if (discardBufferBits & gl.COLOR_BUFFER_BIT)
        attachments.push(gl.COLOR);

    if (discardBufferBits & gl.DEPTH_BUFFER_BIT)
        attachments.push(gl.DEPTH);

    if (discardBufferBits & gl.STENCIL_BUFFER_BIT)
        attachments.push(gl.STENCIL);

    return attachments;
};

var getFBODiscardAttachments = function(discardBufferBits) {
    var attachments = [];

    if (discardBufferBits & gl.COLOR_BUFFER_BIT)
        attachments.push(gl.COLOR_ATTACHMENT0);

    // \note DEPTH_STENCIL_ATTACHMENT is allowed when discarding FBO, but not with default FB
    if ((discardBufferBits & (gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT)) == (gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT))
        attachments.push(gl.DEPTH_STENCIL_ATTACHMENT);
    else if (discardBufferBits & gl.DEPTH_BUFFER_BIT)
        attachments.push(gl.DEPTH_ATTACHMENT);
    else if (discardBufferBits & gl.STENCIL_BUFFER_BIT)
        attachments.push(gl.STENCIL_ATTACHMENT);

    return attachments;
};

var getCompatibleColorFormat = function() {
    var redBits = gl.getParameter(gl.RED_BITS);
    var greenBits = gl.getParameter(gl.GREEN_BITS);
    var blueBits = gl.getParameter(gl.BLUE_BITS);
    var alphaBits = gl.getParameter(gl.ALPHA_BITS);
    switch ('' + redBits + greenBits + blueBits + alphaBits) {
        case '8888' : return gl.RGBA8;
        case '8880' : return gl.RGB8;
        default:
            throw new Error('Unexpected bit depth');
    }
};

var getCompatibleDepthStencilFormat = function() {
    var depthBits = /** @type {number} */ (gl.getParameter(gl.DEPTH_BITS));
    var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));
    var hasDepth = depthBits > 0;
    var hasStencil = stencilBits > 0;

    if (!hasDepth || !hasStencil || (stencilBits != 8))
        return gl.NONE;

    if (depthBits == 32)
        return gl.DEPTH32F_STENCIL8;
    else if (depthBits == 24)
        return gl.DEPTH24_STENCIL8;
    else
        return gl.NONE;
};

var hasAttachment = function(attachments, attachment) {
    return attachments.indexOf(attachment) >= 0;
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} buffers
 * @param {number=} target
 */
es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase = function(name, desc, buffers, target) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_buffers = buffers;
    this.m_fboTarget = target || gl.FRAMEBUFFER;
};

setParentClass(es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var attachments = getDefaultFBDiscardAttachments(this.m_buffers);

    var shader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var program = ctx.createProgram(shader);
    shader.setColor(ctx, program, [1, 0, 0, 1]);
    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    rrUtil.drawQuad(ctx, program, [-1, -1, -1], [1, 1, 1]);
    ctx.invalidateFramebuffer(this.m_fboTarget, attachments);

    if ((this.m_buffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Color was not preserved - fill with green.
        ctx.disable(gl.DEPTH_TEST);
        ctx.disable(gl.STENCIL_TEST);

        shader.setColor(ctx, program, [0, 1, 0, 1]);
        rrUtil.drawQuad(ctx, program, [-1, -1, 0], [1, 1, 0]);

        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
    }

    if ((this.m_buffers & gl.DEPTH_BUFFER_BIT) != 0) {
        // Depth was not preserved.
        ctx.depthFunc(gl.ALWAYS);
    }

    if ((this.m_buffers & gl.STENCIL_BUFFER_BIT) == 0) {
        // Stencil was preserved.
        ctx.stencilFunc(gl.EQUAL, 1, 0xff);
    }

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    shader.setColor(ctx, program, [0, 0, 1, 1]);
    rrUtil.drawQuad(ctx, program, [-1, -1, 0], [1, 1, 0]);
    dst.readViewport(ctx);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} buffers
 */
es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase = function(name, desc, buffers) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_buffers = buffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var attachments = getDefaultFBDiscardAttachments(this.m_buffers);

    var shader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var program = ctx.createProgram(shader);

    /** @type {es3fFboTestUtil.Texture2DShader} */
    var texShader = new es3fFboTestUtil.Texture2DShader(
        [gluShaderUtil.DataType.SAMPLER_2D], gluShaderUtil.DataType.FLOAT_VEC4);

    /** @type {es3fFboTestUtil.GradientShader} */
    var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);

    var texShaderID = ctx.createProgram(texShader);
    var gradShaderID = ctx.createProgram(gradShader);
    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
    // Create fbo.
    var fbo = ctx.createFramebuffer();
    var tex = ctx.createTexture();
    ctx.bindTexture(gl.TEXTURE_2D, tex);
    ctx.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, this.getWidth(), this.getHeight(), 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    ctx.bindTexture(gl.TEXTURE_2D, null);
    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    shader.setColor(ctx, program, [1, 0, 0, 1]);
    rrUtil.drawQuad(ctx, program, [-1, -1, -1], [1, 1, 1]);

    ctx.invalidateFramebuffer(gl.FRAMEBUFFER, attachments);

    // Switch to fbo and render gradient into it.
    ctx.disable(gl.DEPTH_TEST);
    ctx.disable(gl.STENCIL_TEST);
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);

    gradShader.setGradient(ctx, gradShaderID, [0, 0, 0, 0], [1, 1, 1, 1]);
    rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, 0], [1, 1, 0]);
    // Restore default fbo.
    ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

    if ((this.m_buffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Color was not preserved - fill with green.
        shader.setColor(ctx, program, [0, 1, 0, 1]);
        rrUtil.drawQuad(ctx, program, [-1, -1, 0], [1, 1, 0]);
    }

    if ((this.m_buffers & gl.DEPTH_BUFFER_BIT) != 0) {
        // Depth was not preserved.
        ctx.depthFunc(gl.ALWAYS);
    }

    if ((this.m_buffers & gl.STENCIL_BUFFER_BIT) == 0) {
        // Stencil was preserved.
        ctx.stencilFunc(gl.EQUAL, 1, 0xff);
    }

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);
    ctx.bindTexture(gl.TEXTURE_2D, tex);

    texShader.setUniforms(ctx, texShaderID);
    rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);

    dst.readViewport(ctx);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} buffers
 * @param {number=} target
 */
es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase = function(name, desc, buffers, target) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_buffers = buffers;
    this.m_fboTarget = target || gl.FRAMEBUFFER;
};

setParentClass(es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var invalidateX = this.getWidth() / 4;
    var invalidateY = this.getHeight() / 4;
    var invalidateW = this.getWidth() / 2;
    var invalidateH = this.getHeight() / 2;
    var attachments = getDefaultFBDiscardAttachments(this.m_buffers);

    var shader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var program = ctx.createProgram(shader);
    shader.setColor(ctx, program, [1, 0, 0, 1]);
    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    rrUtil.drawQuad(ctx, program, [-1, -1, -1], [1, 1, 1]);
    ctx.invalidateSubFramebuffer(this.m_fboTarget, attachments, invalidateX, invalidateY, invalidateW, invalidateH);

    // Clear invalidated buffers.
    ctx.clearColor(0, 1, 0, 1);
    ctx.clearStencil(1);
    ctx.scissor(invalidateX, invalidateY, invalidateW, invalidateH);
    ctx.enable(gl.SCISSOR_TEST);
    ctx.clear(this.m_buffers);
    ctx.disable(gl.SCISSOR_TEST);

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    shader.setColor(ctx, program, [0, 0, 1, 1]);
    rrUtil.drawQuad(ctx, program, [-1, -1, 0], [1, 1, 0]);
    dst.readViewport(ctx);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} buffers
 */
es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase = function(name, desc, buffers) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_buffers = buffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var attachments = getDefaultFBDiscardAttachments(this.m_buffers);
    var invalidateX = this.getWidth() / 4;
    var invalidateY = this.getHeight() / 4;
    var invalidateW = this.getWidth() / 2;
    var invalidateH = this.getHeight() / 2;

    var shader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var program = ctx.createProgram(shader);

    /** @type {es3fFboTestUtil.Texture2DShader} */
    var texShader = new es3fFboTestUtil.Texture2DShader(
        [gluShaderUtil.DataType.SAMPLER_2D], gluShaderUtil.DataType.FLOAT_VEC4);

    /** @type {es3fFboTestUtil.GradientShader} */
    var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);

    var texShaderID = ctx.createProgram(texShader);
    var gradShaderID = ctx.createProgram(gradShader);
    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
    // Create fbo.
    var fbo = ctx.createFramebuffer();
    var tex = ctx.createTexture();
    ctx.bindTexture(gl.TEXTURE_2D, tex);
    ctx.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, this.getWidth(), this.getHeight(), 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    ctx.bindTexture(gl.TEXTURE_2D, null);
    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    shader.setColor(ctx, program, [1, 0, 0, 1]);
    rrUtil.drawQuad(ctx, program, [-1, -1, -1], [1, 1, 1]);

    ctx.invalidateSubFramebuffer(gl.FRAMEBUFFER, attachments, invalidateX, invalidateY, invalidateW, invalidateH);

    // Switch to fbo and render gradient into it.
    ctx.disable(gl.DEPTH_TEST);
    ctx.disable(gl.STENCIL_TEST);
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);

    gradShader.setGradient(ctx, gradShaderID, [0, 0, 0, 0], [1, 1, 1, 1]);
    rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, 0], [1, 1, 0]);
    // Restore default fbo.
    ctx.bindFramebuffer(gl.FRAMEBUFFER, null);

    if ((this.m_buffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Color was not preserved - fill with green.
        shader.setColor(ctx, program, [0, 1, 0, 1]);
        rrUtil.drawQuad(ctx, program, [-1, -1, 0], [1, 1, 0]);
    }

    // Clear invalidated buffers.
    ctx.clearColor(0, 1, 0, 1);
    ctx.clearStencil(1);
    ctx.scissor(invalidateX, invalidateY, invalidateW, invalidateH);
    ctx.enable(gl.SCISSOR_TEST);
    ctx.clear(this.m_buffers);
    ctx.disable(gl.SCISSOR_TEST);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);
    ctx.bindTexture(gl.TEXTURE_2D, tex);

    texShader.setUniforms(ctx, texShaderID);
    rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);

    dst.readViewport(ctx);
    ctx.disable(gl.DEPTH_TEST);
    ctx.disable(gl.STENCIL_TEST);
    ctx.disable(gl.BLEND);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} colorFmt
 * @param {number} depthStencilFmt
 * @param {number} invalidateBuffers
 */
es3fFboInvalidateTests.InvalidateFboRenderCase = function(name, desc, colorFmt, depthStencilFmt, invalidateBuffers) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_colorFmt = colorFmt;
    this.m_depthStencilFmt = depthStencilFmt;
    this.m_invalidateBuffers = invalidateBuffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateFboRenderCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateFboRenderCase.prototype.preCheck = function() {
    if (this.m_colorFmt != gl.NONE) this.checkFormatSupport(this.m_colorFmt);
    if (this.m_depthStencilFmt != gl.NONE) this.checkFormatSupport(this.m_depthStencilFmt);
    return true; // No exception thrown
};

es3fFboInvalidateTests.InvalidateFboRenderCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var colorFmt = gluTextureUtil.mapGLInternalFormat(this.m_colorFmt);
    var depthStencilFmt = this.m_depthStencilFmt != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFmt) : new tcuTexture.TextureFormat(null, null);
    var colorFmtInfo = tcuTextureUtil.getTextureFormatInfo(colorFmt);
    var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var cBias = colorFmtInfo.valueMin;
    var cScale = deMath.subtract(colorFmtInfo.valueMax, colorFmtInfo.valueMin);
    var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var attachments = getFBODiscardAttachments(this.m_invalidateBuffers);
    var flatShaderID = ctx.createProgram(flatShader);

    // Create fbo.
    var colorRbo = ctx.createRenderbuffer();
    ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
    ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_colorFmt, this.getWidth(), this.getHeight());

    if (this.m_depthStencilFmt != gl.NONE) {
        var depthStencilRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_depthStencilFmt, this.getWidth(), this.getHeight());
    }

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);

    if (depth)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    if (stencil)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([1, 0, 0, 1], cScale), cBias));
    rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, -1], [1, 1, 1]);

    ctx.invalidateFramebuffer(gl.FRAMEBUFFER, attachments);

    if ((this.m_invalidateBuffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Color was not preserved - fill with green.
        ctx.disable(gl.DEPTH_TEST);
        ctx.disable(gl.STENCIL_TEST);

        flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([0, 1, 0, 1], cScale), cBias));
        rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
    }

    if ((this.m_invalidateBuffers & gl.DEPTH_BUFFER_BIT) != 0) {
        // Depth was not preserved.
        ctx.depthFunc(gl.ALWAYS);
    }

    if ((this.m_invalidateBuffers & gl.STENCIL_BUFFER_BIT) == 0) {
        // Stencil was preserved.
        ctx.stencilFunc(gl.EQUAL, 1, 0xff);
    }

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([0, 0, 1, 1], cScale), cBias));
    rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

    es3fFboTestUtil.readPixels(ctx, dst, 0, 0, this.getWidth(), this.getHeight(), colorFmt, colorFmtInfo.lookupScale, colorFmtInfo.lookupBias);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} colorFmt
 * @param {number} depthStencilFmt
 * @param {number} invalidateBuffers
 */
es3fFboInvalidateTests.InvalidateFboUnbindReadCase = function(name, desc, colorFmt, depthStencilFmt, invalidateBuffers) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_colorFmt = colorFmt;
    this.m_depthStencilFmt = depthStencilFmt;
    this.m_invalidateBuffers = invalidateBuffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateFboUnbindReadCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateFboUnbindReadCase.prototype.preCheck = function() {
    if (this.m_colorFmt != gl.NONE) this.checkFormatSupport(this.m_colorFmt);
    if (this.m_depthStencilFmt != gl.NONE) this.checkFormatSupport(this.m_depthStencilFmt);
    return true; // No exception thrown
};

es3fFboInvalidateTests.InvalidateFboUnbindReadCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var colorFmt = gluTextureUtil.mapGLInternalFormat(this.m_colorFmt);
    var depthStencilFmt = this.m_depthStencilFmt != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFmt) : new tcuTexture.TextureFormat(null, null);
    var colorFmtInfo = tcuTextureUtil.getTextureFormatInfo(colorFmt);
    var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var attachments = getFBODiscardAttachments(this.m_invalidateBuffers);
    // Create fbo.
    var transferFmt = gluTextureUtil.getTransferFormat(colorFmt);
    var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var gradShaderID = ctx.createProgram(gradShader);

    var colorTex = ctx.createTexture();
    ctx.bindTexture(gl.TEXTURE_2D, colorTex);
    ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_colorFmt, this.getWidth(), this.getHeight(), 0, transferFmt.format, transferFmt.dataType, null);
    ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);

    if (this.m_depthStencilFmt != gl.NONE) {
        transferFmt = gluTextureUtil.getTransferFormat(depthStencilFmt);

        var depthStencilTex = ctx.createTexture();
        ctx.bindTexture(gl.TEXTURE_2D, depthStencilTex);
        ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_depthStencilFmt, this.getWidth(), this.getHeight(), 0, transferFmt.format, transferFmt.dataType, null);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    }

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, colorTex, 0);

    if (depth)
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, depthStencilTex, 0);

    if (stencil)
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.TEXTURE_2D, depthStencilTex, 0);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    gradShader.setGradient(ctx, gradShaderID, colorFmtInfo.valueMin, colorFmtInfo.valueMax);
    rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, -1], [1, 1, 1]);

    ctx.invalidateFramebuffer(gl.FRAMEBUFFER, attachments);

    ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
    ctx.disable(gl.DEPTH_TEST);
    ctx.disable(gl.STENCIL_TEST);

    if ((this.m_invalidateBuffers & gl.DEPTH_BUFFER_BIT) != 0) {
        // Render color.
        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluTextureUtil.getSampler2DType(colorFmt)], gluShaderUtil.DataType.FLOAT_VEC4);
        var texShaderID = ctx.createProgram(texShader);

        texShader.setTexScaleBias(0, colorFmtInfo.lookupScale, colorFmtInfo.lookupBias);
        texShader.setUniforms(ctx, texShaderID);

        ctx.bindTexture(gl.TEXTURE_2D, colorTex);
        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);
    } else {
        // Render depth.
        texShader = new es3fFboTestUtil.Texture2DShader(
            [gluTextureUtil.getSampler2DType(depthStencilFmt)], gluShaderUtil.DataType.FLOAT_VEC4);
        texShaderID = ctx.createProgram(texShader);

        texShader.setUniforms(ctx, texShaderID);

        ctx.bindTexture(gl.TEXTURE_2D, depthStencilTex);
        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);
    }

    dst.readViewport(ctx);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} numSamples
 * @param {number} invalidateBuffers
 */
es3fFboInvalidateTests.InvalidateFboUnbindBlitCase = function(name, desc, numSamples, invalidateBuffers) {
// \note Use fullscreen viewport when multisampling - we can't allow GLES3Context do its
//       behing-the-scenes viewport position randomization, because with glBlitFramebuffer,
//       source and destination rectangles must match when multisampling.
    es3fFboTestCase.FboTestCase.call(this, name, desc, numSamples > 0);
    this.m_numSamples = numSamples;
    this.m_colorFmt = getCompatibleColorFormat();
    this.m_depthStencilFmt = getCompatibleDepthStencilFormat();
    this.m_invalidateBuffers = invalidateBuffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateFboUnbindBlitCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateFboUnbindBlitCase.prototype.preCheck = function() {
    if (this.m_colorFmt != gl.NONE) this.checkFormatSupport(this.m_colorFmt);
    if (this.m_depthStencilFmt != gl.NONE) this.checkFormatSupport(this.m_depthStencilFmt);
    return true; // No exception thrown
};

es3fFboInvalidateTests.InvalidateFboUnbindBlitCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var quadSizePixels = [this.m_numSamples == 0 ? this.getWidth() : Math.min(128, this.getWidth()),
                                                     this.m_numSamples == 0 ? this.getHeight() : Math.min(128, this.getHeight())];
    var quadNDCLeftBottomXY = [-1, -1];
    var quadNDCSize = [2 * quadSizePixels[0] / this.getWidth(), 2 * quadSizePixels[1] / this.getHeight()];
    var quadNDCRightTopXY = deMath.add(quadNDCLeftBottomXY, quadNDCSize);
    var depthStencilFmt = this.m_depthStencilFmt != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFmt) : new tcuTexture.TextureFormat(null, null);
    var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var attachments = getFBODiscardAttachments(this.m_invalidateBuffers);
    var flatShaderID = ctx.createProgram(flatShader);

    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    // Create fbo.
    var colorRbo = ctx.createRenderbuffer();
    ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
    ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, this.m_colorFmt, quadSizePixels[0], quadSizePixels[1]);

    if (this.m_depthStencilFmt != gl.NONE) {
        var depthStencilRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
        ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, this.m_depthStencilFmt, quadSizePixels[0], quadSizePixels[1]);
    }

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);

    if (depth)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    if (stencil)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    flatShader.setColor(ctx, flatShaderID, [1, 0, 0, 1]);
    rrUtil.drawQuad(ctx, flatShaderID,
        [quadNDCLeftBottomXY[0], quadNDCLeftBottomXY[1], -1],
        [quadNDCRightTopXY[0], quadNDCRightTopXY[1], 1]);

    ctx.invalidateFramebuffer(gl.FRAMEBUFFER, attachments);

    // Set default framebuffer as draw framebuffer and blit preserved buffers.
    ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, null);
    ctx.blitFramebuffer(0, 0, quadSizePixels[0], quadSizePixels[1],
                         0, 0, quadSizePixels[0], quadSizePixels[1],
                         (gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT) & ~this.m_invalidateBuffers, gl.NEAREST);
    ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, null);

    if ((this.m_invalidateBuffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Color was not preserved - fill with green.
        ctx.disable(gl.DEPTH_TEST);
        ctx.disable(gl.STENCIL_TEST);

        flatShader.setColor(ctx, flatShaderID, [0, 1, 0, 1]);
        rrUtil.drawQuad(ctx, flatShaderID,
            [quadNDCLeftBottomXY[0], quadNDCLeftBottomXY[1], 0],
            [quadNDCRightTopXY[0], quadNDCRightTopXY[1], 0]);

        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
    }

    if ((this.m_invalidateBuffers & gl.DEPTH_BUFFER_BIT) != 0) {
        // Depth was not preserved.
        ctx.depthFunc(gl.ALWAYS);
    }

    if ((this.m_invalidateBuffers & gl.STENCIL_BUFFER_BIT) == 0) {
        // Stencil was preserved.
        ctx.stencilFunc(gl.EQUAL, 1, 0xff);
    }

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    flatShader.setColor(ctx, flatShaderID, [0, 0, 1, 1]);
    rrUtil.drawQuad(ctx, flatShaderID,
        [quadNDCLeftBottomXY[0], quadNDCLeftBottomXY[1], 0],
        [quadNDCRightTopXY[0], quadNDCRightTopXY[1], 0]);

    dst.readViewport(ctx, [0, 0, quadSizePixels[0], quadSizePixels[1]]);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} colorFmt
 * @param {number} depthStencilFmt
 * @param {number} invalidateBuffers
 */
es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase = function(name, desc, colorFmt, depthStencilFmt, invalidateBuffers) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_colorFmt = colorFmt;
    this.m_depthStencilFmt = depthStencilFmt;
    this.m_invalidateBuffers = invalidateBuffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase.prototype.preCheck = function() {
    if (this.m_colorFmt != gl.NONE) this.checkFormatSupport(this.m_colorFmt);
    if (this.m_depthStencilFmt != gl.NONE) this.checkFormatSupport(this.m_depthStencilFmt);
    return true; // No exception thrown
};

es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase.prototype.compare = function(reference, result) {
    var threshold = tcuRGBA.max(es3fFboTestUtil.getFormatThreshold(this.m_colorFmt), new tcuRGBA.RGBA([12, 12, 12, 12]));
    return tcuImageCompare.bilinearCompare('Result', 'Image comparison result', reference.getAccess(), result.getAccess(), threshold);
};

es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var colorFmt = gluTextureUtil.mapGLInternalFormat(this.m_colorFmt);
    var depthStencilFmt = this.m_depthStencilFmt != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFmt) : new tcuTexture.TextureFormat(null, null);
    var colorFmtInfo = tcuTextureUtil.getTextureFormatInfo(colorFmt);
    var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var attachments = getFBODiscardAttachments(this.m_invalidateBuffers);
    // Create fbo.
    var transferFmt = gluTextureUtil.getTransferFormat(colorFmt);
    var gradShader = new es3fFboTestUtil.GradientShader(es3fFboTestUtil.getFragmentOutputType(colorFmt));
    var gradShaderID = ctx.createProgram(gradShader);
    var invalidateX = 0;
    var invalidateY = 0;
    var invalidateW = this.getWidth() / 2;
    var invalidateH = this.getHeight();
    var readX = invalidateW;
    var readY = 0;
    var readW = this.getWidth() / 2;
    var readH = this.getHeight();

    var colorTex = ctx.createTexture();
    ctx.bindTexture(gl.TEXTURE_2D, colorTex);
    ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_colorFmt, this.getWidth(), this.getHeight(), 0, transferFmt.format, transferFmt.dataType, null);
    ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

    if (this.m_depthStencilFmt != gl.NONE) {
        transferFmt = gluTextureUtil.getTransferFormat(depthStencilFmt);

        var depthStencilTex = ctx.createTexture();
        ctx.bindTexture(gl.TEXTURE_2D, depthStencilTex);
        ctx.texImage2D(gl.TEXTURE_2D, 0, this.m_depthStencilFmt, this.getWidth(), this.getHeight(), 0, transferFmt.format, transferFmt.dataType, null);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    }

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, colorTex, 0);

    if (depth)
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, depthStencilTex, 0);

    if (stencil)
        ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.TEXTURE_2D, depthStencilTex, 0);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    this.clearColorBuffer(colorFmt, [0.0, 0.0, 0.0, 1.0]);
    ctx.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    gradShader.setGradient(ctx, gradShaderID, colorFmtInfo.valueMin, colorFmtInfo.valueMax);
    rrUtil.drawQuad(ctx, gradShaderID, [-1, -1, -1], [1, 1, 1]);

    ctx.invalidateSubFramebuffer(gl.FRAMEBUFFER, attachments, invalidateX, invalidateY, invalidateW, invalidateH);

    ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
    ctx.disable(gl.DEPTH_TEST);
    ctx.disable(gl.STENCIL_TEST);

    ctx.clearColor(0.25, 0.5, 0.75, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    // Limit read area using scissor.
    ctx.scissor(readX, readY, readW, readH);
    ctx.enable(gl.SCISSOR_TEST);

    if ((this.m_invalidateBuffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Render color.
        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluTextureUtil.getSampler2DType(colorFmt)], gluShaderUtil.DataType.FLOAT_VEC4);
        var texShaderID = ctx.createProgram(texShader);

        texShader.setTexScaleBias(0, colorFmtInfo.lookupScale, colorFmtInfo.lookupBias);
        texShader.setUniforms(ctx, texShaderID);

        ctx.bindTexture(gl.TEXTURE_2D, colorTex);
        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);
    } else {
        // Render depth.
        texShader = new es3fFboTestUtil.Texture2DShader(
            [gluTextureUtil.getSampler2DType(depthStencilFmt)], gluShaderUtil.DataType.FLOAT_VEC4);
        texShaderID = ctx.createProgram(texShader);

        texShader.setUniforms(ctx, texShaderID);

        ctx.bindTexture(gl.TEXTURE_2D, depthStencilTex);
        rrUtil.drawQuad(ctx, texShaderID, [-1, -1, 0], [1, 1, 0]);
    }

    dst.readViewport(ctx);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} colorFmt
 * @param {number} depthStencilFmt
 * @param {number} invalidateBuffers
 */
es3fFboInvalidateTests.InvalidateSubFboRenderCase = function(name, desc, colorFmt, depthStencilFmt, invalidateBuffers) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_colorFmt = colorFmt;
    this.m_depthStencilFmt = depthStencilFmt;
    this.m_invalidateBuffers = invalidateBuffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateSubFboRenderCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateSubFboRenderCase.prototype.preCheck = function() {
    if (this.m_colorFmt != gl.NONE) this.checkFormatSupport(this.m_colorFmt);
    if (this.m_depthStencilFmt != gl.NONE) this.checkFormatSupport(this.m_depthStencilFmt);
    return true; // No exception thrown
};

es3fFboInvalidateTests.InvalidateSubFboRenderCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var colorFmt = gluTextureUtil.mapGLInternalFormat(this.m_colorFmt);
    var depthStencilFmt = this.m_depthStencilFmt != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFmt) : new tcuTexture.TextureFormat(null, null);
    var colorFmtInfo = tcuTextureUtil.getTextureFormatInfo(colorFmt);
    var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var cBias = colorFmtInfo.valueMin;
    var cScale = deMath.subtract(colorFmtInfo.valueMax, colorFmtInfo.valueMin);
    var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var attachments = getFBODiscardAttachments(this.m_invalidateBuffers);
    var flatShaderID = ctx.createProgram(flatShader);
    var invalidateX = this.getWidth() / 4;
    var invalidateY = this.getHeight() / 4;
    var invalidateW = this.getWidth() / 2;
    var invalidateH = this.getHeight() / 2;

    // Create fbo.
    var colorRbo = ctx.createRenderbuffer();
    ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
    ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_colorFmt, this.getWidth(), this.getHeight());

    if (this.m_depthStencilFmt != gl.NONE) {
        var depthStencilRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_depthStencilFmt, this.getWidth(), this.getHeight());
    }

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);

    if (depth)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    if (stencil)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.clearBufferfv(gl.COLOR, 0, deMath.add(deMath.multiply([0, 0, 0, 1], cScale), cBias));
    ctx.clearBufferfi(gl.DEPTH_STENCIL, 0, 1, 0);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([1, 0, 0, 1], cScale), cBias));
    rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, -1], [1, 1, 1]);

    ctx.invalidateSubFramebuffer(gl.FRAMEBUFFER, attachments, invalidateX, invalidateY, invalidateW, invalidateH);

    // Clear invalidated buffers.
    ctx.scissor(invalidateX, invalidateY, invalidateW, invalidateH);
    ctx.enable(gl.SCISSOR_TEST);

    if (this.m_invalidateBuffers & gl.COLOR_BUFFER_BIT)
        ctx.clearBufferfv(gl.COLOR, 0, deMath.add(deMath.multiply([0, 1, 0, 1], cScale), cBias));

    ctx.clear(this.m_invalidateBuffers & ~gl.COLOR_BUFFER_BIT);
    ctx.disable(gl.SCISSOR_TEST);

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([0, 0, 1, 1], cScale), cBias));
    rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

    es3fFboTestUtil.readPixels(ctx, dst, 0, 0, this.getWidth(), this.getHeight(), colorFmt, colorFmtInfo.lookupScale, colorFmtInfo.lookupBias);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} numSamples
 * @param {number} invalidateBuffers
 */
es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase = function(name, desc, numSamples, invalidateBuffers) {
// \note Use fullscreen viewport when multisampling - we can't allow GLES3Context do its
//       behing-the-scenes viewport position randomization, because with glBlitFramebuffer,
//       source and destination rectangles must match when multisampling.
    es3fFboTestCase.FboTestCase.call(this, name, desc, numSamples > 0);
    this.m_numSamples = numSamples;
    this.m_colorFmt = getCompatibleColorFormat();
    this.m_depthStencilFmt = getCompatibleDepthStencilFormat();
    this.m_invalidateBuffers = invalidateBuffers;
};

setParentClass(es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase.prototype.preCheck = function() {
    if (this.m_colorFmt != gl.NONE) this.checkFormatSupport(this.m_colorFmt);
    if (this.m_depthStencilFmt != gl.NONE) this.checkFormatSupport(this.m_depthStencilFmt);
    return true; // No exception thrown
};

es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var quadSizePixels = [this.m_numSamples == 0 ? this.getWidth() : Math.min(128, this.getWidth()),
                                                     this.m_numSamples == 0 ? this.getHeight() : Math.min(128, this.getHeight())];
    var quadNDCLeftBottomXY = [-1, -1];
    var quadNDCSize = [2 * quadSizePixels[0] / this.getWidth(), 2 * quadSizePixels[1] / this.getHeight()];
    var quadNDCRightTopXY = deMath.add(quadNDCLeftBottomXY, quadNDCSize);
    var depthStencilFmt = this.m_depthStencilFmt != gl.NONE ? gluTextureUtil.mapGLInternalFormat(this.m_depthStencilFmt) : new tcuTexture.TextureFormat(null, null);
    var depth = depthStencilFmt.order == tcuTexture.ChannelOrder.D || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var stencil = depthStencilFmt.order == tcuTexture.ChannelOrder.S || depthStencilFmt.order == tcuTexture.ChannelOrder.DS;
    var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var attachments = getFBODiscardAttachments(this.m_invalidateBuffers);
    var flatShaderID = ctx.createProgram(flatShader);
    var invalidateX = 0;
    var invalidateY = 0;
    var invalidateW = quadSizePixels[0] / 2;
    var invalidateH = quadSizePixels[1];
    var blitX0 = invalidateW;
    var blitY0 = 0;
    var blitX1 = blitX0 + quadSizePixels[0] / 2;
    var blitY1 = blitY0 + quadSizePixels[1];

    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    // Create fbo.
    var colorRbo = ctx.createRenderbuffer();
    ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
    ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, this.m_colorFmt, quadSizePixels[0], quadSizePixels[1]);

    if (this.m_depthStencilFmt != gl.NONE) {
        var depthStencilRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
        ctx.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, this.m_depthStencilFmt, quadSizePixels[0], quadSizePixels[1]);
    }

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);

    if (depth)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    if (stencil)
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    flatShader.setColor(ctx, flatShaderID, [1, 0, 0, 1]);
    rrUtil.drawQuad(ctx, flatShaderID,
        [quadNDCLeftBottomXY[0], quadNDCLeftBottomXY[1], -1],
        [quadNDCRightTopXY[0], quadNDCRightTopXY[1], 1]);

    ctx.invalidateSubFramebuffer(gl.FRAMEBUFFER, attachments, invalidateX, invalidateY, invalidateW, invalidateH);

    // Set default framebuffer as draw framebuffer and blit preserved buffers.
    ctx.bindFramebuffer(gl.DRAW_FRAMEBUFFER, null);
    ctx.blitFramebuffer(blitX0, blitY0, blitX1, blitY1, blitX0, blitY0, blitX1, blitY1,
                         (gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT) & ~this.m_invalidateBuffers, gl.NEAREST);
    ctx.bindFramebuffer(gl.READ_FRAMEBUFFER, null);

    if ((this.m_invalidateBuffers & gl.COLOR_BUFFER_BIT) != 0) {
        // Color was not preserved - fill with green.
        ctx.disable(gl.DEPTH_TEST);
        ctx.disable(gl.STENCIL_TEST);

        flatShader.setColor(ctx, flatShaderID, [0, 1, 0, 1]);
        rrUtil.drawQuad(ctx, flatShaderID,
            [quadNDCLeftBottomXY[0], quadNDCLeftBottomXY[1], 0],
            [quadNDCRightTopXY[0], quadNDCRightTopXY[1], 0]);

        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
    }

    if ((this.m_invalidateBuffers & gl.DEPTH_BUFFER_BIT) != 0) {
        // Depth was not preserved.
        ctx.depthFunc(gl.ALWAYS);
    }

    if ((this.m_invalidateBuffers & gl.STENCIL_BUFFER_BIT) == 0) {
        // Stencil was preserved.
        ctx.stencilFunc(gl.EQUAL, 1, 0xff);
    }

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    flatShader.setColor(ctx, flatShaderID, [0, 0, 1, 1]);
    rrUtil.drawQuad(ctx, flatShaderID,
        [quadNDCLeftBottomXY[0], quadNDCLeftBottomXY[1], 0],
        [quadNDCRightTopXY[0], quadNDCRightTopXY[1], 0]);

    dst.readViewport(ctx, [0, 0, quadSizePixels[0], quadSizePixels[1]]);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} boundTarget
 * @param {number} invalidateTarget
 * @param {Array<number>} invalidateAttachments
 */
es3fFboInvalidateTests.InvalidateFboTargetCase = function(name, desc, boundTarget, invalidateTarget, invalidateAttachments) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_boundTarget = boundTarget;
    this.m_invalidateTarget = invalidateTarget;
    this.m_invalidateAttachments = invalidateAttachments;
};

setParentClass(es3fFboInvalidateTests.InvalidateFboTargetCase, es3fFboTestCase.FboTestCase);

es3fFboInvalidateTests.InvalidateFboTargetCase.prototype.render = function(dst) {
    var ctx = this.getCurrentContext();
    var colorFormat = gl.RGBA8;
    var depthStencilFormat = gl.DEPTH24_STENCIL8;
    var colorFmt = gluTextureUtil.mapGLInternalFormat(colorFormat);
    var colorFmtInfo = tcuTextureUtil.getTextureFormatInfo(colorFmt);
    var cBias = colorFmtInfo.valueMin;
    var cScale = deMath.subtract(colorFmtInfo.valueMax, colorFmtInfo.valueMin);
    var isDiscarded = (this.m_boundTarget == gl.FRAMEBUFFER) ||
                                    (this.m_invalidateTarget == gl.FRAMEBUFFER && this.m_boundTarget == gl.DRAW_FRAMEBUFFER) ||
                                    (this.m_invalidateTarget == this.m_boundTarget);
    var isColorDiscarded = isDiscarded && hasAttachment(this.m_invalidateAttachments, gl.COLOR_ATTACHMENT0);
    var isDepthDiscarded = isDiscarded && (hasAttachment(this.m_invalidateAttachments, gl.DEPTH_ATTACHMENT) || hasAttachment(this.m_invalidateAttachments, gl.DEPTH_STENCIL_ATTACHMENT));
    var isStencilDiscarded = isDiscarded && (hasAttachment(this.m_invalidateAttachments, gl.STENCIL_ATTACHMENT) || hasAttachment(this.m_invalidateAttachments, gl.DEPTH_STENCIL_ATTACHMENT));

    var flatShader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4);
    var flatShaderID = ctx.createProgram(flatShader);

    // Create fbo.
    var colorRbo = ctx.createRenderbuffer();
    ctx.bindRenderbuffer(gl.RENDERBUFFER, colorRbo);
    ctx.renderbufferStorage(gl.RENDERBUFFER, colorFormat, this.getWidth(), this.getHeight());

    var depthStencilRbo = ctx.createRenderbuffer();
    ctx.bindRenderbuffer(gl.RENDERBUFFER, depthStencilRbo);
    ctx.renderbufferStorage(gl.RENDERBUFFER, depthStencilFormat, this.getWidth(), this.getHeight());

    var fbo = ctx.createFramebuffer();
    ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorRbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);
    ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, depthStencilRbo);

    this.checkFramebufferStatus(gl.FRAMEBUFFER);

    ctx.clearColor(0, 0, 0, 1);
    ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    ctx.enable(gl.DEPTH_TEST);
    ctx.enable(gl.STENCIL_TEST);
    ctx.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP);
    ctx.stencilFunc(gl.ALWAYS, 1, 0xff);

    flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([1, 0, 0, 1], cScale), cBias));
    rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, -1], [1, 1, 1]);

    // Bound FBO to test target and default to other
    if (this.m_boundTarget != gl.FRAMEBUFFER) {
        // Dummy fbo is used as complemeting target (read when discarding draw for example).
        // \note Framework takes care of deleting objects at the end of test case.
        var dummyTarget = this.m_boundTarget == gl.DRAW_FRAMEBUFFER ? gl.READ_FRAMEBUFFER : gl.DRAW_FRAMEBUFFER;

        var dummyColorRbo = ctx.createRenderbuffer();
        ctx.bindRenderbuffer(gl.RENDERBUFFER, dummyColorRbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 64, 64);
        var dummyFbo = ctx.createFramebuffer();
        ctx.bindFramebuffer(dummyTarget, dummyFbo);
        ctx.framebufferRenderbuffer(dummyTarget, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, dummyColorRbo);

        ctx.bindFramebuffer(this.m_boundTarget, fbo);
    }

    ctx.invalidateFramebuffer(this.m_invalidateTarget, this.m_invalidateAttachments);

    if (this.m_boundTarget != gl.FRAMEBUFFER)
        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);

    if (isColorDiscarded) {
        // Color was not preserved - fill with green.
        ctx.disable(gl.DEPTH_TEST);
        ctx.disable(gl.STENCIL_TEST);

        flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([0, 1, 0, 1], cScale), cBias));
        rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

        ctx.enable(gl.DEPTH_TEST);
        ctx.enable(gl.STENCIL_TEST);
    }

    if (isDepthDiscarded) {
        // Depth was not preserved.
        ctx.depthFunc(gl.ALWAYS);
    }

    if (!isStencilDiscarded) {
        // Stencil was preserved.
        ctx.stencilFunc(gl.EQUAL, 1, 0xff);
    }

    ctx.enable(gl.BLEND);
    ctx.blendFunc(gl.ONE, gl.ONE);
    ctx.blendEquation(gl.FUNC_ADD);

    flatShader.setColor(ctx, flatShaderID, deMath.add(deMath.multiply([0, 0, 1, 1], cScale), cBias));
    rrUtil.drawQuad(ctx, flatShaderID, [-1, -1, 0], [1, 1, 0]);

    es3fFboTestUtil.readPixels(ctx, dst, 0, 0, this.getWidth(), this.getHeight(), colorFmt, colorFmtInfo.lookupScale, colorFmtInfo.lookupBias);
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fFboInvalidateTests.FboInvalidateTests = function() {
    tcuTestCase.DeqpTest.call(this, 'invalidate', 'Framebuffer invalidate tests');
};

es3fFboInvalidateTests.FboInvalidateTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fFboInvalidateTests.FboInvalidateTests.prototype.constructor = es3fFboInvalidateTests.FboInvalidateTests;

es3fFboInvalidateTests.FboInvalidateTests.prototype.init = function() {
    var defaultFbGroup = new tcuTestCase.DeqpTest('default', 'Default framebuffer invalidate tests');
    this.addChild(defaultFbGroup);

    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('render_none', 'Invalidating no framebuffers (ref)', 0));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('render_color', 'Rendering after invalidating colorbuffer', gl.COLOR_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('render_depth', 'Rendering after invalidating depthbuffer', gl.DEPTH_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('render_stencil', 'Rendering after invalidating stencilbuffer', gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('render_depth_stencil', 'Rendering after invalidating depth- and stencilbuffers', gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('render_all', 'Rendering after invalidating all buffers', gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase('bind_color', 'Binding fbo after invalidating colorbuffer', gl.COLOR_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase('bind_depth', 'Binding fbo after invalidating depthbuffer', gl.DEPTH_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase('bind_stencil', 'Binding fbo after invalidating stencilbuffer', gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase('bind_depth_stencil', 'Binding fbo after invalidating depth- and stencilbuffers', gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferBindCase('bind_all', 'Binding fbo after invalidating all buffers', gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase('sub_render_color', 'Rendering after invalidating colorbuffer', gl.COLOR_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase('sub_render_depth', 'Rendering after invalidating depthbuffer', gl.DEPTH_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase('sub_render_stencil', 'Rendering after invalidating stencilbuffer', gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase('sub_render_depth_stencil', 'Rendering after invalidating depth- and stencilbuffers', gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferRenderCase('sub_render_all', 'Rendering after invalidating all buffers', gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase('sub_bind_color', 'Binding fbo after invalidating colorbuffer', gl.COLOR_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase('sub_bind_depth', 'Binding fbo after invalidating depthbuffer', gl.DEPTH_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase('sub_bind_stencil', 'Binding fbo after invalidating stencilbuffer', gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase('sub_bind_depth_stencil', 'Binding fbo after invalidating depth- and stencilbuffers', gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultSubFramebufferBindCase('sub_bind_all', 'Binding fbo after invalidating all buffers', gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('draw_framebuffer_color', 'Invalidating gl.COLOR in gl.DRAW_FRAMEBUFFER', gl.COLOR_BUFFER_BIT, gl.DRAW_FRAMEBUFFER));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('draw_framebuffer_all', 'Invalidating all in gl.DRAW_FRAMEBUFFER', gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT, gl.DRAW_FRAMEBUFFER));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('read_framebuffer_color', 'Invalidating gl.COLOR in gl.READ_FRAMEBUFFER', gl.COLOR_BUFFER_BIT, gl.READ_FRAMEBUFFER));
    defaultFbGroup.addChild(new es3fFboInvalidateTests.InvalidateDefaultFramebufferRenderCase('read_framebuffer_all', 'Invalidating all in gl.READ_FRAMEBUFFER', gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT, gl.READ_FRAMEBUFFER));

    // invalidate.whole.
    var wholeFboGroup = new tcuTestCase.DeqpTest('whole', 'Invalidating whole framebuffer object');
    this.addChild(wholeFboGroup);
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboRenderCase('render_none', '', gl.RGBA8, gl.DEPTH24_STENCIL8, 0));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboRenderCase('render_color', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboRenderCase('render_depth', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboRenderCase('render_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.STENCIL_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboRenderCase('render_depth_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboRenderCase('render_all', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));

    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindReadCase('unbind_read_color', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindReadCase('unbind_read_depth', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindReadCase('unbind_read_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.STENCIL_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindReadCase('unbind_read_depth_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindReadCase('unbind_read_color_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));

    if (getCompatibleDepthStencilFormat() !== gl.NONE) {
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_color', '', 0, gl.COLOR_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_depth', '', 0, gl.DEPTH_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_stencil', '', 0, gl.STENCIL_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_depth_stencil', '', 0, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_msaa_color', '', 4, gl.COLOR_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_msaa_depth', '', 4, gl.DEPTH_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_msaa_stencil', '', 4, gl.STENCIL_BUFFER_BIT));
        wholeFboGroup.addChild(new es3fFboInvalidateTests.InvalidateFboUnbindBlitCase('unbind_blit_msaa_depth_stencil', '', 4, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    }

    // invalidate.sub.
    var subFboGroup = new tcuTestCase.DeqpTest('sub', 'Invalidating subsection of framebuffer object');
    this.addChild(subFboGroup);
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase('render_none', '', gl.RGBA8, gl.DEPTH24_STENCIL8, 0));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase('render_color', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase('render_depth', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase('render_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.STENCIL_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase('render_depth_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase('render_all', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));

    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase('unbind_read_color', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase('unbind_read_depth', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase('unbind_read_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.STENCIL_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase('unbind_read_depth_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase('unbind_read_color_stencil', '', gl.RGBA8, gl.DEPTH24_STENCIL8, gl.COLOR_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));

    if (getCompatibleDepthStencilFormat() !== gl.NONE) {
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_color', '', 0, gl.COLOR_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_depth', '', 0, gl.DEPTH_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_stencil', '', 0, gl.STENCIL_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_depth_stencil', '', 0, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_msaa_color', '', 4, gl.COLOR_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_msaa_depth', '', 4, gl.DEPTH_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_msaa_stencil', '', 4, gl.STENCIL_BUFFER_BIT));
        subFboGroup.addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindBlitCase('unbind_blit_msaa_depth_stencil', '', 4, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));
    }
    // invalidate.format.
    var numFormatSubGroups = 3;
    var formatGroup = [];
    for (var ii = 0; ii < numFormatSubGroups; ++ii) {
        formatGroup[ii] = new tcuTestCase.DeqpTest('format', 'Invalidating framebuffers with selected formats');
        this.addChild(formatGroup[ii]);
    }
    // Color buffer formats.
    var colorFormats = [
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

    // Depth/stencilbuffer formats.
    var depthStencilFormats = [
        gl.DEPTH_COMPONENT32F,
        gl.DEPTH_COMPONENT24,
        gl.DEPTH_COMPONENT16,
        gl.DEPTH32F_STENCIL8,
        gl.DEPTH24_STENCIL8,
        gl.STENCIL_INDEX8
    ];

    // Colorbuffer tests use invalidate, unbind, read test.
    for (var ndx = 0; ndx < colorFormats.length; ndx++)
        formatGroup[ndx % numFormatSubGroups].addChild(new es3fFboInvalidateTests.InvalidateSubFboUnbindReadCase(es3fFboTestUtil.getFormatName(colorFormats[ndx]), '', colorFormats[ndx], gl.NONE, gl.COLOR_BUFFER_BIT));

    // Depth/stencilbuffer tests use invalidate, render test.
    for (var ndx = 0; ndx < depthStencilFormats.length; ndx++)
        formatGroup[ndx % numFormatSubGroups].addChild(new es3fFboInvalidateTests.InvalidateSubFboRenderCase(es3fFboTestUtil.getFormatName(depthStencilFormats[ndx]), '', gl.RGBA8, depthStencilFormats[ndx], gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT));

    // invalidate.target
    var targetGroup = new tcuTestCase.DeqpTest('target', 'Invalidate target');
    this.addChild(targetGroup);

    var s_targetCases = [
        ['framebuffer_framebuffer', gl.FRAMEBUFFER, gl.FRAMEBUFFER],
        ['framebuffer_read_framebuffer', gl.FRAMEBUFFER, gl.READ_FRAMEBUFFER],
        ['framebuffer_draw_framebuffer', gl.FRAMEBUFFER, gl.DRAW_FRAMEBUFFER],
        ['read_framebuffer_framebuffer', gl.READ_FRAMEBUFFER, gl.FRAMEBUFFER],
        ['read_framebuffer_read_framebuffer', gl.READ_FRAMEBUFFER, gl.READ_FRAMEBUFFER],
        ['read_framebuffer_draw_framebuffer', gl.READ_FRAMEBUFFER, gl.DRAW_FRAMEBUFFER],
        ['draw_framebuffer_framebuffer', gl.DRAW_FRAMEBUFFER, gl.FRAMEBUFFER],
        ['draw_framebuffer_read_framebuffer', gl.DRAW_FRAMEBUFFER, gl.READ_FRAMEBUFFER],
        ['draw_framebuffer_draw_framebuffer', gl.DRAW_FRAMEBUFFER, gl.DRAW_FRAMEBUFFER]
    ];

    var colorAttachment = [gl.COLOR_ATTACHMENT0];
    var depthStencilAttachment = [gl.DEPTH_STENCIL_ATTACHMENT];
    var allAttachments = [gl.COLOR_ATTACHMENT0, gl.DEPTH_ATTACHMENT, gl.STENCIL_ATTACHMENT];

    for (var caseNdx = 0; caseNdx < s_targetCases.length; caseNdx++) {
        var baseName = s_targetCases[caseNdx][0];
        var invalidateT = s_targetCases[caseNdx][1];
        var boundT = s_targetCases[caseNdx][1];

        targetGroup.addChild(new es3fFboInvalidateTests.InvalidateFboTargetCase(baseName + '_color', '', boundT, invalidateT, colorAttachment));
        targetGroup.addChild(new es3fFboInvalidateTests.InvalidateFboTargetCase(baseName + '_depth_stencil', '', boundT, invalidateT, depthStencilAttachment));
        targetGroup.addChild(new es3fFboInvalidateTests.InvalidateFboTargetCase(baseName + '_all', '', boundT, invalidateT, allAttachments));
    }

};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fFboInvalidateTests.run = function(context, range) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fFboInvalidateTests.FboInvalidateTests());

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
        testFailedOptions('Failed to es3fFboInvalidateTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

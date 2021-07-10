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
goog.provide('functional.gles3.es3fFboColorbufferTests');
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
var es3fFboColorbufferTests = functional.gles3.es3fFboColorbufferTests;
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

/** @const*/ var MIN_THRESHOLD = new tcuRGBA.RGBA([12, 12, 12, 12]);

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @param {deRandom.Random} rnd
 * @param {Array<number>} minVal
 * @param {Array<number>} maxVal
 * @return {Array<number>}
 */
es3fFboColorbufferTests.randomVector = function(rnd, minVal, maxVal) {
    var res = [];
    for (var ndx = 0; ndx < minVal.length; ndx++)
        res[ndx] = rnd.getFloat(minVal[ndx], maxVal[ndx]);
    return res;
};

/**
 * @param {deRandom.Random} rnd
 * @return {Array<number>}
 */
es3fFboColorbufferTests.generateRandomColor = function(rnd) {
    var retVal = [];

    for (var i = 0; i < 3; ++i)
        retVal[i] = rnd.getFloat();
    retVal[3] = 1;

    return retVal;
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} format
 */
es3fFboColorbufferTests.FboColorbufferCase = function(name, desc, format) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_format = format;
};

setParentClass(es3fFboColorbufferTests.FboColorbufferCase, es3fFboTestCase.FboTestCase);

/**
 * @param {tcuSurface.Surface} reference
 * @param {tcuSurface.Surface} result
 * @return {boolean}
 */
es3fFboColorbufferTests.FboColorbufferCase.prototype.compare = function(reference, result) {
        /** @type {tcuRGBA.RGBA} */ var threshold = tcuRGBA.max(es3fFboTestUtil.getFormatThreshold(this.m_format), MIN_THRESHOLD);

        bufferedLogToConsole('Comparing images, threshold: ' + threshold);

        return tcuImageCompare.bilinearCompare('Result', 'Image comparison result', reference.getAccess(), result.getAccess(), threshold);
    };

/**
 * Deinit. Clear some GL state variables
 */
es3fFboColorbufferTests.FboColorbufferCase.prototype.deinit = function() {
        // Texture state
        {
            // Only TEXTURE0 and TEXTURE1 are used in this test
            var numTexUnits = 2;

            for (var ndx = 0; ndx < numTexUnits; ndx++) {
                gl.activeTexture(gl.TEXTURE0 + ndx);

                // Reset 2D texture
                gl.bindTexture(gl.TEXTURE_2D, null);

                // Reset cube map texture
                gl.bindTexture(gl.TEXTURE_CUBE_MAP, null);

                // Reset 2D array texture
                gl.bindTexture(gl.TEXTURE_2D_ARRAY, null);

                // Reset 3D texture
                gl.bindTexture(gl.TEXTURE_3D, null);
            }

            gl.activeTexture(gl.TEXTURE0);
        }

        // Pixel operations
        {
            gl.disable(gl.SCISSOR_TEST);
            gl.disable(gl.BLEND);
        }

        // Framebuffer control
        {
            gl.clearColor(0.0, 0.0, 0.0, 0.0);
        }
    };

/**
 * @constructor
 * @extends {es3fFboColorbufferTests.FboColorbufferCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} format
 * @param {number} width
 * @param {number} height
 */
es3fFboColorbufferTests.FboColorClearCase = function(name, desc, format, width, height) {
    es3fFboColorbufferTests.FboColorbufferCase.call(this, name, desc, format);
    this.m_width = width;
    this.m_height = height;
};

setParentClass(es3fFboColorbufferTests.FboColorClearCase, es3fFboColorbufferTests.FboColorbufferCase);

es3fFboColorbufferTests.FboColorClearCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

es3fFboColorbufferTests.FboColorClearCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var fboFormat = gluTextureUtil.mapGLInternalFormat(this.m_format);
        var fmtClass = tcuTexture.getTextureChannelClass(fboFormat.type);
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(fboFormat);
        var rnd = new deRandom.Random(17);
        var numClears = 16;

        var fbo = ctx.createFramebuffer();
        var rbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, rbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_format, this.m_width, this.m_height);
        this.checkError();

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_width, this.m_height);

        // Initialize to transparent black.
        switch (fmtClass) {
            case tcuTexture.TextureChannelClass.FLOATING_POINT:
            case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
            case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
                ctx.clearBufferfv(gl.COLOR, 0, new Float32Array(4));
                break;

            case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
                ctx.clearBufferuiv(gl.COLOR, 0, new Uint32Array(4));
                break;

            case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                ctx.clearBufferiv(gl.COLOR, 0, new Int32Array(4));
                break;

            default:
                throw new Error('Invalid channelclass ' + fmtClass);
        }

        // Do random scissored clears.
        ctx.enable(gl.SCISSOR_TEST);
        for (var ndx = 0; ndx < numClears; ndx++) {
            var x = rnd.getInt(0, this.m_width - 1);
            var y = rnd.getInt(0, this.m_height - 1);
            var w = rnd.getInt(1, this.m_width - x);
            var h = rnd.getInt(1, this.m_height - y);
            var color = es3fFboColorbufferTests.randomVector(rnd, fmtInfo.valueMin, fmtInfo.valueMax);

            ctx.scissor(x, y, w, h);

            switch (fmtClass) {
                case tcuTexture.TextureChannelClass.FLOATING_POINT:
                case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
                case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
                    ctx.clearBufferfv(gl.COLOR, 0, color);
                    break;

                case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
                    ctx.clearBufferuiv(gl.COLOR, 0, color);
                    break;

                case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                    ctx.clearBufferiv(gl.COLOR, 0, color);
                    break;

                default:
                    throw new Error('Invalid channelclass ' + fmtClass);
            }
        }

        // Read results from renderbuffer.
        this.readPixelsUsingFormat(dst, 0, 0, this.m_width, this.m_height, fboFormat, fmtInfo.lookupScale, fmtInfo.lookupBias);
        this.checkError();
    };

/**
 * @constructor
 * @extends {es3fFboColorbufferTests.FboColorbufferCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} tex0Fmt
 * @param {Array<number>} tex0Size
 * @param {number} tex1Fmt
 * @param {Array<number>} tex1Size
 */
es3fFboColorbufferTests.FboColorMultiTex2DCase = function(name, desc, tex0Fmt, tex0Size, tex1Fmt, tex1Size) {
    es3fFboColorbufferTests.FboColorbufferCase.call(this, name, desc, tex0Fmt);
    this.m_tex0Fmt = tex0Fmt;
    this.m_tex0Size = tex0Size;
    this.m_tex1Fmt = tex1Fmt;
    this.m_tex1Size = tex1Size;
};

setParentClass(es3fFboColorbufferTests.FboColorMultiTex2DCase, es3fFboColorbufferTests.FboColorbufferCase);

es3fFboColorbufferTests.FboColorMultiTex2DCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_tex0Fmt);
        this.checkFormatSupport(this.m_tex1Fmt);
        return true; // No exception thrown
    };

es3fFboColorbufferTests.FboColorMultiTex2DCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var texFmt0 = gluTextureUtil.mapGLInternalFormat(this.m_tex0Fmt);
        var texFmt1 = gluTextureUtil.mapGLInternalFormat(this.m_tex1Fmt);
        var fmtInfo0 = tcuTextureUtil.getTextureFormatInfo(texFmt0);
        var fmtInfo1 = tcuTextureUtil.getTextureFormatInfo(texFmt1);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFbo0Shader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], es3fFboTestUtil.getFragmentOutputType(texFmt0),
            deMath.subtract(fmtInfo0.valueMax, fmtInfo0.valueMin),
            fmtInfo0.valueMin);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFbo1Shader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], es3fFboTestUtil.getFragmentOutputType(texFmt1),
            deMath.subtract(fmtInfo1.valueMax, fmtInfo1.valueMin),
            fmtInfo1.valueMin);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var multiTexShader = new es3fFboTestUtil.Texture2DShader(
            [gluTextureUtil.getSampler2DType(texFmt0), gluTextureUtil.getSampler2DType(texFmt1)],
            gluShaderUtil.DataType.FLOAT_VEC4);

        var texToFbo0ShaderID = ctx.createProgram(texToFbo0Shader);
        var texToFbo1ShaderID = ctx.createProgram(texToFbo1Shader);
        var multiTexShaderID = ctx.createProgram(multiTexShader);

        // Setup shaders
        multiTexShader.setTexScaleBias(0, deMath.scale(fmtInfo0.lookupScale, 0.5), deMath.scale(fmtInfo0.lookupBias, 0.5));
        multiTexShader.setTexScaleBias(1, deMath.scale(fmtInfo1.lookupScale, 0.5), deMath.scale(fmtInfo1.lookupBias, 0.5));
        texToFbo0Shader.setUniforms(ctx, texToFbo0ShaderID);
        texToFbo1Shader.setUniforms(ctx, texToFbo1ShaderID);
        multiTexShader.setUniforms(ctx, multiTexShaderID);

        var fbo0 = ctx.createFramebuffer();
        var fbo1 = ctx.createFramebuffer();
        var tex0 = ctx.createTexture();
        var tex1 = ctx.createTexture();

        for (var ndx = 0; ndx < 2; ndx++) {
            var transferFmt = gluTextureUtil.getTransferFormat(ndx ? texFmt1 : texFmt0);
            var format = ndx ? this.m_tex1Fmt : this.m_tex0Fmt;
            var isFilterable = gluTextureUtil.isGLInternalColorFormatFilterable(format);
            var size = ndx ? this.m_tex1Size : this.m_tex0Size;
            var fbo = ndx ? fbo1 : fbo0;
            var tex = ndx ? tex1 : tex0;

            ctx.bindTexture(gl.TEXTURE_2D, tex);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, isFilterable ? gl.LINEAR : gl.NEAREST);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, isFilterable ? gl.LINEAR : gl.NEAREST);
            ctx.texImage2D(gl.TEXTURE_2D, 0, format, size[0], size[1], 0, transferFmt.format, transferFmt.dataType, null);

            ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
            this.checkError();
            this.checkFramebufferStatus(gl.FRAMEBUFFER);
        }

        // Render textures to both framebuffers.
        for (var ndx = 0; ndx < 2; ndx++) {
            var format = gl.RGBA;
            var dataType = gl.UNSIGNED_BYTE;
            var texW = 128;
            var texH = 128;
            var tmpTex;
            var fbo = ndx ? fbo1 : fbo0;
            var viewport = ndx ? this.m_tex1Size : this.m_tex0Size;
            var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

            if (ndx == 0)
                tcuTextureUtil.fillWithComponentGradients(data.getAccess(), [0, 0, 0, 0], [1, 1, 1, 1]);
            else
                tcuTextureUtil.fillWithGrid(data.getAccess(), 8, [0.2, 0.7, 0.1, 1.0], [0.7, 0.1, 0.5, 0.8]);

            tmpTex = ctx.createTexture();
            ctx.bindTexture(gl.TEXTURE_2D, tmpTex);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
            ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

            ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            ctx.viewport(0, 0, viewport[0], viewport[1]);
            rrUtil.drawQuad(ctx, ndx ? texToFbo1ShaderID : texToFbo0ShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
        }

        // Render to framebuffer.
        ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
        ctx.viewport(0, 0, ctx.getWidth(), ctx.getHeight());
        ctx.activeTexture(gl.TEXTURE0);
        ctx.bindTexture(gl.TEXTURE_2D, tex0);
        ctx.activeTexture(gl.TEXTURE1);
        ctx.bindTexture(gl.TEXTURE_2D, tex1);
        rrUtil.drawQuad(ctx, multiTexShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        this.readPixels(dst, 0, 0, ctx.getWidth(), ctx.getHeight());
    };

/**
 * @constructor
 * @extends {es3fFboColorbufferTests.FboColorbufferCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} texFmt
 * @param {Array<number>} texSize
 */
es3fFboColorbufferTests.FboColorTexCubeCase = function(name, desc, texFmt, texSize) {
    es3fFboColorbufferTests.FboColorbufferCase.call(this, name, desc, texFmt);
    this.m_texSize = texSize;
};

setParentClass(es3fFboColorbufferTests.FboColorTexCubeCase, es3fFboColorbufferTests.FboColorbufferCase);

es3fFboColorbufferTests.FboColorTexCubeCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

es3fFboColorbufferTests.FboColorTexCubeCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_format);
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);

        var cubeGLFaces = [
            gl.TEXTURE_CUBE_MAP_POSITIVE_X,
            gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
            gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
            gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
            gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
            gl.TEXTURE_CUBE_MAP_NEGATIVE_Z
        ];

        var cubeTexFaces = [
            tcuTexture.CubeFace.CUBEFACE_POSITIVE_X,
            tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y,
            tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z,
            tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X,
            tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y,
            tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z
        ];

        var rnd = new deRandom.Random(21);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFboShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], es3fFboTestUtil.getFragmentOutputType(texFmt),
            deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin),
            fmtInfo.valueMin);

        /** @type {es3fFboTestUtil.TextureCubeShader} */
        var cubeTexShader = new es3fFboTestUtil.TextureCubeShader(
            gluTextureUtil.getSamplerCubeType(texFmt),
            gluShaderUtil.DataType.FLOAT_VEC4);

        var texToFboShaderID = ctx.createProgram(texToFboShader);
        var cubeTexShaderID = ctx.createProgram(cubeTexShader);

        // Setup shaders
        texToFboShader.setUniforms(ctx, texToFboShaderID);
        cubeTexShader.setTexScaleBias(fmtInfo.lookupScale, fmtInfo.lookupBias);

        // Framebuffers.
        var fbos = [];
        var tex;

        var transferFmt = gluTextureUtil.getTransferFormat(texFmt);
        var isFilterable = gluTextureUtil.isGLInternalColorFormatFilterable(this.m_format);
        var size = this.m_texSize;

        tex = ctx.createTexture();

        ctx.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        ctx.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, isFilterable ? gl.LINEAR : gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, isFilterable ? gl.LINEAR : gl.NEAREST);

        // Generate an image and FBO for each cube face
        for (var ndx = 0; ndx < cubeGLFaces.length; ndx++)
            ctx.texImage2D(cubeGLFaces[ndx], 0, this.m_format, size[0], size[1], 0, transferFmt.format, transferFmt.dataType, null);
        this.checkError();

        for (var ndx = 0; ndx < cubeGLFaces.length; ndx++) {
            var layerFbo = ctx.createFramebuffer();
            ctx.bindFramebuffer(gl.FRAMEBUFFER, layerFbo);
            ctx.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, cubeGLFaces[ndx], tex, 0);
            this.checkError();
            this.checkFramebufferStatus(gl.FRAMEBUFFER);

            fbos.push(layerFbo);
        }

        // Render test images to random cube faces
        var order = [];

        for (var n = 0; n < fbos.length; n++)
            order.push(n);
        rnd.shuffle(order);

        for (var ndx = 0; ndx < 4; ndx++) {
            var face = order[ndx];
            var format = gl.RGBA;
            var dataType = gl.UNSIGNED_BYTE;
            var texW = 128;
            var texH = 128;
            var tmpTex;
            var fbo = fbos[face];
            var viewport = this.m_texSize;
            var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

            tcuTextureUtil.fillWithGrid(data.getAccess(), 8, es3fFboColorbufferTests.generateRandomColor(rnd), [0, 0, 0, 0]);

            tmpTex = ctx.createTexture();
            ctx.bindTexture(gl.TEXTURE_2D, tmpTex);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
            ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

            ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            ctx.viewport(0, 0, viewport[0], viewport[1]);
            rrUtil.drawQuad(ctx, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
            this.checkError();

            // Render to framebuffer
            var p0 = [(ndx % 2) - 1.0, Math.floor(ndx / 2) - 1.0, 0.0];
            var p1 = deMath.add(p0, [1.0, 1.0, 0.0]);

            ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
            ctx.viewport(0, 0, ctx.getWidth(), ctx.getHeight());

            ctx.activeTexture(gl.TEXTURE0);
            ctx.bindTexture(gl.TEXTURE_CUBE_MAP, tex);

            cubeTexShader.setFace(cubeTexFaces[face]);
            cubeTexShader.setUniforms(ctx, cubeTexShaderID);

            rrUtil.drawQuad(ctx, cubeTexShaderID, p0, p1);
            this.checkError();
        }

        this.readPixels(dst, 0, 0, ctx.getWidth(), ctx.getHeight());
    };

/**
 * @constructor
 * @extends {es3fFboColorbufferTests.FboColorbufferCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} texFmt
 * @param {Array<number>} texSize
 */
es3fFboColorbufferTests.FboColorTex2DArrayCase = function(name, desc, texFmt, texSize) {
    es3fFboColorbufferTests.FboColorbufferCase.call(this, name, desc, texFmt);
    this.m_texSize = texSize;
};

setParentClass(es3fFboColorbufferTests.FboColorTex2DArrayCase, es3fFboColorbufferTests.FboColorbufferCase);

es3fFboColorbufferTests.FboColorTex2DArrayCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

    es3fFboColorbufferTests.FboColorTex2DArrayCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_format);
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        var rnd = new deRandom.Random(100);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFboShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], es3fFboTestUtil.getFragmentOutputType(texFmt),
            deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin),
            fmtInfo.valueMin);

        /** @type {es3fFboTestUtil.Texture2DArrayShader} */
        var arrayTexShader = new es3fFboTestUtil.Texture2DArrayShader(
            gluTextureUtil.getSampler2DArrayType(texFmt),
            gluShaderUtil.DataType.FLOAT_VEC4);

        var texToFboShaderID = ctx.createProgram(texToFboShader);
        var arrayTexShaderID = ctx.createProgram(arrayTexShader);

        // Setup textures
        texToFboShader.setUniforms(ctx, texToFboShaderID);
        arrayTexShader.setTexScaleBias(fmtInfo.lookupScale, fmtInfo.lookupBias);

        // Framebuffers.
        var fbos = [];
        var tex;

        var transferFmt = gluTextureUtil.getTransferFormat(texFmt);
        var isFilterable = gluTextureUtil.isGLInternalColorFormatFilterable(this.m_format);
        var size = this.m_texSize;

        tex = ctx.createTexture();

        ctx.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        ctx.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
        ctx.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, isFilterable ? gl.LINEAR : gl.NEAREST);
        ctx.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, isFilterable ? gl.LINEAR : gl.NEAREST);
        ctx.texImage3D(gl.TEXTURE_2D_ARRAY, 0, this.m_format, size[0], size[1], size[2], 0, transferFmt.format, transferFmt.dataType, null);

        // Generate an FBO for each layer
        for (var ndx = 0; ndx < this.m_texSize[2]; ndx++) {
            var layerFbo = ctx.createFramebuffer();
            ctx.bindFramebuffer(gl.FRAMEBUFFER, layerFbo);
            ctx.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex, 0, ndx);
            this.checkError();
            this.checkFramebufferStatus(gl.FRAMEBUFFER);

            fbos.push(layerFbo);
        }

        // Render test images to random texture layers
        var order = [];

        for (var n = 0; n < fbos.length; n++)
            order.push(n);
        rnd.shuffle(order);

        for (var ndx = 0; ndx < 4; ndx++) {
            var layer = order[ndx];
            var format = gl.RGBA;
            var dataType = gl.UNSIGNED_BYTE;
            var texW = 128;
            var texH = 128;
            var fbo = fbos[layer];
            var viewport = this.m_texSize;
            var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

            tcuTextureUtil.fillWithGrid(data.getAccess(), 8, es3fFboColorbufferTests.generateRandomColor(rnd), [0, 0, 0, 0]);

            var tmpTex = ctx.createTexture();
            ctx.bindTexture(gl.TEXTURE_2D, tmpTex);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
            ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

            ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            ctx.viewport(0, 0, viewport[0], viewport[1]);
            rrUtil.drawQuad(ctx, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
            this.checkError();

            // Render to framebuffer
            var p0 = [(ndx % 2) - 1.0, Math.floor(ndx / 2) - 1.0, 0.0];
            var p1 = deMath.add(p0, [1.0, 1.0, 0.0]);
            debug('Layer:' + layer + ' rectangle: ' + p0 + ' ' + p1);

            ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
            ctx.viewport(0, 0, ctx.getWidth(), ctx.getHeight());

            ctx.activeTexture(gl.TEXTURE0);
            ctx.bindTexture(gl.TEXTURE_2D_ARRAY, tex);

            arrayTexShader.setLayer(layer);
            arrayTexShader.setUniforms(ctx, arrayTexShaderID);

            rrUtil.drawQuad(ctx, arrayTexShaderID, p0, p1);
            this.checkError();
        }

        this.readPixels(dst, 0, 0, ctx.getWidth(), ctx.getHeight());
    };

/**
 * @constructor
 * @extends {es3fFboColorbufferTests.FboColorbufferCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} texFmt
 * @param {Array<number>} texSize
 */
es3fFboColorbufferTests.FboColorTex3DCase = function(name, desc, texFmt, texSize) {
    es3fFboColorbufferTests.FboColorbufferCase.call(this, name, desc, texFmt);
    this.m_texSize = texSize;
};

setParentClass(es3fFboColorbufferTests.FboColorTex3DCase, es3fFboColorbufferTests.FboColorbufferCase);

es3fFboColorbufferTests.FboColorTex3DCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    };

    es3fFboColorbufferTests.FboColorTex3DCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_format);
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        var rnd = new deRandom.Random(100);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texToFboShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], es3fFboTestUtil.getFragmentOutputType(texFmt),
            deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin),
            fmtInfo.valueMin);

        /** @type {es3fFboTestUtil.Texture3DShader} */
        var tdTexShader = new es3fFboTestUtil.Texture3DShader(
            gluTextureUtil.getSampler3D(texFmt),
            gluShaderUtil.DataType.FLOAT_VEC4);

        var texToFboShaderID = ctx.createProgram(texToFboShader);
        var tdTexShaderID = ctx.createProgram(tdTexShader);

        // Setup textures
        texToFboShader.setUniforms(ctx, texToFboShaderID);
        tdTexShader.setTexScaleBias(fmtInfo.lookupScale, fmtInfo.lookupBias);

        // Framebuffers.
        var fbos = [];
        var tex;{
            var transferFmt = gluTextureUtil.getTransferFormat(texFmt);
            var size = this.m_texSize;

            tex = ctx.createTexture();

            ctx.bindTexture(gl.TEXTURE_3D, tex);
            ctx.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
            ctx.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
            ctx.texImage3D(gl.TEXTURE_3D, 0, this.m_format, size[0], size[1], size[2], 0, transferFmt.format, transferFmt.dataType, null);

            // Generate an FBO for each layer
            for (var ndx = 0; ndx < this.m_texSize[2]; ndx++) {
                var layerFbo = ctx.createFramebuffer();
                ctx.bindFramebuffer(gl.FRAMEBUFFER, layerFbo);
                ctx.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex, 0, ndx);
                this.checkError();
                this.checkFramebufferStatus(gl.FRAMEBUFFER);

                fbos.push(layerFbo);
            }
        }

        // Render test images to random texture layers
        var order = [];

        for (var n = 0; n < fbos.length; n++)
            order.push(n);
        rnd.shuffle(order);

        for (var ndx = 0; ndx < 4; ndx++) {
            var layer = order[ndx];
            var format = gl.RGBA;
            var dataType = gl.UNSIGNED_BYTE;
            var texW = 128;
            var texH = 128;
            var fbo = fbos[layer];
            var viewport = this.m_texSize;
            var data = new tcuTexture.TextureLevel(gluTextureUtil.mapGLTransferFormat(format, dataType), texW, texH, 1);

            tcuTextureUtil.fillWithGrid(data.getAccess(), 8, es3fFboColorbufferTests.generateRandomColor(rnd), [0, 0, 0, 0]);

            var tmpTex = ctx.createTexture();
            ctx.bindTexture(gl.TEXTURE_2D, tmpTex);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            ctx.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
            ctx.texImage2D(gl.TEXTURE_2D, 0, format, texW, texH, 0, format, dataType, data.getAccess().getDataPtr());

            ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            ctx.viewport(0, 0, viewport[0], viewport[1]);
            rrUtil.drawQuad(ctx, texToFboShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);
            this.checkError();

            // Render to framebuffer
            var p0 = [(ndx % 2) - 1.0, Math.floor(ndx / 2) - 1.0, 0.0];
            var p1 = deMath.add(p0, [1.0, 1.0, 0.0]);
            debug('Layer:' + layer + ' rectangle: ' + p0 + ' ' + p1);

            ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
            ctx.viewport(0, 0, ctx.getWidth(), ctx.getHeight());

            ctx.activeTexture(gl.TEXTURE0);
            ctx.bindTexture(gl.TEXTURE_3D, tex);

            tdTexShader.setDepth(layer / (this.m_texSize[2] - 1));
            tdTexShader.setUniforms(ctx, tdTexShaderID);

            rrUtil.drawQuad(ctx, tdTexShaderID, p0, p1);
            this.checkError();
        }

        this.readPixels(dst, 0, 0, ctx.getWidth(), ctx.getHeight());
};

/**
 * @constructor
 * @extends {es3fFboColorbufferTests.FboColorbufferCase}
 * @param {string} name
 * @param {string} desc
 * @param {number} format
 * @param {Array<number>} size
 * @param {number} funcRGB
 * @param {number} funcAlpha
 * @param {number} srcRGB
 * @param {number} dstRGB
 * @param {number} srcAlpha
 * @param {number} dstAlpha
 */
es3fFboColorbufferTests.FboBlendCase = function(name, desc, format, size, funcRGB, funcAlpha, srcRGB, dstRGB, srcAlpha, dstAlpha) {
    es3fFboColorbufferTests.FboColorbufferCase.call(this, name, desc, format);
    this.m_size = size;
    this.m_funcRGB = funcRGB;
    this.m_funcAlpha = funcAlpha;
    this.m_srcRGB = srcRGB;
    this.m_dstRGB = dstRGB;
    this.m_srcAlpha = srcAlpha;
    this.m_dstAlpha = dstAlpha
};

setParentClass(es3fFboColorbufferTests.FboBlendCase, es3fFboColorbufferTests.FboColorbufferCase);

es3fFboColorbufferTests.FboBlendCase.prototype.preCheck = function() {
        this.checkFormatSupport(this.m_format);
        return true; // No exception thrown
    }

    es3fFboColorbufferTests.FboBlendCase.prototype.render = function(dst) {
        // \note Assumes floating-point or fixed-point format.
        var ctx = this.getCurrentContext();
        var fboFmt = gluTextureUtil.mapGLInternalFormat(this.m_format);
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(fboFmt);

        /** @type {es3fFboTestUtil.Texture2DShader} */
        var texShader = new es3fFboTestUtil.Texture2DShader(
            [gluShaderUtil.DataType.SAMPLER_2D], gluShaderUtil.DataType.FLOAT_VEC4);

        /** @type {es3fFboTestUtil.GradientShader} */
        var gradShader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);

        var texShaderID = ctx.createProgram(texShader);
        var gradShaderID = ctx.createProgram(gradShader);

        // Setup shaders
        texShader.setUniforms (ctx, texShaderID);
        gradShader.setGradient(ctx, gradShaderID, [0, 0, 0, 0], [1, 1, 1, 1]);

        var fbo = ctx.createFramebuffer();
        var rbo = ctx.createRenderbuffer();

        ctx.bindRenderbuffer(gl.RENDERBUFFER, rbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, this.m_format, this.m_size[0], this.m_size[1]);
        this.checkError();

        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo);
        this.checkError();
        this.checkFramebufferStatus(gl.FRAMEBUFFER);

        ctx.viewport(0, 0, this.m_size[0], this.m_size[1]);

        // Fill framebuffer with grid pattern.
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

        // Setup blend.
        ctx.enable(gl.BLEND);
        ctx.blendEquationSeparate(this.m_funcRGB, this.m_funcAlpha);
        ctx.blendFuncSeparate(this.m_srcRGB, this.m_dstRGB, this.m_srcAlpha, this.m_dstAlpha);

        // Render gradient with blend.
        rrUtil.drawQuad(ctx, gradShaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]);

        es3fFboTestUtil.readPixels(ctx, dst, 0, 0, this.m_size[0], this.m_size[1], fboFmt, [1, 1, 1, 1], [0, 0, 0, 0]);
    };

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fFboColorbufferTests.FboColorbufferTests = function() {
    tcuTestCase.DeqpTest.call(this, 'color', 'Colorbuffer tests');
};

setParentClass(es3fFboColorbufferTests.FboColorbufferTests, tcuTestCase.DeqpTest);

es3fFboColorbufferTests.FboColorbufferTests.prototype.init = function() {
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
        gl.R16F,

        // gl.EXT_color_buffer_half_float is not exposed in WebGL 2.0.
        // gl.RGB16F
    ];

    // .clear
    var clearGroup = tcuTestCase.newTest("clear", "Color clears");
    this.addChild(clearGroup);

    for (var ndx = 0; ndx < colorFormats.length; ndx++) {
        clearGroup.addChild(new es3fFboColorbufferTests.FboColorClearCase(
            es3fFboTestUtil.getFormatName(colorFormats[ndx]), "", colorFormats[ndx], 129, 117));
    }

    var numGroups = 6;

    // .tex2d
    var tex2DGroup = [];
    for (var ii = 0; ii < numGroups; ++ii) {
        tex2DGroup[ii] = tcuTestCase.newTest("tex2d", "Texture 2D tests");
        this.addChild(tex2DGroup[ii]);
    }
    for (var ndx = 0; ndx < colorFormats.length; ndx++) {
        tex2DGroup[ndx % numGroups].addChild(new es3fFboColorbufferTests.FboColorMultiTex2DCase(
            es3fFboTestUtil.getFormatName(colorFormats[ndx]), "", colorFormats[ndx], [129, 117], colorFormats[ndx], [99, 128]));
    }

    // .texcube
    var texCubeGroup = [];
    for (var ii = 0; ii < numGroups; ++ii) {
        texCubeGroup[ii] = tcuTestCase.newTest("texcube", "Texture cube map tests");
        this.addChild(texCubeGroup[ii]);
    }
    for (var ndx = 0; ndx < colorFormats.length; ndx++) {
        texCubeGroup[ndx % numGroups].addChild(new es3fFboColorbufferTests.FboColorTexCubeCase(
            es3fFboTestUtil.getFormatName(colorFormats[ndx]), "", colorFormats[ndx], [128, 128]));
    }

    // .tex2darray
    var tex2DArrayGroup = [];
    for (var ii = 0; ii < numGroups; ++ii) {
        tex2DArrayGroup[ii] = tcuTestCase.newTest("tex2darray", "Texture 2D array tests");
        this.addChild(tex2DArrayGroup[ii]);
    }
    for (var ndx = 0; ndx < colorFormats.length; ndx++) {
        tex2DArrayGroup[ndx % numGroups].addChild(new es3fFboColorbufferTests.FboColorTex2DArrayCase(
            es3fFboTestUtil.getFormatName(colorFormats[ndx]), "", colorFormats[ndx], [128, 128, 5]));
    }

    // .tex3d
    var tex3DGroup = [];
    for (var ii = 0; ii < numGroups; ++ii) {
        tex3DGroup[ii] = tcuTestCase.newTest("tex3d", "Texture 3D tests");
        this.addChild(tex3DGroup[ii]);
    }
    for (var ndx = 0; ndx < colorFormats.length; ndx++) {
        tex3DGroup[ndx % numGroups].addChild(new es3fFboColorbufferTests.FboColorTex3DCase(
            es3fFboTestUtil.getFormatName(colorFormats[ndx]), "", colorFormats[ndx], [128, 128, 5]));
    }

    // .blend
    var blendGroup = tcuTestCase.newTest("blend", "Blending tests");
    this.addChild(blendGroup);

    for (var ndx = 0; ndx < colorFormats.length; ndx++) {
        var format = colorFormats[ndx];
        var texFmt = gluTextureUtil.mapGLInternalFormat(format);
        var fmtClass = tcuTexture.getTextureChannelClass(texFmt.type);
        var fmtName = es3fFboTestUtil.getFormatName(format);

        if (texFmt.type == tcuTexture.ChannelType.FLOAT ||
            fmtClass == tcuTexture.TextureChannelClass.SIGNED_INTEGER ||
            fmtClass == tcuTexture.TextureChannelClass.UNSIGNED_INTEGER)
            continue; // Blending is not supported.

        blendGroup.addChild(new es3fFboColorbufferTests.FboBlendCase(fmtName + "_src_over", "", format,
            [127, 111], gl.FUNC_ADD, gl.FUNC_ADD, gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ZERO, gl.ONE));
    }
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fFboColorbufferTests.run = function(context, range) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fFboColorbufferTests.FboColorbufferTests());

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
        testFailedOptions('Failed to es3fFboColorbufferTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

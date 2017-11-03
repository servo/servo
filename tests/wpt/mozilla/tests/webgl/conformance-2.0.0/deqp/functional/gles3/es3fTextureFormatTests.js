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
goog.provide('functional.gles3.es3fTextureFormatTests');
goog.require('framework.common.tcuCompressedTexture');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluStrUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluTextureUtil');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {

var es3fTextureFormatTests = functional.gles3.es3fTextureFormatTests;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var deRandom = framework.delibs.debase.deRandom;
var tcuTestCase = framework.common.tcuTestCase;
var tcuSurface = framework.common.tcuSurface;
var gluTexture = framework.opengl.gluTexture;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var gluStrUtil = framework.opengl.gluStrUtil;
var deMath = framework.delibs.debase.deMath;
var tcuCompressedTexture = framework.common.tcuCompressedTexture;

/** @type {WebGL2RenderingContext} */ var gl;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

es3fTextureFormatTests.version = '300 es';

es3fTextureFormatTests.testDescription = function() {
    var test = tcuTestCase.runner.currentTest;
    return test.description;
};

es3fTextureFormatTests.setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fTextureFormatTests.Texture2DFormatCase = function(descriptor) {
    tcuTestCase.DeqpTest.call(this, descriptor.name, descriptor.description);
    this.m_format = descriptor.format;
    this.m_dataType = descriptor.dataType;
    this.m_width = descriptor.width;
    this.m_height = descriptor.height;
    this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureFormatTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
};

es3fTextureFormatTests.setParentClass(es3fTextureFormatTests.Texture2DFormatCase, tcuTestCase.DeqpTest);

es3fTextureFormatTests.Texture2DFormatCase.prototype.init = function() {
    /*tcu::TextureFormat*/ var fmt = this.m_dataType ? gluTextureUtil.mapGLTransferFormat(this.m_format, this.m_dataType) : gluTextureUtil.mapGLInternalFormat(this.m_format);
    /*tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(fmt);
    /* TODO : Port

    std::ostringstream fmtName;

    if (m_dataType)
        fmtName << glu::getPixelFormatStr(m_format) << ", " << glu::getTypeStr(m_dataType);
    else
        fmtName << glu::getPixelFormatStr(m_format);

    log << TestLog::Message << "2D texture, " << fmtName.str() << ", " << m_width << "x" << m_height
                            << ",\n fill with " << formatGradient(&spec.valueMin, &spec.valueMax) << " gradient"
        << TestLog::EndMessage;
    */

    this.m_texture = this.m_dataType ?
              gluTexture.texture2DFromFormat(gl, this.m_format, this.m_dataType, this.m_width, this.m_height) : // Implicit internal format.
              gluTexture.texture2DFromInternalFormat(gl, this.m_format, this.m_width, this.m_height); // Explicit internal format.

    // Fill level 0.
    this.m_texture.getRefTexture().allocLevel(0);
    tcuTextureUtil.fillWithComponentGradients(this.m_texture.getRefTexture().getLevel(0), spec.valueMin, spec.valueMax);
};

es3fTextureFormatTests.Texture2DFormatCase.prototype.deinit = function() {
    /* TODO: Implement */
};

es3fTextureFormatTests.Texture2DFormatCase.prototype.iterate = function() {
    /* TODO: Implement */

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), this.m_width, this.m_height/*, deStringHash(getName())*/);

    /* tcu::Surface */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* tcu::Surface */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* TODO: Implement
    // tcu::RGBA threshold = m_renderCtx.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(1,1,1,1);
    */
    var threshold = [3, 3, 3, 3];
    var renderParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_2D);

    /* tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(this.m_texture.getRefTexture().getFormat());
    /** @const */ var wrapS = gl.CLAMP_TO_EDGE;
    /** @const */ var wrapT = gl.CLAMP_TO_EDGE;
    /** @const */ var minFilter = gl.NEAREST;
    /** @const */ var magFilter = gl.NEAREST;

    renderParams.flags.log_programs = true;
    renderParams.flags.log_uniforms = true;

    renderParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
    renderParams.sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
     tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST);
    renderParams.colorScale = spec.lookupScale;
    renderParams.colorBias = spec.lookupBias;

    var texCoord = glsTextureTestUtil.computeQuadTexCoord2D([0, 0], [1, 1]);

    // log << TestLog::Message << "Texture parameters:"
    // << "\n WRAP_S = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_S, wrapS)
    // << "\n WRAP_T = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_T, wrapT)
    // << "\n MIN_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MIN_FILTER, minFilter)
    // << "\n MAG_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MAG_FILTER, magFilter)
    // << TestLog::EndMessage;

    // Setup base viewport.
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    // Upload texture data to GL.
    this.m_texture.upload();

    // Bind to unit 0.
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.m_texture.getGLTexture());

    // Setup nearest neighbor filtering and clamp-to-edge.
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, minFilter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, magFilter);

    // // Draw.
    this.m_renderer.renderQuad(0, texCoord, renderParams);
    renderedFrame.readViewport(gl, viewport);

    // // Compute reference.
    glsTextureTestUtil.sampleTexture2D(new glsTextureTestUtil.SurfaceAccess(referenceFrame, undefined /*m_renderCtx.getRenderTarget().getPixelFormat()*/),
        this.m_texture.getRefTexture(), texCoord, renderParams);

    // Compare and log.
    var isOk = glsTextureTestUtil.compareImages(referenceFrame, renderedFrame, threshold);

    assertMsgOptions(isOk, es3fTextureFormatTests.testDescription(), true, false);
    return tcuTestCase.IterateResult.STOP;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fTextureFormatTests.TextureCubeFormatCase = function(descriptor) {
    tcuTestCase.DeqpTest.call(this, descriptor.name, descriptor.description);
    this.m_format = descriptor.format;
    this.m_dataType = descriptor.dataType;
    this.m_width = descriptor.width;
    this.m_height = descriptor.height;
    this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureFormatTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
    DE_ASSERT(this.m_width == this.m_height);
};

es3fTextureFormatTests.setParentClass(es3fTextureFormatTests.TextureCubeFormatCase, tcuTestCase.DeqpTest);

es3fTextureFormatTests.TextureCubeFormatCase.prototype.init = function() {
    /*tcu::TextureFormat*/ var fmt = this.m_dataType ? gluTextureUtil.mapGLTransferFormat(this.m_format, this.m_dataType) : gluTextureUtil.mapGLInternalFormat(this.m_format);
    /*tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(fmt);
    /* TODO : Port

    std::ostringstream fmtName;

    if (m_dataType)
        fmtName << glu::getPixelFormatStr(m_format) << ", " << glu::getTypeStr(m_dataType);
    else
        fmtName << glu::getPixelFormatStr(m_format);

    log << TestLog::Message << "2D texture, " << fmtName.str() << ", " << m_width << "x" << m_height
                            << ",\n fill with " << formatGradient(&spec.valueMin, &spec.valueMax) << " gradient"
        << TestLog::EndMessage;
    */

    this.m_texture = this.m_dataType ?
              gluTexture.cubeFromFormat(gl, this.m_format, this.m_dataType, this.m_width) : // Implicit internal format.
              gluTexture.cubeFromInternalFormat(gl, this.m_format, this.m_width); // Explicit internal format.

    // Fill level 0.
    for (var face in tcuTexture.CubeFace) {
        var gMin = null;
        var gMax = null;

        switch (tcuTexture.CubeFace[face]) {
            case 0: gMin = deMath.swizzle(spec.valueMin, [0, 1, 2, 3]); gMax = deMath.swizzle(spec.valueMax, [0, 1, 2, 3]); break;
            case 1: gMin = deMath.swizzle(spec.valueMin, [2, 1, 0, 3]); gMax = deMath.swizzle(spec.valueMax, [2, 1, 0, 3]); break;
            case 2: gMin = deMath.swizzle(spec.valueMin, [1, 2, 0, 3]); gMax = deMath.swizzle(spec.valueMax, [1, 2, 0, 3]); break;
            case 3: gMin = deMath.swizzle(spec.valueMax, [0, 1, 2, 3]); gMax = deMath.swizzle(spec.valueMin, [0, 1, 2, 3]); break;
            case 4: gMin = deMath.swizzle(spec.valueMax, [2, 1, 0, 3]); gMax = deMath.swizzle(spec.valueMin, [2, 1, 0, 3]); break;
            case 5: gMin = deMath.swizzle(spec.valueMax, [1, 2, 0, 3]); gMax = deMath.swizzle(spec.valueMin, [1, 2, 0, 3]); break;
            default:
                DE_ASSERT(false);
        }

        this.m_texture.getRefTexture().allocLevel(tcuTexture.CubeFace[face], 0);
        tcuTextureUtil.fillWithComponentGradients(this.m_texture.getRefTexture().getLevelFace(0, tcuTexture.CubeFace[face]), gMin, gMax);
    }

    this.m_texture.upload();
    this.m_curFace = 0;
    this.m_isOk = true;
};

es3fTextureFormatTests.TextureCubeFormatCase.prototype.testFace = function(face) {
    /* TODO: Implement */

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), this.m_width, this.m_height/*, deStringHash(getName())*/);

    /* tcu::Surface */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* tcu::Surface */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* TODO: Implement
    // tcu::RGBA threshold = m_renderCtx.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(1,1,1,1);
    */
    var threshold = [3, 3, 3, 3];
    var renderParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_CUBE);

    /* tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(this.m_texture.getRefTexture().getFormat());
    /** @const */ var wrapS = gl.CLAMP_TO_EDGE;
    /** @const */ var wrapT = gl.CLAMP_TO_EDGE;
    /** @const */ var minFilter = gl.NEAREST;
    /** @const */ var magFilter = gl.NEAREST;

    renderParams.flags.log_programs = true;
    renderParams.flags.log_uniforms = true;

    renderParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
    renderParams.sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
     tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST);
    renderParams.colorScale = spec.lookupScale;
    renderParams.colorBias = spec.lookupBias;

    // Log render info on first face.
    if (face === tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X) {
        renderParams.flags.log_programs = true;
        renderParams.flags.log_uniforms = true;
    }

    var texCoord = glsTextureTestUtil.computeQuadTexCoordCube(face);

    // log << TestLog::Message << "Texture parameters:"
    // << "\n WRAP_S = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_S, wrapS)
    // << "\n WRAP_T = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_T, wrapT)
    // << "\n MIN_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MIN_FILTER, minFilter)
    // << "\n MAG_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MAG_FILTER, magFilter)
    // << TestLog::EndMessage;

    // Setup base viewport.
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    // Bind to unit 0.
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_CUBE_MAP, this.m_texture.getGLTexture());

    // Setup nearest neighbor filtering and clamp-to-edge.
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, minFilter);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, magFilter);

    // // Draw.
    this.m_renderer.renderQuad(0, texCoord, renderParams);
    renderedFrame.readViewport(gl, viewport);

    // // Compute reference.
    glsTextureTestUtil.sampleTextureCube(new glsTextureTestUtil.SurfaceAccess(referenceFrame, undefined /*m_renderCtx.getRenderTarget().getPixelFormat()*/),
        this.m_texture.getRefTexture(), texCoord, renderParams);

    // Compare and log.
    var skipPixels = null;
    if (renderParams.samplerType == glsTextureTestUtil.samplerType.SAMPLERTYPE_INT ||
        renderParams.samplerType == glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT) {
        // Skip top right pixel due to Mac Intel driver bug.
        // https://github.com/KhronosGroup/WebGL/issues/1819
        skipPixels = [
            [this.m_width - 1, this.m_height - 1]
        ];
    }
    var isOk = glsTextureTestUtil.compareImages(referenceFrame, renderedFrame, threshold, skipPixels);

    assertMsgOptions(isOk, 'Face: ' + this.m_curFace + ' ' + es3fTextureFormatTests.testDescription(), true, false);
    return isOk;
};

es3fTextureFormatTests.TextureCubeFormatCase.prototype.iterate = function() {
    debug('Testing face ' + this.m_curFace);
    // Execute test for all faces.
    if (!this.testFace(this.m_curFace))
        this.m_isOk = false;

    this.m_curFace += 1;

    if (this.m_curFace < Object.keys(tcuTexture.CubeFace).length)
        return tcuTestCase.IterateResult.CONTINUE;
    else
        return tcuTestCase.IterateResult.STOP;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fTextureFormatTests.Texture2DArrayFormatCase = function(descriptor) {
    tcuTestCase.DeqpTest.call(this, descriptor.name, descriptor.description);
    this.m_format = descriptor.format;
    this.m_dataType = descriptor.dataType;
    this.m_width = descriptor.width;
    this.m_height = descriptor.height;
    this.m_numLayers = descriptor.numLayers;
    this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureFormatTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
};

es3fTextureFormatTests.setParentClass(es3fTextureFormatTests.Texture2DArrayFormatCase, tcuTestCase.DeqpTest);

es3fTextureFormatTests.Texture2DArrayFormatCase.prototype.init = function() {
    /*tcu::TextureFormat*/ var fmt = this.m_dataType ? gluTextureUtil.mapGLTransferFormat(this.m_format, this.m_dataType) : gluTextureUtil.mapGLInternalFormat(this.m_format);
    /*tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(fmt);
    /* TODO : Port

    std::ostringstream fmtName;

    if (m_dataType)
        fmtName << glu::getPixelFormatStr(m_format) << ", " << glu::getTypeStr(m_dataType);
    else
        fmtName << glu::getPixelFormatStr(m_format);

    log << TestLog::Message << "2D texture, " << fmtName.str() << ", " << m_width << "x" << m_height
                            << ",\n fill with " << formatGradient(&spec.valueMin, &spec.valueMax) << " gradient"
        << TestLog::EndMessage;
    */

    this.m_texture = this.m_dataType ?
              gluTexture.texture2DArrayFromFormat(gl, this.m_format, this.m_dataType, this.m_width, this.m_height, this.m_numLayers) : // Implicit internal format.
              gluTexture.texture2DArrayFromInternalFormat(gl, this.m_format, this.m_width, this.m_height, this.m_numLayers); // Explicit internal format.

    this.m_texture.getRefTexture().allocLevel(0);
    tcuTextureUtil.fillWithComponentGradients(this.m_texture.getRefTexture().getLevel(0), spec.valueMin, spec.valueMax);

    this.m_curLayer = 0;
    this.m_isOk = true;
};

es3fTextureFormatTests.Texture2DArrayFormatCase.prototype.testLayer = function(layerNdx) {
    /* TODO: Implement */

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), this.m_width, this.m_height/*, deStringHash(getName())*/);

    /* tcu::Surface */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* tcu::Surface */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* TODO: Implement
    // tcu::RGBA threshold = m_renderCtx.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(1,1,1,1);
    */
    var threshold = [3, 3, 3, 3];
    var renderParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_2D_ARRAY);

    /* tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(this.m_texture.getRefTexture().getFormat());
    /** @const */ var wrapS = gl.CLAMP_TO_EDGE;
    /** @const */ var wrapT = gl.CLAMP_TO_EDGE;
    /** @const */ var minFilter = gl.NEAREST;
    /** @const */ var magFilter = gl.NEAREST;

    renderParams.flags.log_programs = true;
    renderParams.flags.log_uniforms = true;

    renderParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
    renderParams.sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
     tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST);
    renderParams.colorScale = spec.lookupScale;
    renderParams.colorBias = spec.lookupBias;

    var texCoord = glsTextureTestUtil.computeQuadTexCoord2DArray(layerNdx, [0, 0], [1, 1]);

    // log << TestLog::Message << "Texture parameters:"
    // << "\n WRAP_S = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_S, wrapS)
    // << "\n WRAP_T = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_T, wrapT)
    // << "\n MIN_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MIN_FILTER, minFilter)
    // << "\n MAG_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MAG_FILTER, magFilter)
    // << TestLog::EndMessage;

    // Setup base viewport.
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    this.m_texture.upload();

    // Bind to unit 0.
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.m_texture.getGLTexture());

    // Setup nearest neighbor filtering and clamp-to-edge.
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, minFilter);
    gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, magFilter);

    // // Draw.
    this.m_renderer.renderQuad(0, texCoord, renderParams);
    renderedFrame.readViewport(gl, viewport);

    // // Compute reference.
    glsTextureTestUtil.sampleTexture2DArray(new glsTextureTestUtil.SurfaceAccess(referenceFrame, undefined /*m_renderCtx.getRenderTarget().getPixelFormat()*/),
        this.m_texture.getRefTexture().getView(), texCoord, renderParams);

    // Compare and log.
    var isOk = glsTextureTestUtil.compareImages(referenceFrame, renderedFrame, threshold);

    assertMsgOptions(isOk, 'Layer: ' + this.m_curLayer + ' ' + es3fTextureFormatTests.testDescription(), true, false);
    return isOk;
};

es3fTextureFormatTests.Texture2DArrayFormatCase.prototype.iterate = function() {
    debug('Testing layer ' + this.m_curLayer);
    // Execute test for all layers.
    if (!this.testLayer(this.m_curLayer))
        this.m_isOk = false;

    this.m_curLayer += 1;

    if (this.m_curLayer == this.m_numLayers)
        return tcuTestCase.IterateResult.STOP;
    else
        return tcuTestCase.IterateResult.CONTINUE;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fTextureFormatTests.Texture3DFormatCase = function(descriptor) {
    tcuTestCase.DeqpTest.call(this, descriptor.name, descriptor.description);
    this.m_format = descriptor.format;
    this.m_dataType = descriptor.dataType;
    this.m_width = descriptor.width;
    this.m_height = descriptor.height;
    this.m_depth = descriptor.depth;
    this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureFormatTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
};

es3fTextureFormatTests.setParentClass(es3fTextureFormatTests.Texture3DFormatCase, tcuTestCase.DeqpTest);

es3fTextureFormatTests.Texture3DFormatCase.prototype.init = function() {
    /*tcu::TextureFormat*/ var fmt = this.m_dataType ? gluTextureUtil.mapGLTransferFormat(this.m_format, this.m_dataType) : gluTextureUtil.mapGLInternalFormat(this.m_format);
    /*tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(fmt);
    /* TODO : Port

    std::ostringstream fmtName;

    if (m_dataType)
        fmtName << glu::getPixelFormatStr(m_format) << ", " << glu::getTypeStr(m_dataType);
    else
        fmtName << glu::getPixelFormatStr(m_format);

    log << TestLog::Message << "2D texture, " << fmtName.str() << ", " << m_width << "x" << m_height
                            << ",\n fill with " << formatGradient(&spec.valueMin, &spec.valueMax) << " gradient"
        << TestLog::EndMessage;
    */

    this.m_texture = this.m_dataType ?
              gluTexture.texture3DFromFormat(gl, this.m_format, this.m_dataType, this.m_width, this.m_height, this.m_depth) : // Implicit internal format.
              gluTexture.texture3DFromInternalFormat(gl, this.m_format, this.m_width, this.m_height, this.m_depth); // Explicit internal format.

    this.m_texture.getRefTexture().allocLevel(0);
    tcuTextureUtil.fillWithComponentGradients(this.m_texture.getRefTexture().getLevel(0), spec.valueMin, spec.valueMax);

    this.m_curSlice = 0;
    this.m_isOk = true;
};

es3fTextureFormatTests.Texture3DFormatCase.prototype.testSlice = function(sliceNdx) {
    /* TODO: Implement */

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), this.m_width, this.m_height/*, deStringHash(getName())*/);

    /* tcu::Surface */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* tcu::Surface */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* TODO: Implement
    // tcu::RGBA threshold = m_renderCtx.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(1,1,1,1);
    */
    var threshold = [3, 3, 3, 3];
    var renderParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_3D);

    /* tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(this.m_texture.getRefTexture().getFormat());
    var r = (sliceNdx + 0.5) / this.m_depth;
    /** @const */ var wrapS = gl.CLAMP_TO_EDGE;
    /** @const */ var wrapT = gl.CLAMP_TO_EDGE;
    /** @const */ var minFilter = gl.NEAREST;
    /** @const */ var magFilter = gl.NEAREST;

    renderParams.flags.log_programs = true;
    renderParams.flags.log_uniforms = true;

    renderParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
    renderParams.sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
     tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST);
    renderParams.colorScale = spec.lookupScale;
    renderParams.colorBias = spec.lookupBias;

    var texCoord = glsTextureTestUtil.computeQuadTexCoord3D([0, 0, r], [1, 1, r], [0, 1, 2]);

    // log << TestLog::Message << "Texture parameters:"
    // << "\n WRAP_S = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_S, wrapS)
    // << "\n WRAP_T = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_T, wrapT)
    // << "\n MIN_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MIN_FILTER, minFilter)
    // << "\n MAG_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MAG_FILTER, magFilter)
    // << TestLog::EndMessage;

    // Setup base viewport.
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    this.m_texture.upload();

    // Bind to unit 0.
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_3D, this.m_texture.getGLTexture());

    // Setup nearest neighbor filtering and clamp-to-edge.
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, minFilter);
    gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, magFilter);

    // // Draw.
    this.m_renderer.renderQuad(0, texCoord, renderParams);
    renderedFrame.readViewport(gl, viewport);

    // // Compute reference.
    glsTextureTestUtil.sampleTexture3D(new glsTextureTestUtil.SurfaceAccess(referenceFrame, undefined /*m_renderCtx.getRenderTarget().getPixelFormat()*/),
        this.m_texture.getRefTexture(), texCoord, renderParams);

    // Compare and log.
    var isOk = glsTextureTestUtil.compareImages(referenceFrame, renderedFrame, threshold);

    assertMsgOptions(isOk, 'Slice: ' + this.m_curSlice + ' ' + es3fTextureFormatTests.testDescription(), true, false);
    return isOk;
};

es3fTextureFormatTests.Texture3DFormatCase.prototype.iterate = function() {
    debug('Testing slice ' + this.m_curSlice);
    // Execute test for all layers.
    if (!this.testSlice(this.m_curSlice))
        this.m_isOk = false;

    this.m_curSlice += 1;

    if (this.m_curSlice >= this.m_depth)
        return tcuTestCase.IterateResult.STOP;
    else
        return tcuTestCase.IterateResult.CONTINUE;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fTextureFormatTests.Compressed2DFormatCase = function(descriptor) {
    tcuTestCase.DeqpTest.call(this, descriptor.name, descriptor.description);
    this.m_format = descriptor.format;
    this.m_dataType = descriptor.dataType;
    this.m_width = descriptor.width;
    this.m_height = descriptor.height;
    this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureFormatTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
};

es3fTextureFormatTests.setParentClass(es3fTextureFormatTests.Compressed2DFormatCase, tcuTestCase.DeqpTest);

es3fTextureFormatTests.Compressed2DFormatCase.prototype.init = function() {
    var compressed = new tcuCompressedTexture.CompressedTexture(this.m_format, this.m_width, this.m_height);
    var rand = new deRandom.Random(0);
    for (var i = 0; i < compressed.m_data.length; i++) {
        compressed.m_data[i] = rand.getInt(0, 255);
    }
    this.m_texture = gluTexture.compressed2DFromInternalFormat(gl, this.m_format, this.m_width, this.m_height, compressed);
};

es3fTextureFormatTests.Compressed2DFormatCase.prototype.deinit = function() {
    /* TODO: Implement */
};

es3fTextureFormatTests.Compressed2DFormatCase.prototype.iterate = function() {
    /* TODO: Implement */

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), this.m_width, this.m_height/*, deStringHash(getName())*/);

    /* tcu::Surface */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* tcu::Surface */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* TODO: Implement
    // tcu::RGBA threshold = m_renderCtx.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(1,1,1,1);
    */
    var threshold = [3, 3, 3, 3];
    var renderParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_2D);

    /* tcu::TextureFormatInfo*/ var spec = tcuTextureUtil.getTextureFormatInfo(this.m_texture.getRefTexture().getFormat());
    /** @const */ var wrapS = gl.CLAMP_TO_EDGE;
    /** @const */ var wrapT = gl.CLAMP_TO_EDGE;
    /** @const */ var minFilter = gl.NEAREST;
    /** @const */ var magFilter = gl.NEAREST;

    renderParams.flags.log_programs = true;
    renderParams.flags.log_uniforms = true;

    renderParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
    renderParams.sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
     tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST);
    renderParams.colorScale = spec.lookupScale;
    renderParams.colorBias = spec.lookupBias;

    var texCoord = glsTextureTestUtil.computeQuadTexCoord2D([0, 0], [1, 1]);

    // log << TestLog::Message << "Texture parameters:"
    // << "\n WRAP_S = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_S, wrapS)
    // << "\n WRAP_T = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_T, wrapT)
    // << "\n MIN_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MIN_FILTER, minFilter)
    // << "\n MAG_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MAG_FILTER, magFilter)
    // << TestLog::EndMessage;

    // Setup base viewport.
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    // Bind to unit 0.
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.m_texture.getGLTexture());

    // Setup nearest neighbor filtering and clamp-to-edge.
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, minFilter);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, magFilter);

    // // Draw.
    this.m_renderer.renderQuad(0, texCoord, renderParams);
    renderedFrame.readViewport(gl, viewport);

    // // Compute reference.
    glsTextureTestUtil.sampleTexture2D(new glsTextureTestUtil.SurfaceAccess(referenceFrame, undefined /*m_renderCtx.getRenderTarget().getPixelFormat()*/),
        this.m_texture.getRefTexture(), texCoord, renderParams);

    // Compare and log.
    var isOk = glsTextureTestUtil.compareImages(referenceFrame, renderedFrame, threshold);

    assertMsgOptions(isOk, es3fTextureFormatTests.testDescription(), true, false);
    return tcuTestCase.IterateResult.STOP;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fTextureFormatTests.CompressedCubeFormatCase = function(descriptor) {
    tcuTestCase.DeqpTest.call(this, descriptor.name, descriptor.description);
    this.m_format = descriptor.format;
    this.m_dataType = descriptor.dataType;
    this.m_width = descriptor.width;
    this.m_height = descriptor.height;
    this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureFormatTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
    this.m_curFace = 0;
    this.m_isOk = true;
    DE_ASSERT(this.m_width == this.m_height);
};

es3fTextureFormatTests.setParentClass(es3fTextureFormatTests.CompressedCubeFormatCase, tcuTestCase.DeqpTest);

es3fTextureFormatTests.CompressedCubeFormatCase.prototype.init = function() {
    var compressed = new tcuCompressedTexture.CompressedTexture(this.m_format, this.m_width, this.m_height);
    var rand = new deRandom.Random(0);
    for (var i = 0; i < compressed.m_data.length; i++) {
        compressed.m_data[i] = rand.getInt(0, 255);
    }
    this.m_texture = gluTexture.compressedCubeFromInternalFormat(gl, this.m_format, this.m_width, compressed);
};

es3fTextureFormatTests.CompressedCubeFormatCase.prototype.testFace = function(face) {
    /* TODO: Implement */

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), this.m_width, this.m_height/*, deStringHash(getName())*/);

    /* tcu::Surface */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* tcu::Surface */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
    /* TODO: Implement
    // tcu::RGBA threshold = m_renderCtx.getRenderTarget().getPixelFormat().getColorThreshold() + tcu::RGBA(1,1,1,1);
    */
    // Threshold high enough to cover numerical errors in software decoders on Windows and Mac.  Threshold is 17 in native dEQP.
    var threshold = [6, 6, 6, 6];
    var renderParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_CUBE);

    /** @const */ var wrapS = gl.CLAMP_TO_EDGE;
    /** @const */ var wrapT = gl.CLAMP_TO_EDGE;
    /** @const */ var minFilter = gl.NEAREST;
    /** @const */ var magFilter = gl.NEAREST;

    renderParams.flags.log_programs = true;
    renderParams.flags.log_uniforms = true;

    renderParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
    renderParams.sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
     tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST);

    // Log render info on first face.
    if (face === tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X) {
        renderParams.flags.log_programs = true;
        renderParams.flags.log_uniforms = true;
    }

    var texCoord = glsTextureTestUtil.computeQuadTexCoordCube(face);

    // log << TestLog::Message << "Texture parameters:"
    // << "\n WRAP_S = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_S, wrapS)
    // << "\n WRAP_T = " << glu::getTextureParameterValueStr(gl.TEXTURE_WRAP_T, wrapT)
    // << "\n MIN_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MIN_FILTER, minFilter)
    // << "\n MAG_FILTER = " << glu::getTextureParameterValueStr(gl.TEXTURE_MAG_FILTER, magFilter)
    // << TestLog::EndMessage;

    // Setup base viewport.
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    // Bind to unit 0.
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_CUBE_MAP, this.m_texture.getGLTexture());

    // Setup nearest neighbor filtering and clamp-to-edge.
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, minFilter);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, magFilter);

    // // Draw.
    this.m_renderer.renderQuad(0, texCoord, renderParams);
    renderedFrame.readViewport(gl, viewport);

    // // Compute reference.
    glsTextureTestUtil.sampleTextureCube(new glsTextureTestUtil.SurfaceAccess(referenceFrame, undefined /*m_renderCtx.getRenderTarget().getPixelFormat()*/),
        this.m_texture.getRefTexture(), texCoord, renderParams);

    // Compare and log.
    var isOk = glsTextureTestUtil.compareImages(referenceFrame, renderedFrame, threshold);

    assertMsgOptions(isOk, 'Face: ' + this.m_curFace + ' ' + es3fTextureFormatTests.testDescription(), true, false);
    return isOk;
};

es3fTextureFormatTests.CompressedCubeFormatCase.prototype.iterate = function() {
    debug('Testing face ' + this.m_curFace);
    // Execute test for all faces.
    if (!this.testFace(this.m_curFace))
        this.m_isOk = false;

    this.m_curFace += 1;

    if (this.m_curFace < Object.keys(tcuTexture.CubeFace).length)
        return tcuTestCase.IterateResult.CONTINUE;
    else
        return tcuTestCase.IterateResult.STOP;
};

es3fTextureFormatTests.genTestCases = function() {
    var state = tcuTestCase.runner;
    state.setRoot(tcuTestCase.newTest('texture_format', 'Top level'));

    var texFormats = [
        ['alpha', gl.ALPHA, gl.UNSIGNED_BYTE],
        ['luminance', gl.LUMINANCE, gl.UNSIGNED_BYTE],
        ['luminance_alpha', gl.LUMINANCE_ALPHA, gl.UNSIGNED_BYTE],
        ['rgb_unsigned_short_5_6_5', gl.RGB, gl.UNSIGNED_SHORT_5_6_5],
        ['rgb_unsigned_byte', gl.RGB, gl.UNSIGNED_BYTE],
        ['rgba_unsigned_short_4_4_4_4', gl.RGBA, gl.UNSIGNED_SHORT_4_4_4_4],
        ['rgba_unsigned_short_5_5_5_1', gl.RGBA, gl.UNSIGNED_SHORT_5_5_5_1],
        ['rgba_unsigned_byte', gl.RGBA, gl.UNSIGNED_BYTE]
    ];

    var unsized2DGroup = tcuTestCase.newTest('unsized', 'Unsized formats (2D, Cubemap)');
    state.testCases.addChild(unsized2DGroup);
    var unsized2DArrayGroup = tcuTestCase.newTest('unsized', 'Unsized formats (2D Array)');
    state.testCases.addChild(unsized2DArrayGroup);
    var unsized3DGroup = tcuTestCase.newTest('unsized', 'Unsized formats (3D)');
    state.testCases.addChild(unsized3DGroup);

    texFormats.forEach(function(elem) {
        var format = elem[1];
        var dataType = elem[2];
        var nameBase = elem[0];
        var descriptionBase = gluStrUtil.getPixelFormatName(format) + ', ' + gluStrUtil.getTypeName(dataType);
        unsized2DGroup.addChild(new es3fTextureFormatTests.Texture2DFormatCase({
            name: nameBase + '_2d_pot',
            description: descriptionBase + ' gl.TEXTURE_2D',
            format: format,
            dataType: dataType,
            width: 128,
            height: 128
        }));
        unsized2DGroup.addChild(new es3fTextureFormatTests.Texture2DFormatCase({
            name: nameBase + '_2d_npot',
            description: descriptionBase + ' gl.TEXTURE_2D',
            format: format,
            dataType: dataType,
            width: 63,
            height: 112
        }));
        unsized2DGroup.addChild(new es3fTextureFormatTests.TextureCubeFormatCase({
            name: nameBase + '_cube_pot',
            description: descriptionBase + ' gl.TEXTURE_CUBE_MAP',
            format: format,
            dataType: dataType,
            width: 64,
            height: 64
        }));
        unsized2DGroup.addChild(new es3fTextureFormatTests.TextureCubeFormatCase({
            name: nameBase + '_cube_npot',
            description: descriptionBase + ' gl.TEXTURE_CUBE_MAP',
            format: format,
            dataType: dataType,
            width: 57,
            height: 57
        }));
        unsized2DArrayGroup.addChild(new es3fTextureFormatTests.Texture2DArrayFormatCase({
            name: nameBase + '_2d_array_pot',
            description: descriptionBase + ' gl.TEXTURE_2D_ARRAY',
            format: format,
            dataType: dataType,
            width: 64,
            height: 64,
            numLayers: 8
        }));
        unsized2DArrayGroup.addChild(new es3fTextureFormatTests.Texture2DArrayFormatCase({
            name: nameBase + '_2d_array_npot',
            description: descriptionBase + ' gl.TEXTURE_2D_ARRAY',
            format: format,
            dataType: dataType,
            width: 63,
            height: 57,
            numLayers: 7
        }));
        unsized3DGroup.addChild(new es3fTextureFormatTests.Texture3DFormatCase({
            name: nameBase + '_3d_pot',
            description: descriptionBase + ' gl.TEXTURE_3D',
            format: format,
            dataType: dataType,
            width: 8,
            height: 32,
            depth: 16
        }));
        unsized3DGroup.addChild(new es3fTextureFormatTests.Texture3DFormatCase({
            name: nameBase + '_3d_npot',
            description: descriptionBase + ' gl.TEXTURE_3D',
            format: format,
            dataType: dataType,
            width: 11,
            height: 31,
            depth: 7
        }));
    });

    var sizedColorFormats = [
        ['rgba32f', gl.RGBA32F],
        ['rgba32i', gl.RGBA32I],
        ['rgba32ui', gl.RGBA32UI],
        ['rgba16f', gl.RGBA16F],
        ['rgba16i', gl.RGBA16I],
        ['rgba16ui', gl.RGBA16UI],
        ['rgba8', gl.RGBA8],
        ['rgba8i', gl.RGBA8I],
        ['rgba8ui', gl.RGBA8UI],
        ['srgb8_alpha8', gl.SRGB8_ALPHA8],
        ['rgb10_a2', gl.RGB10_A2],
        ['rgb10_a2ui', gl.RGB10_A2UI],
        ['rgba4', gl.RGBA4],
        ['rgb5_a1', gl.RGB5_A1],
        ['rgba8_snorm', gl.RGBA8_SNORM],
        ['rgb8', gl.RGB8],
        ['rgb565', gl.RGB565],
        ['r11f_g11f_b10f', gl.R11F_G11F_B10F],
        ['rgb32f', gl.RGB32F],
        ['rgb32i', gl.RGB32I],
        ['rgb32ui', gl.RGB32UI],
        ['rgb16f', gl.RGB16F],
        ['rgb16i', gl.RGB16I],
        ['rgb16ui', gl.RGB16UI],
        ['rgb8_snorm', gl.RGB8_SNORM],
        ['rgb8i', gl.RGB8I],
        ['rgb8ui', gl.RGB8UI],
        ['srgb8', gl.SRGB8],
        ['rgb9_e5', gl.RGB9_E5],
        ['rg32f', gl.RG32F],
        ['rg32i', gl.RG32I],
        ['rg32ui', gl.RG32UI],
        ['rg16f', gl.RG16F],
        ['rg16i', gl.RG16I],
        ['rg16ui', gl.RG16UI],
        ['rg8', gl.RG8],
        ['rg8i', gl.RG8I],
        ['rg8ui', gl.RG8UI],
        ['rg8_snorm', gl.RG8_SNORM],
        ['r32f', gl.R32F],
        ['r32i', gl.R32I],
        ['r32ui', gl.R32UI],
        ['r16f', gl.R16F],
        ['r16i', gl.R16I],
        ['r16ui', gl.R16UI],
        ['r8', gl.R8],
        ['r8i', gl.R8I],
        ['r8ui', gl.R8UI],
        ['r8_snorm', gl.R8_SNORM]
    ];

    var splitSizedColorTests = 4;
    var sizedColor2DPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColor2DPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (2D POT)'));
        state.testCases.addChild(sizedColor2DPOTGroup[ii]);
    }
    var sizedColor2DNPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColor2DNPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (2D NPOT)'));
        state.testCases.addChild(sizedColor2DNPOTGroup[ii]);
    }
    var sizedColorCubePOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColorCubePOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (Cubemap POT)'));
        state.testCases.addChild(sizedColorCubePOTGroup[ii]);
    }
    var sizedColorCubeNPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColorCubeNPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (Cubemap NPOT)'));
        state.testCases.addChild(sizedColorCubeNPOTGroup[ii]);
    }
    var sizedColor2DArrayPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColor2DArrayPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (2D Array POT)'));
        state.testCases.addChild(sizedColor2DArrayPOTGroup[ii]);
    }
    var sizedColor2DArrayNPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColor2DArrayNPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (2D Array NPOT)'));
        state.testCases.addChild(sizedColor2DArrayNPOTGroup[ii]);
    }
    var sizedColor3DPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColor3DPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (3D POT)'));
        state.testCases.addChild(sizedColor3DPOTGroup[ii]);
    }
    var sizedColor3DNPOTGroup = [];
    for (var ii = 0; ii < splitSizedColorTests; ++ii) {
        sizedColor3DNPOTGroup.push(tcuTestCase.newTest('sized', 'Sized formats (3D NPOT)'));
        state.testCases.addChild(sizedColor3DNPOTGroup[ii]);
    }

    for (var ii = 0; ii < sizedColorFormats.length; ++ii) {
        var internalFormat = sizedColorFormats[ii][1];
        var nameBase = sizedColorFormats[ii][0];
        var descriptionBase = gluStrUtil.getPixelFormatName(internalFormat);
        sizedColor2DPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.Texture2DFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_2D',
            format: internalFormat,
            width: 128,
            height: 128
        }));
        sizedColor2DNPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.Texture2DFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_2D',
            format: internalFormat,
            width: 63,
            height: 112
        }));
        sizedColorCubePOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.TextureCubeFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_CUBE_MAP',
            format: internalFormat,
            width: 64,
            height: 64
        }));
        sizedColorCubeNPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.TextureCubeFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_CUBE_MAP',
            format: internalFormat,
            width: 57,
            height: 57
        }));
        sizedColor2DArrayPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.Texture2DArrayFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_2D_ARRAY',
            format: internalFormat,
            width: 64,
            height: 64,
            numLayers: 8

        }));
        sizedColor2DArrayNPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.Texture2DArrayFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_2D_ARRAY',
            format: internalFormat,
            width: 63,
            height: 57,
            numLayers: 7
        }));
        sizedColor3DPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.Texture3DFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_3D',
            format: internalFormat,
            width: 8,
            height: 32,
            depth: 16
        }));
        sizedColor3DNPOTGroup[ii % splitSizedColorTests].addChild(new es3fTextureFormatTests.Texture3DFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_3D',
            format: internalFormat,
            width: 11,
            height: 31,
            depth: 7
        }));
    }

    var sizedDepthStencilFormats = [
        // Depth and stencil formats
        ['depth_component32f', gl.DEPTH_COMPONENT32F],
        ['depth_component24', gl.DEPTH_COMPONENT24],
        ['depth_component16', gl.DEPTH_COMPONENT16],
        // The following format is restricted in WebGL2.
        // ['depth32f_stencil8', gl.DEPTH32F_STENCIL8],
        ['depth24_stencil8', gl.DEPTH24_STENCIL8]
    ];
    var sizedDepthStencilGroup = tcuTestCase.newTest('sized', 'Sized formats (Depth Stencil)');
    state.testCases.addChild(sizedDepthStencilGroup);
    sizedDepthStencilFormats.forEach(function(elem) {
        var internalFormat = elem[1];
        var nameBase = elem[0];
        var descriptionBase = gluStrUtil.getPixelFormatName(internalFormat);
        sizedDepthStencilGroup.addChild(new es3fTextureFormatTests.Texture2DFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_2D',
            format: internalFormat,
            width: 128,
            height: 128
        }));
        sizedDepthStencilGroup.addChild(new es3fTextureFormatTests.Texture2DFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_2D',
            format: internalFormat,
            width: 63,
            height: 112
        }));
        sizedDepthStencilGroup.addChild(new es3fTextureFormatTests.TextureCubeFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_CUBE_MAP',
            format: internalFormat,
            width: 64,
            height: 64
        }));
        sizedDepthStencilGroup.addChild(new es3fTextureFormatTests.TextureCubeFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_CUBE_MAP',
            format: internalFormat,
            width: 57,
            height: 57
        }));
        sizedDepthStencilGroup.addChild(new es3fTextureFormatTests.Texture2DArrayFormatCase({
            name: nameBase + '_pot',
            description: descriptionBase + ' gl.TEXTURE_2D_ARRAY',
            format: internalFormat,
            width: 64,
            height: 64,
            numLayers: 8
        }));
        sizedDepthStencilGroup.addChild(new es3fTextureFormatTests.Texture2DArrayFormatCase({
            name: nameBase + '_npot',
            description: descriptionBase + ' gl.TEXTURE_2D_ARRAY',
            format: internalFormat,
            width: 63,
            height: 57,
            numLayers: 7
        }));
    });

    var compressed2DGroup = tcuTestCase.newTest('compressed', 'Compressed formats (2D)');
    state.testCases.addChild(compressed2DGroup);
    var compressedCubeGroup = tcuTestCase.newTest('compressed', 'Compressed formats (Cubemap)');
    state.testCases.addChild(compressedCubeGroup);
    var etc2Formats = [
        ['gl.COMPRESSED_R11_EAC', 'eac_r11', tcuCompressedTexture.Format.EAC_R11],
        ['gl.COMPRESSED_SIGNED_R11_EAC', 'eac_signed_r11', tcuCompressedTexture.Format.EAC_SIGNED_R11],
        ['gl.COMPRESSED_RG11_EAC', 'eac_rg11', tcuCompressedTexture.Format.EAC_RG11],
        ['gl.COMPRESSED_SIGNED_RG11_EAC', 'eac_signed_rg11', tcuCompressedTexture.Format.EAC_SIGNED_RG11],
        ['gl.COMPRESSED_RGB8_ETC2', 'etc2_rgb8', tcuCompressedTexture.Format.ETC2_RGB8],
        ['gl.COMPRESSED_SRGB8_ETC2', 'etc2_srgb8', tcuCompressedTexture.Format.ETC2_SRGB8],
        ['gl.COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2', 'etc2_rgb8_punchthrough_alpha1', tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1],
        ['gl.COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2', 'etc2_srgb8_punchthrough_alpha1', tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1],
        ['gl.COMPRESSED_RGBA8_ETC2_EAC', 'etc2_eac_rgba8', tcuCompressedTexture.Format.ETC2_EAC_RGBA8],
        ['gl.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC', 'etc2_eac_srgb8_alpha8', tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8]
    ];
    if (!gluTextureUtil.enableCompressedTextureETC()) {
        debug('Skipping ETC2/EAC texture format tests: no support for WEBGL_compressed_texture_etc');
        etc2Formats = [];
    }
    etc2Formats.forEach(function(elem) {
        var nameBase = elem[1];
        var descriptionBase = elem[0];
        var format = elem[2];
        compressed2DGroup.addChild(new es3fTextureFormatTests.Compressed2DFormatCase({
            name: nameBase + '_2d_pot',
            description: descriptionBase + ', gl.TEXTURE_2D',
            format: format,
            width: 128,
            height: 64
        }));
        compressedCubeGroup.addChild(new es3fTextureFormatTests.CompressedCubeFormatCase({
            name: nameBase + '_cube_pot',
            description: descriptionBase + ', gl.TEXTURE_CUBE_MAP',
            format: format,
            width: 64,
            height: 64
        }));
        compressed2DGroup.addChild(new es3fTextureFormatTests.Compressed2DFormatCase({
            name: nameBase + '_2d_pot',
            description: descriptionBase + ', gl.TEXTURE_2D',
            format: format,
            width: 128,
            height: 64
        }));
        compressedCubeGroup.addChild(new es3fTextureFormatTests.CompressedCubeFormatCase({
            name: nameBase + '_cube_npot',
            description: descriptionBase + ', gl.TEXTURE_CUBE_MAP',
            format: format,
            width: 51,
            height: 51
        }));
    });
};

/**
 * Create and execute the test cases
 */
es3fTextureFormatTests.run = function(context, range) {
    gl = context;
    var state = tcuTestCase.runner;
    try {
        es3fTextureFormatTests.genTestCases();
        if (range)
            state.setRange(range);
        state.runCallback(tcuTestCase.runTestCases);
    } catch (err) {
        bufferedLogToConsole(err);
        state.terminate();
    }

};

});

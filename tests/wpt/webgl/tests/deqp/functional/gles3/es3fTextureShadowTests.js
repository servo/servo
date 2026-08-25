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
goog.provide('functional.gles3.es3fTextureShadowTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexCompareVerifier');
goog.require('framework.common.tcuTexLookupVerifier');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.referencerenderer.rrMultisamplePixelBufferAccess');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {

var es3fTextureShadowTests = functional.gles3.es3fTextureShadowTests;
var tcuTestCase = framework.common.tcuTestCase;
var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var gluTexture = framework.opengl.gluTexture;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var tcuImageCompare = framework.common.tcuImageCompare;
var tcuLogImage = framework.common.tcuLogImage;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuRGBA = framework.common.tcuRGBA;
var deMath = framework.delibs.debase.deMath;
var tcuPixelFormat = framework.common.tcuPixelFormat;
var tcuSurface = framework.common.tcuSurface;
var tcuTexCompareVerifier = framework.common.tcuTexCompareVerifier;
var tcuTexLookupVerifier = framework.common.tcuTexLookupVerifier;
var rrMultisamplePixelBufferAccess = framework.referencerenderer.rrMultisamplePixelBufferAccess;
var deString = framework.delibs.debase.deString;
var deUtil = framework.delibs.debase.deUtil;

    es3fTextureShadowTests.version = '300 es';

    /** @const */ var VIEWPORT_WIDTH = 64;
    /** @const */ var VIEWPORT_HEIGHT = 64;
    /** @const */ var MIN_VIEWPORT_WIDTH = 64;
    /** @const */ var MIN_VIEWPORT_HEIGHT = 64;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * @param {tcuTexture.TextureFormat} format
     * @return {boolean}
     */
    es3fTextureShadowTests.isFloatingPointDepthFormat = function(format) {
        // Only two depth and depth-stencil formats are floating point
        return (format.order == tcuTexture.ChannelOrder.D && format.type == tcuTexture.ChannelType.FLOAT) || (format.order == tcuTexture.ChannelOrder.DS && format.type == tcuTexture.ChannelType.FLOAT_UNSIGNED_INT_24_8_REV);
    };

    /**
     * @param {tcuTexture.PixelBufferAccess} access
     */
    es3fTextureShadowTests.clampFloatingPointTexture = function(access) {
        DE_ASSERT(es3fTextureShadowTests.isFloatingPointDepthFormat(access.getFormat()));
        for (var z = 0; z < access.getDepth(); ++z)
            for (var y = 0; y < access.getHeight(); ++y)
                for (var x = 0; x < access.getWidth(); ++x)
                    access.setPixDepth(deMath.clamp(access.getPixDepth(x, y, z), 0.0, 1.0), x, y, z);
    };

    /**
     * @param {tcuTexture.Texture2D|tcuTexture.Texture2DArray} target
     */
    es3fTextureShadowTests.clampFloatingPointTexture2D = function(target) {
        for (var level = 0; level < target.getNumLevels(); ++level)
            if (!target.isLevelEmpty(level))
                es3fTextureShadowTests.clampFloatingPointTexture(target.getLevel(level));
    };

    /**
     * @param {tcuTexture.TextureCube} target
     */
    es3fTextureShadowTests.clampFloatingPointTextureCube = function(target) {
        for (var level = 0; level < target.getNumLevels(); ++level)
            for (var face = tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X; face < Object.keys(tcuTexture.CubeFace).length; face++)
                es3fTextureShadowTests.clampFloatingPointTexture(target.getLevelFace(level, face));
    };

    /**
     * @param {?} textureType
     * @param {tcuTexture.ConstPixelBufferAccess} result
     * @param {tcuTexture.Texture2D|tcuTexture.Texture2DArray|tcuTexture.TextureCube} src
     * @param {Array<number>} texCoord
     * @param {glsTextureTestUtil.ReferenceParams} sampleParams
     * @param {tcuTexCompareVerifier.TexComparePrecision} comparePrec
     * @param {tcuTexLookupVerifier.LodPrecision} lodPrecision
     * @param {tcuPixelFormat.PixelFormat} pixelFormat
     */
    es3fTextureShadowTests.verifyTexCompareResult = function(textureType, result, src, texCoord, sampleParams, comparePrec, lodPrecision, pixelFormat) {
        var reference = new tcuSurface.Surface(result.getWidth(), result.getHeight());
        var errorMask = new tcuSurface.Surface(result.getWidth(), result.getHeight());
        var nonShadowThreshold = deMath.swizzle(tcuTexLookupVerifier.computeFixedPointThreshold(deMath.subtract(glsTextureTestUtil.getBitsVec(pixelFormat), [1, 1, 1, 1])), [1, 2, 3]);
        var numFailedPixels;

        if (es3fTextureShadowTests.isFloatingPointDepthFormat(src.getFormat())) {
            var clampedSource = /*deUtil.clone(*/src/*)*/;

            if (textureType == tcuTexture.Texture2D) {
                es3fTextureShadowTests.clampFloatingPointTexture2D(/** @type {tcuTexture.Texture2D} */(clampedSource));
                glsTextureTestUtil.sampleTexture2D(new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat), /** @type {tcuTexture.Texture2DView} */ (clampedSource.getView()), texCoord, sampleParams);
                // sample clamped values
                numFailedPixels = glsTextureTestUtil.computeTextureCompareDiff2D(result, reference.getAccess(), errorMask.getAccess(), /** @type {tcuTexture.Texture2DView} */ (clampedSource.getView()), texCoord, sampleParams, comparePrec, lodPrecision, nonShadowThreshold);
            } else if (textureType == tcuTexture.Texture2DArray) {
                es3fTextureShadowTests.clampFloatingPointTexture2D(/** @type {tcuTexture.Texture2DArray} */(clampedSource));
                glsTextureTestUtil.sampleTexture2DArray(new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat), /** @type {tcuTexture.Texture2DArrayView} */ (clampedSource.getView()), texCoord, sampleParams);
                // sample clamped values
                numFailedPixels = glsTextureTestUtil.computeTextureCompareDiff2DArray(result, reference.getAccess(), errorMask.getAccess(), /** @type {tcuTexture.Texture2DArrayView} */ (clampedSource.getView()), texCoord, sampleParams, comparePrec, lodPrecision, nonShadowThreshold);
            } else if (textureType == tcuTexture.TextureCube) {
                es3fTextureShadowTests.clampFloatingPointTextureCube(/** @type {tcuTexture.TextureCube} */(clampedSource));
                glsTextureTestUtil.sampleTextureCube(new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat), /** @type {tcuTexture.TextureCubeView} */ (clampedSource.getView()), texCoord, sampleParams);
                // sample clamped values
                numFailedPixels = glsTextureTestUtil.computeTextureCompareDiffCube(result, reference.getAccess(), errorMask.getAccess(), /** @type {tcuTexture.TextureCubeView} */ (clampedSource.getView()), texCoord, sampleParams, comparePrec, lodPrecision, nonShadowThreshold);
            } else
                throw new Error('Invalid texture type');

        } else {
            if (textureType == tcuTexture.Texture2D) {
                glsTextureTestUtil.sampleTexture2D(new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat), /** @type {tcuTexture.Texture2DView} */ (src.getView()), texCoord, sampleParams);
                // sample raw values (they are guaranteed to be in [0, 1] range as the format cannot represent any other values)
                numFailedPixels = glsTextureTestUtil.computeTextureCompareDiff2D(result, reference.getAccess(), errorMask.getAccess(), /** @type {tcuTexture.Texture2DView} */ (src.getView()), texCoord, sampleParams, comparePrec, lodPrecision, nonShadowThreshold);
            } else if (textureType == tcuTexture.Texture2DArray) {
                glsTextureTestUtil.sampleTexture2DArray(new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat), /** @type {tcuTexture.Texture2DArrayView} */ (src.getView()), texCoord, sampleParams);
                // sample raw values (they are guaranteed to be in [0, 1] range as the format cannot represent any other values)
                numFailedPixels = glsTextureTestUtil.computeTextureCompareDiff2DArray(result, reference.getAccess(), errorMask.getAccess(), /** @type {tcuTexture.Texture2DArrayView} */ (src.getView()), texCoord, sampleParams, comparePrec, lodPrecision, nonShadowThreshold);
            } else if (textureType == tcuTexture.TextureCube) {
                glsTextureTestUtil.sampleTextureCube(new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat), /** @type {tcuTexture.TextureCubeView} */ (src.getView()), texCoord, sampleParams);
                // sample raw values (they are guaranteed to be in [0, 1] range as the format cannot represent any other values)
                numFailedPixels = glsTextureTestUtil.computeTextureCompareDiffCube(result, reference.getAccess(), errorMask.getAccess(), /** @type {tcuTexture.TextureCubeView} */ (src.getView()), texCoord, sampleParams, comparePrec, lodPrecision, nonShadowThreshold);
            } else
                throw new Error('Invalid texture type');
        }

        if (numFailedPixels > 0)
            bufferedLogToConsole('ERROR: Result verification failed, got ' + numFailedPixels + ' invalid pixels!');

        if (numFailedPixels > 0)
            tcuImageCompare.displayImages(result, reference.getAccess(), errorMask.getAccess());
        else
            tcuLogImage.logImageWithInfo(result, 'Result');

        return numFailedPixels == 0;

    };

    /**
     * @constructor
     * @param {string} name
     * @param {number} format
     * @struct
     */
    es3fTextureShadowTests.Format = function(name, format) {
        /** @type {string} */ this.name = name;
        /** @type {number} */ this.format = format;
    };

    /**
     * @constructor
     * @param {string} name
     * @param {number} minFilter
     * @param {number} magFilter
     * @struct
     */
    es3fTextureShadowTests.Filter = function(name, minFilter, magFilter) {
        /** @type {string} */ this.name = name;
        /** @type {number} */ this.minFilter = minFilter;
        /** @type {number} */ this.magFilter = magFilter;
    };

    /**
     * @constructor
     * @param {string} name
     * @param {number} func
     * @struct
     */
    es3fTextureShadowTests.CompareFunc = function(name, func) {
        /** @type {string} */ this.name = name;
        /** @type {number} */ this.func = func;
    };

    /**
     * @constructor
     * @param {number} texNdx
     * @param {number} ref
     * @param {number} lodX
     * @param {number} lodY
     * @param {number} oX
     * @param {number} oY
     * @struct
     */
    es3fTextureShadowTests.TestCase = function(texNdx, ref, lodX, lodY, oX, oY) {
        /** @type {number} */ this.texNdx = texNdx;
        /** @type {number} */ this.ref = ref;
        /** @type {number} */ this.lodX = lodX;
        /** @type {number} */ this.lodY = lodY;
        /** @type {number} */ this.oX = oX
        /** @type {number} */ this.oY = oY;
    };

    /**
     * @constructor
     * @param {?gluTexture.Texture2D|?gluTexture.TextureCube|?gluTexture.Texture2DArray} tex
     * @param {number} ref
     * @param {Array<number>} minCoord
     * @param {Array<number>} maxCoord
     * @struct
     */
    es3fTextureShadowTests.FilterCase = function(tex, ref, minCoord, maxCoord) {
        /** @type {?gluTexture.Texture2D|?gluTexture.TextureCube|?gluTexture.Texture2DArray} */ this.texture = tex;
        /** @type {Array<number>} */ this.minCoord = minCoord;
        /** @type {Array<number>} */ this.maxCoord = maxCoord;
        /** @type {number} */ this.ref = ref;
    };

    /**
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} format
     * @param {number} width
     * @param {number} height
     * @param {number} compareFunc
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fTextureShadowTests.Texture2DShadowCase = function(name, desc, minFilter, magFilter, wrapS, wrapT, format, width, height, compareFunc) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_minFilter = minFilter;
        this.m_magFilter = magFilter;
        this.m_wrapS = wrapS;
        this.m_wrapT = wrapT;
        this.m_format = format;
        this.m_width = width;
        this.m_height = height;
        this.m_compareFunc = compareFunc;
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureShadowTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
        this.m_caseNdx = 0;
        this.m_cases = [];
    };

    es3fTextureShadowTests.Texture2DShadowCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fTextureShadowTests.Texture2DShadowCase.prototype.constructor = es3fTextureShadowTests.Texture2DShadowCase;

    es3fTextureShadowTests.Texture2DShadowCase.prototype.init = function() {

        // Create 2 textures.
        this.m_textures = [];
        this.m_textures[0] = gluTexture.texture2DFromInternalFormat(gl, this.m_format, this.m_width, this.m_height);
        this.m_textures[1] = gluTexture.texture2DFromInternalFormat(gl, this.m_format, this.m_width, this.m_height);

        var numLevels = this.m_textures[0].getRefTexture().getNumLevels();

        for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
            this.m_textures[0].getRefTexture().allocLevel(levelNdx);
            tcuTextureUtil.fillWithComponentGradients(this.m_textures[0].getRefTexture().getLevel(levelNdx), [-0.5, -0.5, -0.5, 2.0], [1, 1, 1, 0]);
        }

        for (levelNdx = 0; levelNdx < numLevels; levelNdx++) {
            var step = 0x00ffffff / numLevels;
            var rgb = step * levelNdx;
            var colorA = 0xff000000 | rgb;
            var colorB = 0xff000000 | ~rgb;

            this.m_textures[1].getRefTexture().allocLevel(levelNdx);
            tcuTextureUtil.fillWithGrid(this.m_textures[1].getRefTexture().getLevel(levelNdx), 4, tcuRGBA.newRGBAFromValue(colorA).toVec(), tcuRGBA.newRGBAFromValue(colorB).toVec());
        }

        for (var i = 0; i < this.m_textures.length; i++)
            this.m_textures[i].upload();

        var refInRangeUpper = (this.m_compareFunc == gl.EQUAL || this.m_compareFunc == gl.NOTEQUAL) ? 1.0 : 0.5;
        var refInRangeLower = (this.m_compareFunc == gl.EQUAL || this.m_compareFunc == gl.NOTEQUAL) ? 0.0 : 0.5;

        var refOutOfBoundsUpper = 1.1;
        var refOutOfBoundsLower = -0.1;

        numLevels = this.m_textures[0].getRefTexture().getNumLevels();

        var cases = [];
        cases.push(new es3fTextureShadowTests.TestCase(0, refInRangeUpper, 1.6, 2.9, -1.0, -2.7));
        cases.push(new es3fTextureShadowTests.TestCase(0, refInRangeLower, -2.0, -1.35, -0.2, 0.7));
        cases.push(new es3fTextureShadowTests.TestCase(1, refInRangeUpper, 0.14, 0.275, -1.5, -1.1));
        cases.push(new es3fTextureShadowTests.TestCase(1, refInRangeLower, -0.92, -2.64, 0.4, -0.1));
        cases.push(new es3fTextureShadowTests.TestCase(1, refOutOfBoundsUpper, -0.39, -0.52, 0.65, 0.87));
        cases.push(new es3fTextureShadowTests.TestCase(1, refOutOfBoundsLower, -1.55, 0.65, 0.35, 0.91));

        var viewportW = Math.min(VIEWPORT_WIDTH, gl.canvas.width);
        var viewportH = Math.min(VIEWPORT_HEIGHT, gl.canvas.height);

        for (var caseNdx = 0; caseNdx < cases.length; caseNdx++) {
            var texNdx = deMath.clamp(cases[caseNdx].texNdx, 0, this.m_textures.length - 1);
            var ref = cases[caseNdx].ref;
            var lodX = cases[caseNdx].lodX;
            var lodY = cases[caseNdx].lodY;
            var oX = cases[caseNdx].oX;
            var oY = cases[caseNdx].oY;
            var sX = Math.exp(lodX * Math.log(2)) * viewportW / this.m_textures[texNdx].getRefTexture().getWidth();
            var sY = Math.exp(lodY * Math.log(2)) * viewportH / this.m_textures[texNdx].getRefTexture().getHeight();

            this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_textures[texNdx], ref, [oX, oY], [oX + sX, oY + sY]));
        }

        this.m_caseNdx = 0;
    };

    es3fTextureShadowTests.Texture2DShadowCase.prototype.iterate = function() {

        var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        var curCase = this.m_cases[this.m_caseNdx];
        var sampleParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_2D);
        var rendered = new tcuSurface.Surface(viewport.width, viewport.height);
        var texCoord = [];

        if (viewport.width < MIN_VIEWPORT_WIDTH || viewport.height < MIN_VIEWPORT_HEIGHT)
            throw new Error('Too small render target');

        // Setup params for reference.
        sampleParams.sampler = gluTextureUtil.mapGLSampler(this.m_wrapS, this.m_wrapT, gl.CLAMP_TO_EDGE, this.m_minFilter, this.m_magFilter);
        sampleParams.sampler.compare = gluTextureUtil.mapGLCompareFunc(this.m_compareFunc);
        sampleParams.samplerType = glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW;
        sampleParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        sampleParams.ref = curCase.ref;

        bufferedLogToConsole('Compare reference value = ' + sampleParams.ref);

        // Compute texture coordinates.
        bufferedLogToConsole('Texture coordinates: ' + curCase.minCoord + ' -> ' + curCase.maxCoord);

        texCoord = glsTextureTestUtil.computeQuadTexCoord2D(curCase.minCoord, curCase.maxCoord);

        gl.bindTexture(gl.TEXTURE_2D, curCase.texture.getGLTexture());
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, this.m_minFilter);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, this.m_magFilter);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, this.m_wrapT);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_FUNC, this.m_compareFunc);

        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        this.m_renderer.renderQuad(0, texCoord, sampleParams);
        rendered.readViewport(gl, viewport);


        var pixelFormat = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var lodPrecision = new tcuTexLookupVerifier.LodPrecision(18, 6);
        var texComparePrecision = new tcuTexCompareVerifier.TexComparePrecision([20, 20, 0], [7, 7, 0], 5, 16, pixelFormat.redBits - 1);

        var isHighQuality = es3fTextureShadowTests.verifyTexCompareResult(tcuTexture.Texture2D, rendered.getAccess(), curCase.texture.getRefTexture(),
                                                      texCoord, sampleParams, texComparePrecision, lodPrecision, pixelFormat);

        if (!isHighQuality) {
            bufferedLogToConsole('Warning: Verification assuming high-quality PCF filtering failed.');

            lodPrecision.lodBits = 4;
            texComparePrecision.uvwBits = [4, 4, 0];
            texComparePrecision.pcfBits = 0;

            var isOk = es3fTextureShadowTests.verifyTexCompareResult(tcuTexture.Texture2D, rendered.getAccess(), curCase.texture.getRefTexture(),
                                                     texCoord, sampleParams, texComparePrecision, lodPrecision, pixelFormat);

            if (!isOk) {
                bufferedLogToConsole('ERROR: Verification against low precision requirements failed, failing test case.');
                testFailedOptions('Image verification failed', false);
            } else
                testPassedOptions('Low-quality result', true);
        } else
            testPassedOptions('High-quality result', true);

        this.m_caseNdx += 1;
        return this.m_caseNdx < this.m_cases.length ? tcuTestCase.IterateResult.CONTINUE : tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} format
     * @param {number} size
     * @param {number} compareFunc
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fTextureShadowTests.TextureCubeShadowCase = function(name, desc, minFilter, magFilter, wrapS, wrapT, format, size, compareFunc) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_minFilter = minFilter;
        this.m_magFilter = magFilter;
        this.m_wrapS = wrapS;
        this.m_wrapT = wrapT;
        this.m_format = format;
        this.m_size = size;
        this.m_compareFunc = compareFunc;
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureShadowTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
        this.m_caseNdx = 0;
        this.m_cases = [];
    };

    es3fTextureShadowTests.TextureCubeShadowCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fTextureShadowTests.TextureCubeShadowCase.prototype.constructor = es3fTextureShadowTests.TextureCubeShadowCase;

    es3fTextureShadowTests.TextureCubeShadowCase.prototype.init = function() {
        DE_ASSERT(!this.m_gradientTex && !this.m_gridTex);

        var numLevels = Math.floor(Math.log2(this.m_size)) + 1;
        /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_format);
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        /** @type {Array<number>} */ var cBias = fmtInfo.valueMin;
        /** @type {Array<number>} */ var cScale = deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin);

        // Create textures.
        this.m_gradientTex = gluTexture.cubeFromInternalFormat(gl, this.m_format, this.m_size);
        this.m_gridTex = gluTexture.cubeFromInternalFormat(gl, this.m_format, this.m_size);

        // Fill first with gradient texture.
        var gradients = [[[-1.0, -1.0, -1.0, 2.0], [1.0, 1.0, 1.0, 0.0]], // negative x
            [[0.0, -1.0, -1.0, 2.0], [1.0, 1.0, 1.0, 0.0]], // positive x
            [[-1.0, 0.0, -1.0, 2.0], [1.0, 1.0, 1.0, 0.0]], // negative y
            [[-1.0, -1.0, 0.0, 2.0], [1.0, 1.0, 1.0, 0.0]], // positive y
            [[-1.0, -1.0, -1.0, 0.0], [1.0, 1.0, 1.0, 1.0]], // negative z
            [[0.0, 0.0, 0.0, 2.0], [1.0, 1.0, 1.0, 0.0]]]; // positive z

        for (var face in tcuTexture.CubeFace) {
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                this.m_gradientTex.getRefTexture().allocLevel(tcuTexture.CubeFace[face], levelNdx);
                tcuTextureUtil.fillWithComponentGradients(this.m_gradientTex.getRefTexture().getLevelFace(levelNdx, tcuTexture.CubeFace[face]), deMath.add(deMath.multiply(gradients[tcuTexture.CubeFace[face]][0], cScale), cBias), deMath.add(deMath.multiply(gradients[tcuTexture.CubeFace[face]][1], cScale), cBias));
            }
        }

        // Fill second with grid texture.
        for (var face in tcuTexture.CubeFace) {
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                var step = 0x00ffffff / (numLevels * Object.keys(tcuTexture.CubeFace).length);
                var rgb = step * levelNdx * face;
                var colorA = 0xff000000 | rgb;
                var colorB = 0xff000000 | ~rgb;

                this.m_gridTex.getRefTexture().allocLevel(tcuTexture.CubeFace[face], levelNdx);
                tcuTextureUtil.fillWithGrid(this.m_gridTex.getRefTexture().getLevelFace(levelNdx, tcuTexture.CubeFace[face]), 4, deMath.add(deMath.multiply(tcuRGBA.newRGBAFromValue(colorA).toVec(), cScale), cBias), deMath.add(deMath.multiply(tcuRGBA.newRGBAFromValue(colorB).toVec(), cScale), cBias));
            }
        }

        // Upload.
        this.m_gradientTex.upload();
        this.m_gridTex.upload();

        var refInRangeUpper = (this.m_compareFunc == gl.EQUAL || this.m_compareFunc == gl.NOTEQUAL) ? 1.0 : 0.5;
        var refInRangeLower = (this.m_compareFunc == gl.EQUAL || this.m_compareFunc == gl.NOTEQUAL) ? 0.0 : 0.5;
        var refOutOfBoundsUpper = 1.1;
        var refOutOfBoundsLower = -0.1;
        var singleSample = new rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess().getNumSamples() == 0;
        //var singleSample = this.m_context.getRenderTarget().getNumSamples() == 0;

        if (singleSample)
            this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gradientTex, refInRangeUpper, [-1.25, -1.2], [1.2, 1.25])); // minification
        else
            this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gradientTex, refInRangeUpper, [-1.19, -1.3], [1.1, 1.35])); // minification - w/ tuned coordinates to avoid hitting triangle edges

        this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gradientTex, refInRangeLower, [0.8, 0.8], [1.25, 1.20])); // magnification
        this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gridTex, refInRangeUpper, [-1.19, -1.3], [1.1, 1.35])); // minification
        this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gridTex, refInRangeLower, [-1.2, -1.1], [-0.8, -0.8])); // magnification
        this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gridTex, refOutOfBoundsUpper, [-0.61, -0.1], [0.9, 1.18])); // reference value clamp, upper

        if (singleSample)
            this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gridTex, refOutOfBoundsLower, [-0.75, 1.0], [0.05, 0.75])); // reference value clamp, lower
        else
            this.m_cases.push(new es3fTextureShadowTests.FilterCase(this.m_gridTex, refOutOfBoundsLower, [-0.75, 1.0], [0.25, 0.75])); // reference value clamp, lower

        this.m_caseNdx = 0;
    };

    es3fTextureShadowTests.TextureCubeShadowCase.prototype.iterate = function() {
        var viewportSize = 28;
        var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), viewportSize, viewportSize, deString.deStringHash(this.fullName()) ^ deMath.deMathHash(this.m_caseNdx));
        var curCase = this.m_cases[this.m_caseNdx];
        var sampleParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_CUBE);

        if (viewport.width < viewportSize || viewport.height < viewportSize)
            throw new Error('Too small render target');

        // Setup texture
        gl.bindTexture(gl.TEXTURE_CUBE_MAP, curCase.texture.getGLTexture());
        gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, this.m_minFilter);
        gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, this.m_magFilter);
        gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, this.m_wrapT);
        gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
        gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_COMPARE_FUNC, this.m_compareFunc);

        // Other state
        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

        // Params for reference computation.
        sampleParams.sampler = gluTextureUtil.mapGLSampler(gl.CLAMP_TO_EDGE, gl.CLAMP_TO_EDGE, gl.CLAMP_TO_EDGE, this.m_minFilter, this.m_magFilter);
        sampleParams.sampler.seamlessCubeMap = true;
        sampleParams.sampler.compare = gluTextureUtil.mapGLCompareFunc(this.m_compareFunc);
        sampleParams.samplerType = glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW;
        sampleParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        sampleParams.ref = curCase.ref;

        bufferedLogToConsole(
            'Compare reference value = ' + sampleParams.ref + '\n' +
            'Coordinates: ' + curCase.minCoord + ' -> ' + curCase.maxCoord);

        for (var faceNdx in tcuTexture.CubeFace) {
            var face = tcuTexture.CubeFace[faceNdx];
            var result = new tcuSurface.Surface(viewport.width, viewport.height);
            var texCoord = [];

            texCoord = glsTextureTestUtil.computeQuadTexCoordCubeFace(face, curCase.minCoord, curCase.maxCoord);

            this.m_renderer.renderQuad(0, texCoord, sampleParams);

            result.readViewport(gl, viewport);

            var pixelFormat = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
            /** @type {tcuTexLookupVerifier.LodPrecision} */ var lodPrecision = new tcuTexLookupVerifier.LodPrecision(10, 5);
            /** @type {tcuTexCompareVerifier.TexComparePrecision} */ var texComparePrecision = new tcuTexCompareVerifier.TexComparePrecision(
                [10, 10, 10],
                [6, 6, 0],
                5,
                16,
                pixelFormat.redBits - 1
            );

            var isHighQuality = es3fTextureShadowTests.verifyTexCompareResult(tcuTexture.TextureCube, result.getAccess(), curCase.texture.getRefTexture(),
                                                     texCoord, sampleParams, texComparePrecision, lodPrecision, pixelFormat);

            if (!isHighQuality) {
                bufferedLogToConsole('Warning: Verification assuming high-quality PCF filtering failed.');

                lodPrecision.lodBits = 4;
                texComparePrecision.uvwBits = [4, 4, 0];
                texComparePrecision.pcfBits = 0;

                var isOk = es3fTextureShadowTests.verifyTexCompareResult(tcuTexture.TextureCube, result.getAccess(), curCase.texture.getRefTexture(),
                                                                                                  texCoord, sampleParams, texComparePrecision, lodPrecision, pixelFormat);
                if (!isOk) {
                    bufferedLogToConsole('ERROR: Verification against low precision requirements failed, failing test case.');
                    testFailedOptions('Image verification failed', false);
                } else
                    testPassedOptions('Low-quality result', true);
            }
            else
                testPassedOptions('High-quality result', true);
        }

        this.m_caseNdx += 1;
        return this.m_caseNdx < this.m_cases.length ? tcuTestCase.IterateResult.CONTINUE : tcuTestCase.IterateResult.STOP;
    };

    /**
     * Testure2DArrayShadowCase
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} format
     * @param {number} width
     * @param {number} height
     * @param {number} numLayers
     * @param {number} compareFunc
     */
    es3fTextureShadowTests.Texture2DArrayShadowCase = function(name, desc, minFilter, magFilter, wrapS, wrapT, format, width, height, numLayers, compareFunc) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        /** @type {number} */ this.m_minFilter = minFilter;
        /** @type {number} */ this.m_magFilter = magFilter;
        /** @type {number} */ this.m_wrapS = wrapS;
        /** @type {number} */ this.m_wrapT = wrapT;
        /** @type {number} */ this.m_format = format;
        /** @type {number} */ this.m_width = width;
        /** @type {number} */ this.m_height = height;
        /** @type {number} */ this.m_numLayers = numLayers;
        /** @type {number} */ this.m_compareFunc = compareFunc;
        /** @type {?gluTexture.Texture2DArray} */ this.m_gradientTex = null;
        /** @type {?gluTexture.Texture2DArray} */ this.m_gridTex = null;
        /** @type {glsTextureTestUtil.TextureRenderer} */ this.m_renderer = new glsTextureTestUtil.TextureRenderer(es3fTextureShadowTests.version, gluShaderUtil.precision.PRECISION_HIGHP);
        /** @type {Array<es3fTextureShadowTests.FilterCase>} */ this.m_cases = [];
        /** @type {number} */ this.m_caseNdx = 0;
    };

    es3fTextureShadowTests.Texture2DArrayShadowCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fTextureShadowTests.Texture2DArrayShadowCase.prototype.constructor = es3fTextureShadowTests.Texture2DArrayShadowCase;

    /**
     * init
     */
    es3fTextureShadowTests.Texture2DArrayShadowCase.prototype.init = function() {
        /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_format);
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        /** @type {Array<number>}*/ var cScale = deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin);
        /** @type {Array<number>}*/ var cBias = fmtInfo.valueMin;
        /** @type {number}*/ var numLevels = deMath.logToFloor(Math.max(this.m_width, this.m_height)) + 1;

        // Create textures.
        this.m_gradientTex = gluTexture.texture2DArrayFromInternalFormat(gl, this.m_format, this.m_width, this.m_height, this.m_numLayers);
        this.m_gridTex = gluTexture.texture2DArrayFromInternalFormat(gl, this.m_format, this.m_width, this.m_height, this.m_numLayers);

        // Fill first gradient texture.
        for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
            /** @type {Array<number>}*/ var gMin = deMath.add(deMath.multiply([-0.5, -0.5, -0.5, 2.0], cScale), cBias);
            /** @type {Array<number>}*/ var gMax = deMath.add(deMath.multiply([1.0, 1.0, 1.0, 0.0], cScale), cBias);

            this.m_gradientTex.getRefTexture().allocLevel(levelNdx);
            tcuTextureUtil.fillWithComponentGradients(
                /** @type {tcuTexture.PixelBufferAccess} */ (this.m_gradientTex.getRefTexture().getLevel(levelNdx)), gMin, gMax);
        }

        // Fill second with grid texture.
        for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
            /** @type {number}*/ var step = Math.floor(0x00ffffff / numLevels);
            /** @type {number}*/ var rgb = step * levelNdx;
            /** @type {number}*/ var colorA = deMath.binaryOp(0xff000000, rgb, deMath.BinaryOp.OR);
            /** @type {number}*/ var colorB = deMath.binaryOp(0xff000000, deMath.binaryNot(rgb), deMath.BinaryOp.OR);

            this.m_gridTex.getRefTexture().allocLevel(levelNdx);
            tcuTextureUtil.fillWithGrid(
                /** @type {tcuTexture.PixelBufferAccess} */ (this.m_gridTex.getRefTexture().getLevel(levelNdx)), 4,
                deMath.add(deMath.multiply(tcuRGBA.newRGBAFromValue(colorA).toVec(), cScale), cBias),
                deMath.add(deMath.multiply(tcuRGBA.newRGBAFromValue(colorB).toVec(), cScale), cBias)
            );
        }

        // Upload.
        this.m_gradientTex.upload();
        this.m_gridTex.upload();

        // Compute cases.
        /** @type {number} */ var refInRangeUpper = (this.m_compareFunc == gl.EQUAL || this.m_compareFunc == gl.NOTEQUAL) ? 1.0 : 0.5;
        /** @type {number} */ var refInRangeLower = (this.m_compareFunc == gl.EQUAL || this.m_compareFunc == gl.NOTEQUAL) ? 0.0 : 0.5;
        /** @type {number} */ var refOutOfBoundsUpper = 1.1; // !< lookup function should clamp values to [0, 1] range
        /** @type {number} */ var refOutOfBoundsLower = -0.1;

        /** @type {Array<{texNdx: number, ref: number, lodX: number, lodY: number, oX: number, oY: number}>} */
        var cases = [{ texNdx: 0, ref: refInRangeUpper, lodX: 1.6, lodY: 2.9, oX: -1.0, oY: -2.7 },{ texNdx: 0, ref: refInRangeLower, lodX: -2.0, lodY: -1.35, oX: -0.2, oY: 0.7 },{ texNdx: 1, ref: refInRangeUpper, lodX: 0.14, lodY: 0.275, oX: -1.5, oY: -1.1 },{ texNdx: 1, ref: refInRangeLower, lodX: -0.92, lodY: -2.64, oX: 0.4, oY: -0.1 },{ texNdx: 1, ref: refOutOfBoundsUpper, lodX: -0.49, lodY: -0.22, oX: 0.45, oY: 0.97 },{ texNdx: 1, ref: refOutOfBoundsLower, lodX: -0.85, lodY: 0.75, oX: 0.25, oY: 0.61 }
        ];

        var viewportW = Math.min(VIEWPORT_WIDTH, gl.canvas.width);
        var viewportH = Math.min(VIEWPORT_HEIGHT, gl.canvas.height);

        /** @type {number} */ var minLayer = -0.5;
        /** @type {number} */ var maxLayer = this.m_numLayers;

        for (var caseNdx = 0; caseNdx < cases.length; caseNdx++) {
            var tex = cases[caseNdx].texNdx > 0 ? this.m_gridTex : this.m_gradientTex;
            /** @type {number} */ var ref = cases[caseNdx].ref;
            /** @type {number} */ var lodX = cases[caseNdx].lodX;
            /** @type {number} */ var lodY = cases[caseNdx].lodY;
            /** @type {number} */ var oX = cases[caseNdx].oX;
            /** @type {number} */ var oY = cases[caseNdx].oY;
            /** @type {number} */ var sX = Math.exp(lodX * Math.LN2) * viewportW / tex.getRefTexture().getWidth();
            /** @type {number} */ var sY = Math.exp(lodY * Math.LN2) * viewportH / tex.getRefTexture().getHeight();

            this.m_cases.push(new es3fTextureShadowTests.FilterCase(tex, ref, [oX, oY, minLayer], [oX + sX, oY + sY, maxLayer]));
        }

        this.m_caseNdx = 0;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureShadowTests.Texture2DArrayShadowCase.prototype.iterate = function() {
        var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), VIEWPORT_WIDTH, VIEWPORT_HEIGHT, deString.deStringHash(this.fullName()) ^ deMath.deMathHash(this.m_caseNdx));
        var curCase = this.m_cases[this.m_caseNdx];
        var sampleParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_2D_ARRAY);
        var rendered = new tcuSurface.Surface(viewport.width, viewport.height);
        var texCoord = [];

        texCoord = [curCase.minCoord[0], curCase.minCoord[1], curCase.minCoord[2],
            curCase.minCoord[0], curCase.maxCoord[1], (curCase.minCoord[2] + curCase.maxCoord[2]) / 2.0,
            curCase.maxCoord[0], curCase.minCoord[1], (curCase.minCoord[2] + curCase.maxCoord[2]) / 2.0,
            curCase.maxCoord[0], curCase.maxCoord[1], curCase.maxCoord[2]];

        if (viewport.width < MIN_VIEWPORT_WIDTH || viewport.height < MIN_VIEWPORT_HEIGHT)
            throw new Error('Too small render target');

        sampleParams.sampler = gluTextureUtil.mapGLSamplerWrapST(this.m_wrapS, this.m_wrapT, this.m_minFilter, this.m_magFilter);
        sampleParams.sampler.compare = gluTextureUtil.mapGLCompareFunc(this.m_compareFunc);
        sampleParams.samplerType = glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW;
        sampleParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        sampleParams.ref = curCase.ref;

        bufferedLogToConsole(
            'Compare reference value = ' + sampleParams.ref + '\n' +
            'Texture Coordinates: ' + curCase.minCoord + ' -> ' + curCase.maxCoord
        );

        gl.bindTexture(gl.TEXTURE_2D_ARRAY, curCase.texture.getGLTexture());
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, this.m_minFilter);
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, this.m_magFilter);
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, this.m_wrapT);
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_COMPARE_FUNC, this.m_compareFunc);

        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        this.m_renderer.renderQuad(0, texCoord, sampleParams);
        rendered.readViewport(gl, viewport);

        var pixelFormat = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        /** @type {tcuTexLookupVerifier.LodPrecision} */ var lodPrecision = new tcuTexLookupVerifier.LodPrecision(18, 6);
        /** @type {tcuTexCompareVerifier.TexComparePrecision} */ var texComparePrecision = new tcuTexCompareVerifier.TexComparePrecision(
            [20, 20, 20],
            [7, 7, 7],
            5,
            16,
            pixelFormat.redBits - 1
        );

        var isHighQuality = es3fTextureShadowTests.verifyTexCompareResult(tcuTexture.Texture2DArray, rendered.getAccess(), curCase.texture.getRefTexture(),
                                                          texCoord, sampleParams, texComparePrecision, lodPrecision, pixelFormat);

        if (!isHighQuality) {
            bufferedLogToConsole('Warning: Verification assuming high-quality PCF filtering failed');

            lodPrecision.lodBits = 4;
            texComparePrecision.uvwBits = [4, 4, 4];
            texComparePrecision.pcfBits = 0;

            var isOk = es3fTextureShadowTests.verifyTexCompareResult(tcuTexture.Texture2DArray, rendered.getAccess(), curCase.texture.getRefTexture(),
                                                                                              texCoord, sampleParams, texComparePrecision, lodPrecision, pixelFormat);

            if (!isOk) {
                bufferedLogToConsole('ERROR: Verification against low precision requirements failed, failing test case.');
                testFailedOptions('Image verification failed', false);
            } else
                testPassedOptions('Low-quality result', true);
        } else
            testPassedOptions('High-quality result', true);

        this.m_caseNdx += 1;
        return this.m_caseNdx < this.m_cases.length ? tcuTestCase.IterateResult.CONTINUE : tcuTestCase.IterateResult.STOP;
    };

    es3fTextureShadowTests.init = function() {
        /** @type {Array<es3fTextureShadowTests.Format>} */ var formats = [];
        formats.push(new es3fTextureShadowTests.Format('depth_component16', gl.DEPTH_COMPONENT16));
        formats.push(new es3fTextureShadowTests.Format('depth_component32f', gl.DEPTH_COMPONENT32F));
        formats.push(new es3fTextureShadowTests.Format('depth24_stencil8', gl.DEPTH24_STENCIL8));

        /** @type {Array<es3fTextureShadowTests.Filter>} */ var filters = [];
        filters.push(new es3fTextureShadowTests.Filter('nearest', gl.NEAREST, gl.NEAREST));
        filters.push(new es3fTextureShadowTests.Filter('linear', gl.LINEAR, gl.LINEAR));
        filters.push(new es3fTextureShadowTests.Filter('nearest_mipmap_nearest', gl.NEAREST_MIPMAP_NEAREST, gl.LINEAR));
        filters.push(new es3fTextureShadowTests.Filter('linear_mipmap_nearest', gl.LINEAR_MIPMAP_NEAREST, gl.LINEAR));
        filters.push(new es3fTextureShadowTests.Filter('nearest_mipmap_linear', gl.NEAREST_MIPMAP_LINEAR, gl.LINEAR));
        filters.push(new es3fTextureShadowTests.Filter('linear_mipmap_linear', gl.LINEAR_MIPMAP_LINEAR, gl.LINEAR));

        /** @type {Array<es3fTextureShadowTests.CompareFunc>} */ var compareFuncs = [];
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('less_or_equal', gl.LEQUAL));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('greater_or_equal', gl.GEQUAL));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('less', gl.LESS));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('greater', gl.GREATER));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('equal', gl.EQUAL));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('not_equal', gl.NOTEQUAL));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('always', gl.ALWAYS));
        compareFuncs.push(new es3fTextureShadowTests.CompareFunc('never', gl.NEVER));

        var state = tcuTestCase.runner;
        /** @type {tcuTestCase.DeqpTest} */ var testGroup = state.testCases;

        for (var filterNdx = 0; filterNdx < filters.length; filterNdx++) {
            for (var compareNdx = 0; compareNdx < compareFuncs.length; compareNdx++) {
                var filterGroup = tcuTestCase.newTest(
                    '2d.' + filters[filterNdx].name, '2D texture shadow lookup tests');
                testGroup.addChild(filterGroup);

                for (var formatNdx = 0; formatNdx < formats.length; formatNdx++) {
                    /** @type {number} */ var minFilter = filters[filterNdx].minFilter;
                    /** @type {number} */ var magFilter = filters[filterNdx].magFilter;
                    /** @type {number} */ var format = formats[formatNdx].format;
                    /** @type {number} */ var compareFunc = compareFuncs[compareNdx].func;
                    /** @type {number} */ var wrapS = gl.REPEAT;
                    /** @type {number} */ var wrapT = gl.REPEAT;
                    /** @type {number} */ var width = 32;
                    /** @type {number} */ var height = 64;
                    /** @type {string} */ var name = compareFuncs[compareNdx].name + '_' + formats[formatNdx].name;

                    filterGroup.addChild(new es3fTextureShadowTests.Texture2DShadowCase(name, '', minFilter, magFilter, wrapS, wrapT, format, width, height, compareFunc));
                }
            }
        }

        for (filterNdx = 0; filterNdx < filters.length; filterNdx++) {
            for (compareNdx = 0; compareNdx < compareFuncs.length; compareNdx++) {
                filterGroup = tcuTestCase.newTest(
                    'cube.' + filters[filterNdx].name, 'Cube map texture shadow lookup tests');
                testGroup.addChild(filterGroup);

                for (formatNdx = 0; formatNdx < formats.length; formatNdx++) {
                    minFilter = filters[filterNdx].minFilter;
                    magFilter = filters[filterNdx].magFilter;
                    format = formats[formatNdx].format;
                    compareFunc = compareFuncs[compareNdx].func;
                    wrapS = gl.REPEAT;
                    wrapT = gl.REPEAT;
                    var size = 32;
                    name = compareFuncs[compareNdx].name + '_' + formats[formatNdx].name;

                    filterGroup.addChild(new es3fTextureShadowTests.TextureCubeShadowCase(name, '', minFilter, magFilter, wrapS, wrapT, format, size, compareFunc));
                }
            }
        }

        for (var filterNdx = 0; filterNdx < filters.length; filterNdx++) {
            for (var compareNdx = 0; compareNdx < compareFuncs.length; compareNdx++) {
                filterGroup = tcuTestCase.newTest(
                    '2d_array.' + filters[filterNdx].name, '2D texture array shadow lookup tests');
                testGroup.addChild(filterGroup);

                for (var formatNdx = 0; formatNdx < formats.length; formatNdx++) {
                    minFilter = filters[filterNdx].minFilter;
                    magFilter = filters[filterNdx].magFilter;
                    format = formats[formatNdx].format;
                    compareFunc = compareFuncs[compareNdx].func;
                    wrapS = gl.REPEAT;
                    wrapT = gl.REPEAT;
                    width = 32;
                    height = 64;
                    var numLayers = 8;
                    name = compareFuncs[compareNdx].name + '_' + formats[formatNdx].name;

                    filterGroup.addChild(new es3fTextureShadowTests.Texture2DArrayShadowCase(name, '', minFilter, magFilter, wrapS, wrapT, format, width, height, numLayers, compareFunc));
                }
            }
        }
    };

    es3fTextureShadowTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'texture_shadow';
        var testDescription = 'Texture Shadow Test';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fTextureShadowTests.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };
});

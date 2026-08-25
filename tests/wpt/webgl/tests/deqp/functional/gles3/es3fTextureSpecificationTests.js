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
goog.provide('functional.gles3.es3fTextureSpecificationTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {
    var es3fTextureSpecificationTests =
        functional.gles3.es3fTextureSpecificationTests;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;
    var deString = framework.delibs.debase.deString;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var sglrReferenceContext =
        framework.opengl.simplereference.sglrReferenceContext;
    var rrUtil = framework.referencerenderer.rrUtil;
    var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
    var tcuLogImage = framework.common.tcuLogImage;

    /**
     * @param {number} internalFormat
     * @return {tcuTexture.TextureFormat}
     */
    es3fTextureSpecificationTests.mapGLUnsizedInternalFormat = function(
        internalFormat
    ) {
        switch (internalFormat) {
            case gl.ALPHA:
                return new tcuTexture.TextureFormat(
                    tcuTexture.ChannelOrder.A,
                    tcuTexture.ChannelType.UNORM_INT8
                );
            case gl.LUMINANCE:
                return new tcuTexture.TextureFormat(
                    tcuTexture.ChannelOrder.L,
                    tcuTexture.ChannelType.UNORM_INT8
                );
            case gl.LUMINANCE_ALPHA:
                return new tcuTexture.TextureFormat(
                    tcuTexture.ChannelOrder.LA,
                    tcuTexture.ChannelType.UNORM_INT8
                );
            case gl.RGB:
                return new tcuTexture.TextureFormat(
                    tcuTexture.ChannelOrder.RGB,
                    tcuTexture.ChannelType.UNORM_INT8
                );
            case gl.RGBA:
                return new tcuTexture.TextureFormat(
                    tcuTexture.ChannelOrder.RGBA,
                    tcuTexture.ChannelType.UNORM_INT8
                );
            default:
                throw new Error(
                    'Can\'t map GL unsized internal format (' +
                    internalFormat.toString(16) + ') to texture format'
                );
        }
    };

    var VIEWPORT_WIDTH = 256;
    var VIEWPORT_HEIGHT = 256;

    /**
     * @param {number} width
     * @param {number} height
     * @param {number=} depth
     * @return {number}
     */
    es3fTextureSpecificationTests.maxLevelCount = function(
        width, height, depth
    ) {
        depth = depth || 0;
        return deMath.logToFloor(Math.max(width, Math.max(height, depth))) + 1;
    };

    /**
     * @param {deRandom.Random} rnd
     * @param {Array<number>} minVal
     * @param {Array<number>} maxVal
     * @param {number} size
     * @return {Array<number>}
     */
    es3fTextureSpecificationTests.randomVector = function(
        rnd, minVal, maxVal, size
    ) {
        var res = [];
        for (var ndx = 0; ndx < size; ndx++)
            res[ndx] = rnd.getFloat(minVal[ndx], maxVal[ndx]);
        return res;
    };

    /**
     * @param {tcuPixelFormat.PixelFormat} pixelFormat
     * @param {tcuTexture.TextureFormat} textureFormat
     * @return {Array<number>} (ivec4)
     */
    es3fTextureSpecificationTests.getPixelFormatCompareDepth = function(
        pixelFormat, textureFormat
    ) {
        switch (textureFormat.order) {
            case tcuTexture.ChannelOrder.L:
            case tcuTexture.ChannelOrder.LA:
                return [
                    pixelFormat.redBits, pixelFormat.redBits,
                    pixelFormat.redBits, pixelFormat.alphaBits
                ];
            default:
                return [
                    pixelFormat.redBits, pixelFormat.greenBits,
                    pixelFormat.blueBits, pixelFormat.alphaBits
                ];
        }
    };

    /**
     * @param {tcuPixelFormat.PixelFormat} pixelFormat
     * @param {tcuTexture.TextureFormat} textureFormat
     * @return {Array<number>} (uvec4)
     */
    es3fTextureSpecificationTests.computeCompareThreshold = function(
        pixelFormat, textureFormat
    ) {
        /** @type {Array<number>} */
        var texFormatBits = tcuTextureUtil.getTextureFormatBitDepth(
            textureFormat
        );
        /** @type {Array<number>} */
        var pixelFormatBits =
            es3fTextureSpecificationTests.getPixelFormatCompareDepth(
                pixelFormat, textureFormat
            );
        /** @type {Array<number>} */
        var accurateFmtBits = deMath.min(pixelFormatBits, texFormatBits);
        /** @type {Array<number>} */
        var compareBits = deMath.addScalar(
            tcuTextureUtil.select(
                accurateFmtBits, [8, 8, 8, 8],
                deMath.greaterThan(accurateFmtBits, [0, 0, 0, 0])
            ), - 1
        );

        var result = [];
        for (var i = 0; i < compareBits.length; i++)
            result.push(1 << compareBits[i]);
        return result;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * context
     */
    es3fTextureSpecificationTests.TextureSpecCase = function(name, desc) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        /**
         * @type {
         *     ?sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext
         * }
         */
         this.m_context = null;
    };

    es3fTextureSpecificationTests.TextureSpecCase.prototype = Object.create(
        tcuTestCase.DeqpTest.prototype
    );

    es3fTextureSpecificationTests.TextureSpecCase.prototype.constructor =
        es3fTextureSpecificationTests.TextureSpecCase;

    /**
     * deinit
     */
    es3fTextureSpecificationTests.TextureSpecCase.prototype.deinit =
    function() {
        gl.pixelStorei(gl.UNPACK_ALIGNMENT, 4);
        gl.pixelStorei(gl.UNPACK_ROW_LENGTH, 0);
        gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_IMAGES, 0);

        gl.bindBuffer(gl.PIXEL_UNPACK_BUFFER, null);
    }

    /**
     * createTexture - Needs to be overridden
     */
    es3fTextureSpecificationTests.TextureSpecCase.prototype.createTexture =
    function() {
        throw new Error('Must override');
    };

    /**
     * verifyTexture - Needs to be overridden
     * @param {sglrGLContext.GLContext} webgl2Context
     * @param {sglrReferenceContext.ReferenceContext} refContext
     * @return {boolean}
     */
    es3fTextureSpecificationTests.TextureSpecCase.prototype.verifyTexture =
    function(
        webgl2Context, refContext
    ) {
        throw new Error('Must override');
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureSpecificationTests.TextureSpecCase.prototype.iterate = function() {
        if (gl.canvas.width < VIEWPORT_WIDTH ||
            gl.canvas.height < VIEWPORT_HEIGHT)
            throw new Error('Too small viewport', '');

        // Context size, and viewport for GLES3
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var width = Math.min(gl.canvas.width, VIEWPORT_WIDTH);
        var height = Math.min(gl.canvas.height, VIEWPORT_HEIGHT);
        var x = rnd.getInt(0, gl.canvas.width - width);
        var y = rnd.getInt(0, gl.canvas.height - height);

        // Contexts.
        /** @type {sglrGLContext.GLContext} */
        var webgl2Context = new sglrGLContext.GLContext(
            gl, [x, y, width, height]
        );

        /** @type {sglrReferenceContext.ReferenceContextBuffers} */
        var refBuffers = new sglrReferenceContext.ReferenceContextBuffers(
            new tcuPixelFormat.PixelFormat(
                8, 8, 8, gl.getParameter(gl.ALPHA_BITS) ? 8 : 0
            ), 0 /* depth */, 0 /* stencil */, width, height
        );

        /** @type {sglrReferenceContext.ReferenceContext} */
        var refContext = new sglrReferenceContext.ReferenceContext(
            new sglrReferenceContext.ReferenceContextLimits(gl),
            refBuffers.getColorbuffer(), refBuffers.getDepthbuffer(),
            refBuffers.getStencilbuffer()
        );

        // Clear color buffer.
        for (var ndx = 0; ndx < 2; ndx++) {
            this.m_context = ndx ? refContext : webgl2Context;
            // C++ port uses (0.125, 0.25, 0.5, 1.0), but here we use (0, 0, 0, 0)
            // in order to optimize the `clear' op in ReferenceContext.
            this.m_context.clearColor(0, 0, 0, 0);
            this.m_context.clear(
                gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT |
                gl.STENCIL_BUFFER_BIT
            );
        }

        // Construct texture using both GLES3 and reference contexts.
        for (var ndx = 0; ndx < 2; ndx++) {
            this.m_context = ndx ? refContext : webgl2Context;
            this.createTexture();
            checkMessage(
                this.m_context.getError() == gl.NO_ERROR,
                'Problem creating texture.'
            );
        }

        // Verify results.
        if (this.verifyTexture(webgl2Context, refContext))
            testPassed();
        else
            testFailed('Verification failed');

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @param {tcuSurface.Surface} dst
     * @param {
     *  ?WebGLProgram|
     *  framework.opengl.simplereference.sglrShaderProgram.ShaderProgram
     * } program
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.TextureSpecCase.prototype.renderTex =
    function(dst, program, width, height) {
        var targetW = this.m_context.getWidth();
        var targetH = this.m_context.getHeight();

        var w = width / targetW;
        var h = height / targetH;

        rrUtil.drawQuad(
            this.m_context, program, [-1.0, -1.0, 0.0],
            [-1.0 + w * 2.0, -1.0 + h * 2.0, 0.0]
        );

        // Read pixels back.
        dst.readViewport(this.m_context, [0, 0, width, height]);
    };

    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {tcuTexture.TextureFormat} format
     * @param {number} width
     * @param {number} height
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.Texture2DSpecCase = function(
        name, desc, format, width, height, numLevels
    ) {
        es3fTextureSpecificationTests.TextureSpecCase.call(this, name, desc);

        this.m_texFormat = format;
        this.m_texFormatInfo = tcuTextureUtil.getTextureFormatInfo(format);
        this.m_width = width;
        this.m_height = height;
        this.m_numLevels = numLevels;
    };

    es3fTextureSpecificationTests.Texture2DSpecCase.prototype = Object.create(
        es3fTextureSpecificationTests.TextureSpecCase.prototype
    );

    es3fTextureSpecificationTests.Texture2DSpecCase.prototype.constructor =
        es3fTextureSpecificationTests.Texture2DSpecCase;

    /**
     * @param {sglrGLContext.GLContext} webgl2Context
     * @param {sglrReferenceContext.ReferenceContext} refContext
     */
    es3fTextureSpecificationTests.Texture2DSpecCase.prototype.verifyTexture =
    function(
        webgl2Context, refContext
    ) {
        /** @type {es3fFboTestUtil.Texture2DShader} */
        var shader = new es3fFboTestUtil.Texture2DShader(
            [gluTextureUtil.getSampler2DType(this.m_texFormat)],
            gluShaderUtil.DataType.FLOAT_VEC4
        );

        var shaderIDgles = webgl2Context.createProgram(shader);
        var shaderIDRef = refContext.createProgram(shader);

        shader.setTexScaleBias(
            0, this.m_texFormatInfo.lookupScale,
            this.m_texFormatInfo.lookupBias
        );

        // Set state.
        for (var ndx = 0; ndx < 2; ndx++) {
            var ctx = ndx ? refContext : webgl2Context;

            this.m_context = ctx;

            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, this.m_numLevels - 1
            );
        }

        for (var levelNdx = 0; levelNdx < this.m_numLevels; levelNdx++) {
            var levelW = Math.max(1, this.m_width >> levelNdx);
            var levelH = Math.max(1, this.m_height >> levelNdx);
            /** @type {tcuSurface.Surface} */ var reference = new tcuSurface.Surface();
            /** @type {tcuSurface.Surface} */ var result = new tcuSurface.Surface();

            for (var ndx = 0; ndx < 2; ndx++) {
                /** @type {tcuSurface.Surface} */
                var dst = ndx ? reference : result;
                var ctx = ndx ? refContext : webgl2Context;
                var shaderID = ndx ? shaderIDRef : shaderIDgles;

                this.m_context = ctx;
                shader.setUniforms(ctx, shaderID);
                this.renderTex(dst, shaderID, levelW, levelH);
            }

            var threshold =
            es3fTextureSpecificationTests.computeCompareThreshold(
                tcuPixelFormat.PixelFormatFromContext(gl), this.m_texFormat
            );
            var levelStr = levelNdx.toString();
            var name = 'Level' + levelStr;
            var desc = 'Level ' + levelStr;
            var isOk = tcuImageCompare.intThresholdCompare(
                name, desc, reference.getAccess(), result.getAccess(),
                threshold, levelNdx == 0 ?
                tcuImageCompare.CompareLogMode.RESULT :
                tcuImageCompare.CompareLogMode.ON_ERROR
            );

            if (!isOk) {
                testFailed('Image comparison failed');
                return false;
            } else {
                // tcuLogImage.logImageWithInfo(result.getAccess(),'Comparison OK on level: ' + levelNdx);
            }
        }
        return true;
    };

    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {tcuTexture.TextureFormat} format
     * @param {number} size
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.TextureCubeSpecCase = function(
        name, desc, format, size, numLevels
    ) {
        es3fTextureSpecificationTests.TextureSpecCase.call(
            this, name, desc
        );
        this.m_texFormat = format;
        this.m_texFormatInfo = tcuTextureUtil.getTextureFormatInfo(format);
        this.m_size = size;
        this.m_numLevels = numLevels;
    };

    es3fTextureSpecificationTests.TextureCubeSpecCase.prototype =
        Object.create(es3fTextureSpecificationTests.TextureSpecCase.prototype);

    es3fTextureSpecificationTests.TextureCubeSpecCase.prototype.constructor =
        es3fTextureSpecificationTests.TextureCubeSpecCase;

    /**
     * @param {sglrGLContext.GLContext} webgl2Context
     * @param {sglrReferenceContext.ReferenceContext} refContext
     */
    es3fTextureSpecificationTests.TextureCubeSpecCase.prototype.verifyTexture =
    function(
        webgl2Context, refContext
    ) {
        /** @type {es3fFboTestUtil.TextureCubeShader} */
        var shader = new es3fFboTestUtil.TextureCubeShader(
            gluTextureUtil.getSamplerCubeType(this.m_texFormat),
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderIDgles = webgl2Context.createProgram(shader);
        var shaderIDRef = refContext.createProgram(shader);

        shader.setTexScaleBias(
            this.m_texFormatInfo.lookupScale, this.m_texFormatInfo.lookupBias
        );

        // Set state.
        for (var ndx = 0; ndx < 2; ndx++) {
            var ctx = ndx ? refContext : webgl2Context;

            this.m_context = ctx;

            this.m_context.texParameteri(
                gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER,
                gl.NEAREST_MIPMAP_NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER,
                gl.NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAX_LEVEL,
                this.m_numLevels - 1
            );
        }

        for (var levelNdx = 0; levelNdx < this.m_numLevels; levelNdx++) {
            var levelSize = Math.max(1, this.m_size >> levelNdx);
            var isOk = true;

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                /** @type {tcuSurface.Surface} */
                var reference = new tcuSurface.Surface();
                /** @type {tcuSurface.Surface} */
                var result = new tcuSurface.Surface();

                if (levelSize <= 2)
                    continue; // Fuzzy compare doesn't work for images this small.

                shader.setFace(face);

                for (var ndx = 0; ndx < 2; ndx++) {
                    /** @type {tcuSurface.Surface} */
                    var dst = ndx ? reference : result;
                    ctx = ndx ? refContext : webgl2Context;
                    var shaderID = ndx ? shaderIDRef : shaderIDgles;

                    this.m_context = ctx;
                    shader.setUniforms(ctx, shaderID);
                    this.renderTex(dst, shaderID, levelSize, levelSize);
                }

                var threshold = 0.02;
                var faceStr = face.toString();
                var levelStr = levelNdx.toString();
                var name = 'Level' + levelStr;
                var desc = 'Level ' + levelStr + ', face ' + faceStr;
                var isFaceOk = tcuImageCompare.fuzzyCompare(
                    name, desc, reference.getAccess(), result.getAccess(),
                    threshold, levelNdx == 0 ?
                    tcuImageCompare.CompareLogMode.RESULT :
                    tcuImageCompare.CompareLogMode.ON_ERROR
                );

                if (!isFaceOk) {
                    testFailed('Image comparison failed');
                    return false;
                }
            }

        }
        return true;
    };

    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {tcuTexture.TextureFormat} format
     * @param {number} width
     * @param {number} height
     * @param {number} numLayers
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.Texture2DArraySpecCase = function(
        name, desc, format, width, height, numLayers, numLevels
    ) {
        es3fTextureSpecificationTests.TextureSpecCase.call(
            this, name, desc
        );
        this.m_texFormat = format;
        this.m_texFormatInfo = tcuTextureUtil.getTextureFormatInfo(format);
        this.m_width = width;
        this.m_height = height;
        this.m_numLayers = numLayers;
        this.m_numLevels = numLevels;
    };

    es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype =
        Object.create(es3fTextureSpecificationTests.TextureSpecCase.prototype);

    es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype.constructor =
        es3fTextureSpecificationTests.Texture2DArraySpecCase;

    /**
     * @param {sglrGLContext.GLContext} webgl2Context
     * @param {sglrReferenceContext.ReferenceContext} refContext
     */
    es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype.verifyTexture =
    function(
        webgl2Context, refContext
    ) {
        /** @type {es3fFboTestUtil.Texture2DArrayShader} */
        var shader = new es3fFboTestUtil.Texture2DArrayShader(
            gluTextureUtil.getSampler2DArrayType(this.m_texFormat),
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderIDgles = webgl2Context.createProgram(shader);
        var shaderIDRef = refContext.createProgram(shader);

        shader.setTexScaleBias(
            this.m_texFormatInfo.lookupScale, this.m_texFormatInfo.lookupBias
        );

        // Set state.
        for (var ndx = 0; ndx < 2; ndx++) {
            var ctx = ndx ? refContext : webgl2Context;

            this.m_context = ctx;

            this.m_context.texParameteri(
                gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER,
                gl.NEAREST_MIPMAP_NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER,
                gl.NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_R,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAX_LEVEL,
                this.m_numLevels - 1
            );
        }

        for (var layerNdx = 0; layerNdx < this.m_numLayers; layerNdx++) {
            var layerOk = true;

            shader.setLayer(layerNdx);
            for (var levelNdx = 0; levelNdx < this.m_numLevels; levelNdx++) {
                var levelW = Math.max(1, this.m_width >> levelNdx);
                var levelH = Math.max(1, this.m_height >> levelNdx);

                if (levelW == 1 || levelH == 1) {
                    // Rendering to texture of size x1 is problematic in referencerenderer
                    // due to its deviation from c++ code: crbug.com/613206
                    continue;
                }
                /** @type {tcuSurface.Surface} */
                var reference = new tcuSurface.Surface();
                /** @type {tcuSurface.Surface} */
                var result = new tcuSurface.Surface();

                var isOk = true;

                for (var ndx = 0; ndx < 2; ndx++) {
                    /** @type {tcuSurface.Surface} */
                    var dst = ndx ? reference : result;
                    ctx = ndx ? refContext : webgl2Context;
                    var shaderID = ndx ? shaderIDRef : shaderIDgles;

                    this.m_context = ctx;
                    shader.setUniforms(ctx, shaderID);
                    this.renderTex(dst, shaderID, levelW, levelH);
                }

                var threshold =
                es3fTextureSpecificationTests.computeCompareThreshold(
                    tcuPixelFormat.PixelFormatFromContext(gl), this.m_texFormat
                );
                var levelStr = levelNdx.toString();
                var layerStr = layerNdx.toString();
                var name = 'Layer' + layerStr + 'Level' + levelStr;
                var desc = 'Layer ' + layerStr + ', Level ' + levelStr;
                var depthOk = tcuImageCompare.intThresholdCompare(
                    name, desc, reference.getAccess(), result.getAccess(),
                    threshold, (levelNdx == 0 && layerNdx == 0) ?
                    tcuImageCompare.CompareLogMode.RESULT :
                    tcuImageCompare.CompareLogMode.ON_ERROR
                );

                if (!depthOk) {
                    testFailed('Image comparison failed');
                    return false;
                }
            }

        }
        return true;
    };

    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {tcuTexture.TextureFormat} format
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.Texture3DSpecCase = function(
        name, desc, format, width, height, depth, numLevels
    ) {
        es3fTextureSpecificationTests.TextureSpecCase.call(
            this, name, desc
        );
        this.m_texFormat = format;
        this.m_texFormatInfo = tcuTextureUtil.getTextureFormatInfo(format);
        this.m_width = width;
        this.m_height = height;
        this.m_depth = depth;
        this.m_numLevels = numLevels;
    };

    es3fTextureSpecificationTests.Texture3DSpecCase.prototype =
        Object.create(es3fTextureSpecificationTests.TextureSpecCase.prototype);

    es3fTextureSpecificationTests.Texture3DSpecCase.prototype.constructor =
        es3fTextureSpecificationTests.Texture3DSpecCase;

    /**
     * @param {sglrGLContext.GLContext} webgl2Context
     * @param {sglrReferenceContext.ReferenceContext} refContext
     */
    es3fTextureSpecificationTests.Texture3DSpecCase.prototype.verifyTexture =
    function(
        webgl2Context, refContext
    ) {
        /** @type {es3fFboTestUtil.Texture3DShader} */
        var shader = new es3fFboTestUtil.Texture3DShader(
            gluTextureUtil.getSampler3D(this.m_texFormat),
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderIDgles = webgl2Context.createProgram(shader);
        var shaderIDRef = refContext.createProgram(shader);

        shader.setTexScaleBias(
            this.m_texFormatInfo.lookupScale, this.m_texFormatInfo.lookupBias
        );

        // Set state.
        for (var ndx = 0; ndx < 2; ndx++) {
            var ctx = ndx ? refContext : webgl2Context;

            this.m_context = ctx;

            this.m_context.texParameteri(
                gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER,
                gl.NEAREST_MIPMAP_NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER,
                gl.NEAREST
            );
            this.m_context.texParameteri(
                gl.TEXTURE_3D, gl.TEXTURE_WRAP_S,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_3D, gl.TEXTURE_WRAP_T,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_3D, gl.TEXTURE_WRAP_R,
                gl.CLAMP_TO_EDGE
            );
            this.m_context.texParameteri(
                gl.TEXTURE_3D, gl.TEXTURE_MAX_LEVEL,
                this.m_numLevels - 1
            );
        }

        for (var levelNdx = 0; levelNdx < this.m_numLevels; levelNdx++) {
            var levelW = Math.max(1, this.m_width >> levelNdx);
            var levelH = Math.max(1, this.m_height >> levelNdx);
            var levelD = Math.max(1, this.m_depth >> levelNdx);
            var levelOk = true;

            if (levelW == 1 || levelH == 1) {
                // Rendering to texture of size x1 is problematic in referencerenderer
                // due to its deviation from c++ code: crbug.com/613206
                continue;
            }

            for (var depth = 0; depth < levelD; depth++) {
                /** @type {tcuSurface.Surface} */
                var reference = new tcuSurface.Surface();
                /** @type {tcuSurface.Surface} */
                var result = new tcuSurface.Surface();

                shader.setDepth((depth + 0.5) / levelD);

                for (var ndx = 0; ndx < 2; ndx++) {
                    /** @type {tcuSurface.Surface} */
                    var dst = ndx ? reference : result;
                    ctx = ndx ? refContext : webgl2Context;
                    var shaderID = ndx ? shaderIDRef : shaderIDgles;

                    this.m_context = ctx;
                    shader.setUniforms(ctx, shaderID);
                    this.renderTex(dst, shaderID, levelW, levelH);
                }

                var threshold =
                es3fTextureSpecificationTests.computeCompareThreshold(
                    tcuPixelFormat.PixelFormatFromContext(gl), this.m_texFormat
                );
                var levelStr = levelNdx.toString();
                var sliceStr = depth.toString();
                var name = 'Layer' + levelStr + 'Slice' + sliceStr;
                var desc = 'Layer ' + levelStr + ', Slice ' + sliceStr;
                var depthOk = tcuImageCompare.intThresholdCompare(
                    name, desc, reference.getAccess(), result.getAccess(),
                    threshold, (levelNdx == 0 && depth == 0) ?
                    tcuImageCompare.CompareLogMode.RESULT :
                    tcuImageCompare.CompareLogMode.ON_ERROR
                );

                if (!depthOk) {
                    testFailed('Image comparison failed');
                    return false;
                }
            }

        }
        return true;
    };

    // Basic TexImage2D() with 2D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.BasicTexImage2DCase = function(
        name, desc, format, dataType, width, height
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height,
            es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.BasicTexImage2DCase.prototype = Object.create(
        es3fTextureSpecificationTests.Texture2DSpecCase.prototype
    );

    es3fTextureSpecificationTests.BasicTexImage2DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexImage2DCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @return {es3fTextureSpecificationTests.BasicTexImage2DCase}
     */
    es3fTextureSpecificationTests.newBasicTexImage2DCaseInternal = function(
        name, desc, internalFormat, width, height
    ) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.BasicTexImage2DCase(
            name, desc, fmt.format, fmt.dataType, width, height
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.BasicTexImage2DCase.prototype.createTexture =
    function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, levelW, levelH, 0,
                this.m_format, this.m_dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // Basic TexImage2D() with cubemap usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     */
    es3fTextureSpecificationTests.BasicTexImageCubeCase = function(
        name, desc, format, dataType, size
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, deMath.logToFloor(size) + 1
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.BasicTexImageCubeCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicTexImageCubeCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexImageCubeCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @return {es3fTextureSpecificationTests.BasicTexImageCubeCase}
     */
    es3fTextureSpecificationTests.newBasicTexImageCubeCaseInternal = function(
        name, desc, internalFormat, size
    ) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.BasicTexImageCubeCase(
            name, desc, fmt.format, fmt.dataType, size
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexImageCubeCase.prototype.createTexture =
    function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_size, this.m_size
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            levelData.setSize(levelSize, levelSize);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                var gMin = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );
                var gMax = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );

                tcuTextureUtil.fillWithComponentGradients(
                    levelData.getAccess(), gMin, gMax
                );

                this.m_context.texImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, this.m_internalFormat, levelSize, levelSize, 0,
                    this.m_format, this.m_dataType,
                    levelData.getAccess().getDataPtr()
                );
            }
        }
    };

    // Basic TexImage3D() with 2D array texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} numLayers
     */
    es3fTextureSpecificationTests.BasicTexImage2DArrayCase = function(
        name, desc, internalFormat, width, height, numLayers
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat), width, height,
            numLayers, es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexImage2DArrayCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicTexImage2DArrayCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexImage2DArrayCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexImage2DArrayCase.prototype.createTexture =
    function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH, this.m_numLayers);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage3D(
                gl.TEXTURE_2D_ARRAY, ndx, this.m_internalFormat, levelW, levelH,
                this.m_numLayers, 0, transferFmt.format,
                transferFmt.dataType, levelData.getAccess().getDataPtr()
            );
        }
    };

    // Basic TexImage3D() with 3D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     */
    es3fTextureSpecificationTests.BasicTexImage3DCase = function(
        name, desc, internalFormat, width, height, depth
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat), width, height,
            depth, es3fTextureSpecificationTests.maxLevelCount(
                width, height, depth
            )
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexImage3DCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture3DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicTexImage3DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexImage3DCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexImage3DCase.prototype.createTexture =
    function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var levelD = Math.max(1, this.m_depth >> ndx);

            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH, levelD);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage3D(
                gl.TEXTURE_3D, ndx, this.m_internalFormat, levelW, levelH,
                levelD, 0, transferFmt.format, transferFmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // Randomized 2D texture specification using TexImage2D
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.RandomOrderTexImage2DCase = function(
        name, desc, format, dataType, width, height
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height,
            es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.RandomOrderTexImage2DCase.prototype = Object.create(
        es3fTextureSpecificationTests.Texture2DSpecCase.prototype
    );

    es3fTextureSpecificationTests.
    RandomOrderTexImage2DCase.prototype.constructor =
        es3fTextureSpecificationTests.RandomOrderTexImage2DCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @return {es3fTextureSpecificationTests.RandomOrderTexImage2DCase}
     */
    es3fTextureSpecificationTests.newRandomOrderTexImage2DCaseInternal =
    function(name, desc, internalFormat, width, height) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.RandomOrderTexImage2DCase(
            name, desc, fmt.format, fmt.dataType, width, height
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    RandomOrderTexImage2DCase.prototype.createTexture = function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        var levels = [];
        for (var i = 0; i < this.m_numLevels; i++)
            levels[i] = i;
        levels = rnd.shuffle(levels);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelNdx = levels[ndx];
            var levelW = Math.max(1, this.m_width >> levelNdx);
            var levelH = Math.max(1, this.m_height >> levelNdx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, levelNdx, this.m_internalFormat, levelW, levelH, 0,
                this.m_format, this.m_dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // Randomized cubemap texture specification using TexImage2D
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     */
    es3fTextureSpecificationTests.RandomOrderTexImageCubeCase = function(
        name, desc, format, dataType, size
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, deMath.logToFloor(size) + 1
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.RandomOrderTexImageCubeCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    RandomOrderTexImageCubeCase.prototype.constructor =
        es3fTextureSpecificationTests.RandomOrderTexImageCubeCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @return {es3fTextureSpecificationTests.RandomOrderTexImageCubeCase}
     */
    es3fTextureSpecificationTests.newRandomOrderTexImageCubeCaseInternal =
    function(name, desc, internalFormat, size) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.RandomOrderTexImageCubeCase(
            name, desc, fmt.format, fmt.dataType, size
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    RandomOrderTexImageCubeCase.prototype.createTexture =
    function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_size, this.m_size
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // Level-face pairs.
        var images = [];

        for (var ndx = 0; ndx < this.m_numLevels; ndx++)
            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                images[ndx * 6 + face] = {ndx: ndx, face: face};
            }

        images = rnd.shuffle(images);

        for (var ndx = 0; ndx < images.length; ndx++) {
            var levelNdx = images[ndx].ndx;
            /** @type {framework.common.tcuTexture.CubeFace} */
            var face = images[ndx].face;
            var levelSize = Math.max(1, this.m_size >> levelNdx);

            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelSize, levelSize);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage2D(
                es3fTextureSpecificationTests.s_cubeMapFaces[face],
                levelNdx, this.m_internalFormat, levelSize, levelSize, 0,
                this.m_format, this.m_dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // TexImage2D() unpack alignment case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     * @param {number} numLevels
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexImage2DAlignCase = function(
        name, desc, format, dataType, width, height, numLevels, alignment
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height, numLevels
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexImage2DAlignCase.prototype = Object.create(
        es3fTextureSpecificationTests.Texture2DSpecCase.prototype
    );

    es3fTextureSpecificationTests.
    TexImage2DAlignCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DAlignCase;

    /**
    * @param {string} name
    * @param {string} desc
    * @param {number} internalFormat
    * @param {number} width
    * @param {number} height
    * @param {number} numLevels
    * @param {number} alignment
     * @return {es3fTextureSpecificationTests.TexImage2DAlignCase}
     */
    es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal = function(
        name, desc, internalFormat, width, height, numLevels, alignment
    ) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.TexImage2DAlignCase(
            name, desc, fmt.format, fmt.dataType,
            width, height, numLevels, alignment
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.TexImage2DAlignCase.prototype.createTexture =
    function() {
        var tex = null;
        /** @type {ArrayBuffer} */ var data;

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var colorA = deMath.add(
                deMath.multiply(
                    [1.0, 0.0, 0.0, 1.0],
                    deMath.subtract(
                        this.m_texFormatInfo.valueMax,
                        this.m_texFormatInfo.valueMin
                    )
                ), this.m_texFormatInfo.valueMin
            );
            var colorB = deMath.add(
                deMath.multiply(
                    [0.0, 1.0, 0.0, 1.0],
                    deMath.subtract(
                        this.m_texFormatInfo.valueMax,
                        this.m_texFormatInfo.valueMin
                    )
                ), this.m_texFormatInfo.valueMin
            );

            var rowPitch = deMath.deAlign32(
                levelW * this.m_texFormat.getPixelSize(), this.m_alignment
            );
            var cellSize = Math.max(1, Math.min(levelW >> 2, levelH >> 2));
            data = new ArrayBuffer(rowPitch * levelH);
            var access = new tcuTexture.PixelBufferAccess({format: this.m_texFormat, width: levelW,
                    height: levelH, rowPitch: rowPitch, data: data}
                );
            tcuTextureUtil.fillWithGrid(access, cellSize, colorA, colorB
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, levelW, levelH, 0,
                this.m_format, this.m_dataType,
                access.getDataPtr()
            );
        }
    };

    // TexImageCube unpack alignment case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     * @param {number} numLevels
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexImageCubeAlignCase = function(
        name, desc, format, dataType, size, numLevels, alignment
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, numLevels
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexImageCubeAlignCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImageCubeAlignCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImageCubeAlignCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @param {number} numLevels
     * @param {number} alignment
     * @return {es3fTextureSpecificationTests.TexImageCubeAlignCase}
     */
    es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal =
    function(name, desc, internalFormat, size, numLevels, alignment) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.TexImageCubeAlignCase(
            name, desc, fmt.format, fmt.dataType, size, numLevels, alignment
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImageCubeAlignCase.prototype.createTexture =
    function() {
        var tex = null;
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        /** @type {ArrayBuffer} */ var data;

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);
            var cellSize = Math.max(1, levelSize >> 2);
            var rowPitch = deMath.deAlign32(
                this.m_texFormat.getPixelSize() * levelSize, this.m_alignment
            );
            var colorA = deMath.add(deMath.multiply(
                [1.0, 0.0, 0.0, 1.0], deMath.subtract(
                    this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin
                )), this.m_texFormatInfo.valueMin
            );
            var colorB = deMath.add(deMath.multiply(
                [0.0, 1.0, 0.0, 1.0], deMath.subtract(
                    this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin
                )), this.m_texFormatInfo.valueMin
            );

            data = new ArrayBuffer(rowPitch * levelSize);
            var access = new tcuTexture.PixelBufferAccess({format: this.m_texFormat, width: levelSize,
                    height: levelSize, rowPitch: rowPitch, data: data}
                );
            tcuTextureUtil.fillWithGrid(access, cellSize, colorA, colorB
            );

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                this.m_context.texImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, this.m_internalFormat, levelSize, levelSize, 0,
                    this.m_format, this.m_dataType,
                    access.getDataPtr()
                );
            }
        }
    };

    // TexImage2D() unpack parameters case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexImage2DParamsCase = function(
        name, desc, internalFormat, width, height, rowLength, skipRows,
        skipPixels, alignment
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat),
            width, height, 1
        );
        this.m_internalFormat = internalFormat;
        this.m_rowLength = rowLength;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexImage2DParamsCase.prototype =
    Object.create(es3fTextureSpecificationTests.Texture2DSpecCase.prototype);

    es3fTextureSpecificationTests.TexImage2DParamsCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DParamsCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.TexImage2DParamsCase.prototype.createTexture =
    function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_width;
        var rowPitch = deMath.deAlign32(rowLength * pixelSize, this.m_alignment);
        var height = this.m_height + this.m_skipRows;
        var tex = null;
        /** @type {ArrayBuffer} */ var data;

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels different to 1',
            false, true
        );

        // Fill data with grid.
        data = new ArrayBuffer(rowPitch * height + this.m_skipPixels * pixelSize);

        var cScale = deMath.subtract(
            this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin
        );
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply(
                [1.0, 0.0, 0.0, 1.0], cScale
            ), cBias
        );
        var colorB = deMath.add(
            deMath.multiply(
                [0.0, 1.0, 0.0, 1.0], cScale
            ), cBias
        );

        var accessWithOffset = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_width,
                height: this.m_height,
                rowPitch: rowPitch,
                data: data,
                offset: this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize
                });
        tcuTextureUtil.fillWithGrid(accessWithOffset, 4, colorA, colorB
        );
        var access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_width,
                height: this.m_height,
                rowPitch: rowPitch,
                data: data,
                });

        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.texImage2D(
            gl.TEXTURE_2D, 0, this.m_internalFormat,
            this.m_width, this.m_height, 0,
            transferFmt.format, transferFmt.dataType, access.getDataPtr()
        );
    };

    // TexImage3D() unpack parameters case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} imageHeight
     * @param {number} rowLength
     * @param {number} skipImages
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexImage3DParamsCase = function(
        name, desc, internalFormat, width, height, depth, imageHeight,
        rowLength, skipImages, skipRows, skipPixels, alignment
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat),
            width, height, depth, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_imageHeight = imageHeight;
        this.m_rowLength = rowLength;
        this.m_skipImages = skipImages;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexImage3DParamsCase.prototype =
    Object.create(es3fTextureSpecificationTests.Texture3DSpecCase.prototype);

    es3fTextureSpecificationTests.TexImage3DParamsCase.prototype.constructor =
        es3fTextureSpecificationTests.Texture3DSpecCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.TexImage3DParamsCase.prototype.createTexture =
    function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_width;
        var rowPitch = deMath.deAlign32(rowLength * pixelSize, this.m_alignment);
        var imageHeight = this.m_imageHeight > 0 ?
            this.m_imageHeight : this.m_height;
        var slicePitch = imageHeight * rowPitch;

        var tex = null;
        /** @type {ArrayBuffer} */ var data;

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels different to 1',
            false, true
        );

        // Fill data with grid.
        data = new ArrayBuffer(slicePitch * (this.m_depth + this.m_skipImages) +
                               this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize);

        var cScale = deMath.subtract(
            this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin
        );
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply(
                [1.0, 0.0, 0.0, 1.0], cScale
            ), cBias
        );
        var colorB = deMath.add(
            deMath.multiply(
                [0.0, 1.0, 0.0, 1.0], cScale
            ), cBias
        );

        var accessWithOffset = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_width,
                height: this.m_height,
                depth: this.m_depth,
                rowPitch: rowPitch,
                slicePitch: slicePitch,
                data: data,
                offset: this.m_skipImages * slicePitch + this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize
            });
        var access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_width,
                height: this.m_height,
                depth: this.m_depth,
                rowPitch: rowPitch,
                slicePitch: slicePitch,
                data: data
            });
        tcuTextureUtil.fillWithGrid(
            accessWithOffset, 4, colorA, colorB
        );

        this.m_context.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, this.m_imageHeight);
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_IMAGES, this.m_skipImages);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);
        this.m_context.texImage3D(
            gl.TEXTURE_3D, 0, this.m_internalFormat,
            this.m_width, this.m_height, this.m_depth, 0,
            transferFmt.format, transferFmt.dataType, access.getDataPtr()
        );
    };

    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.BasicTexSubImage2DCase = function(
        name, desc, format, dataType, width, height
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height, es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_format = format;
        this.m_internalFormat = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.BasicTexSubImage2DCase.prototype =
    Object.create(es3fTextureSpecificationTests.Texture2DSpecCase.prototype);

    es3fTextureSpecificationTests.BasicTexSubImage2DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexSubImage2DCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @return {es3fTextureSpecificationTests.BasicTexSubImage2DCase}
     */
    es3fTextureSpecificationTests.newBasicTexSubImage2DCaseInternal =
    function(name, desc, internalFormat, width, height) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.BasicTexSubImage2DCase(
            name, desc, fmt.format, fmt.dataType, width, height
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     */
    es3fTextureSpecificationTests.
    BasicTexSubImage2DCase.prototype.createTexture = function() {
        var tex = null;
        var data = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            data.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                data.getAccess(), gMin, gMax
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, levelW, levelH, 0,
                this.m_format, this.m_dataType,
                data.getAccess().getDataPtr()
            );
        }

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            var w = rnd.getInt(1, levelW);
            var h = rnd.getInt(1, levelH);
            var x = rnd.getInt(0, levelW - w);
            var y = rnd.getInt(0, levelH - h);

            var colorA = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var colorB = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var cellSize = rnd.getInt(2, 16);

            data.setSize(w, h);
            tcuTextureUtil.fillWithGrid(
                data.getAccess(), cellSize, colorA, colorB
            );

            this.m_context.texSubImage2D(
                gl.TEXTURE_2D, ndx, x, y, w, h, this.m_format, this.m_dataType,
                data.getAccess().getDataPtr()
            );
        }
    };

    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     */
    es3fTextureSpecificationTests.BasicTexSubImageCubeCase = function(
        name, desc, format, dataType, size
    ) {
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, deMath.logToFloor(size) + 1
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.BasicTexSubImageCubeCase.prototype =
    Object.create(es3fTextureSpecificationTests.TextureCubeSpecCase.prototype);

    es3fTextureSpecificationTests.
    BasicTexSubImageCubeCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexSubImageCubeCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     */
    es3fTextureSpecificationTests.newBasicTexSubImageCubeCaseInternal =
    function(name, desc, internalFormat, size) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase =
            new es3fTextureSpecificationTests.BasicTexSubImageCubeCase(
            name, desc, fmt.format, fmt.dataType,
            size);
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexSubImageCubeCase.prototype.createTexture = function() {
        var tex = null;
        var data = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_size, this.m_size
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);
            data.setSize(levelSize, levelSize);

            for (var face = /** @type {tcuTexture.CubeFace} */ (0);
                face < es3fTextureSpecificationTests.s_cubeMapFaces.length;
                face++) {
                var gMin = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );
                var gMax = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );

                tcuTextureUtil.fillWithComponentGradients(
                    data.getAccess(), gMin, gMax
                );

                this.m_context.texImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face], ndx,
                    this.m_internalFormat, levelSize, levelSize, 0,
                    this.m_format, this.m_dataType,
                    data.getAccess().getDataPtr()
                );
            }
        }

        // Re-specify parts of each face and level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];

                var w = rnd.getInt(1, levelSize);
                var h = rnd.getInt(1, levelSize);
                var x = rnd.getInt(0, levelSize - w);
                var y = rnd.getInt(0, levelSize - h);

                var colorA = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );
                var colorB = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );
                var cellSize = rnd.getInt(2, 16);

                data.setSize(w, h);
                tcuTextureUtil.fillWithGrid(
                    data.getAccess(), cellSize, colorA, colorB
                );

                this.m_context.texSubImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, x, y, w, h, this.m_format,
                    this.m_dataType, data.getAccess().getDataPtr()
                );
            }
        }
    };

    // TexSubImage2D() unpack parameters case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexSubImage2DParamsCase = function(
        name, desc, internalFormat, width, height, subX, subY, subW, subH,
        rowLength, skipRows, skipPixels, alignment
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat),
            width, height, 1
        );
        this.m_internalFormat = internalFormat;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_rowLength = rowLength;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexSubImage2DParamsCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage2DParamsCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DParamsCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DParamsCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var tex = null;
        /** @type {ArrayBuffer} */ var data;

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels different to 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);

        // First fill texture with gradient.
        data = new ArrayBuffer(
            deMath.deAlign32(this.m_width * pixelSize, 4) * this.m_height
        );
        var access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_width,
                height: this.m_height,
                rowPitch: deMath.deAlign32(this.m_width * pixelSize, 4),
                data: data
            });
        tcuTextureUtil.fillWithComponentGradients(access, this.m_texFormatInfo.valueMin, this.m_texFormatInfo.valueMax
        );
        this.m_context.texImage2D(
            gl.TEXTURE_2D, 0, this.m_internalFormat, this.m_width,
            this.m_height, 0, transferFmt.format, transferFmt.dataType,
            access.getDataPtr()
        );

        // Fill data with grid.
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_subW;
        var rowPitch = deMath.deAlign32(rowLength * pixelSize, this.m_alignment);
        var height = this.m_subH + this.m_skipRows;
        var cScale = deMath.subtract(
            this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin
        );
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply(
                [1.0, 0.0, 0.0, 1.0], cScale
            ), cBias
        );
        var colorB = deMath.add(
            deMath.multiply(
                [0.0, 1.0, 0.0, 1.0], cScale
            ), cBias
        );

        data = new ArrayBuffer(rowPitch * height + this.m_skipPixels * pixelSize);
        tcuTextureUtil.fillWithGrid(
            new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_subW,
                height: this.m_subH,
                rowPitch: rowPitch,
                data: data,
                offset: this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize
                }) ,
            4, colorA, colorB
        );

        access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_subW,
                height: this.m_subH,
                rowPitch: rowPitch,
                data: data
            });
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);
        this.m_context.texSubImage2D(
            gl.TEXTURE_2D, 0, this.m_subX, this.m_subY,
            this.m_subW, this.m_subH,
            transferFmt.format, transferFmt.dataType,
            access.getDataPtr()
        );
    };

    // Basic TexSubImage3D() with 3D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     */
    es3fTextureSpecificationTests.BasicTexSubImage3DCase = function(
        name, desc, internalFormat, width, height, depth
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat),
            width, height, depth, es3fTextureSpecificationTests.maxLevelCount(
                width, height, depth
            )
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexSubImage3DCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture3DSpecCase.prototype
        );

    es3fTextureSpecificationTests.BasicTexSubImage3DCase.prototype.constructor =
    es3fTextureSpecificationTests.BasicTexSubImage3DCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexSubImage3DCase.prototype.createTexture = function() {
        var tex = null;
        var data = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // First specify full texture.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var levelD = Math.max(1, this.m_depth >> ndx);

            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            data.setSize(levelW, levelH, levelD);
            tcuTextureUtil.fillWithComponentGradients(
                data.getAccess(), gMin, gMax
            );

            this.m_context.texImage3D(
                gl.TEXTURE_3D, ndx, this.m_internalFormat, levelW, levelH,
                levelD, 0, transferFmt.format, transferFmt.dataType,
                data.getAccess().getDataPtr()
            );
        }

        // Re-specify parts of each level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var levelD = Math.max(1, this.m_depth >> ndx);

            var w = rnd.getInt(1, levelW);
            var h = rnd.getInt(1, levelH);
            var d = rnd.getInt(1, levelD);
            var x = rnd.getInt(0, levelW - w);
            var y = rnd.getInt(0, levelH - h);
            var z = rnd.getInt(0, levelD - d);

            var colorA = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var colorB = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var cellSize = rnd.getInt(2, 16);

            data.setSize(w, h, d);
            tcuTextureUtil.fillWithGrid(
                data.getAccess(), cellSize, colorA, colorB
            );

            this.m_context.texSubImage3D(
                gl.TEXTURE_3D, ndx, x, y, z, w, h, d,
                transferFmt.format, transferFmt.dataType,
                data.getAccess().getDataPtr()
            );
        }
    };

    // TexSubImage2D() to texture initialized with empty data
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.TexSubImage2DEmptyTexCase = function(
        name, desc, format, dataType, width, height
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height, es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_format = format;
        this.m_internalFormat = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.TexSubImage2DEmptyTexCase.prototype =
    Object.create(es3fTextureSpecificationTests.Texture2DSpecCase.prototype);

    es3fTextureSpecificationTests.
    TexSubImage2DEmptyTexCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DEmptyTexCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.newTexSubImage2DEmptyTexCaseInternal =
    function(name, desc, internalFormat, width, height) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase =
            new es3fTextureSpecificationTests.TexSubImage2DEmptyTexCase(
                name, desc, fmt.format, fmt.dataType, width, height
            );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DEmptyTexCase.prototype.createTexture = function() {
        var tex = null;
        var data = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // First allocate storage for each level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, levelW, levelH, 0,
                this.m_format, this.m_dataType,
                null
            );
        }

        // Specify pixel data to all levels using glTexSubImage2D()
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            data.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                data.getAccess(), gMin, gMax
            );

            this.m_context.texSubImage2D(
                gl.TEXTURE_2D, ndx, 0, 0, levelW, levelH,
                this.m_format, this.m_dataType,
                data.getAccess().getDataPtr()
            );
        }
    };

    // TexSubImage2D() to empty cubemap texture
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     */
    es3fTextureSpecificationTests.TexSubImageCubeEmptyTexCase = function(
        name, desc, format, dataType, size
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, deMath.logToFloor(size) + 1
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.TexSubImageCubeEmptyTexCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImageCubeEmptyTexCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImageCubeEmptyTexCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @return {es3fTextureSpecificationTests.TexSubImageCubeEmptyTexCase}
     */
    es3fTextureSpecificationTests.newTexSubImageCubeEmptyTexCaseInternal =
    function(name, desc, internalFormat, size) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase =
            new es3fTextureSpecificationTests.TexSubImageCubeEmptyTexCase(
                name, desc, fmt.format, fmt.dataType, size
            );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImageCubeEmptyTexCase.prototype.createTexture =
    function() {
        var tex = null;
        var data = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_size, this.m_size
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // Specify storage for each level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                this.m_context.texImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, this.m_internalFormat, levelSize, levelSize, 0,
                    this.m_format, this.m_dataType,
                    null
                );
            }
        }

        // Specify data using glTexSubImage2D()
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            data.setSize(levelSize, levelSize);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                var gMin = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );
                var gMax = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );

                tcuTextureUtil.fillWithComponentGradients(
                    data.getAccess(), gMin, gMax
                );

                this.m_context.texSubImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, 0, 0, levelSize, levelSize, this.m_format,
                    this.m_dataType, data.getAccess().getDataPtr()
                );
            }
        }
    };

    // TexSubImage2D() unpack alignment with 2D texture
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexSubImage2DAlignCase = function(
        name, desc, format, dataType, width, height, subX, subY, subW, subH,
        alignment
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height, 1
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexSubImage2DAlignCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage2DAlignCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DAlignCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} alignment
     * @return {es3fTextureSpecificationTests.TexSubImage2DAlignCase}
     */
    es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal = function(
        name, desc, internalFormat, width, height, subX, subY, subW, subH,
        alignment
    ) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase = new es3fTextureSpecificationTests.TexSubImage2DAlignCase(
            name, desc, fmt.format, fmt.dataType,
            width, height, subX, subY, subW, subH, alignment
        );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DAlignCase.prototype.createTexture = function() {
        var tex = null;
        /** @type {ArrayBuffer} */ var data;

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);

        // Specify base level.
        data = new ArrayBuffer(this.m_texFormat.getPixelSize() *
            this.m_width * this.m_height
        );
        var access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_width,
                height: this.m_height,
                data: data
            });
        tcuTextureUtil.fillWithComponentGradients(access, [0, 0, 0, 0], [1, 1, 1, 1]);

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
        this.m_context.texImage2D(gl.TEXTURE_2D, 0, this.m_internalFormat,
            this.m_width, this.m_height, 0, this.m_format, this.m_dataType,
            access.getDataPtr()
        );

        // Re-specify subrectangle.
        var rowPitch = deMath.deAlign32(
            this.m_texFormat.getPixelSize() * this.m_subW, this.m_alignment
        );
        data = new ArrayBuffer(rowPitch * this.m_subH);
        access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_subW,
                height: this.m_subH,
                rowPitch: rowPitch,
                data: data
            });
        tcuTextureUtil.fillWithGrid(access, 4, [1, 0, 0, 1], [0, 1, 0, 1]
        );

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);
        this.m_context.texSubImage2D(
            gl.TEXTURE_2D, 0, this.m_subX, this.m_subY, this.m_subW,
            this.m_subH, this.m_format, this.m_dataType, access.getDataPtr()
        );
    };

    // TexSubImage2D() unpack alignment with cubemap texture
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexSubImageCubeAlignCase = function(
        name, desc, format, dataType, size, subX, subY, subW, subH, alignment
    ) {
        // Unsized internal format.
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, 1
        );

        this.m_internalFormat = format;
        this.m_format = format;
        this.m_dataType = dataType;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexSubImageCubeAlignCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImageCubeAlignCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImageCubeAlignCase;

    /**
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} alignment
     * @return {es3fTextureSpecificationTests.TexSubImageCubeAlignCase}
     */
    es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal =
    function(
        name, desc, internalFormat, size,
        subX, subY, subW, subH, alignment
    ) {
        // Sized internal format.
        var fmt = gluTextureUtil.getTransferFormat(
            gluTextureUtil.mapGLInternalFormat(internalFormat)
        );
        var testcase =
            new es3fTextureSpecificationTests.TexSubImageCubeAlignCase(
                name, desc, fmt.format, fmt.dataType, size,
                subX, subY, subW, subH, alignment
            );
        testcase.m_internalFormat = internalFormat;
        return testcase;
    };

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImageCubeAlignCase.prototype.createTexture =
    function() {
        var tex = null;
        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);

        var data = new ArrayBuffer(this.m_texFormat.getPixelSize() * this.m_size * this.m_size);
        var access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_size,
                height: this.m_size,
                data: data
            });
        tcuTextureUtil.fillWithComponentGradients(access, [0, 0, 0, 0], [1, 1, 1, 1]
        );

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
        for (var f in tcuTexture.CubeFace) {
            var face = tcuTexture.CubeFace[f];
            this.m_context.texImage2D(
                es3fTextureSpecificationTests.s_cubeMapFaces[face],
                0, this.m_internalFormat, this.m_size, this.m_size, 0,
                this.m_format, this.m_dataType,
                access.getDataPtr()
            );
        }

        // Re-specify subrectangle.
        var rowPitch = deMath.deAlign32(
            this.m_texFormat.getPixelSize() * this.m_subW, this.m_alignment
        );
        data = new ArrayBuffer(rowPitch * this.m_subH);
        access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_subW,
                height: this.m_subH,
                rowPitch: rowPitch,
                data: data
        });
        tcuTextureUtil.fillWithGrid(access, 4, [1, 0, 0, 1], [0, 1, 0, 1]
        );
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);
        for (var f in tcuTexture.CubeFace) {
            var face = tcuTexture.CubeFace[f];
            this.m_context.texSubImage2D(
                es3fTextureSpecificationTests.s_cubeMapFaces[face],
                0, this.m_subX, this.m_subY, this.m_subW, this.m_subH,
                this.m_format, this.m_dataType, access.getDataPtr()
            );
        }
    };

    // TexSubImage3D() unpack parameters case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} subX ,
     * @param {number} subY ,
     * @param {number} subZ ,
     * @param {number} subW ,
     * @param {number} subH ,
     * @param {number} subD ,
     * @param {number} imageHeight ,
     * @param {number} rowLength ,
     * @param {number} skipImages ,
     * @param {number} skipRows ,
     * @param {number} skipPixels ,
     * @param {number} alignment
     */
    es3fTextureSpecificationTests.TexSubImage3DParamsCase = function(
        name, desc, internalFormat, width, height, depth,
        subX, subY, subZ, subW, subH, subD,
        imageHeight, rowLength, skipImages, skipRows, skipPixels, alignment
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc,
            gluTextureUtil.mapGLInternalFormat(internalFormat),
            width, height, depth, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subZ = subZ;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_subD = subD;
        this.m_imageHeight = imageHeight;
        this.m_rowLength = rowLength;
        this.m_skipImages = skipImages;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
    };

    es3fTextureSpecificationTests.TexSubImage3DParamsCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture3DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage3DParamsCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage3DParamsCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage3DParamsCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var tex = null;
        var rowPitch = deMath.deAlign32(pixelSize * this.m_width, 4);
        var slicePitch = rowPitch * this.m_height;
        assertMsgOptions(
            this.m_numLevels == 1, 'Numbel of levels different than 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);

        // Fill with gradient.

        var data = new ArrayBuffer(slicePitch * this.m_depth);
        var access = new tcuTexture.PixelBufferAccess({
                            format: this.m_texFormat,
                            width: this.m_width,
                            height: this.m_height,
                            depth: this.m_depth,
                            rowPitch: rowPitch,
                            slicePitch: slicePitch,
                            data: data
                        });
        tcuTextureUtil.fillWithComponentGradients(access, this.m_texFormatInfo.valueMin, this.m_texFormatInfo.valueMax);

        this.m_context.texImage3D(
            gl.TEXTURE_3D, 0, this.m_internalFormat, this.m_width,
            this.m_height, this.m_depth, 0, transferFmt.format,
            transferFmt.dataType, access.getDataPtr()
        );

        // Fill data with grid.
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_subW;
        rowPitch = deMath.deAlign32(rowLength * pixelSize, this.m_alignment);
        var imageHeight = this.m_imageHeight > 0 ? this.m_imageHeight : this.m_subH;
        slicePitch = imageHeight * rowPitch;
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        data = new ArrayBuffer(slicePitch * (this.m_depth + this.m_skipImages) +
                               this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize);
        var accessWithOffset = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_subW,
                height: this.m_subH,
                depth: this.m_subD,
                rowPitch: rowPitch,
                slicePitch: slicePitch,
                data: data,
                offset: this.m_skipImages * slicePitch + this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize
            });
        access = new tcuTexture.PixelBufferAccess({
                format: this.m_texFormat,
                width: this.m_subW,
                height: this.m_subH,
                depth: this.m_subD,
                rowPitch: rowPitch,
                slicePitch: slicePitch,
                data: data
            });
        tcuTextureUtil.fillWithGrid(accessWithOffset, 4, colorA, colorB);

        this.m_context.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, this.m_imageHeight);
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_IMAGES, this.m_skipImages);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);
        this.m_context.texSubImage3D(
            gl.TEXTURE_3D, 0, this.m_subX, this.m_subY, this.m_subZ,
            this.m_subW, this.m_subH, this.m_subD,
            transferFmt.format, transferFmt.dataType, access.getDataPtr()
        );
    };

    // Basic CopyTexImage2D() with 2D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.BasicCopyTexImage2DCase = function(
        name, desc, internalFormat, width, height
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                internalFormat, gl.UNSIGNED_BYTE
            ), width, height, es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicCopyTexImage2DCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicCopyTexImage2DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicCopyTexImage2DCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicCopyTexImage2DCase.prototype.createTexture = function() {
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
        var targetHasRGB = pixelFormat.redBits > 0 &&
            pixelFormat.greenBits > 0 &&
            pixelFormat.blueBits > 0;
        var targetHasAlpha = pixelFormat.alphaBits > 0;
        var fmt = es3fTextureSpecificationTests.mapGLUnsizedInternalFormat(
                this.m_internalFormat
        );
        var texHasRGB = fmt.order != tcuTexture.ChannelOrder.A;
        var texHasAlpha = fmt.order == tcuTexture.ChannelOrder.RGBA ||
            fmt.order == tcuTexture.ChannelOrder.LA ||
            fmt.order == tcuTexture.ChannelOrder.A;
        var tex = null;
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        /** @type {es3fFboTestUtil.GradientShader} */
        var shader = new es3fFboTestUtil.GradientShader(
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderID = this.m_context.createProgram(shader);

        if ((texHasRGB && !targetHasRGB) || (texHasAlpha && !targetHasAlpha))
            throw new Error(
                'Copying from current framebuffer is not supported'
            );

        // Fill render target with gradient.
        shader.setGradient(
            this.m_context, shaderID, [0, 0, 0, 0], [1, 1, 1, 1]
        );
        rrUtil.drawQuad(
            this.m_context, shaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var x = rnd.getInt(0, this.m_width - levelW);
            var y = rnd.getInt(0, this.m_height - levelH);

            this.m_context.copyTexImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, x, y,
                levelW, levelH, 0
            );
        }
    };

    // Basic CopyTexImage2D() with cubemap usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     */
    es3fTextureSpecificationTests.BasicCopyTexImageCubeCase = function(
        name, desc, internalFormat, size
    ) {
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                internalFormat, gl.UNSIGNED_BYTE
            ), size, deMath.logToFloor(size) + 1
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicCopyTexImageCubeCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicCopyTexImageCubeCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicCopyTexImageCubeCase;

    es3fTextureSpecificationTests.
    BasicCopyTexImageCubeCase.prototype.createTexture = function() {
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
        var targetHasRGB = pixelFormat.redBits > 0 &&
            pixelFormat.greenBits > 0 &&
            pixelFormat.blueBits > 0;
        var targetHasAlpha = pixelFormat.alphaBits > 0;
        var fmt = es3fTextureSpecificationTests.mapGLUnsizedInternalFormat(
                this.m_internalFormat
        );
        var texHasRGB = fmt.order != tcuTexture.ChannelOrder.A;
        var texHasAlpha = fmt.order == tcuTexture.ChannelOrder.RGBA ||
            fmt.order == tcuTexture.ChannelOrder.LA ||
            fmt.order == tcuTexture.ChannelOrder.A;
        var tex = null;
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        /** @type {es3fFboTestUtil.GradientShader} */
        var shader = new es3fFboTestUtil.GradientShader(
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderID = this.m_context.createProgram(shader);

        if ((texHasRGB && !targetHasRGB) || (texHasAlpha && !targetHasAlpha))
            throw new Error(
                'Copying from current framebuffer is not supported'
            );

        // Fill render target with gradient.
        shader.setGradient(
            this.m_context, shaderID, [0, 0, 0, 0], [1, 1, 1, 1]
        );
        rrUtil.drawQuad(
            this.m_context, shaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                var x = rnd.getInt(0, this.m_size - levelSize);
                var y = rnd.getInt(0, this.m_size - levelSize);

                this.m_context.copyTexImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face], ndx,
                    this.m_internalFormat, x, y, levelSize, levelSize, 0
                );
            }
        }
    };

    // Basic CopyTexSubImage2D() with 2D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} width
     * @param {number} height
     */
    es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase = function(
        name, desc, format, dataType, width, height
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), width, height, es3fTextureSpecificationTests.maxLevelCount(
                width, height
            )
        );

        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicCopyTexSubImage2DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicCopyTexSubImage2DCase.prototype.createTexture = function() {
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
        var targetHasRGB = pixelFormat.redBits > 0 &&
            pixelFormat.greenBits > 0 &&
            pixelFormat.blueBits > 0;
        var targetHasAlpha = pixelFormat.alphaBits > 0;
        var fmt = gluTextureUtil.mapGLTransferFormat(
                this.m_format, this.m_dataType
        );
        var texHasRGB = fmt.order != tcuTexture.ChannelOrder.A;
        var texHasAlpha = fmt.order == tcuTexture.ChannelOrder.RGBA ||
            fmt.order == tcuTexture.ChannelOrder.LA ||
            fmt.order == tcuTexture.ChannelOrder.A;
        var tex = null;
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        /** @type {es3fFboTestUtil.GradientShader} */
        var shader = new es3fFboTestUtil.GradientShader(
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderID = this.m_context.createProgram(shader);

        if ((texHasRGB && !targetHasRGB) || (texHasAlpha && !targetHasAlpha))
            throw new Error(
                'Copying from current framebuffer is not supported'
            );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // First specify full texture.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            var colorA = es3fTextureSpecificationTests.randomVector(
                rnd, [0, 0, 0, 0], [1, 1, 1, 1], 4
            );
            var colorB = es3fTextureSpecificationTests.randomVector(
                rnd, [0, 0, 0, 0], [1, 1, 1, 1], 4
            );
            var cellSize = rnd.getInt(2, 16);

            var data = new tcuTexture.TextureLevel(fmt, levelW, levelH);
            tcuTextureUtil.fillWithGrid(
                data.getAccess(), cellSize, colorA, colorB
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_format, levelW, levelH, 0, this.m_format, this.m_dataType, data.getAccess().getDataPtr()
            );
        }

        // Fill render target with gradient.
        shader.setGradient(
            this.m_context, shaderID, [0, 0, 0, 0], [1, 1, 1, 1]
        );
        rrUtil.drawQuad(
            this.m_context, shaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        // Re-specify parts of each level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            var w = rnd.getInt(1, levelW);
            var h = rnd.getInt(1, levelH);
            var xo = rnd.getInt(0, levelW - w);
            var yo = rnd.getInt(0, levelH - h);

            var x = rnd.getInt(0, this.m_width - w);
            var y = rnd.getInt(0, this.m_height - h);

            this.m_context.copyTexSubImage2D(
                gl.TEXTURE_2D, ndx, xo, yo, x, y, w, h
            );
        }
    };

    // Basic CopyTexSubImage2D() with cubemap usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} format
     * @param {number} dataType
     * @param {number} size
     */
    es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase = function(
        name, desc, format, dataType, size
    ) {
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLTransferFormat(
                format, dataType
            ), size, deMath.logToFloor(size) + 1
        );

        this.m_format = format;
        this.m_dataType = dataType;
    };

    es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicCopyTexSubImageCubeCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase;

    es3fTextureSpecificationTests.
    BasicCopyTexSubImageCubeCase.prototype.createTexture = function() {
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
        var targetHasRGB = pixelFormat.redBits > 0 &&
            pixelFormat.greenBits > 0 &&
            pixelFormat.blueBits > 0;
        var targetHasAlpha = pixelFormat.alphaBits > 0;
        var fmt = gluTextureUtil.mapGLTransferFormat(this.m_format, this.m_dataType);
        var texHasRGB = fmt.order != tcuTexture.ChannelOrder.A;
        var texHasAlpha = fmt.order == tcuTexture.ChannelOrder.RGBA ||
            fmt.order == tcuTexture.ChannelOrder.LA ||
            fmt.order == tcuTexture.ChannelOrder.A;
        var tex = null;
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        /** @type {es3fFboTestUtil.GradientShader} */
        var shader = new es3fFboTestUtil.GradientShader(
            gluShaderUtil.DataType.FLOAT_VEC4
        );
        var shaderID = this.m_context.createProgram(shader);

        if ((texHasRGB && !targetHasRGB) || (texHasAlpha && !targetHasAlpha))
            throw new Error(
                'Copying from current framebuffer is not supported'
            );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        var data = new tcuTexture.TextureLevel(fmt);

        // First specify full texture.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            data.setSize(levelSize, levelSize);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                var colorA = es3fTextureSpecificationTests.randomVector(
                    rnd, [0, 0, 0, 0], [1, 1, 1, 1], 4
                );
                var colorB = es3fTextureSpecificationTests.randomVector(
                    rnd, [0, 0, 0, 0], [1, 1, 1, 1], 4
                );
                var cellSize = rnd.getInt(2, 16);

                tcuTextureUtil.fillWithGrid(
                    data.getAccess(), cellSize, colorA, colorB
                );

                this.m_context.texImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, this.m_format, levelSize, levelSize, 0, this.m_format,
                    this.m_dataType, data.getAccess().getDataPtr()
                );
            }
        }

        // Fill render target with gradient.
        shader.setGradient(
            this.m_context, shaderID, [0, 0, 0, 0], [1, 1, 1, 1]
        );
        rrUtil.drawQuad(
            this.m_context, shaderID, [-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]
        );

        // Re-specify parts of each face and level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            for (var f in es3fTextureSpecificationTests.s_cubeMapFaces) {
                var w = rnd.getInt(1, levelSize);
                var h = rnd.getInt(1, levelSize);
                var xo = rnd.getInt(0, levelSize - w);
                var yo = rnd.getInt(0, levelSize - h);

                var x = rnd.getInt(0, this.m_size - w);
                var y = rnd.getInt(0, this.m_size - h);

                this.m_context.copyTexSubImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[f],
                    ndx, xo, yo, x, y, w, h
                );
            }
        }
    };

    // Basic glTexStorage2D() with 2D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.BasicTexStorage2DCase = function(
        name, desc, internalFormat, width, height, numLevels
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, numLevels
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexStorage2DCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.BasicTexStorage2DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexStorage2DCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexStorage2DCase.prototype.createTexture = function() {
        var fmt = gluTextureUtil.mapGLInternalFormat(this.m_internalFormat);
        var transferFmt = gluTextureUtil.getTransferFormat(fmt);
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            fmt, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.texStorage2D(
            gl.TEXTURE_2D, this.m_numLevels, this.m_internalFormat,
            this.m_width, this.m_height
        );

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texSubImage2D(
                gl.TEXTURE_2D, ndx, 0, 0, levelW, levelH,
                transferFmt.format, transferFmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // Basic glTexStorage2D() with cubemap usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.BasicTexStorageCubeCase = function(
        name, desc, internalFormat, size, numLevels
    ) {
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), size, numLevels
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexStorageCubeCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.BasicTexStorageCubeCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexStorageCubeCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.BasicTexStorageCubeCase.prototype.createTexture = function() {
        var fmt = gluTextureUtil.mapGLInternalFormat(this.m_internalFormat);
        var transferFmt = gluTextureUtil.getTransferFormat(fmt);
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            fmt, this.m_size, this.m_size
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.texStorage2D(
            gl.TEXTURE_CUBE_MAP, this.m_numLevels, this.m_internalFormat,
            this.m_size, this.m_size
        );

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelSize = Math.max(1, this.m_size >> ndx);

            levelData.setSize(levelSize, levelSize);

            for (var f in tcuTexture.CubeFace) {
                var face = tcuTexture.CubeFace[f];
                var gMin = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );
                var gMax = es3fTextureSpecificationTests.randomVector(
                    rnd, this.m_texFormatInfo.valueMin,
                    this.m_texFormatInfo.valueMax, 4
                );

                tcuTextureUtil.fillWithComponentGradients(
                    levelData.getAccess(), gMin, gMax
                );

                this.m_context.texSubImage2D(
                    es3fTextureSpecificationTests.s_cubeMapFaces[face],
                    ndx, 0, 0, levelSize, levelSize,
                    transferFmt.format, transferFmt.dataType,
                    levelData.getAccess().getDataPtr()
                );
            }
        }
    };

    // Basic glTexStorage3D() with 2D array texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} numLayers
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.BasicTexStorage2DArrayCase = function(
        name, desc, internalFormat, width, height, numLayers, numLevels
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, numLayers, numLevels
        );
        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexStorage2DArrayCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicTexStorage2DArrayCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexStorage2DArrayCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.BasicTexStorage2DArrayCase.prototype.createTexture = function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        this.m_context.texStorage3D(
            gl.TEXTURE_2D_ARRAY, this.m_numLevels, this.m_internalFormat,
            this.m_width, this.m_height, this.m_numLayers
        );

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH, this.m_numLayers);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texSubImage3D(
                gl.TEXTURE_2D_ARRAY, ndx, 0, 0, 0, levelW, levelH,
                this.m_numLayers, transferFmt.format, transferFmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // Basic TexStorage3D() with 3D texture usage
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} numLevels
     */
    es3fTextureSpecificationTests.BasicTexStorage3DCase = function(
        name, desc, internalFormat, width, height, depth, numLevels
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, depth, numLevels
        );

        this.m_internalFormat = internalFormat;
    };

    es3fTextureSpecificationTests.BasicTexStorage3DCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture3DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    BasicTexStorage3DCase.prototype.constructor =
        es3fTextureSpecificationTests.BasicTexStorage3DCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    BasicTexStorage3DCase.prototype.createTexture = function() {
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);
        this.m_context.texStorage3D(
            gl.TEXTURE_3D, this.m_numLevels, this.m_internalFormat,
            this.m_width, this.m_height, this.m_depth
        );

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var levelD = Math.max(1, this.m_depth >> ndx);
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            levelData.setSize(levelW, levelH, levelD);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texSubImage3D(
                gl.TEXTURE_3D, ndx, 0, 0, 0, levelW, levelH,
                levelD, transferFmt.format, transferFmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // Pixel buffer object cases.

    // TexImage2D() from pixel buffer object.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexImage2DBufferCase = function(
        name, desc, internalFormat, width, height, rowLength,
        skipRows, skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_rowLength = rowLength;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexImage2DBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage2DBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.TexImage2DBufferCase.prototype.createTexture =
    function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_rowLength > 0 ?
            this.m_rowLength :
            this.m_width + this.m_skipPixels;
        var rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var height = this.m_height + this.m_skipRows;
        var buf = null;
        var tex = null;
        var data = new ArrayBuffer(rowPitch * height + this.m_skipPixels * pixelSize + this.m_offset);

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels different than 1',
            false, true
        );

        // Fill data with grid.
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        var access = new tcuTexture.PixelBufferAccess({
                    format: this.m_texFormat,
                    width: this.m_width,
                    height: this.m_height,
                    rowPitch: rowPitch,
                    data: data,
                    offset: this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize + this.m_offset
                    });
        tcuTextureUtil.fillWithGrid(access, 4, colorA, colorB);

        // Create buffer and upload.
        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(gl.PIXEL_UNPACK_BUFFER, data, gl.STATIC_DRAW
        );

        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.texImage2D(gl.TEXTURE_2D, 0, this.m_internalFormat,
            this.m_width, this.m_height, 0, transferFmt.format, transferFmt.dataType,
            this.m_offset
        );
    };

    // TexImage2D() cubemap from pixel buffer object case
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexImageCubeBufferCase = function(
        name, desc, internalFormat, size, rowLength, skipRows, skipPixels,
        alignment, offset
    ) {
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), size, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_rowLength = rowLength;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexImageCubeBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImageCubeBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImageCubeBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImageCubeBufferCase.prototype.createTexture = function() {
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var tex = null;
        var fmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_rowLength > 0 ?
            this.m_rowLength : this.m_size + this.m_skipPixels;
        var rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var height = this.m_size + this.m_skipRows;

        var data = new ArrayBuffer(rowPitch * height + this.m_skipPixels * pixelSize + this.m_offset);
        var access = new tcuTexture.PixelBufferAccess({
                            format: this.m_texFormat,
                            width: this.m_size,
                            height: this.m_size,
                            rowPitch: rowPitch,
                            data: data,
                            offset: this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize + this.m_offset
                        });

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var f in tcuTexture.CubeFace) {
            var face = tcuTexture.CubeFace[f];
            var buf = null;

            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            tcuTextureUtil.fillWithComponentGradients(access, gMin, gMax);

            // Create buffer and upload.
            buf = this.m_context.createBuffer();
            this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
            this.m_context.bufferData(gl.PIXEL_UNPACK_BUFFER, data, gl.STATIC_DRAW);

            this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
            this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
            this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
            this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

            this.m_context.texImage2D(
                es3fTextureSpecificationTests.s_cubeMapFaces[face], 0,
                this.m_internalFormat, this.m_size, this.m_size, 0, fmt.format,
                fmt.dataType, this.m_offset);
        }
    };

    // TexImage3D() 2D array from pixel buffer object.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} imageHeight
     * @param {number} rowLength
     * @param {number} skipImages
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexImage2DArrayBufferCase = function(
        name, desc, internalFormat, width, height, depth, imageHeight,
        rowLength, skipImages, skipRows, skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, depth, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_imageHeight = imageHeight;
        this.m_rowLength = rowLength;
        this.m_skipImages = skipImages;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexImage2DArrayBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage2DArrayBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DArrayBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImage2DArrayBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_width;
        var rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var imageHeight = this.m_imageHeight > 0 ?
            this.m_imageHeight : this.m_height;
        var slicePitch = imageHeight * rowPitch;
        var tex = null;
        var buf = null;
        var data = new ArrayBuffer(
            slicePitch * (this.m_numLayers + this.m_skipImages) +
            this.m_skipRows * rowPitch + this.m_skipPixels * pixelSize + this.m_offset
        );

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        // Fill data with grid.
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        depth: this.m_numLayers,
                        rowPitch: rowPitch,
                        slicePitch: slicePitch,
                        data: data,
                        offset: this.m_skipImages * slicePitch +
                            this.m_skipRows * rowPitch +
                            this.m_skipPixels * pixelSize +
                            this.m_offset
                        });
        tcuTextureUtil.fillWithGrid(access, 4, colorA, colorB);

        // Create buffer and upload.
        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(gl.PIXEL_UNPACK_BUFFER, data, gl.STATIC_DRAW);

        this.m_context.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, this.m_imageHeight);
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_IMAGES, this.m_skipImages);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        this.m_context.texImage3D(
            gl.TEXTURE_2D_ARRAY, 0, this.m_internalFormat, this.m_width,
            this.m_height, this.m_numLayers, 0, transferFmt.format,
            transferFmt.dataType, this.m_offset
        );
    };

    // TexImage3D() from pixel buffer object.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} imageHeight
     * @param {number} rowLength
     * @param {number} skipImages
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexImage3DBufferCase = function(
        name, desc, internalFormat, width, height, depth, imageHeight,
        rowLength, skipImages, skipRows, skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, depth, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_imageHeight = imageHeight;
        this.m_rowLength = rowLength;
        this.m_skipImages = skipImages;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexImage3DBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture3DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage3DBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage3DBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImage3DBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_width;
        var rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var imageHeight = this.m_imageHeight > 0 ?
            this.m_imageHeight : this.m_height;
        var slicePitch = imageHeight * rowPitch;
        var tex = null;
        var buf = null;
        var data = new ArrayBuffer(
            slicePitch * (this.m_depth + this.m_skipImages) +
            rowPitch * this.m_skipRows + pixelSize * this.m_skipPixels + this.m_offset
        );

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        // Fill data with grid.
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        depth: this.m_depth,
                        rowPitch: rowPitch,
                        slicePitch: slicePitch,
                        data: data,
                        offset: this.m_skipImages * slicePitch +
                            this.m_skipRows * rowPitch +
                            this.m_skipPixels * pixelSize +
                            this.m_offset
                        });
        tcuTextureUtil.fillWithGrid(access, 4, colorA, colorB);

        // Create buffer and upload.
        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(gl.PIXEL_UNPACK_BUFFER, data, gl.STATIC_DRAW);

        this.m_context.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, this.m_imageHeight);
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_IMAGES, this.m_skipImages);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);
        this.m_context.texImage3D(
            gl.TEXTURE_3D, 0, this.m_internalFormat, this.m_width,
            this.m_height, this.m_depth, 0, transferFmt.format,
            transferFmt.dataType, this.m_offset
        );
    };

    // TexSubImage2D() PBO case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexSubImage2DBufferCase = function(
        name, desc, internalFormat, width, height, subX, subY, subW, subH,
        rowLength, skipRows, skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, 1
        );
        this.m_internalFormat = internalFormat;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_rowLength = rowLength;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexSubImage2DBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage2DBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );
        var pixelSize = this.m_texFormat.getPixelSize();
        var tex = null;
        var buf = null;
        var data = new ArrayBuffer(
            deMath.deAlign32(this.m_width * pixelSize, 4) * this.m_height
        );

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);

        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        rowPitch: deMath.deAlign32(this.m_width * pixelSize, 4),
                        data: data
                        });
        // First fill texture with gradient.
        tcuTextureUtil.fillWithComponentGradients(access, this.m_texFormatInfo.valueMin, this.m_texFormatInfo.valueMax);
        this.m_context.texImage2D(
            gl.TEXTURE_2D, 0, this.m_internalFormat,
            this.m_width, this.m_height, 0, transferFmt.format,
            transferFmt.dataType, access.getDataPtr()
        );

        // Fill data with grid.
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_subW;
        var rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var height = this.m_subH + this.m_skipRows;
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_subW,
                        height: this.m_subH,
                        rowPitch: rowPitch,
                        data: new ArrayBuffer(rowPitch * height + this.m_offset),
                        offset: this.m_skipRows * rowPitch +
                                this.m_skipPixels * pixelSize +
                                this.m_offset
                        });
        tcuTextureUtil.fillWithGrid(access, 4, colorA, colorB);

        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(gl.PIXEL_UNPACK_BUFFER, access.getBuffer(), gl.STATIC_DRAW);

        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        this.m_context.texSubImage2D(
            gl.TEXTURE_2D, 0, this.m_subX, this.m_subY,
            this.m_subW, this.m_subH, transferFmt.format,
            transferFmt.dataType, this.m_offset
        );
    };

    // TexSubImage2D() cubemap PBO case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.TextureCubeSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} size
     * @param {number} subX
     * @param {number} subY
     * @param {number} subW
     * @param {number} subH
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexSubImageCubeBufferCase = function(
        name, desc, internalFormat, size, subX, subY, subW, subH, rowLength,
        skipRows, skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.TextureCubeSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), size, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_rowLength = rowLength;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexSubImageCubeBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.TextureCubeSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImageCubeBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImageCubeBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImageCubeBufferCase.prototype.createTexture = function() {
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );
        var pixelSize = this.m_texFormat.getPixelSize();
        var tex = null;
        var buf = null;
        var data = new ArrayBuffer(
            deMath.deAlign32(this.m_size * pixelSize, 4) * this.m_size
        );
        var access = new tcuTexture.PixelBufferAccess({
                            format: this.m_texFormat,
                            width: this.m_size,
                            height: this.m_size,
                            rowPitch: deMath.deAlign32(this.m_size * pixelSize, 4),
                            data: data
                        });
        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_CUBE_MAP, tex);

        // Fill faces with different gradients.

        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var f in tcuTexture.CubeFace) {
            var face = tcuTexture.CubeFace[f];
            var gMin = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );
            var gMax = es3fTextureSpecificationTests.randomVector(
                rnd, this.m_texFormatInfo.valueMin,
                this.m_texFormatInfo.valueMax, 4
            );

            tcuTextureUtil.fillWithComponentGradients(access, gMin, gMax);

            this.m_context.texImage2D(
                es3fTextureSpecificationTests.s_cubeMapFaces[face], 0,
                this.m_internalFormat, this.m_size, this.m_size, 0,
                transferFmt.format, transferFmt.dataType,
                access.getDataPtr()
            );
        }

        // Fill data with grid.
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_subW;
        var rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var height = this.m_subH + this.m_skipRows;
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        data = new ArrayBuffer(rowPitch * height + this.m_skipPixels * pixelSize + this.m_offset);
        var accessSub = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_subW,
                        height: this.m_subH,
                        rowPitch: rowPitch,
                        data: data,
                        offset: this.m_skipRows * rowPitch +
                            this.m_skipPixels * pixelSize +
                            this.m_offset
                        });
        tcuTextureUtil.fillWithGrid(accessSub, 4, colorA, colorB);

        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(
            gl.PIXEL_UNPACK_BUFFER, data, gl.STATIC_DRAW
        );

        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);

        for (var f in tcuTexture.CubeFace) {
            var face = tcuTexture.CubeFace[f];
            this.m_context.texSubImage2D(
                es3fTextureSpecificationTests.s_cubeMapFaces[face], 0,
                this.m_subX, this.m_subY, this.m_subW, this.m_subH,
                transferFmt.format, transferFmt.dataType, this.m_offset
            );
        }
    };

    // TexSubImage3D() 2D array PBO case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} subX
     * @param {number} subY
     * @param {number} subZ
     * @param {number} subW
     * @param {number} subH
     * @param {number} subD
     * @param {number} imageHeight
     * @param {number} rowLength
     * @param {number} skipImages
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexSubImage2DArrayBufferCase = function(
        name, desc, internalFormat, width, height, depth, subX, subY, subZ,
        subW, subH, subD, imageHeight, rowLength, skipImages, skipRows,
        skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, depth, 1
        );
        this.m_internalFormat = internalFormat;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subZ = subZ;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_subD = subD;
        this.m_imageHeight = imageHeight;
        this.m_rowLength = rowLength;
        this.m_skipImages = skipImages;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexSubImage2DArrayBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage2DArrayBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DArrayBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DArrayBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );
        var pixelSize = this.m_texFormat.getPixelSize();
        var tex = null;
        var buf = null;
        /** @type {ArrayBuffer} */ var data;

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);

        // Fill with gradient.
        var rowPitch = deMath.deAlign32(pixelSize * this.m_width, 4);
        var slicePitch = rowPitch * this.m_height;

        data = new ArrayBuffer(slicePitch * this.m_numLayers);
        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        depth: this.m_numLayers,
                        rowPitch: rowPitch,
                        slicePitch: slicePitch,
                        data: data
                    });
        tcuTextureUtil.fillWithComponentGradients(access, this.m_texFormatInfo.valueMin, this.m_texFormatInfo.valueMax
        );

        this.m_context.texImage3D(gl.TEXTURE_2D_ARRAY, 0, this.m_internalFormat, this.m_width, this.m_height,
            this.m_numLayers, 0, transferFmt.format, transferFmt.dataType, access.getDataPtr());

        // Fill data with grid.
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_subW;
        rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var height = this.m_subH + this.m_skipRows;
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);

        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        var accessSub = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_subW,
                        height: this.m_subH,
                        rowPitch: rowPitch,
                        data: new ArrayBuffer(rowPitch * height + this.m_offset),
                        offset: this.m_skipRows * rowPitch +
                                this.m_skipPixels * pixelSize +
                                this.m_offset
                    });
        tcuTextureUtil.fillWithGrid(accessSub, 4, colorA, colorB);

        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(
            gl.PIXEL_UNPACK_BUFFER, accessSub.getBuffer(), gl.STATIC_DRAW
        );

        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);
    };

    // TexSubImage3D() PBO case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture3DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} subX
     * @param {number} subY
     * @param {number} subZ
     * @param {number} subW
     * @param {number} subH
     * @param {number} subD
     * @param {number} imageHeight
     * @param {number} rowLength
     * @param {number} skipImages
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number} alignment
     * @param {number} offset
     */
    es3fTextureSpecificationTests.TexSubImage3DBufferCase = function(
        name, desc, internalFormat, width, height, depth, subX, subY, subZ,
        subW, subH, subD, imageHeight, rowLength, skipImages, skipRows,
        skipPixels, alignment, offset
    ) {
        es3fTextureSpecificationTests.Texture3DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), width, height, depth, 1
        );

        this.m_internalFormat = internalFormat;
        this.m_subX = subX;
        this.m_subY = subY;
        this.m_subZ = subZ;
        this.m_subW = subW;
        this.m_subH = subH;
        this.m_subD = subD;
        this.m_imageHeight = imageHeight;
        this.m_rowLength = rowLength;
        this.m_skipImages = skipImages;
        this.m_skipRows = skipRows;
        this.m_skipPixels = skipPixels;
        this.m_alignment = alignment;
        this.m_offset = offset;
    };

    es3fTextureSpecificationTests.TexSubImage3DBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture3DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage3DBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage3DBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage3DBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(
            this.m_texFormat
        );
        var pixelSize = this.m_texFormat.getPixelSize();
        var tex = null;
        var buf = null;
        /** @type {ArrayBuffer} */ var data;

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_3D, tex);

        // Fill with gradient.
        var rowPitch = deMath.deAlign32(pixelSize * this.m_width, 4);
        var slicePitch = rowPitch * this.m_height;

        data = new ArrayBuffer(slicePitch * this.m_depth);
        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        depth: this.m_depth,
                        rowPitch: rowPitch,
                        slicePitch: slicePitch,
                        data: data});
        tcuTextureUtil.fillWithComponentGradients(access, this.m_texFormatInfo.valueMin, this.m_texFormatInfo.valueMax
        );

        this.m_context.texImage3D(
            gl.TEXTURE_3D, 0, this.m_internalFormat, this.m_width,
            this.m_height, this.m_depth, 0, transferFmt.format,
            transferFmt.dataType, access.getDataPtr()
        );

        // Fill data with grid.
        var rowLength = this.m_rowLength > 0 ? this.m_rowLength : this.m_subW;
        rowPitch = deMath.deAlign32(
            rowLength * pixelSize, this.m_alignment
        );
        var imageHeight = this.m_imageHeight > 0 ?
            this.m_imageHeight : this.m_subH;
        slicePitch = imageHeight * rowPitch;
        var cScale = deMath.subtract(this.m_texFormatInfo.valueMax, this.m_texFormatInfo.valueMin);
        var cBias = this.m_texFormatInfo.valueMin;
        var colorA = deMath.add(
            deMath.multiply([1.0, 0.0, 0.0, 1.0], cScale), cBias
        );
        var colorB = deMath.add(
            deMath.multiply([0.0, 1.0, 0.0, 1.0], cScale), cBias
        );

        data = new ArrayBuffer(slicePitch * (this.m_subD + this.m_skipImages) +
                               rowPitch * this.m_skipRows + pixelSize * this.m_skipPixels + this.m_offset);
        var accessSub = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_subW,
                        height: this.m_subH,
                        depth: this.m_subD,
                        rowPitch: rowPitch,
                        slicePitch: slicePitch,
                        data: data,
                        offset: this.m_skipImages * slicePitch +
                                this.m_skipRows * rowPitch +
                                this.m_skipPixels * pixelSize +
                                this.m_offset
                    });
        tcuTextureUtil.fillWithGrid(accessSub, 4, colorA, colorB
        );

        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(
            gl.PIXEL_UNPACK_BUFFER, data, gl.STATIC_DRAW
        );

        this.m_context.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, this.m_imageHeight);
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, this.m_rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_IMAGES, this.m_skipImages);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, this.m_skipRows);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, this.m_skipPixels);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, this.m_alignment);
        this.m_context.texSubImage3D(
            gl.TEXTURE_3D, 0, this.m_subX, this.m_subY, this.m_subZ,
            this.m_subW, this.m_subH, this.m_subD, transferFmt.format,
            transferFmt.dataType, this.m_offset
        );
    };

    // TexImage2D() depth case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} imageWidth
     * @param {number} imageHeight
     */
    es3fTextureSpecificationTests.TexImage2DDepthCase = function(
        name, desc, internalFormat, imageWidth, imageHeight
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), imageWidth, imageHeight,
            es3fTextureSpecificationTests.maxLevelCount(
                imageWidth, imageHeight
            )
        );

        this.m_internalFormat = internalFormat;
        // we are interested in the behavior near [-2, 2], map it to visible range [0, 1]
        this.m_texFormatInfo.lookupBias = [0.25, 0.0, 0.0, 1.0];
        this.m_texFormatInfo.lookupScale = [0.5, 1.0, 1.0, 0.0];
    };

    es3fTextureSpecificationTests.TexImage2DDepthCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage2DDepthCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DDepthCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.TexImage2DDepthCase.prototype.createTexture =
    function() {
        var fmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = [-1.5, -2.0, 1.7, -1.5];
            var gMax = [2.0, 1.5, -1.0, 2.0];

            levelData.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, levelW, levelH, 0,
                fmt.format, fmt.dataType, levelData.getAccess().getDataPtr()
            );
        }
    };

    // TexImage3D() depth case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} imageWidth
     * @param {number} imageHeight
     * @param {number} numLayers
     */
    es3fTextureSpecificationTests.TexImage2DArrayDepthCase = function(
        name, desc, internalFormat, imageWidth, imageHeight, numLayers
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), imageWidth, imageHeight, numLayers,
            es3fTextureSpecificationTests.maxLevelCount(
                imageWidth, imageHeight
            )
        );

        this.m_internalFormat = internalFormat;
        // we are interested in the behavior near [-2, 2], map it to visible range [0, 1]
        this.m_texFormatInfo.lookupBias = [0.25, 0.0, 0.0, 1.0];
        this.m_texFormatInfo.lookupScale = [0.5, 1.0, 1.0, 0.0];
    };

    es3fTextureSpecificationTests.TexImage2DArrayDepthCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage2DArrayDepthCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DArrayDepthCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImage2DArrayDepthCase.prototype.createTexture = function() {
        var fmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = [-1.5, -2.0, 1.7, -1.5];
            var gMax = [2.0, 1.5, -1.0, 2.0];

            levelData.setSize(levelW, levelH, this.m_numLayers);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage3D(
                gl.TEXTURE_2D_ARRAY, ndx, this.m_internalFormat, levelW, levelH,
                this.m_numLayers, 0, fmt.format, fmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // TexSubImage2D() depth case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} imageWidth
     * @param {number} imageHeight
     */
    es3fTextureSpecificationTests.TexSubImage2DDepthCase = function(
        name, desc, internalFormat, imageWidth, imageHeight
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), imageWidth, imageHeight,
            es3fTextureSpecificationTests.maxLevelCount(
                imageWidth, imageHeight
            )
        );

        this.m_internalFormat = internalFormat;
        // we are interested in the behavior near [-2, 2], map it to visible range [0, 1]
        this.m_texFormatInfo.lookupBias = [0.25, 0.0, 0.0, 1.0];
        this.m_texFormatInfo.lookupScale = [0.5, 1.0, 1.0, 0.0];
    };

    es3fTextureSpecificationTests.TexSubImage2DDepthCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage2DDepthCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DDepthCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DDepthCase.prototype.createTexture = function() {
        var fmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // First specify full texture.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = [-1.5, -2.0, 1.7, -1.5];
            var gMax = [2.0, 1.5, -1.0, 2.0];

            levelData.setSize(levelW, levelH);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );

            this.m_context.texImage2D(
                gl.TEXTURE_2D, ndx, this.m_internalFormat, levelW, levelH, 0,
                fmt.format, fmt.dataType, levelData.getAccess().getDataPtr()
            );
        }

        // Re-specify parts of each level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            var w = rnd.getInt(1, levelW);
            var h = rnd.getInt(1, levelH);
            var x = rnd.getInt(0, levelW - w);
            var y = rnd.getInt(0, levelH - h);

             var colorA = [2.0, 1.5, -1.0, 2.0];
             var colorB = [-1.5, -2.0, 1.7, -1.5];
             var cellSize = rnd.getInt(2, 16);

            levelData.setSize(w, h);
            tcuTextureUtil.fillWithGrid(
                levelData.getAccess(), cellSize, colorA, colorB
            );

            this.m_context.texSubImage2D(
                gl.TEXTURE_2D, ndx, x, y, w, h, fmt.format, fmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }
    };

    // TexSubImage3D() depth case.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} imageWidth
     * @param {number} imageHeight
     * @param {number} numLayers
     */
    es3fTextureSpecificationTests.TexSubImage2DArrayDepthCase = function(
        name, desc, internalFormat, imageWidth, imageHeight, numLayers
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), imageWidth, imageHeight, numLayers,
            es3fTextureSpecificationTests.maxLevelCount(
                imageWidth, imageHeight
            )
        );

        this.m_internalFormat = internalFormat;
        // we are interested in the behavior near [-2, 2], map it to visible range [0, 1]
        this.m_texFormatInfo.lookupBias = [0.25, 0.0, 0.0, 1.0];
        this.m_texFormatInfo.lookupScale = [0.5, 1.0, 1.0, 0.0];
    };

    es3fTextureSpecificationTests.TexSubImage2DArrayDepthCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexSubImage2DArrayDepthCase.prototype.constructor =
        es3fTextureSpecificationTests.TexSubImage2DArrayDepthCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexSubImage2DArrayDepthCase.prototype.createTexture = function() {
        var fmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var rnd = new deRandom.Random(deString.deStringHash(this.fullName()));
        var tex = null;
        var levelData = new tcuTexture.TextureLevel(
            this.m_texFormat, this.m_width, this.m_height
        );

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, 1);

        // First specify full texture.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);
            var gMin = [-1.5, -2.0, 1.7, -1.5];
            var gMax = [2.0, 1.5, -1.0, 2.0];

            levelData.setSize(levelW, levelH, this.m_numLayers);
            tcuTextureUtil.fillWithComponentGradients(
                levelData.getAccess(), gMin, gMax
            );
            this.m_context.texImage3D(
                gl.TEXTURE_2D_ARRAY, ndx, this.m_internalFormat, levelW, levelH,
                this.m_numLayers, 0, fmt.format, fmt.dataType,
                levelData.getAccess().getDataPtr()
            );
        }

        // Re-specify parts of each level.
        for (var ndx = 0; ndx < this.m_numLevels; ndx++) {
            var levelW = Math.max(1, this.m_width >> ndx);
            var levelH = Math.max(1, this.m_height >> ndx);

            var w = rnd.getInt(1, levelW);
            var h = rnd.getInt(1, levelH);
            var d = rnd.getInt(1, this.m_numLayers);
            var x = rnd.getInt(0, levelW - w);
            var y = rnd.getInt(0, levelH - h);
            var z = rnd.getInt(0, this.m_numLayers - d);

             var colorA = [2.0, 1.5, -1.0, 2.0];
             var colorB = [-1.5, -2.0, 1.7, -1.5];
             var cellSize = rnd.getInt(2, 16);

            levelData.setSize(w, h, d);
            tcuTextureUtil.fillWithGrid(
                levelData.getAccess(), cellSize, colorA, colorB
            );

            this.m_context.texSubImage3D(
                gl.TEXTURE_2D_ARRAY, ndx, x, y, z, w, h, d, fmt.format,
                fmt.dataType, levelData.getAccess().getDataPtr()
            );
        }
    };

    // TexImage2D() depth case with pbo.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DSpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} imageWidth
     * @param {number} imageHeight
     */
    es3fTextureSpecificationTests.TexImage2DDepthBufferCase = function(
        name, desc, internalFormat, imageWidth, imageHeight
    ) {
        es3fTextureSpecificationTests.Texture2DSpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), imageWidth, imageHeight, 1
        );

        this.m_internalFormat = internalFormat;
        // we are interested in the behavior near [-2, 2], map it to visible range [0, 1]
        this.m_texFormatInfo.lookupBias = [0.25, 0.0, 0.0, 1.0];
        this.m_texFormatInfo.lookupScale = [0.5, 1.0, 1.0, 0.0];
    };

    es3fTextureSpecificationTests.TexImage2DDepthBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DSpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage2DDepthBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DDepthBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImage2DDepthBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_width;
        var alignment = 4;
        var rowPitch = deMath.deAlign32(rowLength * pixelSize, alignment);
        var height = this.m_height;
        var buf = null;
        var tex = null;
        var data = new ArrayBuffer(rowPitch * height);

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        // Fill data with gradient
        var gMin = [-1.5, -2.0, 1.7, -1.5];
        var gMax = [2.0, 1.5, -1.0, 2.0];

        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        rowPitch: rowPitch,
                        data: data
                    });
        tcuTextureUtil.fillWithComponentGradients(access, gMin, gMax);

        // Create buffer and upload.
        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(
            gl.PIXEL_UNPACK_BUFFER, access.getBuffer(), gl.STATIC_DRAW
        );

        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D, tex);
        this.m_context.texImage2D(
            gl.TEXTURE_2D, 0, this.m_internalFormat, this.m_width,
            this.m_height, 0, transferFmt.format, transferFmt.dataType, 0
        );
        this.m_context.deleteBuffer(buf);
    };

    // TexImage3D() depth case with pbo.
    /**
     * @constructor
     * @extends {es3fTextureSpecificationTests.Texture2DArraySpecCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} internalFormat
     * @param {number} imageWidth
     * @param {number} imageHeight
     */
    es3fTextureSpecificationTests.TexImage2DArrayDepthBufferCase = function(
        name, desc, internalFormat, imageWidth, imageHeight, numLayers
    ) {
        es3fTextureSpecificationTests.Texture2DArraySpecCase.call(
            this, name, desc, gluTextureUtil.mapGLInternalFormat(
                internalFormat
            ), imageWidth, imageHeight, numLayers, 1
        );

        this.m_internalFormat = internalFormat;
        // we are interested in the behavior near [-2, 2], map it to visible range [0, 1]
        this.m_texFormatInfo.lookupBias = [0.25, 0.0, 0.0, 1.0];
        this.m_texFormatInfo.lookupScale = [0.5, 1.0, 1.0, 0.0];
    };

    es3fTextureSpecificationTests.TexImage2DArrayDepthBufferCase.prototype =
        Object.create(
            es3fTextureSpecificationTests.Texture2DArraySpecCase.prototype
        );

    es3fTextureSpecificationTests.
    TexImage2DArrayDepthBufferCase.prototype.constructor =
        es3fTextureSpecificationTests.TexImage2DArrayDepthBufferCase;

    /**
     * createTexture
     */
    es3fTextureSpecificationTests.
    TexImage2DArrayDepthBufferCase.prototype.createTexture = function() {
        var transferFmt = gluTextureUtil.getTransferFormat(this.m_texFormat);
        var pixelSize = this.m_texFormat.getPixelSize();
        var rowLength = this.m_width;
        var alignment = 4;
        var rowPitch = deMath.deAlign32(rowLength * pixelSize, alignment);
        var imageHeight = this.m_height;
        var slicePitch = imageHeight * rowPitch;
        var tex = null;
        var buf = null;
        var data = new ArrayBuffer(slicePitch * this.m_numLayers);

        assertMsgOptions(
            this.m_numLevels == 1, 'Number of levels is different than 1',
            false, true
        );

        // Fill data with gradient
        var gMin = [-1.5, -2.0, 1.7, -1.5];
        var gMax = [2.0, 1.5, -1.0, 2.0];

        var access = new tcuTexture.PixelBufferAccess({
                        format: this.m_texFormat,
                        width: this.m_width,
                        height: this.m_height,
                        depth: this.m_numLayers,
                        rowPitch: rowPitch,
                        slicePitch: slicePitch,
                        data: data
                    });
        tcuTextureUtil.fillWithComponentGradients(access, gMin, gMax);

        buf = this.m_context.createBuffer();
        this.m_context.bindBuffer(gl.PIXEL_UNPACK_BUFFER, buf);
        this.m_context.bufferData(
            gl.PIXEL_UNPACK_BUFFER, access.getBuffer(), gl.STATIC_DRAW
        );

        this.m_context.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, rowLength);
        this.m_context.pixelStorei(gl.UNPACK_ROW_LENGTH, rowLength);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_IMAGES, 0);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        this.m_context.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        this.m_context.pixelStorei(gl.UNPACK_ALIGNMENT, alignment);

        tex = this.m_context.createTexture();
        this.m_context.bindTexture(gl.TEXTURE_2D_ARRAY, tex);
        this.m_context.texImage3D(
            gl.TEXTURE_2D_ARRAY, 0, this.m_internalFormat, this.m_width,
            this.m_height, this.m_numLayers, 0, transferFmt.format,
            transferFmt.dataType, 0
        );
        this.m_context.deleteBuffer(buf);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fTextureSpecificationTests.TextureSpecificationTests = function() {
        tcuTestCase.DeqpTest.call(
            this, 'specification', 'Texture Specification Tests'
        );
    };

    es3fTextureSpecificationTests.TextureSpecificationTests.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);
    es3fTextureSpecificationTests.TextureSpecificationTests.prototype.constructor =
        es3fTextureSpecificationTests.TextureSpecificationTests;

    es3fTextureSpecificationTests.TextureSpecificationTests.prototype.init = function() {
        /**
         * @type {Array<number>}
         */
        es3fTextureSpecificationTests.s_cubeMapFaces = [
            gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
            gl.TEXTURE_CUBE_MAP_POSITIVE_X,
            gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
            gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
            gl.TEXTURE_CUBE_MAP_NEGATIVE_Z,
            gl.TEXTURE_CUBE_MAP_POSITIVE_Z
        ];

        /** @type {Array<{name: string, format: number, dataType: number}>} */
        var unsizedFormats = [{
                name: 'alpha_unsigned_byte',
                format: gl.ALPHA,
                dataType: gl.UNSIGNED_BYTE
            }, {
                name: 'luminance_unsigned_byte',
                format: gl.LUMINANCE,
                dataType: gl.UNSIGNED_BYTE
            }, {
                name: 'luminance_alpha_unsigned_byte',
                format: gl.LUMINANCE_ALPHA,
                dataType: gl.UNSIGNED_BYTE
            }, {
                name: 'rgb_unsigned_short_5_6_5',
                format: gl.RGB,
                dataType: gl.UNSIGNED_SHORT_5_6_5
            }, {
                name: 'rgb_unsigned_byte',
                format: gl.RGB,
                dataType: gl.UNSIGNED_BYTE
            }, {
                name: 'rgba_unsigned_short_4_4_4_4',
                format: gl.RGBA,
                dataType: gl.UNSIGNED_SHORT_4_4_4_4
            }, {
                name: 'rgba_unsigned_short_5_5_5_1',
                format: gl.RGBA,
                dataType: gl.UNSIGNED_SHORT_5_5_5_1
            }, {
                name: 'rgba_unsigned_byte',
                format: gl.RGBA,
                dataType: gl.UNSIGNED_BYTE
            }
        ];

        /** @type {Array<{name: string, internalFormat: number}>} */
        var colorFormats = [{
                name: 'rgba32f', internalFormat: gl.RGBA32F
            }, {
                name: 'rgba32i', internalFormat: gl.RGBA32I
            }, {
                name: 'rgba32ui', internalFormat: gl.RGBA32UI
            }, {
                name: 'rgba16f', internalFormat: gl.RGBA16F
            }, {
                name: 'rgba16i', internalFormat: gl.RGBA16I
            }, {
                name: 'rgba16ui', internalFormat: gl.RGBA16UI
            }, {
                name: 'rgba8', internalFormat: gl.RGBA8
            }, {
                name: 'rgba8i', internalFormat: gl.RGBA8I
            }, {
                name: 'rgba8ui', internalFormat: gl.RGBA8UI
            }, {
                name: 'srgb8_alpha8', internalFormat: gl.SRGB8_ALPHA8
            }, {
                name: 'rgb10_a2', internalFormat: gl.RGB10_A2
            }, {
                name: 'rgb10_a2ui', internalFormat: gl.RGB10_A2UI
            }, {
                name: 'rgba4', internalFormat: gl.RGBA4
            }, {
                name: 'rgb5_a1', internalFormat: gl.RGB5_A1
            }, {
                name: 'rgba8_snorm', internalFormat: gl.RGBA8_SNORM
            }, {
                name: 'rgb8', internalFormat: gl.RGB8
            }, {
                name: 'rgb565', internalFormat: gl.RGB565
            }, {
                name: 'r11f_g11f_b10f', internalFormat: gl.R11F_G11F_B10F
            }, {
                name: 'rgb32f', internalFormat: gl.RGB32F
            }, {
                name: 'rgb32i', internalFormat: gl.RGB32I
            }, {
                name: 'rgb32ui', internalFormat: gl.RGB32UI
            }, {
                name: 'rgb16f', internalFormat: gl.RGB16F
            }, {
                name: 'rgb16i', internalFormat: gl.RGB16I
            }, {
                name: 'rgb16ui', internalFormat: gl.RGB16UI
            }, {
                name: 'rgb8_snorm', internalFormat: gl.RGB8_SNORM
            }, {
                name: 'rgb8i', internalFormat: gl.RGB8I
            }, {
                name: 'rgb8ui', internalFormat: gl.RGB8UI
            }, {
                name: 'srgb8', internalFormat: gl.SRGB8
            }, {
                name: 'rgb9_e5', internalFormat: gl.RGB9_E5
            }, {
                name: 'rg32f', internalFormat: gl.RG32F
            }, {
                name: 'rg32i', internalFormat: gl.RG32I
            }, {
                name: 'rg32ui', internalFormat: gl.RG32UI
            }, {
                name: 'rg16f', internalFormat: gl.RG16F
            }, {
                name: 'rg16i', internalFormat: gl.RG16I
            }, {
                name: 'rg16ui', internalFormat: gl.RG16UI
            }, {
                name: 'rg8', internalFormat: gl.RG8
            }, {
                name: 'rg8i', internalFormat: gl.RG8I
            }, {
                name: 'rg8ui', internalFormat: gl.RG8UI
            }, {
                name: 'rg8_snorm', internalFormat: gl.RG8_SNORM
            }, {
                name: 'r32f', internalFormat: gl.R32F
            }, {
                name: 'r32i', internalFormat: gl.R32I
            }, {
                name: 'r32ui', internalFormat: gl.R32UI
            }, {
                name: 'r16f', internalFormat: gl.R16F
            }, {
                name: 'r16i', internalFormat: gl.R16I
            }, {
                name: 'r16ui', internalFormat: gl.R16UI
            }, {
                name: 'r8', internalFormat: gl.R8
            }, {
                name: 'r8i', internalFormat: gl.R8I
            }, {
                name: 'r8ui', internalFormat: gl.R8UI
            }, {
                name: 'r8_snorm', internalFormat: gl.R8_SNORM
            }
        ];

        // Depth and stencil formats
        /** @type {Array<{name: string, internalFormat: number}>} */
        var depthStencilFormats = [{
                name: 'depth_component32f',
                internalFormat: gl.DEPTH_COMPONENT32F
            }, {
                name: 'depth_component24',
                internalFormat: gl.DEPTH_COMPONENT24
            }, {
                name: 'depth_component16',
                internalFormat: gl.DEPTH_COMPONENT16
            }, {
                name: 'depth32f_stencil8',
                internalFormat: gl.DEPTH32F_STENCIL8
            }, {
                name: 'depth24_stencil8',
                internalFormat: gl.DEPTH24_STENCIL8
            }
        ];

        // Basic TexImage2D usage.
        var splitBasicTex2D = 2, splitBasicTexCube = 5;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */
        var basicTexImageGroup2D = [];
        for (var ii = 0; ii < splitBasicTex2D; ++ii) {
            basicTexImageGroup2D.push(
                new tcuTestCase.DeqpTest('basic_teximage2d', 'Basic glTexImage2D() usage')
            );
            this.addChild(basicTexImageGroup2D[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */
        var basicTexImageGroupCube = [];
        for (var ii = 0; ii < splitBasicTexCube; ++ii) {
            basicTexImageGroupCube.push(
                new tcuTestCase.DeqpTest('basic_teximage2d', 'Basic glTexImage2D() usage')
            );
            this.addChild(basicTexImageGroupCube[ii]);
        }
        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            /** @type {string} */
            var fmtName = colorFormats[formatNdx].name;
            /** @type {number} */
            var format = colorFormats[formatNdx].internalFormat;
            /** @type {number} */
            var tex2DWidth = 64;
            /** @type {number} */
            var tex2DHeight = 128;
            /** @type {number} */
            var texCubeSize = 64;

            basicTexImageGroup2D[formatNdx % splitBasicTex2D].addChild(
                es3fTextureSpecificationTests.newBasicTexImage2DCaseInternal(
                    fmtName + '_2d', '', format, tex2DWidth, tex2DHeight
                )
            );
            basicTexImageGroupCube[formatNdx % splitBasicTexCube].addChild(
                es3fTextureSpecificationTests.newBasicTexImageCubeCaseInternal(
                    fmtName + '_cube', '', format, texCubeSize
                )
            );
        }

        // Randomized TexImage2D order.
        /** @type {tcuTestCase.DeqpTest} */
        var randomTexImageGroup = new tcuTestCase.DeqpTest(
            'random_teximage2d', 'Randomized glTexImage2D() usage'
        );
        this.addChild(randomTexImageGroup);
        var rnd = new deRandom.Random(9);

        // 2D cases.
        for (var ndx = 0; ndx < 10; ndx++) {
            var formatNdx = rnd.getInt(0, colorFormats.length - 1);
            var width = 1 << rnd.getInt(2, 8);
            var height = 1 << rnd.getInt(2, 8);

            randomTexImageGroup.addChild(
                es3fTextureSpecificationTests.newRandomOrderTexImage2DCaseInternal(
                    '2d.' + colorFormats[formatNdx].name, '',
                    colorFormats[formatNdx].internalFormat, width, height
                )
            );
        }

        // Cubemap cases.
        randomTexImageGroup = new tcuTestCase.DeqpTest(
            'random_teximage2d', 'Randomized glTexImage2D() usage'
        );
        this.addChild(randomTexImageGroup);

        for (var ndx = 0; ndx < 10; ndx++) {
            formatNdx = rnd.getInt(0, colorFormats.length - 1);
            /** @type {number} */ var size = 1 << rnd.getInt(2, 8);

            randomTexImageGroup.addChild(
                es3fTextureSpecificationTests.newRandomOrderTexImageCubeCaseInternal(
                    'cube.' + colorFormats[formatNdx].name, '',
                    colorFormats[formatNdx].internalFormat, size
                )
            );
        }

        // TexImage2D unpack alignment.
        /** @type {tcuTestCase.DeqpTest} */
        var alignGroup = new tcuTestCase.DeqpTest(
            'teximage2d_align', 'glTexImage2D() unpack alignment tests'
        );
        this.addChild(alignGroup);

        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_r8_4_8', '', gl.R8, 4, 8, 4, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_r8_63_1', '', gl.R8, 63, 30, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_r8_63_2', '', gl.R8, 63, 30, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_r8_63_4', '', gl.R8, 63, 30, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_r8_63_8', '', gl.R8, 63, 30, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba4_51_1', '', gl.RGBA4, 51, 30, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba4_51_2', '', gl.RGBA4, 51, 30, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba4_51_4', '', gl.RGBA4, 51, 30, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba4_51_8', '', gl.RGBA4, 51, 30, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgb8_39_1', '', gl.RGB8, 39, 43, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgb8_39_2', '', gl.RGB8, 39, 43, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgb8_39_4', '', gl.RGB8, 39, 43, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgb8_39_8', '', gl.RGB8, 39, 43, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba8_47_1', '', gl.RGBA8, 47, 27, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba8_47_2', '', gl.RGBA8, 47, 27, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba8_47_4', '', gl.RGBA8, 47, 27, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImage2DAlignCaseInternal(
                '2d_rgba8_47_8', '', gl.RGBA8, 47, 27, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_r8_4_8', '', gl.R8, 4, 3, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_r8_63_1', '', gl.R8, 63, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_r8_63_2', '', gl.R8, 63, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_r8_63_4', '', gl.R8, 63, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_r8_63_8', '', gl.R8, 63, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba4_51_1', '', gl.RGBA4, 51, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba4_51_2', '', gl.RGBA4, 51, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba4_51_4', '', gl.RGBA4, 51, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba4_51_8', '', gl.RGBA4, 51, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgb8_39_1', '', gl.RGB8, 39, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgb8_39_2', '', gl.RGB8, 39, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgb8_39_4', '', gl.RGB8, 39, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgb8_39_8', '', gl.RGB8, 39, 1, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba8_47_1', '', gl.RGBA8, 47, 1, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba8_47_2', '', gl.RGBA8, 47, 1, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba8_47_4', '', gl.RGBA8, 47, 1, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexImageCubeAlignCaseInternal(
                'cube_rgba8_47_8', '', gl.RGBA8, 47, 1, 8
            )
        );

        // glTexImage2D() unpack parameter cases.
        /** @type {tcuTestCase.DeqpTest} */
        var paramGroup = new tcuTestCase.DeqpTest(
            'teximage2d_unpack_params',
            'glTexImage2D() pixel transfer mode cases'
        );
        this.addChild(paramGroup);

        var cases = [{
                name: 'rgb8_alignment', format: gl.RGB8, width: 31,
                height: 30, rowLength: 0, skipRows: 0, skipPixels: 0,
                alignment: 2
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 31,
                height: 30, rowLength: 50, skipRows: 0, skipPixels: 0,
                alignment: 4
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 31,
                height: 30, rowLength: 0, skipRows: 3, skipPixels: 0,
                alignment: 4
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 31,
                height: 30, rowLength: 36, skipRows: 0, skipPixels: 5,
                alignment: 4
            }, {
                name: 'r8_complex1', format: gl.R8, width: 31,
                height: 30, rowLength: 64, skipRows: 1, skipPixels: 3,
                alignment: 1
            }, {
                name: 'r8_complex2', format: gl.R8, width: 31,
                height: 30, rowLength: 64, skipRows: 1, skipPixels: 3,
                alignment: 2
            }, {
                name: 'r8_complex3', format: gl.R8, width: 31,
                height: 30, rowLength: 64, skipRows: 1, skipPixels: 3,
                alignment: 4
            }, {
                name: 'r8_complex4', format: gl.R8, width: 31,
                height: 30, rowLength: 64, skipRows: 1, skipPixels: 3,
                alignment: 8
            }, {
                name: 'rgba8_complex1', format: gl.RGBA8, width: 56,
                height: 61, rowLength: 69, skipRows: 0, skipPixels: 0,
                alignment: 8
            }, {
                name: 'rgba8_complex2', format: gl.RGBA8, width: 56,
                height: 61, rowLength: 69, skipRows: 0, skipPixels: 7,
                alignment: 8
            }, {
                name: 'rgba8_complex3', format: gl.RGBA8, width: 56,
                height: 61, rowLength: 69, skipRows: 3, skipPixels: 0,
                alignment: 8
            }, {
                name: 'rgba8_complex4', format: gl.RGBA8, width: 56,
                height: 61, rowLength: 69, skipRows: 3, skipPixels: 7,
                alignment: 8
            }, {
                name: 'rgba32f_complex', format: gl.RGBA32F, width: 19,
                height: 10, rowLength: 27, skipRows: 1, skipPixels: 7,
                alignment: 8
            }
        ];

        for (var ndx = 0; ndx < cases.length; ndx++)
            paramGroup.addChild(
                new es3fTextureSpecificationTests.TexImage2DParamsCase(
                    cases[ndx].name, '', cases[ndx].format, cases[ndx].width,
                    cases[ndx].height, cases[ndx].rowLength,
                    cases[ndx].skipRows, cases[ndx].skipPixels,
                    cases[ndx].alignment
                )
            );

        // glTexImage2D() pbo cases.
        var splitPboTex2D = 2, splitPboTexCube = 5;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var pboGroup2D = [];
        for (var ii = 0; ii < splitPboTex2D; ++ii) {
            pboGroup2D.push(new tcuTestCase.DeqpTest('teximage2d_pbo', 'glTexImage2D() from PBO'));
            this.addChild(pboGroup2D[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var pboGroupCube = [];
        for (var ii = 0; ii < splitPboTexCube; ++ii) {
            pboGroupCube.push(new tcuTestCase.DeqpTest('teximage2d_pbo', 'glTexImage2D() from PBO'));
           this.addChild(pboGroupCube[ii]);
        }
        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            format = colorFormats[formatNdx].internalFormat;
            tex2DWidth = 65;
            tex2DHeight = 37;
            texCubeSize = 64;

            pboGroup2D[formatNdx % splitPboTex2D].addChild(
                new es3fTextureSpecificationTests.TexImage2DBufferCase(
                    fmtName + '_2d', '', format,
                    tex2DWidth, tex2DHeight, 0, 0, 0, 4, 0
                )
            );
            pboGroupCube[formatNdx % splitPboTexCube].addChild(
                new es3fTextureSpecificationTests.TexImageCubeBufferCase(
                    fmtName + '_cube', '', format,
                    texCubeSize, 0, 0, 0, 4, 0
                )
            );
        }

        // Parameter cases
        var pboGroupParams = new tcuTestCase.DeqpTest(
            'teximage2d_pbo', 'glTexImage2D() from PBO'
        );
        this.addChild(pboGroupParams);
        /**
         * @type {Array<{name: string, format: number, width: number,
         * height: number, rowLength: number, skipRows: number,
         * skipPixels: number, alignment: number, offset: number}>}
         */
        var parameterCases = [{
                name: 'rgb8_offset', format: gl.RGB8, width: 31,
                height: 30, rowLength: 0, skipRows: 0, skipPixels: 0,
                alignment: 4, offset: 67
            }, {
                name: 'rgb8_alignment', format: gl.RGB8, width: 31,
                height: 30, rowLength: 0, skipRows: 0, skipPixels: 0,
                alignment: 2, offset: 0
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 31,
                height: 30, rowLength: 50, skipRows: 0, skipPixels: 0,
                alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 31,
                height: 30, rowLength: 0, skipRows: 3, skipPixels: 0,
                alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 31,
                height: 30, rowLength: 36, skipRows: 0, skipPixels: 5,
                alignment: 4, offset: 0
            }
        ];
        for (var ndx = 0; ndx < parameterCases.length; ndx++) {
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexImage2DBufferCase(
                    parameterCases[ndx].name + '_2d', '',
                    parameterCases[ndx].format, parameterCases[ndx].width,
                    parameterCases[ndx].height, parameterCases[ndx].rowLength,
                    parameterCases[ndx].skipRows,
                    parameterCases[ndx].skipPixels,
                    parameterCases[ndx].alignment,
                    parameterCases[ndx].offset
                )
            );
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexImageCubeBufferCase(
                    parameterCases[ndx].name + '_cube', '',
                    parameterCases[ndx].format, parameterCases[ndx].width,
                    parameterCases[ndx].rowLength, parameterCases[ndx].skipRows,
                    parameterCases[ndx].skipPixels,
                    parameterCases[ndx].alignment, parameterCases[ndx].offset
                )
            );
        }

        // glTexImage2D() depth cases.
        /** @type {tcuTestCase.DeqpTest} */
        var shadow2dGroup = new tcuTestCase.DeqpTest(
            'teximage2d_depth',
            'glTexImage2D() with depth or depth/stencil format'
        );
        this.addChild(shadow2dGroup);

        for (var ndx = 0; ndx < depthStencilFormats.length; ndx++) {
            // WebGL 2 specific constraint.
            if (depthStencilFormats[ndx].internalFormat == gl.DEPTH32F_STENCIL8)
                continue;
            var tex2DWidth = 64;
            var tex2DHeight = 128;

            shadow2dGroup.addChild(
                new es3fTextureSpecificationTests.TexImage2DDepthCase(
                    depthStencilFormats[ndx].name, '',
                    depthStencilFormats[ndx].internalFormat,
                    tex2DWidth, tex2DHeight
                )
            );
        }

        // glTexImage2D() depth cases with pbo.
        shadow2dGroup = new tcuTestCase.DeqpTest(
            'teximage2d_depth_pbo',
            'glTexImage2D() with depth or depth/stencil format with pbo'
        );
        this.addChild(shadow2dGroup);

        for (var ndx = 0; ndx < depthStencilFormats.length; ndx++) {
            tex2DWidth = 64;
            tex2DHeight = 128;

            shadow2dGroup.addChild(
                new es3fTextureSpecificationTests.TexImage2DDepthBufferCase(
                    depthStencilFormats[ndx].name, '',
                    depthStencilFormats[ndx].internalFormat,
                    tex2DWidth, tex2DHeight
                )
            );
        }

        // Basic TexSubImage2D usage.
        splitBasicTex2D = 3;
        splitBasicTexCube = 5;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */
        var basicTexSubImageGroup2D = [];
        for (var ii = 0; ii < splitBasicTex2D; ++ii) {
            basicTexSubImageGroup2D.push(
                new tcuTestCase.DeqpTest('basic_texsubimage2d', 'Basic glTexSubImage2D() usage')
            );
            this.addChild(basicTexSubImageGroup2D[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */
        var basicTexSubImageGroupCube = [];
        for (var ii = 0; ii < splitBasicTexCube; ++ii) {
            basicTexSubImageGroupCube.push(
                new tcuTestCase.DeqpTest('basic_texsubimage2d', 'Basic glTexSubImage2D() usage')
            );
            this.addChild(basicTexSubImageGroupCube[ii]);
        }
        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            format = colorFormats[formatNdx].internalFormat;
            tex2DWidth = 64;
            tex2DHeight = 128;
            texCubeSize = 64;

            basicTexSubImageGroup2D[formatNdx % splitBasicTex2D].addChild(
                es3fTextureSpecificationTests.newBasicTexSubImage2DCaseInternal(
                    fmtName + '_2d', '', format, tex2DWidth, tex2DHeight
                )
            );
            basicTexSubImageGroupCube[formatNdx % splitBasicTexCube].addChild(
                es3fTextureSpecificationTests.newBasicTexImageCubeCaseInternal(
                    fmtName + '_cube', '', format, texCubeSize
                )
            );
        }

        // TexSubImage2D to empty texture.
        /** @type {tcuTestCase.DeqpTest} */
        var texSubImageEmptyTexGroup = new tcuTestCase.DeqpTest(
            'texsubimage2d_empty_tex',
            'glTexSubImage2D() to texture that has storage but no data'
        );
        this.addChild(texSubImageEmptyTexGroup);
        for (var formatNdx = 0; formatNdx < unsizedFormats.length; formatNdx++) {
            fmtName = unsizedFormats[formatNdx].name;
            format = unsizedFormats[formatNdx].format;
            /** @type {number} */
            var dataType = unsizedFormats[formatNdx].dataType;
            tex2DWidth = 64;
            tex2DHeight = 32;
            texCubeSize = 32;

            texSubImageEmptyTexGroup.addChild(
                new es3fTextureSpecificationTests.TexSubImage2DEmptyTexCase(
                    fmtName + '_2d', '', format, dataType, tex2DWidth, tex2DHeight
                )
            );
            texSubImageEmptyTexGroup.addChild(
                new es3fTextureSpecificationTests.TexSubImageCubeEmptyTexCase(
                    fmtName + '_cube', '', format, dataType, texCubeSize
                )
            );
        }

        // TexSubImage2D alignment cases.
        alignGroup = new tcuTestCase.DeqpTest(
            'texsubimage2d_align', 'glTexSubImage2D() unpack alignment tests'
        );
        this.addChild(alignGroup);

        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_1_1', '', gl.R8, 64, 64, 13, 17, 1, 6, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_1_2', '', gl.R8, 64, 64, 13, 17, 1, 6, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_1_4', '', gl.R8, 64, 64, 13, 17, 1, 6, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_1_8', '', gl.R8, 64, 64, 13, 17, 1, 6, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_63_1', '', gl.R8, 64, 64, 1, 9, 63, 30, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_63_2', '', gl.R8, 64, 64, 1, 9, 63, 30, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_63_4', '', gl.R8, 64, 64, 1, 9, 63, 30, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_r8_63_8', '', gl.R8, 64, 64, 1, 9, 63, 30, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba4_51_1', '', gl.RGBA4, 64, 64, 7, 29, 51, 30, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba4_51_2', '', gl.RGBA4, 64, 64, 7, 29, 51, 30, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba4_51_4', '', gl.RGBA4, 64, 64, 7, 29, 51, 30, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba4_51_8', '', gl.RGBA4, 64, 64, 7, 29, 51, 30, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgb8_39_1', '', gl.RGB8, 64, 64, 11, 8, 39, 43, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgb8_39_2', '', gl.RGB8, 64, 64, 11, 8, 39, 43, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgb8_39_4', '', gl.RGB8, 64, 64, 11, 8, 39, 43, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgb8_39_8', '', gl.RGB8, 64, 64, 11, 8, 39, 43, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba8_47_1', '', gl.RGBA8, 64, 64, 10, 1, 47, 27, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba8_47_2', '', gl.RGBA8, 64, 64, 10, 1, 47, 27, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba8_47_4', '', gl.RGBA8, 64, 64, 10, 1, 47, 27, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImage2DAlignCaseInternal(
                '2d_rgba8_47_8', '', gl.RGBA8, 64, 64, 10, 1, 47, 27, 8
            )
        );

        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_1_1', '', gl.R8, 64, 13, 17, 1, 6, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_1_2', '', gl.R8, 64, 13, 17, 1, 6, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_1_4', '', gl.R8, 64, 13, 17, 1, 6, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_1_8', '', gl.R8, 64, 13, 17, 1, 6, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_63_1', '', gl.R8, 64, 1, 9, 63, 30, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_63_2', '', gl.R8, 64, 1, 9, 63, 30, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_63_4', '', gl.R8, 64, 1, 9, 63, 30, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_r8_63_8', '', gl.R8, 64, 1, 9, 63, 30, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba4_51_1', '', gl.RGBA4, 64, 7, 29, 51, 30, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba4_51_2', '', gl.RGBA4, 64, 7, 29, 51, 30, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba4_51_4', '', gl.RGBA4, 64, 7, 29, 51, 30, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba4_51_8', '', gl.RGBA4, 64, 7, 29, 51, 30, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgb8_39_1', '', gl.RGB8, 64, 11, 8, 39, 43, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgb8_39_2', '', gl.RGB8, 64, 11, 8, 39, 43, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgb8_39_4', '', gl.RGB8, 64, 11, 8, 39, 43, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgb8_39_8', '', gl.RGB8, 64, 11, 8, 39, 43, 8
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba8_47_1', '', gl.RGBA8, 64, 10, 1, 47, 27, 1
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba8_47_2', '', gl.RGBA8, 64, 10, 1, 47, 27, 2
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba8_47_4', '', gl.RGBA8, 64, 10, 1, 47, 27, 4
            )
        );
        alignGroup.addChild(
            es3fTextureSpecificationTests.newTexSubImageCubeAlignCaseInternal(
                'cube_rgba8_47_8', '', gl.RGBA8, 64, 10, 1, 47, 27, 8
            )
        );

        // glTexSubImage2D() pixel transfer mode cases.
        paramGroup = new tcuTestCase.DeqpTest(
            'texsubimage2d_unpack_params',
            'glTexSubImage2D() pixel transfer mode cases'
        );
        this.addChild(paramGroup);

        cases = [{
                name: 'rgb8_alignment', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 0, skipRows: 0, skipPixels: 0, alignment: 2
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 50, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 0, skipRows: 3, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 36, skipRows: 0, skipPixels: 5, alignment: 4
            }, {
                name: 'r8_complex1', format: gl.R8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 64, skipRows: 1, skipPixels: 3, alignment: 1
            }, {
                name: 'r8_complex2', format: gl.R8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 64, skipRows: 1, skipPixels: 3, alignment: 2
            }, {
                name: 'r8_complex3', format: gl.R8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 64, skipRows: 1, skipPixels: 3, alignment: 4
            }, {
                name: 'r8_complex4', format: gl.R8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 64, skipRows: 1, skipPixels: 3, alignment: 8
            }, {
                name: 'rgba8_complex1', format: gl.RGBA8, width: 92,
                height: 84, subX: 13, subY: 19, subW: 56, subH: 61,
                rowLength: 69, skipRows: 0, skipPixels: 0, alignment: 8
            }, {
                name: 'rgba8_complex2', format: gl.RGBA8, width: 92,
                height: 84, subX: 13, subY: 19, subW: 56, subH: 61,
                rowLength: 69, skipRows: 0, skipPixels: 7, alignment: 8
            }, {
                name: 'rgba8_complex3', format: gl.RGBA8, width: 92,
                height: 84, subX: 13, subY: 19, subW: 56, subH: 61,
                rowLength: 69, skipRows: 3, skipPixels: 0, alignment: 8
            }, {
                name: 'rgba8_complex4', format: gl.RGBA8, width: 92,
                height: 84, subX: 13, subY: 19, subW: 56, subH: 61,
                rowLength: 69, skipRows: 3, skipPixels: 7, alignment: 8
            }, {
                name: 'rgba32f_complex', format: gl.RGBA32F, width: 92,
                height: 84, subX: 13, subY: 19, subW: 56, subH: 61,
                rowLength: 69, skipRows: 3, skipPixels: 7, alignment: 8
            }
        ];

        for (var ndx = 0; ndx < cases.length; ndx++)
            paramGroup.addChild(
                new es3fTextureSpecificationTests.TexSubImage2DParamsCase(
                    cases[ndx].name, '', cases[ndx].format, cases[ndx].width,
                    cases[ndx].height, cases[ndx].subX, cases[ndx].subY,
                    cases[ndx].subW, cases[ndx].subH, cases[ndx].rowLength,
                    cases[ndx].skipRows, cases[ndx].skipPixels,
                    cases[ndx].alignment
                )
            );

        // glTexSubImage2D() PBO cases.
        splitPboTex2D = 2;
        splitPboTexCube = 5;
        pboGroup2D = [];
        for (var ii = 0; ii < splitPboTex2D; ++ii) {
            pboGroup2D.push(new tcuTestCase.DeqpTest(
                'texsubimage2d_pbo',
                'glTexSubImage2D() pixel buffer object tests'
            ));
            this.addChild(pboGroup2D[ii]);
        }
        pboGroupCube = [];
        for (var ii = 0; ii < splitPboTexCube; ++ii) {
            pboGroupCube.push(new tcuTestCase.DeqpTest(
                'texsubimage2d_pbo',
                'glTexSubImage2D() pixel buffer object tests'
            ));
            this.addChild(pboGroupCube[ii]);
        }

        for (var ndx = 0; ndx < colorFormats.length; ndx++) {
            pboGroup2D[ndx % splitPboTex2D].addChild(
                new es3fTextureSpecificationTests.TexSubImage2DBufferCase(
                    colorFormats[ndx].name + '_2d', '',
                    colorFormats[ndx].internalFormat,
                    54, // Width
                    60, // Height
                    11, // Sub X
                    7, // Sub Y
                    31, // Sub W
                    30, // Sub H
                    0, // Row len
                    0, // Skip rows
                    0, // Skip pixels
                    4, // Alignment
                    0 /* offset */
                )
            );
            pboGroupCube[ndx % splitPboTexCube].addChild(
                new es3fTextureSpecificationTests.TexSubImageCubeBufferCase(
                    colorFormats[ndx].name + '_cube', '',
                    colorFormats[ndx].internalFormat,
                    64, // Size
                    11, // Sub X
                    7, // Sub Y
                    31, // Sub W
                    30, // Sub H
                    0, // Row len
                    0, // Skip rows
                    0, // Skip pixels
                    4, // Alignment
                    0 /* offset */
                )
            );
        }

        pboGroupParams = new tcuTestCase.DeqpTest(
            'texsubimage2d_pbo',
            'glTexSubImage2D() pixel buffer object tests'
        );
        this.addChild(pboGroupParams);
        /** @type {Array<{name: string, format: number, width: number,
         * height: number, subX: number, subY: number,
         * subW: number, subH: number, rowLength: number, skipRows: number,
         * skipPixels: number, alignment: number, offset: number}>}
         */
        var paramCases = [{
                name: 'rgb8_offset', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 0, skipRows: 0, skipPixels: 0,
                alignment: 4, offset: 67
            }, {
                name: 'rgb8_alignment', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 0, skipRows: 0, skipPixels: 0,
                alignment: 2, offset: 0
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 50, skipRows: 0, skipPixels: 0,
                alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 0, skipRows: 3, skipPixels: 0,
                alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 54,
                height: 60, subX: 11, subY: 7, subW: 31, subH: 30,
                rowLength: 36, skipRows: 0, skipPixels: 5,
                alignment: 4, offset: 0
            }
        ];

        for (var ndx = 0; ndx < paramCases.length; ndx++) {
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexSubImage2DBufferCase(
                    paramCases[ndx].name + '_2d', '',
                    paramCases[ndx].format,
                    paramCases[ndx].width,
                    paramCases[ndx].height,
                    paramCases[ndx].subX,
                    paramCases[ndx].subY,
                    paramCases[ndx].subW,
                    paramCases[ndx].subH,
                    paramCases[ndx].rowLength,
                    paramCases[ndx].skipRows,
                    paramCases[ndx].skipPixels,
                    paramCases[ndx].alignment,
                    paramCases[ndx].offset));
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexSubImageCubeBufferCase(
                    paramCases[ndx].name + '_cube', '',
                    paramCases[ndx].format,
                    paramCases[ndx].width,
                    paramCases[ndx].subX,
                    paramCases[ndx].subY,
                    paramCases[ndx].subW,
                    paramCases[ndx].subH,
                    paramCases[ndx].rowLength,
                    paramCases[ndx].skipRows,
                    paramCases[ndx].skipPixels,
                    paramCases[ndx].alignment,
                    paramCases[ndx].offset
                )
            );
        }

        // glTexSubImage2D() depth cases.
        shadow2dGroup = new tcuTestCase.DeqpTest(
            'texsubimage2d_depth',
            'glTexSubImage2D() with depth or depth/stencil format'
        );
        this.addChild(shadow2dGroup);

        for (var ndx = 0; ndx < depthStencilFormats.length; ndx++) {
            // WebGL 2 specific constraint.
            if (depthStencilFormats[ndx].internalFormat == gl.DEPTH32F_STENCIL8)
                continue;
            tex2DWidth = 64;
            tex2DHeight = 32;

            shadow2dGroup.addChild(
                new es3fTextureSpecificationTests.TexSubImage2DDepthCase(
                    depthStencilFormats[ndx].name, '',
                    depthStencilFormats[ndx].internalFormat,
                    tex2DWidth, tex2DHeight
                )
            );
        }

        // Basic glCopyTexImage2D() cases
        /** @type {tcuTestCase.DeqpTest} */
        var copyTexImageGroup = new tcuTestCase.DeqpTest(
            'basic_copyteximage2d', 'Basic glCopyTexImage2D() usage'
        );
        this.addChild(copyTexImageGroup);

        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImage2DCase(
                '2d_alpha', '', gl.ALPHA, 128, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImage2DCase(
                '2d_luminance', '', gl.LUMINANCE, 128, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImage2DCase(
                '2d_luminance_alpha', '', gl.LUMINANCE_ALPHA, 128, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImage2DCase(
                '2d_rgb', '', gl.RGB, 128, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImage2DCase(
                '2d_rgba', '', gl.RGBA, 128, 64
            )
        );

        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImageCubeCase(
                'cube_alpha', '', gl.ALPHA, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImageCubeCase(
                'cube_luminance', '', gl.LUMINANCE, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImageCubeCase(
                'cube_luminance_alpha', '', gl.LUMINANCE_ALPHA, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImageCubeCase(
                'cube_rgb', '', gl.RGB, 64
            )
        );
        copyTexImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexImageCubeCase(
                'cube_rgba', '', gl.RGBA, 64
            )
        );

        // Basic glCopyTexSubImage2D() cases
        /** @type {tcuTestCase.DeqpTest} */
        var copyTexSubImageGroup = new tcuTestCase.DeqpTest(
            'basic_copytexsubimage2d', 'Basic glCopyTexSubImage2D() usage'
        );
        this.addChild(copyTexSubImageGroup);

        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase(
                '2d_alpha', '', gl.ALPHA, gl.UNSIGNED_BYTE, 128, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase(
                '2d_luminance', '', gl.LUMINANCE, gl.UNSIGNED_BYTE, 128, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase(
                '2d_luminance_alpha', '', gl.LUMINANCE_ALPHA, gl.UNSIGNED_BYTE, 128, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase(
                '2d_rgb', '', gl.RGB, gl.UNSIGNED_BYTE, 128, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImage2DCase(
                '2d_rgba', '', gl.RGBA, gl.UNSIGNED_BYTE, 128, 64
            )
        );

        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase(
                'cube_alpha', '', gl.ALPHA, gl.UNSIGNED_BYTE, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase(
                'cube_luminance', '', gl.LUMINANCE, gl.UNSIGNED_BYTE, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase(
                'cube_luminance_alpha', '', gl.LUMINANCE_ALPHA, gl.UNSIGNED_BYTE, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase(
                'cube_rgb', '', gl.RGB, gl.UNSIGNED_BYTE, 64
            )
        );
        copyTexSubImageGroup.addChild(
            new es3fTextureSpecificationTests.BasicCopyTexSubImageCubeCase(
                'cube_rgba', '', gl.RGBA, gl.UNSIGNED_BYTE, 64
            )
        );

        // Basic TexImage3D usage.
        var splitBasicTex2DArray = 3, splitBasicTex3D = 5;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */
        var basicTexImageGroup2DArray = [];
        for (var ii = 0; ii < splitBasicTex2DArray; ++ii) {
            basicTexImageGroup2DArray.push(
                new tcuTestCase.DeqpTest('basic_teximage3d', 'Basic glTexImage3D() usage')
            );
            this.addChild(basicTexImageGroup2DArray[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */
        var basicTexImageGroup3D = [];
        for (var ii = 0; ii < splitBasicTex3D; ++ii) {
            basicTexImageGroup3D.push(
                new tcuTestCase.DeqpTest('basic_teximage3d', 'Basic glTexImage3D() usage')
            );
            this.addChild(basicTexImageGroup3D[ii]);
        }
        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            format = colorFormats[formatNdx].internalFormat;
            /** @type {number} */ var tex2DArrayWidth = 57;
            /** @type {number} */ var tex2DArrayHeight = 44;
            /** @type {number} */ var tex2DArrayLevels = 5;
            /** @type {number} */ var tex3DWidth = 63;
            /** @type {number} */ var tex3DHeight = 29;
            /** @type {number} */ var tex3DDepth = 11;

            basicTexImageGroup2DArray[formatNdx % splitBasicTex2DArray].addChild(
                new es3fTextureSpecificationTests.BasicTexImage2DArrayCase(
                    fmtName + '_2d_array', '', format,
                    tex2DArrayWidth, tex2DArrayHeight, tex2DArrayLevels
                )
            );
            basicTexImageGroup3D[formatNdx % splitBasicTex3D].addChild(
                new es3fTextureSpecificationTests.BasicTexImage3DCase(
                    fmtName + '_3d', '', format,
                    tex3DWidth, tex3DHeight, tex3DDepth
                )
            );
        }

        // glTexImage3D() unpack params cases.
        paramGroup = new tcuTestCase.DeqpTest(
            'teximage3d_unpack_params', 'glTexImage3D() unpack parameters'
        );
        this.addChild(paramGroup);

        cases = [{
                name: 'rgb8_image_height', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 26, rowLength: 0,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 27,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_images', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 0,
                skipImages: 3, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 22, rowLength: 0,
                skipImages: 0, skipRows: 3, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 25,
                skipImages: 0, skipRows: 0, skipPixels: 2, alignment: 4
            }, {
                name: 'r8_complex1', format: gl.R8, width: 13,
                height: 17, depth: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 1
            }, {
                name: 'r8_complex2', format: gl.R8, width: 13,
                height: 17, depth: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 2
            }, {
                name: 'r8_complex3', format: gl.R8, width: 13,
                height: 17, depth: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 4
            }, {
                name: 'r8_complex4', format: gl.R8, width: 13,
                height: 17, depth: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 8
            }, {
                name: 'rgba8_complex1', format: gl.RGBA8, width: 11,
                height: 20, depth: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 8
            }, {
                name: 'rgba8_complex2', format: gl.RGBA8, width: 11,
                height: 20, depth: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 2, skipPixels: 0, alignment: 8
            }, {
                name: 'rgba8_complex3', format: gl.RGBA8, width: 11,
                height: 20, depth: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 0, skipPixels: 3, alignment: 8
            }, {
                name: 'rgba8_complex4', format: gl.RGBA8, width: 11,
                height: 20, depth: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 2, skipPixels: 3, alignment: 8
            }, {
                name: 'rgba32f_complex', format: gl.RGBA32F, width: 11,
                height: 20, depth: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 2, skipPixels: 3, alignment: 8
             }
        ];

        for (var ndx = 0; ndx < cases.length; ndx++)
            paramGroup.addChild(
                new es3fTextureSpecificationTests.TexImage3DParamsCase(
                    cases[ndx].name, '', cases[ndx].format, cases[ndx].width,
                    cases[ndx].height, cases[ndx].depth, cases[ndx].imageHeight,
                    cases[ndx].rowLength, cases[ndx].skipImages,
                    cases[ndx].skipRows, cases[ndx].skipPixels,
                    cases[ndx].alignment
                )
            );

        // glTexImage3D() pbo cases.
        var splitTex2DArray = 2, splitTex3D = 2;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var pboGroup2DArray = [];
        for (var ii = 0; ii < splitTex2DArray; ++ii) {
            pboGroup2DArray.push(
                new tcuTestCase.DeqpTest('teximage3d_pbo', 'glTexImage3D() from PBO')
            );
            this.addChild(pboGroup2DArray[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var pboGroup3D = [];
        for (var ii = 0; ii < splitTex3D; ++ii) {
            pboGroup3D.push(
                new tcuTestCase.DeqpTest('teximage3d_pbo', 'glTexImage3D() from PBO')
            );
            this.addChild(pboGroup3D[ii]);
        }

        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            format = colorFormats[formatNdx].internalFormat;
            tex3DWidth = 11;
            tex3DHeight = 20;
            tex3DDepth = 8;

            pboGroup2DArray[formatNdx % splitTex2DArray].addChild(
                new es3fTextureSpecificationTests.TexImage2DArrayBufferCase(
                    fmtName + '_2d_array', '', format, tex3DWidth, tex3DHeight,
                    tex3DDepth, 0, 0, 0, 0, 0, 4, 0
                )
            );
            pboGroup3D[formatNdx % splitTex3D].addChild(
                new es3fTextureSpecificationTests.TexImage3DBufferCase(
                    fmtName + '_3d', '', format, tex3DWidth, tex3DHeight,
                    tex3DDepth, 0, 0, 0, 0, 0, 4, 0
                )
            );
        }

        // Parameter cases
        parameterCases = [{
                name: 'rgb8_offset', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 0,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 1,
                offset: 67
            }, {
                name: 'rgb8_alignment', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 0,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 2,
                offset: 0
            }, {
                name: 'rgb8_image_height', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 26, rowLength: 0,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 4,
                offset: 0
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 27,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 4,
                offset: 0
            }, {
                name: 'rgb8_skip_images', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 0,
                skipImages: 3, skipRows: 0, skipPixels: 0, alignment: 4,
                offset: 0
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 22, rowLength: 0,
                skipImages: 0, skipRows: 3, skipPixels: 0, alignment: 4,
                offset: 0
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 23,
                height: 19, depth: 8, imageHeight: 0, rowLength: 25,
                skipImages: 0, skipRows: 0, skipPixels: 2, alignment: 4,
                offset: 0
            }
        ];

        pboGroupParams = new tcuTestCase.DeqpTest('teximage3d_pbo', 'glTexImage3D() from PBO');
        this.addChild(pboGroupParams);
        for (var ndx = 0; ndx < parameterCases.length; ndx++) {
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexImage2DArrayBufferCase(
                    parameterCases[ndx].name + '_2d_array', '',
                    parameterCases[ndx].format, parameterCases[ndx].width,
                    parameterCases[ndx].height, parameterCases[ndx].depth,
                    parameterCases[ndx].imageHeight,
                    parameterCases[ndx].rowLength,
                    parameterCases[ndx].skipImages,
                    parameterCases[ndx].skipRows,
                    parameterCases[ndx].skipPixels,
                    parameterCases[ndx].alignment, parameterCases[ndx].offset
                )
            );
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexImage3DBufferCase(
                    parameterCases[ndx].name + '_3d', '',
                    parameterCases[ndx].format, parameterCases[ndx].width,
                    parameterCases[ndx].height, parameterCases[ndx].depth,
                    parameterCases[ndx].imageHeight,
                    parameterCases[ndx].rowLength,
                    parameterCases[ndx].skipImages,
                    parameterCases[ndx].skipRows,
                    parameterCases[ndx].skipPixels,
                    parameterCases[ndx].alignment, parameterCases[ndx].offset
                )
            );
        }

        // glTexImage3D() depth cases.
        /** @type {tcuTestCase.DeqpTest} */
        var shadow3dGroup = new tcuTestCase.DeqpTest(
            'teximage3d_depth',
            'glTexImage3D() with depth or depth/stencil format'
        );
        this.addChild(shadow3dGroup);

        for (var ndx = 0; ndx < depthStencilFormats.length; ndx++) {
            // WebGL 2 specific constraint.
            if (depthStencilFormats[ndx].internalFormat == gl.DEPTH32F_STENCIL8)
                continue;
            tex3DWidth = 32;
            tex3DHeight = 64;
            tex3DDepth = 8;

            shadow3dGroup.addChild(
                new es3fTextureSpecificationTests.TexImage2DArrayDepthCase(
                    depthStencilFormats[ndx].name + '_2d_array', '',
                    depthStencilFormats[ndx].internalFormat,
                    tex3DWidth, tex3DHeight, tex3DDepth
                )
            );
        }

        // glTexImage3D() depth cases with pbo.
        shadow3dGroup = new tcuTestCase.DeqpTest(
            'teximage3d_depth_pbo',
            'glTexImage3D() with depth or depth/stencil format with pbo'
        );
        this.addChild(shadow3dGroup);

        for (var ndx = 0; ndx < depthStencilFormats.length; ndx++) {
            tex3DWidth = 32;
            tex3DHeight = 64;
            tex3DDepth = 8;

            shadow3dGroup.addChild(
                new es3fTextureSpecificationTests.TexImage2DArrayDepthBufferCase(
                    depthStencilFormats[ndx].name + '_2d_array', '',
                    depthStencilFormats[ndx].internalFormat,
                    tex3DWidth, tex3DHeight, tex3DDepth
                )
            );
        }

        // Basic TexSubImage3D usage.
        splitTex3D = 5;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var basicTexSubImageGroup = [];
        for (var ii = 0; ii < splitTex3D; ++ii) {
            basicTexSubImageGroup.push(
                new tcuTestCase.DeqpTest('basic_texsubimage3d', 'Basic glTexSubImage3D() usage')
            );
            this.addChild(basicTexSubImageGroup[ii]);
        }

        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            format = colorFormats[formatNdx].internalFormat;
            tex3DWidth = 32;
            tex3DHeight = 64;
            tex3DDepth = 8;

            basicTexSubImageGroup[formatNdx % splitTex3D].addChild(
                new es3fTextureSpecificationTests.BasicTexSubImage3DCase(
                    fmtName + '_3d', '', format,
                    tex3DWidth, tex3DHeight, tex3DDepth
                )
            );
        }

        // glTexSubImage3D() unpack params cases.
        paramGroup = new tcuTestCase.DeqpTest(
            'texsubimage3d_unpack_params', 'glTexSubImage3D() unpack parameters'
        );
        this.addChild(paramGroup);

        /** @type {Array<{name: string, format: number, width: number,
         * height: number, depth: number, subX: number, subY: number,
         * subZ: number, subW: number, subH: number, subD: number,
         * imageHeight: number, rowLength: number, skipImages: number,
         * skipRows: number, skipPixels: number, alignment: number}>}
         */
        var casesSubImage3D = [{
                name: 'rgb8_image_height', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 26, rowLength: 0,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 27,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_images', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 0,
                skipImages: 3, skipRows: 0, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 22, rowLength: 0,
                skipImages: 0, skipRows: 3, skipPixels: 0, alignment: 4
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 25,
                skipImages: 0, skipRows: 0, skipPixels: 2, alignment: 4
            }, {
                name: 'r8_complex1', format: gl.R8, width: 15,
                height: 20, depth: 11, subX: 1, subY: 1, subZ: 0, subW: 13,
                subH: 17, subD: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 1
            }, {
                name: 'r8_complex2', format: gl.R8, width: 15,
                height: 20, depth: 11, subX: 1, subY: 1, subZ: 0, subW: 13,
                subH: 17, subD: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 2
            }, {
                name: 'r8_complex3', format: gl.R8, width: 15,
                height: 20, depth: 11, subX: 1, subY: 1, subZ: 0, subW: 13,
                subH: 17, subD: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 4
            }, {
                name: 'r8_complex4', format: gl.R8, width: 15,
                height: 20, depth: 11, subX: 1, subY: 1, subZ: 0, subW: 13,
                subH: 17, subD: 11, imageHeight: 23, rowLength: 15,
                skipImages: 2, skipRows: 3, skipPixels: 1, alignment: 8
            }, {
                name: 'rgba8_complex1', format: gl.RGBA8, width: 15,
                height: 25, depth: 10, subX: 0, subY: 5, subZ: 1, subW: 11,
                subH: 20, subD: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 0, skipPixels: 0, alignment: 8
            }, {
                name: 'rgba8_complex2', format: gl.RGBA8, width: 15,
                height: 25, depth: 10, subX: 0, subY: 5, subZ: 1, subW: 11,
                subH: 20, subD: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 2, skipPixels: 0, alignment: 8
            }, {
                name: 'rgba8_complex3', format: gl.RGBA8, width: 15,
                height: 25, depth: 10, subX: 0, subY: 5, subZ: 1, subW: 11,
                subH: 20, subD: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 0, skipPixels: 3, alignment: 8
            }, {
                name: 'rgba8_complex4', format: gl.RGBA8, width: 15,
                height: 25, depth: 10, subX: 0, subY: 5, subZ: 1, subW: 11,
                subH: 20, subD: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 2, skipPixels: 3, alignment: 8
            }, {
                name: 'rgba32f_complex', format: gl.RGBA32F, width: 15,
                height: 25, depth: 10, subX: 0, subY: 5, subZ: 1, subW: 11,
                subH: 20, subD: 8, imageHeight: 25, rowLength: 14,
                skipImages: 0, skipRows: 2, skipPixels: 3, alignment: 8
            }
        ];

        for (var ndx = 0; ndx < casesSubImage3D.length; ndx++)
            paramGroup.addChild(
                new es3fTextureSpecificationTests.TexSubImage3DParamsCase(
                    casesSubImage3D[ndx].name, '', casesSubImage3D[ndx].format,
                    casesSubImage3D[ndx].width, casesSubImage3D[ndx].height, casesSubImage3D[ndx].depth,
                    casesSubImage3D[ndx].subX, casesSubImage3D[ndx].subY, casesSubImage3D[ndx].subZ,
                    casesSubImage3D[ndx].subW, casesSubImage3D[ndx].subH, casesSubImage3D[ndx].subD,
                    casesSubImage3D[ndx].imageHeight, casesSubImage3D[ndx].rowLength,
                    casesSubImage3D[ndx].skipImages, casesSubImage3D[ndx].skipRows,
                    casesSubImage3D[ndx].skipPixels, casesSubImage3D[ndx].alignment
                )
            );

        // glTexSubImage3D() PBO cases.
        splitTex2DArray = 2;
        pboGroup2DArray = [];
        for (var ii = 0; ii < splitTex2DArray; ++ii) {
            pboGroup2DArray.push(
                new tcuTestCase.DeqpTest('texsubimage3d_pbo', 'glTexSubImage3D() pixel buffer object tests')
            );
            this.addChild(pboGroup2DArray[ii]);
        }

        splitTex3D = 2;
        pboGroup3D = [];
        for (var ii = 0; ii < splitTex3D; ++ii) {
            pboGroup3D.push(
                new tcuTestCase.DeqpTest('texsubimage3d_pbo', 'glTexSubImage3D() pixel buffer object tests')
            );
            this.addChild(pboGroup3D[ii]);
        }

        for (var ndx = 0; ndx < colorFormats.length; ndx++) {
            pboGroup2DArray[ndx % splitTex2DArray].addChild(
                new es3fTextureSpecificationTests.TexSubImage2DArrayBufferCase(
                    colorFormats[ndx].name + '_2d_array', '',
                    colorFormats[ndx].internalFormat,
                    26, // Width
                    25, // Height
                    10, // Depth
                    1, // Sub X
                    2, // Sub Y
                    0, // Sub Z
                    23, // Sub W
                    19, // Sub H
                    8, // Sub D
                    0, // Image height
                    0, // Row length
                    0, // Skip images
                    0, // Skip rows
                    0, // Skip pixels
                    4, // Alignment
                    0 // offset
                )
            );
            pboGroup3D[ndx % splitTex3D].addChild(
                new es3fTextureSpecificationTests.TexSubImage3DBufferCase(
                    colorFormats[ndx].name + '_3d', '',
                    colorFormats[ndx].internalFormat,
                    26, // Width
                    25, // Height
                    10, // Depth
                    1, // Sub X
                    2, // Sub Y
                    0, // Sub Z
                    23, // Sub W
                    19, // Sub H
                    8, // Sub D
                    0, // Image height
                    0, // Row length
                    0, // Skip images
                    0, // Skip rows
                    0, // Skip pixels
                    4, // Alignment
                    0 // offset
                )
            );
        }

        paramCases = [{
                name: 'rgb8_offset', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 0, skipImages: 0,
                skipRows: 0, skipPixels: 0, alignment: 4, offset: 67
            }, {
                name: 'rgb8_image_height', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 26, rowLength: 0, skipImages: 0,
                skipRows: 0, skipPixels: 0, alignment: 4, offset: 0
            }, {
                name: 'rgb8_row_length', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 27, skipImages: 0,
                skipRows: 0, skipPixels: 0, alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_images', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 0, skipImages: 3,
                skipRows: 0, skipPixels: 0, alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_rows', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 22, rowLength: 0, skipImages: 0,
                skipRows: 3, skipPixels: 0, alignment: 4, offset: 0
            }, {
                name: 'rgb8_skip_pixels', format: gl.RGB8, width: 26,
                height: 25, depth: 10, subX: 1, subY: 2, subZ: 1, subW: 23,
                subH: 19, subD: 8, imageHeight: 0, rowLength: 25, skipImages: 0,
                skipRows: 0, skipPixels: 2, alignment: 4, offset: 0
            }
        ];

        pboGroupParams = new tcuTestCase.DeqpTest(
            'texsubimage3d_pbo', 'glTexSubImage3D() pixel buffer object tests'
        );
        this.addChild(pboGroupParams);

        for (var ndx = 0; ndx < paramCases.length; ndx++) {
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexSubImage2DArrayBufferCase(
                    paramCases[ndx].name + '_2d_array', '',
                    paramCases[ndx].format, paramCases[ndx].width,
                    paramCases[ndx].height, paramCases[ndx].depth,
                    paramCases[ndx].subX, paramCases[ndx].subY,
                    paramCases[ndx].subZ, paramCases[ndx].subW,
                    paramCases[ndx].subH, paramCases[ndx].subD,
                    paramCases[ndx].imageHeight, paramCases[ndx].rowLength,
                    paramCases[ndx].skipImages, paramCases[ndx].skipRows,
                    paramCases[ndx].skipPixels, paramCases[ndx].alignment,
                    paramCases[ndx].offset
                )
            );
            pboGroupParams.addChild(
                new es3fTextureSpecificationTests.TexSubImage3DBufferCase(
                    paramCases[ndx].name + '_3d', '',
                    paramCases[ndx].format, paramCases[ndx].width,
                    paramCases[ndx].height, paramCases[ndx].depth,
                    paramCases[ndx].subX, paramCases[ndx].subY,
                    paramCases[ndx].subZ, paramCases[ndx].subW,
                    paramCases[ndx].subH, paramCases[ndx].subD,
                    paramCases[ndx].imageHeight, paramCases[ndx].rowLength,
                    paramCases[ndx].skipImages, paramCases[ndx].skipRows,
                    paramCases[ndx].skipPixels, paramCases[ndx].alignment,
                    paramCases[ndx].offset
                )
            );
        }

        // glTexSubImage3D() depth cases.
        shadow3dGroup = new tcuTestCase.DeqpTest(
            'texsubimage3d_depth',
            'glTexSubImage3D() with depth or depth/stencil format'
        );
        this.addChild(shadow3dGroup);

        for (var ndx = 0; ndx < depthStencilFormats.length; ndx++) {
            // WebGL 2 specific constraint.
            if (depthStencilFormats[ndx].internalFormat == gl.DEPTH32F_STENCIL8)
                continue;
            tex2DArrayWidth = 57;
            tex2DArrayHeight = 44;
            tex2DArrayLevels = 5;

            shadow3dGroup.addChild(
                new es3fTextureSpecificationTests.TexSubImage2DArrayDepthCase(
                    depthStencilFormats[ndx].name + '_2d_array', '',
                    depthStencilFormats[ndx].internalFormat, tex2DArrayWidth,
                    tex2DArrayHeight, tex2DArrayLevels
                )
            );
        }

        // glTexStorage2D() cases.

        // Color formats
        var splitStorage2D = 3, splitStorageCube = 5;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var colorFormatGroup2D = [];
        for (var ii = 0; ii < splitStorage2D; ++ii) {
            colorFormatGroup2D.push(
                new tcuTestCase.DeqpTest('texstorage2d.format', 'glTexStorage2D() with all formats')
            );
            this.addChild(colorFormatGroup2D[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var colorFormatGroupCube = [];
        for (var ii = 0; ii < splitStorageCube; ++ii) {
            colorFormatGroupCube.push(
                new tcuTestCase.DeqpTest('texstorage2d.format', 'glTexStorage2D() with all formats')
            );
            this.addChild(colorFormatGroupCube[ii]);
        }

        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            /** @type {number} */ var internalFormat = colorFormats[formatNdx].internalFormat;
            var tex2DWidth = 117;
            var tex2DHeight = 97;
            var tex2DLevels = es3fTextureSpecificationTests.maxLevelCount(tex2DWidth, tex2DHeight);
            /** @type {number} */ var cubeSize = 57;
            /** @type {number} */
            var cubeLevels = es3fTextureSpecificationTests.maxLevelCount(
                cubeSize, cubeSize
            );

            colorFormatGroup2D[formatNdx % splitStorage2D].addChild(
                new es3fTextureSpecificationTests.BasicTexStorage2DCase(
                    fmtName + '_2d', '', internalFormat,
                    tex2DWidth, tex2DHeight, tex2DLevels
                )
            );
            colorFormatGroupCube[formatNdx % splitStorageCube].addChild(
                new es3fTextureSpecificationTests.BasicTexStorageCubeCase(
                    fmtName + '_cube', '', internalFormat, cubeSize, cubeLevels
                )
            );
        }

        // Depth / stencil formats.
        /** @type {tcuTestCase.DeqpTest} */
        var storageGroup = new tcuTestCase.DeqpTest(
            'texstorage2d.format',
            'glTexStorage2D() with all formats'
        );
        this.addChild(storageGroup);

        for (var formatNdx = 0; formatNdx < depthStencilFormats.length; formatNdx++) {
            fmtName = depthStencilFormats[formatNdx].name;
            internalFormat = depthStencilFormats[formatNdx].internalFormat;
            // WebGL 2 specific constraint.
            if (internalFormat == gl.DEPTH32F_STENCIL8)
                continue;

            tex2DWidth = 117;
            tex2DHeight = 97;
            tex2DLevels = es3fTextureSpecificationTests.maxLevelCount(
                tex2DWidth, tex2DHeight
            );
            cubeSize = 57;
            cubeLevels = es3fTextureSpecificationTests.maxLevelCount(
                cubeSize, cubeSize
            );

            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorage2DCase(
                    fmtName + '_2d', '', internalFormat,
                    tex2DWidth, tex2DHeight, tex2DLevels
                )
            );
            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorageCubeCase(
                    fmtName + '_cube', '', internalFormat, cubeSize, cubeLevels
                )
            );
        }

        // Sizes.
        storageGroup = new tcuTestCase.DeqpTest(
            'texstorage2d.size',
            'glTexStorage2D() with various sizes'
        );
        this.addChild(storageGroup);

        //    W H L
        /** @type {Array<{width: number, height: number, levels: number}>} */
        var tex2DSizes = [{
                width: 1, height: 1, levels: 1
            }, {
                width: 2, height: 2, levels: 2
            }, {
                width: 64, height: 32, levels: 7
            }, {
                width: 32, height: 64, levels: 4
            }, {
                width: 57, height: 63, levels: 1
            }, {
                width: 57, height: 63, levels: 2
            }, {
                width: 57, height: 63, levels: 6
            }
        ];

        //    S L
        /** @type {Array<{sizes: number, levels: number}>} */
        var cubeSizes = [{
                sizes: 1, levels: 1
            }, {
                sizes: 2, levels: 2
            }, {
                sizes: 57, levels: 1
            }, {
                sizes: 57, levels: 2
            }, {
                sizes: 57, levels: 6
            }, {
                sizes: 64, levels: 4
            }, {
                sizes: 64, levels: 7
            }
        ];

        for (var ndx = 0; ndx < tex2DSizes.length; ndx++) {
            format = gl.RGBA8;
            /** @type {number} */ var width = tex2DSizes[ndx].width;
            /** @type {number} */ var height = tex2DSizes[ndx].height;
            /** @type {number} */ var levels = tex2DSizes[ndx].levels;
            /** @type {string} */
            var name = '2d_' + width + 'x' + height + '_' + levels + '_levels';

            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorage2DCase(
                    name, '', format, width, height, levels
                )
            );
        }

        for (var ndx = 0; ndx < cubeSizes.length; ndx++) {
            format = gl.RGBA8;
            size = cubeSizes[ndx].sizes;
            levels = cubeSizes[ndx].levels;
            name = 'cube_' + size + 'x' + size + '_' + levels + '_levels';

            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorageCubeCase(
                    name, '', format, size, levels
                )
            );
        }

        // glTexStorage3D() cases.

        // Color formats.
        var splitStorage2DArray = 3, splitStorage3D = 4;
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var colorFormatGroup2DArray = [];
        for (var ii = 0; ii < splitStorage2DArray; ++ii) {
            colorFormatGroup2DArray.push(
                new tcuTestCase.DeqpTest('texstorage3d.format', 'glTexStorage3D() with all formats')
            );
            this.addChild(colorFormatGroup2DArray[ii]);
        }
        /** @type {Array<{tcuTestCase.DeqpTest}>} */ var colorFormatGroup3D = [];
        for (var ii = 0; ii < splitStorage3D; ++ii) {
            colorFormatGroup3D.push(
                new tcuTestCase.DeqpTest('texstorage3d.format', 'glTexStorage3D() with all formats')
            );
            this.addChild(colorFormatGroup3D[ii]);
        }

        // Color formats.
        for (var formatNdx = 0; formatNdx < colorFormats.length; formatNdx++) {
            fmtName = colorFormats[formatNdx].name;
            internalFormat = colorFormats[formatNdx].internalFormat;
            tex2DArrayWidth = 57;
            tex2DArrayHeight = 13;
            var tex2DArrayLayers = 7;
            tex2DArrayLevels = es3fTextureSpecificationTests.maxLevelCount(
                tex2DArrayWidth, tex2DArrayHeight
            );
            tex3DWidth = 59;
            tex3DHeight = 37;
            tex3DDepth = 11;
            var tex3DLevels = es3fTextureSpecificationTests.maxLevelCount(
                tex3DWidth, tex3DHeight, tex3DDepth
            );

            colorFormatGroup2DArray[formatNdx % splitStorage2DArray].addChild(
                new es3fTextureSpecificationTests.BasicTexStorage2DArrayCase(
                    fmtName + '_2d_array', '', internalFormat, tex2DArrayWidth,
                    tex2DArrayHeight, tex2DArrayLayers, tex2DArrayLevels
                )
            );
            colorFormatGroup3D[formatNdx % splitStorage3D].addChild(
                new es3fTextureSpecificationTests.BasicTexStorage3DCase(
                    fmtName + '_3d', '', internalFormat, tex3DWidth,
                    tex3DHeight, tex3DDepth, tex3DLevels
                )
            );
        }

        storageGroup = new tcuTestCase.DeqpTest(
            'texstorage3d.format',
            'glTexStorage3D() with all formats'
        );
        this.addChild(storageGroup);

        // Depth/stencil formats (only 2D texture array is supported).
        for (var formatNdx = 0;
            formatNdx < depthStencilFormats.length;
            formatNdx++) {
            fmtName = depthStencilFormats[formatNdx].name;
            internalFormat = depthStencilFormats[formatNdx].internalFormat;
            // WebGL 2 specific constraint.
            if (internalFormat == gl.DEPTH32F_STENCIL8)
                continue;

            tex2DArrayWidth = 57;
            tex2DArrayHeight = 13;
            tex2DArrayLayers = 7;
            tex2DArrayLevels = es3fTextureSpecificationTests.maxLevelCount(
                tex2DArrayWidth, tex2DArrayHeight
            );

            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorage2DArrayCase(
                    fmtName + '_2d_array', '', internalFormat, tex2DArrayWidth,
                    tex2DArrayHeight, tex2DArrayLayers, tex2DArrayLevels
                )
            );
        }

        // Sizes.
        //    W H La Le
        /**
         * @type {Array<{width: number, height: number,
         * layers: number, levels: number}>}
         */
        var tex2DArraySizes = [{
                width: 1, height: 1, layers: 1, levels: 1
            }, {
                width: 2, height: 2, layers: 2, levels: 2
            }, {
                width: 64, height: 32, layers: 3, levels: 7
            }, {
                width: 32, height: 64, layers: 3, levels: 4
            }, {
                width: 57, height: 63, layers: 5, levels: 1
            }, {
                width: 57, height: 63, layers: 5, levels: 2
            }, {
                width: 57, height: 63, layers: 5, levels: 6
            }
        ];

        //    W H D L
        /**
         * @type {Array<{width: number, height: number,
         * depth: number, levels: number}>}
         */
        var tex3DSizes = [{
                width: 1, height: 1, depth: 1, levels: 1
            }, {
                width: 2, height: 2, depth: 2, levels: 2
            }, {
                width: 64, height: 32, depth: 16, levels: 7
            }, {
                width: 32, height: 64, depth: 16, levels: 4
            }, {
                width: 32, height: 16, depth: 64, levels: 4
            }, {
                width: 57, height: 63, depth: 11, levels: 1
            }, {
                width: 57, height: 63, depth: 11, levels: 2
            }, {
                width: 57, height: 63, depth: 11, levels: 6
            }
        ];

        storageGroup = new tcuTestCase.DeqpTest(
            'texstorage3d.size', 'glTexStorage3D() with various sizes'
        );
        this.addChild(storageGroup);

        for (var ndx = 0; ndx < tex2DArraySizes.length; ndx++) {
            format = gl.RGBA8;
            width = tex2DArraySizes[ndx].width;
            height = tex2DArraySizes[ndx].height;
            /** @type {number} */ var layers = tex2DArraySizes[ndx].layers;
            levels = tex2DArraySizes[ndx].levels;
            name = '2d_array_' + width + 'x' + height + 'x' +
                layers + '_' + levels + '_levels';

            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorage2DArrayCase(
                    name, '', format, width, height, layers, levels
                )
            );
        }

        for (var ndx = 0; ndx < tex3DSizes.length; ndx++) {
            format = gl.RGBA8;
            width = tex3DSizes[ndx].width;
            height = tex3DSizes[ndx].height;
            var depth = tex3DSizes[ndx].depth;
            levels = tex3DSizes[ndx].levels;
            name = '3d_' + width + 'x' + height + 'x' +
                depth + '_' + levels + '_levels';

            storageGroup.addChild(
                new es3fTextureSpecificationTests.BasicTexStorage3DCase(
                    name, '', format, width, height, depth, levels
                )
            );
        }
    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    * @param {Array<number>=} range Test range
    */
    es3fTextureSpecificationTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fTextureSpecificationTests.TextureSpecificationTests());
        if (range)
            state.setRange(range);

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fTextureSpecificationTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

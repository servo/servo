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
goog.provide('functional.gles3.es3fShaderTextureFunctionTests');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('modules.shared.glsShaderRenderCase');

goog.scope(function() {
    var es3fShaderTextureFunctionTests = functional.gles3.es3fShaderTextureFunctionTests;
    var deMath = framework.delibs.debase.deMath;
    var tcuMatrix = framework.common.tcuMatrix;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluTexture = framework.opengl.gluTexture;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var glsShaderRenderCase = modules.shared.glsShaderRenderCase;

    /**
     * @enum
     */
    es3fShaderTextureFunctionTests.TexFunction = {
        TEXTURE: 0, //!< texture(), textureOffset()
        TEXTUREPROJ: 1, //!< textureProj(), textureProjOffset()
        TEXTUREPROJ3: 2, //!< textureProj(sampler2D, vec3)
        TEXTURELOD: 3, // ...
        TEXTUREPROJLOD: 4,
        TEXTUREPROJLOD3: 5, //!< textureProjLod(sampler2D, vec3)
        TEXTUREGRAD: 6,
        TEXTUREPROJGRAD: 7,
        TEXTUREPROJGRAD3: 8, //!< textureProjGrad(sampler2D, vec3)
        TEXELFETCH: 9
    };

    /**
     * @param {gluShaderProgram.shaderType} shaderType
     * @param {es3fShaderTextureFunctionTests.TexFunction} function_
     * @return {boolean}
     */
    es3fShaderTextureFunctionTests.functionHasAutoLod = function(shaderType, function_) {
        return shaderType === gluShaderProgram.shaderType.FRAGMENT &&
            (function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTURE ||
            function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ ||
            function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3);
    };

    /**
     * @param {es3fShaderTextureFunctionTests.TexFunction} function_
     * @return {boolean}
     */
    es3fShaderTextureFunctionTests.functionHasProj = function(function_) {
        return function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ ||
           function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3 ||
           function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD ||
           function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD ||
           function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3 ||
           function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3;
    };

    /**
     * @param {es3fShaderTextureFunctionTests.TexFunction} function_
     * @return {boolean}
     */
    es3fShaderTextureFunctionTests.functionHasGrad = function(function_) {
        return function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD ||
            function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD ||
             function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3;
    };

    /**
     * @param {es3fShaderTextureFunctionTests.TexFunction} function_
     * @return {boolean}
     */
    es3fShaderTextureFunctionTests.functionHasLod = function(function_) {
        return function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD ||
               function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD ||
               function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3 ||
               function_ === es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH;
    };

    /**
     * @struct
     * @constructor
     * @param {es3fShaderTextureFunctionTests.TexFunction} func
     * @param {Array<number>} minCoord
     * @param {Array<number>} maxCoord
     * @param {boolean} useBias
     * @param {number} minLodBias
     * @param {number} maxLodBias
     * @param {Array<number>} minDX For *Grad* functions
     * @param {Array<number>} maxDX For *Grad* functions
     * @param {Array<number>} minDY For *Grad* functions
     * @param {Array<number>} maxDY For *Grad* functions
     * @param {boolean} useOffset
     * @param {Array<number>} offset
     */
    es3fShaderTextureFunctionTests.TextureLookupSpec = function(func, minCoord, maxCoord, useBias, minLodBias, maxLodBias, minDX, maxDX, minDY, maxDY, useOffset, offset) {
        /** @type {es3fShaderTextureFunctionTests.TexFunction} */ this.func = func;
        /** @type {Array<number>} */ this.minCoord = minCoord;
        /** @type {Array<number>} */ this.maxCoord = maxCoord;
        // Bias
        /** @type {boolean} */ this.useBias = useBias;
        // Bias or Lod for *Lod* functions
        /** @type {number} */ this.minLodBias = minLodBias;
        /** @type {number} */ this.maxLodBias = maxLodBias;
        // For *Grad* functions
        /** @type {Array<number>} */ this.minDX = minDX;
        /** @type {Array<number>} */ this.maxDX = maxDX;
        /** @type {Array<number>} */ this.minDY = minDY;
        /** @type {Array<number>} */ this.maxDY = maxDY;
        /** @type {boolean} */ this.useOffset = useOffset;
        /** @type {Array<number>} */ this.offset = offset;
    };

    /**
     * @enum
     */
    es3fShaderTextureFunctionTests.TextureType = {
        TEXTURETYPE_2D: 0,
        TEXTURETYPE_CUBE_MAP: 1,
        TEXTURETYPE_2D_ARRAY: 2,
        TEXTURETYPE_3D: 3
    };

    /**
     * @struct
     * @constructor
     * @param {?es3fShaderTextureFunctionTests.TextureType} type
     * @param {number} format
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @param {number} numLevels
     * @param {?tcuTexture.Sampler} sampler
     */
    es3fShaderTextureFunctionTests.TextureSpec = function(type, format, width, height, depth, numLevels, sampler) {
        /** @type {?es3fShaderTextureFunctionTests.TextureType} */ this.type = type; //!< Texture type (2D, cubemap, ...)
        /** @type {number} */ this.format = format; //!< Internal format.
        /** @type {number} */ this.width = width;
        /** @type {number} */ this.height = height;
        /** @type {number} */ this.depth = depth;
        /** @type {number} */ this.numLevels = numLevels;
        /** @type {?tcuTexture.Sampler} */ this.sampler = sampler;
    };

    /**
     * @struct
     * @constructor
     */
    es3fShaderTextureFunctionTests.TexLookupParams = function() {
        /** @type {number} */ this.lod = 0;
        /** @type {Array<number>} */ this.offset = [0, 0, 0];
        /** @type {Array<number>} */ this.scale = [1.0, 1.0, 1.0, 1.0];
        /** @type {Array<number>} */ this.bias = [0.0, 0.0, 0.0, 0.0];
    };

    /**
     * @enum
     */
    es3fShaderTextureFunctionTests.LodMode = {
        EXACT: 0,
        MIN_BOUND: 1,
        MAX_BOUND: 2
    };

    /** @const {es3fShaderTextureFunctionTests.LodMode} */ es3fShaderTextureFunctionTests.DEFAULT_LOD_MODE = es3fShaderTextureFunctionTests.LodMode.EXACT;

    /**
     * @param {number} dudx
     * @param {number} dvdx
     * @param {number} dudy
     * @param {number} dvdy
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromDerivates_UV = function(dudx, dvdx, dudy, dvdy) {
        /** @type {es3fShaderTextureFunctionTests.LodMode} */ var mode = es3fShaderTextureFunctionTests.DEFAULT_LOD_MODE;
        /** @type {number} */ var p;

        switch (mode) {
            case es3fShaderTextureFunctionTests.LodMode.EXACT:
                p = Math.max(Math.sqrt(dudx * dudx + dvdx * dvdx), Math.sqrt(dudy * dudy + dvdy * dvdy));
                break;

            case es3fShaderTextureFunctionTests.LodMode.MIN_BOUND:
            case es3fShaderTextureFunctionTests.LodMode.MAX_BOUND:
                /** @type {number} */ var mu = Math.max(Math.abs(dudx), Math.abs(dudy));
                /** @type {number} */ var mv = Math.max(Math.abs(dvdx), Math.abs(dvdy));

                p = mode === es3fShaderTextureFunctionTests.LodMode.MIN_BOUND ? Math.max(mu, mv) : mu + mv;
                break;

            default:
                throw new Error('LOD_MODE not supported.');
        }

        return Math.log2(p);
    };

    /**
     * @param {number} dudx
     * @param {number} dvdx
     * @param {number} dwdx
     * @param {number} dudy
     * @param {number} dvdy
     * @param {number} dwdy
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromDerivates_UVW = function(dudx, dvdx, dwdx, dudy, dvdy, dwdy) {
        /** @type {es3fShaderTextureFunctionTests.LodMode} */ var mode = es3fShaderTextureFunctionTests.DEFAULT_LOD_MODE;
        /** @type {number} */ var p;

        switch (mode) {
            case es3fShaderTextureFunctionTests.LodMode.EXACT:
                p = Math.max(Math.sqrt(dudx * dudx + dvdx * dvdx + dwdx * dwdx), Math.sqrt(dudy * dudy + dvdy * dvdy + dwdy * dwdy));
                break;

            case es3fShaderTextureFunctionTests.LodMode.MIN_BOUND:
            case es3fShaderTextureFunctionTests.LodMode.MAX_BOUND:
                /** @type {number} */ var mu = Math.max(Math.abs(dudx), Math.abs(dudy));
                /** @type {number} */ var mv = Math.max(Math.abs(dvdx), Math.abs(dvdy));
                /** @type {number} */ var mw = Math.max(Math.abs(dwdx), Math.abs(dwdy));

                p = mode === es3fShaderTextureFunctionTests.LodMode.MIN_BOUND ? Math.max(mu, mv, mw) : (mu + mv + mw);
                break;

            default:
                throw new Error('LOD_MODE not supported.');
        }

        return Math.log2(p);
    };

    /**
     * [dag] Wrapper function for computeLodFromDerivates_UV or computeLodFromDerivates_UVW
     * @param {number} dudx
     * @param {number} dvdx
     * @param {number} dwdxOrdudy
     * @param {number} dudyOrdvdy
     * @param {number=} dvdy
     * @param {number=} dwdy
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromDerivates = function(dudx, dvdx, dwdxOrdudy, dudyOrdvdy, dvdy, dwdy) {
        if (arguments.length === 4)
            return es3fShaderTextureFunctionTests.computeLodFromDerivates_UV(dudx, dvdx, dwdxOrdudy, dudyOrdvdy);
        else
            return es3fShaderTextureFunctionTests.computeLodFromDerivates_UVW(dudx, dvdx, dwdxOrdudy, dudyOrdvdy, /** @type {number} */ (dvdy), /** @type {number} */ (dwdy));
    };

       /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromGrad2D = function(c) {
        /** @type {number} */ var w = c.textures[0].tex2D.getWidth();
        /** @type {number} */ var h = c.textures[0].tex2D.getHeight();
        return es3fShaderTextureFunctionTests.computeLodFromDerivates(c.in_[1][0] * w, c.in_[1][1] * h, c.in_[2][0] * w, c.in_[2][1] * h);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromGrad2DArray = function(c) {
        /** @type {number} */ var w = c.textures[0].tex2DArray.getWidth();
        /** @type {number} */ var h = c.textures[0].tex2DArray.getHeight();
        return es3fShaderTextureFunctionTests.computeLodFromDerivates(c.in_[1][0] * w, c.in_[1][1] * h, c.in_[2][0] * w, c.in_[2][1] * h);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromGrad3D = function(c) {
        /** @type {number} */ var w = c.textures[0].tex3D.getWidth();
        /** @type {number} */ var h = c.textures[0].tex3D.getHeight();
        /** @type {number} */ var d = c.textures[0].tex3D.getDepth();
        return es3fShaderTextureFunctionTests.computeLodFromDerivates(c.in_[1][0] * w, c.in_[1][1] * h, c.in_[1][2] * d, c.in_[2][0] * w, c.in_[2][1] * h, c.in_[2][2] * d);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @return {number}
     */
    es3fShaderTextureFunctionTests.computeLodFromGradCube = function(c) {
        // \note Major axis is always -Z or +Z
        /** @type {number} */ var m = Math.abs(c.in_[0][2]);
        /** @type {number} */ var d = c.textures[0].texCube.getSize();
        /** @type {number} */ var s = d / (2.0 * m);
        /** @type {number} */ var t = d / (2.0 * m);
        return es3fShaderTextureFunctionTests.computeLodFromDerivates(c.in_[1][0] * s, c.in_[1][1] * t, c.in_[2][0] * s, c.in_[2][1] * t);
    };

    /** @typedef {function(glsShaderRenderCase.ShaderEvalContext, es3fShaderTextureFunctionTests.TexLookupParams)} */ es3fShaderTextureFunctionTests.TexEvalFunc;

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} lod
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.texture2D = function(c, s, t, lod) {
        return c.textures[0].tex2D.getView().sample(c.textures[0].sampler, [s, t], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.textureCube = function(c, s, t, r, lod) {
        return c.textures[0].texCube.getView().sample(c.textures[0].sampler, [s, t, r], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.texture2DArray = function(c, s, t, r, lod) {
        return c.textures[0].tex2DArray.getView().sample(c.textures[0].sampler, [s, t, r], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.texture3D = function(c, s, t, r, lod) {
        return c.textures[0].tex3D.getView().sample(c.textures[0].sampler, [s, t, r], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} ref
     * @param {number} s
     * @param {number} t
     * @param {number} lod
     * @return {number}
     */
    es3fShaderTextureFunctionTests.texture2DShadow = function(c, ref, s, t, lod) {
        return c.textures[0].tex2D.getView().sampleCompare(c.textures[0].sampler, ref, [s, t], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} ref
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @return {number}
     */
    es3fShaderTextureFunctionTests.textureCubeShadow = function(c, ref, s, t, r, lod) {
        return c.textures[0].texCube.getView().sampleCompare(c.textures[0].sampler, ref, [s, t, r], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} ref
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @return {number}
     */
    es3fShaderTextureFunctionTests.texture2DArrayShadow = function(c, ref, s, t, r, lod) {
        return c.textures[0].tex2DArray.getView().sampleCompare(c.textures[0].sampler, ref, [s, t, r], lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} lod
     * @param {Array<number>} offset
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.texture2DOffset = function(c, s, t, lod, offset) {
        return c.textures[0].tex2D.getView().sampleOffset(c.textures[0].sampler, [s, t], lod, offset);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @param {Array<number>} offset
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.texture2DArrayOffset = function(c, s, t, r, lod, offset) {
        return c.textures[0].tex2DArray.getView().sampleOffset(c.textures[0].sampler, [s, t, r], lod, offset);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @param {Array<number>} offset
     * @return {Array<number>}
     */
    es3fShaderTextureFunctionTests.texture3DOffset = function(c, s, t, r, lod, offset) {
        return c.textures[0].tex3D.getView().sampleOffset(c.textures[0].sampler, [s, t, r], lod, offset);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} ref
     * @param {number} s
     * @param {number} t
     * @param {number} lod
     * @param {Array<number>} offset
     * @return {number}
     */
    es3fShaderTextureFunctionTests.texture2DShadowOffset = function(c, ref, s, t, lod, offset) {
        return c.textures[0].tex2D.getView().sampleCompareOffset(c.textures[0].sampler, ref, [s, t], lod, offset);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {number} ref
     * @param {number} s
     * @param {number} t
     * @param {number} r
     * @param {number} lod
     * @param {Array<number>} offset
     * @return {number}
     */
    es3fShaderTextureFunctionTests.texture2DArrayShadowOffset = function(c, ref, s, t, r, lod, offset) {
        return c.textures[0].tex2DArray.getView().sampleCompareOffset(c.textures[0].sampler, ref, [s, t, r], lod, offset);
    };

    // Eval functions.
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2D = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0], c.in_[0][1], p.lod), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCube = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.textureCube(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArray = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArray(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3D = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0], c.in_[0][1], p.lod+c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCubeBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.textureCube(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod+c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArray(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod+c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod+c.in_[1][0]), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProj3 = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], p.lod), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProj3Bias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], p.lod+c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProj = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod+c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProj = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], p.lod), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], p.lod+c.in_[1][0]), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DLod = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0], c.in_[0][1], c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCubeLod = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.textureCube(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayLod = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArray(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DLod = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], c.in_[1][0]), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjLod3 = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjLod = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[1][0]), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjLod = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], c.in_[1][0]), p.scale), p.bias);
    };

    // Offset variants

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0], c.in_[0][1], p.lod, deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArrayOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod, deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod, p.offset), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DOffsetBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0], c.in_[0][1], p.lod+c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayOffsetBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArrayOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod+c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DOffsetBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod+c.in_[1][0], p.offset), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DLodOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0], c.in_[0][1], c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayLodOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArrayOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DLodOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], c.in_[1][0], p.offset), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProj3Offset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], p.lod, deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProj3OffsetBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], p.lod+c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod, deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjOffsetBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod+c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], p.lod, p.offset), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjOffsetBias = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], p.lod+c.in_[1][0], p.offset), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjLod3Offset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjLodOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[1][0], deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjLodOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], c.in_[1][0], p.offset), p.scale), p.bias);
    };

    // Shadow variants

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadow = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], p.lod);
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowBias = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], p.lod+c.in_[1][0]);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCubeShadow = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.textureCubeShadow(c, c.in_[0][3], c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod);
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCubeShadowBias = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.textureCubeShadow(c, c.in_[0][3], c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod+c.in_[1][0]);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayShadow = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DArrayShadow(c, c.in_[0][3], c.in_[0][0], c.in_[0][1], c.in_[0][2], p.lod);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowLod = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], c.in_[1][0]);
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowLodOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], c.in_[1][0], deMath.swizzle(p.offset, [0,1]));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProj = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod);
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjBias = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod+c.in_[1][0]);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjLod = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[1][0]);
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjLodOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[1][0], deMath.swizzle(p.offset, [0,1]));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], p.lod, deMath.swizzle(p.offset, [0,1]));
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowOffsetBias = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], p.lod+c.in_[1][0], deMath.swizzle(p.offset, [0,1]));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod, deMath.swizzle(p.offset, [0,1]));
    };
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjOffsetBias = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], p.lod+c.in_[1][0], deMath.swizzle(p.offset, [0,1]));
    };

    // Gradient variarts

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DGrad = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0], c.in_[0][1], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c)), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCubeGrad = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.textureCube(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGradCube(c)), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayGrad = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArray(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad2DArray(c)), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DGrad = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad3D(c)), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowGrad = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTextureCubeShadowGrad = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.textureCubeShadow(c, c.in_[0][3], c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGradCube(c));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayShadowGrad = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DArrayShadow(c, c.in_[0][3], c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad2DArray(c));
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DGradOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0], c.in_[0][1], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c), deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayGradOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DArrayOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad2DArray(c), deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DGradOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad3D(c), p.offset), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowGradOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2], c.in_[0][0], c.in_[0][1], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c), deMath.swizzle(p.offset, [0,1]));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DArrayShadowGradOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DArrayShadowOffset(c, c.in_[0][3], c.in_[0][0], c.in_[0][1], c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad2DArray(c), deMath.swizzle(p.offset, [0,1]));
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjGrad = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadow(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c));
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DShadowProjGradOffset = function(c, p) {
        c.color[0] = es3fShaderTextureFunctionTests.texture2DShadowOffset(c, c.in_[0][2]/c.in_[0][3], c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c), deMath.swizzle(p.offset, [0,1]));
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjGrad3 = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c)), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjGrad = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c)), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjGrad = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3D(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], es3fShaderTextureFunctionTests.computeLodFromGrad3D(c)), p.scale), p.bias);
    };


    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjGrad3Offset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][2], c.in_[0][1]/c.in_[0][2], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c), deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture2DProjGradOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture2DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], es3fShaderTextureFunctionTests.computeLodFromGrad2D(c), deMath.swizzle(p.offset, [0,1])), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset = function(c, p) {
        c.color = deMath.add(deMath.multiply(es3fShaderTextureFunctionTests.texture3DOffset(c, c.in_[0][0]/c.in_[0][3], c.in_[0][1]/c.in_[0][3], c.in_[0][2]/c.in_[0][3], es3fShaderTextureFunctionTests.computeLodFromGrad3D(c), p.offset), p.scale), p.bias);
    };

    // Texel fetch variants
    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexelFetch2D = function(c, p) {
        /** @type {number} */ var x    = Math.trunc(c.in_[0][0]) + p.offset[0];
        /** @type {number} */ var y    = Math.trunc(c.in_[0][1]) + p.offset[1];
        /** @type {number} */ var lod = Math.trunc(c.in_[1][0]);
        c.color = deMath.add(deMath.multiply(c.textures[0].tex2D.getLevel(lod).getPixel(x, y), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexelFetch2DArray = function(c, p) {
        /** @type {number} */ var x    = Math.trunc(c.in_[0][0]) + p.offset[0];
        /** @type {number} */ var y    = Math.trunc(c.in_[0][1]) + p.offset[1];
        /** @type {number} */ var l    = Math.trunc(c.in_[0][2]);
        /** @type {number} */ var lod = Math.trunc(c.in_[1][0]);
        c.color = deMath.add(deMath.multiply(c.textures[0].tex2DArray.getLevel(lod).getPixel(x, y, l), p.scale), p.bias);
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} c
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} p
     */
    es3fShaderTextureFunctionTests.evalTexelFetch3D = function(c, p) {
        /** @type {number} */ var x    = Math.trunc(c.in_[0][0]) + p.offset[0];
        /** @type {number} */ var y    = Math.trunc(c.in_[0][1]) + p.offset[1];
        /** @type {number} */ var z    = Math.trunc(c.in_[0][2]) + p.offset[2];
        /** @type {number} */ var lod = Math.trunc(c.in_[1][0]);
        c.color = deMath.add(deMath.multiply(c.textures[0].tex3D.getLevel(lod).getPixel(x, y, z), p.scale), p.bias);
    };

    /**
     * @constructor
     * @extends {glsShaderRenderCase.ShaderEvaluator}
     * @param {es3fShaderTextureFunctionTests.TexEvalFunc} evalFunc
     * @param {es3fShaderTextureFunctionTests.TexLookupParams} lookupParams
     */
    es3fShaderTextureFunctionTests.TexLookupEvaluator = function(evalFunc, lookupParams) {
        /** @type {es3fShaderTextureFunctionTests.TexEvalFunc} */ this.m_evalFunc = evalFunc;
        /** @type {es3fShaderTextureFunctionTests.TexLookupParams} */ this.m_lookupParams = lookupParams;
    };

    es3fShaderTextureFunctionTests.TexLookupEvaluator.prototype = Object.create(glsShaderRenderCase.ShaderEvaluator.prototype);
    es3fShaderTextureFunctionTests.TexLookupEvaluator.prototype.constructor = es3fShaderTextureFunctionTests.TexLookupEvaluator;

    /**
     * @param  {glsShaderRenderCase.ShaderEvalContext} ctx
     */
    es3fShaderTextureFunctionTests.TexLookupEvaluator.prototype.evaluate = function(ctx) {
        this.m_evalFunc(ctx, this.m_lookupParams);
    };

    /**
     * @constructor
     * @extends {glsShaderRenderCase.ShaderRenderCase}
     * @param {string} name
     * @param {string} desc
     * @param {es3fShaderTextureFunctionTests.TextureLookupSpec} lookup
     * @param {es3fShaderTextureFunctionTests.TextureSpec} texture
     * @param {es3fShaderTextureFunctionTests.TexEvalFunc} evalFunc
     * @param {boolean} isVertexCase
     */
    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase = function(name, desc, lookup, texture, evalFunc, isVertexCase) {
        glsShaderRenderCase.ShaderRenderCase.call(this, name, desc, isVertexCase);

        /** @type {es3fShaderTextureFunctionTests.TextureLookupSpec} */ this.m_lookupSpec = lookup;
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ this.m_textureSpec = texture;
        /** @type {es3fShaderTextureFunctionTests.TexLookupParams} */ this.m_lookupParams = new es3fShaderTextureFunctionTests.TexLookupParams();
        /** @type {es3fShaderTextureFunctionTests.TexLookupEvaluator} */ this.m_evaluator = new es3fShaderTextureFunctionTests.TexLookupEvaluator(evalFunc, this.m_lookupParams);

        /** @type {gluTexture.Texture2D} */ this.m_texture2D = null;
        /** @type {gluTexture.TextureCube} */ this.m_textureCube = null;
        /** @type {gluTexture.Texture2DArray} */ this.m_texture2DArray = null;
        /** @type {gluTexture.Texture3D} */ this.m_texture3D = null;
    };

    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype = Object.create(glsShaderRenderCase.ShaderRenderCase.prototype);
    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype.constructor = es3fShaderTextureFunctionTests.ShaderTextureFunctionCase;

    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype.init = function() {

        // Base coord scale & bias
        /** @type {(Array<number>|number)} */ var s = deMath.subtract(this.m_lookupSpec.maxCoord, this.m_lookupSpec.minCoord);
        /** @type {(Array<number>|number)} */ var b = this.m_lookupSpec.minCoord;

        /** @type {Array<number>} */ var baseCoordTrans = [
            s[0], 0.0, 0.0, b[0],
            0.0, s[1], 0., b[1],
            s[2]/2.0, -s[2]/2.0, 0.0, s[2]/2.0 + b[2],
            -s[3]/2.0, s[3]/2.0, 0.0, s[3]/2.0 + b[3]
        ];
        this.m_userAttribTransforms.push(tcuMatrix.matrixFromArray(4, 4, baseCoordTrans));

        /** @type {boolean} */ var hasLodBias = es3fShaderTextureFunctionTests.functionHasLod(this.m_lookupSpec.func) || this.m_lookupSpec.useBias;
        /** @type {boolean} */ var isGrad = es3fShaderTextureFunctionTests.functionHasGrad(this.m_lookupSpec.func);
        assertMsgOptions(!isGrad || !hasLodBias, 'Assert Error. expected: isGrad || hasLodBias === false', false, true);

        if (hasLodBias) {
            s = this.m_lookupSpec.maxLodBias - this.m_lookupSpec.minLodBias;
            b = this.m_lookupSpec.minLodBias;
            /** @type {Array<number>} */ var lodCoordTrans = [
                s/2.0, s/2.0, 0.0, b,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0
            ];

            this.m_userAttribTransforms.push(tcuMatrix.matrixFromArray(4, 4, lodCoordTrans));
        }
        else if (isGrad) {
            /** @type {Array<number>} */ var sx = deMath.subtract(this.m_lookupSpec.maxDX, this.m_lookupSpec.minDX);
            /** @type {Array<number>} */ var sy = deMath.subtract(this.m_lookupSpec.maxDY, this.m_lookupSpec.minDY);
            /** @type {Array<number>} */ var gradDxTrans = [
                sx[0]/2.0, sx[0]/2.0, 0.0, this.m_lookupSpec.minDX[0],
                sx[1]/2.0, sx[1]/2.0, 0.0, this.m_lookupSpec.minDX[1],
                sx[2]/2.0, sx[2]/2.0, 0.0, this.m_lookupSpec.minDX[2],
                0.0, 0.0, 0.0, 0.0
            ];
            /** @type {Array<number>} */ var gradDyTrans = [
                -sy[0]/2.0, -sy[0]/2.0, 0.0, this.m_lookupSpec.maxDY[0],
                -sy[1]/2.0, -sy[1]/2.0, 0.0, this.m_lookupSpec.maxDY[1],
                -sy[2]/2.0, -sy[2]/2.0, 0.0, this.m_lookupSpec.maxDY[2],
                0.0, 0.0, 0.0, 0.0
             ];

            this.m_userAttribTransforms.push(tcuMatrix.matrixFromArray(4, 4, gradDxTrans));
            this.m_userAttribTransforms.push(tcuMatrix.matrixFromArray(4, 4, gradDyTrans));
        }

        this.initShaderSources();
        this.initTexture();

        this.postinit();

    };

    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype.initTexture = function() {
        /** @type {Array<Array<number>>} */ var texCubeSwz = [
            [0, 0, 1, 1],
            [1, 1, 0, 0],
            [0, 1, 0, 1],
            [1, 0, 1, 0],
            [0, 1, 1, 0],
            [1, 0, 0, 1]
        ];

        assertMsgOptions(texCubeSwz.length === 6, 'Cube should have 6 faces.', false, true);

        /** @type {number} */ var levelStep;
        /** @type {Array<number>} */ var cScale;
        /** @type {Array<number>} */ var cBias;
        /** @type {number} */ var baseCellSize;

        /** @type {number} */ var fA;
        /** @type {number} */ var fB;
        /** @type {Array<number>} */ var colorA;
        /** @type {Array<number>} */ var colorB;

        /** @type {number} */ var dudx;
        /** @type {number} */ var dvdy;

        /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_textureSpec.format);
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        /** @type {Array<number>} */ var viewportSize = this.getViewportSize();
        /** @type {boolean} */ var isProj = es3fShaderTextureFunctionTests.functionHasProj(this.m_lookupSpec.func);
        /** @type {boolean} */ var isAutoLod = es3fShaderTextureFunctionTests.functionHasAutoLod(
            this.m_isVertexCase ? gluShaderProgram.shaderType.VERTEX : gluShaderProgram.shaderType.FRAGMENT,
            this.m_lookupSpec.func); // LOD can vary significantly
        /** @type {number} */ var proj = isProj ?
            1.0 / this.m_lookupSpec.minCoord[this.m_lookupSpec.func === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3 ? 2 : 3] :
            1.0;

        switch (this.m_textureSpec.type) {
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D:
                levelStep = isAutoLod ? 0.0 : 1.0 / Math.max(1, this.m_textureSpec.numLevels - 1);
                cScale = deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin);
                cBias = fmtInfo.valueMin;
                baseCellSize = Math.min(this.m_textureSpec.width / 4, this.m_textureSpec.height / 4);

                this.m_texture2D = gluTexture.texture2DFromInternalFormat(gl, this.m_textureSpec.format, this.m_textureSpec.width, this.m_textureSpec.height);
                for (var level = 0; level < this.m_textureSpec.numLevels; level++) {
                    fA = level * levelStep;
                    fB = 1.0 - fA;
                    colorA = deMath.add(cBias, deMath.multiply(cScale, [fA, fB, fA, fB]));
                    colorB = deMath.add(cBias, deMath.multiply(cScale, [fB, fA, fB, fA]));

                    this.m_texture2D.getRefTexture().allocLevel(level);
                    tcuTextureUtil.fillWithGrid(this.m_texture2D.getRefTexture().getLevel(level), Math.max(1, baseCellSize >> level), colorA, colorB);
                }
                this.m_texture2D.upload();

                // Compute LOD.
                dudx = (this.m_lookupSpec.maxCoord[0] - this.m_lookupSpec.minCoord[0]) * proj * this.m_textureSpec.width / viewportSize[0];
                dvdy = (this.m_lookupSpec.maxCoord[1] - this.m_lookupSpec.minCoord[1]) * proj * this.m_textureSpec.height / viewportSize[1];
                this.m_lookupParams.lod = es3fShaderTextureFunctionTests.computeLodFromDerivates(dudx, 0.0, 0.0, dvdy);

                // Append to texture list.
                this.m_textures.push(new glsShaderRenderCase.TextureBinding(this.m_texture2D, this.m_textureSpec.sampler));
                break;

            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP:
                levelStep = isAutoLod ? 0.0 : 1.0 / Math.max(1, this.m_textureSpec.numLevels - 1);
                cScale = deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin);
                cBias = fmtInfo.valueMin;
                /** @type {Array<number>} */ var cCorner = deMath.add(cBias, deMath.scale(cScale, 0.5));
                baseCellSize = Math.min(this.m_textureSpec.width / 4, this.m_textureSpec.height / 4);

                assertMsgOptions(this.m_textureSpec.width === this.m_textureSpec.height, 'Expected width === height', false, true);
                this.m_textureCube = gluTexture.cubeFromInternalFormat(gl, this.m_textureSpec.format, this.m_textureSpec.width);
                for (var level = 0; level < this.m_textureSpec.numLevels; level++) {
                    fA = level * levelStep;
                    fB = 1.0 - fA;
                    /** @type {Array<number>} */ var f = [fA, fB];

                    for (var face = 0; face < 6; face++) {
                        /** @type {Array<number>} */ var swzA = texCubeSwz[face];
                        /** @type {Array<number>} */ var swzB = deMath.subtract([1, 1, 1, 1], swzA);
                        colorA = deMath.add(cBias, deMath.multiply(cScale, deMath.swizzle(f, [swzA[0], swzA[1], swzA[2], swzA[3]])));
                        colorB = deMath.add(cBias, deMath.multiply(cScale, deMath.swizzle(f, [swzB[0], swzB[1], swzB[2], swzB[3]])));

                        this.m_textureCube.getRefTexture().allocLevel(face, level);

                        /** @type {tcuTexture.PixelBufferAccess} */ var access = this.m_textureCube.getRefTexture().getLevelFace(level, face);
                        /** @type {number} */ var lastPix = access.getWidth() - 1;

                        tcuTextureUtil.fillWithGrid(access, Math.max(1, baseCellSize >> level), colorA, colorB);

                        // Ensure all corners have identical colors in order to avoid dealing with ambiguous corner texel filtering
                        access.setPixel(cCorner, 0, 0);
                        access.setPixel(cCorner, 0, lastPix);
                        access.setPixel(cCorner, lastPix, 0);
                        access.setPixel(cCorner, lastPix, lastPix);
                    }
                }
                this.m_textureCube.upload();

                // Compute LOD \note Assumes that only single side is accessed and R is constant major axis.
                assertMsgOptions(Math.abs(this.m_lookupSpec.minCoord[2] - this.m_lookupSpec.maxCoord[2]) < 0.005, 'Expected abs(minCoord-maxCoord) < 0.005', false, true);
                assertMsgOptions(Math.abs(this.m_lookupSpec.minCoord[0]) < Math.abs(this.m_lookupSpec.minCoord[2]) && Math.abs(this.m_lookupSpec.maxCoord[0]) < Math.abs(this.m_lookupSpec.minCoord[2]), 'Assert error: minCoord, maxCoord', false, true);
                assertMsgOptions(Math.abs(this.m_lookupSpec.minCoord[1]) < Math.abs(this.m_lookupSpec.minCoord[2]) && Math.abs(this.m_lookupSpec.maxCoord[1]) < Math.abs(this.m_lookupSpec.minCoord[2]), 'Assert error: minCoord, maxCoord', false, true);

                /** @type {tcuTexture.CubeFaceCoords} */ var c00 = tcuTexture.getCubeFaceCoords([this.m_lookupSpec.minCoord[0] * proj, this.m_lookupSpec.minCoord[1] * proj, this.m_lookupSpec.minCoord[2] * proj]);
                /** @type {tcuTexture.CubeFaceCoords} */ var c10 = tcuTexture.getCubeFaceCoords([this.m_lookupSpec.maxCoord[0] * proj, this.m_lookupSpec.minCoord[1] * proj, this.m_lookupSpec.minCoord[2] * proj]);
                /** @type {tcuTexture.CubeFaceCoords} */ var c01 = tcuTexture.getCubeFaceCoords([this.m_lookupSpec.minCoord[0] * proj, this.m_lookupSpec.maxCoord[1] * proj, this.m_lookupSpec.minCoord[2] * proj]);
                dudx = (c10.s - c00.s) * this.m_textureSpec.width / viewportSize[0];
                dvdy = (c01.t - c00.t) * this.m_textureSpec.height / viewportSize[1];

                this.m_lookupParams.lod = es3fShaderTextureFunctionTests.computeLodFromDerivates(dudx, 0.0, 0.0, dvdy);

                this.m_textures.push(new glsShaderRenderCase.TextureBinding(this.m_textureCube, this.m_textureSpec.sampler));
                break;

            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY:
                /** @type {number} */ var layerStep = 1.0 / this.m_textureSpec.depth;
                levelStep = isAutoLod ? 0.0 : 1.0 / (Math.max(1, this.m_textureSpec.numLevels - 1) * this.m_textureSpec.depth);
                cScale = deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin);
                cBias = fmtInfo.valueMin;
                baseCellSize = Math.min(this.m_textureSpec.width / 4, this.m_textureSpec.height / 4);

                this.m_texture2DArray = gluTexture.texture2DArrayFromInternalFormat(gl,
                    this.m_textureSpec.format,
                    this.m_textureSpec.width,
                    this.m_textureSpec.height,
                    this.m_textureSpec.depth);

                for (var level = 0; level < this.m_textureSpec.numLevels; level++) {
                    this.m_texture2DArray.getRefTexture().allocLevel(level);
                    /** @type {tcuTexture.PixelBufferAccess} */ var levelAccess = this.m_texture2DArray.getRefTexture().getLevel(level);

                    for (var layer = 0; layer < levelAccess.getDepth(); layer++) {
                        fA = layer * layerStep + level * levelStep;
                        fB = 1.0 - fA;
                        colorA = deMath.add(cBias, deMath.multiply(cScale, [fA, fB, fA, fB]));
                        colorB = deMath.add(cBias, deMath.multiply(cScale, [fB, fA, fB, fA]));

                        tcuTextureUtil.fillWithGrid(tcuTextureUtil.getSubregion(levelAccess, 0, 0, layer, levelAccess.getWidth(), levelAccess.getHeight(), 1), Math.max(1, baseCellSize >> level), colorA, colorB);
                    }
                }
                this.m_texture2DArray.upload();

                // Compute LOD.
                dudx = (this.m_lookupSpec.maxCoord[0] - this.m_lookupSpec.minCoord[0]) * proj * this.m_textureSpec.width / viewportSize[0];
                dvdy = (this.m_lookupSpec.maxCoord[1] - this.m_lookupSpec.minCoord[1]) * proj * this.m_textureSpec.height / viewportSize[1];
                this.m_lookupParams.lod = es3fShaderTextureFunctionTests.computeLodFromDerivates(dudx, 0.0, 0.0, dvdy);

                // Append to texture list.
                this.m_textures.push(new glsShaderRenderCase.TextureBinding(this.m_texture2DArray, this.m_textureSpec.sampler));
                break;

            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D:
                levelStep = isAutoLod ? 0.0 : 1.0 / Math.max(1, this.m_textureSpec.numLevels - 1);
                cScale = deMath.subtract(fmtInfo.valueMax, fmtInfo.valueMin);
                cBias = fmtInfo.valueMin;
                baseCellSize = Math.min(this.m_textureSpec.width / 2, this.m_textureSpec.height / 2, this.m_textureSpec.depth / 2);

                this.m_texture3D = gluTexture.texture3DFromInternalFormat(gl, this.m_textureSpec.format, this.m_textureSpec.width, this.m_textureSpec.height, this.m_textureSpec.depth);
                for (var level = 0; level < this.m_textureSpec.numLevels; level++) {
                    fA = level * levelStep;
                    fB = 1.0 - fA;
                    colorA = deMath.add(cBias, deMath.multiply(cScale, [fA, fB, fA, fB]));
                    colorB = deMath.add(cBias, deMath.multiply(cScale, [fB, fA, fB, fA]));

                    this.m_texture3D.getRefTexture().allocLevel(level);
                    tcuTextureUtil.fillWithGrid(this.m_texture3D.getRefTexture().getLevel(level), Math.max(1, baseCellSize >> level), colorA, colorB);
                }
                this.m_texture3D.upload();

                // Compute LOD.
                dudx = (this.m_lookupSpec.maxCoord[0] - this.m_lookupSpec.minCoord[0]) * proj * this.m_textureSpec.width / viewportSize[0];
                dvdy = (this.m_lookupSpec.maxCoord[1] - this.m_lookupSpec.minCoord[1]) * proj * this.m_textureSpec.height / viewportSize[1];
                /** @type {number} */ var dwdx = (this.m_lookupSpec.maxCoord[2] - this.m_lookupSpec.minCoord[2]) * 0.5 * proj * this.m_textureSpec.depth / viewportSize[0];
                /** @type {number} */ var dwdy = (this.m_lookupSpec.maxCoord[2] - this.m_lookupSpec.minCoord[2]) * 0.5 * proj * this.m_textureSpec.depth / viewportSize[1];
                this.m_lookupParams.lod = es3fShaderTextureFunctionTests.computeLodFromDerivates(dudx, 0.0, dwdx, 0.0, dvdy, dwdy);

                // Append to texture list.
                this.m_textures.push(new glsShaderRenderCase.TextureBinding(this.m_texture3D, this.m_textureSpec.sampler));
                break;

            default:
                throw new Error('Texture type not supported.');
        }

        // Set lookup scale & bias
        this.m_lookupParams.scale = fmtInfo.lookupScale;
        this.m_lookupParams.bias = fmtInfo.lookupBias;
        this.m_lookupParams.offset = this.m_lookupSpec.offset;
    };

    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype.initShaderSources = function() {
        /** @type {es3fShaderTextureFunctionTests.TexFunction} */ var function_ = this.m_lookupSpec.func;
        /** @type {boolean} */ var isVtxCase = this.m_isVertexCase;
        /** @type {boolean} */ var isProj = es3fShaderTextureFunctionTests.functionHasProj(function_);
        /** @type {boolean} */ var isGrad = es3fShaderTextureFunctionTests.functionHasGrad(function_);
        /** @type {boolean} */ var isShadow = this.m_textureSpec.sampler.compare !== tcuTexture.CompareMode.COMPAREMODE_NONE;
        /** @type {boolean} */ var is2DProj4 = !isShadow && this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D && (function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ || function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD || function_ === es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD);
        /** @type {boolean} */ var isIntCoord = function_ === es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH;
        /** @type {boolean} */ var hasLodBias = es3fShaderTextureFunctionTests.functionHasLod(this.m_lookupSpec.func) || this.m_lookupSpec.useBias;
        /** @type {number} */ var texCoordComps = this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D ? 2 : 3;
        /** @type {number} */ var extraCoordComps = (isProj ? (is2DProj4 ? 2 : 1) : 0) + (isShadow ? 1 : 0);
        /** @type {gluShaderUtil.DataType} */ var coordType = gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.FLOAT, texCoordComps+extraCoordComps);
        /** @type {gluShaderUtil.precision} */ var coordPrec = gluShaderUtil.precision.PRECISION_HIGHP;
        /** @type {string} */ var coordTypeName = gluShaderUtil.getDataTypeName(coordType);
        /** @type {string} */ var coordPrecName = gluShaderUtil.getPrecisionName(coordPrec);
        /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_textureSpec.format);
        /** @type {?gluShaderUtil.DataType} */ var samplerType = null;
        /** @type {gluShaderUtil.DataType} */ var gradType = (this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP || this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D) ? gluShaderUtil.DataType.FLOAT_VEC3 : gluShaderUtil.DataType.FLOAT_VEC2;
        /** @type {string} */ var gradTypeName = gluShaderUtil.getDataTypeName(gradType);
        /** @type {string} */ var baseFuncName = '';

        assertMsgOptions(!isGrad || !hasLodBias, 'Expected !isGrad || !hasLodBias', false, true);

        switch (this.m_textureSpec.type) {
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D:
                samplerType = isShadow ? gluShaderUtil.DataType.SAMPLER_2D_SHADOW : gluTextureUtil.getSampler2DType(texFmt);
                break;
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP:
                samplerType = isShadow ? gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW : gluTextureUtil.getSamplerCubeType(texFmt);
                break;
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY:
                samplerType = isShadow ? gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW : gluTextureUtil.getSampler2DArrayType(texFmt);
                break;
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D:
                assertMsgOptions(!isShadow, 'Expected !isShadow', false, true);
                samplerType = gluTextureUtil.getSampler3DType(texFmt);
                break;
            default:
                throw new Error('Unexpected type.');
        }

        switch (this.m_lookupSpec.func) {
            case es3fShaderTextureFunctionTests.TexFunction.TEXTURE: baseFuncName = 'texture'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ: baseFuncName = 'textureProj'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3: baseFuncName = 'textureProj'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD: baseFuncName = 'textureLod'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD: baseFuncName = 'textureProjLod'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3: baseFuncName = 'textureProjLod'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD: baseFuncName = 'textureGrad'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD: baseFuncName = 'textureProjGrad'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3: baseFuncName = 'textureProjGrad'; break;
            case es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH: baseFuncName = 'texelFetch'; break;
            default:
                throw new Error('Unexpected function.');
        }

        /** @type {string} */ var vert = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        vert += '#version 300 es\n' +
                'in highp vec4 a_position;\n' +
                'in ' + coordPrecName + ' ' + coordTypeName + ' a_in0;\n';

        if (isGrad) {
            vert += 'in ' + coordPrecName + ' ' + gradTypeName + ' a_in1;\n';
            vert += 'in ' + coordPrecName + ' ' + gradTypeName + ' a_in2;\n';
        }
        else if (hasLodBias)
            vert += 'in ' + coordPrecName + ' float a_in1;\n';

        frag += '#version 300 es\n' +
                'layout(location = 0) out mediump vec4 o_color;\n';

        if (isVtxCase) {
            vert += 'out mediump vec4 v_color;\n';
            frag += 'in mediump vec4 v_color;\n';
        }
        else
        {
            vert += 'out ' + coordPrecName + ' ' + coordTypeName + ' v_texCoord;\n';
            frag += 'in ' + coordPrecName + ' ' + coordTypeName + ' v_texCoord;\n';

            if (isGrad) {
                vert += 'out ' + coordPrecName + ' ' + gradTypeName + ' v_gradX;\n';
                vert += 'out ' + coordPrecName + ' ' + gradTypeName + ' v_gradY;\n';
                frag += 'in ' + coordPrecName + ' ' + gradTypeName + ' v_gradX;\n';
                frag += 'in ' + coordPrecName + ' ' + gradTypeName + ' v_gradY;\n';
            }

            if (hasLodBias) {
                vert += 'out ' + coordPrecName + ' float v_lodBias;\n';
                frag += 'in ' + coordPrecName + ' float v_lodBias;\n';
            }
        }

        // Uniforms
        op += 'uniform highp ' + gluShaderUtil.getDataTypeName(samplerType) + ' u_sampler;\n' +
              'uniform highp vec4 u_scale;\n' +
              'uniform highp vec4 u_bias;\n';

        vert += isVtxCase ? op : '';
        frag += isVtxCase ? '' : op;
        op = '';

        vert += '\nvoid main()\n{\n' +
                '\tgl_Position = a_position;\n';
        frag += '\nvoid main()\n{\n';

        if (isVtxCase)
            vert += '\tv_color = ';
        else
            frag += '\to_color = ';

        // Op.
        /** @type {string} */ var texCoord = isVtxCase ? 'a_in0' : 'v_texCoord';
        /** @type {string} */ var gradX = isVtxCase ? 'a_in1' : 'v_gradX';
        /** @type {string} */ var gradY = isVtxCase ? 'a_in2' : 'v_gradY';
        /** @type {string} */ var lodBias = isVtxCase ? 'a_in1' : 'v_lodBias';

        op += 'vec4(' + baseFuncName;
        if (this.m_lookupSpec.useOffset)
            op += 'Offset';
        op += '(u_sampler, ';

        if (isIntCoord)
            op += 'ivec' + (texCoordComps+extraCoordComps) + '(';

        op += texCoord;

        if (isIntCoord)
            op += ')';

        if (isGrad)
            op += ', ' + gradX + ', ' + gradY;

        if (es3fShaderTextureFunctionTests.functionHasLod(function_)) {
            if (isIntCoord)
                op += ', int(' + lodBias + ')';
            else
                op += ', ' + lodBias;
        }

        if (this.m_lookupSpec.useOffset) {
            /** @type {number} */ var offsetComps = this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D ? 3 : 2;

            op += ', ivec' + offsetComps + '(';
            for (var ndx = 0; ndx < offsetComps; ndx++) {
                if (ndx !== 0)
                    op += ', ';
                op += this.m_lookupSpec.offset[ndx];
            }
            op += ')';
        }

        if (this.m_lookupSpec.useBias)
            op += ', ' + lodBias;

        op += ')';

        if (isShadow)
            op += ', 0.0, 0.0, 1.0)';
        else
            op += ')*u_scale + u_bias';

        op += ';\n';

        vert += isVtxCase ? op : '';
        frag += isVtxCase ? '' : op;
        op = '';

        if (isVtxCase)
            frag += '\to_color = v_color;\n';
        else {
            vert += '\tv_texCoord = a_in0;\n';

            if (isGrad) {
                vert += '\tv_gradX = a_in1;\n';
                vert += '\tv_gradY = a_in2;\n';
            }
            else if (hasLodBias)
                vert += '\tv_lodBias = a_in1;\n';
        }

        vert += '}\n';
        frag += '}\n';

        this.m_vertShaderSource = vert;
        this.m_fragShaderSource = frag;
    };

    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype.deinit = function() {
        this.m_program = null;
        this.m_texture2D = null;
        this.m_textureCube = null;
        this.m_texture2DArray = null;
        this.m_texture3D = null;
    };

    /**
     * @param  {WebGLProgram} programID
     * @param  {Array<number>} constCoords
     */
    es3fShaderTextureFunctionTests.ShaderTextureFunctionCase.prototype.setupUniforms = function(programID, constCoords) {
        gl.uniform1i(gl.getUniformLocation(programID, 'u_sampler'), 0);
        gl.uniform4fv(gl.getUniformLocation(programID, 'u_scale'), this.m_lookupParams.scale);
        gl.uniform4fv(gl.getUniformLocation(programID, 'u_bias'), this.m_lookupParams.bias);
    };


    /**
     * @struct
     * @constructor
     * @param {Array<number>} textureSize
     * @param {number} lod
     * @param {number} lodBase
     * @param {Array<number>} expectedSize
     */
    es3fShaderTextureFunctionTests.TestSize = function(textureSize, lod, lodBase, expectedSize) {
        /** @type {Array<number>} */ this.textureSize = textureSize;
        /** @type {number} */ this.lod = lod;
        /** @type {number} */ this.lodBase = lodBase;
        /** @type {Array<number>} */ this.expectedSize = expectedSize;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {string} samplerType
     * @param {es3fShaderTextureFunctionTests.TextureSpec} texture
     * @param {boolean} isVertexCase
     */
    es3fShaderTextureFunctionTests.TextureSizeCase = function(name, desc, samplerType, texture, isVertexCase) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        /** @type {string} */ this.m_samplerTypeStr = samplerType;
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ this.m_textureSpec = texture;
        /** @type {boolean} */ this.m_isVertexCase = isVertexCase;
        /** @type {boolean} */ this.m_has3DSize = texture.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D || texture.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY;
        /** @type {?gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {number} */ this.m_iterationCounter = 0;
    };

    es3fShaderTextureFunctionTests.TextureSizeCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.constructor = es3fShaderTextureFunctionTests.TextureSizeCase;

    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.deinit = function() {
        this.freeShader();
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.iterate = function() {
        /** @type {number} */ var currentIteration = this.m_iterationCounter++;
        /** @type {Array<es3fShaderTextureFunctionTests.TestSize>} */ var testSizes = [
            new es3fShaderTextureFunctionTests.TestSize([1, 2, 1], 1, 0, [1, 1, 1]),
            new es3fShaderTextureFunctionTests.TestSize([1, 2, 1], 0, 0, [1, 2, 1]),

            new es3fShaderTextureFunctionTests.TestSize([1, 3, 2], 0, 0, [1, 3, 2]),
            new es3fShaderTextureFunctionTests.TestSize([1, 3, 2], 1, 0, [1, 1, 1]),

            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 0, 0, [100, 31, 18]),
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 1, 0, [50, 15, 9]),
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 2, 0, [25, 7, 4]),
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 3, 0, [12, 3, 2]),
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 4, 0, [6, 1, 1]),
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 5, 0, [3, 1, 1]),
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 6, 0, [1, 1, 1]),

            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 0, 0, [100, 128, 32]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 1, 0, [50, 64, 16]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 2, 0, [25, 32, 8]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 3, 0, [12, 16, 4]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 4, 0, [6, 8, 2]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 5, 0, [3, 4, 1]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 6, 0, [1, 2, 1]),
            new es3fShaderTextureFunctionTests.TestSize([100, 128, 32], 7, 0, [1, 1, 1]),

            // pow 2
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 0, 0, [128, 64, 32]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 1, 0, [64, 32, 16]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 2, 0, [32, 16, 8]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 3, 0, [16, 8, 4]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 4, 0, [8, 4, 2]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 5, 0, [4, 2, 1]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 6, 0, [2, 1, 1]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 7, 0, [1, 1, 1]),

            // w === h
            new es3fShaderTextureFunctionTests.TestSize([1, 1, 1], 0, 0, [1, 1, 1]),
            new es3fShaderTextureFunctionTests.TestSize([64, 64, 64], 0, 0, [64, 64, 64]),
            new es3fShaderTextureFunctionTests.TestSize([64, 64, 64], 1, 0, [32, 32, 32]),
            new es3fShaderTextureFunctionTests.TestSize([64, 64, 64], 2, 0, [16, 16, 16]),
            new es3fShaderTextureFunctionTests.TestSize([64, 64, 64], 3, 0, [8, 8, 8]),
            new es3fShaderTextureFunctionTests.TestSize([64, 64, 64], 4, 0, [4, 4, 4]),

            // with lod base
            new es3fShaderTextureFunctionTests.TestSize([100, 31, 18], 3, 1, [6, 1, 1]),
            new es3fShaderTextureFunctionTests.TestSize([128, 64, 32], 3, 1, [8, 4, 2]),
            new es3fShaderTextureFunctionTests.TestSize([64, 64, 64], 1, 1, [16, 16, 16])

        ];
        /** @type {number} */ var lastIterationIndex = testSizes.length + 1;

        if (currentIteration === 0) {
            return this.initShader() ? tcuTestCase.IterateResult.CONTINUE : tcuTestCase.IterateResult.STOP;
        }
        else if (currentIteration === lastIterationIndex) {
            this.freeShader();
            return tcuTestCase.IterateResult.STOP;
        }
        else {
            if (!this.testTextureSize(testSizes[currentIteration - 1]))
                testFailedOptions('Fail: Case ' + (currentIteration - 1) + ' Got unexpected texture size', false);
            else
                testPassedOptions('Pass', true);

            return tcuTestCase.IterateResult.CONTINUE;
        }
    };

    /**
     * @return {boolean}
     */
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.initShader = function() {
        /** @type {string} */ var vertSrc = this.genVertexShader();
        /** @type {string} */ var fragSrc = this.genFragmentShader();

        assertMsgOptions(this.m_program === null, 'Program should be null', false, true);
        this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertSrc, fragSrc));

        if (!this.m_program.isOk()) {
            testFailedOptions('Fail: Shader failed', false);
            return false;
        }

        return true;
    };

    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.freeShader = function() {
        this.m_program = null;
    };

    /**
     * @param {es3fShaderTextureFunctionTests.TestSize} testSize
     * @return {boolean}
     */
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.testTextureSize = function(testSize) {
        /** @type {Array<number>} */ var triangle = [ // covers entire viewport
            -1, -1, 0, 1, // was a 3x4 matrix
            4, -1, 0, 1,
            -1, 4, 0, 1
        ];

        /** @type {number} */ var positionLoc = gl.getAttribLocation(this.m_program.getProgram(), 'a_position');
        /** @type {WebGLUniformLocation} */ var samplerLoc = gl.getUniformLocation(this.m_program.getProgram(), 'u_sampler');
        /** @type {WebGLUniformLocation} */ var sizeLoc = gl.getUniformLocation(this.m_program.getProgram(), 'u_texSize');
        /** @type {WebGLUniformLocation} */ var lodLoc = gl.getUniformLocation(this.m_program.getProgram(), 'u_lod');
        /** @type {number} */ var textureTarget = this.getGLTextureTarget();
        /** @type {boolean} */ var isSquare = testSize.textureSize[0] === testSize.textureSize[1];
        /** @type {boolean} */ var is2DLodValid = (testSize.textureSize[0] >> (testSize.lod + testSize.lodBase)) !== 0 || (testSize.textureSize[1] >> (testSize.lod + testSize.lodBase)) !== 0;
        /** @type {boolean} */ var success = true;
        /** @type {number} */ var errorValue;

        // Skip incompatible cases
        if (this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP && !isSquare)
            return true;
        if (this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D && !is2DLodValid)
            return true;
        if (this.m_textureSpec.type === es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY && !is2DLodValid)
            return true;

        // setup rendering
        gl.useProgram(this.m_program.getProgram());
        gl.uniform1i(samplerLoc, 0);
        gl.clearColor(0.5, 0.5, 0.5, 1.0);
        gl.viewport(0, 0, 1, 1);

        /** @type {WebGLBuffer} */ var triangleGlBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, triangleGlBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(triangle), gl.STATIC_DRAW);

        gl.vertexAttribPointer(positionLoc, 4, gl.FLOAT, false, 0, 0);
        gl.enableVertexAttribArray(positionLoc);

        // setup texture
        /** @type {number} */ var maxLevel = testSize.lod + testSize.lodBase;
        /** @type {number} */ var levels = maxLevel + 1;
        /** @type {?WebGLTexture} */ var texId = null;

        // gen texture
        texId = gl.createTexture();
        gl.bindTexture(textureTarget, texId);
        gl.texParameteri(textureTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(textureTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(textureTarget, gl.TEXTURE_BASE_LEVEL, testSize.lodBase);

        // set up texture

        switch (this.m_textureSpec.type) {
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D:
                // bufferedLogToConsole('Testing image size ' + testSize.textureSize[0] + 'x' + testSize.textureSize[1] + 'x' + testSize.textureSize[2]);
                // bufferedLogToConsole('Lod: ' + testSize.lod + ', base level: ' + testSize.lodBase);
                // bufferedLogToConsole('Expecting: ' + testSize.expectedSize[0] + 'x' + testSize.expectedSize[1] + 'x' + testSize.expectedSize[2]);

                gl.uniform3iv(sizeLoc, testSize.expectedSize);
                gl.uniform1iv(lodLoc, [testSize.lod]);

                gl.texStorage3D(textureTarget, levels, this.m_textureSpec.format, testSize.textureSize[0], testSize.textureSize[1], testSize.textureSize[2]);
                break;

            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D:
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP:
                // bufferedLogToConsole('Testing image size ' + testSize.textureSize[0] + 'x' + testSize.textureSize[1]);
                // bufferedLogToConsole('Lod: ' + testSize.lod + ', base level: ' + testSize.lodBase);
                // bufferedLogToConsole('Expecting: ' + testSize.expectedSize[0] + 'x' + testSize.expectedSize[1]);

                gl.uniform2iv(sizeLoc, testSize.expectedSize.slice(0,2));
                gl.uniform1iv(lodLoc, [testSize.lod]);

                gl.texStorage2D(textureTarget, levels, this.m_textureSpec.format, testSize.textureSize[0], testSize.textureSize[1]);
                break;

            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY:
                /** @type {Array<number>} */ var expectedSize = [testSize.expectedSize[0], testSize.expectedSize[1], testSize.textureSize[2]];

                // bufferedLogToConsole('Testing image size ' + testSize.textureSize[0] + 'x' + testSize.textureSize[1] + ' with ' + testSize.textureSize[2] + ' layer(s)');
                // bufferedLogToConsole('Lod: ' + testSize.lod + ', base level: ' + testSize.lodBase);
                // bufferedLogToConsole('Expecting: ' + testSize.expectedSize[0] + 'x' + testSize.expectedSize[1] + ' and ' + testSize.textureSize[2] + ' layer(s)');

                gl.uniform3iv(sizeLoc, expectedSize);
                gl.uniform1iv(lodLoc, [testSize.lod]);

                gl.texStorage3D(textureTarget, levels, this.m_textureSpec.format, testSize.textureSize[0], testSize.textureSize[1], testSize.textureSize[2]);
                break;

            default:
                throw new Error('Type not supported');
        }

        // test
        /** @type {number} */ var colorTolerance = 0.1;
        /** @type {tcuSurface.Surface} */ var sample = new tcuSurface.Surface(1, 1);
        /** @type {Array<number>} */ var outputColor;

        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.TRIANGLES, 0, 3);
        gl.finish();

        sample.readViewport(gl, [0, 0, 1, 1]);

        outputColor = sample.getAccess().getPixel(0, 0);

        if (outputColor[0] >= 1.0 - colorTolerance &&
            outputColor[1] >= 1.0 - colorTolerance &&
            outputColor[2] >= 1.0 - colorTolerance)
            bufferedLogToConsole('Passed');
        else {
            // failure
            bufferedLogToConsole('Failed');
            success = false;
        }

        // free
        gl.bindTexture(textureTarget, null);
        gl.deleteTexture(texId);

        gl.useProgram(null);

        return success;
    };

    /**
     * @return {string}
     */
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.genVertexShader = function()  {
        /** @type {string} */ var vert = '';
        vert += '#version 300 es\n' +
                'in highp vec4 a_position;\n';

        if (this.m_isVertexCase) {
            vert += 'out mediump vec4 v_color;\n' +
                    'uniform highp ' + this.m_samplerTypeStr + ' u_sampler;\n' +
                    'uniform highp ivec' + (this.m_has3DSize ? 3 : 2) + ' u_texSize;\n' +
                    'uniform highp int u_lod;\n';
        }

        vert += 'void main()\n{\n';

        if (this.m_isVertexCase)
            vert += '    v_color = (textureSize(u_sampler, u_lod) == u_texSize ? vec4(1.0, 1.0, 1.0, 1.0) : vec4(0.0, 0.0, 0.0, 1.0));\n';

        vert += '    gl_Position = a_position;\n' +
                '}\n';

        return vert;
    };

    /**
     * @return {string}
     */
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.genFragmentShader = function()  {
        /** @type {string} */ var frag = '';

        frag += '#version 300 es\n' +
                'layout(location = 0) out mediump vec4 o_color;\n';

        if (this.m_isVertexCase)
                frag += 'in mediump vec4 v_color;\n';

        if (!this.m_isVertexCase) {
            frag += 'uniform highp ' + this.m_samplerTypeStr + ' u_sampler;\n' +
                    'uniform highp ivec' + (this.m_has3DSize ? 3 : 2) + ' u_texSize;\n' +
                    'uniform highp int u_lod;\n';
        }

        frag += 'void main()\n{\n';

        if (!this.m_isVertexCase)
            frag += '    o_color = (textureSize(u_sampler, u_lod) == u_texSize ? vec4(1.0, 1.0, 1.0, 1.0) : vec4(0.0, 0.0, 0.0, 1.0));\n';
        else
            frag += '    o_color = v_color;\n';

        frag += '}\n';

        return frag;
    };

    /**
     * @return {number}
     */
    es3fShaderTextureFunctionTests.TextureSizeCase.prototype.getGLTextureTarget = function()  {
        switch (this.m_textureSpec.type) {
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D:
                return gl.TEXTURE_2D;
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP:
                return gl.TEXTURE_CUBE_MAP;
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY:
                return gl.TEXTURE_2D_ARRAY;
            case es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D:
                return gl.TEXTURE_3D;
            default:
                throw new Error('Texture Type not supported.');
        }
    };

    /** @typedef {Array<string, es3fShaderTextureFunctionTests.TextureLookupSpec, es3fShaderTextureFunctionTests.TextureSpec, es3fShaderTextureFunctionTests.EvalFunc, es3fShaderTextureFunctionTests.CaseFlags>} */ es3fShaderTextureFunctionTests.TestSpec;

    /**
     * @param {string} name
     * @param {es3fShaderTextureFunctionTests.TexFunction} func
     * @param {Array<number>} minCoord
     * @param {Array<number>} maxCoord
     * @param {boolean} useBias
     * @param {number} minLodBias
     * @param {number} maxLodBias
     * @param {boolean} useOffset
     * @param {Array<number>} offset
     * @param {es3fShaderTextureFunctionTests.TextureSpec} texSpec
     * @param {es3fShaderTextureFunctionTests.TexEvalFunc} evalFunc
     * @param {es3fShaderTextureFunctionTests.CaseFlags} flags
     * @return {es3fShaderTextureFunctionTests.TexFuncCaseSpec}
     */
    es3fShaderTextureFunctionTests.getCaseSpec = function(name, func, minCoord, maxCoord, useBias, minLodBias, maxLodBias, useOffset, offset, texSpec, evalFunc, flags) {
        return new es3fShaderTextureFunctionTests.TexFuncCaseSpec(name,
            new es3fShaderTextureFunctionTests.TextureLookupSpec(func, minCoord, maxCoord, useBias, minLodBias, maxLodBias, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], useOffset, offset),
            texSpec,
            evalFunc,
            flags);
    };

    /**
     * @param {string} name
     * @param {es3fShaderTextureFunctionTests.TexFunction} func
     * @param {Array<number>} minCoord
     * @param {Array<number>} maxCoord
     * @param {Array<number>} mindx
     * @param {Array<number>} maxdx
     * @param {Array<number>} mindy
     * @param {Array<number>} maxdy
     * @param {boolean} useOffset
     * @param {Array<number>} offset
     * @param {es3fShaderTextureFunctionTests.TextureSpec} texSpec
     * @param {es3fShaderTextureFunctionTests.TexEvalFunc} evalFunc
     * @param {es3fShaderTextureFunctionTests.CaseFlags} flags
     * @return {es3fShaderTextureFunctionTests.TexFuncCaseSpec}
     */
    es3fShaderTextureFunctionTests.getGradCaseSpec = function(name, func, minCoord, maxCoord, mindx, maxdx, mindy, maxdy, useOffset, offset, texSpec, evalFunc, flags) {
        return new es3fShaderTextureFunctionTests.TexFuncCaseSpec(name,
            new es3fShaderTextureFunctionTests.TextureLookupSpec(func, minCoord, maxCoord, false, 0.0, 0.0, mindx, maxdx, mindy, maxdy, useOffset, offset),
            texSpec,
            evalFunc,
            flags);
    };

    /**
     * @enum {number}
     */
    es3fShaderTextureFunctionTests.CaseFlags = {
        VERTEX: 1,
        FRAGMENT: 2,
        BOTH: 3
    };

    /**
     * @struct
     * @constructor
     * @param {string} name
     * @param {es3fShaderTextureFunctionTests.TextureLookupSpec} lookupSpec
     * @param {es3fShaderTextureFunctionTests.TextureSpec} texSpec
     * @param {es3fShaderTextureFunctionTests.TexEvalFunc} evalFunc
     * @param {number} flags
     */
    es3fShaderTextureFunctionTests.TexFuncCaseSpec = function(name, lookupSpec, texSpec, evalFunc, flags) {
        /** @type {string} */ this.name = name;
        /** @type {es3fShaderTextureFunctionTests.TextureLookupSpec} */ this.lookupSpec = lookupSpec;
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ this.texSpec = texSpec;
        /** @type {es3fShaderTextureFunctionTests.TexEvalFunc} */ this.evalFunc = evalFunc;
        /** @type {number} */ this.flags = flags;
    };

    /**
     * @param  {tcuTestCase.DeqpTest} parent
     * @param  {string} groupName
     * @param  {string} groupDesc
     * @param  {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} cases
     */
    es3fShaderTextureFunctionTests.createCaseGroup = function(parent, groupName, groupDesc, cases) {
        /** @type {tcuTestCase.DeqpTest} */ var group = tcuTestCase.newTest(groupName, groupDesc);
        parent.addChild(group);

        for (var ndx = 0; ndx < cases.length; ndx++) {
            /** @type {string} */ var name = cases[ndx].name;
            if (cases[ndx].flags & es3fShaderTextureFunctionTests.CaseFlags.VERTEX)
                group.addChild(new es3fShaderTextureFunctionTests.ShaderTextureFunctionCase(name + '_vertex', '', cases[ndx].lookupSpec, cases[ndx].texSpec, cases[ndx].evalFunc, true));
            if (cases[ndx].flags & es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
                group.addChild(new es3fShaderTextureFunctionTests.ShaderTextureFunctionCase(name + '_fragment', '', cases[ndx].lookupSpec, cases[ndx].texSpec, cases[ndx].evalFunc, false));
        }
    };

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fShaderTextureFunctionTests.ShaderTextureFunctionTests = function() {
        tcuTestCase.DeqpTest.call(this, 'texture_functions', 'Texture Access Function Tests');
    };

    es3fShaderTextureFunctionTests.ShaderTextureFunctionTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderTextureFunctionTests.ShaderTextureFunctionTests.prototype.constructor = es3fShaderTextureFunctionTests.ShaderTextureFunctionTests;

    es3fShaderTextureFunctionTests.ShaderTextureFunctionTests.prototype.init = function() {
        // Samplers
        /** @type {tcuTexture.Sampler} */ var samplerNearestNoMipmap = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST,
                                                             0.0 /* LOD threshold */, true /* normalized coords */, tcuTexture.CompareMode.COMPAREMODE_NONE,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);
        /** @type {tcuTexture.Sampler} */ var samplerLinearNoMipmap = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.LINEAR, tcuTexture.FilterMode.LINEAR,
                                                             0.0 /* LOD threshold */, true /* normalized coords */, tcuTexture.CompareMode.COMPAREMODE_NONE,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);
        /** @type {tcuTexture.Sampler} */ var samplerNearestMipmap = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST, tcuTexture.FilterMode.NEAREST,
                                                             0.0 /* LOD threshold */, true /* normalized coords */, tcuTexture.CompareMode.COMPAREMODE_NONE,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);
        /** @type {tcuTexture.Sampler} */ var samplerLinearMipmap = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.LINEAR_MIPMAP_NEAREST, tcuTexture.FilterMode.LINEAR,
                                                             0.0 /* LOD threshold */, true /* normalized coords */, tcuTexture.CompareMode.COMPAREMODE_NONE,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);

        /** @type {tcuTexture.Sampler} */ var samplerShadowNoMipmap = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.NEAREST, tcuTexture.FilterMode.NEAREST,
                                                             0.0 /* LOD threshold */, true /* normalized coords */, tcuTexture.CompareMode.COMPAREMODE_LESS,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);
        /** @type {tcuTexture.Sampler} */ var samplerShadowMipmap = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST, tcuTexture.FilterMode.NEAREST,
                                                             0.0 /* LOD threshold */, true /* normalized coords */, tcuTexture.CompareMode.COMPAREMODE_LESS,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);

        /** @type {tcuTexture.Sampler} */ var samplerTexelFetch = new tcuTexture.Sampler(tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL, tcuTexture.WrapMode.REPEAT_GL,
                                                             tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST, tcuTexture.FilterMode.NEAREST,
                                                             0.0 /* LOD threshold */, false /* non-normalized coords */, tcuTexture.CompareMode.COMPAREMODE_NONE,
                                                             0 /* cmp channel */, [0.0, 0.0, 0.0, 0.0] /* border color */, true /* seamless cube map */);

        // Default textures.
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8, 256, 256, 1, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA16F, 256, 256, 1, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8I, 256, 256, 1, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8UI, 256, 256, 1, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DMipmapFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8, 256, 256, 1, 9, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DMipmapFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA16F, 256, 256, 1, 9, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DMipmapInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8I, 256, 256, 1, 9, samplerNearestMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DMipmapUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8UI, 256, 256, 1, 9, samplerNearestMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DShadow = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.DEPTH_COMPONENT16, 256, 256, 1, 9, samplerShadowNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DMipmapShadow = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.DEPTH_COMPONENT16, 256, 256, 1, 9, samplerShadowMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DTexelFetchFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8, 256, 256, 1, 9, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DTexelFetchFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA16F, 256, 256, 1, 9, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DTexelFetchInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8I, 256, 256, 1, 9, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DTexelFetchUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D, gl.RGBA8UI, 256, 256, 1, 9, samplerTexelFetch);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA8, 256, 256, 1, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA16F, 256, 256, 1, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA8I, 256, 256, 1, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA8UI, 256, 256, 1, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeMipmapFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA8, 256, 256, 1, 9, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeMipmapFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA16F, 128, 128, 1, 8, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeMipmapInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA8I, 256, 256, 1, 9, samplerNearestMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeMipmapUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.RGBA8UI, 256, 256, 1, 9, samplerNearestMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeShadow = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.DEPTH_COMPONENT16, 256, 256, 1, 1, samplerShadowNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var texCubeMipmapShadow = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_CUBE_MAP, gl.DEPTH_COMPONENT16, 256, 256, 1, 9, samplerShadowMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8, 128, 128, 4, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA16F, 128, 128, 4, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8I, 128, 128, 4, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8UI, 128, 128, 4, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayMipmapFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8, 128, 128, 4, 8, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayMipmapFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA16F, 128, 128, 4, 8, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayMipmapInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8I, 128, 128, 4, 8, samplerNearestMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayMipmapUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8UI, 128, 128, 4, 8, samplerNearestMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayShadow = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.DEPTH_COMPONENT16, 128, 128, 4, 1, samplerShadowNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayMipmapShadow = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.DEPTH_COMPONENT16, 128, 128, 4, 8, samplerShadowMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayTexelFetchFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8, 128, 128, 4, 8, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayTexelFetchFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA16F, 128, 128, 4, 8, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayTexelFetchInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8I, 128, 128, 4, 8, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex2DArrayTexelFetchUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_2D_ARRAY, gl.RGBA8UI, 128, 128, 4, 8, samplerTexelFetch);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8, 64, 32, 32, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA16F, 64, 32, 32, 1, samplerLinearNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8I, 64, 32, 32, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8UI, 64, 32, 32, 1, samplerNearestNoMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DMipmapFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8, 64, 32, 32, 7, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DMipmapFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA16F, 64, 32, 32, 7, samplerLinearMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DMipmapInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8I, 64, 32, 32, 7, samplerNearestMipmap);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DMipmapUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8UI, 64, 32, 32, 7, samplerNearestMipmap);

        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DTexelFetchFixed = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8, 64, 32, 32, 7, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DTexelFetchFloat = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA16F, 64, 32, 32, 7, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DTexelFetchInt = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8I, 64, 32, 32, 7, samplerTexelFetch);
        /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ var tex3DTexelFetchUint = new es3fShaderTextureFunctionTests.TextureSpec(es3fShaderTextureFunctionTests.TextureType.TEXTURETYPE_3D, gl.RGBA8UI, 64, 32, 32, 7, samplerTexelFetch);

        var testGroup = tcuTestCase.runner.testCases;
        // texture() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeFixed, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeMipmapFixed, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeFloat, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeMipmapFloat, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeInt, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeMipmapInt, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeUint, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeMipmapUint, es3fShaderTextureFunctionTests.evalTextureCube, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    texCubeMipmapFixed, es3fShaderTextureFunctionTests.evalTextureCubeBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    texCubeMipmapFloat, es3fShaderTextureFunctionTests.evalTextureCubeBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isamplercube_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    texCubeMipmapInt, es3fShaderTextureFunctionTests.evalTextureCubeBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usamplercube_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    texCubeMipmapUint, es3fShaderTextureFunctionTests.evalTextureCubeBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayFixed, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayFloat, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayInt, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayUint, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArray, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArrayBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArrayBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DFixed, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DFloat, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DInt, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DUint, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3D, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    1.0,    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    1.0,    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DShadow, es3fShaderTextureFunctionTests.evalTexture2DShadow, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadow, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('samplercubeshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 1.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeShadow, es3fShaderTextureFunctionTests.evalTextureCubeShadow, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercubeshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 1.0],    false,    0.0,    0.0,    false, [0, 0, 0],    texCubeMipmapShadow, es3fShaderTextureFunctionTests.evalTextureCubeShadow, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercubeshadow_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 1.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    texCubeMipmapShadow, es3fShaderTextureFunctionTests.evalTextureCubeShadowBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darrayshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadow, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darrayshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadow,        es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)

                // Not in spec.
                // es3fShaderTextureFunctionTests.getCaseSpec('sampler2darrayshadow_bias',    (es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0],    true,    -2.0,    2.0,    Vec2(0.0),    Vec2(0.0), false, [0, 0, 0]),    tex2DArrayMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadowBias,    FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texture', 'texture() Tests', textureCases);

            // textureOffset() cases
            // \note _bias variants are not using mipmap thanks to wide allowed range for LOD computation
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureOffsetCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DArrayFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DArrayFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DArrayInt, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DArrayUint, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArrayOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DArrayFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DArrayFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DArrayInt, es3fShaderTextureFunctionTests.evalTexture2DArrayOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DArrayUint, es3fShaderTextureFunctionTests.evalTexture2DArrayOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DFixed, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [7, 3, -8],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [3, -8, 7],    tex3DFloat, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [7, 3, -8],    tex3DInt, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [3, -8, 7],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DUint, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    0.0,    0.0,    true, [7, 3, -8],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    1.0,    true, [-8, 7, 3],    tex3DFixed, es3fShaderTextureFunctionTests.evalTexture3DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    1.0,    true, [7, 3, -8],    tex3DFloat, es3fShaderTextureFunctionTests.evalTexture3DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    2.0,    true, [3, -8, 7],    tex3DInt, es3fShaderTextureFunctionTests.evalTexture3DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 3],    tex3DUint, es3fShaderTextureFunctionTests.evalTexture3DOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTURE, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowOffsetBias,    es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureoffset', 'textureOffset() Tests', textureOffsetCases);

            // textureProj() cases
            // \note Currently uses constant divider!
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureProjCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProj3, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProj3Bias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProj3Bias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProj3Bias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProj3Bias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DFixed, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DFloat, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DInt, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DUint, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    1.0,    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    1.0,    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProj, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProj, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    true,    -2.0,    2.0,    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjBias,    es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureproj', 'textureProj() Tests', textureProjCases);

            // textureProjOffset() cases
            // \note Currently uses constant divider!
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureProjOffsetCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProj3Offset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DProj3OffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DProj3OffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DProj3OffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DProj3OffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DFixed, es3fShaderTextureFunctionTests.evalTexture2DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DFloat, es3fShaderTextureFunctionTests.evalTexture2DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DInt, es3fShaderTextureFunctionTests.evalTexture2DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    true,    -2.0,    2.0,    true, [7, -8, 0],    tex2DUint, es3fShaderTextureFunctionTests.evalTexture2DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DFixed, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [7, 3, -8],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [3, -8, 7],    tex3DFloat, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [7, 3, -8],    tex3DInt, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [3, -8, 7],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DUint, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    0.0,    0.0,    true, [7, 3, -8],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    2.0,    true, [-8, 7, 3],    tex3DFixed, es3fShaderTextureFunctionTests.evalTexture3DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_bias_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    2.0,    true, [7, 3, -8],    tex3DFloat, es3fShaderTextureFunctionTests.evalTexture3DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    2.0,    true, [3, -8, 7],    tex3DInt, es3fShaderTextureFunctionTests.evalTexture3DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    true,    -2.0,    2.0,    true, [-8, 7, 3],    tex3DUint, es3fShaderTextureFunctionTests.evalTexture3DProjOffsetBias, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                // NOTE: offset changed from [-8, 7, 0] in native dEQP to [7, -8, 0] per https://github.com/KhronosGroup/WebGL/issues/2033
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    false,    0.0,    0.0,    true, [7, -8, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow_bias', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJ, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    true,    -2.0,    2.0,    true, [-8, 7, 0],    tex2DShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjOffsetBias,    es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureprojoffset', 'textureOffsetProj() Tests', textureProjOffsetCases);

            // textureLod() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureLodCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    texCubeMipmapFixed, es3fShaderTextureFunctionTests.evalTextureCubeLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('samplercube_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    texCubeMipmapFloat, es3fShaderTextureFunctionTests.evalTextureCubeLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    texCubeMipmapInt, es3fShaderTextureFunctionTests.evalTextureCubeLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    texCubeMipmapUint, es3fShaderTextureFunctionTests.evalTextureCubeLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    false, [0, 0, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    false, [0, 0, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    false, [0, 0, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArrayLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    false, [0, 0, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArrayLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowLod,    es3fShaderTextureFunctionTests.CaseFlags.BOTH)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texturelod', 'textureLod() Tests', textureLodCases);

            // textureLodOffset() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureLodOffsetCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0],    false,    -1.0,    9.0,    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    true, [-8, 7, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    true, [7, -8, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    true, [-8, 7, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArrayLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0],    false,    -1.0,    8.0,    true, [7, -8, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArrayLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    true, [-8, 7, 3],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    true, [7, 3, -8],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    true, [3, -8, 7],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0],    false,    -1.0,    7.0,    true, [-8, 7, 3],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTURELOD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowLodOffset,    es3fShaderTextureFunctionTests.CaseFlags.BOTH)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texturelodoffset', 'textureLodOffset() Tests', textureLodOffsetCases);

            // textureProjLod() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureProjLodCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjLod3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjLod3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjLod3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjLod3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjLod, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    false,    -1.0,    9.0,    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjLod,    es3fShaderTextureFunctionTests.CaseFlags.BOTH)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureprojlod', 'textureProjLod() Tests', textureProjLodCases);

            // textureProjLodOffset() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureProjLodOffsetCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjLod3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjLod3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjLod3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0],    false,    -1.0,    9.0,    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjLod3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5],    false,    -1.0,    9.0,    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    true, [-8, 7, 3],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    true, [7, 3, -8],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    true, [3, -8, 7],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75],    false,    -1.0,    7.0,    true, [-8, 7, 3],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjLodOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJLOD, [0.2, 0.6, 0.0, 1.5], [-2.25, -3.45, 1.5, 1.5],    false,    -1.0,    9.0,    true, [-8, 7, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjLodOffset,    es3fShaderTextureFunctionTests.CaseFlags.BOTH)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureprojlodoffset', 'textureProjLodOffset() Tests', textureProjLodOffsetCases);

            // textureGrad() cases
            // \note Only one of dudx, dudy, dvdx, dvdy is non-zero since spec allows approximating p from derivates by various methods.
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureGradCases = [
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('samplercube_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    texCubeMipmapFixed, es3fShaderTextureFunctionTests.evalTextureCubeGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('samplercube_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    texCubeMipmapFloat, es3fShaderTextureFunctionTests.evalTextureCubeGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    texCubeMipmapInt, es3fShaderTextureFunctionTests.evalTextureCubeGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usamplercube', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.0, -1.0, -1.01, 0.0], [1.0, 1.0, -1.01, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    texCubeMipmapUint, es3fShaderTextureFunctionTests.evalTextureCubeGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArrayGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArrayGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DGrad, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.2], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DGrad, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DGrad, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, -0.2],    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DGrad, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('samplercubeshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.0, -1.0, 1.01, 0.0], [1.0, 1.0, 1.01, 1.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    texCubeMipmapShadow, es3fShaderTextureFunctionTests.evalTextureCubeShadowGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darrayshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0],    false, [0, 0, 0],    tex2DArrayMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadowGrad, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darrayshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0],    false, [0, 0, 0],    tex2DArrayMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadowGrad, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texturegrad', 'textureGrad() Tests', textureGradCases);

            // textureGradOffset() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureGradOffsetCases = [
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 0],    tex2DArrayMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DArrayGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, -8, 0],    tex2DArrayMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DArrayGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    true, [-8, 7, 0],    tex2DArrayMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DArrayGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    true, [7, -8, 0],    tex2DArrayMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DArrayGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 3],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, 3, -8],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.2], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [3, -8, 7],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    true, [-8, 7, 3],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    true, [7, 3, -8],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -1.4, 0.1, 0.0], [1.5, 2.3, 2.3, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, -0.2],    true, [3, -8, 7],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DGradOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-0.2, -0.4, 0.0, 0.0], [1.5, 2.3, 1.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, -8, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowGradOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darrayshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0],    true, [-8, 7, 0],    tex2DArrayMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadowGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2darrayshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREGRAD, [-1.2, -0.4, -0.5, 0.0], [1.5, 2.3, 3.5, 1.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0],    true, [7, -8, 0],    tex2DArrayMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DArrayShadowGradOffset,    es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texturegradoffset', 'textureGradOffset() Tests', textureGradOffsetCases);

            // textureProjGrad() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureProjGradCases = [
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.2], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    false, [0, 0, 0],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, -0.2],    false, [0, 0, 0],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjGrad, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.2, 0.6, 0.0, -1.5], [-2.25, -3.45, -1.5, -1.5], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjGrad, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.2, 0.6, 0.0, -1.5], [-2.25, -3.45, -1.5, -1.5], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0],    false, [0, 0, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjGrad,    es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureprojgrad', 'textureProjGrad() Tests', textureProjGradCases);

            // textureProjGradOffset() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var textureProjGradOffsetCases = [
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec3_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec3_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2d_vec3', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD3, [-0.3, -0.6, 1.5, 0.0], [2.25, 3.45, 1.5, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjGrad3Offset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec4_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture2DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2d_vec4_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, -8, 0],    tex2DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture2DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapInt, es3fShaderTextureFunctionTests.evalTexture2DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler2d_vec4', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [-0.3, -0.6, 0.0, 1.5], [2.25, 3.45, 0.0, 1.5], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    true, [7, -8, 0],    tex2DMipmapUint, es3fShaderTextureFunctionTests.evalTexture2DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 3],    tex3DMipmapFixed, es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [7, 3, -8],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.2], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [3, -8, 7],    tex3DMipmapFloat, es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),
                es3fShaderTextureFunctionTests.getGradCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [-0.2, 0.0, 0.0],    true, [-8, 7, 3],    tex3DMipmapInt, es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.2, 0.0],    true, [7, 3, -8],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.9, 1.05, -0.08, -0.75], [-1.13, -1.7, -1.7, -0.75], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, -0.2],    true, [3, -8, 7],    tex3DMipmapUint, es3fShaderTextureFunctionTests.evalTexture3DProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT),

                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.2, 0.6, 0.0, -1.5], [-2.25, -3.45, -1.5, -1.5], [0.0, 0.0, 0.0], [0.2, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0],    true, [-8, 7, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjGradOffset, es3fShaderTextureFunctionTests.CaseFlags.VERTEX),
                es3fShaderTextureFunctionTests.getGradCaseSpec('sampler2dshadow', es3fShaderTextureFunctionTests.TexFunction.TEXTUREPROJGRAD, [0.2, 0.6, 0.0, -1.5], [-2.25, -3.45, -1.5, -1.5], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, -0.2, 0.0],    true, [7, -8, 0],    tex2DMipmapShadow, es3fShaderTextureFunctionTests.evalTexture2DShadowProjGradOffset,    es3fShaderTextureFunctionTests.CaseFlags.FRAGMENT)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'textureprojgradoffset', 'textureProjGradOffset() Tests', textureProjGradOffsetCases);

            // texelFetch() cases
            // \note Level is constant across quad
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var texelFetchCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [255.9, 255.9, 0.0, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DTexelFetchFixed, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [127.9, 127.9, 0.0, 0.0],    false,    1.0,    1.0,    false, [0, 0, 0],    tex2DTexelFetchFloat, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [63.9, 63.9, 0.0, 0.0],    false,    2.0,    2.0,    false, [0, 0, 0],    tex2DTexelFetchInt, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [15.9, 15.9, 0.0, 0.0],    false,    4.0,    4.0,    false, [0, 0, 0],    tex2DTexelFetchUint, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [127.9, 127.9, 3.9, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex2DArrayTexelFetchFixed, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [63.9, 63.9, 3.9, 0.0],    false,    1.0,    1.0,    false, [0, 0, 0],    tex2DArrayTexelFetchFloat, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [31.9, 31.9, 3.9, 0.0],    false,    2.0,    2.0,    false, [0, 0, 0],    tex2DArrayTexelFetchInt, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [15.9, 15.9, 3.9, 0.0],    false,    3.0,    3.0,    false, [0, 0, 0],    tex2DArrayTexelFetchUint, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [63.9, 31.9, 31.9, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DTexelFetchFixed, es3fShaderTextureFunctionTests.evalTexelFetch3D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [31.9, 15.9, 15.9, 0.0],    false,    1.0,    1.0,    false, [0, 0, 0],    tex3DTexelFetchFloat, es3fShaderTextureFunctionTests.evalTexelFetch3D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [15.9,  7.9,  7.9, 0.0],    false,    2.0,    2.0,    false, [0, 0, 0],    tex3DTexelFetchInt, es3fShaderTextureFunctionTests.evalTexelFetch3D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [0.0, 0.0, 0.0, 0.0], [63.9, 31.9, 31.9, 0.0],    false,    0.0,    0.0,    false, [0, 0, 0],    tex3DTexelFetchUint, es3fShaderTextureFunctionTests.evalTexelFetch3D,        es3fShaderTextureFunctionTests.CaseFlags.BOTH)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texelfetch', 'texelFetch() Tests', texelFetchCases);

            // texelFetchOffset() cases
            /** @type {Array<es3fShaderTextureFunctionTests.TexFuncCaseSpec>} */ var texelFetchOffsetCases = [
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [8.0, -7.0, 0.0, 0.0], [263.9, 248.9, 0.0, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DTexelFetchFixed, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2d_float', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [-7.0, 8.0, 0.0, 0.0], [120.9, 135.9, 0.0, 0.0],    false,    1.0,    1.0,    true, [7, -8, 0],    tex2DTexelFetchFloat, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [8.0, -7.0, 0.0, 0.0], [71.9, 56.9, 0.0, 0.0],    false,    2.0,    2.0,    true, [-8, 7, 0],    tex2DTexelFetchInt, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [-7.0, 8.0, 0.0, 0.0], [8.9, 23.9, 0.0, 0.0],    false,    4.0,    4.0,    true, [7, -8, 0],    tex2DTexelFetchUint, es3fShaderTextureFunctionTests.evalTexelFetch2D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [8.0, -7.0, 0.0, 0.0], [135.9, 120.9, 3.9, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 0],    tex2DArrayTexelFetchFixed, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler2darray_float', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [-7.0, 8.0, 0.0, 0.0], [56.9, 71.9, 3.9, 0.0],    false,    1.0,    1.0,    true, [7, -8, 0],    tex2DArrayTexelFetchFloat, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [8.0, -7.0, 0.0, 0.0], [39.9, 24.9, 3.9, 0.0],    false,    2.0,    2.0,    true, [-8, 7, 0],    tex2DArrayTexelFetchInt, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler2darray', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [-7.0, 8.0, 0.0, 0.0], [8.9, 23.9, 3.9, 0.0],    false,    3.0,    3.0,    true, [7, -8, 0],    tex2DArrayTexelFetchUint, es3fShaderTextureFunctionTests.evalTexelFetch2DArray, es3fShaderTextureFunctionTests.CaseFlags.BOTH),

                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_fixed', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [8.0, -7.0, -3.0, 0.0], [71.9, 24.9, 28.9, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DTexelFetchFixed, es3fShaderTextureFunctionTests.evalTexelFetch3D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('sampler3d_float', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [-7.0, -3.0, 8.0, 0.0], [24.9, 12.9, 23.9, 0.0],    false,    1.0,    1.0,    true, [7, 3, -8],    tex3DTexelFetchFloat, es3fShaderTextureFunctionTests.evalTexelFetch3D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('isampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [-3.0, 8.0, -7.0, 0.0], [12.9, 15.9,  0.9, 0.0],    false,    2.0,    2.0,    true, [3, -8, 7],    tex3DTexelFetchInt, es3fShaderTextureFunctionTests.evalTexelFetch3D, es3fShaderTextureFunctionTests.CaseFlags.BOTH),
                es3fShaderTextureFunctionTests.getCaseSpec('usampler3d', es3fShaderTextureFunctionTests.TexFunction.TEXELFETCH, [8.0, -7.0, -3.0, 0.0], [71.9, 24.9, 28.9, 0.0],    false,    0.0,    0.0,    true, [-8, 7, 3],    tex3DTexelFetchUint, es3fShaderTextureFunctionTests.evalTexelFetch3D,        es3fShaderTextureFunctionTests.CaseFlags.BOTH)
            ];
            es3fShaderTextureFunctionTests.createCaseGroup(this, 'texelfetchoffset', 'texelFetchOffset() Tests', texelFetchOffsetCases);

            // textureSize() cases
            /**
             * @struct
             * @constructor
             * @param  {string} name
             * @param  {string} samplerName
             * @param  {es3fShaderTextureFunctionTests.TextureSpec} textureSpec
             */
            var TextureSizeCaseSpec = function(name, samplerName, textureSpec) {
                /** @type {string} */ this.name = name;
                /** @type {string} */ this.samplerName = samplerName;
                /** @type {es3fShaderTextureFunctionTests.TextureSpec} */ this.textureSpec = textureSpec;
            };

            /** @type {Array<TextureSizeCaseSpec>} */ var  textureSizeCases = [
                new TextureSizeCaseSpec('sampler2d_fixed', 'sampler2D', tex2DFixed),
                new TextureSizeCaseSpec('sampler2d_float', 'sampler2D', tex2DFloat),
                new TextureSizeCaseSpec('isampler2d', 'isampler2D', tex2DInt),
                new TextureSizeCaseSpec('usampler2d', 'usampler2D', tex2DUint),
                new TextureSizeCaseSpec('sampler2dshadow', 'sampler2DShadow', tex2DShadow),
                new TextureSizeCaseSpec('sampler3d_fixed', 'sampler3D', tex3DFixed),
                new TextureSizeCaseSpec('sampler3d_float', 'sampler3D', tex3DFloat),
                new TextureSizeCaseSpec('isampler3d', 'isampler3D', tex3DInt),
                new TextureSizeCaseSpec('usampler3d', 'usampler3D', tex3DUint),
                new TextureSizeCaseSpec('samplercube_fixed', 'samplerCube', texCubeFixed),
                new TextureSizeCaseSpec('samplercube_float', 'samplerCube', texCubeFloat),
                new TextureSizeCaseSpec('isamplercube', 'isamplerCube', texCubeInt),
                new TextureSizeCaseSpec('usamplercube', 'usamplerCube', texCubeUint),
                new TextureSizeCaseSpec('samplercubeshadow', 'samplerCubeShadow', texCubeShadow),
                new TextureSizeCaseSpec('sampler2darray_fixed', 'sampler2DArray', tex2DArrayFixed),
                new TextureSizeCaseSpec('sampler2darray_float', 'sampler2DArray', tex2DArrayFloat),
                new TextureSizeCaseSpec('isampler2darray', 'isampler2DArray', tex2DArrayInt),
                new TextureSizeCaseSpec('usampler2darray', 'usampler2DArray', tex2DArrayUint),
                new TextureSizeCaseSpec('sampler2darrayshadow', 'sampler2DArrayShadow', tex2DArrayShadow)
            ];

            /** @type {tcuTestCase.DeqpTest} */ var group = tcuTestCase.newTest('texturesize', 'textureSize() Tests');
            testGroup.addChild(group);

            for (var ndx = 0; ndx < textureSizeCases.length; ++ndx) {
                group.addChild(new es3fShaderTextureFunctionTests.TextureSizeCase(textureSizeCases[ndx].name + '_vertex',  '', textureSizeCases[ndx].samplerName, textureSizeCases[ndx].textureSpec, true));
                group.addChild(new es3fShaderTextureFunctionTests.TextureSizeCase(textureSizeCases[ndx].name + '_fragment', '', textureSizeCases[ndx].samplerName, textureSizeCases[ndx].textureSpec, false));
            }

    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    * @param {Array<number>=} range Test range
    */
    es3fShaderTextureFunctionTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderTextureFunctionTests.ShaderTextureFunctionTests());
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
            testFailedOptions('Failed to es3fShaderTextureFunctionTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

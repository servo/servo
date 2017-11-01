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
goog.provide('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuMatrixUtil');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.simplereference.sglrReferenceUtils');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrMultisamplePixelBufferAccess');
goog.require('framework.referencerenderer.rrRenderState');
goog.require('framework.referencerenderer.rrRenderer');
goog.require('framework.referencerenderer.rrVertexAttrib');

goog.scope(function() {

    var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;
    var rrMultisamplePixelBufferAccess = framework.referencerenderer.rrMultisamplePixelBufferAccess;
    var tcuTexture = framework.common.tcuTexture;
    var deMath = framework.delibs.debase.deMath;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var rrRenderer = framework.referencerenderer.rrRenderer;
    var rrDefs = framework.referencerenderer.rrDefs;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
    var rrRenderState = framework.referencerenderer.rrRenderState;
    var sglrReferenceUtils = framework.opengl.simplereference.sglrReferenceUtils;
    var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
    var tcuMatrix = framework.common.tcuMatrix;
    var tcuMatrixUtil = framework.common.tcuMatrixUtil;

    sglrReferenceContext.rrMPBA = rrMultisamplePixelBufferAccess;

    //TODO: Implement automatic error checking in sglrReferenceContext, optional on creation.

    /** @typedef {WebGLRenderbuffer|WebGLTexture|sglrReferenceContext.Renderbuffer|sglrReferenceContext.TextureContainer} */ sglrReferenceContext.AnyRenderbuffer;

    /** @typedef {WebGLFramebuffer|sglrReferenceContext.Framebuffer} */ sglrReferenceContext.AnyFramebuffer;

    /**
     * @param {number} error
     * @param {number} message
     * @throws {Error}
     */
    sglrReferenceContext.GLU_EXPECT_NO_ERROR = function(error, message) {
        if (error !== gl.NONE) {
            bufferedLogToConsole('Assertion failed message:' + message);
        }
    };

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    // /* TODO: remove */
    // /** @type {WebGL2RenderingContext} */ var gl;

    sglrReferenceContext.MAX_TEXTURE_SIZE_LOG2 = 14;
    sglrReferenceContext.MAX_TEXTURE_SIZE = 1 << sglrReferenceContext.MAX_TEXTURE_SIZE_LOG2;

    /**
     * @param {number} width
     * @param {number} height
     * @return {number}
     */
    sglrReferenceContext.getNumMipLevels2D = function(width, height) {
        return Math.floor(Math.log2(Math.max(width, height)) + 1);
    };

    /**
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     * @return {number}
     */
    sglrReferenceContext.getNumMipLevels3D = function(width, height, depth) {
        return Math.floor(Math.log2(Math.max(width, height, depth)) + 1);
    };

    /**
     * @param {number} baseLevelSize
     * @param {number} levelNdx
     * @return {number}
     */
    sglrReferenceContext.getMipLevelSize = function(baseLevelSize, levelNdx) {
        return Math.max(baseLevelSize >> levelNdx, 1);
    };

    sglrReferenceContext.mapGLCubeFace = function(face) {
        switch (face) {
            case gl.TEXTURE_CUBE_MAP_NEGATIVE_X: return tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X;
            case gl.TEXTURE_CUBE_MAP_POSITIVE_X: return tcuTexture.CubeFace.CUBEFACE_POSITIVE_X;
            case gl.TEXTURE_CUBE_MAP_NEGATIVE_Y: return tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y;
            case gl.TEXTURE_CUBE_MAP_POSITIVE_Y: return tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y;
            case gl.TEXTURE_CUBE_MAP_NEGATIVE_Z: return tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z;
            case gl.TEXTURE_CUBE_MAP_POSITIVE_Z: return tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z;
            default: throw new Error('Invalid cube face: ' + face);
        }
    };

    /**
     * @param {tcuTexture.FilterMode} mode
     * @return {boolean}
     */
    sglrReferenceContext.isMipmapFilter = function(/*const tcu::Sampler::FilterMode*/ mode) {
        return mode != tcuTexture.FilterMode.NEAREST && mode != tcuTexture.FilterMode.LINEAR;
    };

    sglrReferenceContext.getNumMipLevels1D = function(size) {
        return Math.floor(Math.log2(size)) + 1;
    };

    /**
     * @param {?sglrReferenceContext.TextureType} type
     * @return {sglrReferenceContext.TexTarget}
     */
    sglrReferenceContext.texLayeredTypeToTarget = function(type) {
        switch (type) {
            case sglrReferenceContext.TextureType.TYPE_2D_ARRAY: return sglrReferenceContext.TexTarget.TEXTARGET_2D_ARRAY;
            case sglrReferenceContext.TextureType.TYPE_3D: return sglrReferenceContext.TexTarget.TEXTARGET_3D;
            case sglrReferenceContext.TextureType.TYPE_CUBE_MAP_ARRAY: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_ARRAY;
            default: throw new Error('Invalid texture type: ' + type);
        }
    };

    /**
     * @param {rrDefs.IndexType} indexType
     * @return {number}
     * @throws {Error}
     */
    sglrReferenceContext.getFixedRestartIndex = function(indexType) {
        switch (indexType) {
            case rrDefs.IndexType.INDEXTYPE_UINT8: return 0xFF;
            case rrDefs.IndexType.INDEXTYPE_UINT16: return 0xFFFF;
            case rrDefs.IndexType.INDEXTYPE_UINT32: return 0xFFFFFFFF;
            default:
                throw new Error('Unrecognized index type: ' + indexType);
            }
    };

    /**
    * @constructor
    * @param {sglrShaderProgram.ShaderProgram} program
    */
    sglrReferenceContext.ShaderProgramObjectContainer = function(program) {
        this.m_program = program;
        /** @type {boolean} */ this.m_deleteFlag = false;
    };

    /**
    * @param {WebGL2RenderingContext} gl
    * @constructor
    */
    sglrReferenceContext.ReferenceContextLimits = function(gl) {
        /** @type {number} */ this.maxTextureImageUnits = 16;
        /** @type {number} */ this.maxTexture2DSize = 2048;
        /** @type {number} */ this.maxTextureCubeSize = 2048;
        /** @type {number} */ this.maxTexture2DArrayLayers = 256;
        /** @type {number} */ this.maxTexture3DSize = 256;
        /** @type {number} */ this.maxRenderbufferSize = 2048;
        /** @type {number} */ this.maxVertexAttribs = 16;

        if (gl) {
            this.maxTextureImageUnits = /** @type {number} */ (gl.getParameter(gl.MAX_TEXTURE_IMAGE_UNITS));
            this.maxTexture2DSize = /** @type {number} */ (gl.getParameter(gl.MAX_TEXTURE_SIZE));
            this.maxTextureCubeSize = /** @type {number} */ (gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE));
            this.maxRenderbufferSize = /** @type {number} */ (gl.getParameter(gl.MAX_RENDERBUFFER_SIZE));
            this.maxVertexAttribs = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            this.maxTexture2DArrayLayers = /** @type {number} */ (gl.getParameter(gl.MAX_ARRAY_TEXTURE_LAYERS));
            this.maxTexture3DSize = /** @type {number} */ (gl.getParameter(gl.MAX_3D_TEXTURE_SIZE));

            // Limit texture sizes to supported values
            this.maxTexture2DSize = Math.min(this.maxTexture2DSize, sglrReferenceContext.MAX_TEXTURE_SIZE);
            this.maxTextureCubeSize = Math.min(this.maxTextureCubeSize, sglrReferenceContext.MAX_TEXTURE_SIZE);
            this.maxTexture3DSize = Math.min(this.maxTexture3DSize, sglrReferenceContext.MAX_TEXTURE_SIZE);

            sglrReferenceContext.GLU_EXPECT_NO_ERROR(gl.getError(), gl.NO_ERROR);
        }

        /* TODO: Port
        // \todo [pyry] Figure out following things:
        // + supported fbo configurations
        // ...

        // \todo [2013-08-01 pyry] Do we want to make these conditional based on renderCtx?
        addExtension("gl.EXT_color_buffer_half_float");
        addExtension("gl.WEBGL_color_buffer_float");
        */
    };

    /**
    * @enum
    */
    sglrReferenceContext.TextureType = {
        TYPE_2D: 0,
        TYPE_CUBE_MAP: 1,
        TYPE_2D_ARRAY: 2,
        TYPE_3D: 3,
        TYPE_CUBE_MAP_ARRAY: 4
    };

    /**
    * @constructor
    * @implements {rrDefs.Sampler}
    * @param {sglrReferenceContext.TextureType} type
    */
    sglrReferenceContext.Texture = function(type) {
        // NamedObject.call(this, name);
        /** @type {sglrReferenceContext.TextureType} */ this.m_type = type;
        /** @type {boolean} */ this.m_immutable = false;
        /** @type {number} */ this.m_baseLevel = 0;
        /** @type {number} */ this.m_maxLevel = 1000;
        /** @type {tcuTexture.Sampler} */ this.m_sampler = new tcuTexture.Sampler(
            tcuTexture.WrapMode.REPEAT_GL,
            tcuTexture.WrapMode.REPEAT_GL,
            tcuTexture.WrapMode.REPEAT_GL,
            tcuTexture.FilterMode.NEAREST_MIPMAP_LINEAR,
            tcuTexture.FilterMode.LINEAR,
            0,
            true,
            tcuTexture.CompareMode.COMPAREMODE_NONE,
            0,
            [0, 0, 0, 0],
            true);
    };

    /**
    * @param {Array<number>} pos
    * @param {number=} lod
    * @throws {Error}
    */
    sglrReferenceContext.Texture.prototype.sample = function(pos, lod) {throw new Error('Intentionally empty. Call method from child class instead'); };

    /**
    * @param {Array<Array<number>>} packetTexcoords
    * @param {number} lodBias
    * @throws {Error}
    */
    sglrReferenceContext.Texture.prototype.sample4 = function(packetTexcoords, lodBias) {throw new Error('Intentionally empty. Call method from child class instead'); };

    // sglrReferenceContext.Texture.prototype = Object.create(NamedObject.prototype);
    // sglrReferenceContext.Texture.prototype.constructor = sglrReferenceContext.Texture;

    /**
    * @return {number}
    */
    sglrReferenceContext.Texture.prototype.getType = function() { return this.m_type; };

    /**
    * @return {number}
    */
    sglrReferenceContext.Texture.prototype.getBaseLevel = function() { return this.m_baseLevel; };

    /**
    * @return {number}
    */
    sglrReferenceContext.Texture.prototype.getMaxLevel = function() { return this.m_maxLevel; };

    /**
    * @return {boolean}
    */
    sglrReferenceContext.Texture.prototype.isImmutable = function() { return this.m_immutable; };

    /**
    * @param {number} baseLevel
    */
    sglrReferenceContext.Texture.prototype.setBaseLevel = function(baseLevel) { this.m_baseLevel = baseLevel; };

    /**
    * @param {number} maxLevel
    */
    sglrReferenceContext.Texture.prototype.setMaxLevel = function(maxLevel) { this.m_maxLevel = maxLevel; };

    /**
    */
    sglrReferenceContext.Texture.prototype.setImmutable = function() { this.m_immutable = true; };

    /**
    * @return {tcuTexture.Sampler}
    */
    sglrReferenceContext.Texture.prototype.getSampler = function() { return this.m_sampler; };

    /**
    * @constructor
    */
    sglrReferenceContext.TextureLevelArray = function() {
        /** @type {Array<ArrayBuffer>} */ this.m_data = [];
        /** @type {Array<tcuTexture.PixelBufferAccess>} */ this.m_access = [];
    };

    /**
     * @param {number} level
     * @return {boolean}
     */
    sglrReferenceContext.TextureLevelArray.prototype.hasLevel = function(level) { return this.m_data[level] != null; };

    /**
     * @param {number} level
     * @return {tcuTexture.PixelBufferAccess}
     * @throws {Error}
     */
    sglrReferenceContext.TextureLevelArray.prototype.getLevel = function(level) {
        if (!this.hasLevel(level))
            throw new Error('Level: ' + level + ' is not defined.');

        return this.m_access[level];
    };

    /**
     * @return {Array<tcuTexture.PixelBufferAccess>}
     */
    sglrReferenceContext.TextureLevelArray.prototype.getLevels = function() { return this.m_access; };

    /**
     * @param {number} level
     * @param {tcuTexture.TextureFormat} format
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     */
    sglrReferenceContext.TextureLevelArray.prototype.allocLevel = function(level, format, width, height, depth) {
        /** @type {number} */ var dataSize = format.getPixelSize() * width * height * depth;
        if (this.hasLevel(level))
            this.clearLevel(level);

        this.m_data[level] = new ArrayBuffer(dataSize);
        this.m_access[level] = new tcuTexture.PixelBufferAccess({
            format: format,
            width: width,
            height: height,
            depth: depth,
            data: this.m_data[level]});
    };

    /**
     * @param {number} level
     */
    sglrReferenceContext.TextureLevelArray.prototype.clearLevel = function(level) {
        delete this.m_data[level];
        delete this.m_access[level];
    };

    /**
    */
    sglrReferenceContext.TextureLevelArray.prototype.clear = function() {
        for (var key in this.m_data)
            delete this.m_data[key];

        for (var key in this.m_access)
            delete this.m_access[key];
    };

    /**
    * @constructor
    * @extends {sglrReferenceContext.Texture}
    */
    sglrReferenceContext.Texture2D = function() {
        sglrReferenceContext.Texture.call(this, sglrReferenceContext.TextureType.TYPE_2D);
        /** @type {tcuTexture.Texture2DView} */ this.m_view = new tcuTexture.Texture2DView(0, null);
        /** @type {sglrReferenceContext.TextureLevelArray} */ this.m_levels = new sglrReferenceContext.TextureLevelArray();
    };

    /**
    */
    sglrReferenceContext.Texture2D.prototype = Object.create(sglrReferenceContext.Texture.prototype);
    sglrReferenceContext.Texture2D.prototype.constructor = sglrReferenceContext.Texture2D;

    sglrReferenceContext.Texture2D.prototype.clearLevels = function() { this.m_levels.clear(); };

    /**
    * @param {number} level
    * @return {boolean}
    */
    sglrReferenceContext.Texture2D.prototype.hasLevel = function(level) { return this.m_levels.hasLevel(level); };

    /**
    * @param {number} level
    * @return {tcuTexture.PixelBufferAccess}
    */
    sglrReferenceContext.Texture2D.prototype.getLevel = function(level) { return this.m_levels.getLevel(level); };

    /**
    * @param {number} level
    * @param {?tcuTexture.TextureFormat} format
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.Texture2D.prototype.allocLevel = function(level, format, width, height) { this.m_levels.allocLevel(level, format, width, height, 1); };

    /**
     * @return {boolean}
     */
    sglrReferenceContext.Texture2D.prototype.isComplete = function() {
        /** @type {number} */ var baseLevel = this.getBaseLevel();

        if (this.hasLevel(baseLevel)) {
            /** @type {tcuTexture.PixelBufferAccess} */ var level0 = this.getLevel(baseLevel);
            /** @type {boolean} */ var mipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);

            if (mipmap) {
                /** @type {tcuTexture.TextureFormat} */ var format = level0.getFormat();
                /** @type {number} */ var w = level0.getWidth();
                /** @type {number} */ var h = level0.getHeight();
                /** @type {number} */ var numLevels = Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels2D(w, h));

                for (var levelNdx = 1; levelNdx < numLevels; levelNdx++) {
                    if (this.hasLevel(baseLevel + levelNdx)) {
                        /** @type {tcuTexture.PixelBufferAccess} */ var level = this.getLevel(baseLevel + levelNdx);
                        /** @type {number} */ var expectedW = sglrReferenceContext.getMipLevelSize(w, levelNdx);
                        /** @type {number} */ var expectedH = sglrReferenceContext.getMipLevelSize(h, levelNdx);

                        if (level.getWidth() != expectedW ||
                            level.getHeight() != expectedH ||
                            !level.getFormat().isEqual(format))
                            return false;
                    } else
                        return false;
                }
            }

            return true;
        } else
            return false;
    };

    /**
     */
    sglrReferenceContext.Texture2D.prototype.updateView = function() {
        /** @type {number} */ var baseLevel = this.getBaseLevel();

        if (this.hasLevel(baseLevel) && !this.getLevel(baseLevel).isEmpty()) {
            // Update number of levels in mipmap pyramid.
            /** @type {number} */ var width = this.getLevel(baseLevel).getWidth();
            /** @type {number} */ var height = this.getLevel(baseLevel).getHeight();
            /** @type {boolean} */ var isMipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);
            /** @type {number} */ var numLevels = isMipmap ? Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels2D(width, height)) : 1;

            this.m_view = new tcuTexture.Texture2DView(numLevels, this.m_levels.getLevels().slice(baseLevel));
        } else
            this.m_view = new tcuTexture.Texture2DView(0, null);
    };

    /**
     * @param {Array<number>} pos
     * @param {number=} lod
     * @return {Array<number>}
     */
    sglrReferenceContext.Texture2D.prototype.sample = function(pos, lod) {
        return this.m_view.sample(this.getSampler(), pos, lod);
    };

    /**
    * @param {Array<Array<number>>} packetTexcoords 4 vec2 coordinates
    * @param {number} lodBias_
    * @return {Array<Array<number>>} 4 vec4 samples
    */
    sglrReferenceContext.Texture2D.prototype.sample4 = function(packetTexcoords, lodBias_) {
        /** @type {number} */ var lodBias = lodBias_ || 0;
        /** @type {number} */ var texWidth = this.m_view.getWidth();
        /** @type {number} */ var texHeight = this.m_view.getHeight();
        /** @type {Array<Array<number>>}*/ var output = [];

        /** @type {Array<number>}*/ var dFdx0 = deMath.subtract(packetTexcoords[1], packetTexcoords[0]);
        /** @type {Array<number>}*/ var dFdx1 = deMath.subtract(packetTexcoords[3], packetTexcoords[2]);
        /** @type {Array<number>}*/ var dFdy0 = deMath.subtract(packetTexcoords[2], packetTexcoords[0]);
        /** @type {Array<number>}*/ var dFdy1 = deMath.subtract(packetTexcoords[3], packetTexcoords[1]);

        for (var fragNdx = 0; fragNdx < 4; ++fragNdx) {
            /** @type {Array<number>}*/var dFdx = (fragNdx & 2) ? dFdx1 : dFdx0;
            /** @type {Array<number>}*/var dFdy = (fragNdx & 1) ? dFdy1 : dFdy0;

            /** @type {number} */ var mu = Math.max(Math.abs(dFdx[0]), Math.abs(dFdy[0]));
            /** @type {number} */ var mv = Math.max(Math.abs(dFdx[1]), Math.abs(dFdy[1]));
            /** @type {number} */ var p = Math.max(mu * texWidth, mv * texHeight);

            /** @type {number} */ var lod = Math.log2(p) + lodBias;

            output.push(this.sample([packetTexcoords[fragNdx][0], packetTexcoords[fragNdx][1]], lod));
        }

        return output;
    };

    /**
    * @constructor
    * @extends {sglrReferenceContext.Texture}
    */
    sglrReferenceContext.TextureCube = function() {
        sglrReferenceContext.Texture.call(this, sglrReferenceContext.TextureType.TYPE_CUBE_MAP);
        /** @type {tcuTexture.TextureCubeView} */ this.m_view = new tcuTexture.TextureCubeView(0, null);
        /** @type {Array<sglrReferenceContext.TextureLevelArray>} */ this.m_levels = [];
        for (var face in tcuTexture.CubeFace)
            this.m_levels[tcuTexture.CubeFace[face]] = new sglrReferenceContext.TextureLevelArray();
    };

    /**
    */
    sglrReferenceContext.TextureCube.prototype = Object.create(sglrReferenceContext.Texture.prototype);
    sglrReferenceContext.TextureCube.prototype.constructor = sglrReferenceContext.Texture2D;

    sglrReferenceContext.TextureCube.prototype.clearLevels = function() {
        for (var face in tcuTexture.CubeFace)
            this.m_levels[tcuTexture.CubeFace[face]].clear();
    };

    /**
    * @param {number} level
    * @param {tcuTexture.CubeFace} face
    * @return {boolean}
    */
    sglrReferenceContext.TextureCube.prototype.hasFace = function(level, face) { return this.m_levels[face].hasLevel(level); };

    /**
    * @param {number} level
    * @param {tcuTexture.CubeFace} face
    * @return {tcuTexture.PixelBufferAccess}
    */
    sglrReferenceContext.TextureCube.prototype.getFace = function(level, face) { return this.m_levels[face].getLevel(level); };

    /**
    * @param {number} level
    * @param {tcuTexture.CubeFace} face
    * @param {?tcuTexture.TextureFormat} format
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.TextureCube.prototype.allocLevel = function(level, face, format, width, height) {
        this.m_levels[face].allocLevel(level, format, width, height, 1);
    };

    sglrReferenceContext.TextureCube.prototype.isComplete = function() {
        var baseLevel = this.getBaseLevel();

        if (this.hasFace(baseLevel, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X)) {
            var level = this.getFace(baseLevel, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X);
            var width = level.getWidth();
            var height = level.getHeight();
            var format = level.getFormat();
            var mipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);
            var numLevels = mipmap ? Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels2D(width, height)) : 1;

            if (width != height)
                return false; // Non-square is not supported.

            // \note Level 0 is always checked for consistency
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                var levelW = sglrReferenceContext.getMipLevelSize(width, levelNdx);
                var levelH = sglrReferenceContext.getMipLevelSize(height, levelNdx);

                for (var face in tcuTexture.CubeFace) {
                    if (this.hasFace(baseLevel + levelNdx, tcuTexture.CubeFace[face])) {
                        level = this.getFace(baseLevel + levelNdx, tcuTexture.CubeFace[face]);

                        if (level.getWidth() != levelW ||
                            level.getHeight() != levelH ||
                            !level.getFormat().isEqual(format))
                            return false;
                    } else
                        return false;
                }
            }

            return true;
        } else
            return false;
    };

    sglrReferenceContext.TextureCube.prototype.updateView = function() {

        var baseLevel = this.getBaseLevel();
        var faces = [];

        if (this.isComplete()) {
            var size = this.getFace(baseLevel, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X).getWidth();
            var isMipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);
            var numLevels = isMipmap ? Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels1D(size)) : 1;

            for (var face in tcuTexture.CubeFace)
                faces[tcuTexture.CubeFace[face]] = this.m_levels[tcuTexture.CubeFace[face]].getLevels().slice(baseLevel);

            this.m_view = new tcuTexture.TextureCubeView(numLevels, faces);
        } else
            this.m_view = new tcuTexture.TextureCubeView(0, null);
    };

    /**
     * @param {Array<number>} pos
     * @param {number=} lod
     * @return {Array<number>}
     */
    sglrReferenceContext.TextureCube.prototype.sample = function(pos, lod) { return this.m_view.sample(this.getSampler(), pos, lod) };

    /**
    * @constructor
    * @extends {sglrReferenceContext.Texture}
    */
    sglrReferenceContext.Texture2DArray = function() {
        sglrReferenceContext.Texture.call(this, sglrReferenceContext.TextureType.TYPE_2D_ARRAY);
        /** @type {tcuTexture.Texture2DArrayView} */ this.m_view = new tcuTexture.Texture2DArrayView(0, null);
        /** @type {sglrReferenceContext.TextureLevelArray} */ this.m_levels = new sglrReferenceContext.TextureLevelArray();
    };

    /**
    */
    sglrReferenceContext.Texture2DArray.prototype = Object.create(sglrReferenceContext.Texture.prototype);
    sglrReferenceContext.Texture2DArray.prototype.constructor = sglrReferenceContext.Texture2DArray;

    sglrReferenceContext.Texture2DArray.prototype.clearLevels = function() { this.m_levels.clear(); };

    /**
    * @param {number} level
    * @return {boolean}
    */
    sglrReferenceContext.Texture2DArray.prototype.hasLevel = function(level) { return this.m_levels.hasLevel(level); };

    /**
    * @param {number} level
    * @return {tcuTexture.PixelBufferAccess}
    */
    sglrReferenceContext.Texture2DArray.prototype.getLevel = function(level) { return this.m_levels.getLevel(level); };

    /**
    * @param {number} level
    * @param {?tcuTexture.TextureFormat} format
    * @param {number} width
    * @param {number} height
    * @param {number} numLayers
    */
    sglrReferenceContext.Texture2DArray.prototype.allocLevel = function(level, format, width, height, numLayers) {
        this.m_levels.allocLevel(level, format, width, height, numLayers);
    };

    /**
     * @return {boolean}
     */
    sglrReferenceContext.Texture2DArray.prototype.isComplete = function() {
        /** @type {number} */ var baseLevel = this.getBaseLevel();

        if (this.hasLevel(baseLevel)) {
            /** @type {tcuTexture.PixelBufferAccess} */ var level0 = this.getLevel(baseLevel);
            /** @type {boolean} */ var mipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);

            if (mipmap) {
                /** @type {tcuTexture.TextureFormat} */ var format = level0.getFormat();
                /** @type {number} */ var w = level0.getWidth();
                /** @type {number} */ var h = level0.getHeight();
                /** @type {number} */ var numLayers = level0.getDepth();
                /** @type {number} */ var numLevels = Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels2D(w, h));

                for (var levelNdx = 1; levelNdx < numLevels; levelNdx++) {
                    if (this.hasLevel(baseLevel + levelNdx)) {
                        /** @type {tcuTexture.PixelBufferAccess} */ var level = this.getLevel(baseLevel + levelNdx);
                        /** @type {number} */ var expectedW = sglrReferenceContext.getMipLevelSize(w, levelNdx);
                        /** @type {number} */ var expectedH = sglrReferenceContext.getMipLevelSize(h, levelNdx);

                        if (level.getWidth() != expectedW ||
                            level.getHeight() != expectedH ||
                            level.getDepth() != numLayers ||
                            !level.getFormat().isEqual(format))
                            return false;
                    } else
                        return false;
                }
            }

            return true;
        } else
            return false;
    };

    /**
     */
    sglrReferenceContext.Texture2DArray.prototype.updateView = function() {
        /** @type {number} */ var baseLevel = this.getBaseLevel();

        if (this.hasLevel(baseLevel) && !this.getLevel(baseLevel).isEmpty()) {
            // Update number of levels in mipmap pyramid.
            /** @type {number} */ var width = this.getLevel(baseLevel).getWidth();
            /** @type {number} */ var height = this.getLevel(baseLevel).getHeight();
            /** @type {boolean} */ var isMipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);
            /** @type {number} */ var numLevels = isMipmap ? Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels2D(width, height)) : 1;

            this.m_view = new tcuTexture.Texture2DArrayView(numLevels, this.m_levels.getLevels().slice(baseLevel));
        } else
            this.m_view = new tcuTexture.Texture2DArrayView(0, null);
    };

    /**
     * @param {Array<number>} pos
     * @param {number=} lod
     * @return {Array<number>}
     */
    sglrReferenceContext.Texture2DArray.prototype.sample = function(pos, lod) {
        return this.m_view.sample(this.getSampler(), pos, lod);
    };

    /**
    * @constructor
    * @extends {sglrReferenceContext.Texture}
    */
    sglrReferenceContext.Texture3D = function() {
        sglrReferenceContext.Texture.call(this, sglrReferenceContext.TextureType.TYPE_2D_ARRAY);
        /** @type {tcuTexture.Texture3DView} */ this.m_view = new tcuTexture.Texture3DView(0, null);
        /** @type {sglrReferenceContext.TextureLevelArray} */ this.m_levels = new sglrReferenceContext.TextureLevelArray();
    };

    /**
    */
    sglrReferenceContext.Texture3D.prototype = Object.create(sglrReferenceContext.Texture.prototype);
    sglrReferenceContext.Texture3D.prototype.constructor = sglrReferenceContext.Texture3D;

    sglrReferenceContext.Texture3D.prototype.clearLevels = function() { this.m_levels.clear(); };

    /**
    * @param {number} level
    * @return {boolean}
    */
    sglrReferenceContext.Texture3D.prototype.hasLevel = function(level) { return this.m_levels.hasLevel(level); };

    /**
    * @param {number} level
    * @return {tcuTexture.PixelBufferAccess}
    */
    sglrReferenceContext.Texture3D.prototype.getLevel = function(level) { return this.m_levels.getLevel(level); };

    /**
    * @param {number} level
    * @param {?tcuTexture.TextureFormat} format
    * @param {number} width
    * @param {number} height
    * @param {number} depth
    */
    sglrReferenceContext.Texture3D.prototype.allocLevel = function(level, format, width, height, depth) {
        this.m_levels.allocLevel(level, format, width, height, depth);
    };

    /**
     * @return {boolean}
     */
    sglrReferenceContext.Texture3D.prototype.isComplete = function() {
        /** @type {number} */ var baseLevel = this.getBaseLevel();

        if (this.hasLevel(baseLevel)) {
            /** @type {tcuTexture.PixelBufferAccess} */ var level0 = this.getLevel(baseLevel);
            /** @type {boolean} */ var mipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);

            if (mipmap) {
                /** @type {tcuTexture.TextureFormat} */ var format = level0.getFormat();
                /** @type {number} */ var w = level0.getWidth();
                /** @type {number} */ var h = level0.getHeight();
                /** @type {number} */ var d = level0.getDepth();
                /** @type {number} */ var numLevels = Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels3D(w, h, d));

                for (var levelNdx = 1; levelNdx < numLevels; levelNdx++) {
                    if (this.hasLevel(baseLevel + levelNdx)) {
                        /** @type {tcuTexture.PixelBufferAccess} */ var level = this.getLevel(baseLevel + levelNdx);
                        /** @type {number} */ var expectedW = sglrReferenceContext.getMipLevelSize(w, levelNdx);
                        /** @type {number} */ var expectedH = sglrReferenceContext.getMipLevelSize(h, levelNdx);
                        /** @type {number} */ var expectedD = sglrReferenceContext.getMipLevelSize(d, levelNdx);

                        if (level.getWidth() != expectedW ||
                            level.getHeight() != expectedH ||
                            level.getDepth() != expectedD ||
                            !level.getFormat().isEqual(format))
                            return false;
                    } else
                        return false;
                }
            }

            return true;
        } else
            return false;
    };

    /**
     */
    sglrReferenceContext.Texture3D.prototype.updateView = function() {
        /** @type {number} */ var baseLevel = this.getBaseLevel();

        if (this.hasLevel(baseLevel) && !this.getLevel(baseLevel).isEmpty()) {
            // Update number of levels in mipmap pyramid.
            /** @type {number} */ var width = this.getLevel(baseLevel).getWidth();
            /** @type {number} */ var height = this.getLevel(baseLevel).getHeight();
            /** @type {boolean} */ var isMipmap = sglrReferenceContext.isMipmapFilter(this.getSampler().minFilter);
            /** @type {number} */ var numLevels = isMipmap ? Math.min(this.getMaxLevel() - baseLevel + 1, sglrReferenceContext.getNumMipLevels2D(width, height)) : 1;

            this.m_view = new tcuTexture.Texture3DView(numLevels, this.m_levels.getLevels().slice(baseLevel));
        } else
            this.m_view = new tcuTexture.Texture3DView(0, null);
    };

    /**
     * @param {Array<number>} pos
     * @param {number=} lod
     * @return {Array<number>}
     */
    sglrReferenceContext.Texture3D.prototype.sample = function(pos, lod) { return this.m_view.sample(this.getSampler(), pos, lod) };

    /**
    * A container object for storing one of texture types;
    * @constructor
    */
    sglrReferenceContext.TextureContainer = function() {
        /** @type {sglrReferenceContext.Texture2D | sglrReferenceContext.TextureCube|sglrReferenceContext.Texture2DArray|sglrReferenceContext.Texture3D} */
         this.texture = null;
        /** @type {?sglrReferenceContext.TextureType} */ this.textureType = null;
    };

    /**
     * @return {?sglrReferenceContext.TextureType}
     */
    sglrReferenceContext.TextureContainer.prototype.getType = function() { return this.textureType; };

    /**
     * @param {number} target
     * @throws {Error}
     */
    sglrReferenceContext.TextureContainer.prototype.init = function(target) {
        switch (target) {
            case gl.TEXTURE_2D:
                this.texture = new sglrReferenceContext.Texture2D();
                this.textureType = sglrReferenceContext.TextureType.TYPE_2D;
                break;
            case gl.TEXTURE_CUBE_MAP:
                this.texture = new sglrReferenceContext.TextureCube();
                this.textureType = sglrReferenceContext.TextureType.TYPE_CUBE_MAP;
                break;
            case gl.TEXTURE_2D_ARRAY:
                this.texture = new sglrReferenceContext.Texture2DArray();
                this.textureType = sglrReferenceContext.TextureType.TYPE_2D_ARRAY;
                break;
            case gl.TEXTURE_3D:
                this.texture = new sglrReferenceContext.Texture3D();
                this.textureType = sglrReferenceContext.TextureType.TYPE_3D;
                break;
            /* TODO: Implement other types */
            // case gl.TEXTURE_CUBE_MAP_ARRAY:
            //     this.textureType = sglrReferenceContext.TextureType.TYPE_CUBE_MAP_ARRAY;
            //     break;
            default: throw new Error('Unrecognized target: ' + target);
        }
    };

    /**
    * @enum
    */
    sglrReferenceContext.AttachmentPoint = {
        ATTACHMENTPOINT_COLOR0: 0,
        ATTACHMENTPOINT_DEPTH: 1,
        ATTACHMENTPOINT_STENCIL: 2
    };

    /**
     * @param {number} attachment
     * @return {sglrReferenceContext.AttachmentPoint}
     * @throws {Error}
     */
    sglrReferenceContext.mapGLAttachmentPoint = function(attachment) {
        switch (attachment) {
            case gl.COLOR_ATTACHMENT0: return sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_COLOR0;
            case gl.DEPTH_ATTACHMENT: return sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_DEPTH;
            case gl.STENCIL_ATTACHMENT: return sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_STENCIL;
            default: throw new Error('Wrong attachment point:' + attachment);
        }
    };

    /**
    * @enum
    */
    sglrReferenceContext.AttachmentType = {
        ATTACHMENTTYPE_RENDERBUFFER: 0,
        ATTACHMENTTYPE_TEXTURE: 1
    };

    /**
    * @enum
    */
    sglrReferenceContext.TexTarget = {
        TEXTARGET_2D: 0,
        TEXTARGET_CUBE_MAP_POSITIVE_X: 1,
        TEXTARGET_CUBE_MAP_POSITIVE_Y: 2,
        TEXTARGET_CUBE_MAP_POSITIVE_Z: 3,
        TEXTARGET_CUBE_MAP_NEGATIVE_X: 4,
        TEXTARGET_CUBE_MAP_NEGATIVE_Y: 5,
        TEXTARGET_CUBE_MAP_NEGATIVE_Z: 6,
        TEXTARGET_2D_ARRAY: 7,
        TEXTARGET_3D: 8,
        TEXTARGET_CUBE_MAP_ARRAY: 9
    };

    /**
     * @param {?sglrReferenceContext.TexTarget} target
     * @return {tcuTexture.CubeFace}
     */
    sglrReferenceContext.texTargetToFace = function(target) {
        switch (target) {
            case sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_X: return tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X;
            case sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_X: return tcuTexture.CubeFace.CUBEFACE_POSITIVE_X;
            case sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_Y: return tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y;
            case sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_Y: return tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y;
            case sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_Z: return tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z;
            case sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_Z: return tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z;
            default: throw new Error('Invalid target ' + target);
        }
    };

    /**
     * @param {sglrReferenceContext.TexTarget} target
     * @return {sglrReferenceContext.TexTarget}
     * @throws {Error}
     */
    sglrReferenceContext.mapGLFboTexTarget = function(target) {
        switch (target) {
            case gl.TEXTURE_2D: return sglrReferenceContext.TexTarget.TEXTARGET_2D;
            case gl.TEXTURE_CUBE_MAP_POSITIVE_X: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_X;
            case gl.TEXTURE_CUBE_MAP_POSITIVE_Y: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_Y;
            case gl.TEXTURE_CUBE_MAP_POSITIVE_Z: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_Z;
            case gl.TEXTURE_CUBE_MAP_NEGATIVE_X: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_X;
            case gl.TEXTURE_CUBE_MAP_NEGATIVE_Y: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_Y;
            case gl.TEXTURE_CUBE_MAP_NEGATIVE_Z: return sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_Z;
            default: throw new Error('Wrong texture target:' + target);
        }
    };

    /**
    * @constructor
    */
    sglrReferenceContext.Attachment = function() {
        /** @type {?sglrReferenceContext.AttachmentType} */ this.type = null;
        /** @type {sglrReferenceContext.TextureContainer|sglrReferenceContext.Renderbuffer} */ this.object = null; // TODO: fix reserved word
        /** @type {?sglrReferenceContext.TexTarget} */ this.texTarget = null;
        /** @type {number} */ this.level = 0;
        /** @type {number} */ this.layer = 0;
    };

    /**
    * @constructor
    */
    sglrReferenceContext.Framebuffer = function() {
        /** @type {Array<sglrReferenceContext.Attachment>} */ this.m_attachments = [];
        for (var key in sglrReferenceContext.AttachmentPoint)
            this.m_attachments[sglrReferenceContext.AttachmentPoint[key]] = new sglrReferenceContext.Attachment();
    };

    /**
    * @param {sglrReferenceContext.AttachmentPoint} point
    * @return {sglrReferenceContext.Attachment}
    */
    sglrReferenceContext.Framebuffer.prototype.getAttachment = function(point) { return this.m_attachments[point]; };

    /**
    * @param {sglrReferenceContext.AttachmentPoint} point
    * @param {sglrReferenceContext.Attachment} attachment
    */
    sglrReferenceContext.Framebuffer.prototype.setAttachment = function(point, attachment) { this.m_attachments[point] = attachment; };

    // /**
    //  * @enum
    //  */
    // var Format = {
    //     FORMAT_DEPTH_COMPONENT16: 0,
    //     FORMAT_RGBA4: 1,
    //     FORMAT_RGB5_A1: 2,
    //     FORMAT_RGB565: 3,
    //     FORMAT_STENCIL_INDEX8: 4
    // };

    /**
    * @constructor
    */
    sglrReferenceContext.Renderbuffer = function() {
        /** @type {tcuTexture.TextureLevel} */ this.m_data;
    };

    /**
    * @param {tcuTexture.TextureFormat} format
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.Renderbuffer.prototype.setStorage = function(format, width, height) {
        this.m_data = new tcuTexture.TextureLevel(format, width, height);
    };

    /**
     * @return {number}
     */
    sglrReferenceContext.Renderbuffer.prototype.getWidth = function() { return this.m_data.getWidth(); };

    /**
     * @return {number}
     */
    sglrReferenceContext.Renderbuffer.prototype.getHeight = function() { return this.m_data.getHeight(); };

    /**
     * @return {?tcuTexture.TextureFormat}
     */
    sglrReferenceContext.Renderbuffer.prototype.getFormat = function() { return this.m_data.getFormat(); };

    /**
     * @return {tcuTexture.PixelBufferAccess}
     */
    sglrReferenceContext.Renderbuffer.prototype.getAccess = function() { return this.m_data.getAccess(); };

    /**
     * @constructor
     * @param {number} maxVertexAttribs
     */
    sglrReferenceContext.VertexArray = function(maxVertexAttribs) {
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_elementArrayBufferBinding = null;

        /** @type {Array<sglrReferenceContext.VertexArray.VertexAttribArray>} */this.m_arrays = [];
        for (var i = 0; i < maxVertexAttribs; i++)
            this.m_arrays.push(new sglrReferenceContext.VertexArray.VertexAttribArray());
    };

    /** @constructor */
    sglrReferenceContext.VertexArray.VertexAttribArray = function() {
        this.enabled = false;
        this.size = 4;
        this.stride = 0;
        this.type = gl.FLOAT;

        this.normalized = false;
        this.integer = false;
        this.divisor = 0;
        this.offset = 0;
        this.bufferBinding = null;
    };

    /**
    * @constructor
    */
    sglrReferenceContext.DataBuffer = function() {
        /** @type {?ArrayBuffer} */ this.m_data = null;
    };

    /**
     * @param {number} size
     */
    sglrReferenceContext.DataBuffer.prototype.setStorage = function(size) {this.m_data = new ArrayBuffer(size); };

    /**
     * @return {number}
     */
    sglrReferenceContext.DataBuffer.prototype.getSize = function() {
        /** @type {number} */ var size = 0;
        if (this.m_data)
            size = this.m_data.byteLength;
        return size;
    };

    /**
     * @return {?ArrayBuffer}
     */
    sglrReferenceContext.DataBuffer.prototype.getData = function() { return this.m_data; };

    /**
     * @param {ArrayBuffer|goog.NumberArray} data
     */
    sglrReferenceContext.DataBuffer.prototype.setData = function(data) {
        /** @type {ArrayBuffer} */ var buffer;
        /** @type {number} */ var offset = 0;
        /** @type {number} */ var byteLength = data.byteLength;
        if (data instanceof ArrayBuffer)
            buffer = data;
        else {
            buffer = data.buffer;
            offset = data.byteOffset;
        }

        if (!buffer)
            throw new Error('Invalid buffer');

        this.m_data = buffer.slice(offset, offset + byteLength);
    };

    /**
     * @param {number} offset
     * @param {goog.NumberArray} data
     */
    sglrReferenceContext.DataBuffer.prototype.setSubData = function(offset, data) {
        /** @type {ArrayBuffer} */ var buffer;
        /** @type {number} */ var srcOffset = 0;
        /** @type {number} */ var byteLength = data.byteLength;
        if (data instanceof ArrayBuffer)
            buffer = data;
        else {
            buffer = data.buffer;
            srcOffset = data.byteOffset;
        }

        if (!buffer)
            throw new Error('Invalid buffer');

        /** @type {goog.NumberArray} */ var src = new Uint8Array(buffer, srcOffset, byteLength);
        /** @type {goog.NumberArray} */ var dst = new Uint8Array(this.m_data, offset, byteLength);
        dst.set(src);
    };

    // /**
    //  * @constructor
    //  */
    // var ObjectManager = function() {
    //     this.m_objects = {};
    // };

    // ObjectManager.prototype.insert = function(obj) {
    //     var name = obj.getName();
    //     if (!name)
    //         throw new Error("Cannot insert unnamed object");
    //     this.m_objects[name] = obj;
    // };

    // ObjectManager.prototype.find = function(name) { return this.m_objects[name]; };

    // ObjectManager.prototype.acquireReference = function(obj) {
    //     if (this.find(obj.getName()) !== obj)
    //         throw new Error("Object is not in the object manager");
    //     obj.incRefCount();
    // };

    // ObjectManager.prototype.releaseReference = function(obj) {
    //     if (this.find(obj.getName()) !== obj)
    //         throw new Error("Object is not in the object manager");

    //     obj.decRefCount();

    //     if (obj.getRefCount() == 0)
    //         delete this.m_objects[obj.getName()];
    // };

    // ObjectManager.prototype.getAll = function() { return this.m_objects; };

    /**
    * @constructor
    */
    sglrReferenceContext.TextureUnit = function() {
        /** @type {?sglrReferenceContext.TextureContainer} */ this.tex2DBinding = null;
        /** @type {?sglrReferenceContext.TextureContainer} */ this.texCubeBinding = null;
        /** @type {?sglrReferenceContext.TextureContainer} */ this.tex2DArrayBinding = null;
        /** @type {?sglrReferenceContext.TextureContainer} */ this.tex3DBinding = null;
        /** @type {?sglrReferenceContext.TextureContainer} */ this.texCubeArrayBinding = null;
    };

    /**
    * @constructor
    */
    sglrReferenceContext.StencilState = function() {
        /** @type {number} */ this.func = gl.ALWAYS;
        /** @type {number} */ this.ref = 0;
        /** @type {number} */ this.opMask = ~0;
        /** @type {number} */ this.opStencilFail = gl.KEEP;
        /** @type {number} */ this.opDepthFail = gl.KEEP;
        /** @type {number} */ this.opDepthPass = gl.KEEP;
        /** @type {number} */ this.writeMask = ~0;
    };

    /**
     * @param {tcuPixelFormat.PixelFormat} pixelFmt
     * @return {tcuTexture.TextureFormat}
     * @throws {Error}
     */
    sglrReferenceContext.toTextureFormat = function(pixelFmt) {
        if (pixelFmt.equals(8, 8, 8, 8))
            return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);
        else if (pixelFmt.equals(8, 8, 8, 0))
            return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT8);
        else if (pixelFmt.equals(4, 4, 4, 4))
            return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_4444);
        else if (pixelFmt.equals(5, 5, 5, 1))
            return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_SHORT_5551);
        else if (pixelFmt.equals(5, 6, 5, 0))
            return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_SHORT_565);

        throw new Error('Could not map pixel format:' + pixelFmt);
    };

    /**
     * @param {number} depthBits
     * @return {tcuTexture.TextureFormat}
     * @throws {Error}
     */
    sglrReferenceContext.getDepthFormat = function(depthBits) {
        switch (depthBits) {
            case 8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNORM_INT8);
            case 16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNORM_INT16);
            case 24: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.UNSIGNED_INT_24_8);
            case 32: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.D, tcuTexture.ChannelType.FLOAT);
            default:
                throw new Error("Can't map depth buffer format, bits: " + depthBits);
        }
    };

    /**
     * @param {number} stencilBits
     * @return {tcuTexture.TextureFormat}
     * @throws {Error}
     */
    sglrReferenceContext.getStencilFormat = function(stencilBits) {
        switch (stencilBits) {
            case 8: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.S, tcuTexture.ChannelType.UNSIGNED_INT8);
            case 16: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.S, tcuTexture.ChannelType.UNSIGNED_INT16);
            case 24: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.S, tcuTexture.ChannelType.UNSIGNED_INT_24_8);
            case 32: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.S, tcuTexture.ChannelType.UNSIGNED_INT32);
            default:
                throw new Error("Can't map stencil buffer format, bits: " + stencilBits);
        }
    };

    /**
    * @constructor
    * @param {tcuPixelFormat.PixelFormat} colorBits
    * @param {number} depthBits
    * @param {number} stencilBits
    * @param {number} width
    * @param {number} height
    * @param {number=} samples_
    */
    sglrReferenceContext.ReferenceContextBuffers = function(colorBits, depthBits, stencilBits, width, height, samples_) {
        if (samples_ === undefined)
            samples_ = 1;

        /** @type {number} */ var samples = samples_;
        /** @type {tcuTexture.TextureLevel} */ this.m_colorbuffer = new tcuTexture.TextureLevel(sglrReferenceContext.toTextureFormat(colorBits), samples, width, height);

        if (depthBits > 0)
            /** @type {tcuTexture.TextureLevel} */ this.m_depthbuffer = new tcuTexture.TextureLevel(sglrReferenceContext.getDepthFormat(depthBits), samples, width, height);

        if (stencilBits > 0)
            /** @type {tcuTexture.TextureLevel} */ this.m_stencilbuffer = new tcuTexture.TextureLevel(sglrReferenceContext.getStencilFormat(stencilBits), samples, width, height);
    };

    /**
     * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
     */
    sglrReferenceContext.ReferenceContextBuffers.prototype.getColorbuffer = function() { return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromMultisampleAccess(this.m_colorbuffer.getAccess()); };

    /**
     * @return {?rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
     */
    sglrReferenceContext.ReferenceContextBuffers.prototype.getDepthbuffer = function() { return this.m_depthbuffer !== undefined ? rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromMultisampleAccess(this.m_depthbuffer.getAccess()) : null; };

    /**
     * @return {?rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
     */
    sglrReferenceContext.ReferenceContextBuffers.prototype.getStencilbuffer = function() { return this.m_stencilbuffer !== undefined ? rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromMultisampleAccess(this.m_stencilbuffer.getAccess()) : null; };

    /**
    * @param {sglrReferenceContext.ReferenceContextLimits} limits
    * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} colorbuffer
    * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} depthbuffer
    * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} stencilbuffer
    * @constructor
    */
    sglrReferenceContext.ReferenceContext = function(limits, colorbuffer, depthbuffer, stencilbuffer) {
        /** @type {sglrReferenceContext.ReferenceContextLimits} */ this.m_limits = limits;
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ this.m_defaultColorbuffer = colorbuffer;
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ this.m_defaultDepthbuffer = depthbuffer;
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ this.m_defaultStencilbuffer = stencilbuffer;
        /** @type {Array<number>} */ this.m_viewport = [0, 0, colorbuffer.raw().getHeight(), colorbuffer.raw().getDepth()];
        /** @type {Array<sglrReferenceContext.TextureUnit>} */ this.m_textureUnits = [];
        for (var i = 0; i < this.m_limits.maxTextureImageUnits; i++)
            this.m_textureUnits.push(new sglrReferenceContext.TextureUnit());
        /** @type {number} */ this.m_activeTexture = 0;
        /** @type {number} */ this.m_lastError = gl.NO_ERROR;
        // this.m_textures = new ObjectManager();
        /** @type {number} */ this.m_pixelUnpackRowLength = 0;
        /** @type {number} */ this.m_pixelUnpackSkipRows = 0;
        /** @type {number} */ this.m_pixelUnpackSkipPixels = 0;
        /** @type {number} */ this.m_pixelUnpackImageHeight = 0;
        /** @type {number} */ this.m_pixelUnpackSkipImages = 0;
        /** @type {number} */ this.m_pixelUnpackAlignment = 4;
        /** @type {number} */ this.m_pixelPackAlignment = 4;
        /** @type {Array<number>} */ this.m_clearColor = [0, 0, 0, 0];
        /** @type {number} */ this.m_clearDepth = 1;
        /** @type {number} */ this.m_clearStencil = 0;
        /** @type {Array<number>} */ this.m_scissorBox = this.m_viewport;
        /** @type {boolean} */ this.m_blendEnabled = false;
        /** @type {boolean} */ this.m_scissorEnabled = false;
        /** @type {boolean} */ this.m_depthTestEnabled = false;
        /** @type {boolean} */ this.m_stencilTestEnabled = false;
        /** @type {boolean} */ this.m_polygonOffsetFillEnabled = false;
        /** @type {boolean} */ this.m_primitiveRestartFixedIndex = true; //always on
        /** @type {boolean} */ this.m_primitiveRestartSettableIndex = true; //always on
        /** @type {Array<sglrReferenceContext.StencilState>} */ this.m_stencil = [];
        for (var type in rrDefs.FaceType)
            this.m_stencil[rrDefs.FaceType[type]] = new sglrReferenceContext.StencilState();
        /** @type {number} */ this.m_depthFunc = gl.LESS;
        /** @type {number} */ this.m_depthRangeNear = 0;
        /** @type {number} */ this.m_depthRangeFar = 1;
        /** @type {number} */ this.m_polygonOffsetFactor = 0;
        /** @type {number} */ this.m_polygonOffsetUnits = 0;
        /** @type {number} */ this.m_blendModeRGB = gl.FUNC_ADD;
        /** @type {number} */ this.m_blendModeAlpha = gl.FUNC_ADD;
        /** @type {number} */ this.m_blendFactorSrcRGB = gl.ONE;
        /** @type {number} */ this.m_blendFactorDstRGB = gl.ZERO;
        /** @type {number} */ this.m_blendFactorSrcAlpha = gl.ONE;
        /** @type {number} */ this.m_blendFactorDstAlpha = gl.ZERO;
        /** @type {Array<number>} */ this.m_blendColor = [0, 0, 0, 0];
        /** @type {boolean} */ this.m_sRGBUpdateEnabled = true;
        /** @type {Array<boolean>} */ this.m_colorMask = [true, true, true, true];
        /** @type {boolean} */ this.m_depthMask = true;
        /** @type {sglrReferenceContext.VertexArray} */ this.m_defaultVAO = new sglrReferenceContext.VertexArray(this.m_limits.maxVertexAttribs);
        /** @type {sglrReferenceContext.VertexArray} */ this.m_vertexArrayBinding = this.m_defaultVAO;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_arrayBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_copyReadBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_copyWriteBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_drawIndirectBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_pixelPackBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_pixelUnpackBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_transformFeedbackBufferBinding = null;
        /** @type {sglrReferenceContext.DataBuffer} */ this.m_uniformBufferBinding = null;
        /** @type {sglrReferenceContext.Framebuffer} */ this.m_readFramebufferBinding = null;
        /** @type {sglrReferenceContext.Framebuffer} */ this.m_drawFramebufferBinding = null;
        /** @type {sglrReferenceContext.Renderbuffer} */ this.m_renderbufferBinding = null;
        /** @type {sglrShaderProgram.ShaderProgram} */ this.m_currentProgram = null;
        /** @type {Array<rrGenericVector.GenericVec4>} */ this.m_currentAttribs = [];
        for (var i = 0; i < this.m_limits.maxVertexAttribs; i++)
            this.m_currentAttribs.push(new rrGenericVector.GenericVec4());
        /** @type {number} */ this.m_lineWidth = 1;

        /** @type {sglrReferenceContext.TextureContainer} */ this.m_emptyTex2D = new sglrReferenceContext.TextureContainer();
        this.m_emptyTex2D.init(gl.TEXTURE_2D);
        this.m_emptyTex2D.texture.getSampler().wrapS = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex2D.texture.getSampler().wrapT = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex2D.texture.getSampler().minFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTex2D.texture.getSampler().magFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTex2D.texture.allocLevel(0, new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8), 1, 1);
        this.m_emptyTex2D.texture.getLevel(0).setPixel([0, 0, 0, 1], 0, 0);
        this.m_emptyTex2D.texture.updateView();

        /** @type {sglrReferenceContext.TextureContainer} */ this.m_emptyTexCube = new sglrReferenceContext.TextureContainer();
        this.m_emptyTexCube.init(gl.TEXTURE_CUBE_MAP);
        this.m_emptyTexCube.texture.getSampler().wrapS = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTexCube.texture.getSampler().wrapT = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTexCube.texture.getSampler().minFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTexCube.texture.getSampler().magFilter = tcuTexture.FilterMode.NEAREST;

        for (var face in tcuTexture.CubeFace) {
            this.m_emptyTexCube.texture.allocLevel(0, tcuTexture.CubeFace[face],
                new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8), 1, 1);
            this.m_emptyTexCube.texture.getFace(0, tcuTexture.CubeFace[face]).setPixel([0, 0, 0, 1], 0, 0);
        }
        this.m_emptyTexCube.texture.updateView();

        /** @type {sglrReferenceContext.TextureContainer} */ this.m_emptyTex2DArray = new sglrReferenceContext.TextureContainer();
        this.m_emptyTex2DArray.init(gl.TEXTURE_2D_ARRAY);
        this.m_emptyTex2DArray.texture.getSampler().wrapS = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex2DArray.texture.getSampler().wrapT = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex2DArray.texture.getSampler().wrapR = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex2DArray.texture.getSampler().minFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTex2DArray.texture.getSampler().magFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTex2DArray.texture.allocLevel(0, new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8), 1, 1);
        this.m_emptyTex2DArray.texture.getLevel(0).setPixel([0, 0, 0, 1], 0, 0);
        this.m_emptyTex2DArray.texture.updateView();

        /** @type {sglrReferenceContext.TextureContainer} */ this.m_emptyTex3D = new sglrReferenceContext.TextureContainer();
        this.m_emptyTex3D.init(gl.TEXTURE_3D);
        this.m_emptyTex3D.texture.getSampler().wrapS = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex3D.texture.getSampler().wrapT = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex3D.texture.getSampler().wrapR = tcuTexture.WrapMode.CLAMP_TO_EDGE;
        this.m_emptyTex3D.texture.getSampler().minFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTex3D.texture.getSampler().magFilter = tcuTexture.FilterMode.NEAREST;
        this.m_emptyTex3D.texture.allocLevel(0, new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8), 1, 1);
        this.m_emptyTex3D.texture.getLevel(0).setPixel([0, 0, 0, 1], 0, 0);
        this.m_emptyTex3D.texture.updateView();

        /** @type {sglrReferenceContext.TextureType} */ this.m_type;

        /** @type {boolean} */ this.m_immutable;

        /** @type {tcuTexture.Sampler} */ this.m_sampler;
        /** @type {number} */ this.m_baseLevel;
        /** @type {number} */ this.m_maxLevel;
    };

    /**
     * @return {number}
     */
    sglrReferenceContext.ReferenceContext.prototype.getWidth = function() { return this.m_defaultColorbuffer.raw().getHeight(); };

    /**
    * @return {number}
    */
    sglrReferenceContext.ReferenceContext.prototype.getHeight = function() { return this.m_defaultColorbuffer.raw().getDepth(); };

    /**
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.ReferenceContext.prototype.viewport = function(x, y, width, height) { this.m_viewport = [x, y, width, height]; };

    /**
    * @param {number} texture
    */
    sglrReferenceContext.ReferenceContext.prototype.activeTexture = function(texture) {
        if (deMath.deInBounds32(texture, gl.TEXTURE0, gl.TEXTURE0 + this.m_textureUnits.length))
            this.m_activeTexture = texture - gl.TEXTURE0;
        else
            this.setError(gl.INVALID_ENUM);
    };

    /**
    * @param {number} error
    */
    sglrReferenceContext.ReferenceContext.prototype.setError = function(error) {
        if (this.m_lastError == gl.NO_ERROR)
            this.m_lastError = error;
    };

    /**
    * @return {number} error
    */
    sglrReferenceContext.ReferenceContext.prototype.getError = function() {
        /** @type {number} */ var err = this.m_lastError;
        this.m_lastError = gl.NO_ERROR;
        return err;
    };

    /**
     * @param {boolean} condition
     * @param {number} error
     */
    sglrReferenceContext.ReferenceContext.prototype.conditionalSetError = function(condition, error) {
        if (condition)
            this.setError(error);
        return condition;
    };

    /**
    * @param {number} target
    * @param {sglrReferenceContext.TextureContainer} texture
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.bindTexture = function(target, texture) {
        /** @type {number} */ var unitNdx = this.m_activeTexture;

        if (this.conditionalSetError((target != gl.TEXTURE_2D &&
                    target != gl.TEXTURE_CUBE_MAP &&
                    target != gl.TEXTURE_2D_ARRAY &&
                    target != gl.TEXTURE_3D), // &&
                    // target != gl.TEXTURE_CUBE_MAP_ARRAY),
                    gl.INVALID_ENUM))
            return;

        if (!texture) {
            // Clear binding.
            switch (target) {
                case gl.TEXTURE_2D: this.setTex2DBinding(unitNdx, null); break;
                case gl.TEXTURE_CUBE_MAP: this.setTexCubeBinding(unitNdx, null); break;
                case gl.TEXTURE_2D_ARRAY: this.setTex2DArrayBinding(unitNdx, null); break;
                case gl.TEXTURE_3D: this.setTex3DBinding(unitNdx, null); break;
                default:
                    throw new Error('Unrecognized target: ' + target);
            }
        } else {
            if (texture.textureType == null) {
                texture.init(target);
            } else {
                // Validate type.
                /** @type {sglrReferenceContext.TextureType} */ var expectedType;
                switch (target) {
                    case gl.TEXTURE_2D: expectedType = sglrReferenceContext.TextureType.TYPE_2D; break;
                    case gl.TEXTURE_CUBE_MAP: expectedType = sglrReferenceContext.TextureType.TYPE_CUBE_MAP; break;
                    case gl.TEXTURE_2D_ARRAY: expectedType = sglrReferenceContext.TextureType.TYPE_2D_ARRAY; break;
                    case gl.TEXTURE_3D: expectedType = sglrReferenceContext.TextureType.TYPE_3D; break;
                    default: throw new Error('Unrecognized target: ' + target);
                }
                if (this.conditionalSetError((texture.textureType != expectedType), gl.INVALID_OPERATION))
                    return;
            }
            switch (target) {
                case gl.TEXTURE_2D: this.setTex2DBinding(unitNdx, texture); break;
                case gl.TEXTURE_CUBE_MAP: this.setTexCubeBinding(unitNdx, texture); break;
                case gl.TEXTURE_2D_ARRAY: this.setTex2DArrayBinding(unitNdx, texture); break;
                case gl.TEXTURE_3D: this.setTex3DBinding(unitNdx, texture); break;
                default:
                    throw new Error('Unrecognized target: ' + target);
            }
        }
    };

    /**
    * @param {number} unitNdx
    * @param {?sglrReferenceContext.TextureContainer} texture
    */
    sglrReferenceContext.ReferenceContext.prototype.setTexCubeBinding = function(unitNdx, texture) {
        if (this.m_textureUnits[unitNdx].texCubeBinding) {
            this.m_textureUnits[unitNdx].texCubeBinding = null;
        }

        if (texture) {
            this.m_textureUnits[unitNdx].texCubeBinding = texture;
        }
    };

    /**
    * @param {number} unitNdx
    * @param {?sglrReferenceContext.TextureContainer} texture
    */
    sglrReferenceContext.ReferenceContext.prototype.setTex2DBinding = function(unitNdx, texture) {
        if (this.m_textureUnits[unitNdx].tex2DBinding) {
            // this.m_textures.releaseReference(this.m_textureUnits[unitNdx].tex2DBinding);
            this.m_textureUnits[unitNdx].tex2DBinding = null;
        }

        if (texture) {
            // this.m_textures.acquireReference(texture);
            this.m_textureUnits[unitNdx].tex2DBinding = texture;
        }
    };

    /**
    * @param {number} unitNdx
    * @param {?sglrReferenceContext.TextureContainer} texture
    */
    sglrReferenceContext.ReferenceContext.prototype.setTex2DArrayBinding = function(unitNdx, texture) {
        if (this.m_textureUnits[unitNdx].tex2DArrayBinding) {
            // this.m_textures.releaseReference(this.m_textureUnits[unitNdx].tex2DArrayBinding);
            this.m_textureUnits[unitNdx].tex2DArrayBinding = null;
        }

        if (texture) {
            // this.m_textures.acquireReference(texture);
            this.m_textureUnits[unitNdx].tex2DArrayBinding = texture;
        }
    };

    /**
    * @param {number} unitNdx
    * @param {?sglrReferenceContext.TextureContainer} texture
    */
    sglrReferenceContext.ReferenceContext.prototype.setTex3DBinding = function(unitNdx, texture) {
        if (this.m_textureUnits[unitNdx].tex3DBinding) {
            // this.m_textures.releaseReference(this.m_textureUnits[unitNdx].tex3DBinding);
            this.m_textureUnits[unitNdx].tex3DBinding = null;
        }

        if (texture) {
            // this.m_textures.acquireReference(texture);
            this.m_textureUnits[unitNdx].tex3DBinding = texture;
        }
    };

    /**
    * @return {sglrReferenceContext.TextureContainer}
    */
    sglrReferenceContext.ReferenceContext.prototype.createTexture = function() { return new sglrReferenceContext.TextureContainer(); };

    /**
    * @param {sglrReferenceContext.Texture} texture
    */
    sglrReferenceContext.ReferenceContext.prototype.deleteTexture = function(texture) { /*empty*/ };

    /**
    * @param {number} target
    * @param {framework.opengl.simplereference.sglrReferenceContext.Framebuffer} fbo
    */
    sglrReferenceContext.ReferenceContext.prototype.bindFramebuffer = function(target, fbo) {
        if (this.conditionalSetError((target != gl.FRAMEBUFFER &&
                    target != gl.DRAW_FRAMEBUFFER &&
                    target != gl.READ_FRAMEBUFFER), gl.INVALID_ENUM))
                    return;
        for (var ndx = 0; ndx < 2; ndx++) {
            /** @type {number} */ var bindingTarget = ndx ? gl.DRAW_FRAMEBUFFER : gl.READ_FRAMEBUFFER;

            if (target != gl.FRAMEBUFFER && target != bindingTarget)
                continue; // Doesn't match this target.

            if (ndx)
                this.m_drawFramebufferBinding = fbo;
            else
                this.m_readFramebufferBinding = fbo;
        }
    };

    /**
    * @return {sglrReferenceContext.Framebuffer}
    */
    sglrReferenceContext.ReferenceContext.prototype.createFramebuffer = function() { return new sglrReferenceContext.Framebuffer(); };

    /**
    * @param {sglrReferenceContext.Framebuffer} fbo
    */
    sglrReferenceContext.ReferenceContext.prototype.deleteFramebuffer = function(fbo) { /*empty*/ };

    /**
    * @param {number} target
    * @param {sglrReferenceContext.Renderbuffer} rbo
    */
    sglrReferenceContext.ReferenceContext.prototype.bindRenderbuffer = function(target, rbo) {
        if (this.conditionalSetError(target != gl.RENDERBUFFER, gl.INVALID_ENUM))
            return;

        this.m_renderbufferBinding = rbo;
    };

    /**
    * @return {sglrReferenceContext.Renderbuffer}
    */
    sglrReferenceContext.ReferenceContext.prototype.createRenderbuffer = function() { return new sglrReferenceContext.Renderbuffer(); };

    /**
    * @param {sglrReferenceContext.Renderbuffer} rbo
    */
    sglrReferenceContext.ReferenceContext.prototype.deleteRenderbuffer = function(rbo) { /*empty*/ };

    /**
    * @param {number} pname
    * @param {number} param
    */
    sglrReferenceContext.ReferenceContext.prototype.pixelStorei = function(pname, param) {
        switch (pname) {
            case gl.UNPACK_ALIGNMENT:
                if (this.conditionalSetError((param != 1 && param != 2 && param != 4 && param != 8), gl.INVALID_VALUE)) return;
                this.m_pixelUnpackAlignment = param;
                break;

            case gl.PACK_ALIGNMENT:
                if (this.conditionalSetError((param != 1 && param != 2 && param != 4 && param != 8), gl.INVALID_VALUE)) return;
                this.m_pixelPackAlignment = param;
                break;

            case gl.UNPACK_ROW_LENGTH:
                if (this.conditionalSetError(param < 0, gl.INVALID_VALUE)) return;
                this.m_pixelUnpackRowLength = param;
                break;

            case gl.UNPACK_SKIP_ROWS:
                if (this.conditionalSetError(param < 0, gl.INVALID_VALUE)) return;
                this.m_pixelUnpackSkipRows = param;
                break;

            case gl.UNPACK_SKIP_PIXELS:
                if (this.conditionalSetError(param < 0, gl.INVALID_VALUE)) return;
                this.m_pixelUnpackSkipPixels = param;
                break;

            case gl.UNPACK_IMAGE_HEIGHT:
                if (this.conditionalSetError(param < 0, gl.INVALID_VALUE)) return;
                this.m_pixelUnpackImageHeight = param;
                break;

            case gl.UNPACK_SKIP_IMAGES:
                if (this.conditionalSetError(param < 0, gl.INVALID_VALUE)) return;
                this.m_pixelUnpackSkipImages = param;
                break;

            default:
                this.setError(gl.INVALID_ENUM);
        }
    };

    /**
    * @param {number} red
    * @param {number} green
    * @param {number} blue
    * @param {number} alpha
    */
    sglrReferenceContext.ReferenceContext.prototype.clearColor = function(red, green, blue, alpha) {
        this.m_clearColor = [deMath.clamp(red, 0, 1),
                            deMath.clamp(green, 0, 1),
                            deMath.clamp(blue, 0, 1),
                            deMath.clamp(alpha, 0, 1)];
    };

    /**
    * @param {number} depth
    */
    sglrReferenceContext.ReferenceContext.prototype.clearDepthf = function(depth) {
        this.m_clearDepth = deMath.clamp(depth, 0, 1);
    };

    /**
    * @param {number} stencil
    */
    sglrReferenceContext.ReferenceContext.prototype.clearStencil = function(stencil) {
        this.m_clearStencil = stencil;
    };

    /**
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.ReferenceContext.prototype.scissor = function(x, y, width, height) {
        if (this.conditionalSetError(width < 0 || height < 0, gl.INVALID_VALUE))
            return;
        this.m_scissorBox = [x, y, width, height];
    };

    /**
    * @param {number} cap
    */
    sglrReferenceContext.ReferenceContext.prototype.enable = function(cap) {
        switch (cap) {
            case gl.BLEND: this.m_blendEnabled = true; break;
            case gl.SCISSOR_TEST: this.m_scissorEnabled = true; break;
            case gl.DEPTH_TEST: this.m_depthTestEnabled = true; break;
            case gl.STENCIL_TEST: this.m_stencilTestEnabled = true; break;
            case gl.POLYGON_OFFSET_FILL: this.m_polygonOffsetFillEnabled = true; break;

            case gl.DITHER:
                // Not implemented - just ignored.
                break;

            default:
                this.setError(gl.INVALID_ENUM);
                break;
        }
    };

    /**
    * @param {number} cap
    */
    sglrReferenceContext.ReferenceContext.prototype.disable = function(cap) {
        switch (cap) {
            case gl.BLEND: this.m_blendEnabled = false; break;
            case gl.SCISSOR_TEST: this.m_scissorEnabled = false; break;
            case gl.DEPTH_TEST: this.m_depthTestEnabled = false; break;
            case gl.STENCIL_TEST: this.m_stencilTestEnabled = false; break;
            case gl.POLYGON_OFFSET_FILL: this.m_polygonOffsetFillEnabled = false; break;

            case gl.DITHER:
                // Not implemented - just ignored.
                break;

            default:
                this.setError(gl.INVALID_ENUM);
                break;
        }
    };

    /**
    * @param {number} func
    * @param {number} ref
    * @param {number} mask
    */
    sglrReferenceContext.ReferenceContext.prototype.stencilFunc = function(func, ref, mask) {
        this.stencilFuncSeparate(gl.FRONT_AND_BACK, func, ref, mask);
    };

    /**
    * @param {number} face
    * @param {number} func
    * @param {number} ref
    * @param {number} mask
    */
    sglrReferenceContext.ReferenceContext.prototype.stencilFuncSeparate = function(face, func, ref, mask) {
        /** @type {boolean} */ var setFront = face == gl.FRONT || face == gl.FRONT_AND_BACK;
        /** @type {boolean} */ var setBack = face == gl.BACK || face == gl.FRONT_AND_BACK;

        if (this.conditionalSetError(!sglrReferenceContext.isValidCompareFunc(func), gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(!setFront && !setBack, gl.INVALID_ENUM))
            return;

        for (var key in rrDefs.FaceType) {
            /** @type {number} */ var type = rrDefs.FaceType[key];
            if ((type == rrDefs.FaceType.FACETYPE_FRONT && setFront) ||
                (type == rrDefs.FaceType.FACETYPE_BACK && setBack)) {
                this.m_stencil[type].func = func;
                this.m_stencil[type].ref = ref;
                this.m_stencil[type].opMask = mask;
            }
        }
    };

    /**
    * @param {number} func
    * @return {boolean}
    */
    sglrReferenceContext.isValidCompareFunc = function(func) {
        switch (func) {
            case gl.NEVER:
            case gl.LESS:
            case gl.LEQUAL:
            case gl.GREATER:
            case gl.GEQUAL:
            case gl.EQUAL:
            case gl.NOTEQUAL:
            case gl.ALWAYS:
                return true;

            default:
                return false;
        }
    };

    /**
    * @param {number} op
    * @return {boolean}
    */
    sglrReferenceContext.isValidStencilOp = function(op) {
        switch (op) {
            case gl.KEEP:
            case gl.ZERO:
            case gl.REPLACE:
            case gl.INCR:
            case gl.INCR_WRAP:
            case gl.DECR:
            case gl.DECR_WRAP:
            case gl.INVERT:
                return true;

            default:
                return false;
        }
    };

    /**
    * @param {number} sfail
    * @param {number} dpfail
    * @param {number} dppass
    */
    sglrReferenceContext.ReferenceContext.prototype.stencilOp = function(sfail, dpfail, dppass) {
        this.stencilOpSeparate(gl.FRONT_AND_BACK, sfail, dpfail, dppass);
    };

    /**
    * @param {number} face
    * @param {number} sfail
    * @param {number} dpfail
    * @param {number} dppass
    */
    sglrReferenceContext.ReferenceContext.prototype.stencilOpSeparate = function(face, sfail, dpfail, dppass) {
        /** @type {boolean} */ var setFront = face == gl.FRONT || face == gl.FRONT_AND_BACK;
        /** @type {boolean} */ var setBack = face == gl.BACK || face == gl.FRONT_AND_BACK;

        if (this.conditionalSetError((!sglrReferenceContext.isValidStencilOp(sfail) ||
                    !sglrReferenceContext.isValidStencilOp(dpfail) ||
                    !sglrReferenceContext.isValidStencilOp(dppass)),
                    gl.INVALID_ENUM))
            return;

        if (this.conditionalSetError(!setFront && !setBack, gl.INVALID_ENUM))
            return;

    for (var key in rrDefs.FaceType) {
            /** @type {number} */ var type = rrDefs.FaceType[key];
            if ((type == rrDefs.FaceType.FACETYPE_FRONT && setFront) ||
                (type == rrDefs.FaceType.FACETYPE_BACK && setBack)) {
                this.m_stencil[type].opStencilFail = sfail;
                this.m_stencil[type].opDepthFail = dpfail;
                this.m_stencil[type].opDepthPass = dppass;
            }
        }
    };

    /**
    * @param {number} func
    */
    sglrReferenceContext.ReferenceContext.prototype.depthFunc = function(func) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidCompareFunc(func), gl.INVALID_ENUM))
            return;
        this.m_depthFunc = func;
    };

    /**
    * @param {number} n
    * @param {number} f
    */
    sglrReferenceContext.ReferenceContext.prototype.depthRange = function(n, f) {
        this.m_depthRangeNear = deMath.clamp(n, 0, 1);
        this.m_depthRangeFar = deMath.clamp(f, 0, 1);
    };

    /**
    * @param {number} factor
    * @param {number} units
    */
    sglrReferenceContext.ReferenceContext.prototype.polygonOffset = function(factor, units) {
        this.m_polygonOffsetFactor = factor;
        this.m_polygonOffsetUnits = units;
    };

    /**
    * @param {number} mode
    * @return {boolean}
    */
    sglrReferenceContext.isValidBlendEquation = function(mode) {
        return mode == gl.FUNC_ADD ||
            mode == gl.FUNC_SUBTRACT ||
            mode == gl.FUNC_REVERSE_SUBTRACT ||
            mode == gl.MIN ||
            mode == gl.MAX;
    };

    /**
    * @param {number} factor
    * @return {boolean}
    */
    sglrReferenceContext.isValidBlendFactor = function(factor) {
        switch (factor) {
            case gl.ZERO:
            case gl.ONE:
            case gl.SRC_COLOR:
            case gl.ONE_MINUS_SRC_COLOR:
            case gl.DST_COLOR:
            case gl.ONE_MINUS_DST_COLOR:
            case gl.SRC_ALPHA:
            case gl.ONE_MINUS_SRC_ALPHA:
            case gl.DST_ALPHA:
            case gl.ONE_MINUS_DST_ALPHA:
            case gl.CONSTANT_COLOR:
            case gl.ONE_MINUS_CONSTANT_COLOR:
            case gl.CONSTANT_ALPHA:
            case gl.ONE_MINUS_CONSTANT_ALPHA:
            case gl.SRC_ALPHA_SATURATE:
                return true;

            default:
                return false;
        }
    };

    /**
    * @param {number} mode
    */
    sglrReferenceContext.ReferenceContext.prototype.blendEquation = function(mode) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBlendEquation(mode), gl.INVALID_ENUM))
            return;
        this.m_blendModeRGB = mode;
        this.m_blendModeAlpha = mode;
    };

    /**
    * @param {number} modeRGB
    * @param {number} modeAlpha
    */
    sglrReferenceContext.ReferenceContext.prototype.blendEquationSeparate = function(modeRGB, modeAlpha) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBlendEquation(modeRGB) ||
                    !sglrReferenceContext.isValidBlendEquation(modeAlpha),
                    gl.INVALID_ENUM))
            return;

        this.m_blendModeRGB = modeRGB;
        this.m_blendModeAlpha = modeAlpha;
    };

    /**
    * @param {number} src
    * @param {number} dst
    */
    sglrReferenceContext.ReferenceContext.prototype.blendFunc = function(src, dst) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBlendFactor(src) ||
                    !sglrReferenceContext.isValidBlendFactor(dst),
                    gl.INVALID_ENUM))
            return;

        this.m_blendFactorSrcRGB = src;
        this.m_blendFactorSrcAlpha = src;
        this.m_blendFactorDstRGB = dst;
        this.m_blendFactorDstAlpha = dst;
    };

    /**
    * @param {number} srcRGB
    * @param {number} dstRGB
    * @param {number} srcAlpha
    * @param {number} dstAlpha
    */
    sglrReferenceContext.ReferenceContext.prototype.blendFuncSeparate = function(srcRGB, dstRGB, srcAlpha, dstAlpha) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBlendFactor(srcRGB) ||
                    !sglrReferenceContext.isValidBlendFactor(dstRGB) ||
                    !sglrReferenceContext.isValidBlendFactor(srcAlpha) ||
                    !sglrReferenceContext.isValidBlendFactor(dstAlpha),
                    gl.INVALID_ENUM))
            return;

        this.m_blendFactorSrcRGB = srcRGB;
        this.m_blendFactorSrcAlpha = srcAlpha;
        this.m_blendFactorDstRGB = dstRGB;
        this.m_blendFactorDstAlpha = dstAlpha;
    };

    /**
    * @param {number} red
    * @param {number} green
    * @param {number} blue
    * @param {number} alpha
    */
    sglrReferenceContext.ReferenceContext.prototype.blendColor = function(red, green, blue, alpha) {
        this.m_blendColor = [deMath.clamp(red, 0, 1),
                            deMath.clamp(green, 0, 1),
                            deMath.clamp(blue, 0, 1),
                            deMath.clamp(alpha, 0, 1)];
    };

    /**
    * @param {boolean} r
    * @param {boolean} g
    * @param {boolean} b
    * @param {boolean} a
    */
    sglrReferenceContext.ReferenceContext.prototype.colorMask = function(r, g, b, a) {
        this.m_colorMask = [r, g, b, a];
    };

    /**
    * @param {boolean} mask
    */
    sglrReferenceContext.ReferenceContext.prototype.depthMask = function(mask) {
        this.m_depthMask = mask;
    };

    /**
    * @param {number} mask
    */
    sglrReferenceContext.ReferenceContext.prototype.stencilMask = function(mask) {
        this.stencilMaskSeparate(gl.FRONT_AND_BACK, mask);
    };

    /**
    * @param {number} face
    * @param {number} mask
    */
    sglrReferenceContext.ReferenceContext.prototype.stencilMaskSeparate = function(face, mask) {
        /** @type {boolean} */ var setFront = face == gl.FRONT || face == gl.FRONT_AND_BACK;
        /** @type {boolean} */ var setBack = face == gl.BACK || face == gl.FRONT_AND_BACK;

        if (this.conditionalSetError(!setFront && !setBack, gl.INVALID_ENUM))
            return;

        if (setFront) this.m_stencil[rrDefs.FaceType.FACETYPE_FRONT].writeMask = mask;
        if (setBack) this.m_stencil[rrDefs.FaceType.FACETYPE_BACK].writeMask = mask;
    };

    /**
    * @param {sglrReferenceContext.VertexArray} array
    */
    sglrReferenceContext.ReferenceContext.prototype.bindVertexArray = function(array) {
        if (array)
            this.m_vertexArrayBinding = array;
        else
            this.m_vertexArrayBinding = this.m_defaultVAO;
    };

    /**
    * @return {sglrReferenceContext.VertexArray}
    */
    sglrReferenceContext.ReferenceContext.prototype.createVertexArray = function() { return new sglrReferenceContext.VertexArray(this.m_limits.maxVertexAttribs); };

    /**
    * @param {number} array
    */
    sglrReferenceContext.ReferenceContext.prototype.deleteVertexArray = function(array) {};

    /**
    * @param {number} index
    * @param {number} rawSize
    * @param {number} type
    * @param {boolean} normalized
    * @param {number} stride
    * @param {number} offset
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttribPointer = function(index, rawSize, type, normalized, stride, offset) {
        /** @type {boolean} */ var allowBGRA = false;
        /** @type {number} */ var effectiveSize = rawSize;

        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(effectiveSize <= 0 || effectiveSize > 4, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(type != gl.BYTE && type != gl.UNSIGNED_BYTE &&
                    type != gl.SHORT && type != gl.UNSIGNED_SHORT &&
                    type != gl.INT && type != gl.UNSIGNED_INT &&
                    type != gl.FLOAT && type != gl.HALF_FLOAT &&
                    type != gl.INT_2_10_10_10_REV && type != gl.UNSIGNED_INT_2_10_10_10_REV, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(normalized != true && normalized != false, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(stride < 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError((type == gl.INT_2_10_10_10_REV || type == gl.UNSIGNED_INT_2_10_10_10_REV) && effectiveSize != 4, gl.INVALID_OPERATION))
            return;
        if (this.conditionalSetError(this.m_vertexArrayBinding != null && this.m_arrayBufferBinding == null && offset != 0, gl.INVALID_OPERATION))
            return;

        /** @type {?(sglrReferenceContext.VertexArray.VertexAttribArray)} */ var array_ = this.m_vertexArrayBinding.m_arrays[index]; // TODO: fix type

        array_.size = rawSize;
        array_.stride = stride;
        array_.type = type;
        array_.normalized = normalized;
        array_.integer = false;
        array_.offset = offset;

        array_.bufferBinding = this.m_arrayBufferBinding;
    };

    /**
    * @param {number} index
    * @param {number} size
    * @param {number} type
    * @param {number} stride
    * @param {number} offset
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttribIPointer = function(index, size, type, stride, offset) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(size <= 0 || size > 4, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(type != gl.BYTE && type != gl.UNSIGNED_BYTE &&
                    type != gl.SHORT && type != gl.UNSIGNED_SHORT &&
                    type != gl.INT && type != gl.UNSIGNED_INT, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(stride < 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(this.m_vertexArrayBinding != null && this.m_arrayBufferBinding == null && offset != 0, gl.INVALID_OPERATION))
            return;

        /** @type {?(sglrReferenceContext.VertexArray.VertexAttribArray)} */ var array_ = this.m_vertexArrayBinding.m_arrays[index]; // TODO: fix type

        array_.size = size;
        array_.stride = stride;
        array_.type = type;
        array_.normalized = false;
        array_.integer = true;
        array_.offset = offset;

        array_.bufferBinding = this.m_arrayBufferBinding;
    };

    /**
    * @param {number} index
    */
    sglrReferenceContext.ReferenceContext.prototype.enableVertexAttribArray = function(index) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_vertexArrayBinding.m_arrays[index].enabled = true;
    };

    /**
    * @param {number} index
    */
    sglrReferenceContext.ReferenceContext.prototype.disableVertexAttribArray = function(index) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_vertexArrayBinding.m_arrays[index].enabled = false;
    };

    /**
    * @param {number} index
    * @param {number} divisor
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttribDivisor = function(index, divisor) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_vertexArrayBinding.m_arrays[index].divisor = divisor;
    };

    /**
    * @param {number} index
    * @param {number} x
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttrib1f = function(index, x) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_currentAttribs[index] = new rrGenericVector.GenericVec4(x, 0, 0, 1);
    };

    /**
    * @param {number} index
    * @param {number} x
    * @param {number} y
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttrib2f = function(index, x, y) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_currentAttribs[index] = new rrGenericVector.GenericVec4(x, y, 0, 1);
    };

    /**
    * @param {number} index
    * @param {number} x
    * @param {number} y
    * @param {number} z
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttrib3f = function(index, x, y, z) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_currentAttribs[index] = new rrGenericVector.GenericVec4(x, y, z, 1);
    };

    /**
    * @param {number} index
    * @param {number} x
    * @param {number} y
    * @param {number} z
    * @param {number} w
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttrib4f = function(index, x, y, z, w) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_currentAttribs[index] = new rrGenericVector.GenericVec4(x, y, z, w);
    };

    /**
    * @param {number} index
    * @param {number} x
    * @param {number} y
    * @param {number} z
    * @param {number} w
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttribI4i = function(index, x, y, z, w) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_currentAttribs[index] = new rrGenericVector.GenericVec4(x, y, z, w);
    };

    /**
    * @param {number} index
    * @param {number} x
    * @param {number} y
    * @param {number} z
    * @param {number} w
    */
    sglrReferenceContext.ReferenceContext.prototype.vertexAttribI4ui = function(index, x, y, z, w) {
        if (this.conditionalSetError(index >= this.m_limits.maxVertexAttribs, gl.INVALID_VALUE))
            return;

        this.m_currentAttribs[index] = new rrGenericVector.GenericVec4(x, y, z, w);
    };

    /**
    * @param {sglrShaderProgram.ShaderProgram} program
    * @param {string} name
    * @return {number}
    */
    sglrReferenceContext.ReferenceContext.prototype.getAttribLocation = function(program, name) {
        if (this.conditionalSetError(!(program), gl.INVALID_OPERATION))
            return -1;

        for (var i = 0; i < program.m_attributeNames.length; i++)
            if (program.m_attributeNames[i] === name)
                return i;

        return -1;
    };

   /**
    * @param {number} pname
    */
    sglrReferenceContext.ReferenceContext.prototype.getParameter = function(pname) {
        switch (pname) {
            case (gl.VIEWPORT): return new Int32Array(this.m_viewport);
            case (gl.SCISSOR_BOX): return new Int32Array(this.m_scissorBox);
            default:
                throw new Error('Unimplemented');
        }
    };

    /**
    * @param {number} location
    * @param {gluShaderUtil.DataType} type
    * @param {Array<number>} value
    */
    sglrReferenceContext.ReferenceContext.prototype.uniformValue = function(location, type, value) {
        if (this.conditionalSetError(!this.m_currentProgram, gl.INVALID_OPERATION))
            return;

        if (location === null)
            return;

        /** @type {sglrShaderProgram.Uniform} */ var uniform = this.m_currentProgram.m_uniforms[location];

        if (this.conditionalSetError(!uniform, gl.INVALID_OPERATION))
            return;

        if (gluShaderUtil.isDataTypeSampler(uniform.type)) {
            if (this.conditionalSetError(type != gluShaderUtil.DataType.INT, gl.INVALID_OPERATION))
                return;
        } else if (this.conditionalSetError(uniform.type != type, gl.INVALID_OPERATION))
            return;
        /* TODO: Do we need to copy objects? */
        uniform.value = value;
    };

    /**
     * @param {number} location
     * @param {number} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform1f = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT, [x]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform1fv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform1i = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT, [x]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform1iv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     * @param {number} y
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform2f = function(location, x, y) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_VEC2, [x, y]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform2fv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_VEC2, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     * @param {number} y
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform2i = function(location, x, y) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT_VEC2, [x, y]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform2iv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT_VEC2, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform3f = function(location, x, y, z) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_VEC3, [x, y, z]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform3fv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_VEC3, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     * @param {number} y
     * @param {number} z
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform3i = function(location, x, y, z) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT_VEC3, [x, y, z]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform3iv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT_VEC3, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform4f = function(location, x, y, z, w) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_VEC4, [x, y, z, w]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform4fv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_VEC4, x);
    };

    /**
     * @param {number} location
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @param {number} w
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform4i = function(location, x, y, z, w) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT_VEC4, [x, y, z, w]);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniform4iv = function(location, x) {
        return this.uniformValue(location, gluShaderUtil.DataType.INT_VEC4, x);
    };

    /**
     * @return {Array<string>}
     */
    sglrReferenceContext.ReferenceContext.prototype.getSupportedExtensions = function() {
        var extensions = gl.getSupportedExtensions(); //TODO: Let's just return gl's supported extensions for now
        return extensions;
    };

    /**
     * @param {string} name
     * @return {*}
     */
    sglrReferenceContext.ReferenceContext.prototype.getExtension = function(name) {
        return gl.getExtension(name); //TODO: Let's just return gl's supported extensions for now
    };

    /** transpose matrix 'x' of 'size' columns and rows
     * @param {number} size
     * @param {Array<number>} x
     * @return {Array<number>}
     */
    sglrReferenceContext.trans = function(size, x) {
        /** @type {Array<number>} */ var result = [];
        for (var row = 0; row < size; ++row)
            for (var col = 0; col < size; ++col)
            result[row * size + col] = x[col * size + row];

        return result;
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniformMatrix2fv = function(location, transpose, x) {
        /* change from column-major to internal row-major if transpose if FALSE */
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_MAT2, !transpose ? sglrReferenceContext.trans(2, x) : x);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniformMatrix3fv = function(location, transpose, x) {
        /* change from column-major to internal row-major if transpose if FALSE */
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_MAT3, !transpose ? sglrReferenceContext.trans(3, x) : x);
    };

    /**
     * @param {number} location
     * @param {Array<number>} x
     */
    sglrReferenceContext.ReferenceContext.prototype.uniformMatrix4fv = function(location, transpose, x) {
        /* change from column-major to internal row-major if transpose if FALSE */
        return this.uniformValue(location, gluShaderUtil.DataType.FLOAT_MAT4, !transpose ? sglrReferenceContext.trans(4, x) : x);
    };

    /**
    * @param {sglrShaderProgram.ShaderProgram} program
    * @param {string} name
    * @return {number}
    */
    sglrReferenceContext.ReferenceContext.prototype.getUniformLocation = function(program, name) {
        if (this.conditionalSetError(!program, gl.INVALID_OPERATION))
            return -1;

        for (var i = 0; i < program.m_uniforms.length; i++)
            if (program.m_uniforms[i].name === name)
                return i;

        return -1;
    };

    /**
    * @param {number} w
    */
    sglrReferenceContext.ReferenceContext.prototype.lineWidth = function(w) {
        if (this.conditionalSetError(w < 0, gl.INVALID_VALUE))
            return;
        this.m_lineWidth = w;
    };

    /**
    * @param {number} target
    * @return {boolean}
    */
    sglrReferenceContext.isValidBufferTarget = function(target) {
        switch (target) {
            case gl.ARRAY_BUFFER:
            case gl.COPY_READ_BUFFER:
            case gl.COPY_WRITE_BUFFER:
            case gl.ELEMENT_ARRAY_BUFFER:
            case gl.PIXEL_PACK_BUFFER:
            case gl.PIXEL_UNPACK_BUFFER:
            case gl.TRANSFORM_FEEDBACK_BUFFER:
            case gl.UNIFORM_BUFFER:
                return true;

            default:
                return false;
        }
    };

    /**
    * @param {number} target
    * @param {sglrReferenceContext.DataBuffer} buffer
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.setBufferBinding = function(target, buffer) {
        switch (target) {
            case gl.ARRAY_BUFFER: this.m_arrayBufferBinding = buffer; break;
            case gl.COPY_READ_BUFFER: this.m_copyReadBufferBinding = buffer; break;
            case gl.COPY_WRITE_BUFFER: this.m_copyWriteBufferBinding = buffer; break;
            case gl.ELEMENT_ARRAY_BUFFER: this.m_vertexArrayBinding.m_elementArrayBufferBinding = buffer; break;
            case gl.PIXEL_PACK_BUFFER: this.m_pixelPackBufferBinding = buffer; break;
            case gl.PIXEL_UNPACK_BUFFER: this.m_pixelUnpackBufferBinding = buffer; break;
            case gl.TRANSFORM_FEEDBACK_BUFFER: this.m_transformFeedbackBufferBinding = buffer; break;
            case gl.UNIFORM_BUFFER: this.m_uniformBufferBinding = buffer; break;
            default:
                throw new Error('Unrecognized target: ' + target);
        }
    };

    /**
    * @param {number} target
    * @return {sglrReferenceContext.DataBuffer}
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.getBufferBinding = function(target) {
        switch (target) {
            case gl.ARRAY_BUFFER: return this.m_arrayBufferBinding;
            case gl.COPY_READ_BUFFER: return this.m_copyReadBufferBinding;
            case gl.COPY_WRITE_BUFFER: return this.m_copyWriteBufferBinding;
            case gl.ELEMENT_ARRAY_BUFFER: return this.m_vertexArrayBinding.m_elementArrayBufferBinding;
            case gl.PIXEL_PACK_BUFFER: return this.m_pixelPackBufferBinding;
            case gl.PIXEL_UNPACK_BUFFER: return this.m_pixelUnpackBufferBinding;
            case gl.TRANSFORM_FEEDBACK_BUFFER: return this.m_transformFeedbackBufferBinding;
            case gl.UNIFORM_BUFFER: return this.m_uniformBufferBinding;
            default:
                throw new Error('Unrecognized target: ' + target);
        }
    };

    /**
    * @param {number} target
    * @param {sglrReferenceContext.DataBuffer} buffer
    */
    sglrReferenceContext.ReferenceContext.prototype.bindBuffer = function(target, buffer) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBufferTarget(target), gl.INVALID_ENUM))
            return;

        this.setBufferBinding(target, buffer);
    };

    /**
    * @return {sglrReferenceContext.DataBuffer}
    */
    sglrReferenceContext.ReferenceContext.prototype.createBuffer = function() { return new sglrReferenceContext.DataBuffer(); };

    /**
    * @param {number} buffer
    */
    sglrReferenceContext.ReferenceContext.prototype.deleteBuffer = function(buffer) {};

    /**
    * @param {number} target
    * @param {number|goog.NumberArray} input
    * @param {number} usage
    */
    sglrReferenceContext.ReferenceContext.prototype.bufferData = function(target, input, usage) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBufferTarget(target), gl.INVALID_ENUM))
            return;
        /** @type {sglrReferenceContext.DataBuffer} */ var buffer = this.getBufferBinding(target);
        if (this.conditionalSetError(!buffer, gl.INVALID_OPERATION))
            return;

        if (typeof input == 'number') {
            if (this.conditionalSetError(input < 0, gl.INVALID_VALUE))
                return;
            buffer.setStorage(input);
        } else {
            buffer.setData(input);
        }
    };

    /**
    * @param {number} target
    * @param {number} offset
    * @param {goog.NumberArray} data
    */
    sglrReferenceContext.ReferenceContext.prototype.bufferSubData = function(target, offset, data) {
        if (this.conditionalSetError(!sglrReferenceContext.isValidBufferTarget(target), gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(offset < 0, gl.INVALID_VALUE))
            return;
        /** @type {sglrReferenceContext.DataBuffer} */ var buffer = this.getBufferBinding(target);
        if (this.conditionalSetError(!buffer, gl.INVALID_OPERATION))
            return;

        if (this.conditionalSetError(offset + data.byteLength > buffer.getSize(), gl.INVALID_VALUE))
            return;
        buffer.setSubData(offset, data);
    };

    /**
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    * @param {number} format
    * @param {number} type
    * @param {goog.NumberArray} pixels
    */
    sglrReferenceContext.ReferenceContext.prototype.readPixels = function(x, y, width, height, format, type, pixels) {
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var src = this.getReadColorbuffer();

        // Map transfer format.
        /** @type {tcuTexture.TextureFormat} */ var transferFmt = gluTextureUtil.mapGLTransferFormat(format, type);

        // Clamp input values
        /** @type {number} */ var copyX = deMath.clamp(x, 0, src.raw().getHeight());
        /** @type {number} */ var copyY = deMath.clamp(y, 0, src.raw().getDepth());
        /** @type {number} */ var copyWidth = deMath.clamp(width, 0, src.raw().getHeight() - x);
        /** @type {number} */ var copyHeight = deMath.clamp(height, 0, src.raw().getDepth() - y);

        /** @type {?ArrayBuffer} */ var data;
        /** @type {number} */ var offset;
        if (this.m_pixelPackBufferBinding) {
            if (this.conditionalSetError(typeof pixels !== 'number', gl.INVALID_VALUE))
                return;
            data = this.m_pixelPackBufferBinding.getData();
            offset = pixels.byteOffset;
        } else {
            if (pixels instanceof ArrayBuffer) {
                data = pixels;
                offset = 0;
            } else {
                data = pixels.buffer;
                offset = pixels.byteOffset;
            }
        }

        /** @type {tcuTexture.PixelBufferAccess} */
        var dst = new tcuTexture.PixelBufferAccess({
            format: transferFmt,
            width: width,
            height: height,
            depth: 1,
            rowPitch: deMath.deAlign32(width * transferFmt.getPixelSize(), this.m_pixelPackAlignment),
            slicePitch: 0,
            data: data,
            offset: offset});

        src = src.getSubregion([copyX, copyY, copyWidth, copyHeight]);
        src.resolveMultisampleColorBuffer(tcuTextureUtil.getSubregion(dst, 0, 0, 0, copyWidth, copyHeight, 1));
    };

    /**
    * @return {number}
    */
    sglrReferenceContext.ReferenceContext.prototype.getType = function() {
        return this.m_type;
    };

    /**
    * @return {tcuTexture.PixelBufferAccess}
    */
    sglrReferenceContext.nullAccess = function() {
        return new tcuTexture.PixelBufferAccess({
            width: 0,
            height: 0});
    };

    /**
    * @param {sglrReferenceContext.Framebuffer} framebuffer
    * @param {sglrReferenceContext.AttachmentPoint} point
    * @return {tcuTexture.PixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getFboAttachment = function(framebuffer, point) {
        /** @type {sglrReferenceContext.Attachment} */ var attachment = framebuffer.getAttachment(point);

        switch (attachment.type) {
            case sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_TEXTURE: {
                var container = /** @type {sglrReferenceContext.TextureContainer} */ (attachment.object);
                /** @type {?sglrReferenceContext.TextureType} */ var type = container.getType();
                var texture = container.texture;

                if (type == sglrReferenceContext.TextureType.TYPE_2D)
                    return texture.getLevel(attachment.level);
                else if (type == sglrReferenceContext.TextureType.TYPE_CUBE_MAP)
                    return texture.getFace(attachment.level, sglrReferenceContext.texTargetToFace(attachment.texTarget));
                else if (type == sglrReferenceContext.TextureType.TYPE_2D_ARRAY ||
                        type == sglrReferenceContext.TextureType.TYPE_3D ||
                        type == sglrReferenceContext.TextureType.TYPE_CUBE_MAP_ARRAY) {
                    /** @type {tcuTexture.PixelBufferAccess} */ var level = texture.getLevel(attachment.level);

                    return new tcuTexture.PixelBufferAccess({
                        format: level.getFormat(),
                        width: level.getWidth(),
                        height: level.getHeight(),
                        depth: 1,
                        rowPitch: level.getRowPitch(),
                        slicePitch: 0,
                        data: level.getBuffer(),
                        offset: level.getSlicePitch() * attachment.layer});
                } else
                    return sglrReferenceContext.nullAccess();
            }

            case sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_RENDERBUFFER: {
                var rbo = /** @type {sglrReferenceContext.Renderbuffer} */ (attachment.object);
                return rbo.getAccess();
            }

            default:
                return sglrReferenceContext.nullAccess();
        }
    };

    /**
    * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getReadColorbuffer = function() {
        if (this.m_readFramebufferBinding)
            return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess(this.getFboAttachment(this.m_readFramebufferBinding, sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_COLOR0));
        else
            return this.m_defaultColorbuffer;
    };

    // sglrReferenceContext.ReferenceContext.prototype.drawArrays = function(mode, first, count) {
    //     this.drawArraysInstanced(mode, first, count, 1);
    // };

    /**
     * @param {number} target
     * @return {number}
     * @throws {Error}
     */
    sglrReferenceContext.ReferenceContext.prototype.checkFramebufferStatus = function(target) {
        if (this.conditionalSetError(target != gl.FRAMEBUFFER &&
                    target != gl.DRAW_FRAMEBUFFER &&
                    target != gl.READ_FRAMEBUFFER, gl.INVALID_ENUM))
            return 0;

        // Select binding point.
        /** @type {sglrReferenceContext.Framebuffer} */ var framebufferBinding = (target == gl.FRAMEBUFFER || target == gl.DRAW_FRAMEBUFFER) ? this.m_drawFramebufferBinding : this.m_readFramebufferBinding;

        // Default framebuffer is always complete.
        if (!framebufferBinding)
            return gl.FRAMEBUFFER_COMPLETE;

        /** @type {number} */ var width = -1;
        /** @type {number} */ var height = -1;
        /** @type {boolean} */ var hasAttachment = false;
        /** @type {boolean} */ var attachmentComplete = true;
        /** @type {boolean} */ var dimensionsOk = true;

        for (var key in sglrReferenceContext.AttachmentPoint) {
            /** @type {sglrReferenceContext.AttachmentPoint} */ var point = sglrReferenceContext.AttachmentPoint[key];
            /** @type {sglrReferenceContext.Attachment} */ var attachment = framebufferBinding.getAttachment(point);
            /** @type {number} */ var attachmentWidth = 0;
            /** @type {number} */ var attachmentHeight = 0;
            /** @type {tcuTexture.TextureFormat} */ var attachmentFormat;

            if (attachment.type == sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_TEXTURE) {
                var container = /** @type {sglrReferenceContext.TextureContainer} */ (attachment.object);
                /** @type {tcuTexture.ConstPixelBufferAccess} */ var level;

                if (attachment.texTarget == sglrReferenceContext.TexTarget.TEXTARGET_2D) {
                    DE_ASSERT(container.textureType == sglrReferenceContext.TextureType.TYPE_2D);
                    /** @type {sglrReferenceContext.Texture2D} */ var tex2D = /** @type {sglrReferenceContext.Texture2D} */ (container.texture);

                    if (tex2D.hasLevel(attachment.level))
                        level = tex2D.getLevel(attachment.level);
                // TODO: implement CUBE_MAP, 2D_ARRAY, 3D, CUBE_MAP_ARRAY
                } else if (deMath.deInRange32(attachment.texTarget, sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_X,
                                                        sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_Z)) {
                    DE_ASSERT(container.textureType == sglrReferenceContext.TextureType.TYPE_CUBE_MAP);

                    var texCube = /** @type {sglrReferenceContext.TextureCube} */ (container.texture);
                    var face = sglrReferenceContext.texTargetToFace(attachment.texTarget);

                    if (texCube.hasFace(attachment.level, face))
                        level = texCube.getFace(attachment.level, face);
                } else if (attachment.texTarget == sglrReferenceContext.TexTarget.TEXTARGET_2D_ARRAY) {
                    DE_ASSERT(container.textureType == sglrReferenceContext.TextureType.TYPE_2D_ARRAY);
                    var tex2DArr = /** @type {sglrReferenceContext.Texture2DArray} */ (container.texture);

                    if (tex2DArr.hasLevel(attachment.level))
                        level = tex2DArr.getLevel(attachment.level); // \note Slice doesn't matter here.
                } else if (attachment.texTarget == sglrReferenceContext.TexTarget.TEXTARGET_3D) {
                    DE_ASSERT(container.textureType == sglrReferenceContext.TextureType.TYPE_3D);
                    var tex3D = /** @type {sglrReferenceContext.Texture3D} */ (container.texture);

                    if (tex3D.hasLevel(attachment.level))
                        level = tex3D.getLevel(attachment.level); // \note Slice doesn't matter here.
                // } else if (attachment.texTarget == sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_ARRAY) {
                //     DE_ASSERT(container.textureType == sglrReferenceContext.TextureType.TYPE_CUBE_MAP_ARRAY);
                //     var texCubeArr = container.texture;
                //
                //     if (texCubeArr.hasLevel(attachment.level))
                //         level = texCubeArr.getLevel(attachment.level); // \note Slice doesn't matter here.
                } else
                    throw new Error('sglrReferenceContext.Framebuffer attached to a texture but no valid target specified.');

                attachmentWidth = level.getWidth();
                attachmentHeight = level.getHeight();
                attachmentFormat = level.getFormat();
            } else if (attachment.type == sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_RENDERBUFFER) {
                var renderbuffer = attachment.object;

                attachmentWidth = renderbuffer.getWidth();
                attachmentHeight = renderbuffer.getHeight();
                attachmentFormat = renderbuffer.getFormat();
            } else
                continue; // Skip rest of checks.

            if (!hasAttachment && attachmentWidth > 0 && attachmentHeight > 0) {
                width = attachmentWidth;
                height = attachmentHeight;
                hasAttachment = true;
            } else if (attachmentWidth != width || attachmentHeight != height)
                dimensionsOk = false;

            // Validate attachment point compatibility.
            switch (attachmentFormat.order) {
                case tcuTexture.ChannelOrder.R:
                case tcuTexture.ChannelOrder.RG:
                case tcuTexture.ChannelOrder.RGB:
                case tcuTexture.ChannelOrder.RGBA:
                case tcuTexture.ChannelOrder.sRGB:
                case tcuTexture.ChannelOrder.sRGBA:
                    if (point != sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_COLOR0)
                        attachmentComplete = false;
                    break;

                case tcuTexture.ChannelOrder.D:
                    if (point != sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_DEPTH)
                        attachmentComplete = false;
                    break;

                case tcuTexture.ChannelOrder.S:
                    if (point != sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_STENCIL)
                        attachmentComplete = false;
                    break;

                case tcuTexture.ChannelOrder.DS:
                    if (point != sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_DEPTH &&
                        point != sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_STENCIL)
                        attachmentComplete = false;
                    break;

                default:
                    throw new Error('Unsupported attachment channel order:' + attachmentFormat.order);
            }
        }

        if (!attachmentComplete)
            return gl.FRAMEBUFFER_INCOMPLETE_ATTACHMENT;
        else if (!hasAttachment)
            return gl.FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT;
        else if (!dimensionsOk)
            return gl.FRAMEBUFFER_INCOMPLETE_DIMENSIONS;
        else
            return gl.FRAMEBUFFER_COMPLETE;
    };

    /**
     * @param {number} mode
     * @return {boolean}
     */
    sglrReferenceContext.ReferenceContext.prototype.predrawErrorChecks = function(mode) {
        if (this.conditionalSetError(mode != gl.POINTS &&
                    mode != gl.LINE_STRIP && mode != gl.LINE_LOOP && mode != gl.LINES &&
                    mode != gl.TRIANGLE_STRIP && mode != gl.TRIANGLE_FAN && mode != gl.TRIANGLES,
                    gl.INVALID_ENUM))
            return false;

        // \todo [jarkko] Uncomment following code when the buffer mapping support is added
        //for (size_t ndx = 0; ndx < vao.m_arrays.length; ++ndx)
        //  if (vao.m_arrays[ndx].enabled && vao.m_arrays[ndx].bufferBinding && vao.m_arrays[ndx].bufferBinding->isMapped)
        //      RC_ERROR_RET(gl.INVALID_OPERATION, RC_RET_VOID);

        if (this.conditionalSetError(this.checkFramebufferStatus(gl.DRAW_FRAMEBUFFER) != gl.FRAMEBUFFER_COMPLETE, gl.INVALID_FRAMEBUFFER_OPERATION))
            return false;

        return true;
    };

    /**
    * Draws quads from vertex arrays
    * @param {number} mode GL primitive type to draw with.
    * @param {number} first First vertex to begin drawing with
    * @param {number} count How many vertices to draw (not counting vertices before first)
    * @param {number} instanceCount
    */
    sglrReferenceContext.ReferenceContext.prototype.drawArraysInstanced = function(mode, first, count, instanceCount) {
        if (this.conditionalSetError(first < 0 || count < 0 || instanceCount < 0, gl.INVALID_VALUE))
            return;

        if (!this.predrawErrorChecks(mode))
            return;

        // All is ok
        this.drawQuads(mode, first, count, instanceCount);
    };

    /**
    * @param {number} mode GL primitive type to draw with.
    * @param {number} start
    * @param {number} end
    * @param {number} count How many vertices to draw (not counting vertices before first)
    * @param {number} type Data type
    * @param {number} offset
    */
    sglrReferenceContext.ReferenceContext.prototype.drawRangeElements = function(mode, start, end, count, type, offset) {
        if (this.conditionalSetError(end < start, gl.INVALID_VALUE))
            return;

        this.drawElements(mode, count, type, offset);
    };

    /**
    * @param {number} mode GL primitive type to draw with.
    * @param {number} count How many vertices to draw (not counting vertices before first)
    * @param {number} type Data type
    * @param {number} offset
    */
    sglrReferenceContext.ReferenceContext.prototype.drawElements = function(mode, count, type, offset) {
        this.drawElementsInstanced(mode, count, type, offset, 1);
    };

    /**
    * @param {number} mode GL primitive type to draw with.
    * @param {number} count How many vertices to draw (not counting vertices before first)
    * @param {number} type Data type
    * @param {number} offset
    * @param {number} instanceCount
    */
    sglrReferenceContext.ReferenceContext.prototype.drawElementsInstanced = function(mode, count, type, offset, instanceCount) {
        this.drawElementsInstancedBaseVertex(mode, count, type, offset, instanceCount, 0);
    };

    /**
    * @param {number} mode GL primitive type to draw with.
    * @param {number} count How many vertices to draw (not counting vertices before first)
    * @param {number} type Data type
    * @param {number} offset
    * @param {number} instanceCount
    * @param {number} baseVertex
    */
    sglrReferenceContext.ReferenceContext.prototype.drawElementsInstancedBaseVertex = function(mode, count, type, offset, instanceCount, baseVertex) {
        var vao = this.m_vertexArrayBinding;

        if (this.conditionalSetError(type != gl.UNSIGNED_BYTE &&
                    type != gl.UNSIGNED_SHORT &&
                    type != gl.UNSIGNED_INT, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(count < 0 || instanceCount < 0, gl.INVALID_VALUE))
            return;

        if (!this.predrawErrorChecks(mode))
            return;

        if (this.conditionalSetError(count > 0 && !vao.m_elementArrayBufferBinding, gl.INVALID_OPERATION))
            return;
        // All is ok
        var data = vao.m_elementArrayBufferBinding.getData();
        var indices = new rrRenderer.DrawIndices(data, sglrReferenceUtils.mapGLIndexType(type), offset, baseVertex);

        this.drawQuads(mode, indices, count, instanceCount);
    };

    /**
    * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} access
    * @return {Array<number>}
    */
    sglrReferenceContext.getBufferRect = function(access) { return [0, 0, access.raw().getHeight(), access.raw().getDepth()]; };

    /**
    * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getDrawColorbuffer = function() {
        if (this.m_drawFramebufferBinding)
            return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess(this.getFboAttachment(this.m_drawFramebufferBinding, sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_COLOR0));
        return this.m_defaultColorbuffer;
    };

    /**
    * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getDrawDepthbuffer = function() {
        if (this.m_drawFramebufferBinding)
            return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess(this.getFboAttachment(this.m_drawFramebufferBinding, sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_DEPTH));
        return this.m_defaultDepthbuffer;
    };

    /**
    * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getDrawStencilbuffer = function() {
        if (this.m_drawFramebufferBinding)
            return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess(this.getFboAttachment(this.m_drawFramebufferBinding, sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_STENCIL));
        return this.m_defaultStencilbuffer;
    };

    /**
    * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getReadDepthbuffer = function() {
        if (this.m_readFramebufferBinding)
            return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess(this.getFboAttachment(this.m_readFramebufferBinding, sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_DEPTH));
        return this.m_defaultDepthbuffer;
    };

    /**
    * @return {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess}
    */
    sglrReferenceContext.ReferenceContext.prototype.getReadStencilbuffer = function() {
        if (this.m_readFramebufferBinding)
            return rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess.fromSinglesampleAccess(this.getFboAttachment(this.m_readFramebufferBinding, sglrReferenceContext.AttachmentPoint.ATTACHMENTPOINT_STENCIL));
        return this.m_defaultStencilbuffer;
    };

    /**
    * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} access
    * @param {number} s
    * @param {number} x
    * @param {number} y
    * @param {number} depth
    */
    sglrReferenceContext.writeDepthOnly = function(access, s, x, y, depth) { access.raw().setPixDepth(depth, s, x, y); };

    /**
    * @param {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} access
    * @param {number} s
    * @param {number} x
    * @param {number} y
    * @param {number} stencil
    * @param {number} writeMask
    */
    sglrReferenceContext.writeStencilOnly = function(access, s, x, y, stencil, writeMask) {
        /** @type {number} */ var oldVal = access.raw().getPixelInt(s, x, y)[3];
        access.raw().setPixStencil((oldVal & ~writeMask) | (stencil & writeMask), s, x, y);
    };

    /**
    * @param {number} bits
    * @param {number} s
    * @return {number}
    */
    sglrReferenceContext.maskStencil = function(bits, s) { return s & ((1 << bits) - 1); };

    /**
    * @param {number} buffers
    */
    sglrReferenceContext.ReferenceContext.prototype.clear = function(buffers) {
        if (this.conditionalSetError((buffers & ~(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT)) != 0, gl.INVALID_VALUE))
            return;

        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var colorBuf0 = this.getDrawColorbuffer();
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var depthBuf = this.getDrawDepthbuffer();
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var stencilBuf = this.getDrawStencilbuffer();
        /** @type {boolean} */ var hasColor0 = /** @type {!boolean} */ (colorBuf0 && !colorBuf0.isEmpty());
        /** @type {boolean} */ var hasDepth = /** @type {!boolean} */ (depthBuf && !depthBuf.isEmpty());
        /** @type {boolean} */ var hasStencil = /** @type {!boolean} */ (stencilBuf && !stencilBuf.isEmpty());
        /** @type {Array<number>} */ var baseArea = this.m_scissorEnabled ? this.m_scissorBox : [0, 0, 0x7fffffff, 0x7fffffff];

        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var access;
        /** @type {boolean} */ var isSharedDepthStencil;

        if (hasColor0 && (buffers & gl.COLOR_BUFFER_BIT) != 0) {
            /** @type {Array<number>} */ var colorArea = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(colorBuf0));
            access = colorBuf0.getSubregion(colorArea);
            /** @type {boolean} */ var isSRGB = colorBuf0.raw().getFormat().isSRGB();
            /** @type {Array<number>} */ var c = (isSRGB && this.m_sRGBUpdateEnabled) ? tcuTextureUtil.linearToSRGB(this.m_clearColor) : this.m_clearColor;
            /** @type {boolean} */ var maskUsed = !this.m_colorMask[0] || !this.m_colorMask[1] || !this.m_colorMask[2] || !this.m_colorMask[3];
            /** @type {boolean} */ var maskZero = !this.m_colorMask[0] && !this.m_colorMask[1] && !this.m_colorMask[2] && !this.m_colorMask[3];

            if (!maskUsed)
                access.clear(c);
            else if (!maskZero) {
                for (var y = 0; y < access.raw().getDepth(); y++)
                    for (var x = 0; x < access.raw().getHeight(); x++)
                        for (var s = 0; s < access.getNumSamples(); s++)
                            access.raw().setPixel(tcuTextureUtil.select(c, access.raw().getPixel(s, x, y), this.m_colorMask), s, x, y);
            }
            // else all channels masked out
        }

        if (hasDepth && (buffers & gl.DEPTH_BUFFER_BIT) != 0 && this.m_depthMask) {
            /** @type {Array<number>} */ var depthArea = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(depthBuf));
            access = depthBuf.getSubregion(depthArea);
            isSharedDepthStencil = depthBuf.raw().getFormat().order != tcuTexture.ChannelOrder.D;

            if (isSharedDepthStencil) {
                // Slow path where stencil is masked out in write.
                for (var y = 0; y < access.raw().getDepth(); y++)
                    for (var x = 0; x < access.raw().getHeight(); x++)
                        for (var s = 0; s < access.getNumSamples(); s++)
                            sglrReferenceContext.writeDepthOnly(access, s, x, y, this.m_clearDepth);
            } else
                access.clear([this.m_clearDepth, 0, 0, 0]);
        }

        if (hasStencil && (buffers & gl.STENCIL_BUFFER_BIT) != 0) {
            /** @type {Array<number>} */ var stencilArea = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(stencilBuf));
            access = stencilBuf.getSubregion(stencilArea);
            /** @type {number} */ var stencilBits = stencilBuf.raw().getFormat().getNumStencilBits();
            /** @type {number} */ var stencil = sglrReferenceContext.maskStencil(stencilBits, this.m_clearStencil);
            isSharedDepthStencil = stencilBuf.raw().getFormat().order != tcuTexture.ChannelOrder.S;

            if (isSharedDepthStencil || ((this.m_stencil[rrDefs.FaceType.FACETYPE_FRONT].writeMask & ((1 << stencilBits) - 1)) != ((1 << stencilBits) - 1))) {
                // Slow path where depth or stencil is masked out in write.
                for (var y = 0; y < access.raw().getDepth(); y++)
                    for (var x = 0; x < access.raw().getHeight(); x++)
                        for (var s = 0; s < access.getNumSamples(); s++)
                            sglrReferenceContext.writeStencilOnly(access, s, x, y, stencil, this.m_stencil[rrDefs.FaceType.FACETYPE_FRONT].writeMask);
            } else
                access.clear([0, 0, 0, stencil]);
        }
    };

    /**
    * @param {number} buffer
    * @param {number} drawbuffer
    * @param {Array<number>} value
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.clearBufferiv = function(buffer, drawbuffer, value) {
        if (this.conditionalSetError(buffer != gl.COLOR && buffer != gl.STENCIL, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(drawbuffer != 0, gl.INVALID_VALUE))
            return;

        /** @type {Array<number>} */ var baseArea = this.m_scissorEnabled ? this.m_scissorBox : [0, 0, 0x7fffffff, 0x7fffffff];

        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var access;

        if (buffer == gl.COLOR) {
            /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var colorBuf = this.getDrawColorbuffer();
            /** @type {boolean} */ var maskUsed = !this.m_colorMask[0] || !this.m_colorMask[1] || !this.m_colorMask[2] || !this.m_colorMask[3];
            /** @type {boolean} */ var maskZero = !this.m_colorMask[0] && !this.m_colorMask[1] && !this.m_colorMask[2] && !this.m_colorMask[3];

            if (!colorBuf.isEmpty() && !maskZero) {
                /** @type {Array<number>} */ var colorArea = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(colorBuf));
                access = colorBuf.getSubregion(colorArea);

                    if (!maskUsed)
                        access.clear(value);
                    else {
                    for (var y = 0; y < access.raw().getDepth(); y++)
                        for (var x = 0; x < access.raw().getHeight(); x++)
                            for (var s = 0; s < access.getNumSamples(); s++)
                                access.raw().setPixel(tcuTextureUtil.select(value, access.raw().getPixel(s, x, y), this.m_colorMask), s, x, y);
                    }
            }
        } else {
            if (buffer !== gl.STENCIL)
                throw new Error('Unexpected buffer type: ' + buffer);

                /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var stencilBuf = this.getDrawStencilbuffer();

            if (!stencilBuf.isEmpty() && this.m_stencil[rrDefs.FaceType.FACETYPE_FRONT].writeMask != 0) {
                /** @type {Array<number>} */ var area = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(stencilBuf));
                access = stencilBuf.getSubregion(area);
                /** @type {number} */ var stencil = value[0];

            for (var y = 0; y < access.raw().getDepth(); y++)
                    for (var x = 0; x < access.raw().getHeight(); x++)
                        for (var s = 0; s < access.getNumSamples(); s++)
                            sglrReferenceContext.writeStencilOnly(access, s, x, y, stencil, this.m_stencil[rrDefs.FaceType.FACETYPE_FRONT].writeMask);
            }
        }
    };

    /**
    * @param {number} buffer
    * @param {number} drawbuffer
    * @param {Array<number>} value
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.clearBufferfv = function(buffer, drawbuffer, value) {
        if (this.conditionalSetError(buffer != gl.COLOR && buffer != gl.DEPTH, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(drawbuffer != 0, gl.INVALID_VALUE))
            return;

        /** @type {Array<number>} */ var baseArea = this.m_scissorEnabled ? this.m_scissorBox : [0, 0, 0x7fffffff, 0x7fffffff];
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var access;
        if (buffer == gl.COLOR) {
            /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var colorBuf = this.getDrawColorbuffer();
            /** @type {boolean} */ var maskUsed = !this.m_colorMask[0] || !this.m_colorMask[1] || !this.m_colorMask[2] || !this.m_colorMask[3];
            /** @type {boolean} */ var maskZero = !this.m_colorMask[0] && !this.m_colorMask[1] && !this.m_colorMask[2] && !this.m_colorMask[3];

            if (!colorBuf.isEmpty() && !maskZero) {
                /** @type {Array<number>} */ var colorArea = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(colorBuf));
                access = colorBuf.getSubregion(colorArea);
                var color = value;

                if (this.m_sRGBUpdateEnabled && access.raw().getFormat().isSRGB())
                    color = tcuTextureUtil.linearToSRGB(color);

                if (!maskUsed)
                    access.clear(color);
                else {
                    for (var y = 0; y < access.raw().getDepth(); y++)
                        for (var x = 0; x < access.raw().getHeight(); x++)
                            for (var s = 0; s < access.getNumSamples(); s++)
                                access.raw().setPixel(tcuTextureUtil.select(color, access.raw().getPixel(s, x, y), this.m_colorMask), s, x, y);
                }
            }
        } else {
            if (buffer !== gl.DEPTH)
                throw new Error('Unexpected buffer type: ' + buffer);

            /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var depthBuf = this.getDrawDepthbuffer();

            if (!depthBuf.isEmpty() && this.m_depthMask) {
                /** @type {Array<number>} */ var area = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(depthBuf));
                access = depthBuf.getSubregion(area);
                /** @type {number} */ var depth = value[0];

                for (var y = 0; y < access.raw().getDepth(); y++)
                    for (var x = 0; x < access.raw().getHeight(); x++)
                        for (var s = 0; s < access.getNumSamples(); s++)
                            sglrReferenceContext.writeDepthOnly(access, s, x, y, depth);
            }
        }
    };

    /**
    * @param {number} buffer
    * @param {number} drawbuffer
    * @param {Array<number>} value
    */
    sglrReferenceContext.ReferenceContext.prototype.clearBufferuiv = function(buffer, drawbuffer, value) {
        if (this.conditionalSetError(buffer != gl.COLOR, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(drawbuffer != 0, gl.INVALID_VALUE))
            return;

        /** @type {Array<number>} */ var baseArea = this.m_scissorEnabled ? this.m_scissorBox : [0, 0, 0x7fffffff, 0x7fffffff];

        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var colorBuf = this.getDrawColorbuffer();
        /** @type {boolean} */ var maskUsed = !this.m_colorMask[0] || !this.m_colorMask[1] || !this.m_colorMask[2] || !this.m_colorMask[3];
        /** @type {boolean} */ var maskZero = !this.m_colorMask[0] && !this.m_colorMask[1] && !this.m_colorMask[2] && !this.m_colorMask[3];

        if (!colorBuf.isEmpty() && !maskZero) {
            /** @type {Array<number>} */ var colorArea = deMath.intersect(baseArea, sglrReferenceContext.getBufferRect(colorBuf));
            /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var access = colorBuf.getSubregion(colorArea);

            if (!maskUsed)
                access.clear(value);
            else {
            for (var y = 0; y < access.raw().getDepth(); y++)
                for (var x = 0; x < access.raw().getHeight(); x++)
                    for (var s = 0; s < access.getNumSamples(); s++)
                        access.raw().setPixel(tcuTextureUtil.select(value, access.raw().getPixel(s, x, y), this.m_colorMask), s, x, y);
            }
        }
    };

    /**
    * @param {number} buffer
    * @param {number} drawbuffer
    * @param {number} depth
    * @param {number} stencil
    */
    sglrReferenceContext.ReferenceContext.prototype.clearBufferfi = function(buffer, drawbuffer, depth, stencil) {
        if (this.conditionalSetError(buffer != gl.DEPTH_STENCIL, gl.INVALID_ENUM))
            return;
        this.clearBufferfv(gl.DEPTH, drawbuffer, [depth]);
        this.clearBufferiv(gl.STENCIL, drawbuffer, [stencil]);
    };

    /**
    * @param {number} target
    * @param {number} attachment
    * @param {sglrReferenceContext.TexTarget} textarget
    * @param {sglrReferenceContext.TextureContainer} texture
    * @param {number} level
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.framebufferTexture2D = function(target, attachment, textarget, texture, level) {
        if (attachment == gl.DEPTH_STENCIL_ATTACHMENT) {
            // Attach to both depth and stencil.
            this.framebufferTexture2D(target, gl.DEPTH_ATTACHMENT, textarget, texture, level);
            this.framebufferTexture2D(target, gl.STENCIL_ATTACHMENT, textarget, texture, level);
        } else {
            /** @type {sglrReferenceContext.AttachmentPoint} */ var point = sglrReferenceContext.mapGLAttachmentPoint(attachment);
            /** @type {sglrReferenceContext.TexTarget} */ var fboTexTarget = sglrReferenceContext.mapGLFboTexTarget(textarget);

            if (this.conditionalSetError(target != gl.FRAMEBUFFER &&
                        target != gl.DRAW_FRAMEBUFFER &&
                        target != gl.READ_FRAMEBUFFER, gl.INVALID_ENUM))
                return;
            if (this.conditionalSetError(point == undefined, gl.INVALID_ENUM))
                return;

            // Select binding point.
            /** @type {sglrReferenceContext.Framebuffer} */ var framebufferBinding = (target == gl.FRAMEBUFFER || target == gl.DRAW_FRAMEBUFFER) ? this.m_drawFramebufferBinding : this.m_readFramebufferBinding;
            if (this.conditionalSetError(!framebufferBinding, gl.INVALID_OPERATION))
                return;

            if (texture) {
                if (this.conditionalSetError(level != 0, gl.INVALID_VALUE))
                    return;

                if (texture.getType() == sglrReferenceContext.TextureType.TYPE_2D) {
                    if (this.conditionalSetError(fboTexTarget != sglrReferenceContext.TexTarget.TEXTARGET_2D, gl.INVALID_OPERATION))
                        return;
                } else {
                    if (!texture.getType() == sglrReferenceContext.TextureType.TYPE_CUBE_MAP)
                        throw new Error('Unsupported texture type');
                    if (this.conditionalSetError(!deMath.deInRange32(fboTexTarget, sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_POSITIVE_X, sglrReferenceContext.TexTarget.TEXTARGET_CUBE_MAP_NEGATIVE_Z), gl.INVALID_OPERATION))
                        return;
                }
            }

            /** @type {sglrReferenceContext.Attachment} */ var fboAttachment = new sglrReferenceContext.Attachment();

            if (texture) {
                fboAttachment.type = sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_TEXTURE;
                fboAttachment.object = texture;
                fboAttachment.texTarget = fboTexTarget;
                fboAttachment.level = level;
            }
            framebufferBinding.setAttachment(point, fboAttachment);
        }
    };

    /**
    * @param {number} target
    * @param {number} attachment
    * @param {sglrReferenceContext.TextureContainer} texture
    * @param {number} level
    * @param {number} layer
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.framebufferTextureLayer = function(target, attachment, texture, level, layer) {
        if (attachment == gl.DEPTH_STENCIL_ATTACHMENT) {
            // Attach to both depth and stencil.
            this.framebufferTextureLayer(target, gl.DEPTH_ATTACHMENT, texture, level, layer);
            this.framebufferTextureLayer(target, gl.STENCIL_ATTACHMENT, texture, level, layer);
        } else {
            /** @type {sglrReferenceContext.AttachmentPoint} */ var point = sglrReferenceContext.mapGLAttachmentPoint(attachment);

            if (this.conditionalSetError(target != gl.FRAMEBUFFER &&
                        target != gl.DRAW_FRAMEBUFFER &&
                        target != gl.READ_FRAMEBUFFER, gl.INVALID_ENUM))
                return;
            if (this.conditionalSetError(point === undefined, gl.INVALID_ENUM))
                return;

            // Select binding point.
            /** @type {sglrReferenceContext.Framebuffer} */ var framebufferBinding = (target == gl.FRAMEBUFFER || target == gl.DRAW_FRAMEBUFFER) ? this.m_drawFramebufferBinding : this.m_readFramebufferBinding;
            if (this.conditionalSetError(!framebufferBinding, gl.INVALID_OPERATION))
                return;

            if (texture) {
                if (this.conditionalSetError(level != 0, gl.INVALID_VALUE))
                    return;

                if (this.conditionalSetError(texture.getType() != sglrReferenceContext.TextureType.TYPE_2D_ARRAY &&
                            texture.getType() != sglrReferenceContext.TextureType.TYPE_3D &&
                            texture.getType() != sglrReferenceContext.TextureType.TYPE_CUBE_MAP_ARRAY, gl.INVALID_OPERATION))
                    return;

                if (texture.getType() == sglrReferenceContext.TextureType.TYPE_2D_ARRAY || texture.getType() == sglrReferenceContext.TextureType.TYPE_CUBE_MAP_ARRAY) {
                    if (this.conditionalSetError((layer < 0) || (layer >= gl.MAX_ARRAY_TEXTURE_LAYERS), gl.INVALID_VALUE))
                        return;
                    if (this.conditionalSetError((level < 0) || (level > Math.floor(Math.log2(gl.MAX_TEXTURE_SIZE))), gl.INVALID_VALUE))
                        return;
                } else if (texture.getType() == sglrReferenceContext.TextureType.TYPE_3D) {
                    if (this.conditionalSetError((layer < 0) || (layer >= gl.MAX_3D_TEXTURE_SIZE), gl.INVALID_VALUE))
                        return;
                    if (this.conditionalSetError((level < 0) || (level > Math.floor(Math.log2(gl.MAX_3D_TEXTURE_SIZE))), gl.INVALID_VALUE))
                        return;
                }
            }

            /** @type {sglrReferenceContext.Attachment} */ var fboAttachment = new sglrReferenceContext.Attachment();

            if (texture) {
                fboAttachment.type = sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_TEXTURE;
                fboAttachment.object = texture;
                fboAttachment.texTarget = sglrReferenceContext.texLayeredTypeToTarget(texture.getType());
                fboAttachment.level = level;
                fboAttachment.layer = layer;
            }
            framebufferBinding.setAttachment(point, fboAttachment);

        }
    };

    /**
    * @param {number} target
    * @param {number} attachment
    * @param {number} renderbuffertarget
    * @param {sglrReferenceContext.Renderbuffer} renderbuffer
    */
    sglrReferenceContext.ReferenceContext.prototype.framebufferRenderbuffer = function(target, attachment, renderbuffertarget, renderbuffer) {
        if (attachment == gl.DEPTH_STENCIL_ATTACHMENT) {
            // Attach both to depth and stencil.
            this.framebufferRenderbuffer(target, gl.DEPTH_ATTACHMENT, renderbuffertarget, renderbuffer);
            this.framebufferRenderbuffer(target, gl.STENCIL_ATTACHMENT, renderbuffertarget, renderbuffer);
        } else {
            /** @type {sglrReferenceContext.AttachmentPoint} */ var point = sglrReferenceContext.mapGLAttachmentPoint(attachment);

            if (this.conditionalSetError(target != gl.FRAMEBUFFER &&
                        target != gl.DRAW_FRAMEBUFFER &&
                        target != gl.READ_FRAMEBUFFER, gl.INVALID_ENUM))
                return;
            if (this.conditionalSetError(point == undefined, gl.INVALID_ENUM))
                return;

            // Select binding point.
            /** @type {sglrReferenceContext.Framebuffer} */ var framebufferBinding = (target == gl.FRAMEBUFFER || target == gl.DRAW_FRAMEBUFFER) ? this.m_drawFramebufferBinding : this.m_readFramebufferBinding;
            if (this.conditionalSetError(!framebufferBinding, gl.INVALID_OPERATION))
                return;

            if (renderbuffer) {
                if (this.conditionalSetError(renderbuffertarget != gl.RENDERBUFFER, gl.INVALID_ENUM))
                    return;
            }

            /** @type {sglrReferenceContext.Attachment} */ var fboAttachment = new sglrReferenceContext.Attachment();

            if (renderbuffer) {
                fboAttachment.type = sglrReferenceContext.AttachmentType.ATTACHMENTTYPE_RENDERBUFFER;
                fboAttachment.object = renderbuffer;
            }
            framebufferBinding.setAttachment(point, fboAttachment);
        }
    };

    /**
    * @param {number} target
    * @param {number} internalformat
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.ReferenceContext.prototype.renderbufferStorage = function(target, internalformat, width, height) {
        /** @type {tcuTexture.TextureFormat} */ var format = gluTextureUtil.mapGLInternalFormat(internalformat);

        if (this.conditionalSetError(target != gl.RENDERBUFFER, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError(!this.m_renderbufferBinding, gl.INVALID_OPERATION))
            return;
        if (this.conditionalSetError(!deMath.deInRange32(width, 0, this.m_limits.maxRenderbufferSize) ||
                    !deMath.deInRange32(height, 0, this.m_limits.maxRenderbufferSize),
                    gl.INVALID_OPERATION))
            return;
        if (this.conditionalSetError(!format, gl.INVALID_ENUM))
            return;

        this.m_renderbufferBinding.setStorage(format, width, height);
    };

    /**
     * @param {number} target
     * @param {number} samples
     * @param {number} internalformat
     * @param {number} width
     * @param {number} height
     */
    sglrReferenceContext.ReferenceContext.prototype.renderbufferStorageMultisample = function(target, samples, internalformat, width, height) {
        this.renderbufferStorage(target, internalformat, width, height);
    };

    /**
    * @param {rrRenderer.PrimitiveType} derivedType
    * @return {rrRenderer.PrimitiveType}
    * @throws {Error}
    */
    sglrReferenceContext.getPrimitiveBaseType = function(derivedType) {
        switch (derivedType) {
            case rrRenderer.PrimitiveType.TRIANGLES:
            case rrRenderer.PrimitiveType.TRIANGLE_STRIP:
            case rrRenderer.PrimitiveType.TRIANGLE_FAN:
                return rrRenderer.PrimitiveType.TRIANGLES;

            case rrRenderer.PrimitiveType.LINES:
            case rrRenderer.PrimitiveType.LINE_STRIP:
            case rrRenderer.PrimitiveType.LINE_LOOP:
                return rrRenderer.PrimitiveType.LINES;

            case rrRenderer.PrimitiveType.POINTS:
                return rrRenderer.PrimitiveType.POINTS;

            default:
                throw new Error('Unrecognized primitive type:' + derivedType);
        }
    };

    /**
    * createProgram
    * @param {sglrShaderProgram.ShaderProgram} program
    * @return {sglrShaderProgram.ShaderProgram}
    */
    sglrReferenceContext.ReferenceContext.prototype.createProgram = function(program) {
        return program;
    };

    /**
     * deleteProgram
     * @param {sglrShaderProgram.ShaderProgram} program
     */
    sglrReferenceContext.ReferenceContext.prototype.deleteProgram = function(program) {};

    /**
    * @param {sglrShaderProgram.ShaderProgram} program
    */
    sglrReferenceContext.ReferenceContext.prototype.useProgram = function(program) {
        this.m_currentProgram = program;
    };

    /**
    * Draws quads from vertex arrays
    * @param {number} primitive GL primitive type to draw with.
    * @param {number} first First vertex to begin drawing with
    * @param {number} count How many vertices to draw (not counting vertices before first)
    */
    sglrReferenceContext.ReferenceContext.prototype.drawArrays = function(primitive, first, count) {
        this.drawQuads(primitive, first, count, 1);
    };

    /**
    * Draws quads from vertex arrays
    * @param {number} primitive GL primitive type to draw with.
    * @param {(number|rrRenderer.DrawIndices)} first First vertex to begin drawing with
    * @param {number} count Number of vertices
    * @param {number=} instances Number of instances
    */
    sglrReferenceContext.ReferenceContext.prototype.drawQuads = function(primitive, first, count, instances) {
        // undefined results
        if (!this.m_currentProgram)
            return;

        if (typeof instances === 'undefined')
            instances = 1;

        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var colorBuf0 = this.getDrawColorbuffer();
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var depthBuf = this.getDrawDepthbuffer();
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var stencilBuf = this.getDrawStencilbuffer();
        /** @type {boolean} */ var hasStencil = /** @type {!boolean} */ (stencilBuf && !stencilBuf.isEmpty());
        /** @type {number} */ var stencilBits = (hasStencil) ? stencilBuf.raw().getFormat().getNumStencilBits() : 0;

        /** @type {rrRenderer.RenderTarget} */ var renderTarget = new rrRenderer.RenderTarget(colorBuf0,
                                                    depthBuf,
                                                    stencilBuf);
        /** @type {sglrShaderProgram.ShaderProgram} */ var program = this.m_currentProgram;

        /*new rrRenderer.Program(
        *   this.m_currentProgram.getVertexShader(),
        *   this.m_currentProgram.getFragmentShader());*/

        /** @type {rrRenderState.ViewportState} */ var viewportState = new rrRenderState.ViewportState(colorBuf0);
        /** @type {rrRenderState.RenderState} */ var state = new rrRenderState.RenderState(viewportState);

        /** @type {Array<rrVertexAttrib.VertexAttrib>} */ var vertexAttribs = [];

        // Gen state
        /** @type {rrRenderer.PrimitiveType} */ var baseType = rrRenderer.PrimitiveType.TRIANGLES;
        /** @type {boolean} */ var polygonOffsetEnabled =
            (baseType == rrRenderer.PrimitiveType.TRIANGLES) ?
            (this.m_polygonOffsetFillEnabled) :
            (false);

        //state.cullMode = m_cullMode

        state.fragOps.scissorTestEnabled = this.m_scissorEnabled;
        state.fragOps.scissorRectangle = new rrRenderState.WindowRectangle(this.m_scissorBox);

        state.fragOps.numStencilBits = stencilBits;
        state.fragOps.stencilTestEnabled = this.m_stencilTestEnabled;

        for (var key in rrDefs.FaceType) {
            /** @type {number} */ var faceType = rrDefs.FaceType[key];
            state.fragOps.stencilStates[faceType].compMask = this.m_stencil[faceType].opMask;
            state.fragOps.stencilStates[faceType].writeMask = this.m_stencil[faceType].writeMask;
            state.fragOps.stencilStates[faceType].ref = this.m_stencil[faceType].ref;
            state.fragOps.stencilStates[faceType].func = sglrReferenceUtils.mapGLTestFunc(this.m_stencil[faceType].func);
            state.fragOps.stencilStates[faceType].sFail = sglrReferenceUtils.mapGLStencilOp(this.m_stencil[faceType].opStencilFail);
            state.fragOps.stencilStates[faceType].dpFail = sglrReferenceUtils.mapGLStencilOp(this.m_stencil[faceType].opDepthFail);
            state.fragOps.stencilStates[faceType].dpPass = sglrReferenceUtils.mapGLStencilOp(this.m_stencil[faceType].opDepthPass);
        }

        state.fragOps.depthTestEnabled = this.m_depthTestEnabled;
        state.fragOps.depthFunc = sglrReferenceUtils.mapGLTestFunc(this.m_depthFunc);
        state.fragOps.depthMask = this.m_depthMask;

        state.fragOps.blendMode = this.m_blendEnabled ? rrRenderState.BlendMode.STANDARD : rrRenderState.BlendMode.NONE;
        state.fragOps.blendRGBState.equation = sglrReferenceUtils.mapGLBlendEquation(this.m_blendModeRGB);
        state.fragOps.blendRGBState.srcFunc = sglrReferenceUtils.mapGLBlendFunc(this.m_blendFactorSrcRGB);
        state.fragOps.blendRGBState.dstFunc = sglrReferenceUtils.mapGLBlendFunc(this.m_blendFactorDstRGB);
        state.fragOps.blendAState.equation = sglrReferenceUtils.mapGLBlendEquation(this.m_blendModeAlpha);
        state.fragOps.blendAState.srcFunc = sglrReferenceUtils.mapGLBlendFunc(this.m_blendFactorSrcAlpha);
        state.fragOps.blendAState.dstFunc = sglrReferenceUtils.mapGLBlendFunc(this.m_blendFactorDstAlpha);
        state.fragOps.blendColor = this.m_blendColor;

        state.fragOps.colorMask = this.m_colorMask;

        state.viewport.rect = new rrRenderState.WindowRectangle(this.m_viewport);
        state.viewport.zn = this.m_depthRangeNear;
        state.viewport.zf = this.m_depthRangeFar;

        //state.point.pointSize = this.m_pointSize;
        state.line.lineWidth = this.m_lineWidth;

        state.fragOps.polygonOffsetEnabled = polygonOffsetEnabled;
        state.fragOps.polygonOffsetFactor = this.m_polygonOffsetFactor;
        state.fragOps.polygonOffsetUnits = this.m_polygonOffsetUnits;

        state.provokingVertexConvention = (this.m_provokingFirstVertexConvention) ? (rrDefs.ProvokingVertex.PROVOKINGVERTEX_FIRST) : (rrDefs.ProvokingVertex.PROVOKINGVERTEX_LAST);

        // gen attributes
        /** @type {sglrReferenceContext.VertexArray} */ var vao = this.m_vertexArrayBinding;
        for (var ndx = 0; ndx < vao.m_arrays.length; ++ndx) {
            vertexAttribs[ndx] = new rrVertexAttrib.VertexAttrib();
            if (!vao.m_arrays[ndx].enabled) {
                vertexAttribs[ndx].type = rrVertexAttrib.VertexAttribType.DONT_CARE; // reading with wrong type is allowed, but results are undefined
                vertexAttribs[ndx].generic = this.m_currentAttribs[ndx];
            } else {
                vertexAttribs[ndx].type = (vao.m_arrays[ndx].integer) ?
                (sglrReferenceUtils.mapGLPureIntegerVertexAttributeType(vao.m_arrays[ndx].type)) :
                (sglrReferenceUtils.mapGLFloatVertexAttributeType(vao.m_arrays[ndx].type, vao.m_arrays[ndx].normalized, vao.m_arrays[ndx].size));
                vertexAttribs[ndx].size = sglrReferenceUtils.mapGLSize(vao.m_arrays[ndx].size);
                vertexAttribs[ndx].stride = vao.m_arrays[ndx].stride;
                vertexAttribs[ndx].instanceDivisor = vao.m_arrays[ndx].divisor;
                vertexAttribs[ndx].pointer = vao.m_arrays[ndx].bufferBinding.getData();
                vertexAttribs[ndx].offset = vao.m_arrays[ndx].offset;
                vertexAttribs[ndx].componentCount = vao.m_arrays[ndx].size;
            }
        }

        // Set shader samplers
        for (var uniformNdx = 0; uniformNdx < this.m_currentProgram.m_uniforms.length; ++uniformNdx) {
            /** @type {number} */ var texNdx = this.m_currentProgram.m_uniforms[uniformNdx].value[0];

            switch (this.m_currentProgram.m_uniforms[uniformNdx].type) {
                case gluShaderUtil.DataType.SAMPLER_2D:
                case gluShaderUtil.DataType.UINT_SAMPLER_2D:
                case gluShaderUtil.DataType.INT_SAMPLER_2D: {
                    /** @type {sglrReferenceContext.Texture2D} */ var tex;

                    if (texNdx >= 0 && texNdx < this.m_textureUnits.length)
                        tex = /** @type {sglrReferenceContext.Texture2D} */ (this.m_textureUnits[texNdx].tex2DBinding.texture);

                    if (tex && tex.isComplete()) {
                        tex.updateView();
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = tex;
                    } else
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = this.m_emptyTex2D.texture;

                    break;
                }
                case gluShaderUtil.DataType.SAMPLER_CUBE:
                case gluShaderUtil.DataType.UINT_SAMPLER_CUBE:
                case gluShaderUtil.DataType.INT_SAMPLER_CUBE: {
                    /** @type {sglrReferenceContext.TextureCube} */ var texCube;

                    if (texNdx >= 0 && texNdx < this.m_textureUnits.length)
                        texCube = /** @type {sglrReferenceContext.TextureCube} */ (this.m_textureUnits[texNdx].texCubeBinding.texture);

                    if (texCube && texCube.isComplete()) {
                        texCube.updateView();
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = texCube;
                    } else
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = this.m_emptyTexCube.texture;

                    break;
                }
                case gluShaderUtil.DataType.SAMPLER_2D_ARRAY:
                case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY:
                case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY: {
                    /** @type {sglrReferenceContext.Texture2DArray} */ var tex2DArray;

                    if (texNdx >= 0 && texNdx < this.m_textureUnits.length)
                        tex2DArray = /** @type {sglrReferenceContext.Texture2DArray} */ (this.m_textureUnits[texNdx].tex2DArrayBinding.texture);

                    if (tex2DArray && tex2DArray.isComplete()) {
                        tex2DArray.updateView();
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = tex2DArray;
                    } else
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = this.m_emptyTex2DArray.texture;

                    break;
                }
                case gluShaderUtil.DataType.SAMPLER_3D:
                case gluShaderUtil.DataType.UINT_SAMPLER_3D:
                case gluShaderUtil.DataType.INT_SAMPLER_3D: {
                    /** @type {sglrReferenceContext.Texture3D} */ var tex3D;

                    if (texNdx >= 0 && texNdx < this.m_textureUnits.length)
                        tex3D = /** @type {sglrReferenceContext.Texture3D} */ (this.m_textureUnits[texNdx].tex3DBinding.texture);

                    if (tex3D && tex3D.isComplete()) {
                        tex3D.updateView();
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = tex3D;
                    } else
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler = this.m_emptyTex3D.texture;

                    break;
                }
                /* TODO: Port
                case gluShaderUtil.DataType.SAMPLER_CUBE_ARRAY:
                case gluShaderUtil.DataType.UINT_SAMPLER_CUBE_ARRAY:
                case gluShaderUtil.DataType.INT_SAMPLER_CUBE_ARRAY:{
                    rc::TextureCubeArray* tex = DE_NULL;

                    if (texNdx >= 0 && (size_t)texNdx < m_textureUnits.length)
                        tex = (this.m_textureUnits[texNdx].texCubeArrayBinding) ? (this.m_textureUnits[texNdx].texCubeArrayBinding) : (&this.m_textureUnits[texNdx].defaultCubeArrayTex);

                    if (tex && tex.isComplete()) {
                        tex.updateView();
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler.texCubeArray = tex;
                    } else
                        this.m_currentProgram.m_uniforms[uniformNdx].sampler.texCubeArray = &this.m_emptyTexCubeArray;

                    break;
                }
                */
                default:
                    // nothing
                    break;
            }
        }

        var primitiveType = sglrReferenceUtils.mapGLPrimitiveType(primitive);

        var renderFunction = rrRenderer.drawTriangles;
        if (primitiveType == rrRenderer.PrimitiveType.LINES ||
            primitiveType == rrRenderer.PrimitiveType.LINE_STRIP ||
            primitiveType == rrRenderer.PrimitiveType.LINE_LOOP)
            renderFunction = rrRenderer.drawLines;
        else if (primitiveType == rrRenderer.PrimitiveType.POINTS)
            renderFunction = rrRenderer.drawPoints;

        for (var instanceID = 0; instanceID < instances; instanceID++)
            renderFunction(state, renderTarget, program, vertexAttribs, primitiveType, first, count, instanceID);
    };

    /**
    * @param {Array<number>} rect
    * @return {boolean}
    */
    sglrReferenceContext.isEmpty = function(rect) { return rect[2] == 0 || rect[3] == 0; };

    /**
    * @param {number} mask
    * @param {Array<number>} srcRect
    * @param {Array<number>} dstRect
    * @param {boolean} flipX
    * @param {boolean} flipY
    * @throws {Error}
    */
    sglrReferenceContext.ReferenceContext.prototype.blitResolveMultisampleFramebuffer = function(mask, srcRect, dstRect, flipX, flipY) {
        throw new Error('Unimplemented');
    };

    /**
    * @param {number} srcX0
    * @param {number} srcY0
    * @param {number} srcX1
    * @param {number} srcY1
    * @param {number} dstX0
    * @param {number} dstY0
    * @param {number} dstX1
    * @param {number} dstY1
    * @param {number} mask
    * @param {number} filter
    */
    sglrReferenceContext.ReferenceContext.prototype.blitFramebuffer = function(srcX0, srcY0, srcX1, srcY1, dstX0, dstY0, dstX1, dstY1, mask, filter) {
        // p0 in inclusive, p1 exclusive.
        // Negative width/height means swap.
        /** @type {boolean} */ var swapSrcX = srcX1 < srcX0;
        /** @type {boolean} */ var swapSrcY = srcY1 < srcY0;
        /** @type {boolean} */ var swapDstX = dstX1 < dstX0;
        /** @type {boolean} */ var swapDstY = dstY1 < dstY0;
        /** @type {number} */ var srcW = Math.abs(srcX1 - srcX0);
        /** @type {number} */ var srcH = Math.abs(srcY1 - srcY0);
        /** @type {number} */ var dstW = Math.abs(dstX1 - dstX0);
        /** @type {number} */ var dstH = Math.abs(dstY1 - dstY0);
        /** @type {boolean} */ var scale = srcW != dstW || srcH != dstH;
        /** @type {number} */ var srcOriginX = swapSrcX ? srcX1 : srcX0;
        /** @type {number} */ var srcOriginY = swapSrcY ? srcY1 : srcY0;
        /** @type {number} */ var dstOriginX = swapDstX ? dstX1 : dstX0;
        /** @type {number} */ var dstOriginY = swapDstY ? dstY1 : dstY0;
        /** @type {Array<number>} */ var srcRect = [srcOriginX, srcOriginY, srcW, srcH];
        /** @type {Array<number>} */ var dstRect = [dstOriginX, dstOriginY, dstW, dstH];

        if (this.conditionalSetError(filter != gl.NEAREST && filter != gl.LINEAR, gl.INVALID_ENUM))
            return;
        if (this.conditionalSetError((mask & (gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT)) != 0 && filter != gl.NEAREST, gl.INVALID_OPERATION))
            return;

        // Validate that both targets are complete.
        if (this.conditionalSetError(this.checkFramebufferStatus(gl.DRAW_FRAMEBUFFER) != gl.FRAMEBUFFER_COMPLETE ||
                    this.checkFramebufferStatus(gl.READ_FRAMEBUFFER) != gl.FRAMEBUFFER_COMPLETE, gl.INVALID_OPERATION))
            return;

        // Check samples count is valid
        if (this.conditionalSetError(this.getDrawColorbuffer().getNumSamples() != 1, gl.INVALID_OPERATION))
            return;

        // Check size restrictions of multisampled case
        if (this.getReadColorbuffer().getNumSamples() != 1) {
            // Src and Dst rect dimensions must be the same
            if (this.conditionalSetError(srcW != dstW || srcH != dstH, gl.INVALID_OPERATION))
                return;

            // sglrReferenceContext.Framebuffer formats must match
            if (mask & gl.COLOR_BUFFER_BIT)
                if (this.conditionalSetError(this.getReadColorbuffer().raw().getFormat() != this.getDrawColorbuffer().raw().getFormat(), gl.INVALID_OPERATION))
                    return;
            if (mask & gl.DEPTH_BUFFER_BIT)
                if (this.conditionalSetError(this.getReadDepthbuffer().raw().getFormat() != this.getDrawDepthbuffer().raw().getFormat(), gl.INVALID_OPERATION))
                    return;
            if (mask & gl.STENCIL_BUFFER_BIT)
            if (this.conditionalSetError(this.getReadStencilbuffer().raw().getFormat() != this.getDrawStencilbuffer().raw().getFormat(), gl.INVALID_OPERATION))
                return;
        }

        // Compute actual source rect.
        srcRect = (mask & gl.COLOR_BUFFER_BIT) ? deMath.intersect(srcRect, sglrReferenceContext.getBufferRect(this.getReadColorbuffer())) : srcRect;
        srcRect = (mask & gl.DEPTH_BUFFER_BIT) ? deMath.intersect(srcRect, sglrReferenceContext.getBufferRect(this.getReadDepthbuffer())) : srcRect;
        srcRect = (mask & gl.STENCIL_BUFFER_BIT) ? deMath.intersect(srcRect, sglrReferenceContext.getBufferRect(this.getReadStencilbuffer())) : srcRect;

        // Compute destination rect.
        dstRect = (mask & gl.COLOR_BUFFER_BIT) ? deMath.intersect(dstRect, sglrReferenceContext.getBufferRect(this.getDrawColorbuffer())) : dstRect;
        dstRect = (mask & gl.DEPTH_BUFFER_BIT) ? deMath.intersect(dstRect, sglrReferenceContext.getBufferRect(this.getDrawDepthbuffer())) : dstRect;
        dstRect = (mask & gl.STENCIL_BUFFER_BIT) ? deMath.intersect(dstRect, sglrReferenceContext.getBufferRect(this.getDrawStencilbuffer())) : dstRect;
        dstRect = this.m_scissorEnabled ? deMath.intersect(dstRect, this.m_scissorBox) : dstRect;

        if (sglrReferenceContext.isEmpty(srcRect) || sglrReferenceContext.isEmpty(dstRect))
            return; // Don't attempt copy.

        // Multisampled read buffer is a special case
        if (this.getReadColorbuffer().getNumSamples() != 1) {
            /** @type {boolean} */ var swapX = swapSrcX ^ swapDstX ? true : false;
            /** @type {boolean} */ var swapY = swapSrcY ^ swapDstY ? true : false;
            var error = this.blitResolveMultisampleFramebuffer(mask, srcRect, dstRect, swapX, swapY);

            if (error != gl.NO_ERROR)
                this.setError(error);

            return;
        }

        // \note Multisample pixel buffers can now be accessed like non-multisampled because multisample read buffer case is already handled. => sample count must be 1

        // Coordinate transformation:
        // Dst offset space -> dst rectangle space -> src rectangle space -> src offset space.
        /** @type {tcuMatrix.Matrix} */ var matrix = tcuMatrixUtil.translationMatrix([srcX0 - srcRect[0], srcY0 - srcRect[1]]);
        matrix = tcuMatrix.multiply(matrix, tcuMatrix.matrixFromVector(3, 3, [(srcX1 - srcX0) / (dstX1 - dstX0), (srcY1 - srcY0) / (dstY1 - dstY0), 1]));
        matrix = tcuMatrix.multiply(matrix, tcuMatrixUtil.translationMatrix([dstRect[0] - dstX0, dstRect[1] - dstY0]));

        /**
         * @param {number} x
         * @param {number} y
         * @return {number}
         */
        var transform = function(x, y) { return matrix.get(x, y); };

        /** @type {number} */ var dX;
        /** @type {number} */ var dY;
        /** @type {number} */ var sX;
        /** @type {number} */ var sY;
        /** @type {tcuTexture.PixelBufferAccess|rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var src;
        /** @type {tcuTexture.PixelBufferAccess|rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var dst;

        if (mask & gl.COLOR_BUFFER_BIT) {
            src = tcuTextureUtil.getSubregion(this.getReadColorbuffer().toSinglesampleAccess(), srcRect[0], srcRect[1], 0, srcRect[2], srcRect[3], 1);
            dst = tcuTextureUtil.getSubregion(this.getDrawColorbuffer().toSinglesampleAccess(), dstRect[0], dstRect[1], 0, dstRect[2], dstRect[3], 1);
            /** @type {tcuTexture.TextureChannelClass} */ var dstClass = tcuTexture.getTextureChannelClass(dst.getFormat().type);
            /** @type {boolean} */ var dstIsFloat = dstClass == tcuTexture.TextureChannelClass.FLOATING_POINT ||
                                                        dstClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT ||
                                                        dstClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT;
            /** @type {tcuTexture.FilterMode} */ var sFilter = (scale && filter == gl.LINEAR) ? tcuTexture.FilterMode.LINEAR : tcuTexture.FilterMode.NEAREST;
            /** @type {tcuTexture.Sampler} */ var sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
                                                        sFilter, sFilter, 0.0 /* lod threshold */, false /* non-normalized coords */);
            /** @type {boolean} */ var srcIsSRGB = src.getFormat().order == tcuTexture.ChannelOrder.sRGB || src.getFormat().order == tcuTexture.ChannelOrder.sRGBA;
            /** @type {boolean} */ var dstIsSRGB = dst.getFormat().order == tcuTexture.ChannelOrder.sRGB || dst.getFormat().order == tcuTexture.ChannelOrder.sRGBA;
            /** @type {boolean} */ var convertSRGB = this.m_sRGBUpdateEnabled;

            // \note We don't check for unsupported conversions, unlike spec requires.

            for (var yo = 0; yo < dstRect[3]; yo++) {
                for (var xo = 0; xo < dstRect[2]; xo++) {
                    dX = xo + 0.5;
                    dY = yo + 0.5;

                    // \note Only affine part is used.
                    sX = transform(0, 0) * dX + transform(0, 1) * dY + transform(0, 2);
                    sY = transform(1, 0) * dX + transform(1, 1) * dY + transform(1, 2);

                    // do not copy pixels outside the modified source region (modified by buffer intersection)
                    if (sX < 0.0 || sX >= srcRect[2] ||
                        sY < 0.0 || sY >= srcRect[3])
                        continue;

                    if (dstIsFloat || srcIsSRGB || filter == tcuTexture.FilterMode.LINEAR) {
                        /** @type {Array<number>} */ var p = src.sample2D(sampler, sampler.minFilter, sX, sY, 0);
                        dst.setPixel((dstIsSRGB && convertSRGB) ? tcuTextureUtil.linearToSRGB(p) : p, xo, yo);
                    } else
                        dst.setPixelInt(src.getPixelInt(Math.floor(sX), Math.floor(sY)), xo, yo);
                }
            }
        }

        if ((mask & gl.DEPTH_BUFFER_BIT) && this.m_depthMask) {
            src = this.getReadDepthbuffer().getSubregion(srcRect);
            dst = this.getDrawDepthbuffer().getSubregion(dstRect);

            for (var yo = 0; yo < dstRect[3]; yo++) {
                for (var xo = 0; xo < dstRect[2]; xo++) {
                    var sampleNdx = 0; // multisample read buffer case is already handled

                    dX = xo + 0.5;
                    dY = yo + 0.5;
                    sX = transform(0, 0) * dX + transform(0, 1) * dY + transform(0, 2);
                    sY = transform(1, 0) * dX + transform(1, 1) * dY + transform(1, 2);

                    sglrReferenceContext.writeDepthOnly(dst, sampleNdx, xo, yo, src.raw().getPixel(sampleNdx, Math.floor(sX), Math.floor(sY))[0]);
                }
            }
        }

        if (mask & gl.STENCIL_BUFFER_BIT) {
            src = this.getReadStencilbuffer().getSubregion(srcRect);
            dst = this.getDrawStencilbuffer().getSubregion(dstRect);

            for (var yo = 0; yo < dstRect[3]; yo++) {
                for (var xo = 0; xo < dstRect[2]; xo++) {
                    var sampleNdx = 0; // multisample read buffer case is already handled

                    dX = xo + 0.5;
                    dY = yo + 0.5;
                    sX = transform(0, 0) * dX + transform(0, 1) * dY + transform(0, 2);
                    sY = transform(1, 0) * dX + transform(1, 1) * dY + transform(1, 2);

                    sglrReferenceContext.writeStencilOnly(dst, sampleNdx, xo, yo, src.raw().getPixelInt(sampleNdx, Math.floor(sX), Math.floor(sY))[3], this.m_stencil[rrDefs.FaceType.FACETYPE_FRONT].writeMask);
                }
            }
        }
    };

    /**
    * @param {number} internalFormat
    * @return {tcuTexture.TextureFormat}
    */
    sglrReferenceContext.mapInternalFormat = function(internalFormat) {
        switch (internalFormat) {
            case gl.ALPHA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.A, tcuTexture.ChannelType.UNORM_INT8);
            case gl.LUMINANCE: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.L, tcuTexture.ChannelType.UNORM_INT8);
            case gl.LUMINANCE_ALPHA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.LA, tcuTexture.ChannelType.UNORM_INT8);
            case gl.RGB: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGB, tcuTexture.ChannelType.UNORM_INT8);
            case gl.RGBA: return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);

            default:
                return gluTextureUtil.mapGLInternalFormat(internalFormat);
        }
    };

    /**
    * @param {tcuTexture.PixelBufferAccess} dst
    * @param {tcuTexture.ConstPixelBufferAccess} src
    */
    sglrReferenceContext.depthValueFloatClampCopy = function(dst, src) {
        /** @type {number} */ var width = dst.getWidth();
        /** @type {number} */ var height = dst.getHeight();
        /** @type {number} */ var depth = dst.getDepth();

        DE_ASSERT(src.getWidth() == width && src.getHeight() == height && src.getDepth() == depth);

        // clamping copy
        for (var z = 0; z < depth; z++)
        for (var y = 0; y < height; y++)
        for (var x = 0; x < width; x++) {
            /** @type {Array<number>} */ var data = src.getPixel(x, y, z);
            dst.setPixel([deMath.clamp(data[0], 0.0, 1.0), data[1], data[2], data[3]], x, y, z);
        }
    };

    /**
    * @param {number} target
    * @param {number} level
    * @param {number} internalFormat
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.ReferenceContext.prototype.texImage2DDelegate = function (target, level, internalFormat, width, height) {
        var format;
        var dataType;

        switch (internalFormat)
        {
            case gl.ALPHA:
            case gl.LUMINANCE:
            case gl.LUMINANCE_ALPHA:
            case gl.RGB:
            case gl.RGBA:
                format = internalFormat;
                dataType = GL.UNSIGNED_BYTE;
                break;
            default:
            {
                var transferFmt = gluTextureUtil.getTransferFormat(gluTextureUtil.mapGLInternalFormat(internalFormat));
                format = transferFmt.format;
                dataType = transferFmt.dataType;
                break;
            }
        }
        this.texImage2D(target, level, internalFormat, width, height, 0, format, dataType, null);
    };

    /**
    * @param {number} target
    * @param {number} level
    * @param {number} internalFormat
    * @param {number} width
    * @param {number} height
    * @param {number} border
    * @param {number} format
    * @param {number} type
    * @param {number} pixels
    */
    sglrReferenceContext.ReferenceContext.prototype.texImage2D = function(target, level, internalFormat, width, height, border, format, type, pixels) {
        this.texImage3D(target, level, internalFormat, width, height, 1, border, format, type, pixels);
    };

    sglrReferenceContext.ReferenceContext.prototype.texImage3D = function(target, level, internalFormat, width, height, depth, border, format, type, pixels) {
        /** @type {sglrReferenceContext.TextureUnit} */var unit = this.m_textureUnits[this.m_activeTexture];
        /** @type {ArrayBuffer} */ var data = null;
        /** @type {number} */ var offset = 0;
        /** @type {tcuTexture.PixelBufferAccess} */ var dst;
        /** @type {tcuTexture.ConstPixelBufferAccess} */ var src;
        if (this.m_pixelUnpackBufferBinding) {
            if (this.conditionalSetError(typeof pixels !== 'number', gl.INVALID_VALUE))
                return;
            data = this.m_pixelUnpackBufferBinding.getData();
            offset = pixels;
        } else if (pixels) {
            if (pixels instanceof ArrayBuffer) {
                data = pixels;
                offset = 0;
            } else {
                data = pixels.buffer;
                offset = pixels.byteOffset;
            }
        }
        /** @type {boolean} */ var isDstFloatDepthFormat = (internalFormat == gl.DEPTH_COMPONENT32F || internalFormat == gl.DEPTH32F_STENCIL8); // depth components are limited to [0,1] range

        if (this.conditionalSetError(border != 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(width < 0 || height < 0 || depth < 0 || level < 0, gl.INVALID_VALUE))
            return;

        // Map storage format.
        /** @type {tcuTexture.TextureFormat} */ var storageFmt = sglrReferenceContext.mapInternalFormat(internalFormat);
        if (this.conditionalSetError(!storageFmt, gl.INVALID_ENUM))
            return;

        // Map transfer format.
        /** @type {tcuTexture.TextureFormat} */ var transferFmt = gluTextureUtil.mapGLTransferFormat(format, type);
        if (this.conditionalSetError(!transferFmt, gl.INVALID_ENUM))
            return;

        if (target == gl.TEXTURE_2D) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize || height > this.m_limits.maxTexture2DSize || depth != 1, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.log2(this.m_limits.maxTexture2DSize), gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture2D} */
            var texture = /** @type {sglrReferenceContext.Texture2D} */ (unit.tex2DBinding.texture);

            if (texture.isImmutable()) {
                if (this.conditionalSetError(!texture.hasLevel(level), gl.INVALID_OPERATION))
                    return;

                //NOTE: replaces this: var dst = tcuTexture.PixelBufferAccess.newFromTextureLevel(texture.getLevel(level));
                dst = texture.getLevel(level);

                if (this.conditionalSetError(!storageFmt.isEqual(dst.getFormat()) ||
                            width != dst.getWidth() ||
                            height != dst.getHeight(), gl.INVALID_OPERATION))
                    return;
            } else
                texture.allocLevel(level, storageFmt, width, height);

            if (data) {
                var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
                var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
                var skip = this.m_pixelUnpackSkipRows * rowPitch + this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
                src = new tcuTexture.ConstPixelBufferAccess({
                    format: transferFmt,
                    width: width,
                    height: height,
                    rowPitch: rowPitch,
                    data: data,
                    offset: offset + skip});

                //NOTE: replaces this: var dst = tcuTexture.PixelBufferAccess.newFromTextureLevel(texture.getLevel(level));
                dst = texture.getLevel(level);

                if (isDstFloatDepthFormat)
                    sglrReferenceContext.depthValueFloatClampCopy(dst, src);
                else
                    tcuTextureUtil.copy(dst, src);
            } else {
                // No data supplied, clear to black.

                //NOTE: replaces this: var dst = tcuTexture.PixelBufferAccess.newFromTextureLevel(texture.getLevel(level));
                dst = texture.getLevel(level);
                dst.clear([0.0, 0.0, 0.0, 1.0]);
            }
        } else if (target == gl.TEXTURE_CUBE_MAP_NEGATIVE_X ||
                 target == gl.TEXTURE_CUBE_MAP_POSITIVE_X ||
                 target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Y ||
                 target == gl.TEXTURE_CUBE_MAP_POSITIVE_Y ||
                 target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Z ||
                 target == gl.TEXTURE_CUBE_MAP_POSITIVE_Z) {
            // Validate size and level.
            if (this.conditionalSetError(width != height || width > this.m_limits.maxTextureCubeSize || depth != 1, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTextureCubeSize)), gl.INVALID_VALUE))
                return;

            var textureCube = /** @type {sglrReferenceContext.TextureCube} */ (unit.texCubeBinding.texture);

            var face = sglrReferenceContext.mapGLCubeFace(target);

            if (textureCube.isImmutable()) {
                if (this.conditionalSetError(!textureCube.hasFace(level, face), gl.INVALID_OPERATION))
                    return;

                dst = textureCube.getFace(level, face);

                if (this.conditionalSetError(!storageFmt.isEqual(dst.getFormat()) ||
                            width != dst.getWidth() ||
                            height != dst.getHeight(), gl.INVALID_OPERATION))
                    return;
            } else
                textureCube.allocLevel(level, face, storageFmt, width, height);

            if (data) {
                var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
                var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
                var skip = this.m_pixelUnpackSkipRows * rowPitch + this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
                src = new tcuTexture.ConstPixelBufferAccess({
                    format: transferFmt,
                    width: width,
                    height: height,
                    rowPitch: rowPitch,
                    data: data,
                    offset: offset + skip});

                dst = textureCube.getFace(level, face);

                if (isDstFloatDepthFormat)
                    sglrReferenceContext.depthValueFloatClampCopy(dst, src);
                else
                    tcuTextureUtil.copy(dst, src);
            } else {
                // No data supplied, clear to black.
                dst = textureCube.getFace(level, face);
                dst.clear([0.0, 0.0, 0.0, 1.0]);
            }
        } else if (target == gl.TEXTURE_2D_ARRAY) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize ||
                        height > this.m_limits.maxTexture2DSize ||
                        depth > this.m_limits.maxTexture2DArrayLayers, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTexture2DSize)), gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture2DArray} */
            var texture2DArray = /** @type {sglrReferenceContext.Texture2DArray} */ (unit.tex2DArrayBinding.texture);

            if (texture2DArray.isImmutable()) {
                if (this.conditionalSetError(!texture2DArray.hasLevel(level), gl.INVALID_OPERATION))
                    return;

                dst = texture2DArray.getLevel(level);
                if (this.conditionalSetError(!storageFmt.isEqual(dst.getFormat()) ||
                            width != dst.getWidth() ||
                            height != dst.getHeight() ||
                            depth != dst.getDepth(), gl.INVALID_OPERATION))
                    return;
            } else
                texture2DArray.allocLevel(level, storageFmt, width, height, depth);

            if (data) {
                var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
                var imageHeight = this.m_pixelUnpackImageHeight > 0 ? this.m_pixelUnpackImageHeight : height;
                var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
                var slicePitch = imageHeight * rowPitch;
                var skip = this.m_pixelUnpackSkipImages * slicePitch + this.m_pixelUnpackSkipRows * rowPitch +
                    this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
                src = new tcuTexture.ConstPixelBufferAccess({
                    format: transferFmt,
                    width: width,
                    height: height,
                    depth: depth,
                    rowPitch: rowPitch,
                    slicePitch: slicePitch,
                    data: data,
                    offset: offset + skip});

                dst = texture2DArray.getLevel(level);

                if (isDstFloatDepthFormat)
                    sglrReferenceContext.depthValueFloatClampCopy(dst, src);
                else
                    tcuTextureUtil.copy(dst, src);
            } else {
                // No data supplied, clear to black.
                dst = texture2DArray.getLevel(level);
                dst.clear([0.0, 0.0, 0.0, 1.0]);
            }
        } else if (target == gl.TEXTURE_3D) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture3DSize ||
                        height > this.m_limits.maxTexture3DSize ||
                        depth > this.m_limits.maxTexture3DSize, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTexture3DSize)), gl.INVALID_VALUE))
                return;

            var texture3D = /** @type {sglrReferenceContext.Texture3D} */ (unit.tex3DBinding.texture);

            if (texture3D.isImmutable()) {
                if (this.conditionalSetError(!texture3D.hasLevel(level), gl.INVALID_OPERATION))
                    return;

                dst = texture3D.getLevel(level);
                if (this.conditionalSetError(!storageFmt.isEqual(dst.getFormat()) ||
                            width != dst.getWidth() ||
                            height != dst.getHeight() ||
                            depth != dst.getDepth(), gl.INVALID_OPERATION))
                    return;
            } else
                texture3D.allocLevel(level, storageFmt, width, height, depth);

            if (data) {
                var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
                var imageHeight = this.m_pixelUnpackImageHeight > 0 ? this.m_pixelUnpackImageHeight : height;
                var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
                var slicePitch = imageHeight * rowPitch;
                var skip = this.m_pixelUnpackSkipImages * slicePitch + this.m_pixelUnpackSkipRows * rowPitch +
                    this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
                src = new tcuTexture.ConstPixelBufferAccess({
                    format: transferFmt,
                    width: width,
                    height: height,
                    depth: depth,
                    rowPitch: rowPitch,
                    slicePitch: slicePitch,
                    data: data,
                    offset: offset + skip});

                dst = texture3D.getLevel(level);

                if (isDstFloatDepthFormat)
                    sglrReferenceContext.depthValueFloatClampCopy(dst, src);
                else
                    tcuTextureUtil.copy(dst, src);

            } else {
                // No data supplied, clear to black.
                dst = texture3D.getLevel(level);
                dst.clear([0.0, 0.0, 0.0, 1.0]);
            }
        }
        // else if (target == gl.TEXTURE_CUBE_MAP_ARRAY)
        // {
        //     // Validate size and level.
        //     RC_IF_ERROR(width != height ||
        //                 width > m_limits.maxTexture2DSize ||
        //                 depth % 6 != 0 ||
        //                 depth > m_limits.maxTexture2DArrayLayers, gl.INVALID_VALUE, RC_RET_VOID);
        //     RC_IF_ERROR(level > deLog2Floor32(m_limits.maxTexture2DSize), gl.INVALID_VALUE, RC_RET_VOID);

        //     TextureCubeArray* texture = unit.texCubeArrayBinding ? unit.texCubeArrayBinding : &unit.defaultCubeArrayTex;

        //     if (texture->isImmutable())
        //     {
        //         RC_IF_ERROR(!texture->hasLevel(level), gl.INVALID_OPERATION, RC_RET_VOID);

        //         ConstPixelBufferAccess dst(texture->getLevel(level));
        //         RC_IF_ERROR(storageFmt != dst.getFormat() ||
        //                     width != dst.getWidth() ||
        //                     height != dst.getHeight() ||
        //                     depth != dst.getDepth(), gl.INVALID_OPERATION, RC_RET_VOID);
        //     }
        //     else
        //         texture->allocLevel(level, storageFmt, width, height, depth);

        //     if (unpackPtr)
        //     {
        //         ConstPixelBufferAccess src = getUnpack3DAccess(transferFmt, width, height, depth, unpackPtr);
        //         PixelBufferAccess dst (texture->getLevel(level));

        //         if (isDstFloatDepthFormat)
        //             sglrReferenceContext.depthValueFloatClampCopy(dst, src);
        //         else
        //             tcu::copy(dst, src);
        //     }
        //     else
        //     {
        //         // No data supplied, clear to black.
        //         PixelBufferAccess dst = texture->getLevel(level);
        //         tcu::clear(dst, Vec4(0.0f, 0.0f, 0.0f, 1.0f));
        //     }
        // } /**/
        else
            this.setError(gl.INVALID_ENUM);
    };

    sglrReferenceContext.ReferenceContext.prototype.texSubImage2D = function(target, level, xoffset, yoffset, width, height, format, type, pixels) {
        this.texSubImage3D(target, level, xoffset, yoffset, 0, width, height, 1, format, type, pixels);
    };

    sglrReferenceContext.ReferenceContext.prototype.texSubImage3D = function(target, level, xoffset, yoffset, zoffset, width, height, depth, format, type, pixels) {
        /** @type {sglrReferenceContext.TextureUnit} */var unit = this.m_textureUnits[this.m_activeTexture];
        /** @type {ArrayBuffer} */ var data = null;
        /** @type {number} */ var offset = 0;
        /** @type {tcuTexture.PixelBufferAccess} */ var dst;
        /** @type {tcuTexture.PixelBufferAccess} */ var sub;
        /** @type {tcuTexture.ConstPixelBufferAccess} */ var src;
        /** @type {boolean} */ var isDstFloatDepthFormat;
        if (this.m_pixelUnpackBufferBinding) {
            if (this.conditionalSetError(typeof pixels !== 'number', gl.INVALID_VALUE))
                return;
            data = this.m_pixelUnpackBufferBinding.getData();
            offset = pixels;
        } else if (pixels) {
            if (pixels instanceof ArrayBuffer) {
                data = pixels;
                offset = 0;
            } else {
                data = pixels.buffer;
                offset = pixels.byteOffset;
            }
        }

        if (this.conditionalSetError(xoffset < 0 || yoffset < 0 || zoffset < 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(width < 0 || height < 0 || depth < 0 || level < 0, gl.INVALID_VALUE))
            return;

        // Map transfer format.
        /** @type {tcuTexture.TextureFormat} */ var transferFmt = gluTextureUtil.mapGLTransferFormat(format, type);
        if (this.conditionalSetError(!transferFmt, gl.INVALID_ENUM))
            return;

        if (target == gl.TEXTURE_2D) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize || height > this.m_limits.maxTexture2DSize || depth != 1, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.log2(this.m_limits.maxTexture2DSize), gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture2D} */
            var texture = /** @type {sglrReferenceContext.Texture2D} */ (unit.tex2DBinding.texture);

            if (this.conditionalSetError(!texture.hasLevel(level), gl.INVALID_OPERATION))
                return;

            //NOTE: replaces this: var dst = tcuTexture.PixelBufferAccess.newFromTextureLevel(texture.getLevel(level));
            dst = texture.getLevel(level);

            if (this.conditionalSetError(xoffset + width > dst.getWidth() ||
                                        yoffset + height > dst.getHeight() ||
                                        zoffset + depth > dst.getDepth(),
                                        gl.INVALID_VALUE))
                return;

            var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
            var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
            var skip = this.m_pixelUnpackSkipRows * rowPitch + this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
            src = new tcuTexture.ConstPixelBufferAccess({
                format: transferFmt,
                width: width,
                height: height,
                rowPitch: rowPitch,
                data: data,
                offset: offset + skip});

            sub = tcuTextureUtil.getSubregion(dst, xoffset, yoffset, zoffset, width, height, depth);
            isDstFloatDepthFormat = (dst.getFormat().order == tcuTexture.ChannelOrder.D || dst.getFormat().order == tcuTexture.ChannelOrder.DS); // depth components are limited to [0,1] range

            if (isDstFloatDepthFormat)
                sglrReferenceContext.depthValueFloatClampCopy(sub, src);
            else
                tcuTextureUtil.copy(sub, src);
        } else if (target == gl.TEXTURE_CUBE_MAP_NEGATIVE_X ||
                 target == gl.TEXTURE_CUBE_MAP_POSITIVE_X ||
                 target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Y ||
                 target == gl.TEXTURE_CUBE_MAP_POSITIVE_Y ||
                 target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Z ||
                 target == gl.TEXTURE_CUBE_MAP_POSITIVE_Z) {
            var textureCube = /** @type {sglrReferenceContext.TextureCube} */ (unit.texCubeBinding.texture);

            var face = sglrReferenceContext.mapGLCubeFace(target);

            if (this.conditionalSetError(!textureCube.hasFace(level, face), gl.INVALID_OPERATION))
                return;

            dst = textureCube.getFace(level, face);

            if (this.conditionalSetError(xoffset + width > dst.getWidth() ||
                                        yoffset + height > dst.getHeight() ||
                                        zoffset + depth > dst.getDepth(),
                                        gl.INVALID_VALUE))
                return;

            var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
            var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
            var skip = this.m_pixelUnpackSkipRows * rowPitch + this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
            src = new tcuTexture.ConstPixelBufferAccess({
                format: transferFmt,
                width: width,
                height: height,
                rowPitch: rowPitch,
                slicePitach: slicePitch,
                data: data,
                offset: offset + skip});

            sub = tcuTextureUtil.getSubregion(dst, xoffset, yoffset, zoffset, width, height, depth);
            isDstFloatDepthFormat = (dst.getFormat().order == tcuTexture.ChannelOrder.D || dst.getFormat().order == tcuTexture.ChannelOrder.DS); // depth components are limited to [0,1] range

            if (isDstFloatDepthFormat)
                sglrReferenceContext.depthValueFloatClampCopy(sub, src);
            else
                tcuTextureUtil.copy(sub, src);
        } else if (target == gl.TEXTURE_2D_ARRAY) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize ||
                        height > this.m_limits.maxTexture2DSize ||
                        depth > this.m_limits.maxTexture2DArrayLayers, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTexture2DSize)), gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture2DArray} */
            var texture2DArray = /** @type {sglrReferenceContext.Texture2DArray} */ (unit.tex2DArrayBinding.texture);

            if (this.conditionalSetError(!texture2DArray.hasLevel(level), gl.INVALID_OPERATION))
                return;

            dst = texture2DArray.getLevel(level);
            if (this.conditionalSetError(xoffset + width > dst.getWidth() ||
                                        yoffset + height > dst.getHeight() ||
                                        zoffset + depth > dst.getDepth(),
                                        gl.INVALID_VALUE))
                return;

            var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
            var imageHeight = this.m_pixelUnpackImageHeight > 0 ? this.m_pixelUnpackImageHeight : height;
            var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
            var slicePitch = imageHeight * rowPitch;
            var skip = this.m_pixelUnpackSkipImages * slicePitch + this.m_pixelUnpackSkipRows * rowPitch +
                this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
            src = new tcuTexture.ConstPixelBufferAccess({
                format: transferFmt,
                width: width,
                height: height,
                depth: depth,
                rowPitch: rowPitch,
                slicePitch: slicePitch,
                data: data,
                offset: offset + skip});

            sub = tcuTextureUtil.getSubregion(dst, xoffset, yoffset, zoffset, width, height, depth);
            isDstFloatDepthFormat = (dst.getFormat().order == tcuTexture.ChannelOrder.D || dst.getFormat().order == tcuTexture.ChannelOrder.DS); // depth components are limited to [0,1] range

            if (isDstFloatDepthFormat)
                sglrReferenceContext.depthValueFloatClampCopy(sub, src);
            else
                tcuTextureUtil.copy(sub, src);
        } else if (target == gl.TEXTURE_3D) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture3DSize ||
                        height > this.m_limits.maxTexture3DSize ||
                        depth > this.m_limits.maxTexture3DSize, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTexture3DSize)), gl.INVALID_VALUE))
                return;

            var texture3D = /** @type {sglrReferenceContext.Texture3D} */ (unit.tex3DBinding.texture);

            if (this.conditionalSetError(!texture3D.hasLevel(level), gl.INVALID_OPERATION))
                return;

            dst = texture3D.getLevel(level);
            if (this.conditionalSetError(xoffset + width > dst.getWidth() ||
                                        yoffset + height > dst.getHeight() ||
                                        zoffset + depth > dst.getDepth(),
                                        gl.INVALID_VALUE))
                return;

            var rowLen = this.m_pixelUnpackRowLength > 0 ? this.m_pixelUnpackRowLength : width;
            var imageHeight = this.m_pixelUnpackImageHeight > 0 ? this.m_pixelUnpackImageHeight : height;
            var rowPitch = deMath.deAlign32(rowLen * transferFmt.getPixelSize(), this.m_pixelUnpackAlignment);
            var slicePitch = imageHeight * rowPitch;
            var skip = this.m_pixelUnpackSkipImages * slicePitch + this.m_pixelUnpackSkipRows * rowPitch +
                this.m_pixelUnpackSkipPixels * transferFmt.getPixelSize();
            src = new tcuTexture.ConstPixelBufferAccess({
                format: transferFmt,
                width: width,
                height: height,
                depth: depth,
                rowPitch: rowPitch,
                slicePitch: slicePitch,
                data: data,
                offset: offset + skip});

            sub = tcuTextureUtil.getSubregion(dst, xoffset, yoffset, zoffset, width, height, depth);

            isDstFloatDepthFormat = (dst.getFormat().order == tcuTexture.ChannelOrder.D || dst.getFormat().order == tcuTexture.ChannelOrder.DS); // depth components are limited to [0,1] range
            if (isDstFloatDepthFormat)
                sglrReferenceContext.depthValueFloatClampCopy(sub, src);
            else
                tcuTextureUtil.copy(sub, src);
        } else
            this.setError(gl.INVALID_ENUM);
    };

    /**
    * @param {number} target
    * @param {number} level
    * @param {number} internalFormat
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    * @param {number} border
    */
    sglrReferenceContext.ReferenceContext.prototype.copyTexImage2D = function(target, level, internalFormat, x, y, width, height, border) {
        /** @type {sglrReferenceContext.TextureUnit} */var unit = this.m_textureUnits[this.m_activeTexture];
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var src = this.getReadColorbuffer();

        if (this.conditionalSetError(border != 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(width < 0 || height < 0 || level < 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(src.isEmpty(), gl.INVALID_OPERATION))
            return;

        // Map storage format.
        /** @type {tcuTexture.TextureFormat} */ var storageFmt = sglrReferenceContext.mapInternalFormat(internalFormat);
        if (this.conditionalSetError(!storageFmt, gl.INVALID_ENUM))
            return;

        if (target == gl.TEXTURE_2D) {
            // Validate size and level.
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize || height > this.m_limits.maxTexture2DSize, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTexture2DSize)), gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture2D} */
            var texture = /** @type {sglrReferenceContext.Texture2D} */ (unit.tex2DBinding.texture);

            if (texture.isImmutable()) {
                if (this.conditionalSetError(!texture.hasLevel(level), gl.INVALID_OPERATION))
                    return;

                /** @type {tcuTexture.PixelBufferAccess} */ var dst = texture.getLevel(level);
                if (this.conditionalSetError(storageFmt != dst.getFormat() || width != dst.getWidth() || height != dst.getHeight(), gl.INVALID_OPERATION))
                    return;
            } else {
                texture.allocLevel(level, storageFmt, width, height);
            }

            // Copy from current framebuffer.
            /** @type {tcuTexture.PixelBufferAccess} */ var dst = texture.getLevel(level);
            for (var yo = 0; yo < height; yo++) {
                for (var xo = 0; xo < width; xo++) {
                    if (!deMath.deInBounds32(x+xo, 0, src.raw().getHeight()) || !deMath.deInBounds32(y+yo, 0, src.raw().getDepth()))
                        continue; // Undefined pixel.

                    dst.setPixel(src.resolveMultisamplePixel(x+xo, y+yo), xo, yo);
                }
            }
        } else if (target == gl.TEXTURE_CUBE_MAP_NEGATIVE_X ||
                   target == gl.TEXTURE_CUBE_MAP_POSITIVE_X ||
                   target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Y ||
                   target == gl.TEXTURE_CUBE_MAP_POSITIVE_Y ||
                   target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Z ||
                   target == gl.TEXTURE_CUBE_MAP_POSITIVE_Z) {
            // Validate size and level.
            if (this.conditionalSetError(width != height || width > this.m_limits.maxTextureCubeSize, gl.INVALID_VALUE))
                return;
            if (this.conditionalSetError(level > Math.floor(Math.log2(this.m_limits.maxTextureCubeSize)), gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.TextureCube} */
            var texture = /** @type {sglrReferenceContext.TextureCube} */ (unit.texCubeBinding.texture);
            var face = sglrReferenceContext.mapGLCubeFace(target);

            if (texture.isImmutable()) {
                if (this.conditionalSetError(!texture.hasFace(level, face), gl.INVALID_OPERATION))
                    return;

                /** @type {tcuTexture.PixelBufferAccess} */ var dst = texture.getFace(level, face);
                if (this.conditionalSetError(storageFmt != dst.getFormat() || width != dst.getWidth() || height != dst.getHeight(), gl.INVALID_OPERATION))
                    return;
            } else {
                texture.allocLevel(level, face, storageFmt, width, height);
            }

            // Copy from current framebuffer.
            /** @type {tcuTexture.PixelBufferAccess} */ var dst = texture.getFace(level, face);
            for (var yo = 0; yo < height; yo++) {
                for (var xo = 0; xo < width; xo++) {
                    if (!deMath.deInBounds32(x+xo, 0, src.raw().getHeight()) || !deMath.deInBounds32(y+yo, 0, src.raw().getDepth()))
                        continue; // Undefined pixel.

                    dst.setPixel(src.resolveMultisamplePixel(x+xo, y+yo), xo, yo);
                }
            }
        } else {
            this.setError(gl.INVALID_ENUM);
        }
    }

    /**
    * @param {number} target
    * @param {number} level
    * @param {number} xoffset
    * @param {number} yoffset
    * @param {number} x
    * @param {number} y
    * @param {number} width
    * @param {number} height
    */
    sglrReferenceContext.ReferenceContext.prototype.copyTexSubImage2D = function(target, level, xoffset, yoffset, x, y, width, height) {
        /** @type {sglrReferenceContext.TextureUnit} */var unit = this.m_textureUnits[this.m_activeTexture];
        /** @type {rrMultisamplePixelBufferAccess.MultisamplePixelBufferAccess} */ var src = this.getReadColorbuffer();

        if (this.conditionalSetError(xoffset < 0 || yoffset < 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(width < 0 || height < 0 || level < 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(src.isEmpty(), gl.INVALID_OPERATION))
            return;

        if (target == gl.TEXTURE_2D) {
            /** @type {sglrReferenceContext.Texture2D} */
            var texture = /** @type {sglrReferenceContext.Texture2D} */ (unit.tex2DBinding.texture);

            if (this.conditionalSetError(!texture.hasLevel(level), gl.INVALID_VALUE))
                return;

            /** @type {tcuTexture.PixelBufferAccess} */ var dst = texture.getLevel(level);

            if (this.conditionalSetError(xoffset + width > dst.getWidth() || yoffset + height > dst.getHeight(), gl.INVALID_VALUE))
                return;

            for (var yo = 0; yo < height; yo++) {
                for (var xo = 0; xo < width; xo++) {
                    if (!deMath.deInBounds32(x+xo, 0, src.raw().getHeight()) || !deMath.deInBounds32(y+yo, 0, src.raw().getDepth()))
                        continue;

                    dst.setPixel(src.resolveMultisamplePixel(x+xo, y+yo), xo+xoffset, yo+yoffset);
                }
            }
        } else if (target == gl.TEXTURE_CUBE_MAP_NEGATIVE_X ||
                   target == gl.TEXTURE_CUBE_MAP_POSITIVE_X ||
                   target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Y ||
                   target == gl.TEXTURE_CUBE_MAP_POSITIVE_Y ||
                   target == gl.TEXTURE_CUBE_MAP_NEGATIVE_Z ||
                   target == gl.TEXTURE_CUBE_MAP_POSITIVE_Z) {
            /** @type {sglrReferenceContext.TextureCube} */
            var texture = /** @type {sglrReferenceContext.TextureCube} */ (unit.texCubeBinding.texture);
            var face = sglrReferenceContext.mapGLCubeFace(target);

            if (this.conditionalSetError(!texture.hasFace(level, face), gl.INVALID_VALUE))
                return;

            /** @type {tcuTexture.PixelBufferAccess} */ var dst = texture.getFace(level, face);

            if (this.conditionalSetError(xoffset + width > dst.getWidth() || yoffset + height > dst.getHeight(), gl.INVALID_VALUE))
                return;

            for (var yo = 0; yo < height; yo++) {
                for (var xo = 0; xo < width; xo++) {
                    if (!deMath.deInBounds32(x+xo, 0, src.raw().getHeight()) || !deMath.deInBounds32(y+yo, 0, src.raw().getDepth()))
                        continue;

                    dst.setPixel(src.resolveMultisamplePixel(x+xo, y+yo), xo+xoffset, yo+yoffset);
                }
            }
        } else {
            this.setError(gl.INVALID_ENUM);
        }
    }

    sglrReferenceContext.ReferenceContext.prototype.texStorage3D = function(target, levels, internalFormat, width, height, depth) {
        /** @type {sglrReferenceContext.TextureUnit} */var unit = this.m_textureUnits[this.m_activeTexture];

        if (this.conditionalSetError(width <= 0 || height <= 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(levels < 1 || levels > Math.floor(Math.log2(Math.max(width, height))) + 1, gl.INVALID_VALUE))
            return;

        // Map storage format.
        /** @type {tcuTexture.TextureFormat} */ var storageFmt = sglrReferenceContext.mapInternalFormat(internalFormat);
        if (this.conditionalSetError(!storageFmt, gl.INVALID_ENUM))
            return;

        if (target == gl.TEXTURE_2D_ARRAY) {
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize ||
                                        height > this.m_limits.maxTexture2DSize ||
                                        depth >= this.m_limits.maxTexture2DArrayLayers, gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture2DArray} */
            var textureArray = /** @type {sglrReferenceContext.Texture2DArray} */ (unit.tex2DArrayBinding.texture);
            if (this.conditionalSetError(textureArray.isImmutable(), gl.INVALID_OPERATION))
                return;

            textureArray.clearLevels();
            textureArray.setImmutable();

            for (var level = 0; level < levels; level++) {
                var levelW = Math.max(1, width >> level);
                var levelH = Math.max(1, height >> level);

                textureArray.allocLevel(level, storageFmt, levelW, levelH, depth);
            }
        } else if (target == gl.TEXTURE_3D) {
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize ||
                                        height > this.m_limits.maxTexture2DSize ||
                                        depth >= this.m_limits.maxTexture3DSize, gl.INVALID_VALUE))
                return;

            /** @type {sglrReferenceContext.Texture3D} */
            var texture3D = /** @type {sglrReferenceContext.Texture3D} */ (unit.tex3DBinding.texture);
            if (this.conditionalSetError(texture3D.isImmutable(), gl.INVALID_OPERATION))
                return;

            texture3D.clearLevels();
            texture3D.setImmutable();

            for (var level = 0; level < levels; level++) {
                var levelW = Math.max(1, width >> level);
                var levelH = Math.max(1, height >> level);
                var levelD = Math.max(1, depth >> level);

                texture3D.allocLevel(level, storageFmt, levelW, levelH, levelD);
            }
        } else
            this.setError(gl.INVALID_ENUM);
    };

    sglrReferenceContext.ReferenceContext.prototype.texStorage2D = function(target, levels, internalFormat, width, height) {
        /** @type {sglrReferenceContext.TextureUnit} */var unit = this.m_textureUnits[this.m_activeTexture];

        if (this.conditionalSetError(width <= 0 || height <= 0, gl.INVALID_VALUE))
            return;
        if (this.conditionalSetError(levels < 1 || levels > Math.floor(Math.log2(Math.max(width, height))) + 1, gl.INVALID_VALUE))
            return;

        // Map storage format.
        /** @type {tcuTexture.TextureFormat} */ var storageFmt = sglrReferenceContext.mapInternalFormat(internalFormat);
        if (this.conditionalSetError(!storageFmt, gl.INVALID_ENUM))
            return;

        if (target == gl.TEXTURE_2D) {
            if (this.conditionalSetError(width > this.m_limits.maxTexture2DSize || height > this.m_limits.maxTexture2DSize, gl.INVALID_VALUE))
                return;

                /** @type {sglrReferenceContext.Texture2D} */
            var texture = /** @type {sglrReferenceContext.Texture2D} */ (unit.tex2DBinding.texture);
            if (this.conditionalSetError(texture.isImmutable(), gl.INVALID_OPERATION))
                return;

            texture.clearLevels();
            texture.setImmutable();

            for (var level = 0; level < levels; level++) {
                var levelW = Math.max(1, width >> level);
                var levelH = Math.max(1, height >> level);

                texture.allocLevel(level, storageFmt, levelW, levelH);
            }
        } else if (target == gl.TEXTURE_CUBE_MAP) {
            if (this.conditionalSetError(width != height || width > this.m_limits.maxTextureCubeSize, gl.INVALID_VALUE))
                return;
            var textureCube = /** @type {sglrReferenceContext.TextureCube} */ (unit.texCubeBinding.texture);
            if (this.conditionalSetError(textureCube.isImmutable(), gl.INVALID_OPERATION))
                return;

            textureCube.clearLevels();
            textureCube.setImmutable();

            for (var level = 0; level < levels; level++) {
                var levelW = Math.max(1, width >> level);
                var levelH = Math.max(1, height >> level);

                for (var face in tcuTexture.CubeFace)
                    textureCube.allocLevel(level, tcuTexture.CubeFace[face], storageFmt, levelW, levelH);
            }
        } else
            this.setError(gl.INVALID_ENUM);
    };

    /**
     * @param {number} value
     * @return {?tcuTexture.WrapMode}
     */
    sglrReferenceContext.mapGLWrapMode = function(value) {
        switch (value) {
            case gl.CLAMP_TO_EDGE: return tcuTexture.WrapMode.CLAMP_TO_EDGE;
            case gl.REPEAT: return tcuTexture.WrapMode.REPEAT_GL;
            case gl.MIRRORED_REPEAT: return tcuTexture.WrapMode.MIRRORED_REPEAT_GL;
        }
        return null;
    };

     /**
     * @param {number} value
     * @return {?tcuTexture.FilterMode}
     */
    sglrReferenceContext.mapGLFilterMode = function(value) {
        switch (value) {
            case gl.NEAREST: return tcuTexture.FilterMode.NEAREST;
            case gl.LINEAR: return tcuTexture.FilterMode.LINEAR;
            case gl.NEAREST_MIPMAP_NEAREST: return tcuTexture.FilterMode.NEAREST_MIPMAP_NEAREST;
            case gl.NEAREST_MIPMAP_LINEAR: return tcuTexture.FilterMode.NEAREST_MIPMAP_LINEAR;
            case gl.LINEAR_MIPMAP_NEAREST: return tcuTexture.FilterMode.LINEAR_MIPMAP_NEAREST;
            case gl.LINEAR_MIPMAP_LINEAR: return tcuTexture.FilterMode.LINEAR_MIPMAP_LINEAR;
        }
        return null;
    };

    /**
    * @param {number} target
    * @param {number} pname
    * @param {number} value
    */
    sglrReferenceContext.ReferenceContext.prototype.texParameteri = function(target, pname, value) {
        /** @type {sglrReferenceContext.TextureUnit} */ var unit = this.m_textureUnits[this.m_activeTexture];
        /** @type {sglrReferenceContext.TextureContainer} */ var container = null;

        switch (target) {
            case gl.TEXTURE_2D: container = unit.tex2DBinding; break;
            case gl.TEXTURE_CUBE_MAP: container = unit.texCubeBinding; break;
            case gl.TEXTURE_2D_ARRAY: container = unit.tex2DArrayBinding; break;
            case gl.TEXTURE_3D: container = unit.tex3DBinding; break;

            default: this.setError(gl.INVALID_ENUM);
        }

        if (!container)
            return;

        /** @type {sglrReferenceContext.Texture} */
        var texture = container.texture;

        switch (pname) {
            case gl.TEXTURE_WRAP_S: {
                /** @type {?tcuTexture.WrapMode} */ var wrapS = sglrReferenceContext.mapGLWrapMode(value);
                if (this.conditionalSetError(null == wrapS, gl.INVALID_VALUE))
                    return;
                texture.getSampler().wrapS = /** @type {tcuTexture.WrapMode} */ (wrapS);
                break;
            }

            case gl.TEXTURE_WRAP_T: {
                /** @type {?tcuTexture.WrapMode} */ var wrapT = sglrReferenceContext.mapGLWrapMode(value);
                if (this.conditionalSetError(null == wrapT, gl.INVALID_VALUE))
                    return;
                texture.getSampler().wrapT = /** @type {tcuTexture.WrapMode} */ (wrapT);
                break;
            }

            case gl.TEXTURE_WRAP_R: {
                /** @type {?tcuTexture.WrapMode} */ var wrapR = sglrReferenceContext.mapGLWrapMode(value);
                if (this.conditionalSetError(null == wrapR, gl.INVALID_VALUE))
                    return;
                texture.getSampler().wrapR = /** @type {tcuTexture.WrapMode} */ (wrapR);
                break;
            }

            case gl.TEXTURE_MIN_FILTER: {
                /** @type {?tcuTexture.FilterMode} */ var minMode = sglrReferenceContext.mapGLFilterMode(value);
                if (this.conditionalSetError(null == minMode, gl.INVALID_VALUE))
                    return;
                texture.getSampler().minFilter = /** @type {tcuTexture.FilterMode} */ (minMode);
                break;
            }

            case gl.TEXTURE_MAG_FILTER: {
                /** @type {?tcuTexture.FilterMode} */ var magMode = sglrReferenceContext.mapGLFilterMode(value);
                if (this.conditionalSetError(null == magMode, gl.INVALID_VALUE))
                    return;
                texture.getSampler().magFilter = /** @type {tcuTexture.FilterMode} */ (magMode);
                break;
            }

            case gl.TEXTURE_MAX_LEVEL: {
                if (this.conditionalSetError(value < 0, gl.INVALID_VALUE))
                    return;
                texture.setMaxLevel(value);
                break;
            }

            default:
                this.setError(gl.INVALID_ENUM);
                return;
        }
    };

    sglrReferenceContext.ReferenceContext.prototype.invalidateFramebuffer = function(target, attachments) {};
    sglrReferenceContext.ReferenceContext.prototype.invalidateSubFramebuffer = function(target, attachments, x, y, width, height) {};

});

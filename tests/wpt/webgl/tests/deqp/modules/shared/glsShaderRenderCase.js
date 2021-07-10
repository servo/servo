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
goog.provide('modules.shared.glsShaderRenderCase');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {
    var glsShaderRenderCase = modules.shared.glsShaderRenderCase;

    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var deRandom = framework.delibs.debase.deRandom;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluTexture = framework.opengl.gluTexture;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuTexture = framework.common.tcuTexture;
    var tcuMatrix = framework.common.tcuMatrix;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var gluShaderProgram = framework.opengl.gluShaderProgram;

    /** @typedef {function(glsShaderRenderCase.ShaderEvalContext)} */ glsShaderRenderCase.ShaderEvalFunc;

    /** @const {number} */ glsShaderRenderCase.GRID_SIZE = 64;
    /** @const {number} */ glsShaderRenderCase.MAX_RENDER_WIDTH = 128;
    /** @const {number} */ glsShaderRenderCase.MAX_RENDER_HEIGHT = 112;
    /** @const {Array<number>} */ glsShaderRenderCase.DEFAULT_CLEAR_COLOR = [0.125, 0.25, 0.5, 1.0];
    /** @const {number} */ glsShaderRenderCase.MAX_USER_ATTRIBS = 4;
    /** @const {number} */ glsShaderRenderCase.MAX_TEXTURES = 4;

    /**
     * @param  {Array<number>} a
     * @return {tcuRGBA.RGBA}
     */
    glsShaderRenderCase.toRGBA = function(a) {
        return tcuRGBA.newRGBAComponents(
            deMath.clamp(Math.round(a[0] * 255.0), 0, 255),
            deMath.clamp(Math.round(a[1] * 255.0), 0, 255),
            deMath.clamp(Math.round(a[2] * 255.0), 0, 255),
            deMath.clamp(Math.round(a[3] * 255.0), 0, 255));
    };

    /**
     * Helper function
     * @param  {?(gluTexture.Texture2D|gluTexture.TextureCube|gluTexture.Texture2DArray|gluTexture.Texture3D)} tex
     * @return {gluTexture.Type}
     */
    glsShaderRenderCase.getTextureType = function(tex) {
        if (tex === null || tex.getType() <= 0)
            return gluTexture.Type.TYPE_NONE;
        else
            return tex.getType();
    };

    /**
     * @constructor
     * @param {number=} indent
     */
    glsShaderRenderCase.LineStream = function(indent) {
        indent = indent === undefined ? 0 : indent;
        /** @type {number} */ this.m_indent = indent;
        /** @type {string} */ this.m_stream;
        /** @type {string} */ this.m_string;
    };

    /**
     * @return {string}
     */
    glsShaderRenderCase.LineStream.prototype.str = function() {
         this.m_string = this.m_stream;
         return this.m_string;
    };

    /**
     * @constructor
     * @param  {(gluTexture.Texture2D|gluTexture.TextureCube|gluTexture.Texture2DArray|gluTexture.Texture3D)=} tex
     * @param  {tcuTexture.Sampler=} sampler
     */
    glsShaderRenderCase.TextureBinding = function(tex, sampler) {
        tex = tex === undefined ? null : tex;
        sampler = sampler === undefined ? null : sampler;
        /** @type {gluTexture.Type} */ this.m_type = glsShaderRenderCase.getTextureType(tex);
        /** @type {tcuTexture.Sampler} */ this.m_sampler = sampler;
        /** @type {(gluTexture.Texture2D|gluTexture.TextureCube|gluTexture.Texture2DArray|gluTexture.Texture3D)} */
        this.m_binding = tex;
    };

    /**
     * @param {tcuTexture.Sampler} sampler
     */
    glsShaderRenderCase.TextureBinding.prototype.setSampler = function(sampler) {
        this.m_sampler = sampler;
    };

    /**
     * @param {(gluTexture.Texture2D|gluTexture.TextureCube|gluTexture.Texture2DArray|gluTexture.Texture3D)} tex
     */
    glsShaderRenderCase.TextureBinding.prototype.setTexture = function(tex) {
        this.m_type = glsShaderRenderCase.getTextureType(tex);
        this.m_binding = tex;
    };

    /** @return {gluTexture.Type} */
    glsShaderRenderCase.TextureBinding.prototype.getType = function() {
        return this.m_type;
    };

    /** @return {tcuTexture.Sampler} */
    glsShaderRenderCase.TextureBinding.prototype.getSampler = function() {
        return this.m_sampler;
    };

    /** @return {(gluTexture.Texture2D|gluTexture.TextureCube|gluTexture.Texture2DArray|gluTexture.Texture3D)} */
    glsShaderRenderCase.TextureBinding.prototype.getBinding = function() {
        return this.m_binding;
    };

    /**
     * @constructor
     * @param  {number} gridSize
     * @param  {number} width
     * @param  {number} height
     * @param  {Array<number>} constCoords
     * @param  {Array<tcuMatrix.Matrix>} userAttribTransforms
     * @param  {Array<glsShaderRenderCase.TextureBinding>} textures
     */
    glsShaderRenderCase.QuadGrid = function(gridSize, width, height, constCoords, userAttribTransforms, textures) {
        /** @type {number} */ this.m_gridSize = gridSize;
        /** @type {number} */ this.m_numVertices = (gridSize + 1) * (gridSize + 1);
        /** @type {number} */ this.m_numTriangles = (gridSize * gridSize *2);
        /** @type {Array<number>} */ this.m_constCoords = constCoords;
        /** @type {Array<tcuMatrix.Matrix>} */ this.m_userAttribTransforms = userAttribTransforms;
        /** @type {Array<glsShaderRenderCase.TextureBinding>} */ this.m_textures = textures;
        /** @type {Array<Array<number>>} */ this.m_screenPos = [];
        /** @type {Array<Array<number>>} */ this.m_positions = [];
        /** @type {Array<Array<number>>} */ this.m_coords = [];            //!< Near-unit coordinates, roughly [-2.0 .. 2.0].
        /** @type {Array<Array<number>>} */ this.m_unitCoords = [];        //!< Positive-only coordinates [0.0 .. 1.5].
        /** @type {Array<number>} */ this.m_attribOne = [];
        /** @type {Array<Array<number>>} */ this.m_userAttribs = [];
        for (var attribNdx = 0; attribNdx < this.getNumUserAttribs(); attribNdx++)
            this.m_userAttribs[attribNdx] = [];
        /** @type {Array<number>} */ this.m_indices = [];

        /** @type Array<number>} */ var viewportScale = [width, height, 0, 0];
        for (var y = 0; y < gridSize + 1; y++)
        for (var x = 0; x < gridSize + 1; x++) {
            /** @type {number} */ var sx = x / gridSize;
            /** @type {number} */ var sy = y / gridSize;
            /** @type {number} */ var fx = 2.0 * sx - 1.0;
            /** @type {number} */ var fy = 2.0 * sy - 1.0;
            /** @type {number} */ var vtxNdx = ((y * (gridSize + 1)) + x);

            this.m_positions[vtxNdx] = [fx, fy, 0.0, 1.0];
            this.m_attribOne[vtxNdx] = 1.0;
            this.m_screenPos[vtxNdx] = deMath.multiply([sx, sy, 0.0, 1.0], viewportScale);
            this.m_coords[vtxNdx] = this.getCoords(sx, sy);
            this.m_unitCoords[vtxNdx] = this.getUnitCoords(sx, sy);

            for (var attribNdx = 0; attribNdx < this.getNumUserAttribs(); attribNdx++)
                this.m_userAttribs[attribNdx][vtxNdx] = this.getUserAttrib(attribNdx, sx, sy);
        }

        // Compute indices.
        for (var y = 0; y < gridSize; y++)
        for (var x = 0; x < gridSize; x++) {
            /** @type {number} */ var stride = gridSize + 1;
            /** @type {number} */ var v00 = (y * stride) + x;
            /** @type {number} */ var v01 = (y * stride) + x + 1;
            /** @type {number} */ var v10 = ((y + 1) * stride) + x;
            /** @type {number} */ var v11 = ((y + 1) * stride) + x + 1;

            /** @type {number} */ var baseNdx = ((y * gridSize) + x) * 6;
            this.m_indices[baseNdx + 0] = v10;
            this.m_indices[baseNdx + 1] = v00;
            this.m_indices[baseNdx + 2] = v01;

            this.m_indices[baseNdx + 3] = v10;
            this.m_indices[baseNdx + 4] = v01;
            this.m_indices[baseNdx + 5] = v11;
        }
    };

    /** @return {number} */
    glsShaderRenderCase.QuadGrid.prototype.getGridSize = function() {
        return this.m_gridSize;
    };

    /** @return {number} */
    glsShaderRenderCase.QuadGrid.prototype.getNumVertices = function() {
        return this.m_numVertices;
    };

    /** @return {number} */
    glsShaderRenderCase.QuadGrid.prototype.getNumTriangles = function() {
        return this.m_numTriangles;
    };

    /** @return {Array<number>} */
    glsShaderRenderCase.QuadGrid.prototype.getConstCoords = function() {
        return this.m_constCoords;
    };

    /** @return {Array<tcuMatrix.Matrix>} */
    glsShaderRenderCase.QuadGrid.prototype.getUserAttribTransforms = function() {
        return this.m_userAttribTransforms;
    };

    /** @return {Array<glsShaderRenderCase.TextureBinding>} */
    glsShaderRenderCase.QuadGrid.prototype.getTextures = function() {
        return this.m_textures;
    };

    /** @return {Array<Array<number>>} */
    glsShaderRenderCase.QuadGrid.prototype.getPositions = function() {
        return this.m_positions;
    };

    /** @return {Array<number>} */
    glsShaderRenderCase.QuadGrid.prototype.getAttribOne = function() {
        return this.m_attribOne;
    };

    /** @return {Array<Array<number>>} */
    glsShaderRenderCase.QuadGrid.prototype.getCoordsArray = function() {
        return this.m_coords;
    };

    /** @return {Array<Array<number>>} */
    glsShaderRenderCase.QuadGrid.prototype.getUnitCoordsArray = function() {
        return this.m_unitCoords;
    };

    /**
     * @param {number} attribNdx
     * @return {Array<number>}
     */
    glsShaderRenderCase.QuadGrid.prototype.getUserAttribByIndex = function(attribNdx) {
        return this.m_userAttribs[attribNdx];
    };

    /** @return {Array<number>} */
    glsShaderRenderCase.QuadGrid.prototype.getIndices = function() {
        return this.m_indices;
    };

    /**
     * @param {number} sx
     * @param {number} sy
     * @return {Array<number>}
     */
    glsShaderRenderCase.QuadGrid.prototype.getCoords = function(sx, sy) {
        /** @type {number} */ var fx = 2.0 * sx - 1.0;
        /** @type {number} */ var fy = 2.0 * sy - 1.0;
        return [fx, fy, -fx + 0.33 * fy, -0.275 * fx - fy];
    };

    /**
     * @param {number} sx
     * @param {number} sy
     * @return {Array<number>}
     */
    glsShaderRenderCase.QuadGrid.prototype.getUnitCoords = function(sx, sy) {
        return [sx, sy, 0.33 * sx + 0.5 * sy, 0.5 * sx + 0.25 * sy];
    };

    /**
     * @return {number}
     */
    glsShaderRenderCase.QuadGrid.prototype.getNumUserAttribs = function() {
        return this.m_userAttribTransforms.length;
    };

    /**
     * @param {number} attribNdx
     * @param {number} sx
     * @param {number} sy
     * @return {Array<number>}
     */
    glsShaderRenderCase.QuadGrid.prototype.getUserAttrib = function(attribNdx, sx, sy) {
        // homogeneous normalized screen-space coordinates
        return tcuMatrix.multiplyMatVec(this.m_userAttribTransforms[attribNdx], [sx, sy, 0.0, 1.0]);
    };

    /**
     * @constructor
     * @struct
     */
    glsShaderRenderCase.ShaderSampler = function() {
        /** @type {tcuTexture.Sampler} */ this.sampler;
        /** @type {tcuTexture.Texture2D} */ this.tex2D = null;
        /** @type {tcuTexture.TextureCube} */ this.texCube = null;
        /** @type {tcuTexture.Texture2DArray} */ this.tex2DArray = null;
        /** @type {tcuTexture.Texture3D} */ this.tex3D = null;
    };

    /**
     * @constructor
     * @param  {glsShaderRenderCase.QuadGrid} quadGrid_
     */
    glsShaderRenderCase.ShaderEvalContext = function(quadGrid_) {
        /** @type {Array<number>} */ this.coords = [0, 0, 0, 0]
        /** @type {Array<number>} */ this.unitCoords = [0, 0, 0, 0]
        /** @type {Array<number>} */ this.constCoords = quadGrid_.getConstCoords();
        /** @type {Array<Array<number>>} */ this.in_ = [];
        /** @type {Array<glsShaderRenderCase.ShaderSampler>} */ this.textures = [];
        /** @type {Array<number>} */ this.color = [0, 0, 0, 0.0];
        /** @type {boolean} */ this.isDiscarded = false;
        /** @type {glsShaderRenderCase.QuadGrid} */ this.quadGrid = quadGrid_;

        /** @type {Array<glsShaderRenderCase.TextureBinding>} */ var bindings = this.quadGrid.getTextures();
        assertMsgOptions(bindings.length <= glsShaderRenderCase.MAX_TEXTURES, 'Too many bindings.', false, true);

        // Fill in texture array.
        for (var ndx = 0; ndx < bindings.length; ndx++) {
            /** @type {glsShaderRenderCase.TextureBinding} */ var binding = bindings[ndx];

            this.textures[ndx] = new glsShaderRenderCase.ShaderSampler();

            if (binding.getType() == gluTexture.Type.TYPE_NONE)
                continue;

            this.textures[ndx].sampler = binding.getSampler();

            switch (binding.getType()) {
                case gluTexture.Type.TYPE_2D:
                    this.textures[ndx].tex2D = binding.getBinding().getRefTexture();
                    break;
                case gluTexture.Type.TYPE_CUBE_MAP:
                    this.textures[ndx].texCube = binding.getBinding().getRefTexture();
                    break;
                case gluTexture.Type.TYPE_2D_ARRAY:
                    this.textures[ndx].tex2DArray = binding.getBinding().getRefTexture();
                    break;
                case gluTexture.Type.TYPE_3D:
                    this.textures[ndx].tex3D = binding.getBinding().getRefTexture();
                    break;
                default:
                    throw new Error("Binding type not supported");
            }
        }
    };

    /**
     * @param {number} sx
     * @param {number} sy
     */
    glsShaderRenderCase.ShaderEvalContext.prototype.reset = function(sx, sy) {
        // Clear old values
        this.color = [0.0, 0.0, 0.0, 1.0];
        this.isDiscarded = false;

        // Compute coords
        this.coords = this.quadGrid.getCoords(sx, sy);
        this.unitCoords = this.quadGrid.getUnitCoords(sx, sy);

        // Compute user attributes.
        /** @type {number} */ var numAttribs = this.quadGrid.getNumUserAttribs();
        assertMsgOptions(numAttribs <= glsShaderRenderCase.MAX_USER_ATTRIBS, 'numAttribs out of range', false, true);
        for (var attribNdx = 0; attribNdx < numAttribs; attribNdx++)
            this.in_[attribNdx] = this.quadGrid.getUserAttrib(attribNdx, sx, sy);
    };

    glsShaderRenderCase.ShaderEvalContext.prototype.discard = function() {
        this.isDiscarded = true;
    };

    /**
     * @param {number} unitNdx
     * @param {Array<number>} coords
     */
    glsShaderRenderCase.ShaderEvalContext.prototype.texture2D = function(unitNdx, coords) {
        if (this.textures.length > 0 && this.textures[unitNdx].tex2D)
            return this.textures[unitNdx].tex2D.getView().sample(this.textures[unitNdx].sampler, coords, 0.0);
        else
            return [0.0, 0.0, 0.0, 1.0];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    glsShaderRenderCase.evalCoordsPassthroughX = function(c) {
        c.color[0] = c.coords[0];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    glsShaderRenderCase.evalCoordsPassthroughXY = function(c) {
        var swizzle01 = deMath.swizzle(c.coords, [0, 1]);
        c.color[0] = swizzle01[0];
        c.color[1] = swizzle01[1];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    glsShaderRenderCase.evalCoordsPassthroughXYZ = function(c) {
        var swizzle012 = deMath.swizzle(c.coords, [0, 1, 2]);
        c.color[0] = swizzle012[0];
        c.color[1] = swizzle012[1];
        c.color[2] = swizzle012[2];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    glsShaderRenderCase.evalCoordsPassthrough = function(c) {
        c.color = c.coords;
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    glsShaderRenderCase.evalCoordsSwizzleWZYX = function(c) {
        c.color = deMath.swizzle(c.coords, [3, 2, 1, 0]);
    };

    /**
     * @constructor
     * @param  {?glsShaderRenderCase.ShaderEvalFunc=} evalFunc
     */
    glsShaderRenderCase.ShaderEvaluator = function(evalFunc) {
        /** @type {?glsShaderRenderCase.ShaderEvalFunc} */ this.m_evalFunc = evalFunc || null;
    };

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} ctx
     */
    glsShaderRenderCase.ShaderEvaluator.prototype.evaluate = function(ctx) {
        assertMsgOptions(this.m_evalFunc !== null, 'No evaluation function specified.', false, true);
        this.m_evalFunc(ctx);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param  {string} name
     * @param  {string} description
     * @param  {boolean} isVertexCase
     * @param  {glsShaderRenderCase.ShaderEvalFunc=} evalFunc
     */
    glsShaderRenderCase.ShaderRenderCase = function(name, description, isVertexCase, evalFunc) {
        tcuTestCase.DeqpTest.call(this, name, description);
        // evalFunc = evalFunc || null;
        /** @type {boolean} */ this.m_isVertexCase = isVertexCase;
        /** @type {?glsShaderRenderCase.ShaderEvalFunc} */ this.m_defaultEvaluator = evalFunc || null;
        /** @type {glsShaderRenderCase.ShaderEvaluator} */ this.m_evaluator = new glsShaderRenderCase.ShaderEvaluator(this.m_defaultEvaluator);
        /** @type {string} */ this.m_vertShaderSource = '';
        /** @type {string} */ this.m_fragShaderSource = '';
        /** @type {Array<number>} */ this.m_clearColor = glsShaderRenderCase.DEFAULT_CLEAR_COLOR;
        /** @type {Array<tcuMatrix.Matrix>} */ this.m_userAttribTransforms = [];
        /** @type {Array<glsShaderRenderCase.TextureBinding>} */ this.m_textures = [];
        /** @type {?gluShaderProgram.ShaderProgram} */ this.m_program = null;
    };

    /**
     * @param  {string} name
     * @param  {string} description
     * @param  {boolean} isVertexCase
     * @param  {glsShaderRenderCase.ShaderEvaluator} evaluator
     * @return {glsShaderRenderCase.ShaderRenderCase}
     */
    glsShaderRenderCase.ShaderRenderCase.newWithEvaluator = function(name, description, isVertexCase, evaluator) {
        var renderCase = new glsShaderRenderCase.ShaderRenderCase(name, description, isVertexCase);
        renderCase.m_evaluator = evaluator;
        return renderCase;
    };

    glsShaderRenderCase.ShaderRenderCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsShaderRenderCase.ShaderRenderCase.prototype.constructor = glsShaderRenderCase.ShaderRenderCase;

    glsShaderRenderCase.ShaderRenderCase.prototype.deinit = function() {
        this.m_program = null;
    };

    glsShaderRenderCase.ShaderRenderCase.prototype.init = function() {
        this.postinit();
    };

    glsShaderRenderCase.ShaderRenderCase.prototype.postinit = function() {
        if (this.m_vertShaderSource.length === 0 || this.m_fragShaderSource.length === 0) {
            assertMsgOptions(this.m_vertShaderSource.length === 0 && this.m_fragShaderSource.length === 0, 'No shader source.', false, true);
            this.setupShaderData();
        }

        assertMsgOptions(!this.m_program, 'Program defined.', false, true);
        this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(this.m_vertShaderSource, this.m_fragShaderSource));

        try {
            bufferedLogToConsole(this.m_program.program.info.infoLog); // Always log shader program.

            if (!this.m_program.isOk())
                throw new Error("Shader compile error.");
        }
        catch (exception) {
            // Clean up.
            this.deinit();
            throw exception;
        }
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    glsShaderRenderCase.ShaderRenderCase.prototype.postiterate = function() {
        assertMsgOptions(this.m_program !== null, 'Program not specified.', false, true);
        /** @type {?WebGLProgram} */ var programID = this.m_program.getProgram();
        gl.useProgram(programID);

        // Create quad grid.
        /** @type {Array<number>} */ var viewportSize = this.getViewportSize();
        /** @type {number} */ var width = viewportSize[0];
        /** @type {number} */ var height = viewportSize[1];

        // \todo [petri] Better handling of constCoords (render in multiple chunks, vary coords).
        /** @type {glsShaderRenderCase.QuadGrid} */
        var quadGrid = new glsShaderRenderCase.QuadGrid(
            this.m_isVertexCase ? glsShaderRenderCase.GRID_SIZE : 4, width, height,
            [0.125, 0.25, 0.5, 1.0], this.m_userAttribTransforms, this.m_textures);

        // Render result.
        /** @type {tcuSurface.Surface} */ var resImage = new tcuSurface.Surface(width, height);
        this.render(resImage, programID, quadGrid);

        // Compute reference.
        /** @type {tcuSurface.Surface} */ var refImage = new tcuSurface.Surface(width, height);
        if (this.m_isVertexCase)
            this.computeVertexReference(refImage, quadGrid);
        else
            this.computeFragmentReference(refImage, quadGrid);

        // Compare.
        /** @type {boolean} */ var testOk = this.compareImages(resImage, refImage, 0.05);

        // De-initialize.
        gl.useProgram(null);

        if (!testOk)
            testFailedOptions("Fail", false);
        else
            testPassedOptions("Pass", true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    glsShaderRenderCase.ShaderRenderCase.prototype.iterate = function() {
        return this.postiterate();
    };

    glsShaderRenderCase.ShaderRenderCase.prototype.setupShaderData = function() {};

    /**
     * @param {?WebGLProgram} programId
     */
    glsShaderRenderCase.ShaderRenderCase.prototype.setup = function(programId) {};

    /**
     * @param {?WebGLProgram} programId
     * @param {Array<number>} constCoords
     */
    glsShaderRenderCase.ShaderRenderCase.prototype.setupUniforms = function(programId, constCoords) {};

    /**
    * @return  {Array<number>}
    */
    glsShaderRenderCase.ShaderRenderCase.prototype.getViewportSize = function() {
        return [Math.min(gl.canvas.width, glsShaderRenderCase.MAX_RENDER_WIDTH),
                Math.min(gl.canvas.height, glsShaderRenderCase.MAX_RENDER_HEIGHT)];
    };

    /**
     * @param {?WebGLProgram} programId
     */
    glsShaderRenderCase.ShaderRenderCase.prototype.setupDefaultInputs = function(programId) {
        // SETUP UNIFORMS.
        glsShaderRenderCase.setupDefaultUniforms(programId);

        // SETUP TEXTURES.
        for (var ndx = 0; ndx < this.m_textures.length; ndx++) {
            /** @type {glsShaderRenderCase.TextureBinding} */ var tex = this.m_textures[ndx];
            /** @type {tcuTexture.Sampler} */ var sampler = tex.getSampler();
            /** @type {number} */ var texTarget = gl.NONE;
            /** @type {number} */ var texObj = 0;

            if (tex.getType() === gluTexture.Type.TYPE_NONE)
                continue;

            switch (tex.getType()) {
                case gluTexture.Type.TYPE_2D:
                    texTarget = gl.TEXTURE_2D;
                    texObj = tex.getBinding().getGLTexture();
                    break;
                case gluTexture.Type.TYPE_CUBE_MAP:
                    texTarget = gl.TEXTURE_CUBE_MAP;
                    texObj = tex.getBinding().getGLTexture();
                    break;
                case gluTexture.Type.TYPE_2D_ARRAY:
                    texTarget = gl.TEXTURE_2D_ARRAY;
                    texObj = tex.getBinding().getGLTexture();
                    break;
                case gluTexture.Type.TYPE_3D:
                    texTarget = gl.TEXTURE_3D;
                    texObj = tex.getBinding().getGLTexture();
                    break;
                default:
                    throw new Error("Type not supported");
            }

            gl.activeTexture(gl.TEXTURE0+ ndx);
            gl.bindTexture(texTarget, texObj);
            gl.texParameteri(texTarget, gl.TEXTURE_WRAP_S, gluTextureUtil.getGLWrapMode(sampler.wrapS));
            gl.texParameteri(texTarget, gl.TEXTURE_WRAP_T, gluTextureUtil.getGLWrapMode(sampler.wrapT));
            gl.texParameteri(texTarget, gl.TEXTURE_MIN_FILTER, gluTextureUtil.getGLFilterMode(sampler.minFilter));
            gl.texParameteri(texTarget, gl.TEXTURE_MAG_FILTER, gluTextureUtil.getGLFilterMode(sampler.magFilter));

            if (texTarget === gl.TEXTURE_3D)
                gl.texParameteri(texTarget, gl.TEXTURE_WRAP_R, gluTextureUtil.getGLWrapMode(sampler.wrapR));

            if (sampler.compare != tcuTexture.CompareMode.COMPAREMODE_NONE)
            {
                gl.texParameteri(texTarget, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
                gl.texParameteri(texTarget, gl.TEXTURE_COMPARE_FUNC, gluTextureUtil.getGLCompareFunc(sampler.compare));
            }
        }
    };

    /**
     * @param {tcuSurface.Surface} result
     * @param {?WebGLProgram} programId
     * @param {glsShaderRenderCase.QuadGrid} quadGrid
     **/
    glsShaderRenderCase.ShaderRenderCase.prototype.render = function(result, programId, quadGrid) {
        // Buffer info.
        /** @type {number} */ var width = result.getWidth();
        /** @type {number} */ var height = result.getHeight();

        /** @type {number} */ var xOffsetMax = gl.drawingBufferWidth - width;
        /** @type {number} */ var yOffsetMax = gl.drawingBufferHeight - height;

        /** @type {number} */ var hash = deString.deStringHash(this.m_vertShaderSource) + deString.deStringHash(this.m_fragShaderSource);
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(hash);

        /** @type {number} */ var xOffset = rnd.getInt(0, xOffsetMax);
        /** @type {number} */ var yOffset = rnd.getInt(0, yOffsetMax);

        gl.viewport(xOffset, yOffset, width, height);

        // Setup program.
        this.setupUniforms(programId, quadGrid.getConstCoords());
        this.setupDefaultInputs(programId);

        // Clear.
        gl.clearColor(this.m_clearColor[0], this.m_clearColor[1], this.m_clearColor[2], this.m_clearColor[3]);
        gl.clear(gl.COLOR_BUFFER_BIT);

        // Draw.
        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays = [];
        /** @type {number} */ var numElements = quadGrid.getNumTriangles()*3;

        glsShaderRenderCase.getDefaultVertexArrays(quadGrid, programId, vertexArrays);

        gluDrawUtil.draw(gl, programId, vertexArrays, gluDrawUtil.triangles(quadGrid.getIndices()));

        // Read back results.
        result.readViewport(gl, [xOffset, yOffset, width, height]);

    };

    /**
     * @param {tcuSurface.Surface} result
     * @param {glsShaderRenderCase.QuadGrid} quadGrid
     **/
    glsShaderRenderCase.ShaderRenderCase.prototype.computeVertexReference = function(result, quadGrid) {
        // Buffer info.
        /** @type {number} */ var width = result.getWidth();
        /** @type {number} */ var height = result.getHeight();
        /** @type {number} */ var gridSize = quadGrid.getGridSize();
        /** @type {number} */ var stride = gridSize + 1;
        /** @type {boolean} */ var hasAlpha = gl.getContextAttributes().alpha;
        /** @type {glsShaderRenderCase.ShaderEvalContext} */
        var evalCtx = new glsShaderRenderCase.ShaderEvalContext(quadGrid);
        /** @type {Array<number>} */ var color = [];
        // Evaluate color for each vertex.
        /** @type {Array<Array<number>>} */ var colors = [];
        for (var y = 0; y < gridSize + 1; y++)
        for (var x = 0; x < gridSize + 1; x++) {
            /** @type {number} */ var sx = x / gridSize;
            /** @type {number} */ var sy = y / gridSize;
            /** @type {number} */ var vtxNdx = ((y * (gridSize+ 1 )) + x);

            evalCtx.reset(sx, sy);
            this.m_evaluator.evaluate(evalCtx);
            assertMsgOptions(!evalCtx.isDiscarded, 'Discard is not available in vertex shader.', false, true);
            color = evalCtx.color;

            if (!hasAlpha)
                color[3] = 1.0;

            colors[vtxNdx] = color;
        }
        // Render quads.
        for (var y = 0; y < gridSize; y++)
        for (var x = 0; x < gridSize; x++) {
            /** @type {number} */ var x0 = x / gridSize;
            /** @type {number} */ var x1 = (x + 1) / gridSize;
            /** @type {number} */ var y0 = y / gridSize;
            /** @type {number} */ var y1 = (y + 1) / gridSize;

            /** @type {number} */ var sx0 = x0 * width;
            /** @type {number} */ var sx1 = x1 * width;
            /** @type {number} */ var sy0 = y0 * height;
            /** @type {number} */ var sy1 = y1 * height;
            /** @type {number} */ var oosx = 1.0 / (sx1 - sx0);
            /** @type {number} */ var oosy = 1.0 / (sy1 - sy0);

            /** @type {number} */ var ix0 = Math.ceil(sx0 - 0.5);
            /** @type {number} */ var ix1 = Math.ceil(sx1 - 0.5);
            /** @type {number} */ var iy0 = Math.ceil(sy0 - 0.5);
            /** @type {number} */ var iy1 = Math.ceil(sy1 - 0.5);

            /** @type {number} */ var v00 = (y * stride) + x;
            /** @type {number} */ var v01 = (y * stride) + x + 1;
            /** @type {number} */ var v10 = ((y + 1) * stride) + x;
            /** @type {number} */ var v11 = ((y + 1) * stride) + x + 1;
            /** @type {Array<number>} */ var c00 = colors[v00];
            /** @type {Array<number>} */ var c01 = colors[v01];
            /** @type {Array<number>} */ var c10 = colors[v10];
            /** @type {Array<number>} */ var c11 = colors[v11];

            for (var iy = iy0; iy < iy1; iy++)
            for (var ix = ix0; ix < ix1; ix++) {
                assertMsgOptions(deMath.deInBounds32(ix, 0, width), 'Out of bounds.', false, true);
                assertMsgOptions(deMath.deInBounds32(iy, 0, height), 'Out of bounds.', false, true);

                /** @type {number} */ var sfx = ix + 0.5;
                /** @type {number} */ var sfy = iy + 0.5;
                /** @type {number} */ var fx1 = deMath.clamp((sfx - sx0) * oosx, 0.0, 1.0);
                /** @type {number} */ var fy1 = deMath.clamp((sfy - sy0) * oosy, 0.0, 1.0);

                // Triangle quad interpolation.
                /** @type {boolean} */ var tri = fx1 + fy1 <= 1.0;
                /** @type {number} */ var tx = tri ? fx1 : (1.0 - fx1);
                /** @type {number} */ var ty = tri ? fy1 : (1.0 - fy1);
                /** @type {Array<number>} */ var t0 = tri ? c00 : c11;
                /** @type {Array<number>} */ var t1 = tri ? c01 : c10;
                /** @type {Array<number>} */ var t2 = tri ? c10 : c01;
                color = deMath.add(t0, deMath.add(deMath.scale(deMath.subtract(t1, t0), tx), deMath.scale(deMath.subtract(t2, t0), ty)));

                result.setPixel(ix, iy, glsShaderRenderCase.toRGBA(color).toIVec());
            }
        }
    };

    /**
     * @param {tcuSurface.Surface} result
     * @param {glsShaderRenderCase.QuadGrid} quadGrid
     **/
    glsShaderRenderCase.ShaderRenderCase.prototype.computeFragmentReference = function(result, quadGrid) {
        // Buffer info.
        /** @type {number} */ var width = result.getWidth();
        /** @type {number} */ var height = result.getHeight();
        /** @type {boolean} */ var hasAlpha    = gl.getContextAttributes().alpha;
        /** @type {glsShaderRenderCase.ShaderEvalContext} */ var evalCtx = new glsShaderRenderCase.ShaderEvalContext(quadGrid);

        // Render.
        for (var y = 0; y < height; y++)
        for (var x = 0; x < width; x++) {
            /** @type {number} */ var sx = (x + 0.5) / width;
            /** @type {number} */ var sy = (y + 0.5) / height;

            evalCtx.reset(sx, sy);
            this.m_evaluator.evaluate(evalCtx);
            // Select either clear color or computed color based on discarded bit.
            /** @type {Array<number>} */ var color = evalCtx.isDiscarded ? this.m_clearColor : evalCtx.color;

            if (!hasAlpha)
                color[3] = 1.0;

            result.setPixel(x, y, glsShaderRenderCase.toRGBA(color).toIVec());
        }
    };

    /**
     * @param {tcuSurface.Surface} resImage
     * @param {tcuSurface.Surface} refImage
     * @param {number} errorThreshold
     * @return {boolean}
     */
    glsShaderRenderCase.ShaderRenderCase.prototype.compareImages = function(resImage, refImage, errorThreshold) {
        return tcuImageCompare.fuzzyCompare("ComparisonResult", "Image comparison result", refImage.getAccess(), resImage.getAccess(), errorThreshold);
    };

    /**
     * @param {number} number
     * @return {string} */
    glsShaderRenderCase.getIntUniformName = function(number) {
        switch (number) {
            case 0: return "ui_zero";
            case 1: return "ui_one";
            case 2: return "ui_two";
            case 3: return "ui_three";
            case 4: return "ui_four";
            case 5: return "ui_five";
            case 6: return "ui_six";
            case 7: return "ui_seven";
            case 8: return "ui_eight";
            case 101: return "ui_oneHundredOne";
            default:
                throw new Error("Uniform not supported.");
        }
    };

    /**
     * @param {number} number
     * @return {string} */
    glsShaderRenderCase.getFloatUniformName = function(number) {
        switch (number) {
            case 0: return "uf_zero";
            case 1: return "uf_one";
            case 2: return "uf_two";
            case 3: return "uf_three";
            case 4: return "uf_four";
            case 5: return "uf_five";
            case 6: return "uf_six";
            case 7: return "uf_seven";
            case 8: return "uf_eight";
            default:
                throw new Error("Uniform not supported.");
        }
    };

    /**
     * @param {number} number
     * @return {string} */
    glsShaderRenderCase.getFloatFractionUniformName = function(number) {
        switch (number) {
            case 1: return "uf_one";
            case 2: return "uf_half";
            case 3: return "uf_third";
            case 4: return "uf_fourth";
            case 5: return "uf_fifth";
            case 6: return "uf_sixth";
            case 7: return "uf_seventh";
            case 8: return "uf_eighth";
            default:
                throw new Error("Uniform not supported.");
        }
    };

    /**
     * @param {?WebGLProgram} programID
     */
    glsShaderRenderCase.setupDefaultUniforms = function(programID) {
        /** @type {?WebGLUniformLocation} */ var uniLoc;
        // Bool.
        /**
         * @constructor
         * @struct
         */
        var BoolUniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {boolean} */ this.value = value;
        };

        /** @type {Array<BoolUniform>} */ var s_boolUniforms = [
            new BoolUniform("ub_true", true),
            new BoolUniform("ub_false", false)
        ];

        for (var i = 0; i < s_boolUniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_boolUniforms[i].name);
            if (uniLoc != null)
                gl.uniform1i(uniLoc, s_boolUniforms[i].value ? 1 : 0);
        }

        // BVec4.
        /**
         * @constructor
         * @struct
         */
        var BVec4Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<boolean>} */ this.value = value;
        };

        /** @type {Array<BVec4Uniform>} */ var s_bvec4Uniforms = [
            new BVec4Uniform("ub4_true", [true, true, true, true]),
            new BVec4Uniform("ub4_false", [false, false, false, false])
        ];

        for (var i = 0; i < s_bvec4Uniforms.length; i++) {
            /** @type {BVec4Uniform} */ var uni = s_bvec4Uniforms[i];
            /** @type {Array<number>} */ var arr = [];
            arr[0] = uni.value[0] ? 1 : 0;
            arr[1] = uni.value[1] ? 1 : 0;
            arr[2] = uni.value[2] ? 1 : 0;
            arr[3] = uni.value[3] ? 1 : 0;
            uniLoc = gl.getUniformLocation(programID, uni.name);
            if (uniLoc != null)
                gl.uniform4iv(uniLoc, new Int32Array(arr));
        }

        // Int.
        /**
         * @constructor
         * @struct
         */
        var IntUniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.value = value;
        };

        /** @type {Array<IntUniform>} */ var s_intUniforms = [
            new IntUniform("ui_minusOne", -1),
            new IntUniform("ui_zero", 0),
            new IntUniform("ui_one", 1),
            new IntUniform("ui_two", 2),
            new IntUniform("ui_three", 3),
            new IntUniform("ui_four", 4),
            new IntUniform("ui_five", 5),
            new IntUniform("ui_six", 6),
            new IntUniform("ui_seven", 7),
            new IntUniform("ui_eight", 8),
            new IntUniform("ui_oneHundredOne", 101)
        ];

        for (var i = 0; i < s_intUniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_intUniforms[i].name);
            if (uniLoc != null)
                gl.uniform1i(uniLoc, s_intUniforms[i].value);
        }

        // IVec2.
        /**
         * @constructor
         * @struct
         */
        var IVec2Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<number>} */ this.value = value;
        };

        /** @type {Array<IVec2Uniform>} */ var s_ivec2Uniforms = [
            new IVec2Uniform("ui2_minusOne", [-1, -1]),
            new IVec2Uniform("ui2_zero", [0, 0]),
            new IVec2Uniform("ui2_one", [1, 1]),
            new IVec2Uniform("ui2_two", [2, 2]),
            new IVec2Uniform("ui2_four", [4, 4]),
            new IVec2Uniform("ui2_five", [5, 5])
        ];

        for (var i = 0; i < s_ivec2Uniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_ivec2Uniforms[i].name);
            if (uniLoc != null)
                gl.uniform2iv(uniLoc, new Int32Array(s_ivec2Uniforms[i].value));
        }

        // IVec3.
        /**
         * @constructor
         * @struct
         */
        var IVec3Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<number>} */ this.value = value;
        };

        /** @type {Array<IVec3Uniform>} */ var s_ivec3Uniforms = [
            new IVec3Uniform("ui3_minusOne", [-1, -1, -1]),
            new IVec3Uniform("ui3_zero", [0, 0, 0]),
            new IVec3Uniform("ui3_one", [1, 1, 1]),
            new IVec3Uniform("ui3_two", [2, 2, 2]),
            new IVec3Uniform("ui3_four", [4, 4, 4]),
            new IVec3Uniform("ui3_five", [5, 5, 5])
        ];

        for (var i = 0; i < s_ivec3Uniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_ivec3Uniforms[i].name);
            if (uniLoc != null)
                gl.uniform3iv(uniLoc, new Int32Array(s_ivec3Uniforms[i].value));
        }

        // IVec4.
        /**
         * @constructor
         * @struct
         */
        var IVec4Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<number>} */ this.value = value;
        };
        /** @type {Array<IVec4Uniform>} */ var s_ivec4Uniforms = [
            new IVec4Uniform("ui4_minusOne", [-1, -1, -1, -1]),
            new IVec4Uniform("ui4_zero", [0, 0, 0, 0]),
            new IVec4Uniform("ui4_one", [1, 1, 1, 1]),
            new IVec4Uniform("ui4_two", [2, 2, 2, 2]),
            new IVec4Uniform("ui4_four", [4, 4, 4, 4]),
            new IVec4Uniform("ui4_five", [5, 5, 5, 5])
        ];

        for (var i = 0; i < s_ivec4Uniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_ivec4Uniforms[i].name);
            if (uniLoc != null)
                gl.uniform4iv(uniLoc, new Int32Array(s_ivec4Uniforms[i].value));
        }

        // Float.
        /**
         * @constructor
         * @struct
         */
        var FloatUniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.value = value;
        };
        /** @type {Array<FloatUniform>} */ var s_floatUniforms = [
            new FloatUniform("uf_zero", 0.0),
            new FloatUniform("uf_one", 1.0),
            new FloatUniform("uf_two", 2.0),
            new FloatUniform("uf_three", 3.0),
            new FloatUniform("uf_four", 4.0),
            new FloatUniform("uf_five", 5.0),
            new FloatUniform("uf_six", 6.0),
            new FloatUniform("uf_seven", 7.0),
            new FloatUniform("uf_eight", 8.0),
            new FloatUniform("uf_half", 1.0 / 2.0),
            new FloatUniform("uf_third", 1.0 / 3.0),
            new FloatUniform("uf_fourth", 1.0 / 4.0),
            new FloatUniform("uf_fifth", 1.0 / 5.0),
            new FloatUniform("uf_sixth", 1.0 / 6.0),
            new FloatUniform("uf_seventh", 1.0 / 7.0),
            new FloatUniform("uf_eighth", 1.0 / 8.0)
        ];

        for (var i = 0; i < s_floatUniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_floatUniforms[i].name);
            if (uniLoc != null)
                gl.uniform1f(uniLoc, s_floatUniforms[i].value);
        }

        // Vec2.
        /**
         * @constructor
         * @struct
         */
        var Vec2Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<number>} */ this.value = value;
        };
        /** @type {Array<Vec2Uniform>} */ var s_vec2Uniforms = [
            new Vec2Uniform("uv2_minusOne", [-1.0, -1.0]),
            new Vec2Uniform("uv2_zero", [0.0, 0.0]),
            new Vec2Uniform("uv2_half", [0.5, 0.5]),
            new Vec2Uniform("uv2_one", [1.0, 1.0]),
            new Vec2Uniform("uv2_two", [2.0, 2.0])
        ];

        for (var i = 0; i < s_vec2Uniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_vec2Uniforms[i].name);
            if (uniLoc != null)
                gl.uniform2fv(uniLoc, new Float32Array(s_vec2Uniforms[i].value));
        }

        // Vec3.
        /**
         * @constructor
         * @struct
         */
        var Vec3Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<number>} */ this.value = value;
        };
        /** @type {Array<Vec3Uniform>} */ var s_vec3Uniforms = [
            new Vec3Uniform("uv3_minusOne", [-1.0, -1.0, -1.0]),
            new Vec3Uniform("uv3_zero", [0.0, 0.0, 0.0]),
            new Vec3Uniform("uv3_half", [0.5, 0.5, 0.5]),
            new Vec3Uniform("uv3_one", [1.0, 1.0, 1.0]),
            new Vec3Uniform("uv3_two", [2.0, 2.0, 2.0])
        ];

        for (var i = 0; i < s_vec3Uniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_vec3Uniforms[i].name);
            if (uniLoc != null)
                gl.uniform3fv(uniLoc, new Float32Array(s_vec3Uniforms[i].value));
        }

        // Vec4.
        /**
         * @constructor
         * @struct
         */
        var Vec4Uniform = function(name, value) {
            /** @type {string} */ this.name = name;
            /** @type {Array<number>} */ this.value = value;
        };
        /** @type {Array<Vec4Uniform>} */ var s_vec4Uniforms = [
            new Vec4Uniform("uv4_minusOne", [-1.0, -1.0, -1.0, -1.0]),
            new Vec4Uniform("uv4_zero", [0.0, 0.0, 0.0, 0.0]),
            new Vec4Uniform("uv4_half", [0.5, 0.5, 0.5, 0.5]),
            new Vec4Uniform("uv4_one", [1.0, 1.0, 1.0, 1.0]),
            new Vec4Uniform("uv4_two", [2.0, 2.0, 2.0, 2.0]),
            new Vec4Uniform("uv4_black", [0.0, 0.0, 0.0, 1.0]),
            new Vec4Uniform("uv4_gray", [0.5, 0.5, 0.5, 1.0]),
            new Vec4Uniform("uv4_white", [1.0, 1.0, 1.0, 1.0])
        ];

        for (var i = 0; i < s_vec4Uniforms.length; i++) {
            uniLoc = gl.getUniformLocation(programID, s_vec4Uniforms[i].name);
            if (uniLoc != null)
                gl.uniform4fv(uniLoc, new Float32Array(s_vec4Uniforms[i].value));
        }
    };

    /**
     * @param {glsShaderRenderCase.QuadGrid} quadGrid
     * @param {?WebGLProgram} program
     * @param {Array<gluDrawUtil.VertexArrayBinding>} vertexArrays
     */
    glsShaderRenderCase.getDefaultVertexArrays = function(quadGrid, program, vertexArrays) {
        /** @type {number} */ var numElements = quadGrid.getNumVertices();
        var posArray = [].concat.apply([], quadGrid.getPositions());
        var coordsArray = [].concat.apply([], quadGrid.getCoordsArray());
        var unitCoordsArray = [].concat.apply([], quadGrid.getUnitCoordsArray());

        vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding("a_position", 4, numElements, 0, posArray));
        vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding("a_coords", 4, numElements, 0, coordsArray));
        vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding("a_unitCoords", 4, numElements, 0, unitCoordsArray));
        vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding("a_one", 1, numElements, 0, quadGrid.getAttribOne()));

        // a_inN.
        for (var userNdx = 0; userNdx < quadGrid.getNumUserAttribs(); userNdx++) {
            /** @type {string} */ var name = "a_in" + userNdx;
            var userAttribArray = [].concat.apply([], quadGrid.getUserAttribByIndex(userNdx));
            vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding(name, 4, numElements, 0, userAttribArray));
        }

        // Matrix attributes - these are set by location
        /**
         * @constructor
         * @struct
         */
        var Matrix = function(name, cols, rows) {
            this.name = name;
            this.numCols = cols;
            this.numRows = rows;
        };

        /** @type {Array<Matrix>} */ var matrices = [
             new Matrix('a_mat2', 2, 2),
             new Matrix('a_mat2x3', 2, 3),
             new Matrix('a_mat2x4', 2, 4),
             new Matrix('a_mat3x2', 3, 2),
             new Matrix('a_mat3', 3, 3),
             new Matrix('a_mat3x4', 3, 4),
             new Matrix('a_mat4x2', 4, 2),
             new Matrix('a_mat4x3', 4, 3),
             new Matrix('a_mat4', 4, 4)
        ];

        for (var matNdx = 0; matNdx < matrices.length; matNdx++) {
            /** @type {number} */ var loc = gl.getAttribLocation(program, matrices[matNdx].name);

            if (loc < 0)
                continue; // Not used in shader.

            /** @type {number} */ var numRows = matrices[matNdx].numRows;
            /** @type {number} */ var numCols = matrices[matNdx].numCols;

            for (var colNdx = 0; colNdx < numCols; colNdx++) {
                var data = [].concat.apply([], quadGrid.getUserAttribByIndex(colNdx));
                vertexArrays.push(gluDrawUtil.newFloatColumnVertexArrayBinding(matrices[matNdx].name, colNdx, numRows, numElements, 4 * 4, data));
            }
        }
    };
});

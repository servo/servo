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
goog.provide('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrShaders');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');

goog.scope(function() {

    var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
    var rrShaders = framework.referencerenderer.rrShaders;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;
    var tcuTexture = framework.common.tcuTexture;
    var deMath = framework.delibs.debase.deMath;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var rrDefs = framework.referencerenderer.rrDefs;
    var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
    var rrVertexPacket = framework.referencerenderer.rrVertexPacket;
    var rrShadingContext = framework.referencerenderer.rrShadingContext;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * sglrShaderProgram.VaryingFlags
     * @constructor
     * @struct
     */
    sglrShaderProgram.VaryingFlags = function() {
        this.NONE = true; //TODO: is NONE necessary?
        this.FLATSHADE = false;
    };

    /**
     * sglrShaderProgram.VertexAttribute
     * @constructor
     * @param {string} name_
     * @param {rrGenericVector.GenericVecType} type_
     */
    sglrShaderProgram.VertexAttribute = function(name_, type_) {
        this.name = name_;
        this.type = type_;
    };

    /**
     * sglrShaderProgram.VertexToFragmentVarying
     * @constructor
     * @param {rrGenericVector.GenericVecType} type_
     * @param {sglrShaderProgram.VaryingFlags=} flags
     */
    sglrShaderProgram.VertexToFragmentVarying = function(type_, flags) {
        this.type = type_;
        this.flatshade = flags === undefined ? new sglrShaderProgram.VaryingFlags().FLATSHADE : flags.FLATSHADE;
    };

    /**
     * sglrShaderProgram.FragmentOutput
     * @constructor
     * @param {rrGenericVector.GenericVecType} type_
     */
    sglrShaderProgram.FragmentOutput = function(type_) {
        /** @type {rrGenericVector.GenericVecType} */ this.type = type_;
    };

    /**
     * sglrShaderProgram.Uniform
     * @constructor
     * @param {string} name_
     * @param {gluShaderUtil.DataType} type_
     */
    sglrShaderProgram.Uniform = function(name_, type_) {
        /** @type {string} */ this.name = name_;
        /** @type {gluShaderUtil.DataType} */ this.type = type_;
        /** @type {Array<number>} */ this.value;
        /** @type {?rrDefs.Sampler} */ this.sampler = null;
    };

    /**
     * sglrShaderProgram.VertexSource
     * @constructor
     * @param {string} str
     */
    sglrShaderProgram.VertexSource = function(str) {
        /** @type {string} */ this.source = str;
    };

    /**
     * sglrShaderProgram.FragmentSource
     * @constructor
     * @param {string} str
     */
    sglrShaderProgram.FragmentSource = function(str) {
        /** @type {string} */ this.source = str;
    };

    /**
     * sglrShaderProgram.ShaderProgramDeclaration
     * @constructor
     */
    sglrShaderProgram.ShaderProgramDeclaration = function() {
        /** @type {Array<sglrShaderProgram.VertexAttribute>} */ this.m_vertexAttributes = [];
        /** @type {Array<sglrShaderProgram.VertexToFragmentVarying>} */ this.m_vertexToFragmentVaryings = [];
        /** @type {Array<sglrShaderProgram.FragmentOutput>} */ this.m_fragmentOutputs = [];
        /** @type {Array<sglrShaderProgram.Uniform>} */ this.m_uniforms = [];
        /** @type {string} */ this.m_vertexSource;
        /** @type {string} */ this.m_fragmentSource;

        /** @type {boolean} */ this.m_vertexShaderSet = false;
        /** @type {boolean} */ this.m_fragmentShaderSet = false;
    };

    /**
     * Add a vertex attribute to the shader program declaration.
     * @param {sglrShaderProgram.VertexAttribute} v
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.pushVertexAttribute = function(v) {
        this.m_vertexAttributes.push(v);
        return this;
    };

    /**
     * Add a vertex to fragment varying to the shader program declaration.
     * @param {sglrShaderProgram.VertexToFragmentVarying} v
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.pushVertexToFragmentVarying = function(v) {
        this.m_vertexToFragmentVaryings.push(v);
        return this;
    };

    /**
     * Add a fragment output to the shader program declaration.
     * @param {sglrShaderProgram.FragmentOutput} v
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.pushFragmentOutput = function(v) {
        this.m_fragmentOutputs.push(v);
        return this;
    };

    /**
     * Add a uniform to the shader program declaration.
     * @param {sglrShaderProgram.Uniform} v
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.pushUniform = function(v) {
        this.m_uniforms.push(v);
        return this;
    };

    /**
     * @param {sglrShaderProgram.VertexSource} c
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.pushVertexSource = function(c) {
        DE_ASSERT(!this.m_vertexShaderSet);
        this.m_vertexSource = c.source;
        this.m_vertexShaderSet = true;
        return this;
    };

    /**
     * @param {sglrShaderProgram.FragmentSource} c
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.pushFragmentSource = function(c) {
        DE_ASSERT(!this.m_fragmentSource);
        /** @type {sglrShaderProgram.FragmentSource} */ this.m_fragmentSource = c.source;
        /** @type {boolean} */ this.m_fragmentShaderSet = true;
        return this;
    };

    /**
     * @return {boolean}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.valid = function() {
        if (!this.m_vertexShaderSet || !this.m_fragmentShaderSet)
            return false;

        if (this.m_fragmentOutputs.length == 0)
            return false;

        return true;
    };

    /**
     * @return {number}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.getVertexInputCount = function() {
        return this.m_vertexAttributes.length;
    };

    /**
     * @return {number}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.getVertexOutputCount = function() {
        return this.m_vertexToFragmentVaryings.length;
    };

    /**
     * @return {number}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.getFragmentInputCount = function() {
        return this.m_vertexToFragmentVaryings.length;
    };

    /**
     * @return {number}
     */
    sglrShaderProgram.ShaderProgramDeclaration.prototype.getFragmentOutputCount = function() {
        return this.m_fragmentOutputs.length;
    };

    /**
     * @constructor
     * @param {sglrShaderProgram.ShaderProgramDeclaration} decl
     */
    sglrShaderProgram.ShaderProgram = function(decl) {
        /** @type {rrShaders.VertexShader} */ this.vertexShader = new rrShaders.VertexShader(decl.getVertexInputCount(), decl.getVertexOutputCount());
        /** @type {rrShaders.FragmentShader} */ this.fragmentShader = new rrShaders.FragmentShader(decl.getFragmentInputCount(), decl.getFragmentOutputCount());

        /** @type {Array<string>} */ this.m_attributeNames = [];
        /** @type {Array<sglrShaderProgram.Uniform>} */ this.m_uniforms = [];
        /** @type {string} */ this.m_vertSrc = decl.m_vertexSource;
        /** @type {string} */ this.m_fragSrc = decl.m_fragmentSource;

        DE_ASSERT(decl.valid());

        // Set up shader IO

        for (var ndx = 0; ndx < decl.m_vertexAttributes.length; ++ndx) {
            this.vertexShader.m_inputs[ndx].type = decl.m_vertexAttributes[ndx].type;
            this.m_attributeNames[ndx] = decl.m_vertexAttributes[ndx].name;
        }

        for (var ndx = 0; ndx < decl.m_vertexToFragmentVaryings.length; ++ndx) {
            this.vertexShader.m_outputs[ndx].type = decl.m_vertexToFragmentVaryings[ndx].type;
            this.vertexShader.m_outputs[ndx].flatshade = decl.m_vertexToFragmentVaryings[ndx].flatshade;

            this.fragmentShader.m_inputs[ndx] = this.vertexShader.m_outputs[ndx];
        }

        for (var ndx = 0; ndx < decl.m_fragmentOutputs.length; ++ndx)
            this.fragmentShader.m_outputs[ndx].type = decl.m_fragmentOutputs[ndx].type;

        // Set up uniforms

        for (var ndx = 0; ndx < decl.m_uniforms.length; ++ndx)
            this.m_uniforms[ndx] = new sglrShaderProgram.Uniform(decl.m_uniforms[ndx].name, decl.m_uniforms[ndx].type);
    };

    /**
     * @return {rrShaders.VertexShader}
     */
    sglrShaderProgram.ShaderProgram.prototype.getVertexShader = function() {
        return this.vertexShader;
    };

    /**
     * @return {rrShaders.FragmentShader}
     */
    sglrShaderProgram.ShaderProgram.prototype.getFragmentShader = function() {
        return this.fragmentShader;
    };

    /**
     * @param {string} name
     * @return {sglrShaderProgram.Uniform}
     * @throws {Error}
     */
    sglrShaderProgram.ShaderProgram.prototype.getUniformByName = function(name) {
        DE_ASSERT(name);

        for (var ndx = 0; ndx < this.m_uniforms.length; ++ndx)
            if (this.m_uniforms[ndx].name == name)
                return this.m_uniforms[ndx];

        throw new Error('Invalid uniform name, uniform not found.');
    };

    /**
     * shadeFragments - abstract function, to be implemented by children classes
     * @param {Array<rrFragmentOperations.Fragment>} packets
     * @param {rrShadingContext.FragmentShadingContext} context
     * @throws {Error}
     */
    sglrShaderProgram.ShaderProgram.prototype.shadeFragments = function(packets, context) {
        throw new Error('This function needs to be overwritten in a child class.');
    };

    /**
     * shadeVertices - abstract function, to be implemented by children classes
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     * @throws {Error}
     */
     sglrShaderProgram.ShaderProgram.prototype.shadeVertices = function(inputs, packets, numPackets) {
        throw new Error('This function needs to be overwritten in a child class.');
     };

});

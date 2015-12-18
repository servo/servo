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
goog.provide('framework.referencerenderer.rrShaders');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');

goog.scope(function() {

var rrShaders = framework.referencerenderer.rrShaders;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var rrGenericVector = framework.referencerenderer.rrGenericVector;
var rrShadingContext = framework.referencerenderer.rrShadingContext;
var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
var rrVertexPacket = framework.referencerenderer.rrVertexPacket;

    /**
     * Vertex shader input information
     * @constructor
     */
    rrShaders.VertexInputInfo = function() {
        /** @type {rrGenericVector.GenericVecType} */ this.type;
    };

    /**
     * Shader varying information
     * @constructor
     */
    rrShaders.VertexVaryingInfo = function() {
        /** @type {rrGenericVector.GenericVecType} */ this.type;
        /** @type {boolean} */ var flatshade = false;
    };

    /**
     * Fragment shader output information
     * @constructor
     */
    rrShaders.FragmentOutputInfo = function() {
        //Sensible defaults
        /** @type {rrGenericVector.GenericVecType} */ this.type;
    };

    /**
     * Vertex shader interface
     *
     * Vertex shaders execute shading for set of vertex packets. See VertexPacket
     * documentation for more details on shading API.
     * @constructor
     * @param {number} numInputs
     * @param {number} numOutputs
     */
    rrShaders.VertexShader = function(numInputs, numOutputs) {
        /** @type {Array<rrShaders.VertexInputInfo>} */ this.m_inputs = [];
        for (var ndx = 0; ndx < numInputs; ndx++) this.m_inputs[ndx] = new rrShaders.VertexInputInfo();
        /** @type {Array<rrShaders.VertexVaryingInfo>} */ this.m_outputs = [];
        for (var ndx = 0; ndx < numOutputs; ndx++) this.m_outputs[ndx] = new rrShaders.VertexVaryingInfo();
    };

    /**
     * getInputs
     * @return {Array<rrShaders.VertexInputInfo>}
     */
    rrShaders.VertexShader.prototype.getInputs = function() {return this.m_inputs;};

    /**
     * getOutputs
     * @return {Array<rrShaders.VertexVaryingInfo>}
     */
    rrShaders.VertexShader.prototype.getOutputs = function() {return this.m_outputs;};

    /**
     * Fragment shader interface
     *
     * Fragment shader executes shading for list of fragment packets. See
     * FragmentPacket documentation for more details on shading API.
     * @constructor
     * @param {number} numInputs
     * @param {number} numOutputs
     */
    rrShaders.FragmentShader = function(numInputs, numOutputs) {
        /** @type {Array<rrShaders.VertexVaryingInfo>} */ this.m_inputs = [];
        for (var ndx = 0; ndx < numInputs; ndx++) this.m_inputs[ndx] = new rrShaders.VertexVaryingInfo();
        /** @type {Array<rrShaders.FragmentOutputInfo>} */ this.m_outputs = [];
        for (var ndx = 0; ndx < numOutputs; ndx++) this.m_outputs[ndx] = new rrShaders.FragmentOutputInfo();
        /** @type {*} */ this.m_container; // owner object
    };

    /**
     * getInputs
     * @return {Array<rrShaders.VertexVaryingInfo>}
     */
    rrShaders.FragmentShader.prototype.getInputs = function() {return this.m_inputs;};

    /**
     * getOutputs
     * @return {Array<rrShaders.FragmentOutputInfo>}
     */
    rrShaders.FragmentShader.prototype.getOutputs = function() {return this.m_outputs;};

});

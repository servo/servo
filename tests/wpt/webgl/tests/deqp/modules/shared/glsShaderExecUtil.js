/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL (ES) Module
 * -----------------------------------------------
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
 *//*!
 * \file
 * \brief Shader execution utilities.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('modules.shared.glsShaderExecUtil');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuMatrixUtil');
goog.require('framework.common.tcuTexture');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.gluVarType');

goog.scope(function() {

    var glsShaderExecUtil = modules.shared.glsShaderExecUtil;
    var gluVarType = framework.opengl.gluVarType;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var tcuTexture = framework.common.tcuTexture;
    var tcuMatrix = framework.common.tcuMatrix;
    var tcuMatrixUtil = framework.common.tcuMatrixUtil;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    var setParentClass = function(child, parent) {
        child.prototype = Object.create(parent.prototype);
        child.prototype.constructor = child;
    };

    /**
     * @constructor
     * @param {string=} name
     * @param {gluVarType.VarType=} varType
     */
    glsShaderExecUtil.Symbol = function(name, varType) {
        name = name === undefined ? '<unnamed>' : name;
        /** @type {string} */ this.name = name;
        /** @type {gluVarType.VarType} */ this.varType = varType || null;
    };

    //! Complete shader specification.
    /**
     * @constructor
     */
    glsShaderExecUtil.ShaderSpec = function() {
        /** @type {gluShaderUtil.GLSLVersion} */ this.version = gluShaderUtil.GLSLVersion.V300_ES; //!< Shader version.
        /** @type {Array<glsShaderExecUtil.Symbol>} */ this.inputs = [];
        /** @type {Array<glsShaderExecUtil.Symbol>} */ this.outputs = [];
        /** @type {string} */ this.globalDeclarations = ''; //!< These are placed into global scope. Can contain uniform declarations for example.
        /** @type {*} */ this.source; //!< Source snippet to be executed.
    };

    /**
     * Base class for shader executor.
     * @constructor
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     */
    glsShaderExecUtil.ShaderExecutor = function(shaderSpec) {
        /** @type {Array<glsShaderExecUtil.Symbol>} */ this.m_inputs = shaderSpec.inputs;
        /** @type {Array<glsShaderExecUtil.Symbol>} */ this.m_outputs = shaderSpec.outputs;
    };

    glsShaderExecUtil.ShaderExecutor.prototype.useProgram = function() {
        DE_ASSERT(this.isOk);
        gl.useProgram(this.getProgram());
    };

    /**
     * @return {boolean}
     */
    glsShaderExecUtil.ShaderExecutor.prototype.isOk = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @return {WebGLProgram}
     */
    glsShaderExecUtil.ShaderExecutor.prototype.getProgram = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @param {number} numValues
     * @param {Array<Array<number>>} inputs
     * @return {Array<goog.TypedArray>} outputs
     */
    glsShaderExecUtil.ShaderExecutor.prototype.execute = function(numValues, inputs) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * Base class for shader executor.
     * @param {gluShaderProgram.shaderType} shaderType
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     * @return {glsShaderExecUtil.ShaderExecutor}
     */
    glsShaderExecUtil.createExecutor = function(shaderType, shaderSpec) {
        switch (shaderType) {
            case gluShaderProgram.shaderType.VERTEX: return new glsShaderExecUtil.VertexShaderExecutor(shaderSpec);
            case gluShaderProgram.shaderType.FRAGMENT: return new glsShaderExecUtil.FragmentShaderExecutor(shaderSpec);
            default:
                throw new Error('Unsupported shader type: ' + shaderType);
        }
    };

    /**
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     * @return {string}
     */
    glsShaderExecUtil.generateVertexShader = function(shaderSpec) {
        /** @type {boolean} */ var usesInout = true;
        /** @type {string} */ var in_ = usesInout ? 'in' : 'attribute';
        /** @type {string} */ var out = usesInout ? 'out' : 'varying';
        /** @type {string} */ var src = '';
        /** @type {number} */ var vecSize;
        /** @type {gluShaderUtil.DataType} */ var intBaseType;

        src += '#version 300 es\n';

        if (shaderSpec.globalDeclarations.length > 0)
            src += (shaderSpec.globalDeclarations + '\n');

        for (var i = 0; i < shaderSpec.inputs.length; ++i)
            src += (in_ + ' ' + gluVarType.declareVariable(shaderSpec.inputs[i].varType, shaderSpec.inputs[i].name) + ';\n');

        for (var i = 0; i < shaderSpec.outputs.length; i++) {
            var output = shaderSpec.outputs[i];
            DE_ASSERT(output.varType.isBasicType());

            if (gluShaderUtil.isDataTypeBoolOrBVec(output.varType.getBasicType())) {
                vecSize = gluShaderUtil.getDataTypeScalarSize(output.varType.getBasicType());
                intBaseType = vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.INT, vecSize) : gluShaderUtil.DataType.INT;
                /** @type {gluVarType.VarType} */ var intType = new gluVarType.VarType().VarTypeBasic(intBaseType, gluShaderUtil.precision.PRECISION_HIGHP);

                src += ('flat ' + out + ' ' + gluVarType.declareVariable(intType, 'o_' + output.name) + ';\n');
            } else
                src += ('flat ' + out + ' ' + gluVarType.declareVariable(output.varType, output.name) + ';\n');
        }

        src += '\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = vec4(0.0);\n' +
            ' gl_PointSize = 1.0;\n\n';

        // Declare necessary output variables (bools).
        for (var i = 0; i < shaderSpec.outputs.length; i++) {
            if (gluShaderUtil.isDataTypeBoolOrBVec(shaderSpec.outputs[i].varType.getBasicType()))
                src += ('\t' + gluVarType.declareVariable(shaderSpec.outputs[i].varType, shaderSpec.outputs[i].name) + ';\n');
        }

        //Operation - indented to correct level.
        // TODO: Add indenting
        src += shaderSpec.source;

        // Assignments to outputs.
        for (var i = 0; i < shaderSpec.outputs.length; i++) {
            if (gluShaderUtil.isDataTypeBoolOrBVec(output.varType.getBasicType())) {
                vecSize = gluShaderUtil.getDataTypeScalarSize(output.varType.getBasicType());
                intBaseType = vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.INT, vecSize) : gluShaderUtil.DataType.INT;

                src += ('\to_' + output.name + ' = ' + gluShaderUtil.getDataTypeName(intBaseType) + '(' + output.name + ');\n');
            }
        }

        src += '}\n';

        return src;
    };

    /**
     * @return {string}
     */
    glsShaderExecUtil.generateEmptyFragmentSource = function() {
        /** @type {string} */ var src;

        src = '#version 300 es\n';
        src += 'out lowp vec4 color;\n';
        src += 'void main (void)\n{\n';
        src += ' color = vec4(0.0);\n';
        src += '}\n';

        return src;
    };

    /**
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     * @param {string} inputPrefix
     * @param {string} outputPrefix
     * @return {string}
     */
    glsShaderExecUtil.generatePassthroughVertexShader = function(shaderSpec, inputPrefix, outputPrefix) {
        // flat qualifier is not present in earlier versions?
        // DE_ASSERT(glu::glslVersionUsesInOutQualifiers(shaderSpec.version));

        /** @type {string} */ var src;

        src = '#version 300 es\n' +
            'in highp vec4 a_position;\n';

        for (var i = 0; i < shaderSpec.inputs.length; i++) {
            src += ('in ' + gluVarType.declareVariable(shaderSpec.inputs[i].varType, inputPrefix + shaderSpec.inputs[i].name) + ';\n' +
                'flat out ' + gluVarType.declareVariable(shaderSpec.inputs[i].varType, outputPrefix + shaderSpec.inputs[i].name) + ';\n');
        }

        src += '\nvoid main (void)\n{\n' +
            ' gl_Position = a_position;\n' +
            ' gl_PointSize = 1.0;\n';

        for (var i = 0; i < shaderSpec.inputs.length; i++)
            src += ('\t' + outputPrefix + shaderSpec.inputs[i].name + ' = ' + inputPrefix + shaderSpec.inputs[i].name + ';\n');

        src += '}\n';

        return src;
    };

    /**
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     * @param {boolean} useIntOutputs
     * @param {*} outLocationMap
     * @return {string}
     */
    glsShaderExecUtil.generateFragmentShader = function(shaderSpec, useIntOutputs, outLocationMap) {
        /** @type {number} */ var vecSize;
        /** @type {number} */ var numVecs;
        /** @type {gluShaderUtil.DataType} */ var intBasicType;
        /** @type {gluShaderUtil.DataType} */ var uintBasicType;
        /** @type {gluVarType.VarType} */ var uintType;
        /** @type {gluVarType.VarType} */ var intType;

        /** @type {string} */ var src;
        src = '#version 300 es\n';

        if (!shaderSpec.globalDeclarations.length > 0)
            src += (shaderSpec.globalDeclarations + '\n');

        for (var i = 0; i < shaderSpec.inputs.length; i++)
            src += ('flat in ' + gluVarType.declareVariable(shaderSpec.inputs[i].varType, shaderSpec.inputs[i].name) + ';\n');

        for (var outNdx = 0; outNdx < shaderSpec.outputs.length; ++outNdx) {
            /** @type {glsShaderExecUtil.Symbol} */ var output = shaderSpec.outputs[outNdx];
            /** @type {number} */ var location = outLocationMap[output.name];
            /** @type {string} */ var outVarName = 'o_' + output.name;
            /** @type {gluVarType.VariableDeclaration} */ var decl = new gluVarType.VariableDeclaration(output.varType, outVarName, gluVarType.Storage.STORAGE_OUT, undefined, new gluVarType.Layout(location));

            DE_ASSERT(output.varType.isBasicType());

            if (useIntOutputs && gluShaderUtil.isDataTypeFloatOrVec(output.varType.getBasicType())) {
                vecSize = gluShaderUtil.getDataTypeScalarSize(output.varType.getBasicType());
                uintBasicType = vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.UINT, vecSize) : gluShaderUtil.DataType.UINT;
                uintType = gluVarType.newTypeBasic(uintBasicType, gluShaderUtil.precision.PRECISION_HIGHP);

                decl.varType = uintType;
                src += (decl + ';\n');
            } else if (gluShaderUtil.isDataTypeBoolOrBVec(output.varType.getBasicType())) {
                vecSize = gluShaderUtil.getDataTypeScalarSize(output.varType.getBasicType());
                intBasicType = vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.INT, vecSize) : gluShaderUtil.DataType.INT;
                intType = gluVarType.newTypeBasic(intBasicType, gluShaderUtil.precision.PRECISION_HIGHP);

                decl.varType = intType;
                src += (decl + ';\n');
            } else if (gluShaderUtil.isDataTypeMatrix(output.varType.getBasicType())) {
                vecSize = gluShaderUtil.getDataTypeMatrixNumRows(output.varType.getBasicType());
                numVecs = gluShaderUtil.getDataTypeMatrixNumColumns(output.varType.getBasicType());
                uintBasicType = gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.UINT, vecSize);
                uintType = gluVarType.newTypeBasic(uintBasicType, gluShaderUtil.precision.PRECISION_HIGHP);

                decl.varType = uintType;
                for (var vecNdx = 0; vecNdx < numVecs; ++vecNdx) {
                    decl.name = outVarName + '_' + (vecNdx);
                    decl.layout.location = location + vecNdx;
                    src += (decl + ';\n');
                }
            } else //src += '';//glu::VariableDeclaration(output.varType, output.name, glu::STORAGE_OUT, glu::INTERPOLATION_LAST, location) << ";\n";
                src += new gluVarType.VariableDeclaration(output.varType, output.name, gluVarType.Storage.STORAGE_OUT, undefined, new gluVarType.Layout(location)) + ';\n';
        }

        src += '\nvoid main (void)\n{\n';

        for (var i = 0; i < shaderSpec.outputs.length; i++) {
            if ((useIntOutputs && gluShaderUtil.isDataTypeFloatOrVec(shaderSpec.outputs[i].varType.getBasicType())) ||
                gluShaderUtil.isDataTypeBoolOrBVec(shaderSpec.outputs[i].varType.getBasicType()) ||
                gluShaderUtil.isDataTypeMatrix(shaderSpec.outputs[i].varType.getBasicType()))
                src += ('\t' + gluVarType.declareVariable(shaderSpec.outputs[i].varType, shaderSpec.outputs[i].name) + ';\n');
        }

        // Operation - indented to correct level.
        // TODO: Add indenting
        src += shaderSpec.source;
        // {
        //     std::istringstream opSrc (shaderSpec.source);
        //     /** @type{number} */ var line;
        //
        //     while (std::getline(opSrc, line))
        //         src += ('\t' << line << '\n');
        // }

        for (var i = 0; i < shaderSpec.outputs.length; i++) {
            if (useIntOutputs && gluShaderUtil.isDataTypeFloatOrVec(shaderSpec.outputs[i].varType.getBasicType()))
                src += (' o_' + shaderSpec.outputs[i].name + ' = floatBitsToUint(' + shaderSpec.outputs[i].name + ');\n');
            else if (gluShaderUtil.isDataTypeMatrix(shaderSpec.outputs[i].varType.getBasicType())) {
                numVecs = gluShaderUtil.getDataTypeMatrixNumColumns(shaderSpec.outputs[i].varType.getBasicType());

                for (var vecNdx = 0; vecNdx < numVecs; ++vecNdx)
                    if (useIntOutputs)
                        src += ('\to_' + shaderSpec.outputs[i].name + '_' + vecNdx + ' = floatBitsToUint(' + shaderSpec.outputs[i].name + '[' + vecNdx + ']);\n');
                    else
                        src += ('\to_' + shaderSpec.outputs[i].name + '_' + vecNdx + ' = ' + shaderSpec.outputs[i].name + '[' + vecNdx + '];\n');
            } else if (gluShaderUtil.isDataTypeBoolOrBVec(shaderSpec.outputs[i].varType.getBasicType())) {
                vecSize = gluShaderUtil.getDataTypeScalarSize(shaderSpec.outputs[i].varType.getBasicType());
                intBasicType = vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.INT, vecSize) : gluShaderUtil.DataType.INT;

                src += ('\to_' + shaderSpec.outputs[i].name + ' = ' + gluShaderUtil.getDataTypeName(intBasicType) + '(' + shaderSpec.outputs[i].name + ');\n');
            }
        }

        src += '}\n';

        return src;
    };

    /**
     * @param {Array<glsShaderExecUtil.Symbol>} outputs
     * @return {gluShaderProgram.TransformFeedbackVaryings}
     */
    glsShaderExecUtil.getTFVaryings = function(outputs) {
        var names = [];
        for (var i = 0; i < outputs.length; i++) {
            if (gluShaderUtil.isDataTypeBoolOrBVec(outputs[i].varType.getBasicType())) {
                names.push('o_' + outputs[i].name);
            } else {
                names.push(outputs[i].name);
            }
        }
        return new gluShaderProgram.TransformFeedbackVaryings(names);
    };

    // VertexProcessorExecutor (base class for vertex and geometry executors)

    /**
     * @constructor
     * @extends {glsShaderExecUtil.ShaderExecutor}
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     * @param {gluShaderProgram.ProgramSources} sources
     */
    glsShaderExecUtil.VertexProcessorExecutor = function(shaderSpec, sources) {
        sources.add(glsShaderExecUtil.getTFVaryings(shaderSpec.outputs));
        sources.add(new gluShaderProgram.TransformFeedbackMode(gl.INTERLEAVED_ATTRIBS));
        glsShaderExecUtil.ShaderExecutor.call(this, shaderSpec);
        this.m_program = new gluShaderProgram.ShaderProgram(gl, sources);
    };

    setParentClass(glsShaderExecUtil.VertexProcessorExecutor, glsShaderExecUtil.ShaderExecutor);

    /**
     * @return {boolean}
     */
    glsShaderExecUtil.VertexProcessorExecutor.prototype.isOk = function() {
        return this.m_program.isOk();
    };

    /**
     * @return {WebGLProgram}
     */
    glsShaderExecUtil.VertexProcessorExecutor.prototype.getProgram = function() {
        return this.m_program.getProgram();
    };

    /**
     * @param {Array<*>} arr
     * @return {number}
     */
    glsShaderExecUtil.computeTotalScalarSize = function(arr) {
        /** @type {number} */ var size = 0;
        for (var i = 0; i < arr.length; i++)
            size += arr[i].varType.getScalarSize();
        return size;
    };

    /**
     * @param {Array<number>} ptr
     * @param {number} colNdx
     * @param {number} size Column size
     * @return {Array<number>}
     */
    glsShaderExecUtil.getColumn = function(ptr, colNdx, size) {
        var begin = colNdx * size;
        var end = (colNdx + 1) * size;
        return ptr.slice(begin, end);
    };

    glsShaderExecUtil.VertexProcessorExecutor.prototype.execute = function(numValues, inputs) {
        /** @type {glsShaderExecUtil.Symbol} */ var symbol;
        var outputs = [];
        /** @type {boolean} */ var useTFObject = true;
        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays = [];
        var transformFeedback = gl.createTransformFeedback();
        var outputBuffer = gl.createBuffer();

        /** @type {number} */ var outputBufferStride = glsShaderExecUtil.computeTotalScalarSize(this.m_outputs) * 4;

        // Setup inputs.
        for (var inputNdx = 0; inputNdx < this.m_inputs.length; inputNdx++) {
            symbol = this.m_inputs[inputNdx];
            /*const void* */var ptr = inputs[inputNdx];
            /** @type {gluShaderUtil.DataType} */ var basicType = symbol.varType.getBasicType();
            /** @type {number} */ var vecSize = gluShaderUtil.getDataTypeScalarSize(basicType);

            if (gluShaderUtil.isDataTypeFloatOrVec(basicType))
                vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding(symbol.name, vecSize, numValues, 0, ptr));
            else if (gluShaderUtil.isDataTypeIntOrIVec(basicType))
                vertexArrays.push(gluDrawUtil.newInt32VertexArrayBinding(symbol.name, vecSize, numValues, 0, ptr));
            else if (gluShaderUtil.isDataTypeUintOrUVec(basicType))
                vertexArrays.push(gluDrawUtil.newUint32VertexArrayBinding(symbol.name, vecSize, numValues, 0, ptr));
            else if (gluShaderUtil.isDataTypeMatrix(basicType)) {
                /** @type {number} */ var numRows = gluShaderUtil.getDataTypeMatrixNumRows(basicType);
                /** @type {number} */ var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(basicType);
                // A matrix consists of several (column-major) vectors. A buffer is created for
                // every vector in gluDrawUtil.draw() below. Data in every buffer will be tightly
                // packed. So the stride should be 0. This is different from the code in native
                // deqp, which use only one buffer for a matrix, the data is interleaved.
                /** @type {number} */ var stride = 0;

                for (var colNdx = 0; colNdx < numCols; ++colNdx)
                    vertexArrays.push(gluDrawUtil.newFloatColumnVertexArrayBinding(symbol.name,
                        colNdx,
                        numRows,
                        numValues,
                        stride,
                        glsShaderExecUtil.getColumn(ptr, colNdx, numRows * numValues)));
            } else
                DE_ASSERT(false);
        }

        // Setup TF outputs.
        if (useTFObject)
            gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, transformFeedback);
        gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, outputBuffer);
        gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, outputBufferStride * numValues, gl.STREAM_READ);
        gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, outputBuffer);

        // Draw with rasterization disabled.
        gl.beginTransformFeedback(gl.POINTS);
        gl.enable(gl.RASTERIZER_DISCARD);
        gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays,
            new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.POINTS, numValues));
        gl.disable(gl.RASTERIZER_DISCARD);
        gl.endTransformFeedback();

        // Read back data.
        var result = new ArrayBuffer(outputBufferStride * numValues);
        gl.getBufferSubData(gl.TRANSFORM_FEEDBACK_BUFFER, 0, new Uint8Array(result));
        /** @type {number} */ var curOffset = 0; // Offset in buffer in bytes.

        for (var outputNdx = 0; outputNdx < this.m_outputs.length; outputNdx++) {
            symbol = this.m_outputs[outputNdx];
            /** @type {number} */ var scalarSize = symbol.varType.getScalarSize();
            var readPtr = new Uint8Array(result, curOffset);

            if (scalarSize * 4 === outputBufferStride)
                outputs[outputNdx] = readPtr;
            else {
                var dstPtr = new Uint8Array(scalarSize * numValues * 4);

                for (var ndx = 0; ndx < numValues; ndx++)
                    for (var j = 0; j < scalarSize * 4; j++) {
                        dstPtr[scalarSize * 4 * ndx + j] = readPtr[ndx * outputBufferStride + j];
                    }
                outputs[outputNdx] = dstPtr;
            }
            curOffset += scalarSize * 4;
          }

        if (useTFObject)
            gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
        gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, null);

        return outputs;
    };

    // VertexShaderExecutor

    /**
     * @constructor
     * @extends {glsShaderExecUtil.VertexProcessorExecutor}
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     */
    glsShaderExecUtil.VertexShaderExecutor = function(shaderSpec) {
        var sources = gluShaderProgram.makeVtxFragSources(glsShaderExecUtil.generateVertexShader(shaderSpec),
            glsShaderExecUtil.generateEmptyFragmentSource());
        glsShaderExecUtil.VertexProcessorExecutor.call(this, shaderSpec, sources);
    };

    setParentClass(glsShaderExecUtil.VertexShaderExecutor, glsShaderExecUtil.VertexProcessorExecutor);

    /**
     * @constructor
     * @extends {glsShaderExecUtil.ShaderExecutor}
     * @param {glsShaderExecUtil.ShaderSpec} shaderSpec
     */
    glsShaderExecUtil.FragmentShaderExecutor = function(shaderSpec) {
        glsShaderExecUtil.ShaderExecutor.call(this, shaderSpec);
        /** @type {Array<glsShaderExecUtil.Symbol>} */ this.m_outLocationSymbols = [];
        this.m_outLocationMap = glsShaderExecUtil.generateLocationMap(this.m_outputs, this.m_outLocationSymbols);
        var sources = gluShaderProgram.makeVtxFragSources(glsShaderExecUtil.generatePassthroughVertexShader(shaderSpec, 'a_', ''),
            glsShaderExecUtil.generateFragmentShader(shaderSpec, true, this.m_outLocationMap));
        this.m_program = new gluShaderProgram.ShaderProgram(gl, sources);
    };

    setParentClass(glsShaderExecUtil.FragmentShaderExecutor, glsShaderExecUtil.ShaderExecutor);

    /**
     * @return {boolean}
     */
    glsShaderExecUtil.FragmentShaderExecutor.prototype.isOk = function() {
        return this.m_program.isOk();
    };

    /**
     * @return {WebGLProgram}
     */
    glsShaderExecUtil.FragmentShaderExecutor.prototype.getProgram = function() {
        return this.m_program.getProgram();
    };

    /**
     * @param {gluVarType.VarType} outputType
     * @param {boolean} useIntOutputs
     * @return {tcuTexture.TextureFormat}
     */
    glsShaderExecUtil.getRenderbufferFormatForOutput = function(outputType, useIntOutputs) {
        var channelOrderMap = [
            tcuTexture.ChannelOrder.R,
            tcuTexture.ChannelOrder.RG,
            tcuTexture.ChannelOrder.RGBA, // No RGB variants available.
            tcuTexture.ChannelOrder.RGBA
        ];

        var basicType = outputType.getBasicType();
        var numComps = gluShaderUtil.getDataTypeNumComponents(basicType);
        var channelType;

        switch (gluShaderUtil.getDataTypeScalarType(basicType)) {
            case 'uint': channelType = tcuTexture.ChannelType.UNSIGNED_INT32; break;
            case 'int': channelType = tcuTexture.ChannelType.SIGNED_INT32; break;
            case 'bool': channelType = tcuTexture.ChannelType.SIGNED_INT32; break;
            case 'float': channelType = useIntOutputs ? tcuTexture.ChannelType.UNSIGNED_INT32 : tcuTexture.ChannelType.FLOAT; break;
            default:
                throw new Error('Invalid output type ' + gluShaderUtil.getDataTypeScalarType(basicType));
        }

        return new tcuTexture.TextureFormat(channelOrderMap[numComps - 1], channelType);
    };

    glsShaderExecUtil.FragmentShaderExecutor.prototype.execute = function(numValues, inputs) {
        /** @type {boolean} */ var useIntOutputs = true;
        /** @type {glsShaderExecUtil.Symbol} */ var symbol;
        var outputs = [];
        var maxRenderbufferSize = /** @type {number} */ (gl.getParameter(gl.MAX_RENDERBUFFER_SIZE));
        /** @type {number} */ var framebufferW = Math.min(maxRenderbufferSize, numValues);
        /** @type {number} */ var framebufferH = Math.ceil(numValues / framebufferW);

        var framebuffer = gl.createFramebuffer();
        var renderbuffers = [];
        for (var i = 0; i < this.m_outLocationSymbols.length; i++)
         renderbuffers.push(gl.createRenderbuffer());

        var vertexArrays = [];
        var positions = [];

        if (framebufferH > maxRenderbufferSize)
            throw new Error('Value count is too high for maximum supported renderbuffer size');

        // Compute positions - 1px points are used to drive fragment shading.
        for (var valNdx = 0; valNdx < numValues; valNdx++) {
            /** @type {number} */ var ix = valNdx % framebufferW;
            /** @type {number} */ var iy = Math.floor(valNdx / framebufferW);
            var fx = -1 + 2 * (ix + 0.5) / framebufferW;
            var fy = -1 + 2 * (iy + 0.5) / framebufferH;

            positions[2 * valNdx] = fx;
            positions[2 * valNdx + 1] = fy;
        }

        // Vertex inputs.
        vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding('a_position', 2, numValues, 0, positions));

        for (var inputNdx = 0; inputNdx < this.m_inputs.length; inputNdx++) {
            symbol = this.m_inputs[inputNdx];
            var attribName = 'a_' + symbol.name;
            var ptr = inputs[inputNdx];
            /** @type {gluShaderUtil.DataType} */ var basicType = symbol.varType.getBasicType();
            /** @type {number} */ var vecSize = gluShaderUtil.getDataTypeScalarSize(basicType);

            if (gluShaderUtil.isDataTypeFloatOrVec(basicType))
                vertexArrays.push(gluDrawUtil.newFloatVertexArrayBinding(attribName, vecSize, numValues, 0, ptr));
            else if (gluShaderUtil.isDataTypeIntOrIVec(basicType))
                vertexArrays.push(gluDrawUtil.newInt32VertexArrayBinding(attribName, vecSize, numValues, 0, ptr));
            else if (gluShaderUtil.isDataTypeUintOrUVec(basicType))
                vertexArrays.push(gluDrawUtil.newUint32VertexArrayBinding(attribName, vecSize, numValues, 0, ptr));
            else if (gluShaderUtil.isDataTypeMatrix(basicType)) {
                var numRows = gluShaderUtil.getDataTypeMatrixNumRows(basicType);
                var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(basicType);
                // A matrix consists of several (column-major) vectors. A buffer is created for
                // every vector in gluDrawUtil.draw() below. Data in every buffer will be tightly
                // packed. So the stride should be 0. This is different from the code in native
                // deqp, which use only one buffer for a matrix, the data is interleaved.
                var stride = 0;

                for (var colNdx = 0; colNdx < numCols; ++colNdx)
                    vertexArrays.push(gluDrawUtil.newFloatColumnVertexArrayBinding(attribName,
                       colNdx,
                       numRows,
                       numValues,
                       stride,
                       glsShaderExecUtil.getColumn(ptr, colNdx, numRows * numValues)));
            } else
                DE_ASSERT(false);
        }

        // Construct framebuffer.
        gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);

        for (var outNdx = 0; outNdx < this.m_outLocationSymbols.length; ++outNdx) {
            symbol = this.m_outLocationSymbols[outNdx];
            var renderbuffer = renderbuffers[outNdx];
            var format = gluTextureUtil.getInternalFormat(glsShaderExecUtil.getRenderbufferFormatForOutput(symbol.varType, useIntOutputs));

            gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
            gl.renderbufferStorage(gl.RENDERBUFFER, format, framebufferW, framebufferH);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + outNdx, gl.RENDERBUFFER, renderbuffer);
        }
        gl.bindRenderbuffer(gl.RENDERBUFFER, null);
        assertMsgOptions(gl.checkFramebufferStatus(gl.FRAMEBUFFER) == gl.FRAMEBUFFER_COMPLETE, 'Framebuffer is incomplete', false, true);

        var drawBuffers = [];
        for (var ndx = 0; ndx < this.m_outLocationSymbols.length; ndx++)
            drawBuffers[ndx] = gl.COLOR_ATTACHMENT0 + ndx;
        gl.drawBuffers(drawBuffers);

        // Render
        gl.viewport(0, 0, framebufferW, framebufferH);
        gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays,
        new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.POINTS, numValues));

        // Read back pixels.

        // \todo [2013-08-07 pyry] Some fast-paths could be added here.

        for (var outNdx = 0; outNdx < this.m_outputs.length; ++outNdx) {
            symbol = this.m_outputs[outNdx];
            /** @type {number} */ var outSize = symbol.varType.getScalarSize();
            /** @type {number} */ var outVecSize = gluShaderUtil.getDataTypeNumComponents(symbol.varType.getBasicType());
            /** @type {number} */ var outNumLocs = gluShaderUtil.getDataTypeNumLocations(symbol.varType.getBasicType());
            var format = glsShaderExecUtil.getRenderbufferFormatForOutput(symbol.varType, useIntOutputs);
            var readFormat = new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, format.type);
            var transferFormat = gluTextureUtil.getTransferFormat(readFormat);
            /** @type {number} */ var outLocation = this.m_outLocationMap[symbol.name];
            var tmpBuf = new tcuTexture.TextureLevel(readFormat, framebufferW, framebufferH);

            for (var locNdx = 0; locNdx < outNumLocs; ++locNdx) {
                gl.readBuffer(gl.COLOR_ATTACHMENT0 + outLocation + locNdx);
                gl.readPixels(0, 0, framebufferW, framebufferH, transferFormat.format, transferFormat.dataType, tmpBuf.getAccess().getDataPtr());

                if (outSize == 4 && outNumLocs == 1) {
                    outputs[outNdx] = new Uint8Array(tmpBuf.getAccess().getBuffer());
                } else {
                    if (locNdx == 0)
                        outputs[outNdx] = new Uint32Array(numValues * outVecSize);
                    var srcPtr = new Uint32Array(tmpBuf.getAccess().getBuffer());
                    for (var valNdx = 0; valNdx < numValues; valNdx++) {
                        var srcOffset = valNdx * 4;
                        var dstOffset = outSize * valNdx + outVecSize * locNdx;
                        for (var j = 0; j < outVecSize; j++)
                        outputs[outNdx][dstOffset + j] = srcPtr[srcOffset + j];
                    }
                }
            }
        }

        // \todo [2013-08-07 pyry] Clear draw buffers & viewport?
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        return outputs;
    };

    glsShaderExecUtil.generateLocationMap = function(symbols, locationSymbols) {
        var ret = [];
        locationSymbols.length = 0;
        var location = 0;

        for (var i = 0; i < symbols.length; i++) {
            var symbol = symbols[i];
            var numLocations = gluShaderUtil.getDataTypeNumLocations(symbol.varType.getBasicType());
            ret[symbol.name] = location;
            location += numLocations;

            for (var ndx = 0; ndx < numLocations; ++ndx)
                locationSymbols.push(symbol);
        }

        return ret;
    };

});

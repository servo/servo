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
goog.provide('functional.gles3.es3fInstancedRenderingTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

var es3fInstancedRenderingTests = functional.gles3.es3fInstancedRenderingTests;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuTestCase = framework.common.tcuTestCase;
var tcuSurface = framework.common.tcuSurface;
var deString = framework.delibs.debase.deString;
var deRandom = framework.delibs.debase.deRandom;
var tcuImageCompare = framework.common.tcuImageCompare;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var deMath = framework.delibs.debase.deMath;

    /** @type {?WebGL2RenderingContext} */ var gl;

    /** @const @type {number} */ es3fInstancedRenderingTests.MAX_RENDER_WIDTH = 128;
    /** @const @type {number} */ es3fInstancedRenderingTests.MAX_RENDER_HEIGHT = 128;

    /** @const @type {number} */ es3fInstancedRenderingTests.QUAD_GRID_SIZE = 127;

    // Attribute divisors for the attributes defining the color's RGB components.
    /** @const @type {number} */es3fInstancedRenderingTests.ATTRIB_DIVISOR_R = 3;
    /** @const @type {number} */es3fInstancedRenderingTests.ATTRIB_DIVISOR_G = 2;
    /** @const @type {number} */es3fInstancedRenderingTests.ATTRIB_DIVISOR_B = 1;

    /** @const @type {number} */es3fInstancedRenderingTests.OFFSET_COMPONENTS = 3; // \note Affects whether a float or a vecN is used in shader, but only first component is non-zero.

    // Scale and bias values when converting float to integer, when attribute is of integer type.
    /** @const @type {number} */es3fInstancedRenderingTests.FLOAT_INT_SCALE = 100.0;
    /** @const @type {number} */es3fInstancedRenderingTests.FLOAT_INT_BIAS = -50.0;
    /** @const @type {number} */es3fInstancedRenderingTests.FLOAT_UINT_SCALE = 100.0;
    /** @const @type {number} */es3fInstancedRenderingTests.FLOAT_UINT_BIAS = 0.0;

    var DE_ASSERT = function(expression) {
        if (!expression) throw new Error('Assert failed');
    };

    es3fInstancedRenderingTests.TCU_FAIL = function(message) {
        throw new Error(message);
    };

    // es3fInstancedRenderingTests.InstancedRenderingCase

    /**
     * es3fInstancedRenderingTests.DrawFunction
     * @enum {number}
     */
    es3fInstancedRenderingTests.DrawFunction = {
            FUNCTION_DRAW_ARRAYS_INSTANCED: 0,
            FUNCTION_DRAW_ELEMENTS_INSTANCED: 1
    };

    /**
     * es3fInstancedRenderingTests.InstancingType
     * @enum {number}
     */
    es3fInstancedRenderingTests.InstancingType = {
            TYPE_INSTANCE_ID: 0,
            TYPE_ATTRIB_DIVISOR: 1,
            TYPE_MIXED: 2
    };

    /**
    * es3fInstancedRenderingTests.InstancedRenderingCase class, inherits from TestCase class
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    * @param {string} name
    * @param {string} description
    * @param {es3fInstancedRenderingTests.DrawFunction} drawFunction
    * @param {es3fInstancedRenderingTests.InstancingType} instancingType
    * @param {gluShaderUtil.DataType} rgbAttrType
    * @param {number} numInstances
    */
    es3fInstancedRenderingTests.InstancedRenderingCase = function(name, description, drawFunction, instancingType, rgbAttrType, numInstances) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {es3fInstancedRenderingTests.DrawFunction} */ this.m_function = drawFunction;
        /** @type {es3fInstancedRenderingTests.InstancingType} */ this.m_instancingType = instancingType;
        /** @type {gluShaderUtil.DataType} */ this.m_rgbAttrType = rgbAttrType;
        /** @type {number} */ this.m_numInstances = numInstances;
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {Array<number>} */ this.m_gridVertexPositions = [];
        /** @type {Array<number>} */ this.m_gridIndices = [];
        /** @type {Array<number>} */ this.m_instanceOffsets = [];
        /** @type {Array<number>} */ this.m_instanceColorR = [];
        /** @type {Array<number>} */ this.m_instanceColorG = [];
        /** @type {Array<number>} */ this.m_instanceColorB = [];
    };

    es3fInstancedRenderingTests.InstancedRenderingCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.constructor = es3fInstancedRenderingTests.InstancedRenderingCase;

    /**
    * Helper function that does biasing and scaling when converting float to integer.
    * @param {Array<number>} vec
    * @param {number} val
    */
    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.pushVarCompAttrib = function(vec, val) {
        var isFloatCase = gluShaderUtil.isDataTypeFloatOrVec(this.m_rgbAttrType);
        var isIntCase = gluShaderUtil.isDataTypeIntOrIVec(this.m_rgbAttrType);
        var isUintCase = gluShaderUtil.isDataTypeUintOrUVec(this.m_rgbAttrType);
        var isMatCase = gluShaderUtil.isDataTypeMatrix(this.m_rgbAttrType);
        if (isFloatCase || isMatCase)
            vec.push(val);
        else if (isIntCase)
            vec.push(val * es3fInstancedRenderingTests.FLOAT_INT_SCALE + es3fInstancedRenderingTests.FLOAT_INT_BIAS);
        else if (isUintCase)
            vec.push(val * es3fInstancedRenderingTests.FLOAT_UINT_SCALE + es3fInstancedRenderingTests.FLOAT_UINT_BIAS);
        else
            throw new Error('Invalid attribute type.');
    };

    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.init = function() {
        // Clear errors from previous tests
        gl.getError();

        /** @type {boolean} */ var isFloatCase = gluShaderUtil.isDataTypeFloatOrVec(this.m_rgbAttrType);
        /** @type {boolean} */ var isIntCase = gluShaderUtil.isDataTypeIntOrIVec(this.m_rgbAttrType);
        /** @type {boolean} */ var isUintCase = gluShaderUtil.isDataTypeUintOrUVec(this.m_rgbAttrType);
        /** @type {boolean} */ var isMatCase = gluShaderUtil.isDataTypeMatrix(this.m_rgbAttrType);
        /** @type {number} */ var typeSize = gluShaderUtil.getDataTypeScalarSize(this.m_rgbAttrType);
        /** @type {boolean} */ var isScalarCase = typeSize == 1;
        /** @type {string} */ var swizzleFirst = isScalarCase ? '' : '.x';
        /** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(this.m_rgbAttrType);

        /** @type {string} */ var floatIntScaleStr = '(' + es3fInstancedRenderingTests.FLOAT_INT_SCALE.toFixed(3) + ')';
        /** @type {string} */ var floatIntBiasStr = '(' + es3fInstancedRenderingTests.FLOAT_INT_BIAS.toFixed(3) + ')';
        /** @type {string} */ var floatUintScaleStr = '(' + es3fInstancedRenderingTests.FLOAT_UINT_SCALE.toFixed(3) + ')';
        /** @type {string} */ var floatUintBiasStr = '(' + es3fInstancedRenderingTests.FLOAT_UINT_BIAS.toFixed(3) + ')';

        DE_ASSERT(isFloatCase || isIntCase || isUintCase || isMatCase);

        // Generate shader.
        // \note For case TYPE_MIXED, vertex position offset and color red component get their values from instance id, while green and blue get their values from instanced attributes.

        /** @type {string} */ var numInstancesStr = this.m_numInstances.toString() + '.0';

        /** @type {string} */ var instanceAttribs = '';
        /** @type {string} */ var posExpression = '';
        /** @type {string} */ var colorRExpression = '';
        /** @type {string} */ var colorGExpression = '';
        /** @type {string} */ var colorBExpression = '';

        if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_INSTANCE_ID || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED) {
            posExpression = 'a_position + vec4(float(gl_InstanceID) * 2.0 / ' + numInstancesStr + ', 0.0, 0.0, 0.0)';
            colorRExpression = 'float(gl_InstanceID)/' + numInstancesStr;

            if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_INSTANCE_ID) {
                colorGExpression = 'float(gl_InstanceID)*2.0/' + numInstancesStr;
                colorBExpression = '1.0 - float(gl_InstanceID)/' + numInstancesStr;
            }
        }

        if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED) {
            if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR) {
                posExpression = 'a_position + vec4(a_instanceOffset';

                DE_ASSERT(es3fInstancedRenderingTests.OFFSET_COMPONENTS >= 1 && es3fInstancedRenderingTests.OFFSET_COMPONENTS <= 4);

                for (var i = 0; i < 4 - es3fInstancedRenderingTests.OFFSET_COMPONENTS; i++)
                    posExpression += ', 0.0';
                posExpression += ')';

                if (isFloatCase)
                    colorRExpression = 'a_instanceR' + swizzleFirst;
                else if (isIntCase)
                    colorRExpression = '(float(a_instanceR' + swizzleFirst + ') - ' + floatIntBiasStr + ') / ' + floatIntScaleStr;
                else if (isUintCase)
                    colorRExpression = '(float(a_instanceR' + swizzleFirst + ') - ' + floatUintBiasStr + ') / ' + floatUintScaleStr;
                else if (isMatCase)
                    colorRExpression = 'a_instanceR[0][0]';
                else
                    DE_ASSERT(false);

                instanceAttribs += 'in highp ' + (es3fInstancedRenderingTests.OFFSET_COMPONENTS == 1 ? 'float' : 'vec' + es3fInstancedRenderingTests.OFFSET_COMPONENTS.toString()) + ' a_instanceOffset;\n';
                instanceAttribs += 'in mediump ' + typeName + ' a_instanceR;\n';
            }

            if (isFloatCase) {
                colorGExpression = 'a_instanceG' + swizzleFirst;
                colorBExpression = 'a_instanceB' + swizzleFirst;
            } else if (isIntCase) {
                colorGExpression = '(float(a_instanceG' + swizzleFirst + ') - ' + floatIntBiasStr + ') / ' + floatIntScaleStr;
                colorBExpression = '(float(a_instanceB' + swizzleFirst + ') - ' + floatIntBiasStr + ') / ' + floatIntScaleStr;
            } else if (isUintCase) {
                colorGExpression = '(float(a_instanceG' + swizzleFirst + ') - ' + floatUintBiasStr + ') / ' + floatUintScaleStr;
                colorBExpression = '(float(a_instanceB' + swizzleFirst + ') - ' + floatUintBiasStr + ') / ' + floatUintScaleStr;
            } else if (isMatCase) {
                colorGExpression = 'a_instanceG[0][0]';
                colorBExpression = 'a_instanceB[0][0]';
            } else
                DE_ASSERT(false);

            instanceAttribs += 'in mediump ' + typeName + ' a_instanceG;\n';
            instanceAttribs += 'in mediump ' + typeName + ' a_instanceB;\n';
        }

        DE_ASSERT(!(posExpression.length == 0));
        DE_ASSERT(!(colorRExpression.length == 0));
        DE_ASSERT(!(colorGExpression.length == 0));
        DE_ASSERT(!(colorBExpression.length == 0));

        /** @type {string} */ var vertShaderSourceStr =
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            instanceAttribs +
            'out mediump vec4 v_color;\n' +
            '\n' +
            'void main()\n' +
            ' {\n' +
            ' gl_Position = ' + posExpression + ';\n' +
            ' v_color.r = ' + colorRExpression + ';\n' +
            ' v_color.g = ' + colorGExpression + ';\n' +
            ' v_color.b = ' + colorBExpression + ';\n' +
            ' v_color.a = 1.0;\n' +
            '}\n';

        /** @type {string} */ var fragShaderSource =
            '#version 300 es\n' +
            'layout(location = 0) out mediump vec4 o_color;\n' +
            'in mediump vec4 v_color;\n' +
            '\n' +
            'void main()\n' +
            ' {\n' +
            ' o_color = v_color;\n' +
            '}\n';

        // Create shader program and log it.

        DE_ASSERT(!this.m_program);
        this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertShaderSourceStr, fragShaderSource));

        //tcu::TestLog& log = this.m_testCtx.getLog();
        //log << *m_program;
        // TODO: bufferedLogToConsole?
        //bufferedLogToConsole(this.m_program);

        assertMsgOptions(this.m_program.isOk(), 'Failed to compile shader', false, true);

        // Vertex shader attributes.

        if (this.m_function == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ELEMENTS_INSTANCED) {
            // Vertex positions. Positions form a vertical bar of width <screen width>/<number of instances>.

            for (var y = 0; y < es3fInstancedRenderingTests.QUAD_GRID_SIZE + 1; y++)
                for (var x = 0; x < es3fInstancedRenderingTests.QUAD_GRID_SIZE + 1; x++) {
                    /** @type {number} */ var fx = -1.0 + x / es3fInstancedRenderingTests.QUAD_GRID_SIZE * 2.0 / this.m_numInstances;
                    /** @type {number} */ var fy = -1.0 + y / es3fInstancedRenderingTests.QUAD_GRID_SIZE * 2.0;

                    this.m_gridVertexPositions.push(fx);
                    this.m_gridVertexPositions.push(fy);
                }

            // Indices.

            for (var y = 0; y < es3fInstancedRenderingTests.QUAD_GRID_SIZE; y++)
                for (var x = 0; x < es3fInstancedRenderingTests.QUAD_GRID_SIZE; x++) {
                    /** @type {number} */ var ndx00 = y * (es3fInstancedRenderingTests.QUAD_GRID_SIZE + 1) + x;
                    /** @type {number} */ var ndx10 = y * (es3fInstancedRenderingTests.QUAD_GRID_SIZE + 1) + x + 1;
                    /** @type {number} */ var ndx01 = (y + 1) * (es3fInstancedRenderingTests.QUAD_GRID_SIZE + 1) + x;
                    /** @type {number} */ var ndx11 = (y + 1) * (es3fInstancedRenderingTests.QUAD_GRID_SIZE + 1) + x + 1;

                    // Lower-left triangle of a quad.
                    this.m_gridIndices.push(ndx00);
                    this.m_gridIndices.push(ndx10);
                    this.m_gridIndices.push(ndx01);

                    // Upper-right triangle of a quad.
                    this.m_gridIndices.push(ndx11);
                    this.m_gridIndices.push(ndx01);
                    this.m_gridIndices.push(ndx10);
                }
        } else {
            DE_ASSERT(this.m_function == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ARRAYS_INSTANCED);

            // Vertex positions. Positions form a vertical bar of width <screen width>/<number of instances>.

            for (var y = 0; y < es3fInstancedRenderingTests.QUAD_GRID_SIZE; y++)
                for (var x = 0; x < es3fInstancedRenderingTests.QUAD_GRID_SIZE; x++) {
                    /** @type {number} */ var fx0 = -1.0 + (x + 0) / es3fInstancedRenderingTests.QUAD_GRID_SIZE * 2.0 / this.m_numInstances;
                    /** @type {number} */ var fx1 = -1.0 + (x + 1) / es3fInstancedRenderingTests.QUAD_GRID_SIZE * 2.0 / this.m_numInstances;
                    /** @type {number} */ var fy0 = -1.0 + (y + 0) / es3fInstancedRenderingTests.QUAD_GRID_SIZE * 2.0;
                    /** @type {number} */ var fy1 = -1.0 + (y + 1) / es3fInstancedRenderingTests.QUAD_GRID_SIZE * 2.0;

                    // Vertices of a quad's lower-left triangle: (fx0, fy0), (fx1, fy0) and (fx0, fy1)
                    this.m_gridVertexPositions.push(fx0);
                    this.m_gridVertexPositions.push(fy0);
                    this.m_gridVertexPositions.push(fx1);
                    this.m_gridVertexPositions.push(fy0);
                    this.m_gridVertexPositions.push(fx0);
                    this.m_gridVertexPositions.push(fy1);

                    // Vertices of a quad's upper-right triangle: (fx1, fy1), (fx0, fy1) and (fx1, fy0)
                    this.m_gridVertexPositions.push(fx1);
                    this.m_gridVertexPositions.push(fy1);
                    this.m_gridVertexPositions.push(fx0);
                    this.m_gridVertexPositions.push(fy1);
                    this.m_gridVertexPositions.push(fx1);
                    this.m_gridVertexPositions.push(fy0);
                }
        }

        // Instanced attributes: position offset and color RGB components.

        if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED) {
            if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR) {
                // Offsets are such that the vertical bars are drawn next to each other.
                for (var i = 0; i < this.m_numInstances; i++) {
                    this.m_instanceOffsets.push(i * 2.0 / this.m_numInstances);

                    DE_ASSERT(es3fInstancedRenderingTests.OFFSET_COMPONENTS >= 1 && es3fInstancedRenderingTests.OFFSET_COMPONENTS <= 4);

                    for (var j = 0; j < es3fInstancedRenderingTests.OFFSET_COMPONENTS - 1; j++)
                        this.m_instanceOffsets.push(0.0);
                }

                /** @type {number} */ var rInstances = Math.floor(this.m_numInstances / es3fInstancedRenderingTests.ATTRIB_DIVISOR_R) + (this.m_numInstances % es3fInstancedRenderingTests.ATTRIB_DIVISOR_R == 0 ? 0 : 1);
                for (var i = 0; i < rInstances; i++) {
                    this.pushVarCompAttrib(this.m_instanceColorR, i / rInstances);

                    for (var j = 0; j < typeSize - 1; j++)
                        this.pushVarCompAttrib(this.m_instanceColorR, 0.0);
                }
            }

            /** @type {number} */ var gInstances = Math.floor(this.m_numInstances / es3fInstancedRenderingTests.ATTRIB_DIVISOR_G) + (this.m_numInstances % es3fInstancedRenderingTests.ATTRIB_DIVISOR_G == 0 ? 0 : 1);
            for (var i = 0; i < gInstances; i++) {
                this.pushVarCompAttrib(this.m_instanceColorG, i * 2.0 / gInstances);

                for (var j = 0; j < typeSize - 1; j++)
                    this.pushVarCompAttrib(this.m_instanceColorG, 0.0);
            }

            /** @type {number} */ var bInstances = Math.floor(this.m_numInstances / es3fInstancedRenderingTests.ATTRIB_DIVISOR_B) + (this.m_numInstances % es3fInstancedRenderingTests.ATTRIB_DIVISOR_B == 0 ? 0 : 1);
            for (var i = 0; i < bInstances; i++) {
                this.pushVarCompAttrib(this.m_instanceColorB, 1.0 - i / bInstances);

                for (var j = 0; j < typeSize - 1; j++)
                    this.pushVarCompAttrib(this.m_instanceColorB, 0.0);
            }
        }
    };

    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.deinit = function() {
        var numVertexAttribArrays = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
        for (var idx = 0; idx < numVertexAttribArrays; idx++) {
            gl.disableVertexAttribArray(idx);
            gl.vertexAttribDivisor(idx, 0);
        }
    };

    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.iterate = function() {
        /** @type {number} */ var width = Math.min(gl.drawingBufferWidth, es3fInstancedRenderingTests.MAX_RENDER_WIDTH);
        /** @type {number} */ var height = Math.min(gl.drawingBufferHeight, es3fInstancedRenderingTests.MAX_RENDER_HEIGHT);

        /** @type {number} */ var xOffsetMax = gl.drawingBufferWidth - width;
        /** @type {number} */ var yOffsetMax = gl.drawingBufferHeight - height;

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));

        /** @type {number} */ var xOffset = rnd.getInt(0, xOffsetMax);
        /** @type {number} */ var yOffset = rnd.getInt(0, yOffsetMax);

        /** @type {tcuSurface.Surface} */ var referenceImg = new tcuSurface.Surface(width, height);
        /** @type {tcuSurface.Surface} */ var resultImg = new tcuSurface.Surface(width, height);

        // Draw result.

        gl.viewport(xOffset, yOffset, width, height);
        gl.clear(gl.COLOR_BUFFER_BIT);

        this.setupAndRender();

        var resImg = resultImg.getAccess();
        var resImgTransferFormat = gluTextureUtil.getTransferFormat(resImg.getFormat());

        gl.readPixels(xOffset, yOffset, resImg.m_width, resImg.m_height, resImgTransferFormat.format, resImgTransferFormat.dataType, resultImg.m_pixels);

        // Compute reference.
        this.computeReference(referenceImg);

        // Compare.

        // Passing referenceImg.getAccess() and resultImg.getAccess() instead of referenceImg and resultImg
    /** @type {boolean} */ var testOk = tcuImageCompare.fuzzyCompare('ComparisonResult', 'Image comparison result', referenceImg.getAccess(), resultImg.getAccess(), 0.05 /*, gluShaderUtil.COMPARE_LOG_RESULT*/);

        assertMsgOptions(testOk, '', true, false);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
    * @param {Array<number>} attrPtr
    * @param {number} location
    * @param {number} divisor
    */
    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.setupVarAttribPointer = function(attrPtr, location, divisor) {
        /** @type {boolean} */ var isFloatCase = gluShaderUtil.isDataTypeFloatOrVec(this.m_rgbAttrType);
        /** @type {boolean} */ var isIntCase = gluShaderUtil.isDataTypeIntOrIVec(this.m_rgbAttrType);
        /** @type {boolean} */ var isUintCase = gluShaderUtil.isDataTypeUintOrUVec(this.m_rgbAttrType);
        /** @type {boolean} */ var isMatCase = gluShaderUtil.isDataTypeMatrix(this.m_rgbAttrType);
        /** @type {number} */ var typeSize = gluShaderUtil.getDataTypeScalarSize(this.m_rgbAttrType);
        /** @type {number} */ var numSlots = isMatCase ? gluShaderUtil.getDataTypeMatrixNumColumns(this.m_rgbAttrType) : 1; // Matrix uses as many attribute slots as it has columns.

        for (var slotNdx = 0; slotNdx < numSlots; slotNdx++) {
            /** @type {number} */ var curLoc = location + slotNdx;

            gl.enableVertexAttribArray(curLoc);
            gl.vertexAttribDivisor(curLoc, divisor);
            var curLocGlBuffer = gl.createBuffer();
            if (isFloatCase) {
                var bufferCurLoc = new Float32Array(attrPtr);
                gl.bindBuffer(gl.ARRAY_BUFFER, curLocGlBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, bufferCurLoc, gl.STATIC_DRAW);

                gl.vertexAttribPointer(curLoc, typeSize, gl.FLOAT, false, 0, 0);
            } else if (isIntCase) {
                var bufferCurLoc = new Int32Array(attrPtr);
                gl.bindBuffer(gl.ARRAY_BUFFER, curLocGlBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, bufferCurLoc, gl.STATIC_DRAW);

                gl.vertexAttribIPointer(curLoc, typeSize, gl.INT, 0, 0);
            } else if (isUintCase) {
                var bufferCurLoc = new Uint32Array(attrPtr);
                gl.bindBuffer(gl.ARRAY_BUFFER, curLocGlBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, bufferCurLoc, gl.STATIC_DRAW);

                gl.vertexAttribIPointer(curLoc, typeSize, gl.UNSIGNED_INT, 0, 0);
            } else if (isMatCase) {
                /** @type {number} */ var numRows = gluShaderUtil.getDataTypeMatrixNumRows(this.m_rgbAttrType);
                /** @type {number} */ var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(this.m_rgbAttrType);

                var bufferCurLoc = new Float32Array(attrPtr);
                gl.bindBuffer(gl.ARRAY_BUFFER, curLocGlBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, bufferCurLoc, gl.STATIC_DRAW);

                gl.vertexAttribPointer(curLoc, numRows, gl.FLOAT, false, numCols * numRows * 4, 0);
            } else
                DE_ASSERT(false);
        }
    };

    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.setupAndRender = function() {
        /** @type {WebGLProgram} */ var program = this.m_program.getProgram();

        gl.useProgram(program);
        // Setup attributes.

        // Position attribute is non-instanced.
        /** @type {number} */ var positionLoc = gl.getAttribLocation(program, 'a_position');
        gl.enableVertexAttribArray(positionLoc);
        var positionBuffer = gl.createBuffer();
        var bufferGridVertexPosition = new Float32Array(this.m_gridVertexPositions);
        gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, bufferGridVertexPosition, gl.STATIC_DRAW);
        gl.vertexAttribPointer(positionLoc, 2, gl.FLOAT, false, 0, 0);

        if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED) {
            if (this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR) {
                // Position offset attribute is instanced with separate offset for every instance.
                /** @type {number} */ var offsetLoc = gl.getAttribLocation(program, 'a_instanceOffset');
                gl.enableVertexAttribArray(offsetLoc);
                gl.vertexAttribDivisor(offsetLoc, 1);

                var offsetLocGlBuffer = gl.createBuffer();
                var bufferOffsetLoc = new Float32Array(this.m_instanceOffsets);
                gl.bindBuffer(gl.ARRAY_BUFFER, offsetLocGlBuffer);
                gl.bufferData(gl.ARRAY_BUFFER, bufferOffsetLoc, gl.STATIC_DRAW);

                gl.vertexAttribPointer(offsetLoc, es3fInstancedRenderingTests.OFFSET_COMPONENTS, gl.FLOAT, false, 0, 0);

                /** @type {number} */ var rLoc = gl.getAttribLocation(program, 'a_instanceR');
                this.setupVarAttribPointer(this.m_instanceColorR, rLoc, es3fInstancedRenderingTests.ATTRIB_DIVISOR_R);
            }

            /** @type {number} */ var gLoc = gl.getAttribLocation(program, 'a_instanceG');
            this.setupVarAttribPointer(this.m_instanceColorG, gLoc, es3fInstancedRenderingTests.ATTRIB_DIVISOR_G);

            /** @type {number} */ var bLoc = gl.getAttribLocation(program, 'a_instanceB');
            this.setupVarAttribPointer(this.m_instanceColorB, bLoc, es3fInstancedRenderingTests.ATTRIB_DIVISOR_B);
        }

        // Draw using appropriate function.

        if (this.m_function == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ARRAYS_INSTANCED) {
            /** @type {number} */ var numPositionComponents = 2;
            gl.drawArraysInstanced(gl.TRIANGLES, 0, Math.floor(this.m_gridVertexPositions.length / numPositionComponents), this.m_numInstances);
        } else {
            var gridIndicesGLBuffer = gl.createBuffer();
            var bufferGridIndices = new Uint16Array(this.m_gridIndices);
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, gridIndicesGLBuffer);
            gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, bufferGridIndices, gl.STATIC_DRAW);

            gl.drawElementsInstanced(gl.TRIANGLES, this.m_gridIndices.length, gl.UNSIGNED_SHORT, 0, this.m_numInstances);
        }
        gl.useProgram(null);
    };

    /**
    * @param {tcuSurface.Surface} dst
    */
    es3fInstancedRenderingTests.InstancedRenderingCase.prototype.computeReference = function(dst) {
        /** @type {number} */ var wid = dst.getWidth();
        /** @type {number} */ var hei = dst.getHeight();

        // Draw a rectangle (vertical bar) for each instance.

        for (var instanceNdx = 0; instanceNdx < this.m_numInstances; instanceNdx++) {
            /** @type {number} */ var xStart = Math.floor(instanceNdx * wid / this.m_numInstances);
            /** @type {number} */ var xEnd = Math.floor((instanceNdx + 1) * wid / this.m_numInstances);

            // Emulate attribute divisors if that is the case.

            /** @type {number} */ var clrNdxR = this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR ? Math.floor(instanceNdx / es3fInstancedRenderingTests.ATTRIB_DIVISOR_R) : instanceNdx;
            /** @type {number} */ var clrNdxG = this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED ? Math.floor(instanceNdx / es3fInstancedRenderingTests.ATTRIB_DIVISOR_G) : instanceNdx;
            /** @type {number} */ var clrNdxB = this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED ? Math.floor(instanceNdx / es3fInstancedRenderingTests.ATTRIB_DIVISOR_B) : instanceNdx;

            /** @type {number} */ var rInstances = this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR ? Math.floor(this.m_numInstances / es3fInstancedRenderingTests.ATTRIB_DIVISOR_R) + (this.m_numInstances % es3fInstancedRenderingTests.ATTRIB_DIVISOR_R == 0 ? 0 : 1) : this.m_numInstances;
            /** @type {number} */ var gInstances = this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED ? Math.floor(this.m_numInstances / es3fInstancedRenderingTests.ATTRIB_DIVISOR_G) + (this.m_numInstances % es3fInstancedRenderingTests.ATTRIB_DIVISOR_G == 0 ? 0 : 1) : this.m_numInstances;
            /** @type {number} */ var bInstances = this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR || this.m_instancingType == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED ? Math.floor(this.m_numInstances / es3fInstancedRenderingTests.ATTRIB_DIVISOR_B) + (this.m_numInstances % es3fInstancedRenderingTests.ATTRIB_DIVISOR_B == 0 ? 0 : 1) : this.m_numInstances;

            // Calculate colors.

            /** @type {number} */ var r = clrNdxR / rInstances;
            /** @type {number} */ var g = clrNdxG * 2.0 / gInstances;
            /** @type {number} */ var b = 1.0 - clrNdxB / bInstances;

            // Convert to integer and back if shader inputs are integers.

            if (gluShaderUtil.isDataTypeIntOrIVec(this.m_rgbAttrType)) {
                /** @type {number} */var intR = (r * es3fInstancedRenderingTests.FLOAT_INT_SCALE + es3fInstancedRenderingTests.FLOAT_INT_BIAS);
                /** @type {number} */var intG = (g * es3fInstancedRenderingTests.FLOAT_INT_SCALE + es3fInstancedRenderingTests.FLOAT_INT_BIAS);
                /** @type {number} */var intB = (b * es3fInstancedRenderingTests.FLOAT_INT_SCALE + es3fInstancedRenderingTests.FLOAT_INT_BIAS);
                r = (intR - es3fInstancedRenderingTests.FLOAT_INT_BIAS) / es3fInstancedRenderingTests.FLOAT_INT_SCALE;
                g = (intG - es3fInstancedRenderingTests.FLOAT_INT_BIAS) / es3fInstancedRenderingTests.FLOAT_INT_SCALE;
                b = (intB - es3fInstancedRenderingTests.FLOAT_INT_BIAS) / es3fInstancedRenderingTests.FLOAT_INT_SCALE;
            } else if (gluShaderUtil.isDataTypeUintOrUVec(this.m_rgbAttrType)) {
                /** @type {number} */var uintR = (r * es3fInstancedRenderingTests.FLOAT_UINT_SCALE + es3fInstancedRenderingTests.FLOAT_UINT_BIAS);
                /** @type {number} */var uintG = (g * es3fInstancedRenderingTests.FLOAT_UINT_SCALE + es3fInstancedRenderingTests.FLOAT_UINT_BIAS);
                /** @type {number} */var uintB = (b * es3fInstancedRenderingTests.FLOAT_UINT_SCALE + es3fInstancedRenderingTests.FLOAT_UINT_BIAS);
                r = (uintR - es3fInstancedRenderingTests.FLOAT_UINT_BIAS) / es3fInstancedRenderingTests.FLOAT_UINT_SCALE;
                g = (uintG - es3fInstancedRenderingTests.FLOAT_UINT_BIAS) / es3fInstancedRenderingTests.FLOAT_UINT_SCALE;
                b = (uintB - es3fInstancedRenderingTests.FLOAT_UINT_BIAS) / es3fInstancedRenderingTests.FLOAT_UINT_SCALE;
            }

            // Convert from float to unorm8.
            var color = deMath.add(deMath.scale([r, g, b, 1.0], 255), [0.5, 0.5, 0.5, 0.5]);
            color = deMath.clampVector(color, 0, 255);

            // Draw rectangle.
            for (var y = 0; y < hei; y++)
                for (var x = xStart; x < xEnd; x++)
                    dst.setPixel(x, y, color);
        }
    };

    es3fInstancedRenderingTests.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
    /** @type {Array<number>} */ var instanceCounts = [1, 2, 4, 20];

        for (var _function in es3fInstancedRenderingTests.DrawFunction) {
            /** @type {?string} */ var functionName =
                                       es3fInstancedRenderingTests.DrawFunction[_function] == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ARRAYS_INSTANCED ? 'draw_arrays_instanced' :
                                       es3fInstancedRenderingTests.DrawFunction[_function] == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ELEMENTS_INSTANCED ? 'draw_elements_instanced' :
                                       null;

            /** @type {?string} */ var functionDesc =
                                       es3fInstancedRenderingTests.DrawFunction[_function] == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ARRAYS_INSTANCED ? 'Use glDrawArraysInstanced()' :
                                       es3fInstancedRenderingTests.DrawFunction[_function] == es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ELEMENTS_INSTANCED ? 'Use glDrawElementsInstanced()' :
                                       null;

            DE_ASSERT(functionName != null);
            DE_ASSERT(functionDesc != null);

            /** @type {tcuTestCase.DeqpTest} */ var functionGroup = tcuTestCase.newTest(functionName, functionDesc);
            testGroup.addChild(functionGroup);

            for (var instancingType in es3fInstancedRenderingTests.InstancingType) {
                /** @type {?string} */ var instancingTypeName =
                                                 es3fInstancedRenderingTests.InstancingType[instancingType] == es3fInstancedRenderingTests.InstancingType.TYPE_INSTANCE_ID ? 'instance_id' :
                                                 es3fInstancedRenderingTests.InstancingType[instancingType] == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR ? 'attribute_divisor' :
                                                 es3fInstancedRenderingTests.InstancingType[instancingType] == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED ? 'mixed' :
                                                 null;

                /** @type {?string} */ var instancingTypeDesc =
                                                 es3fInstancedRenderingTests.InstancingType[instancingType] == es3fInstancedRenderingTests.InstancingType.TYPE_INSTANCE_ID ? 'Use gl_InstanceID for instancing' :
                                                 es3fInstancedRenderingTests.InstancingType[instancingType] == es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR ? 'Use vertex attribute divisors for instancing' :
                                                 es3fInstancedRenderingTests.InstancingType[instancingType] == es3fInstancedRenderingTests.InstancingType.TYPE_MIXED ? 'Use both gl_InstanceID and vertex attribute divisors for instancing' :
                                                 null;

                DE_ASSERT(instancingTypeName != null);
                DE_ASSERT(instancingTypeDesc != null);

                /** @type {tcuTestCase.DeqpTest} */
                var instancingTypeGroup = tcuTestCase.newTest(instancingTypeName, instancingTypeDesc);

                functionGroup.addChild(instancingTypeGroup);

                for (var countNdx in instanceCounts) {
                    /** @type {string} */ var countName = instanceCounts[countNdx].toString() + '_instances';
                    instancingTypeGroup.addChild(new es3fInstancedRenderingTests.InstancedRenderingCase(countName,
                                                                             '',
                                                                             es3fInstancedRenderingTests.DrawFunction[_function],
                                                                             es3fInstancedRenderingTests.InstancingType[instancingType],
                                                                             gluShaderUtil.DataType.FLOAT,
                                                                             instanceCounts[countNdx]));
                }
            }
        }

        /** @type {Array<gluShaderUtil.DataType>} */ var s_testTypes =
        [
            gluShaderUtil.DataType.FLOAT,
            gluShaderUtil.DataType.FLOAT_VEC2,
            gluShaderUtil.DataType.FLOAT_VEC3,
            gluShaderUtil.DataType.FLOAT_VEC4,
            gluShaderUtil.DataType.FLOAT_MAT2,
            gluShaderUtil.DataType.FLOAT_MAT2X3,
            gluShaderUtil.DataType.FLOAT_MAT2X4,
            gluShaderUtil.DataType.FLOAT_MAT3X2,
            gluShaderUtil.DataType.FLOAT_MAT3,
            gluShaderUtil.DataType.FLOAT_MAT3X4,
            gluShaderUtil.DataType.FLOAT_MAT4X2,
            gluShaderUtil.DataType.FLOAT_MAT4X3,
            gluShaderUtil.DataType.FLOAT_MAT4,

            gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT_VEC2,
            gluShaderUtil.DataType.INT_VEC3,
            gluShaderUtil.DataType.INT_VEC4,

            gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT_VEC2,
            gluShaderUtil.DataType.UINT_VEC3,
            gluShaderUtil.DataType.UINT_VEC4
        ];

        /** @type {number} */ var typeTestNumInstances = 4;

        /** @type {tcuTestCase.DeqpTest} */ var typesGroup = tcuTestCase.newTest('types', 'Tests for instanced attributes of particular data types');

        testGroup.addChild(typesGroup);

        for (var typeNdx in s_testTypes) {
            /** @type {gluShaderUtil.DataType} */ var type = s_testTypes[typeNdx];
            typesGroup.addChild(new es3fInstancedRenderingTests.InstancedRenderingCase(gluShaderUtil.getDataTypeName(type), '',
                                                            es3fInstancedRenderingTests.DrawFunction.FUNCTION_DRAW_ARRAYS_INSTANCED,
                                                            es3fInstancedRenderingTests.InstancingType.TYPE_ATTRIB_DIVISOR,
                                                            type,
                                                            typeTestNumInstances));
        }
    };

    es3fInstancedRenderingTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'instanced_rendering';
        var testDescription = 'Instanced Rendering Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.setRoot(tcuTestCase.newTest(testName, testDescription, null));

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fInstancedRenderingTests.init();
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fInstancedRenderingTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

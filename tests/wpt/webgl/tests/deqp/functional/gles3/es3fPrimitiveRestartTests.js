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
goog.provide('functional.gles3.es3fPrimitiveRestartTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

var es3fPrimitiveRestartTests = functional.gles3.es3fPrimitiveRestartTests;
var tcuTestCase = framework.common.tcuTestCase;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuSurface = framework.common.tcuSurface;
var deMath = framework.delibs.debase.deMath;
var deRandom = framework.delibs.debase.deRandom;
var deString = framework.delibs.debase.deString;
var tcuImageCompare = framework.common.tcuImageCompare;
var gluTextureUtil = framework.opengl.gluTextureUtil;

    /** @type {WebGL2RenderingContext} */ var gl;
    /** @const @type {number} */ es3fPrimitiveRestartTests.MAX_RENDER_WIDTH = 256;
    /** @const @type {number} */ es3fPrimitiveRestartTests.MAX_RENDER_HEIGHT = 256;

    /** @const @type {number} */ es3fPrimitiveRestartTests.MAX_UNSIGNED_BYTE = 255;
    /** @const @type {number} */ es3fPrimitiveRestartTests.MAX_UNSIGNED_SHORT = 65535;
    /** @const @type {number} */ es3fPrimitiveRestartTests.MAX_UNSIGNED_INT = 4294967295;

    /** @const @type {number} */ es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_BYTE = es3fPrimitiveRestartTests.MAX_UNSIGNED_BYTE;
    /** @const @type {number} */ es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_SHORT = es3fPrimitiveRestartTests.MAX_UNSIGNED_SHORT;
    /** @const @type {number} */ es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_INT = es3fPrimitiveRestartTests.MAX_UNSIGNED_INT;

    var DE_ASSERT = function(expression) {
        if (!expression) throw new Error('Assert failed');
    };

    /**
     * @enum
     */
    es3fPrimitiveRestartTests.PrimitiveType = {
        PRIMITIVE_POINTS: 0,
        PRIMITIVE_LINE_STRIP: 1,
        PRIMITIVE_LINE_LOOP: 2,
        PRIMITIVE_LINES: 3,
        PRIMITIVE_TRIANGLE_STRIP: 4,
        PRIMITIVE_TRIANGLE_FAN: 5,
        PRIMITIVE_TRIANGLES: 6
    };

    /**
     * @enum
     */
    es3fPrimitiveRestartTests.IndexType = {
        INDEX_UNSIGNED_BYTE: 0,
        INDEX_UNSIGNED_SHORT: 1,
        INDEX_UNSIGNED_INT: 2
    };

    /**
     * @enum
     */
    es3fPrimitiveRestartTests.DrawFunction = {
        FUNCTION_DRAW_ELEMENTS: 0,
        FUNCTION_DRAW_ELEMENTS_INSTANCED: 1,
        FUNCTION_DRAW_RANGE_ELEMENTS: 2
    };

    /**
    * es3fPrimitiveRestartTests.PrimitiveRestartCase class, inherits from TestCase class
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    * @param {?string} name
    * @param {string} description
    * @param {es3fPrimitiveRestartTests.PrimitiveType} primType
    * @param {es3fPrimitiveRestartTests.IndexType} indexType
    * @param {es3fPrimitiveRestartTests.DrawFunction} _function
    * @param {boolean} beginWithRestart
    * @param {boolean} endWithRestart
    * @param {boolean} duplicateRestarts
    */
    es3fPrimitiveRestartTests.PrimitiveRestartCase = function(name, description, primType, indexType, _function, beginWithRestart, endWithRestart, duplicateRestarts) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {es3fPrimitiveRestartTests.PrimitiveType} */ this.m_primType = primType;
        /** @type {es3fPrimitiveRestartTests.IndexType} */ this.m_indexType = indexType;
        /** @type {es3fPrimitiveRestartTests.DrawFunction} */ this.m_function = _function;
        /** @type {boolean} */ this.m_beginWithRestart = beginWithRestart; // Whether there will be restart indices at the beginning of the index array.
        /** @type {boolean} */ this.m_endWithRestart = endWithRestart; // Whether there will be restart indices at the end of the index array.
        /** @type {boolean} */ this.m_duplicateRestarts = duplicateRestarts; // Whether two consecutive restarts are used instead of one.
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;

        // \note Only one of the following index vectors is used (according to m_indexType).
        /** @type {Array<number>} */ this.m_indicesUB = []; //deUint8
        /** @type {Array<number>} */ this.m_indicesUS = []; //deUint16
        /** @type {Array<number>} */ this.m_indicesUI = []; //deUint32

        /** @type {Array<number>} */ this.m_positions = [];
    };

    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.constructor = es3fPrimitiveRestartTests.PrimitiveRestartCase;

    /**
    * Draw with the appropriate GLES3 draw function.
    * @param {number} startNdx
    * @param {number} count
    */
    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.draw = function(startNdx, count) {
        /** @type {number} */ var primTypeGL;

        switch (this.m_primType) {
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_POINTS:
                primTypeGL = gl.POINTS;
                break;
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINE_STRIP:
                primTypeGL = gl.LINE_STRIP;
                break;
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINE_LOOP:
                primTypeGL = gl.LINE_LOOP;
                break;
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINES:
                primTypeGL = gl.LINES;
                break;
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLE_STRIP:
                primTypeGL = gl.TRIANGLE_STRIP;
                break;
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLE_FAN:
                primTypeGL = gl.TRIANGLE_FAN;
                break;
            case es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLES:
                primTypeGL = gl.TRIANGLES;
                break;
            default:
                DE_ASSERT(false);
                primTypeGL = 0;
        }

        /** @type {number} */ var indexTypeGL;

        switch (this.m_indexType) {
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE:
                indexTypeGL = gl.UNSIGNED_BYTE;
                break;
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT:
                indexTypeGL = gl.UNSIGNED_SHORT;
                break;
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT:
                indexTypeGL = gl.UNSIGNED_INT;
                break;
            default:
                DE_ASSERT(false);
                indexTypeGL = 0;
        }

        /** @type {number} */ var restartIndex = this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_BYTE :
                                                   this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_SHORT :
                                                   this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_INT :
                                                   0;

        DE_ASSERT(restartIndex != 0);

        var indexGLBuffer = gl.createBuffer();
        var bufferIndex = this.getIndexPtr(startNdx);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexGLBuffer);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, bufferIndex, gl.STATIC_DRAW);

        if (this.m_function == es3fPrimitiveRestartTests.DrawFunction.FUNCTION_DRAW_ELEMENTS) {
            gl.drawElements(primTypeGL, count, indexTypeGL, 0);
        } else if (this.m_function == es3fPrimitiveRestartTests.DrawFunction.FUNCTION_DRAW_ELEMENTS_INSTANCED) {
            gl.drawElementsInstanced(primTypeGL, count, indexTypeGL, 0, 1);
        } else {
            DE_ASSERT(this.m_function == es3fPrimitiveRestartTests.DrawFunction.FUNCTION_DRAW_RANGE_ELEMENTS);

            // Find the largest non-restart index in the index array (for glDrawRangeElements() end parameter).

            /** @type {number} */ var max = 0;

            /** @type {number} */ var numIndices = this.getNumIndices();
            for (var i = 0; i < numIndices; i++) {
                /** @type {number} */ var index = this.getIndex(i);
                if (index != restartIndex && index > max)
                    max = index;
            }
            //TODO: drawRangeElements -> check getIndexPtr usage
            gl.drawRangeElements(primTypeGL, 0, max, count, indexTypeGL, 0);
        }
    };

    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.renderWithRestart = function() {
        // Primitive Restart is always on in WebGL2
        //gl.enable(gl.PRIMITIVE_RESTART_FIXED_INDEX);

        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        this.draw(0, this.getNumIndices());
    };

    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.renderWithoutRestart = function() {
        /** @type {number} */ var restartIndex = this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_BYTE :
                                                 this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_SHORT :
                                                 this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_INT :
                                                 0;

        DE_ASSERT(restartIndex != 0);
        // Primitive Restart is always on in WebGL2
        //gl.disable(gl.PRIMITIVE_RESTART_FIXED_INDEX);

        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        // Draw, emulating primitive restart.

        /** @type {number} */ var numIndices = this.getNumIndices();

        DE_ASSERT(numIndices >= 0);

        /** @type {number} */ var indexArrayStartNdx = 0; // Keep track of the draw start index - first index after a primitive restart, or initially the first index altogether.

        for (var indexArrayNdx = 0; indexArrayNdx <= numIndices; indexArrayNdx++) { // \note Goes one "too far" in order to detect end of array as well.
            if (indexArrayNdx >= numIndices || this.getIndex(indexArrayNdx) == restartIndex) {// \note Handle end of array the same way as a restart index encounter.
                if (indexArrayStartNdx < numIndices) {
                    // Draw from index indexArrayStartNdx to index indexArrayNdx-1 .

                    this.draw(indexArrayStartNdx, indexArrayNdx - indexArrayStartNdx);
                }

                indexArrayStartNdx = indexArrayNdx + 1; // Next draw starts just after this restart index.
            }
        }
    };

    /**
    * @param {number} index
    */
    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.addIndex = function(index) {
        if (this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE) {
            DE_ASSERT(deMath.deInRange32(index, 0, es3fPrimitiveRestartTests.MAX_UNSIGNED_BYTE));
            this.m_indicesUB.push(index); // deUint8
        } else if (this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT) {
            DE_ASSERT(deMath.deInRange32(index, 0, es3fPrimitiveRestartTests.MAX_UNSIGNED_SHORT));
            this.m_indicesUS.push(index); // deUint16
        } else if (this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT) {
            DE_ASSERT(deMath.deInRange32(index, 0, es3fPrimitiveRestartTests.MAX_UNSIGNED_INT));
            this.m_indicesUI.push(index); // // deUint32
        } else
            DE_ASSERT(false);
    };

    /**
    * @param {number} indexNdx
    * @return {number}
    */
    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.getIndex = function(indexNdx) {
        switch (this.m_indexType) {
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE:
                return this.m_indicesUB[indexNdx]; //deUint32
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT:
                return this.m_indicesUS[indexNdx]; //deUint32
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT:
                return this.m_indicesUI[indexNdx];
            default:
                DE_ASSERT(false);
                return 0;
        }
    };

    /**
    * @return {number}
    */
    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.getNumIndices = function() {
        switch (this.m_indexType) {
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE:
                return this.m_indicesUB.length;
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT:
                return this.m_indicesUS.length;
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT:
                return this.m_indicesUI.length;
            default:
                DE_ASSERT(false);
                return 0;
        }
    };

    /**
    * Pointer to the index value at index indexNdx.
    * @param {number} indexNdx
    * @return {Uint8Array|Uint16Array|Uint32Array}
    */
    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.getIndexPtr = function(indexNdx) {
        //TODO: implement
        switch (this.m_indexType) {
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE:
                return new Uint8Array(this.m_indicesUB).subarray(indexNdx);
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT:
                return new Uint16Array(this.m_indicesUS).subarray(indexNdx);
            case es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT:
                return new Uint32Array(this.m_indicesUI).subarray(indexNdx);
            default:
                DE_ASSERT(false);
                return null;
        }
    };

    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.init = function() {
        // Clear errors from previous tests
        gl.getError();

        // Create shader program.

        /** @type {string} */ var vertShaderSource =
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            '\n' +
            'void main()\n' +
            ' {\n' +
            ' gl_Position = a_position;\n' +
            '}\n';

            /** @type {string} */ var fragShaderSource =
            '#version 300 es\n' +
            'layout(location = 0) out mediump vec4 o_color;\n' +
            '\n' +
            'void main()\n' +
            ' {\n' +
            ' o_color = vec4(1.0f);\n' +
            '}\n';

        DE_ASSERT(!this.m_program);

        this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertShaderSource, fragShaderSource));

        if (!this.m_program.isOk()) {
            //m_testCtx.getLog() << *this.m_program;
            testFailedOptions('Failed to compile shader', true);
        }

        /** @type {number} */ var restartIndex = this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_BYTE :
                                                 this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_SHORT :
                                                 this.m_indexType == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT ? es3fPrimitiveRestartTests.RESTART_INDEX_UNSIGNED_INT :
                                                 0;

        DE_ASSERT(restartIndex != 0);

        DE_ASSERT(this.getNumIndices() == 0);

        // If testing a case with restart at beginning, add it there.
        if (this.m_beginWithRestart) {
            this.addIndex(restartIndex);
            if (this.m_duplicateRestarts)
                this.addIndex(restartIndex);
        }

        // Generate vertex positions and indices depending on primitive type.
        // \note At this point, restarts shall not be added to the start or the end of the index vector. Those are special cases, and are done above and after the following if-else chain, respectively.
        /** @type {number} */ var curIndex;
        /** @type {number} */ var numRows;
        /** @type {number} */ var numCols;
        /** @type {number} */ var fx;
        /** @type {number} */ var fy;
        /** @type {number} */ var centerY;
        /** @type {number} */ var centerX;
        /** @type {number} */ var numVertices;
        /** @type {number} */ var numArcVertices;
        /** @type {number} */ var numStrips;

        if (this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_POINTS) {
            // Generate rows with different numbers of points.

            curIndex = 0;
            numRows = 20;

            for (var row = 0; row < numRows; row++) {
                for (var col = 0; col < row + 1; col++) {
                    fx = -1.0 + 2.0 * (col + 0.5) / numRows;
                    fy = -1.0 + 2.0 * (row + 0.5) / numRows;

                    this.m_positions.push(fx);
                    this.m_positions.push(fy);

                    this.addIndex(curIndex++);
                }

                if (row < numRows - 1) { // Add a restart after all but last row.
                    this.addIndex(restartIndex);
                    if (this.m_duplicateRestarts)
                        this.addIndex(restartIndex);
                }
            }
        } else if (this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINE_STRIP || this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINE_LOOP || this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINES) {
            // Generate a numRows x numCols arrangement of line polygons of different vertex counts.

            curIndex = 0;
            numRows = 4;
            numCols = 4;

            for (var row = 0; row < numRows; row++) {
                centerY = -1.0 + 2.0 * (row + 0.5) / numRows;

                for (var col = 0; col < numCols; col++) {
                    centerX = -1.0 + 2.0 * (col + 0.5) / numCols;
                    numVertices = row * numCols + col + 1;

                    for (var i = 0; i < numVertices; i++) {
                        fx = centerX + 0.9 * Math.cos(i * 2.0 * Math.PI / numVertices) / numCols;
                        fy = centerY + 0.9 * Math.sin(i * 2.0 * Math.PI / numVertices) / numRows;

                        this.m_positions.push(fx);
                        this.m_positions.push(fy);

                        this.addIndex(curIndex++);
                    }

                    if (col < numCols - 1 || row < numRows - 1) {// Add a restart after all but last polygon.
                        this.addIndex(restartIndex);
                        if (this.m_duplicateRestarts)
                            this.addIndex(restartIndex);
                    }
                }
            }
        } else if (this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLE_STRIP) {
            // Generate a number of horizontal triangle strips of different lengths.

            curIndex = 0;
            numStrips = 20;

            for (var stripNdx = 0; stripNdx < numStrips; stripNdx++) {
                numVertices = stripNdx + 1;

                for (var i = 0; i < numVertices; i++) {
                    fx = -0.9 + 1.8 * (i / 2 * 2) / numStrips;
                    fy = -0.9 + 1.8 * (stripNdx + (i % 2 == 0 ? 0.0 : 0.8)) / numStrips;

                    this.m_positions.push(fx);
                    this.m_positions.push(fy);

                    this.addIndex(curIndex++);
                }

                if (stripNdx < numStrips - 1) { // Add a restart after all but last strip.
                    this.addIndex(restartIndex);
                    if (this.m_duplicateRestarts)
                        this.addIndex(restartIndex);
                }
            }
        } else if (this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLE_FAN) {
            // Generate a numRows x numCols arrangement of triangle fan polygons of different vertex counts.

            curIndex = 0;
            numRows = 4;
            numCols = 4;

            for (var row = 0; row < numRows; row++) {
                centerY = -1.0 + 2.0 * (row + 0.5) / numRows;

                for (var col = 0; col < numCols; col++) {
                    centerX = -1.0 + 2.0 * (col + 0.5) / numCols;
                    numArcVertices = row * numCols + col;

                    this.m_positions.push(centerX);
                    this.m_positions.push(centerY);

                    this.addIndex(curIndex++);

                    for (var i = 0; i < numArcVertices; i++) {
                        fx = centerX + 0.9 * Math.cos(i * 2.0 * Math.PI / numArcVertices) / numCols;
                        fy = centerY + 0.9 * Math.sin(i * 2.0 * Math.PI / numArcVertices) / numRows;

                        this.m_positions.push(fx);
                        this.m_positions.push(fy);

                        this.addIndex(curIndex++);
                    }

                    if (col < numCols - 1 || row < numRows - 1) { // Add a restart after all but last polygon.
                        this.addIndex(restartIndex);
                        if (this.m_duplicateRestarts)
                            this.addIndex(restartIndex);
                    }
                }
            }
        } else if (this.m_primType == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLES) {
            // Generate a number of rows with (potentially incomplete) triangles.

            curIndex = 0;
            numRows = 3 * 7;

            for (var rowNdx = 0; rowNdx < numRows; rowNdx++) {
                numVertices = rowNdx + 1;

                for (var i = 0; i < numVertices; i++) {
                    fx = -0.9 + 1.8 * ((i / 3) + (i % 3 == 2 ? 0.8 : 0.0)) * 3 / numRows;
                    fy = -0.9 + 1.8 * (rowNdx + (i % 3 == 0 ? 0.0 : 0.8)) / numRows;

                    this.m_positions.push(fx);
                    this.m_positions.push(fy);

                    this.addIndex(curIndex++);
                }

                if (rowNdx < numRows - 1) { // Add a restart after all but last row.
                    this.addIndex(restartIndex);
                    if (this.m_duplicateRestarts)
                        this.addIndex(restartIndex);
                }
            }
        } else
            DE_ASSERT(false);

        // If testing a case with restart at end, add it there.
        if (this.m_endWithRestart) {
            this.addIndex(restartIndex);
            if (this.m_duplicateRestarts)
                this.addIndex(restartIndex);
        }

        // Special case assertions.

        /** @type {number} */ var numIndices = this.getNumIndices();

        DE_ASSERT(numIndices > 0);
        DE_ASSERT(this.m_beginWithRestart || this.getIndex(0) != restartIndex); // We don't want restarts at beginning unless the case is a special case.
        DE_ASSERT(this.m_endWithRestart || this.getIndex(numIndices - 1) != restartIndex); // We don't want restarts at end unless the case is a special case.

        if (!this.m_duplicateRestarts)
            for (var i = 1; i < numIndices; i++)
                DE_ASSERT(this.getIndex(i) != restartIndex || this.getIndex(i - 1) != restartIndex); // We don't want duplicate restarts unless the case is a special case.

    };

    es3fPrimitiveRestartTests.PrimitiveRestartCase.prototype.iterate = function() {
        /** @type {number} */ var width = Math.min(gl.drawingBufferWidth, es3fPrimitiveRestartTests.MAX_RENDER_WIDTH);
        /** @type {number} */ var height = Math.min(gl.drawingBufferHeight, es3fPrimitiveRestartTests.MAX_RENDER_HEIGHT);

        /** @type {number} */ var xOffsetMax = gl.drawingBufferWidth - width;
        /** @type {number} */ var yOffsetMax = gl.drawingBufferHeight - height;

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));

        /** @type {number} */ var xOffset = rnd.getInt(0, xOffsetMax);
        /** @type {number} */ var yOffset = rnd.getInt(0, yOffsetMax);
        /** @type {tcuSurface.Surface} */ var referenceImg = new tcuSurface.Surface(width, height);
        /** @type {tcuSurface.Surface} */ var resultImg = new tcuSurface.Surface(width, height);

        gl.viewport(xOffset, yOffset, width, height);
        gl.clearColor(0.0, 0.0, 0.0, 1.0);

        var program = this.m_program.getProgram();
        gl.useProgram(program);

        // Setup position attribute.

        /** @type {number} */ var loc = gl.getAttribLocation(program, 'a_position');
        gl.enableVertexAttribArray(loc);

        var locGlBuffer = gl.createBuffer();
        var bufferLoc = new Float32Array(this.m_positions);
        gl.bindBuffer(gl.ARRAY_BUFFER, locGlBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, bufferLoc, gl.STATIC_DRAW);
        gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);

        // Render result.
        this.renderWithRestart();
        var resImg = resultImg.getAccess();
        var resImgTransferFormat = gluTextureUtil.getTransferFormat(resImg.getFormat());
        gl.readPixels(xOffset, yOffset, resImg.m_width, resImg.m_height, resImgTransferFormat.format, resImgTransferFormat.dataType, resultImg.m_pixels);

        // Render reference (same scene as the real deal, but emulate primitive restart without actually using it).
        this.renderWithoutRestart();

        var refImg = referenceImg.getAccess();
        var refImgTransferFormat = gluTextureUtil.getTransferFormat(refImg.getFormat());

        gl.readPixels(xOffset, yOffset, refImg.m_width, refImg.m_height, refImgTransferFormat.format, refImgTransferFormat.dataType, referenceImg.m_pixels);

        // Compare.
        /** @type {boolean} */ var testOk = tcuImageCompare.pixelThresholdCompare('ComparisonResult', 'Image comparison result', referenceImg, resultImg, [0, 0, 0, 0]);

        assertMsgOptions(testOk, '', true, false);
        gl.useProgram(null);

        return tcuTestCase.IterateResult.STOP;
    };

    es3fPrimitiveRestartTests.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        for (var isRestartBeginCaseI = 0; isRestartBeginCaseI <= 1; isRestartBeginCaseI++) {
            for (var isRestartEndCaseI = 0; isRestartEndCaseI <= 1; isRestartEndCaseI++) {
                for (var isDuplicateRestartCaseI = 0; isDuplicateRestartCaseI <= 1; isDuplicateRestartCaseI++) {
                    /** @type {boolean} */ var isRestartBeginCase = isRestartBeginCaseI != 0;
                    /** @type {boolean} */ var isRestartEndCase = isRestartEndCaseI != 0;
                    /** @type {boolean} */ var isDuplicateRestartCase = isDuplicateRestartCaseI != 0;

                    /** @type {string} */ var specialCaseGroupName = '';

                    if (isRestartBeginCase) specialCaseGroupName = 'begin_restart';
                    if (isRestartEndCase) specialCaseGroupName += (deString.deIsStringEmpty(specialCaseGroupName) ? '' : '_') + 'end_restart';
                    if (isDuplicateRestartCase) specialCaseGroupName += (deString.deIsStringEmpty(specialCaseGroupName) ? '' : '_') + 'duplicate_restarts';

                    if (deString.deIsStringEmpty(specialCaseGroupName))
                        specialCaseGroupName = 'basic';

                    /** @type {tcuTestCase.DeqpTest} */ var specialCaseGroup = tcuTestCase.newTest(specialCaseGroupName, '');
                    testGroup.addChild(specialCaseGroup);

                    for (var primType in es3fPrimitiveRestartTests.PrimitiveType) {
                        /** @type {string} */ var primTypeName = es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_POINTS ? 'points' :
                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINE_STRIP ? 'line_strip' :
                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINE_LOOP ? 'line_loop' :
                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_LINES ? 'lines' :
                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLE_STRIP ? 'triangle_strip' :
                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLE_FAN ? 'triangle_fan' :
                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType] == es3fPrimitiveRestartTests.PrimitiveType.PRIMITIVE_TRIANGLES ? 'triangles' :
                                                                 '';

                        DE_ASSERT(primTypeName != null);

                        /** @type {tcuTestCase.DeqpTest} */ var primTypeGroup = tcuTestCase.newTest(primTypeName, '');
                        specialCaseGroup.addChild(primTypeGroup);

                        for (var indexType in es3fPrimitiveRestartTests.IndexType) {
                            /** @type {string} */ var indexTypeName = es3fPrimitiveRestartTests.IndexType[indexType] == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_BYTE ? 'unsigned_byte' :
                                                                      es3fPrimitiveRestartTests.IndexType[indexType] == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_SHORT ? 'unsigned_short' :
                                                                      es3fPrimitiveRestartTests.IndexType[indexType] == es3fPrimitiveRestartTests.IndexType.INDEX_UNSIGNED_INT ? 'unsigned_int' :
                                                                      '';

                            DE_ASSERT(indexTypeName != null);

                            /** @type {tcuTestCase.DeqpTest} */ var indexTypeGroup = tcuTestCase.newTest(indexTypeName, '');
                            primTypeGroup.addChild(indexTypeGroup);

                            for (var _function in es3fPrimitiveRestartTests.DrawFunction) {
                                /** @type {?string} */ var functionName = es3fPrimitiveRestartTests.DrawFunction[_function] == es3fPrimitiveRestartTests.DrawFunction.FUNCTION_DRAW_ELEMENTS ? 'draw_elements' :
                                                                         es3fPrimitiveRestartTests.DrawFunction[_function] == es3fPrimitiveRestartTests.DrawFunction.FUNCTION_DRAW_ELEMENTS_INSTANCED ? 'draw_elements_instanced' :
                                                                         es3fPrimitiveRestartTests.DrawFunction[_function] == es3fPrimitiveRestartTests.DrawFunction.FUNCTION_DRAW_RANGE_ELEMENTS ? 'draw_range_elements' :
                                                                         null;

                                DE_ASSERT(functionName != null);

                                indexTypeGroup.addChild(new es3fPrimitiveRestartTests.PrimitiveRestartCase(functionName,
                                                                                 '',
                                                                                 es3fPrimitiveRestartTests.PrimitiveType[primType],
                                                                                 es3fPrimitiveRestartTests.IndexType[indexType],
                                                                                 es3fPrimitiveRestartTests.DrawFunction[_function],
                                                                                 isRestartBeginCase,
                                                                                 isRestartEndCase,
                                                                                 isDuplicateRestartCase));
                            }
                        }
                    }
                }
            }
        }
    };

    es3fPrimitiveRestartTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'primitive_restart';
        var testDescription = 'Primitive Restart Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.setRoot(tcuTestCase.newTest(testName, testDescription, null));

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fPrimitiveRestartTests.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fPrimitiveRestartTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

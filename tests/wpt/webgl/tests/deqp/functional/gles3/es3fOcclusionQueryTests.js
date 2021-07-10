/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2 (the 'License');
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an 'AS IS' BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fOcclusionQueryTests');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {
var es3fOcclusionQueryTests = functional.gles3.es3fOcclusionQueryTests;
var tcuTestCase = framework.common.tcuTestCase;
var tcuLogImage = framework.common.tcuLogImage;
var tcuSurface = framework.common.tcuSurface;
var deRandom = framework.delibs.debase.deRandom;
var deString = framework.delibs.debase.deString;
var gluShaderProgram = framework.opengl.gluShaderProgram;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/** @const */ var DEPTH_WRITE_COLOR = [0, 0, 1, 1];
/** @const */ var DEPTH_CLEAR_COLOR = [0, 0.5, 0.8, 1];
/** @const */ var STENCIL_WRITE_COLOR = [0, 1, 0, 1];
/** @const */ var STENCIL_CLEAR_COLOR = [0, 0.8, 0.5, 1];
/** @const */ var TARGET_COLOR = [1, 0, 0, 1];
/** @const */ var ELEMENTS_PER_VERTEX = 4;
/** @const */ var NUM_CASE_ITERATIONS = 10;

// Constants to tweak visible/invisible case probability balance.

/** @const */ var DEPTH_CLEAR_OFFSET = 100;
/** @const */ var STENCIL_CLEAR_OFFSET = 100;
/** @const */ var SCISSOR_OFFSET = 100;
/** @const */ var SCISSOR_MINSIZE = 250;

/** @const */ var OCCLUDER_SCISSOR = (1 << 0);
/** @const */ var OCCLUDER_DEPTH_WRITE = (1 << 1);
/** @const */ var OCCLUDER_DEPTH_CLEAR = (1 << 2);
/** @const */ var OCCLUDER_STENCIL_WRITE = (1 << 3);
/** @const */ var OCCLUDER_STENCIL_CLEAR = (1 << 4);

/**
 * @enum
 */
es3fOcclusionQueryTests.State = {
    DRAW: 0,
    VERIFY: 1,
    FINISH: 2
};

/* Maximum time to wait for query result (in seconds) */
/** @const */ var MAX_VERIFY_WAIT = 5;

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fOcclusionQueryTests.OcclusionQueryCase = function(name, description, numOccluderDraws, numOccludersPerDraw, occluderSize, numTargetDraws, numTargetsPerDraw, targetSize, queryMode, occluderTypes) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_numOccluderDraws = numOccluderDraws;
    this.m_numOccludersPerDraw = numOccludersPerDraw;
    this.m_occluderSize = occluderSize;
    this.m_numTargetDraws = numTargetDraws;
    this.m_numTargetsPerDraw = numTargetsPerDraw;
    this.m_targetSize = targetSize;
    this.m_queryMode = queryMode;
    this.m_occluderTypes = occluderTypes;
    this.m_program = null;
    this.m_iterNdx = 0;
    this.m_rnd = new deRandom.Random(deString.deStringHash(name));
    this.m_state = es3fOcclusionQueryTests.State.DRAW;
    /** @type {WebGLQuery} */ this.m_query;
};

setParentClass(es3fOcclusionQueryTests.OcclusionQueryCase, tcuTestCase.DeqpTest);

es3fOcclusionQueryTests.OcclusionQueryCase.prototype.generateVertices = function(width, height, primitiveCount, verticesPerPrimitive, rnd, primitiveSize, minZ, maxZ) {
    var dst = [];
    var w = width / 2;
    var h = height / 2;
    var s = primitiveSize / 2;

    var vertexCount = verticesPerPrimitive * primitiveCount;

    // First loop gets a random point inside unit square
    for (var i = 0; i < vertexCount; i += 3) {
        var rndX = rnd.getFloat(-w, w);
        var rndY = rnd.getFloat(-h, h);

        // Second loop gets 3 random points within given distance s from (rndX, rndY)
        for (var j = 0; j < verticesPerPrimitive; j++) {
            var offset = (i + j) * ELEMENTS_PER_VERTEX;
            dst[offset] = rndX + rnd.getFloat(-s, s); // x
            dst[offset + 1] = rndY + rnd.getFloat(-s, s); // y
            dst[offset + 2] = rnd.getFloat(minZ, maxZ); // z
            dst[offset + 3] = 1; // w
        }
    }
    return dst;
};

es3fOcclusionQueryTests.OcclusionQueryCase.prototype.init = function() {
    var vertShaderSource =
                '#version 300 es\n' +
                'layout(location = 0) in mediump vec4 a_position;\n' +
                '\n' +
                'void main (void)\n' +
                '{\n' +
                ' gl_Position = a_position;\n' +
                '}\n';

    var fragShaderSource =
                '#version 300 es\n' +
                'layout(location = 0) out mediump vec4 dEQP_FragColor;\n' +
                'uniform mediump vec4 u_color;\n' +
                '\n' +
                'void main (void)\n' +
                '{\n' +
                ' mediump float depth_gradient = gl_FragCoord.z;\n' +
                ' mediump float bias = 0.1;\n' +
                ' dEQP_FragColor = vec4(u_color.xyz * (depth_gradient + bias), 1);\n' +
                '}\n';

    this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertShaderSource, fragShaderSource));

    if (!this.m_program.isOk())
        testFailedOptions('Failed to compile program', true);

    this.m_buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.m_buffer);
    gl.vertexAttribPointer(0, ELEMENTS_PER_VERTEX, gl.FLOAT, false, 0, 0);
};

es3fOcclusionQueryTests.OcclusionQueryCase.prototype.draw = function() {
    var colorUnif = gl.getUniformLocation(this.m_program.getProgram(), 'u_color');

    var targetW = gl.drawingBufferWidth;
    var targetH = gl.drawingBufferHeight;

    bufferedLogToConsole('Case iteration ' + (this.m_iterNdx + 1) + ' / ' + NUM_CASE_ITERATIONS);
    bufferedLogToConsole('Parameters:\n' +
                                 '- ' + this.m_numOccluderDraws + ' occluder draws, ' + this.m_numOccludersPerDraw + ' primitive writes per draw,\n' +
                                 '- ' + this.m_numTargetDraws + ' target draws, ' + this.m_numTargetsPerDraw + ' targets per draw\n');

    gl.clearColor(0, 0, 0, 1);
    gl.clearDepth(1);
    gl.clearStencil(0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
    gl.useProgram(this.m_program.getProgram());
    gl.enableVertexAttribArray(0);

    // Draw occluders

    var occOptions = [];
    if (this.m_occluderTypes & OCCLUDER_DEPTH_WRITE) occOptions.push(OCCLUDER_DEPTH_WRITE);
    if (this.m_occluderTypes & OCCLUDER_DEPTH_CLEAR) occOptions.push(OCCLUDER_DEPTH_CLEAR);
    if (this.m_occluderTypes & OCCLUDER_STENCIL_WRITE) occOptions.push(OCCLUDER_STENCIL_WRITE);
    if (this.m_occluderTypes & OCCLUDER_STENCIL_CLEAR) occOptions.push(OCCLUDER_STENCIL_CLEAR);

    for (var i = 0; i < this.m_numOccluderDraws; i++) {
        if (occOptions.length == 0)
            break;

        var type = occOptions[this.m_rnd.getInt(0, occOptions.length - 1)]; // Choosing a random occluder type from available options

        switch (type) {
            case OCCLUDER_DEPTH_WRITE:
                bufferedLogToConsole('Occluder draw ' + (i + 1) + ' / ' + this.m_numOccluderDraws + ' : ' + 'Depth write');

                var occluderVertices = this.generateVertices(2, 2, this.m_numOccludersPerDraw, 3, this.m_rnd, this.m_occluderSize, 0, 0.6); // Generate vertices for occluding primitives

                gl.enable(gl.DEPTH_TEST);
                gl.uniform4f(colorUnif, DEPTH_WRITE_COLOR[0], DEPTH_WRITE_COLOR[1], DEPTH_WRITE_COLOR[2], DEPTH_WRITE_COLOR[3]);
                gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(occluderVertices), gl.STATIC_DRAW);
                gl.drawArrays(gl.TRIANGLES, 0, 3 * this.m_numOccludersPerDraw);
                gl.disable(gl.DEPTH_TEST);

                break;

            case OCCLUDER_DEPTH_CLEAR: {
                var scissorBoxX = this.m_rnd.getInt(-DEPTH_CLEAR_OFFSET, targetW);
                var scissorBoxY = this.m_rnd.getInt(-DEPTH_CLEAR_OFFSET, targetH);
                var scissorBoxW = this.m_rnd.getInt(DEPTH_CLEAR_OFFSET, targetW + DEPTH_CLEAR_OFFSET);
                var scissorBoxH = this.m_rnd.getInt(DEPTH_CLEAR_OFFSET, targetH + DEPTH_CLEAR_OFFSET);

                bufferedLogToConsole('Occluder draw ' + (i + 1) + ' / ' + this.m_numOccluderDraws + ' : ' + 'Depth clear');
                bufferedLogToConsole('Depth-clearing box drawn at ' +
                                                '(' + scissorBoxX + ', ' + scissorBoxY + ')' +
                                                ', width = ' + scissorBoxW + ', height = ' + scissorBoxH + '.');

                gl.enable(gl.SCISSOR_TEST);
                gl.scissor(scissorBoxX, scissorBoxY, scissorBoxW, scissorBoxH);
                gl.clearDepth(0);
                gl.clearColor(DEPTH_CLEAR_COLOR[0], DEPTH_CLEAR_COLOR[1], DEPTH_CLEAR_COLOR[2], DEPTH_CLEAR_COLOR[3]);
                gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
                gl.disable(gl.SCISSOR_TEST);

                break;
            }

            case OCCLUDER_STENCIL_WRITE:
                bufferedLogToConsole('Occluder draw ' + (i + 1) + ' / ' + this.m_numOccluderDraws + ' : ' + 'Stencil write');

                occluderVertices = this.generateVertices(2, 2, this.m_numOccludersPerDraw, 3, this.m_rnd, this.m_occluderSize, 0, 0.6);

                gl.stencilFunc(gl.ALWAYS, 1, 0xFF);
                gl.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);

                gl.enable(gl.STENCIL_TEST);
                gl.uniform4f(colorUnif, STENCIL_WRITE_COLOR[0], STENCIL_WRITE_COLOR[1], STENCIL_WRITE_COLOR[2], STENCIL_WRITE_COLOR[3]);
                gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(occluderVertices), gl.STATIC_DRAW);
                gl.drawArrays(gl.TRIANGLES, 0, 3 * this.m_numOccludersPerDraw);
                gl.disable(gl.STENCIL_TEST);

                break;

            case OCCLUDER_STENCIL_CLEAR: {
                var scissorBoxX = this.m_rnd.getInt(-STENCIL_CLEAR_OFFSET, targetW);
                var scissorBoxY = this.m_rnd.getInt(-STENCIL_CLEAR_OFFSET, targetH);
                var scissorBoxW = this.m_rnd.getInt(STENCIL_CLEAR_OFFSET, targetW + STENCIL_CLEAR_OFFSET);
                var scissorBoxH = this.m_rnd.getInt(STENCIL_CLEAR_OFFSET, targetH + STENCIL_CLEAR_OFFSET);

                bufferedLogToConsole('Occluder draw ' + (i + 1) + ' / ' + this.m_numOccluderDraws + ' : ' + 'Stencil clear');
                bufferedLogToConsole('Stencil-clearing box drawn at ' +
                                                '(' + scissorBoxX + ', ' + scissorBoxY + ')' +
                                                ', width = ' + scissorBoxW + ', height = ' + scissorBoxH + '.');

                gl.enable(gl.SCISSOR_TEST);
                gl.scissor(scissorBoxX, scissorBoxY, scissorBoxW, scissorBoxH);
                gl.clearStencil(1);
                gl.clearColor(STENCIL_CLEAR_COLOR[0], STENCIL_CLEAR_COLOR[1], STENCIL_CLEAR_COLOR[2], STENCIL_CLEAR_COLOR[3]);
                gl.clear(gl.COLOR_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
                gl.disable(gl.SCISSOR_TEST);

                break;
            }

            default:
                throw new Error('Invalid occluder type: ' + type);
        }
    }

    if (this.m_occluderTypes & OCCLUDER_SCISSOR) {
        var scissorBoxX = this.m_rnd.getInt(-SCISSOR_OFFSET, targetW - SCISSOR_OFFSET);
        var scissorBoxY = this.m_rnd.getInt(-SCISSOR_OFFSET, targetH - SCISSOR_OFFSET);
        var scissorBoxW = this.m_rnd.getInt(SCISSOR_MINSIZE, targetW + SCISSOR_OFFSET);
        var scissorBoxH = this.m_rnd.getInt(SCISSOR_MINSIZE, targetH + SCISSOR_OFFSET);

        bufferedLogToConsole('Scissor box drawn at ' +
                                        '(' + scissorBoxX + ', ' + scissorBoxY + ')' +
                                        ', width = ' + scissorBoxW + ', height = ' + scissorBoxH + '.');

        gl.enable(gl.SCISSOR_TEST);
        gl.scissor(scissorBoxX, scissorBoxY, scissorBoxW, scissorBoxH);
    }

    this.m_query = gl.createQuery();
    gl.beginQuery(this.m_queryMode, this.m_query);

    // Draw target primitives

    gl.enable(gl.DEPTH_TEST);
    gl.enable(gl.STENCIL_TEST);
    gl.stencilFunc(gl.EQUAL, 0, 0xFF);

    for (var i = 0; i < this.m_numTargetDraws; i++) {
        var targetVertices = this.generateVertices(2, 2, this.m_numTargetsPerDraw, 3, this.m_rnd, this.m_targetSize, 0.4, 1); // Generate vertices for target primitives

        if (targetVertices.length > 0) {
            gl.uniform4f(colorUnif, TARGET_COLOR[0], TARGET_COLOR[1], TARGET_COLOR[2], TARGET_COLOR[3]);
            gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(targetVertices), gl.STATIC_DRAW);
            gl.drawArrays(gl.TRIANGLES, 0, 3 * this.m_numTargetsPerDraw);
        }
    }

    gl.endQuery(this.m_queryMode);
    gl.disable(gl.SCISSOR_TEST);
    gl.disable(gl.STENCIL_TEST);
    gl.disable(gl.DEPTH_TEST);
    this.m_state = es3fOcclusionQueryTests.State.VERIFY;
};

es3fOcclusionQueryTests.OcclusionQueryCase.prototype.verify = function() {
    // Check that query result is available.
    var resultAvailable = /** @type {boolean} */ (gl.getQueryParameter(this.m_query, gl.QUERY_RESULT_AVAILABLE));
    if (!resultAvailable) {
        if (!this.m_verifyStart)
            this.m_verifyStart = new Date();
        else {
            var current = new Date();
            var elapsedTime = 0.001 * (current.getTime() - this.m_verifyStart.getTime());
            if (elapsedTime > MAX_VERIFY_WAIT) {
                testFailed('Query result not available after ' + elapsedTime + ' seconds.');
                this.m_state = es3fOcclusionQueryTests.State.FINISH;
            }
        }
        return;
    }

    // Read query result.
    var result = /** @type {number} */ (gl.getQueryParameter(this.m_query, gl.QUERY_RESULT));
    var queryResult = (result > 0);

    gl.deleteQuery(this.m_query);

    // Read pixel data

    var pixels = new tcuSurface.Surface();
    pixels.readViewport(gl);
    var colorReadResult = false;
    var width = pixels.getWidth();
    var height = pixels.getHeight();

    for (var y = 0; y < height; y++) {
        for (var x = 0; x < width; x++) {
            if (pixels.getPixel(x, y)[0] != 0) {
                colorReadResult = true;
                break;
            }
        }
        if (colorReadResult) break;
    }

    var message = 'Occlusion query result: Target ' + (queryResult ? 'visible' : 'invisible') + '. ' +
                                 'Framebuffer read result: Target ' + (colorReadResult ? 'visible' : 'invisible');

    var testOk = false;
    if (this.m_queryMode == gl.ANY_SAMPLES_PASSED_CONSERVATIVE) {
        if (queryResult || colorReadResult)
            testOk = queryResult; // Allow conservative occlusion query to return false positives.
        else
            testOk = queryResult == colorReadResult;
    } else
        testOk = (queryResult == colorReadResult);

    if (!testOk) {
        tcuLogImage.logImage('Result image', 'Result image', pixels.getAccess());
        testFailed(message);
        this.m_state = es3fOcclusionQueryTests.State.FINISH;
        return;
    }

    bufferedLogToConsole(message);
    bufferedLogToConsole('Case passed!');

    if (++this.m_iterNdx < NUM_CASE_ITERATIONS) {
        this.m_state = es3fOcclusionQueryTests.State.DRAW
    } else {
        this.m_state = es3fOcclusionQueryTests.State.FINISH;
        testPassed();
    }
};


es3fOcclusionQueryTests.OcclusionQueryCase.prototype.iterate = function() {
    switch(this.m_state) {
        case es3fOcclusionQueryTests.State.DRAW:
            this.draw();
            break;
        case es3fOcclusionQueryTests.State.VERIFY:
            this.verify();
            break;
        case es3fOcclusionQueryTests.State.FINISH:
            return tcuTestCase.IterateResult.STOP;
        default:
            throw new Error('Invalid state: ' + this.m_state);
    }

    return tcuTestCase.IterateResult.CONTINUE;
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fOcclusionQueryTests.OcclusionQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'occlusion_query', 'Occlusion Query Tests');
};

es3fOcclusionQueryTests.OcclusionQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fOcclusionQueryTests.OcclusionQueryTests.prototype.constructor = es3fOcclusionQueryTests.OcclusionQueryTests;

es3fOcclusionQueryTests.OcclusionQueryTests.prototype.init = function() {
    // Strict occlusion query cases

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor', 'scissor', 1, 10, 1.6, 1, 1, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write', 'depth_write', 8, 10, 1.6, 1, 7, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_clear', 'depth_clear', 5, 10, 1.6, 1, 5, 0.2, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('stencil_write', 'stencil_write', 8, 10, 2.0, 1, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('stencil_clear', 'stencil_clear', 5, 10, 2.0, 1, 3, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write', 'scissor_depth_write', 5, 10, 1.6, 2, 5, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_clear', 'scissor_depth_clear', 7, 10, 1.6, 2, 5, 1.0, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_stencil_write', 'scissor_stencil_write', 4, 10, 1.6, 2, 5, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_stencil_clear', 'scissor_stencil_clear', 4, 10, 1.6, 2, 5, 1.0, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_depth_clear', 'depth_write_depth_clear', 7, 10, 1.6, 1, 5, 0.2, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_stencil_write', 'depth_write_stencil_write', 8, 10, 1.6, 1, 5, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_stencil_clear', 'depth_write_stencil_clear', 8, 10, 1.6, 1, 5, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_clear_stencil_write', 'depth_clear_stencil_write', 8, 10, 1.6, 1, 5, 0.3, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_clear_stencil_clear', 'depth_clear_stencil_clear', 12, 10, 1.6, 1, 5, 0.2, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('stencil_write_stencil_clear', 'stencil_write_stencil_clear', 5, 10, 2.0, 1, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write_depth_clear', 'scissor_depth_write_depth_clear', 5, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write_stencil_write', 'scissor_depth_write_stencil_write', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write_stencil_clear', 'scissor_depth_write_stencil_clear', 6, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_clear_stencil_write', 'scissor_depth_clear_stencil_write', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_clear_stencil_clear', 'scissor_depth_clear_stencil_clear', 5, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_stencil_write_stencil_clear', 'scissor_stencil_write_stencil_clear', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_depth_clear_stencil_write', 'depth_write_depth_clear_stencil_write', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_depth_clear_stencil_clear', 'depth_write_depth_clear_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_stencil_write_stencil_clear', 'depth_write_stencil_write_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_clear_stencil_write_stencil_clear', 'depth_clear_stencil_write_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write_depth_clear_stencil_write', 'scissor_depth_write_depth_clear_stencil_write', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write_depth_clear_stencil_clear', 'scissor_depth_write_depth_clear_stencil_clear', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_write_stencil_write_stencil_clear', 'scissor_depth_write_stencil_write_stencil_clear', 5, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('scissor_depth_clear_stencil_write_stencil_clear', 'scissor_depth_clear_stencil_write_stencil_clear', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('depth_write_depth_clear_stencil_write_stencil_clear', 'depth_write_depth_clear_stencil_write_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('all_occluders', 'all_occluders', 7, 10, 1.6, 3, 5, 0.6, gl.ANY_SAMPLES_PASSED, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    // Conservative occlusion query cases

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor', 'conservative_scissor', 1, 10, 1.6, 1, 1, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write', 'conservative_depth_write', 8, 10, 1.6, 1, 7, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_clear', 'conservative_depth_clear', 5, 10, 1.6, 1, 5, 0.2, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_stencil_write', 'conservative_stencil_write', 8, 10, 2.0, 1, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_stencil_clear', 'conservative_stencil_clear', 5, 10, 2.0, 1, 3, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write', 'conservative_scissor_depth_write', 5, 10, 1.6, 2, 5, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_clear', 'conservative_scissor_depth_clear', 7, 10, 1.6, 2, 5, 1.0, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_stencil_write', 'conservative_scissor_stencil_write', 4, 10, 1.6, 2, 5, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_stencil_clear', 'conservative_scissor_stencil_clear', 4, 10, 1.6, 2, 5, 1.0, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_depth_clear', 'conservative_depth_write_depth_clear', 7, 10, 1.6, 1, 5, 0.2, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_stencil_write', 'conservative_depth_write_stencil_write', 8, 10, 1.6, 1, 5, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_stencil_clear', 'conservative_depth_write_stencil_clear', 8, 10, 1.6, 1, 5, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_clear_stencil_write', 'conservative_depth_clear_stencil_write', 8, 10, 1.6, 1, 5, 0.3, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_clear_stencil_clear', 'conservative_depth_clear_stencil_clear', 12, 10, 1.6, 1, 5, 0.2, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_stencil_write_stencil_clear', 'conservative_stencil_write_stencil_clear', 5, 10, 2.0, 1, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write_depth_clear', 'conservative_scissor_depth_write_depth_clear', 5, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write_stencil_write', 'conservative_scissor_depth_write_stencil_write', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write_stencil_clear', 'conservative_scissor_depth_write_stencil_clear', 6, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_clear_stencil_write', 'conservative_scissor_depth_clear_stencil_write', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_clear_stencil_clear', 'conservative_scissor_depth_clear_stencil_clear', 5, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_stencil_write_stencil_clear', 'conservative_scissor_stencil_write_stencil_clear', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_depth_clear_stencil_write', 'conservative_depth_write_depth_clear_stencil_write', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_depth_clear_stencil_clear', 'conservative_depth_write_depth_clear_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_stencil_write_stencil_clear', 'conservative_depth_write_stencil_write_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_clear_stencil_write_stencil_clear', 'conservative_depth_clear_stencil_write_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write_depth_clear_stencil_write', 'conservative_scissor_depth_write_depth_clear_stencil_write', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write_depth_clear_stencil_clear', 'conservative_scissor_depth_write_depth_clear_stencil_clear', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_write_stencil_write_stencil_clear', 'conservative_scissor_depth_write_stencil_write_stencil_clear', 5, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_scissor_depth_clear_stencil_write_stencil_clear', 'conservative_scissor_depth_clear_stencil_write_stencil_clear', 4, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));
    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_depth_write_depth_clear_stencil_write_stencil_clear', 'conservative_depth_write_depth_clear_stencil_write_stencil_clear', 7, 10, 1.6, 2, 5, 0.4, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

    this.addChild(new es3fOcclusionQueryTests.OcclusionQueryCase('conservative_all_occluders', 'conservative_all_occluders', 7, 10, 1.6, 3, 5, 0.6, gl.ANY_SAMPLES_PASSED_CONSERVATIVE, OCCLUDER_SCISSOR | OCCLUDER_DEPTH_WRITE | OCCLUDER_DEPTH_CLEAR | OCCLUDER_STENCIL_WRITE | OCCLUDER_STENCIL_CLEAR));

};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fOcclusionQueryTests.run = function(context, range) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fOcclusionQueryTests.OcclusionQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        if (range)
            state.setRange(range);
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fOcclusionQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

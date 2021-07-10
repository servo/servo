/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES 3.0 Module
 * -------------------------------------------------
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
 * \brief Rasterizer discard tests.
 *//*--------------------------------------------------------------------*/

goog.provide('functional.gles3.es3fRasterizerDiscardTests');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {
var es3fRasterizerDiscardTests = functional.gles3.es3fRasterizerDiscardTests;
var deString = framework.delibs.debase.deString;
var tcuTestCase = framework.common.tcuTestCase;
var deRandom = framework.delibs.debase.deRandom;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuSurface = framework.common.tcuSurface;
var gluDrawUtil = framework.opengl.gluDrawUtil;
var tcuLogImage = framework.common.tcuLogImage;

/** @const */ var NUM_CASE_ITERATIONS = 1;
/** @const */ var FAIL_COLOR_RED = [1, 0, 0.0, 1];
/** @const */ var PASS_COLOR_BLUE = [0, 0, 0.5, 1];
/** @const */ var BLACK_COLOR = [0, 0, 0.0, 1];
/** @const */ var FAIL_DEPTH = 0;
/** @const */ var FAIL_STENCIL = 1;
/** @const */ var UNIT_SQUARE = [
     1, 1, 0.05, 1,
     1, -1, 0.05, 1,
    -1, 1, 0.05, 1,
    -1, -1, 0.05, 1
];

/** @type {WebGL2RenderingContext} */ var gl;

/**
 * @enum
 */
es3fRasterizerDiscardTests.CaseType = {
    WRITE_DEPTH: 0,
    WRITE_STENCIL: 1,
    CLEAR_COLOR: 2,
    CLEAR_DEPTH: 3,
    CLEAR_STENCIL: 4
};

/**
 * @enum {{useFBO: boolean, useScissor: boolean}}
 */
es3fRasterizerDiscardTests.CaseOptions = {
    FBO: {useFBO: true, useScissor: false},
    SCISSOR: {useFBO: false, useScissor: true}
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 * @param {string} name
 * @param {string} description
 * @param {number} numPrimitives
 * @param {es3fRasterizerDiscardTests.CaseType} caseType
 * @param {?es3fRasterizerDiscardTests.CaseOptions} caseOptions
 * @param {gluDrawUtil.primitiveType=} drawMode
 */
es3fRasterizerDiscardTests.RasterizerDiscardCase = function(name, description, numPrimitives, caseType, caseOptions, drawMode) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_numPrimitives = numPrimitives;
    this.m_caseType = caseType;
    this.m_caseOptions = caseOptions || {useFBO: false, useScissor: false};
    this.m_drawMode = drawMode || gluDrawUtil.primitiveType.TRIANGLES;
    this.m_program = null;
    this.m_fbo = null;
    this.m_colorBuf = null;
    this.m_depthStencilBuf = null;
    this.m_iterNdx = 0;
    this.m_rnd = new deRandom.Random(deString.deStringHash(name));
};

es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype.constructor = es3fRasterizerDiscardTests.RasterizerDiscardCase;

/**
 * @param {number} numPrimitives
 * @param {deRandom.Random} rnd
 * @param {gluDrawUtil.primitiveType} drawMode
 * @return {Array<number>}
 */
es3fRasterizerDiscardTests.generateVertices = function(numPrimitives, rnd, drawMode) {
    var dst = [];
    var numVertices;

    switch (drawMode) {
        case gl.POINTS: numVertices = numPrimitives; break;
        case gl.LINES: numVertices = 2 * numPrimitives; break;
        case gl.LINE_STRIP: numVertices = numPrimitives + 1; break;
        case gl.LINE_LOOP: numVertices = numPrimitives + 2; break;
        case gl.TRIANGLES: numVertices = 3 * numPrimitives; break;
        case gl.TRIANGLE_STRIP: numVertices = numPrimitives + 2; break;
        case gl.TRIANGLE_FAN: numVertices = numPrimitives + 2; break;
        default:
            throw new Error('Invalid drawMode: ' + drawMode);
    }

    for (var i = 0; i < numVertices; i++) {
        dst[i * 4] = rnd.getFloat(-1.0, 1.0); // x
        dst[i * 4 + 1] = rnd.getFloat(-1.0, 1.0); // y
        dst[i * 4 + 2] = rnd.getFloat(0.1, 0.9); // z
        dst[i * 4 + 3] = 1.0; // w
    }

    return dst;
};

es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype.setupFramebufferObject = function() {
    var width = gl.drawingBufferWidth;
    var height = gl.drawingBufferHeight;

    // Create framebuffer object

    this.m_fbo = gl.createFramebuffer();
    this.m_colorBuf = gl.createTexture();
    this.m_depthStencilBuf = gl.createRenderbuffer();

    // Create color texture

    gl.bindTexture(gl.TEXTURE_2D, this.m_colorBuf);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

    // Create depth and stencil buffers

    gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_depthStencilBuf);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH24_STENCIL8, width, height);

    // Attach texture and buffers to FBO

    gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, this.m_colorBuf, 0);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, this.m_depthStencilBuf);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, this.m_depthStencilBuf);

    var fboStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);

    if (fboStatus == gl.FRAMEBUFFER_UNSUPPORTED)
        throw new Error('Framebuffer unsupported');
    else if (fboStatus != gl.FRAMEBUFFER_COMPLETE)
        throw new Error('Failed to create framebuffer object: ' + deString.enumToString(gl, fboStatus));
};

es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype.deleteFramebufferObject = function() {
    gl.deleteTexture(this.m_colorBuf);
    gl.deleteRenderbuffer(this.m_depthStencilBuf);
    gl.deleteFramebuffer(this.m_fbo);
};

es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype.init = function() {
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
                ' dEQP_FragColor = vec4(u_color.xyz * (depth_gradient + bias), 1.0);\n' +
                '}\n';

    this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertShaderSource, fragShaderSource));

    if (!this.m_program.isOk()) {
        bufferedLogToConsole(this.m_program);
        testFailedOptions('Failed to compile shader program', true);
    }
};

es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype.deinit = function() {
    this.deleteFramebufferObject();
    this.m_program = null;
};

es3fRasterizerDiscardTests.RasterizerDiscardCase.prototype.iterate = function() {
    var program = this.m_program.getProgram();
    var colorUnif = gl.getUniformLocation(program, 'u_color');
    var failColorFound = false;
    var passColorFound = false;
    var vertices;

    bufferedLogToConsole('Case iteration ' + (this.m_iterNdx + 1) + ' / ' + NUM_CASE_ITERATIONS);

    // Create and bind FBO if needed

    if (this.m_caseOptions.useFBO) {
        this.setupFramebufferObject();
    }

    if (this.m_caseOptions.useScissor) {
        gl.enable(gl.SCISSOR_TEST);
        gl.scissor(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight);
        bufferedLogToConsole('Scissor test enabled: glScissor(0, 0, ' + gl.drawingBufferWidth + ', ' + gl.drawingBufferHeight + ')');
    }

    gl.useProgram(this.m_program.getProgram());

    gl.enable(gl.DEPTH_TEST);
    gl.depthRange(0, 1);
    gl.depthFunc(gl.LEQUAL);

    gl.enable(gl.STENCIL_TEST);
    gl.stencilFunc(gl.NOTEQUAL, 1, 0xFF);
    gl.stencilOp(gl.REPLACE, gl.KEEP, gl.KEEP);

    gl.clearColor(PASS_COLOR_BLUE[0], PASS_COLOR_BLUE[1], PASS_COLOR_BLUE[2], PASS_COLOR_BLUE[3]);
    gl.clearDepth(1);
    gl.clearStencil(0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    // Generate vertices
    vertices = es3fRasterizerDiscardTests.generateVertices(this.m_numPrimitives, this.m_rnd, this.m_drawMode);
    var posLoc = gl.getAttribLocation(program, 'a_position');
    var vertexArrays = [];
    vertexArrays.push(new gluDrawUtil.VertexArrayBinding(gl.FLOAT, posLoc, 4, vertices.length / 4, vertices));
    // Clear color to black for depth and stencil clear cases

    if (this.m_caseType == es3fRasterizerDiscardTests.CaseType.CLEAR_DEPTH || this.m_caseType == es3fRasterizerDiscardTests.CaseType.CLEAR_STENCIL) {
        gl.clearColor(BLACK_COLOR[0], BLACK_COLOR[1], BLACK_COLOR[2], BLACK_COLOR[3]);
        gl.clear(gl.COLOR_BUFFER_BIT);
    }

    // Set fail values for color, depth and stencil

    gl.uniform4fv(colorUnif, FAIL_COLOR_RED);
    gl.clearColor(FAIL_COLOR_RED[0], FAIL_COLOR_RED[1], FAIL_COLOR_RED[2], FAIL_COLOR_RED[3]);
    gl.clearDepth(FAIL_DEPTH);
    gl.clearStencil(FAIL_STENCIL);

    // Enable rasterizer discard

    gl.enable(gl.RASTERIZER_DISCARD);
    bufferedLogToConsole('Rasterizer discard enabled');

    // Do to-be-discarded primitive draws and buffer clears

    switch (this.m_caseType) {
        case es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH:
            gluDrawUtil.draw(gl, program, vertexArrays, new gluDrawUtil.PrimitiveList(this.m_drawMode, vertices.length / 4));
            break;
        case es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL:
            gluDrawUtil.draw(gl, program, vertexArrays, new gluDrawUtil.PrimitiveList(this.m_drawMode, vertices.length / 4));
            break;
        case es3fRasterizerDiscardTests.CaseType.CLEAR_COLOR:
            if (this.m_caseOptions.useFBO)
                gl.clearBufferfv(gl.COLOR, 0, FAIL_COLOR_RED);
            else
                gl.clear(gl.COLOR_BUFFER_BIT);
            break;
        case es3fRasterizerDiscardTests.CaseType.CLEAR_DEPTH:
            if (this.m_caseOptions.useFBO)
                gl.clearBufferfv(gl.DEPTH, 0, [FAIL_DEPTH]);
            else
                gl.clear(gl.DEPTH_BUFFER_BIT);
            break;
        case es3fRasterizerDiscardTests.CaseType.CLEAR_STENCIL:
            if (this.m_caseOptions.useFBO)
                gl.clearBufferiv(gl.STENCIL, 0, [FAIL_STENCIL]);
            else
                gl.clear(gl.STENCIL_BUFFER_BIT);
            break;
        default:
            throw new Error('Invalid case type ' + this.m_caseType);
    }

    // Disable rasterizer discard

    gl.disable(gl.RASTERIZER_DISCARD);
    bufferedLogToConsole('Rasterizer discard disabled');

    if (this.m_caseType == es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL) {
        if (this.m_caseOptions.useFBO || gl.getContextAttributes().stencil) {
            // Draw a full-screen square that colors all pixels red if they have stencil value 1.
            var square = [new gluDrawUtil.VertexArrayBinding(gl.FLOAT, posLoc, 4, UNIT_SQUARE.length / 4, UNIT_SQUARE)];

            gl.stencilFunc(gl.EQUAL, 1, 0xFF);
            gluDrawUtil.draw(gl, program, square,
             new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLE_STRIP, UNIT_SQUARE.length / 4));
        }
        // \note If no stencil buffers are present and test is rendering to default framebuffer, test will always pass.
    } else if (this.m_caseType == es3fRasterizerDiscardTests.CaseType.CLEAR_DEPTH || this.m_caseType == es3fRasterizerDiscardTests.CaseType.CLEAR_STENCIL) {
        // Draw pass-indicating primitives for depth and stencil clear cases

        gl.uniform4fv(colorUnif, PASS_COLOR_BLUE);
        gluDrawUtil.draw(gl, program, vertexArrays, new gluDrawUtil.PrimitiveList(this.m_drawMode, vertices.length / 4));
    }

    gl.finish();
    gl.disable(gl.STENCIL_TEST);
    gl.disable(gl.DEPTH_TEST);
    gl.disable(gl.SCISSOR_TEST);

    // Read and check pixel data

    var pixels = new tcuSurface.Surface();
    pixels.readViewport(gl);

    var width = pixels.getWidth();
    var height = pixels.getHeight();

    for (var y = 0; y < height; y++) {
        for (var x = 0; x < width; x++) {
            var pixel = pixels.getPixel(x, y);
            if (pixel[2] != 0)
                passColorFound = true;

            if (pixel[0] != 0) {
                failColorFound = true;
                break;
            }
        }
        if (failColorFound) break;
    }

    // Delete FBO if created

    if (this.m_caseOptions.useFBO)
        this.deleteFramebufferObject();

    // Evaluate test result

    var testOk = passColorFound && !failColorFound;

    if (!testOk) {
        tcuLogImage.logImage('Result image', '', pixels.getAccess());
        testFailed('Primitive or buffer clear was not discarded.');
        return tcuTestCase.IterateResult.STOP;
    }
    bufferedLogToConsole('Primitive or buffer clear was discarded correctly.');

    if (++this.m_iterNdx < NUM_CASE_ITERATIONS)
        return tcuTestCase.IterateResult.CONTINUE;

    testPassed();
    return tcuTestCase.IterateResult.STOP;
};

es3fRasterizerDiscardTests.init = function() {
    var state = tcuTestCase.runner;
    var testGroup = state.testCases;

    var basic = tcuTestCase.newTest('basic', 'Rasterizer discard test for default framebuffer');
    var scissor = tcuTestCase.newTest('scissor', 'Rasterizer discard test for default framebuffer with scissor test enabled');
    var fbo = tcuTestCase.newTest('fbo', 'Rasterizer discard test for framebuffer object');

    testGroup.addChild(basic);
    testGroup.addChild(scissor);
    testGroup.addChild(fbo);

    // Default framebuffer cases

    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_points', 'points', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.POINTS));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_lines', 'lines', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.LINES));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_line_strip', 'line_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.LINE_STRIP));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_line_loop', 'line_loop', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.LINE_LOOP));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangles', 'triangles', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.TRIANGLES));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangle_strip', 'triangle_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.TRIANGLE_STRIP));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangle_fan', 'triangle_fan', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, null, gluDrawUtil.primitiveType.TRIANGLE_FAN));

    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_points', 'points', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.POINTS));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_lines', 'lines', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.LINES));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_line_strip', 'line_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.LINE_STRIP));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_line_loop', 'line_loop', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.LINE_LOOP));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangles', 'triangles', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.TRIANGLES));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangle_strip', 'triangle_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.TRIANGLE_STRIP));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangle_fan', 'triangle_fan', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, null, gluDrawUtil.primitiveType.TRIANGLE_FAN));

    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_color', 'clear_color', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_COLOR, null));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_depth', 'clear_depth', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_DEPTH, null));
    basic.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_stencil', 'clear_stencil', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_STENCIL, null));

    // Default framebuffer cases with scissor test enabled

    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_points', 'points', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.POINTS));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_lines', 'lines', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.LINES));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_line_strip', 'line_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.LINE_STRIP));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_line_loop', 'line_loop', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.LINE_LOOP));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangles', 'triangles', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.TRIANGLES));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangle_strip', 'triangle_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.TRIANGLE_STRIP));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangle_fan', 'triangle_fan', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.TRIANGLE_FAN));

    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_points', 'points', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.POINTS));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_lines', 'lines', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.LINES));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_line_strip', 'line_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.LINE_STRIP));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_line_loop', 'line_loop', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.LINE_LOOP));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangles', 'triangles', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.TRIANGLES));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangle_strip', 'triangle_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.TRIANGLE_STRIP));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangle_fan', 'triangle_fan', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR, gluDrawUtil.primitiveType.TRIANGLE_FAN));

    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_color', 'clear_color', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_COLOR, es3fRasterizerDiscardTests.CaseOptions.SCISSOR));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_depth', 'clear_depth', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_DEPTH, es3fRasterizerDiscardTests.CaseOptions.SCISSOR));
    scissor.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_stencil', 'clear_stencil', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_STENCIL, es3fRasterizerDiscardTests.CaseOptions.SCISSOR));

    // FBO cases

    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_points', 'points', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.POINTS));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_lines', 'lines', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.LINES));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_line_strip', 'line_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.LINE_STRIP));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_line_loop', 'line_loop', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.LINE_LOOP));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangles', 'triangles', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.TRIANGLES));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangle_strip', 'triangle_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.TRIANGLE_STRIP));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_depth_triangle_fan', 'triangle_fan', 4, es3fRasterizerDiscardTests.CaseType.WRITE_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.TRIANGLE_FAN));

    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_points', 'points', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.POINTS));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_lines', 'lines', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.LINES));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_line_strip', 'line_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.LINE_STRIP));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_line_loop', 'line_loop', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.LINE_LOOP));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangles', 'triangles', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.TRIANGLES));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangle_strip', 'triangle_strip', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.TRIANGLE_STRIP));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('write_stencil_triangle_fan', 'triangle_fan', 4, es3fRasterizerDiscardTests.CaseType.WRITE_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO, gluDrawUtil.primitiveType.TRIANGLE_FAN));

    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_color', 'clear_color', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_COLOR, es3fRasterizerDiscardTests.CaseOptions.FBO));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_depth', 'clear_depth', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_DEPTH, es3fRasterizerDiscardTests.CaseOptions.FBO));
    fbo.addChild(new es3fRasterizerDiscardTests.RasterizerDiscardCase('clear_stencil', 'clear_stencil', 4, es3fRasterizerDiscardTests.CaseType.CLEAR_STENCIL, es3fRasterizerDiscardTests.CaseOptions.FBO));
};

/**
 * Create and execute the test cases
 */
es3fRasterizerDiscardTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var testName = 'rasterizer_discard';
    var testDescription = 'Rasterizer Discard Tests';
    var state = tcuTestCase.runner;

    state.testName = testName;
    state.testCases = tcuTestCase.newTest(testName, testDescription, null);

    //Set up name and description of this test series.
    setCurrentTestName(testName);
    description(testDescription);

    try {
        es3fRasterizerDiscardTests.init();
        tcuTestCase.runTestCases();
    } catch (err) {
        testFailedOptions('Failed to run tests', false);
        bufferedLogToConsole(err);
        tcuTestCase.runner.terminate();
    }
};

});

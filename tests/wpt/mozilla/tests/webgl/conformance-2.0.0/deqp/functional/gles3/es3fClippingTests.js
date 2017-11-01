/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the 'License');
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an 'AS IS' BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fClippingTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.referencerenderer.rrUtil');
goog.require('functional.gles3.es3fFboTestCase');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {
var es3fClippingTests = functional.gles3.es3fClippingTests;
var tcuImageCompare = framework.common.tcuImageCompare;
var tcuTestCase = framework.common.tcuTestCase;
var es3fFboTestCase = functional.gles3.es3fFboTestCase;
var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var rrUtil = framework.referencerenderer.rrUtil;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var deRandom = framework.delibs.debase.deRandom;
var deMath = framework.delibs.debase.deMath;
var tcuRGBA = framework.common.tcuRGBA;

/** @type {WebGL2RenderingContext} */ var gl;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {Array<number>} viewport
 * @param {Array<number>} rangeX
 * @param {Array<number>} rangeY
 * @param {Array<number>} rangeZ
 */
es3fClippingTests.TriangleCase = function(name, desc, viewport, rangeX, rangeY, rangeZ) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_viewport = viewport;
    this.m_rangeX = rangeX;
    this.m_rangeY = rangeY;
    this.m_rangeZ = rangeZ;
};

setParentClass(es3fClippingTests.TriangleCase, es3fFboTestCase.FboTestCase);

es3fClippingTests.TriangleCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var x = this.m_viewport[0];
        var y = this.m_viewport[1];
        var width = this.m_viewport[2];
        var height = this.m_viewport[3];
        ctx.viewport(x, y, width, height);
        ctx.clearColor(0, 0, 0, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT);

        var shader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
        var program = ctx.createProgram(shader);
        shader.setGradient(ctx, program, [0, 0, 0, 0], [1, 1, 1, 1]);

        rrUtil.drawQuad(ctx, program,
            [this.m_rangeX[0], this.m_rangeY[0], this.m_rangeZ[0]],
            [this.m_rangeX[1], this.m_rangeY[1], this.m_rangeZ[1]]);
        dst.readViewport(ctx, this.m_viewport);
};

/**
 * Move the vertex coordinate to pixel center
 */
var center = function(x, width) {
    var half = width / 2;
    var pos = half + x * half;
    // almost to the center to avoid problems when rounding
    // the position the pixel edge
    pos = Math.round(pos) + 0.49;
    return (pos - half) / half;
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {Array<number>} viewport
 * @param {number} lineWidth
 */
es3fClippingTests.LinesCase = function(name, desc, viewport, lineWidth) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_viewport = viewport;
    this.m_lineWidth = lineWidth;
};

setParentClass(es3fClippingTests.LinesCase, es3fFboTestCase.FboTestCase);

es3fClippingTests.LinesCase.prototype.compare = function(reference, result) {
    return tcuImageCompare.bilinearCompare('Result', 'Image comparison result',
        reference.getAccess(),
        result.getAccess(),
        tcuRGBA.newRGBAComponents(3, 3, 3, 3));
};

es3fClippingTests.LinesCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var x = this.m_viewport[0];
        var y = this.m_viewport[1];
        var width = this.m_viewport[2];
        var height = this.m_viewport[3];
        ctx.viewport(x, y, width, height);
        ctx.clearColor(0, 0, 0, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT);

        var shader = new es3fFboTestUtil.GradientShader(gluShaderUtil.DataType.FLOAT_VEC4);
        var program = ctx.createProgram(shader);
        shader.setGradient(ctx, program, [0, 0, 0, 0], [1, 1, 1, 1]);

        // positions
        var posLoc = ctx.getAttribLocation(program, 'a_position');
        if (posLoc == -1)
            throw new Error('a_position attribute is not defined.');

        var buffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, buffer);

        ctx.lineWidth(this.m_lineWidth);

        var y1 = center(-0.5, height);
        var y2 = center(-0.2, height);
        var y3 = center(0.2, height);
        var y4 = center(0.5, height);
        var y5 = center(0, height);
        var x1 = center(-0.5, width);
        var x2 = center(-0.2, width);
        var x3 = center(0.2, width);
        var x4 = center(0.5, width);
        var positions = [
            // horizontal check
            // both ends outside viewport
            -1 - 1 / width, y1, 0, 1,
            1 + 1 / width, y1, 0, 1,
            // one end inside viewport
            -1 + 1 / width, y2, 0, 1,
            1 + 1 / width, y2, 0, 1,

            -1 - 1 / width, y3, 0, 1,
            1 - 1 / width, y3, 0, 1,
            // both ends inside viewport

            -1 + 1 / width, y4, 0, 1,
            1 - 1 / width, y4, 0, 1,

            //vertical check
            // both ends outside viewport
            x1, -1 - 1 / height, 0, 1,
            x1, 1 + 1 / height, 0, 1,

            // one end inside viewport
            x2, -1 + 1 / height, 0, 1,
            x2, 1 + 1 / height, 0, 1,

            x3, -1 - 1 / height, 0, 1,
            x3, 1 - 1 / height, 0, 1,
            //both ends inside viewport
            x4, -1 + 1 / height, 0, 1,
            x4, 1 - 1 / height, 0, 1,

            //depth check
            -1, y5, -1.5, 1,
            1, y5, 1.1, 1
        ];
        var numVertices = positions.length / 4;

        ctx.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

        ctx.enableVertexAttribArray(posLoc);
        ctx.vertexAttribPointer(posLoc, 4, gl.FLOAT, false, 0, 0);

        //colors
        var coordLoc = ctx.getAttribLocation(program, 'a_coord');
        if (coordLoc == -1)
            throw new Error('a_coord attribute is not defined.');

        var buffer2 = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, buffer2);

        var coords = [];
        for (var i = 0; i < numVertices / 2; i++) {
            coords.push(0, 0, 1, 1);
        }
        ctx.bufferData(gl.ARRAY_BUFFER, new Float32Array(coords), gl.STATIC_DRAW);

        ctx.enableVertexAttribArray(coordLoc);
        ctx.vertexAttribPointer(coordLoc, 2, gl.FLOAT, false, 0, 0);

        ctx.drawArrays(gl.LINES, 0, numVertices);
        ctx.disableVertexAttribArray(posLoc);
        ctx.disableVertexAttribArray(coordLoc);
        ctx.bindBuffer(gl.ARRAY_BUFFER, null);
        ctx.deleteBuffer(buffer);
        ctx.deleteBuffer(buffer2);
        dst.readViewport(ctx, this.m_viewport);
};

/**
 * @constructor
 * @extends {es3fFboTestCase.FboTestCase}
 * @param {string} name
 * @param {string} desc
 * @param {Array<number>} viewport
 * @param {number} pointSize
 */
es3fClippingTests.PointsCase = function(name, desc, viewport, pointSize) {
    es3fFboTestCase.FboTestCase.call(this, name, desc);
    this.m_viewport = viewport;
    this.m_pointSize = pointSize;
};

setParentClass(es3fClippingTests.PointsCase, es3fFboTestCase.FboTestCase);

es3fClippingTests.PointsCase.prototype.compare = function(reference, result) {
    return tcuImageCompare.bilinearCompare('Result', 'Image comparison result',
        reference.getAccess(),
        result.getAccess(),
        tcuRGBA.newRGBAComponents(3, 3, 3, 3));
};

es3fClippingTests.PointsCase.prototype.render = function(dst) {
        var ctx = this.getCurrentContext();
        var x = this.m_viewport[0];
        var y = this.m_viewport[1];
        var width = this.m_viewport[2];
        var height = this.m_viewport[3];
        ctx.viewport(x, y, width, height);
        ctx.clearColor(0, 0, 0, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT);

        var shader = new es3fFboTestUtil.FlatColorShader(gluShaderUtil.DataType.FLOAT_VEC4, this.m_pointSize);
        var program = ctx.createProgram(shader);
        shader.setColor(ctx, program, [0, 1, 0, 1]);

        // positions
        var posLoc = ctx.getAttribLocation(program, 'a_position');
        if (posLoc == -1)
            throw new Error('a_position attribute is not defined.');

        var buffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, buffer);

        var positions = [
            // clipping in X axis
            -1 - 1 / width, -0.5, 0, 1,
            -1, 0, 0, 1,
            -1 + 1 / width, 0.5, 0, 1,
            1 + 1 / width, -0.5, 0, 1,
            1, 0, 0, 1,
            1 - 1 / width, 0.5, 0, 1,
            // clipping in Y axis
            -0.5, -1 - 1 / height, 0, 1,
            0, -1, 0, 1,
            0.5, -1 + 1 / height, 0, 1,
            -0.5, 1 - 1 / height, 0, 1,
            0, 1, 0, 1,
            0.5, 1 + 1 / height, 0, 1,
            // clipping in Z axis
            -0.5, -0.5, -1.5, 1,
            0, 0, 0, 1,
            0.5, 0.5, 1.5, 1
        ];
        // move the vertices to pixel centers to avoid off-by-1 differences
        for (var i = 0; i < positions.length; i += 4) {
            positions[i + 0] = center(positions[i + 0], width);
            positions[i + 1] = center(positions[i + 1], height);
        }
        // positions = [-1 + 3/width + 0.001, 1 + 1/height + 0.001, 0, 1];
        // positions = [-1, -1, 0, 1];

        var numVertices = positions.length / 4;

        ctx.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

        ctx.enableVertexAttribArray(posLoc);
        ctx.vertexAttribPointer(posLoc, 4, gl.FLOAT, false, 0, 0);

        ctx.drawArrays(gl.POINTS, 0, numVertices);
        ctx.disableVertexAttribArray(posLoc);
        ctx.bindBuffer(gl.ARRAY_BUFFER, null);
        ctx.deleteBuffer(buffer);
        dst.readViewport(ctx, this.m_viewport);
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fClippingTests.ClippingTests = function() {
    tcuTestCase.DeqpTest.call(this, 'clipping', 'Clipping tests');
};

es3fClippingTests.ClippingTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fClippingTests.ClippingTests.prototype.constructor = es3fClippingTests.ClippingTests;

es3fClippingTests.ClippingTests.prototype.init = function() {
    var width = gl.drawingBufferWidth;
    var height = gl.drawingBufferHeight;
    /** @const */ var WIDE_POINT = 5;
    /** @const */ var WIDE_LINE = 5;
    var viewports = [{ name: 'full_viewport', v: [0, 0, width, height] }, {
                       name: 'partial_viewport', v: [width * 0.3 , height * 0.2 , width * 0.6, height * 0.5] }
    ];
    var pointSizeRange = gl.getParameter(gl.ALIASED_POINT_SIZE_RANGE);
    var lineWidthRange = gl.getParameter(gl.ALIASED_LINE_WIDTH_RANGE);

    for (var i = 0; i < viewports.length; i++) {
        var v = viewports[i].v.map(Math.floor);
        var vName = viewports[i].name;
        this.addChild(new es3fClippingTests.LinesCase('narrow_lines_' + vName, 'lines', v, 1));
        if (lineWidthRange[1] >= WIDE_LINE)
            this.addChild(new es3fClippingTests.LinesCase('wide_lines_' + vName, 'lines', v, WIDE_LINE));
        this.addChild(new es3fClippingTests.PointsCase('small_points_' + vName, 'points', v, 1));
        if (pointSizeRange[1] >= WIDE_POINT)
            this.addChild(new es3fClippingTests.PointsCase('wide_points_' + vName, 'points', v, WIDE_POINT));
    }

    var rangesX = [
        [-1.2, 1.2],
        [-1.2, 0.8],
        [-0.8, 1.2]
    ];
    var rangesY = [
        [-1.2, 1.2],
        [-1.2, 0.8],
        [-0.8, 1.2]
    ];
    var rangesZ = [
        [-1.2, 1.2],
        [1.2, -1.2]
    ];
    for (var i = 0; i < viewports.length; i++) {
        var v = viewports[i].v.map(Math.floor);
        var vName = viewports[i].name;
        for (var x = 0; x < rangesX.length; x++)
        for (var y = 0; y < rangesY.length; y++)
        for (var z = 0; z < rangesZ.length; z++) {
            var rangeX = rangesX[x];
            var rangeY = rangesY[y];
            var rangeZ = rangesZ[z];
            var name = 'triangles_' + viewports[i].name + '_' +
                        '(' + rangeX[0] + ',' + rangeY[0] + ',' + rangeZ[0] + ')-' +
                        '(' + rangeX[1] + ',' + rangeY[1] + ',' + rangeZ[1] + ')';
            this.addChild(new es3fClippingTests.TriangleCase(name, 'triangles', v,
                            rangeX,
                            rangeY,
                            rangeZ));
        }
    }
};

/**
 * Run test
 * @param {WebGL2RenderingContext} context
 */
es3fClippingTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fClippingTests.ClippingTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fClippingTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

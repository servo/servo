/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

'use strict';
description("This test ensures clipping works with wide points whose centers are out of the viewport");

var wtu = WebGLTestUtils;
var gl = wtu.create3DContext("testbed", undefined, contextVersion);

var pointSize;

function setupProgram() {
    var vs = "attribute vec4 pos;" +
        "uniform float pointSize; " +
        "void main() {" +
        "  gl_PointSize = pointSize;" +
        "  gl_Position = pos;" +
        "}";
    var fs = "precision mediump float;" +
        "void main() {" +
        "  gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);" +
        "}";
    var program = wtu.setupProgram(gl, [vs, fs], ['pos']);
    if (program) {
        var loc = gl.getUniformLocation(program, 'pointSize');
        gl.uniform1f(loc, pointSize);
        gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);
        gl.enableVertexAttribArray(0);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Should be no errors after setting up program");
    }
    return program;
}

function runOneTestCase(vertex) {
    debug("");
    debug("testing point at (" + vertex[0] + ", " + vertex[1] + ", " + vertex[2] + ")");
    var data = new Float32Array(vertex);
    gl.bufferSubData(gl.ARRAY_BUFFER, 0, data);

    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.POINTS, 0, 1);
    wtu.checkCanvasRect(gl, 0, 0, 1, 1, [0, 255, 0]);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Should be no errors after running one test case");
}

function runTests() {
    if (!gl) {
        testFailed("context does not exist");
        return;
    }

    var range = gl.getParameter(gl.ALIASED_POINT_SIZE_RANGE);
    if (range[1] < 2.0) {
        testPassed("ALIASDED_POINT_SIZE_RANGE less than 2");
        return;
    }
    pointSize = 2.0;

    var data = new Float32Array(4);
    var buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);

    var program = setupProgram();
    if (!program) {
        testFailed("fail to set up program");
        return;
    }

    gl.disable(gl.BLEND);
    gl.disable(gl.DITHER);
    gl.disable(gl.DEPTH_TEST);

    gl.clearColor(1.0, 0.0, 0.0, 1.0);

    var vertices = [
        [ 0.99, 0.5, 0.0, 1.0 ],
        [ 1.01, 0.5, 0.0, 1.0 ],
        [ 0.5, 0.99, 0.0, 1.0 ],
        [ 0.5, 1.01, 0.0, 1.0 ],
    ];
    for (var idx = 0; idx < vertices.length; ++idx) {
        runOneTestCase(vertices[idx]);
    }
}

runTests();
debug("");
var successfullyParsed = true;

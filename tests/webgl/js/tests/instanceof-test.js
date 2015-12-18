/*
** Copyright (c) 2015 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/

// This test relies on the surrounding web page defining a variable
// "contextVersion" which indicates what version of WebGL it's running
// on -- 1 for WebGL 1.0, 2 for WebGL 2.0, etc.

"use strict";
var wtu = WebGLTestUtils;
description(document.title);
debug("Tests that instanceof works on WebGL objects.");
debug("");

var gl = wtu.create3DContext("canvas");
if (contextVersion === 1) {
  shouldBeTrue('gl instanceof WebGLRenderingContext');
} else if (contextVersion === 2) {
  shouldBeTrue('gl instanceof WebGL2RenderingContext');
}

shouldBeTrue('gl.createBuffer() instanceof WebGLBuffer');
shouldBeTrue('gl.createFramebuffer() instanceof WebGLFramebuffer');
shouldBeTrue('gl.createProgram() instanceof WebGLProgram');
shouldBeTrue('gl.createRenderbuffer() instanceof WebGLRenderbuffer');
shouldBeTrue('gl.createShader(gl.VERTEX_SHADER) instanceof WebGLShader');
shouldBeTrue('gl.createTexture() instanceof WebGLTexture');
if (contextVersion > 1) {
  shouldBeTrue('gl.createQuery() instanceof WebGLQuery');
  shouldBeTrue('gl.createSampler() instanceof WebGLSampler');
  shouldBeTrue('gl.createTransformFeedback() instanceof WebGLTransformFeedback');
  shouldBeTrue('gl.createVertexArray() instanceof WebGLVertexArrayObject');
}

var program = wtu.setupProgram(gl, ['vshader', 'fshader'], ['vPosition'], [0]);

shouldBeTrue('gl.getUniformLocation(program, "color") instanceof WebGLUniformLocation');
shouldBeTrue('gl.getActiveAttrib(program, 0) instanceof WebGLActiveInfo');
shouldBeTrue('gl.getActiveUniform(program, 0) instanceof WebGLActiveInfo');

debug("");
debug("Tests that those WebGL objects can not be constructed through new operator");
debug("");

function shouldThrowWithNew(objectType, objectName) {
  try {
    new objectType;
    testFailed('new ' + objectName + ' did not throw');
  } catch (e) {
    testPassed('new ' + objectName + ' threw an error');
  }
}

shouldThrowWithNew(window.WebGLRenderingContext, 'WebGLRenderingContext');
shouldThrowWithNew(window.WebGLActiveInfo, 'WebGLActiveInfo');
shouldThrowWithNew(window.WebGLBuffer, 'WebGLBuffer');
shouldThrowWithNew(window.WebGLFramebuffer, 'WebGLFramebuffer');
shouldThrowWithNew(window.WebGLProgram, 'WebGLProgram');
shouldThrowWithNew(window.WebGLRenderbuffer, 'WebGLRenderbuffer');
shouldThrowWithNew(window.WebGLShader, 'WebGLShader');
shouldThrowWithNew(window.WebGLTexture, 'WebGLTexture');
shouldThrowWithNew(window.WebGLUniformLocation, 'WebGLUniformLocation');
shouldThrowWithNew(window.WebGLShaderPrecisionFormat, 'WebGLShaderPrecisionFormat');
if (contextVersion > 1) {
  shouldThrowWithNew(window.WebGLQuery, 'WebGLQuery');
  shouldThrowWithNew(window.WebGLSampler, 'WebGLSampler');
  shouldThrowWithNew(window.WebGLSync, 'WebGLSync');
  shouldThrowWithNew(window.WebGLTransformFeedback, 'WebGLTransformFeedback');
  shouldThrowWithNew(window.WebGLVertexArrayObject, 'WebGLVertexArrayObject');
}

var successfullyParsed = true;

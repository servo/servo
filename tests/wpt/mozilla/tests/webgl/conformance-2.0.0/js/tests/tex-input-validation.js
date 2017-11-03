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
description("Validate tex functions input parameters");

var wtu = WebGLTestUtils;
var gl = null;
var tex = null;
var error = 0;

shouldBeNonNull("gl = wtu.create3DContext(undefined, undefined, contextVersion)");
shouldBeNonNull("tex = gl.createTexture()");
gl.bindTexture(gl.TEXTURE_2D, tex);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

function enumToString(value) {
  return wtu.glEnumToString(gl, value);
}

function testTexParameter(testCase) {
  var msg = "paramName: " + enumToString(testCase.pname);
  error = testCase.expectedError;
  gl.texParameteri(testCase.target, testCase.pname, testCase.param);
  wtu.glErrorShouldBe(gl, error, msg);
  gl.texParameterf(testCase.target, testCase.pname, testCase.param);
  wtu.glErrorShouldBe(gl, error, msg);
}

function testGetTexParameter(testCase) {
  var msg = "paramName: " + enumToString(testCase.pname);
  error = testCase.expectedError;
  gl.getTexParameter(testCase.target, testCase.pname);
  wtu.glErrorShouldBe(gl, error, msg);
}

function testTexImage2D(testCase) {
  var level = 0;
  var width = 16;
  var height = 16;
  var msg = " internalFormat: " + enumToString(testCase.internalFormat) +
            " target: " + enumToString(testCase.target) +
            " format: " + enumToString(testCase.format) +
            " type: " + enumToString(testCase.type) +
            " border: " + testCase.border;

  gl.texImage2D(testCase.target, level, testCase.internalFormat, width, height, testCase.border, testCase.format, testCase.type, null);
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}

function testTexSubImage2D(testCase) {
  var level = 0;
  var xoffset = 0;
  var yoffset = 0;
  var width = 16;
  var height = 16;
  var msg = " format: " + enumToString(testCase.format) +
            " type: " + enumToString(testCase.type);
  var array = new Uint8Array(width * height * 4);

  gl.texSubImage2D(testCase.target, level, xoffset, yoffset, width, height, testCase.format, testCase.type, array);
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}

function testCopyTexImage2D(testCase) {
  var level = 0;
  var x = 0;
  var y = 0;
  var width = 16;
  var height = 16;
  var msg = " colorBufferFormat: " + enumToString(testCase.colorBufferFormat) +
            " internalFormat: " + enumToString(testCase.internalFormat) +
            " target: " + enumToString(testCase.target) +
            " border: " + testCase.border;

  gl.renderbufferStorage(gl.RENDERBUFFER, testCase.colorBufferFormat, width, height);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  shouldBe("gl.checkFramebufferStatus(gl.FRAMEBUFFER)", "gl.FRAMEBUFFER_COMPLETE");

  gl.copyTexImage2D(testCase.target, level, testCase.internalFormat, x, y, width, height, testCase.border);
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}

function testCopyTexSubImage2D(testCase) {
  var level = 0;
  var x = 0;
  var y = 0;
  var width = 16;
  var height = 16;
  var xoffset = 0;
  var yoffset = 0;
  var border = 0;
  var type = gl.UNSIGNED_BYTE;
  var msg = " colorBufferFormat: " + enumToString(testCase.colorBufferFormat) +
            " internalFormat: " + enumToString(testCase.internalFormat) +
            " target: " + enumToString(testCase.target);

  gl.renderbufferStorage(gl.RENDERBUFFER, testCase.colorBufferFormat, width, height);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  shouldBe("gl.checkFramebufferStatus(gl.FRAMEBUFFER)", "gl.FRAMEBUFFER_COMPLETE");

  gl.texImage2D(testCase.target, level, testCase.internalFormat, xoffset + width, yoffset + height, border, testCase.internalFormat, type, null);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);

  gl.copyTexSubImage2D(testCase.target, level, xoffset, yoffset, x, y, width, height);
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}

function testCopyFromInternalFBO(testCase) {
  var target = gl.TEXTURE_2D;
  var level = 0;
  var x = 0;
  var y = 0;
  var width = 16;
  var height = 16;
  var xoffset = 0;
  var yoffset = 0;
  var border = 0;
  var type = gl.UNSIGNED_BYTE;
  var msg = " colorBufferFormat: " + enumToString(testCase.contextAlpha ? gl.RGBA : gl.RGB) +
            " internalFormat: " + enumToString(testCase.internalFormat);

  if (testCase.contextAlpha) {
    gl = wtu.create3DContext(null, { alpha: true }, contextVersion);
  } else {
    gl = wtu.create3DContext(null, { alpha: false }, contextVersion);
  }
  shouldBeNonNull("gl");
  shouldBeNonNull("tex = gl.createTexture()");
  gl.bindTexture(target, tex);
  if (testCase.subImage) {
    gl.texImage2D(target, level, testCase.internalFormat, xoffset + width, yoffset + height, border, testCase.internalFormat, type, null);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR);
    gl.copyTexSubImage2D(target, level, xoffset, yoffset, x, y, width, height);
  } else {
    wtu.glErrorShouldBe(gl, gl.NO_ERROR);
    gl.copyTexImage2D(target, level, testCase.internalFormat, x, y, width, height, border);
  }
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}

// Only for WebGL2.0.
function testTexImage3D(testCase) {
  var level = 0;
  var width = 16;
  var height = 16;
  var depth = 16;
  var msg = " internalFormat: " + enumToString(testCase.internalFormat) +
            " target: " + enumToString(testCase.target) +
            " format: " + enumToString(testCase.format) +
            " type: " + enumToString(testCase.type) +
            " border: " + testCase.border;

  gl.texImage3D(testCase.target, level, testCase.internalFormat, width, height, depth, testCase.border, testCase.format, testCase.type, null);
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}

function testTexSubImage3D(testCase) {
  var level = 0;
  var xoffset = 0;
  var yoffset = 0;
  var zoffset = 0;
  var width = 16;
  var height = 16;
  var depth = 16;
  var msg = " format: " + enumToString(testCase.format) +
            " type: " + enumToString(testCase.type);
  var array = new Uint8Array(width * height * depth * 4);

  gl.texSubImage3D(testCase.target, level, xoffset, yoffset, zoffset, width, height, depth, testCase.format, testCase.type, array);
  error = testCase.expectedError;
  wtu.glErrorShouldBe(gl, error, msg);
}


// Start checking.

debug("");
debug("Checking TexParameter: a set of inputs that are valid in GL but invalid in WebGL");

testCases = [
  { target: 0x0DE0, // GL_TEXTURE_1D
    pname: gl.TEXTURE_WRAP_T,
    param: gl.REPEAT,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_2D,
    pname: gl.TEXTURE_WRAP_T,
    param: 0x2900, // GL_CLAMP
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_2D,
    pname: gl.TEXTURE_WRAP_T,
    param: gl.REPEAT,
    expectedError: gl.NO_ERROR }
];

if (contextVersion < 2) {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      pname: 0x813A, // GL_TEXTURE_MIN_LOD
      param: 0,
      expectedError: gl.INVALID_ENUM }
  ]);
} else {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      pname: 0x8E42, // GL_TEXTURE_SWIZZLE_R
      param: 0x1903, // GL_RED
      expectedError: gl.INVALID_ENUM },
    { target: gl.TEXTURE_2D,
      pname: 0x8072, // GL_TEXTURE_WRAP_R
      param: 0x2900, // GL_CLAMP
      expectedError: gl.INVALID_ENUM }
  ]);
}

for (var ii = 0; ii < testCases.length; ++ii) {
  testTexParameter(testCases[ii]);
}

debug("");
debug("Checking GetTexParameter: a set of inputs that are valid in GL but invalid in WebGL");

testCases = [
  { target: 0x0DE0, // GL_TEXTURE_1D
    pname: gl.TEXTURE_WRAP_T,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_2D,
    pname: gl.TEXTURE_WRAP_T,
    expectedError: gl.NO_ERROR }
];

if (contextVersion < 2) {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      pname: 0x813A, // GL_TEXTURE_MIN_LOD
      expectedError: gl.INVALID_ENUM }
  ]);
} else {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      pname: 0x8E42, // GL_TEXTURE_SWIZZLE_R
      expectedError: gl.INVALID_ENUM }
  ]);
}

for (var ii = 0; ii < testCases.length; ++ii) {
  testGetTexParameter(testCases[ii]);
}

debug("");
debug("Checking TexImage2D: a set of inputs that are valid in GL but invalid in WebGL");

var testCases = [
  { target: 0x8064, // GL_PROXY_TEXTURE_2D
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_2D,
    internalFormat: 0x1903, // GL_RED
    border: 0,
    format: 0x1903, // GL_RED
    type: gl.UNSIGNED_BYTE,
    expectedError: [gl.INVALID_ENUM, gl.INVALID_VALUE] },
  { target: gl.TEXTURE_2D,
    internalFormat: gl.RGBA,
    border: 1,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_VALUE },
  { target: gl.TEXTURE_2D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGB,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_OPERATION },
  { target: gl.TEXTURE_2D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.NO_ERROR }
];

if (contextVersion < 2) {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      internalFormat: gl.RGBA,
      border: 0,
      format: gl.RGBA,
      type: gl.BYTE,
      expectedError: gl.INVALID_ENUM }
  ]);
} else {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      internalFormat: gl.RGBA,
      border: 0,
      format: gl.RGBA,
      type: gl.BYTE,
      expectedError: gl.INVALID_OPERATION },
    { target: gl.TEXTURE_3D,
      internalFormat: gl.RGBA,
      border: 0,
      format: gl.RGBA,
      type: gl.UNSIGNED_BYTE,
      expectedError: gl.INVALID_ENUM }
  ]);
}

for (var ii = 0; ii < testCases.length; ++ii) {
  testTexImage2D(testCases[ii]);
}

debug("");
debug("Checking TexSubImage2D: a set of inputs that are valid in GL but invalid in WebGL");

testCases = [
  { target: gl.TEXTURE_2D,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.NO_ERROR }
];

if (contextVersion < 2) {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      format: 0x1903, // GL_RED
      type: gl.UNSIGNED_BYTE,
      expectedError: gl.INVALID_ENUM },
    { target: gl.TEXTURE_2D,
      format: gl.RGBA,
      type: gl.BYTE,
      expectedError: gl.INVALID_ENUM }
  ]);
} else {
  testCases = testCases.concat([
    { target: gl.TEXTURE_2D,
      format: gl.RED,
      type: gl.UNSIGNED_BYTE,
      expectedError: gl.INVALID_OPERATION },
    { target: gl.TEXTURE_2D,
      format: gl.RGBA,
      type: gl.BYTE,
      expectedError: gl.INVALID_OPERATION },
    { target: gl.TEXTURE_3D,
      format: gl.RGBA,
      type: gl.UNSIGNED_BYTE,
      expectedError: gl.INVALID_ENUM },
  ]);
}

for (var ii = 0; ii < testCases.length; ++ii) {
  testTexSubImage2D(testCases[ii]);
}

debug("");
debug("Checking CopyTexImage2D: a set of inputs that are valid in GL but invalid in WebGL");

var colorBuffer = null;
var fbo = null;

shouldBeNonNull("fbo = gl.createFramebuffer()");
gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
shouldBeNonNull("colorBuffer = gl.createRenderbuffer()");
gl.bindRenderbuffer(gl.RENDERBUFFER, colorBuffer);
gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, colorBuffer);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

testCases = [
  { target: gl.TEXTURE_2D,
    colorBufferFormat: gl.RGB565,
    internalFormat: 0x8054, // GL_RGB16
    border: 0,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_2D,
    colorBufferFormat: gl.RGB565,
    internalFormat: gl.RGBA,
    border: 1,
    expectedError: gl.INVALID_VALUE },
  { target: gl.TEXTURE_2D,
    colorBufferFormat: gl.RGB565,
    internalFormat: gl.RGBA,
    border: 0,
    expectedError: gl.INVALID_OPERATION },
  { target: gl.TEXTURE_2D,
    colorBufferFormat: gl.RGB565,
    internalFormat: gl.RGB,
    border: 0,
    expectedError: gl.NO_ERROR }
];

if (contextVersion > 1) {
  testCases = testCases.concat([
    { target: gl.TEXTURE_3D,
      colorBufferFormat: gl.RGB5_A1,
      internalFormat: gl.RGBA,
      border: 0,
      expectedError: gl.INVALID_ENUM }
  ]);
}

for (var ii = 0; ii < testCases.length; ++ii) {
  testCopyTexImage2D(testCases[ii]);
}

debug("");
debug("Checking CopyTexSubImage2D: a set of inputs that are valid in GL but invalid in WebGL");

testCases = [
  { target: gl.TEXTURE_2D,
    colorBufferFormat: gl.RGB5_A1,
    internalFormat: gl.RGBA,
    expectedError: gl.NO_ERROR },
  { target: gl.TEXTURE_2D,
    colorBufferFormat: gl.RGB565,
    internalFormat: gl.RGBA,
    expectedError: gl.INVALID_OPERATION }
];

for (var ii = 0; ii < testCases.length; ++ii) {
  testCopyTexSubImage2D(testCases[ii]);
}

debug("");
debug("Checking CopyTex{Sub}Image2D: copy from WebGL internal framebuffer");

testCases = [
  { contextAlpha: true,
    internalFormat: gl.RGBA,
    subImage: false,
    expectedError: gl.NO_ERROR },
  { contextAlpha: false,
    internalFormat: gl.RGBA,
    subImage: false,
    expectedError: gl.INVALID_OPERATION },
  { contextAlpha: true,
    internalFormat: gl.RGBA,
    subImage: true,
    expectedError: gl.NO_ERROR },
  { contextAlpha: false,
    internalFormat: gl.RGBA,
    subImage: true,
    expectedError: gl.INVALID_OPERATION }
];

for (var ii = 0; ii < testCases.length; ++ii) {
  testCopyFromInternalFBO(testCases[ii]);
}

if (contextVersion > 1) {
// Create new texture for testing api of WebGL 2.0.
shouldBeNonNull("tex = gl.createTexture()");
gl.bindTexture(gl.TEXTURE_3D, tex);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("Checking TexImage3D: a set of inputs that are valid in GL but invalid in WebGL");

var testCases = [
  { target: 0x8070, // GL_PROXY_TEXTURE_3D
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_3D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGB,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_OPERATION },
  { target: gl.TEXTURE_3D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.BYTE,
    expectedError: gl.INVALID_OPERATION},
  { target: gl.TEXTURE_3D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.NO_ERROR }
];

for (var ii = 0; ii < testCases.length; ++ii) {
  testTexImage3D(testCases[ii]);
}

debug("");
debug("Checking TexImage3D: bad target, internalformats, formats, types");

var testCases = [
  { target: gl.TEXTURE_2D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_3D,
    internalFormat: gl.RG,
    border: 0,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_VALUE},
  { target: gl.TEXTURE_3D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RG8,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_3D,
    internalFormat: gl.RGBA,
    border: 0,
    format: gl.RGBA,
    type: gl.INT,
    expectedError: gl.INVALID_OPERATION},
];

for (var ii = 0; ii < testCases.length; ++ii) {
  testTexImage3D(testCases[ii]);
}

debug("");
debug("Checking TexSubImage3D: a set of inputs that are valid in GL but invalid in WebGL");

testCases = [
  { target: gl.TEXTURE_3D,
    format: 0x80E0, // GL_BGR
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_3D,
    format: gl.RGBA,
    type: 0x8032, // GL_UNSIGNED_BYTE_3_3_2
    expectedError: gl.INVALID_ENUM },
  { target: gl.TEXTURE_3D,
    format: gl.RGBA,
    type: gl.UNSIGNED_BYTE,
    expectedError: gl.NO_ERROR }
];

for (var ii = 0; ii < testCases.length; ++ii) {
  testTexSubImage3D(testCases[ii]);
}

}

var successfullyParsed = true;

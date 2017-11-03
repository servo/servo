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
description("This test ensures various WebGL functions fail when passed invalid OpenGL ES enums.");

debug("");
debug("Canvas.getContext");

var wtu = WebGLTestUtils;
var gl = wtu.create3DContext("canvas", undefined, contextVersion);
if (!gl) {
  testFailed("context does not exist");
} else {
  testPassed("context exists");

  debug("");
  debug("Checking gl enums.");

  var buffer = new ArrayBuffer(2);
  var buf = new Uint16Array(buffer);
  var tex = gl.createTexture();
  var program = wtu.createProgram(gl, wtu.loadStandardVertexShader(gl), wtu.loadStandardFragmentShader(gl));
  gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);

  var tests = [
    "gl.disable(desktopGL['CLIP_PLANE0'])",
    "gl.disable(desktopGL['POINT_SPRITE'])",
    "gl.getBufferParameter(gl.ARRAY_BUFFER, desktopGL['PIXEL_PACK_BUFFER'])",
    "gl.hint(desktopGL['PERSPECTIVE_CORRECTION_HINT'], gl.FASTEST)",
    "gl.isEnabled(desktopGL['CLIP_PLANE0'])",
    "gl.isEnabled(desktopGL['POINT_SPRITE'])",
    "gl.pixelStorei(desktopGL['PACK_SWAP_BYTES'], 1)",
    "gl.getParameter(desktopGL['NUM_COMPRESSED_TEXTURE_FORMATS'])",
    "gl.getParameter(desktopGL['EXTENSIONS'])",
    "gl.getParameter(desktopGL['SHADER_COMPILER'])",
    "gl.getParameter(desktopGL['SHADER_BINARY_FORMATS'])",
    "gl.getParameter(desktopGL['NUM_SHADER_BINARY_FORMATS'])",
  ];

  if (contextVersion < 2) {
    tests = tests.concat([
      "gl.blendEquation(desktopGL['MIN'])",
      "gl.blendEquation(desktopGL['MAX'])",
      "gl.blendEquationSeparate(desktopGL['MIN'], gl.FUNC_ADD)",
      "gl.blendEquationSeparate(desktopGL['MAX'], gl.FUNC_ADD)",
      "gl.blendEquationSeparate(gl.FUNC_ADD, desktopGL['MIN'])",
      "gl.blendEquationSeparate(gl.FUNC_ADD, desktopGL['MAX'])",
      "gl.bufferData(gl.ARRAY_BUFFER, 16, desktopGL['STREAM_READ'])",
      "gl.bufferData(gl.ARRAY_BUFFER, 16, desktopGL['STREAM_COPY'])",
      "gl.bufferData(gl.ARRAY_BUFFER, 16, desktopGL['STATIC_READ'])",
      "gl.bufferData(gl.ARRAY_BUFFER, 16, desktopGL['STATIC_COPY'])",
      "gl.bufferData(gl.ARRAY_BUFFER, 16, desktopGL['DYNAMIC_READ'])",
      "gl.bufferData(gl.ARRAY_BUFFER, 16, desktopGL['DYNAMIC_COPY'])",
      "gl.bindTexture(desktopGL['TEXTURE_2D_ARRAY'], tex)",
      "gl.bindTexture(desktopGL['TEXTURE_3D'], tex)",
    ]);
  } else {
    tests = tests.concat([
      "gl.bindTexture(desktopGL['TEXTURE_RECTANGLE_EXT'], tex)",
      "gl.enable(desktopGL['PRIMITIVE_RESTART_FIXED_INDEX'])",
      "gl.getActiveUniforms(program, [0], desktopGL['UNIFORM_NAME_LENGTH'])",
      "gl.getProgramParameter(program, desktopGL['ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH'])",
      "gl.getProgramParameter(program, desktopGL['TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH'])",
      "gl.getProgramParameter(program, desktopGL['PROGRAM_BINARY_RETRIEVABLE_HINT'])",
      "gl.getProgramParameter(program, desktopGL['PROGRAM_BINARY_LENGTH'])",
      "gl.getParameter(program, desktopGL['NUM_PROGRAM_BINARY_FORMATS'])",
    ]);
  }

  for (var ii = 0; ii < tests.length; ++ii) {
    TestEval(tests[ii]);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, tests[ii] + " should return INVALID_ENUM.");
  }

  gl.bindTexture(gl.TEXTURE_2D, tex);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);

  tests = [
    "gl.getTexParameter(gl.TEXTURE_2D, desktopGL['GENERATE_MIPMAP'])",
    "gl.texParameteri(gl.TEXTURE_2D, desktopGL['GENERATE_MIPMAP'], 1)",
    "gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, desktopGL['CLAMP_TO_BORDER'])",
  ];

  if (contextVersion < 2) {
    tests = tests.concat([
      "gl.texParameteri(desktopGL['TEXTURE_2D_ARRAY'], gl.TEXTURE_MAG_FILTER, gl.NEAREST)",
      "gl.texParameteri(desktopGL['TEXTURE_3D'], gl.TEXTURE_MAG_FILTER, gl.NEAREST)",
    ]);
  } else {
    tests = tests.concat([
      "gl.texParameteri(desktopGL['TEXTURE_2D'], desktopGL['TEXTURE_SWIZZLE_R_EXT'], gl.RED)",
      "gl.texParameteri(desktopGL['TEXTURE_2D'], desktopGL['TEXTURE_SWIZZLE_G_EXT'], gl.RED)",
      "gl.texParameteri(desktopGL['TEXTURE_2D'], desktopGL['TEXTURE_SWIZZLE_B_EXT'], gl.RED)",
      "gl.texParameteri(desktopGL['TEXTURE_2D'], desktopGL['TEXTURE_SWIZZLE_A_EXT'], gl.RED)",
      "gl.texParameteri(desktopGL['TEXTURE_2D'], gl.TEXTURE_WRAP_R, desktopGL['CLAMP_TO_BORDER'])",
    ]);
  }

  for (var ii = 0; ii < tests.length; ++ii) {
    TestEval(tests[ii]);
    wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, tests[ii] + " should return INVALID_ENUM.");
  }
  if (contextVersion >= 2) {
    var uniformBlockProgram = wtu.loadUniformBlockProgram(gl);
    gl.linkProgram(uniformBlockProgram);
    shouldBe('gl.getProgramParameter(uniformBlockProgram, gl.LINK_STATUS)', 'true');
    shouldBe('gl.getError()', 'gl.NO_ERROR');
    gl.getActiveUniformBlockParameter(uniformBlockProgram, 0, desktopGL['UNIFORM_BLOCK_NAME_LENGTH']);
    shouldBe('gl.getError()', 'gl.INVALID_ENUM');
  }
}

debug("");
var successfullyParsed = true;

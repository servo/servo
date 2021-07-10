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
description("Test of get calls against GL objects like getBufferParameter, etc.");

var gl = wtu.create3DContext(undefined, undefined, contextVersion);

function testInvalidArgument(funcName, argumentName, validArgumentArray, func) {
  var validArguments = {};
  for (var ii = 0; ii < validArgumentArray.length; ++ii) {
    validArguments[validArgumentArray[ii]] = true;
  }
  var success = true;
  for (var ii = 0; ii < 0x10000; ++ii) {
    if (!validArguments[ii]) {
      var result = func(ii);
      if (result !== null) {
        success = false;
        testFailed(funcName + " returned " + result + " instead of null for invalid " + argumentName + " enum: " + wtu.glEnumToString(gl, ii));
        break;
      }
      var err = gl.getError();
      if (err != gl.INVALID_ENUM) {
        success = false;
        testFailed(funcName + " did not generate INVALID_ENUM for invalid " + argumentName + " enum: " + wtu.glEnumToString(gl, ii));
        break;
      }
    }
  }
  if (success) {
    testPassed(funcName + " correctly handled invalid " + argumentName + " enums");
  }
}

debug("");
debug("test getBufferParameter");
// Test getBufferParameter
var bufferTypes = [gl.ARRAY_BUFFER, gl.ELEMENT_ARRAY_BUFFER];
if (contextVersion > 1) {
  bufferTypes = bufferTypes.concat([gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, gl.PIXEL_PACK_BUFFER, gl.PIXEL_UNPACK_BUFFER, gl.TRANSFORM_FEEDBACK_BUFFER, gl.UNIFORM_BUFFER]);
}
for (var bb = 0; bb < bufferTypes.length; ++bb) {
  var bufferType = bufferTypes[bb];
  var buffer = gl.createBuffer();
  gl.bindBuffer(bufferType, buffer);
  gl.bufferData(bufferType, 16, gl.DYNAMIC_DRAW);
  var expression1 = "gl.getBufferParameter(gl." + wtu.glEnumToString(gl, bufferType) + ", gl.BUFFER_SIZE)";
  var expression2 = "gl.getBufferParameter(gl." + wtu.glEnumToString(gl, bufferType) + ", gl.BUFFER_USAGE)";
  shouldBe(expression1, '16');
  shouldBe(expression2, 'gl.DYNAMIC_DRAW');
  testInvalidArgument("getBufferParameter", "parameter", [gl.BUFFER_SIZE, gl.BUFFER_USAGE], function(bufferType) {
    return function(parameter) {
      return gl.getBufferParameter(bufferType, parameter);
    };
  }(bufferType));
  gl.bindBuffer(bufferType, null);
}
testInvalidArgument(
    "getBufferParameter",
    "target",
    bufferTypes,
    function(target) {
      return gl.getBufferParameter(target, gl.BUFFER_SIZE);
    }
);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

var testCases = [
  { contextStencil: true},
  { contextStencil: false}
];

for (var run = 0; run < testCases.length; ++run) {
  debug("");
  debug("Test getFramebufferAttachmentParameter with stencil " + testCases[run].contextStencil);

  if (testCases[run].contextStencil) {
    gl = wtu.create3DContext(null, {stencil: true}, contextVersion);
  } else {
    gl = wtu.create3DContext(null, {stencil: false}, contextVersion);
  }

  var texture = gl.createTexture();
  var anotherTexture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE,
                new Uint8Array([
                    0, 0, 0, 255,
                    255, 255, 255, 255,
                    255, 255, 255, 255,
                    0, 0, 0, 255]));
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.bindTexture(gl.TEXTURE_2D, null);
  var framebuffer = gl.createFramebuffer();
  gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
  gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
  var colorAttachmentsNum = 1;
  if (contextVersion > 1) {
    gl.bindTexture(gl.TEXTURE_2D, anotherTexture);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 2, 2, 0, gl.RGBA, gl.UNSIGNED_BYTE,
                  new Uint8Array([
                      0, 0, 0, 255,
                      255, 255, 255, 255,
                      255, 255, 255, 255,
                      0, 0, 0, 255]));
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.bindTexture(gl.TEXTURE_2D, null);
    colorAttachmentsNum = gl.getParameter(gl.MAX_COLOR_ATTACHMENTS);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + colorAttachmentsNum - 1, gl.TEXTURE_2D, anotherTexture, 0);
  }
  var renderbuffer = gl.createRenderbuffer();
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  if (contextVersion == 1)
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, 2, 2);
  else
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH24_STENCIL8, 2, 2);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, renderbuffer);
  if (contextVersion > 1)
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, renderbuffer);
  shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');
  // The for loop tests two color attachments for WebGL 2: the first one (gl.COLOR_ATTACHMENT0)
  // and the last one (gl.COLOR_ATTACHMENT0 + gl.MAX_COLOR_ATTACHMENTS - 1).
  for (var ii = 0; ii < colorAttachmentsNum; ii += (colorAttachmentsNum > 1 ? colorAttachmentsNum - 1 : 1)) {
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.TEXTURE');
    if (ii == 0)
      shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME)', 'texture');
    else
      shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME)', 'anotherTexture');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL)', '0');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE)', '0');
    if (contextVersion > 1) {
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_RED_SIZE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
      shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0 + ' + ii + ', gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER)', '0');
    }
  }
  shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.RENDERBUFFER');
  shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME)', 'renderbuffer');
  if (contextVersion > 1) {
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.RENDERBUFFER');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME)', 'renderbuffer');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.RENDERBUFFER');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME)', 'renderbuffer');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
    wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION, 'gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
  }
  var validParametersForFBAttachment =
      [ gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE,
        gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME,
        gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL,
        gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE
      ];
  if (contextVersion > 1) {
    validParametersForFBAttachment = validParametersForFBAttachment.concat([
        gl.FRAMEBUFFER_ATTACHMENT_RED_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE,
        gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING,
        gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER
    ]);
  }
  testInvalidArgument(
      "getFramebufferAttachmentParameter",
      "parameter",
      validParametersForFBAttachment,
      function(parameter) {
        return gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT, parameter);
      }
  );
  var validTargetsForFBAttachment = [gl.FRAMEBUFFER];
  if (contextVersion > 1) {
    validTargetsForFBAttachment = validTargetsForFBAttachment.concat([gl.READ_FRAMEBUFFER, gl.DRAW_FRAMEBUFFER]);
  }
  testInvalidArgument(
      "getFramebufferAttachmentParameter",
      "target",
      validTargetsForFBAttachment,
      function(target) {
        return gl.getFramebufferAttachmentParameter(target, gl.COLOR_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
      }
  );
  var validAttachmentsForFBAttachment = new Array(
      gl.COLOR_ATTACHMENT0,
      gl.DEPTH_ATTACHMENT,
      gl.STENCIL_ATTACHMENT,
      gl.DEPTH_STENCIL_ATTACHMENT
  );
  if (contextVersion > 1) {
    for (var ii = 1; ii < gl.getParameter(gl.MAX_COLOR_ATTACHMENTS); ++ii) {
      validAttachmentsForFBAttachment[validAttachmentsForFBAttachment.length] = gl.COLOR_ATTACHMENT0 + ii;
    }
  }
  testInvalidArgument(
      "getFramebufferAttachmentParameter",
      "attachment",
      validAttachmentsForFBAttachment,
      function(attachment) {
        return gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, attachment, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
      }
  );
  if (contextVersion > 1) {
    // test default framebuffer
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    shouldBe('gl.checkFramebufferStatus(gl.FRAMEBUFFER)', 'gl.FRAMEBUFFER_COMPLETE');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.FRAMEBUFFER_DEFAULT');
    shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.FRAMEBUFFER_DEFAULT');
    if (testCases[run].contextStencil)
      shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.FRAMEBUFFER_DEFAULT');
    else
      shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.NONE');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_RED_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH, gl.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
    shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
    if (testCases[run].contextStencil) {
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
      shouldBeNonZero('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
      wtu.glErrorShouldBe(gl, gl.NO_ERROR);
    } else {
      wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION, 'gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE)');
      wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION, 'gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE)');
      wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION, 'gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.STENCIL, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING)');
    }
    testInvalidArgument(
        "getFramebufferAttachmentParameter",
        "parameter",
        validParametersForFBAttachment,
        function(parameter) {
          return gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, parameter);
        }
    );
    testInvalidArgument(
        "getFramebufferAttachmentParameter",
        "target",
        validTargetsForFBAttachment,
        function(target) {
          return gl.getFramebufferAttachmentParameter(target, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
        }
    );
    testInvalidArgument(
        "getFramebufferAttachmentParameter",
        "attachment",
        [ gl.BACK,
          gl.DEPTH,
          gl.STENCIL
        ],
        function(attachment) {
          return gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, attachment, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
        }
    );
  }
}
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("test getAttachedShaders");
var standardVert = wtu.loadStandardVertexShader(gl);
var standardFrag = wtu.loadStandardFragmentShader(gl);
var standardProgram = gl.createProgram();
gl.attachShader(standardProgram, standardVert);
gl.attachShader(standardProgram, standardFrag);
gl.linkProgram(standardProgram);
var shaders = gl.getAttachedShaders(standardProgram);
shouldBe('shaders.length', '2');
shouldBeTrue('shaders[0] == standardVert && shaders[1] == standardFrag || shaders[1] == standardVert && shaders[0] == standardFrag');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldThrow('gl.getAttachedShaders(null)');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldThrow('gl.getAttachedShaders(standardVert)');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("Test getProgramParameter");
shouldBe('gl.getProgramParameter(standardProgram, gl.DELETE_STATUS)', 'false');
shouldBe('gl.getProgramParameter(standardProgram, gl.LINK_STATUS)', 'true');
shouldBe('typeof gl.getProgramParameter(standardProgram, gl.VALIDATE_STATUS)', '"boolean"');
shouldBe('gl.getProgramParameter(standardProgram, gl.ATTACHED_SHADERS)', '2');
shouldBe('gl.getProgramParameter(standardProgram, gl.ACTIVE_ATTRIBUTES)', '2');
shouldBe('gl.getProgramParameter(standardProgram, gl.ACTIVE_UNIFORMS)', '1');
if (contextVersion > 1) {
  var buffer = gl.createBuffer();
  gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buffer);
  gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 1024, gl.DYNAMIC_DRAW);
  var uniformBlockProgram = wtu.loadUniformBlockProgram(gl);
  var transformFeedbackVars = ["normal", "ecPosition"];
  gl.transformFeedbackVaryings(uniformBlockProgram, transformFeedbackVars, gl.INTERLEAVED_ATTRIBS);
  gl.linkProgram(uniformBlockProgram);
  shouldBe('gl.getProgramParameter(uniformBlockProgram, gl.LINK_STATUS)', 'true');
  shouldBe('gl.getError()', 'gl.NO_ERROR');
  shouldBe('gl.getProgramParameter(uniformBlockProgram, gl.ACTIVE_UNIFORM_BLOCKS)', '1');
  shouldBe('gl.getProgramParameter(uniformBlockProgram, gl.TRANSFORM_FEEDBACK_VARYINGS)', '2');
  shouldBe('gl.getProgramParameter(uniformBlockProgram, gl.TRANSFORM_FEEDBACK_BUFFER_MODE)', 'gl.INTERLEAVED_ATTRIBS');
}
var program = standardProgram;
var validArrayForProgramParameter = [
    gl.DELETE_STATUS,
    gl.LINK_STATUS,
    gl.VALIDATE_STATUS,
    gl.ATTACHED_SHADERS,
    gl.ACTIVE_ATTRIBUTES,
    gl.ACTIVE_UNIFORMS
];
if (contextVersion > 1) {
  validArrayForProgramParameter = validArrayForProgramParameter.concat([
      gl.ACTIVE_UNIFORM_BLOCKS,
      gl.TRANSFORM_FEEDBACK_VARYINGS,
      gl.TRANSFORM_FEEDBACK_BUFFER_MODE
  ]);
  program = uniformBlockProgram;
}
testInvalidArgument(
    "getProgramParameter",
    "parameter",
    validArrayForProgramParameter,
    function(parameter) {
      return gl.getProgramParameter(program, parameter);
    }
);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("Test getRenderbufferParameter");
shouldBe('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_WIDTH)', '2');
shouldBe('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_HEIGHT)', '2');
// Note: we can't test the actual value of the internal format since
// the implementation is allowed to change it.
shouldBeNonZero('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_INTERNAL_FORMAT)');
shouldBeNonZero('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_DEPTH_SIZE)');
var colorbuffer = gl.createRenderbuffer();
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 2, 2);
shouldBeNonZero('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_RED_SIZE)');
shouldBeNonZero('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_GREEN_SIZE)');
shouldBeNonZero('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_BLUE_SIZE)');
shouldBeNonZero('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_ALPHA_SIZE)');
if (contextVersion > 1) {
  gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA4, 2, 2);
  shouldBe('gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_SAMPLES)', '4');
}
var validArrayForRenderbuffer = new Array(
    gl.RENDERBUFFER_WIDTH,
    gl.RENDERBUFFER_HEIGHT,
    gl.RENDERBUFFER_INTERNAL_FORMAT,
    gl.RENDERBUFFER_RED_SIZE,
    gl.RENDERBUFFER_GREEN_SIZE,
    gl.RENDERBUFFER_BLUE_SIZE,
    gl.RENDERBUFFER_ALPHA_SIZE,
    gl.RENDERBUFFER_DEPTH_SIZE,
    gl.RENDERBUFFER_STENCIL_SIZE
);
if (contextVersion > 1) {
  validArrayForRenderbuffer[validArrayForRenderbuffer.length] = gl.RENDERBUFFER_SAMPLES;
}
testInvalidArgument(
    "getRenderbufferParameter",
    "parameter",
    validArrayForRenderbuffer,
    function(parameter) {
      return gl.getRenderbufferParameter(gl.RENDERBUFFER, parameter);
    });
testInvalidArgument(
    "getRenderbufferParameter",
    "target",
    [ gl.RENDERBUFFER ],
    function(target) {
      return gl.getRenderbufferParameter(target, gl.RENDERBUFFER_WIDTH);
    }
);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("Test getShaderParameter");
shouldBe('gl.getShaderParameter(standardVert, gl.SHADER_TYPE)', 'gl.VERTEX_SHADER');
shouldBe('gl.getShaderParameter(standardVert, gl.DELETE_STATUS)', 'false');
shouldBe('gl.getShaderParameter(standardVert, gl.COMPILE_STATUS)', 'true');
testInvalidArgument(
    "getShaderParameter",
    "parameter",
    [ gl.DELETE_STATUS,
      gl.COMPILE_STATUS,
      gl.SHADER_TYPE
    ],
    function(parameter) {
      return gl.getShaderParameter(standardVert, parameter);
    }
);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("Test getTexParameter");
gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
gl.bindTexture(gl.TEXTURE_2D, texture);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER)', 'gl.NEAREST');
shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER)', 'gl.NEAREST');
shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S)', 'gl.CLAMP_TO_EDGE');
shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T)', 'gl.CLAMP_TO_EDGE');
if (contextVersion > 1) {
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_BASE_LEVEL, 0);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_FUNC, gl.LEQUAL);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL, 10);
  gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MAX_LOD, 10);
  gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MIN_LOD, 0);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_BASE_LEVEL)', '0');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_FUNC)', 'gl.LEQUAL');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_COMPARE_MODE)', 'gl.COMPARE_REF_TO_TEXTURE');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MAX_LEVEL)', '10');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MAX_LOD)', '10');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_MIN_LOD)', '0');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_R)', 'gl.CLAMP_TO_EDGE');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_IMMUTABLE_FORMAT)', 'false');
  shouldBe('gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_IMMUTABLE_LEVELS)', '0');
}
var validParametersForTexture = [
    gl.TEXTURE_MAG_FILTER,
    gl.TEXTURE_MIN_FILTER,
    gl.TEXTURE_WRAP_S,
    gl.TEXTURE_WRAP_T,
];
if (contextVersion > 1) {
  validParametersForTexture = validParametersForTexture.concat([
      gl.TEXTURE_BASE_LEVEL,
      gl.TEXTURE_COMPARE_FUNC,
      gl.TEXTURE_COMPARE_MODE,
      gl.TEXTURE_MAX_LEVEL,
      gl.TEXTURE_MAX_LOD,
      gl.TEXTURE_MIN_LOD,
      gl.TEXTURE_WRAP_R,
      gl.TEXTURE_IMMUTABLE_FORMAT,
      gl.TEXTURE_IMMUTABLE_LEVELS,
  ]);
}
testInvalidArgument(
    "getTexParameter",
    "parameter",
    validParametersForTexture,
    function(parameter) {
      return gl.getTexParameter(gl.TEXTURE_2D, parameter);
    }
);
var validTargetsForTexture = [ gl.TEXTURE_2D, gl.TEXTURE_CUBE_MAP];
if (contextVersion > 1) {
  validTargetsForTexture = validTargetsForTexture.concat([ gl.TEXTURE_3D, gl.TEXTURE_2D_ARRAY]);
}
testInvalidArgument(
    "getTexParameter",
    "target",
    validTargetsForTexture,
    function(target) {
      return gl.getTexParameter(target, gl.TEXTURE_MAG_FILTER);
    }
);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("");
debug("Test getUniform with all variants of data types");
debug("Boolean uniform variables");
var boolProgram = wtu.loadProgramFromFile(gl, "../../resources/boolUniformShader.vert", "../../resources/noopUniformShader.frag");
shouldBe('gl.getProgramParameter(boolProgram, gl.LINK_STATUS)', 'true');
var bvalLoc = gl.getUniformLocation(boolProgram, "bval");
var bval2Loc = gl.getUniformLocation(boolProgram, "bval2");
var bval3Loc = gl.getUniformLocation(boolProgram, "bval3");
var bval4Loc = gl.getUniformLocation(boolProgram, "bval4");
gl.useProgram(boolProgram);
gl.uniform1i(bvalLoc, 1);
gl.uniform2i(bval2Loc, 1, 0);
gl.uniform3i(bval3Loc, 1, 0, 1);
gl.uniform4i(bval4Loc, 1, 0, 1, 0);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldBe('gl.getUniform(boolProgram, bvalLoc)', 'true');
shouldBe('gl.getUniform(boolProgram, bval2Loc)', '[true, false]');
shouldBe('gl.getUniform(boolProgram, bval3Loc)', '[true, false, true]');
shouldBe('gl.getUniform(boolProgram, bval4Loc)', '[true, false, true, false]');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("Integer uniform variables");
var intProgram = wtu.loadProgramFromFile(gl, "../../resources/intUniformShader.vert", "../../resources/noopUniformShader.frag");
shouldBe('gl.getProgramParameter(intProgram, gl.LINK_STATUS)', 'true');
var ivalLoc = gl.getUniformLocation(intProgram, "ival");
var ival2Loc = gl.getUniformLocation(intProgram, "ival2");
var ival3Loc = gl.getUniformLocation(intProgram, "ival3");
var ival4Loc = gl.getUniformLocation(intProgram, "ival4");
gl.useProgram(intProgram);
gl.uniform1i(ivalLoc, 1);
gl.uniform2i(ival2Loc, 2, 3);
gl.uniform3i(ival3Loc, 4, 5, 6);
gl.uniform4i(ival4Loc, 7, 8, 9, 10);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldBe('gl.getUniform(intProgram, ivalLoc)', '1');
shouldBe('gl.getUniform(intProgram, ival2Loc)', '[2, 3]');
shouldBe('gl.getUniform(intProgram, ival3Loc)', '[4, 5, 6]');
shouldBe('gl.getUniform(intProgram, ival4Loc)', '[7, 8, 9, 10]');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("Float uniform variables");
var floatProgram = wtu.loadProgramFromFile(gl, "../../resources/floatUniformShader.vert", "../../resources/noopUniformShader.frag");
shouldBe('gl.getProgramParameter(floatProgram, gl.LINK_STATUS)', 'true');
var fvalLoc = gl.getUniformLocation(floatProgram, "fval");
var fval2Loc = gl.getUniformLocation(floatProgram, "fval2");
var fval3Loc = gl.getUniformLocation(floatProgram, "fval3");
var fval4Loc = gl.getUniformLocation(floatProgram, "fval4");
gl.useProgram(floatProgram);
gl.uniform1f(fvalLoc, 11);
gl.uniform2f(fval2Loc, 12, 13);
gl.uniform3f(fval3Loc, 14, 15, 16);
gl.uniform4f(fval4Loc, 17, 18, 19, 20);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldBe('gl.getUniform(floatProgram, fvalLoc)', '11');
shouldBe('gl.getUniform(floatProgram, fval2Loc)', '[12, 13]');
shouldBe('gl.getUniform(floatProgram, fval3Loc)', '[14, 15, 16]');
shouldBe('gl.getUniform(floatProgram, fval4Loc)', '[17, 18, 19, 20]');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("Sampler uniform variables");
var samplerProgram = wtu.loadProgramFromFile(gl, "../../resources/noopUniformShader.vert", "../../resources/samplerUniformShader.frag");
shouldBe('gl.getProgramParameter(samplerProgram, gl.LINK_STATUS)', 'true');
var s2DValLoc = gl.getUniformLocation(samplerProgram, "s2D");
var sCubeValLoc = gl.getUniformLocation(samplerProgram, "sCube");
gl.useProgram(samplerProgram);
gl.uniform1i(s2DValLoc, 0);
gl.uniform1i(sCubeValLoc, 1);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldBe('gl.getUniform(samplerProgram, s2DValLoc)', '0');
shouldBe('gl.getUniform(samplerProgram, sCubeValLoc)', '1');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

debug("Matrix uniform variables");
var matProgram = wtu.loadProgramFromFile(gl, "../../resources/matUniformShader.vert", "../../resources/noopUniformShader.frag");
shouldBe('gl.getProgramParameter(matProgram, gl.LINK_STATUS)', 'true');
var mval2Loc = gl.getUniformLocation(matProgram, "mval2");
var mval3Loc = gl.getUniformLocation(matProgram, "mval3");
var mval4Loc = gl.getUniformLocation(matProgram, "mval4");
gl.useProgram(matProgram);
gl.uniformMatrix2fv(mval2Loc, false, [1, 2, 3, 4]);
gl.uniformMatrix3fv(mval3Loc, false, [5, 6, 7, 8, 9, 10, 11, 12, 13]);
gl.uniformMatrix4fv(mval4Loc, false, [14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29]);
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
shouldBe('gl.getUniform(matProgram, mval2Loc)', '[1, 2, 3, 4]');
shouldBe('gl.getUniform(matProgram, mval3Loc)', '[5, 6, 7, 8, 9, 10, 11, 12, 13]');
shouldBe('gl.getUniform(matProgram, mval4Loc)', '[14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29]');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

if (contextVersion > 1) {
  debug("Unsigned Integer uniform variables");
  var uintProgram = wtu.loadProgramFromFile(gl, "../../resources/uintUniformShader.vert", "../../resources/noopUniformShaderES3.frag");
  shouldBe('gl.getProgramParameter(uintProgram, gl.LINK_STATUS)', 'true');
  var uvalLoc = gl.getUniformLocation(uintProgram, "uval");
  var uval2Loc = gl.getUniformLocation(uintProgram, "uval2");
  var uval3Loc = gl.getUniformLocation(uintProgram, "uval3");
  var uval4Loc = gl.getUniformLocation(uintProgram, "uval4");
  gl.useProgram(uintProgram);
  gl.uniform1ui(uvalLoc, 1);
  gl.uniform2ui(uval2Loc, 2, 3);
  gl.uniform3ui(uval3Loc, 4, 5, 6);
  gl.uniform4ui(uval4Loc, 7, 8, 9, 10);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  shouldBe('gl.getUniform(uintProgram, uvalLoc)', '1');
  shouldBe('gl.getUniform(uintProgram, uval2Loc)', '[2, 3]');
  shouldBe('gl.getUniform(uintProgram, uval3Loc)', '[4, 5, 6]');
  shouldBe('gl.getUniform(uintProgram, uval4Loc)', '[7, 8, 9, 10]');
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);

  debug("Matrix uniform variables for WebGL 2");
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);

  var matForWebGL2Program = wtu.loadProgramFromFile(gl, "../../resources/matForWebGL2UniformShader.vert", "../../resources/noopUniformShaderES3.frag");
  shouldBe('gl.getProgramParameter(matForWebGL2Program, gl.LINK_STATUS)', 'true');
  var mval2x3Loc = gl.getUniformLocation(matForWebGL2Program, "mval2x3");
  var mval2x4Loc = gl.getUniformLocation(matForWebGL2Program, "mval2x4");
  var mval3x2Loc = gl.getUniformLocation(matForWebGL2Program, "mval3x2");
  var mval3x4Loc = gl.getUniformLocation(matForWebGL2Program, "mval3x4");
  var mval4x2Loc = gl.getUniformLocation(matForWebGL2Program, "mval4x2");
  var mval4x3Loc = gl.getUniformLocation(matForWebGL2Program, "mval4x3");
  gl.useProgram(matForWebGL2Program);
  gl.uniformMatrix2x3fv(mval2x3Loc, false, [1, 2, 3, 4, 5, 6]);
  gl.uniformMatrix2x4fv(mval2x4Loc, false, [7, 8, 9, 10, 11, 12, 13, 14]);
  gl.uniformMatrix3x2fv(mval3x2Loc, false, [15, 16, 17, 18, 19, 20]);
  gl.uniformMatrix3x4fv(mval3x4Loc, false, [21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]);
  gl.uniformMatrix4x2fv(mval4x2Loc, false, [33, 34, 35, 36, 37, 38, 39, 40]);
  gl.uniformMatrix4x3fv(mval4x3Loc, false, [41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52]);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  shouldBe('gl.getUniform(matForWebGL2Program, mval2x3Loc)', '[1, 2, 3, 4, 5, 6]');
  shouldBe('gl.getUniform(matForWebGL2Program, mval2x4Loc)', '[7, 8, 9, 10, 11, 12, 13, 14]');
  shouldBe('gl.getUniform(matForWebGL2Program, mval3x2Loc)', '[15, 16, 17, 18, 19, 20]');
  shouldBe('gl.getUniform(matForWebGL2Program, mval3x4Loc)', '[21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]');
  shouldBe('gl.getUniform(matForWebGL2Program, mval4x2Loc)', '[33, 34, 35, 36, 37, 38, 39, 40]');
  shouldBe('gl.getUniform(matForWebGL2Program, mval4x3Loc)', '[41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52]');
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);

  debug("Sampler uniform variables for WebGL2");
  var samplerForWebGL2Program = wtu.loadProgramFromFile(gl, "../../resources/noopUniformShaderES3.vert", "../../resources/samplerForWebGL2UniformShader.frag");
  shouldBe('gl.getProgramParameter(samplerForWebGL2Program, gl.LINK_STATUS)', 'true');
  var s3DValLoc = gl.getUniformLocation(samplerForWebGL2Program, "s3D");
  var s2DArrayValLoc = gl.getUniformLocation(samplerForWebGL2Program, "s2DArray");
  gl.useProgram(samplerForWebGL2Program);
  gl.uniform1i(s3DValLoc, 0);
  gl.uniform1i(s2DArrayValLoc, 1);
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
  shouldBe('gl.getUniform(samplerForWebGL2Program, s3DValLoc)', '0');
  shouldBe('gl.getUniform(samplerForWebGL2Program, s2DArrayValLoc)', '1');
  wtu.glErrorShouldBe(gl, gl.NO_ERROR);
}

debug("");
debug("test getVertexAttrib");
var array = new Float32Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
var buffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
gl.bufferData(gl.ARRAY_BUFFER, array, gl.DYNAMIC_DRAW);
// Vertex attribute 0 is special in that it has no current state, so
// fetching GL_CURRENT_VERTEX_ATTRIB generates an error. Use attribute
// 1 for these tests instead.
gl.enableVertexAttribArray(1);
gl.vertexAttribPointer(1, 4, gl.FLOAT, false, 0, 0);
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING)', 'buffer');
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_ENABLED)', 'true');
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_SIZE)', '4');
// Stride MUST be the value the user put in.
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_STRIDE)', '0');
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_TYPE)', 'gl.FLOAT');
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_NORMALIZED)', 'false');
if (contextVersion > 1) {
  shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_DIVISOR)', '0');
  shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_INTEGER)', 'false');
  gl.vertexAttribDivisor(1, 2);
  shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_DIVISOR)', '2');
}
gl.vertexAttribPointer(1, 4, gl.FLOAT, false, 36, 12);
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_STRIDE)', '36');
shouldBe('gl.getVertexAttribOffset(1, gl.VERTEX_ATTRIB_ARRAY_POINTER)', '12');
gl.disableVertexAttribArray(1);
shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_ENABLED)', 'false');
gl.vertexAttrib4f(1, 5, 6, 7, 8);
shouldBe('gl.getVertexAttrib(1, gl.CURRENT_VERTEX_ATTRIB)', '[5, 6, 7, 8]');
if (contextVersion > 1) {
  var intArray = new Int32Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
  gl.bufferData(gl.ARRAY_BUFFER, intArray, gl.DYNAMIC_DRAW);
  gl.enableVertexAttribArray(1);
  // feed fixed-point data to buffer
  gl.vertexAttribIPointer(1, 4, gl.INT, false, 0, 0);
  shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_TYPE)', 'gl.INT');
  shouldBe('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_INTEGER)', 'true');
}
wtu.glErrorShouldBe(gl, gl.NO_ERROR);
var validArrayForVertexAttrib = new Array(
    gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING,
    gl.VERTEX_ATTRIB_ARRAY_ENABLED,
    gl.VERTEX_ATTRIB_ARRAY_SIZE,
    gl.VERTEX_ATTRIB_ARRAY_STRIDE,
    gl.VERTEX_ATTRIB_ARRAY_TYPE,
    gl.VERTEX_ATTRIB_ARRAY_NORMALIZED,
    gl.CURRENT_VERTEX_ATTRIB
);
if (contextVersion > 1) {
  validArrayForVertexAttrib[validArrayForVertexAttrib.length] = gl.VERTEX_ATTRIB_ARRAY_DIVISOR;
  validArrayForVertexAttrib[validArrayForVertexAttrib.length] = gl.VERTEX_ATTRIB_ARRAY_INTEGER;
}
testInvalidArgument(
    "getVertexAttrib",
    "parameter",
    validArrayForVertexAttrib,
    function(parameter) {
      return gl.getVertexAttrib(1, parameter);
    }
);
var numVertexAttribs = gl.getParameter(gl.MAX_VERTEX_ATTRIBS);
wtu.shouldGenerateGLError(gl, gl.INVALID_VALUE, 'gl.getVertexAttrib(' + numVertexAttribs + ', gl.CURRENT_VERTEX_ATTRIB)');

debug("");
debug("Test cases where name == 0");
gl.deleteTexture(texture);
shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.NONE');
gl.deleteRenderbuffer(renderbuffer);
gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
wtu.glErrorShouldBe(gl, gl.INVALID_OPERATION);
shouldBe('gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE)', 'gl.NONE');
gl.deleteBuffer(buffer);
shouldBeNull('gl.getVertexAttrib(1, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING)');
wtu.glErrorShouldBe(gl, gl.NO_ERROR);

if (contextVersion > 1) {
    debug("");
    debug("Test getInternalformatParameter")

    shouldBeNonNull('gl.getInternalformatParameter(gl.RENDERBUFFER, gl.R32I, gl.SAMPLES)');
    wtu.glErrorShouldBe(gl, gl.NO_ERROR);

    testInvalidArgument(
        "getInternalformatParameter",
        "target",
        [ gl.RENDERBUFFER ],
        function(target) {
            return gl.getInternalformatParameter(target, gl.R32I, gl.SAMPLES);
    });

    testInvalidArgument(
        "getInternalformatParameter",
        "pname",
        [ gl.SAMPLES ],
        function(pname) {
            return gl.getInternalformatParameter(gl.RENDERBUFFER, gl.RGBA4, pname);
    });

    var validArrayForInterformat = new Array(
        gl.R8, gl.R8_SNORM, gl.RG8, gl.RG8_SNORM,
        gl.RGB8, gl.RGB8_SNORM, gl.RGB565, gl.RGBA4,
        gl.RGB5_A1, gl.RGBA8, gl.RGBA8_SNORM, gl.RGB10_A2,
        gl.RGB10_A2UI, gl.SRGB8, gl.SRGB8_ALPHA8, gl.R16F,
        gl.RG16F, gl.RGB16F, gl.RGBA16F, gl.R32F,
        gl.RG32F, gl.RGB32F, gl.RGBA32F, gl.R11F_G11F_B10F,
        gl.RGB9_E5, gl.R8I, gl.R8UI, gl.R16I,
        gl.R16UI, gl.R32I, gl.R32UI, gl.RG8I,
        gl.RG8UI, gl.RG16I, gl.RG16UI, gl.RG32I,
        gl.RG32UI, gl.RGB8I, gl.RGB8UI, gl.RGB16I,
        gl.RGB16UI, gl.RGB32I, gl.RGB32UI, gl.RGBA8I,
        gl.RGBA8UI, gl.RGBA16I, gl.RGBA16UI, gl.RGBA32I,
        gl.RGBA32UI, gl.RGB, gl.RGBA, gl.DEPTH_STENCIL, gl.DEPTH_COMPONENT16,
        gl.DEPTH_COMPONENT24, gl.DEPTH_COMPONENT32F, gl.DEPTH24_STENCIL8,
        gl.DEPTH32F_STENCIL8, gl.STENCIL_INDEX8
    );
    testInvalidArgument(
        "getInternalformatParameter",
        "internalformat",
        validArrayForInterformat,
        function(internalformat) {
            return gl.getInternalformatParameter(gl.RENDERBUFFER, internalformat, gl.SAMPLES);
    });


    debug("");
    debug("Test getIndexedParameter");
    var buffer = gl.createBuffer();
    gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buffer);
    gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 64, gl.DYNAMIC_DRAW);
    gl.bindBufferRange(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buffer, 4, 8);
    shouldBe('gl.getIndexedParameter(gl.TRANSFORM_FEEDBACK_BUFFER_BINDING, 0)', 'buffer');
    shouldBe('gl.getIndexedParameter(gl.TRANSFORM_FEEDBACK_BUFFER_SIZE, 0)', '8');
    shouldBe('gl.getIndexedParameter(gl.TRANSFORM_FEEDBACK_BUFFER_START, 0)', '4');
    var buffer1 = gl.createBuffer();
    gl.bindBuffer(gl.UNIFORM_BUFFER, buffer1);
    gl.bufferData(gl.UNIFORM_BUFFER, 64, gl.DYNAMIC_DRAW);
    var offsetUniform = gl.getParameter(gl.UNIFORM_BUFFER_OFFSET_ALIGNMENT);
    gl.bindBufferRange(gl.UNIFORM_BUFFER, 1, buffer1, offsetUniform, 8);
    shouldBe('gl.getIndexedParameter(gl.UNIFORM_BUFFER_BINDING, 1)', 'buffer1');
    shouldBe('gl.getIndexedParameter(gl.UNIFORM_BUFFER_SIZE, 1)', '8');
    shouldBe('gl.getIndexedParameter(gl.UNIFORM_BUFFER_START, 1)', 'offsetUniform');

    gl.bindBufferBase(gl.UNIFORM_BUFFER, 1, null);
    shouldBe('gl.getIndexedParameter(gl.UNIFORM_BUFFER_BINDING, 1)', 'null');

    var validArrayForTarget = new Array(
        gl.TRANSFORM_FEEDBACK_BUFFER_BINDING,
        gl.TRANSFORM_FEEDBACK_BUFFER_SIZE,
        gl.TRANSFORM_FEEDBACK_BUFFER_START,
        gl.UNIFORM_BUFFER_BINDING,
        gl.UNIFORM_BUFFER_SIZE,
        gl.UNIFORM_BUFFER_START
    );
    testInvalidArgument(
        "getIndexedParameter",
        "target",
        validArrayForTarget,
        function(target) {
            return gl.getIndexedParameter(target, 0);
    });

    debug("");
    debug("Test getSamplerParameter");
    var sampler = gl.createSampler();
    gl.samplerParameteri(sampler, gl.TEXTURE_COMPARE_FUNC, gl.LEQUAL);
    gl.samplerParameteri(sampler, gl.TEXTURE_COMPARE_MODE, gl.COMPARE_REF_TO_TEXTURE);
    gl.samplerParameteri(sampler, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.samplerParameterf(sampler, gl.TEXTURE_MAX_LOD, 10);
    gl.samplerParameteri(sampler, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.samplerParameterf(sampler, gl.TEXTURE_MIN_LOD, 0);
    gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
    gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_COMPARE_FUNC)', 'gl.LEQUAL');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_COMPARE_MODE)', 'gl.COMPARE_REF_TO_TEXTURE');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_MAG_FILTER)', 'gl.NEAREST');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_MAX_LOD)', '10');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_MIN_FILTER)', 'gl.NEAREST');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_MIN_LOD)', '0');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_WRAP_R)', 'gl.CLAMP_TO_EDGE');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_WRAP_S)', 'gl.CLAMP_TO_EDGE');
    shouldBe('gl.getSamplerParameter(sampler, gl.TEXTURE_WRAP_T)', 'gl.CLAMP_TO_EDGE');
    var validArrayForSamplerParameter = new Array(
        gl.TEXTURE_COMPARE_FUNC,
        gl.TEXTURE_COMPARE_MODE,
        gl.TEXTURE_MAG_FILTER,
        gl.TEXTURE_MAX_LOD,
        gl.TEXTURE_MIN_FILTER,
        gl.TEXTURE_MIN_LOD,
        gl.TEXTURE_WRAP_R,
        gl.TEXTURE_WRAP_S,
        gl.TEXTURE_WRAP_T
    );
    testInvalidArgument(
        "getSamplerParameter",
        "pname",
        validArrayForSamplerParameter,
        function(pname) {
            return gl.getSamplerParameter(sampler, pname);
    });

    debug("");
    debug("Test getSyncParameter");
    var sync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
    shouldBe('gl.getSyncParameter(sync, gl.OBJECT_TYPE)', 'gl.SYNC_FENCE');
    var sync_status = gl.getSyncParameter(sync, gl.SYNC_STATUS);
    switch (sync_status) {
      case gl.UNSIGNALED:
        testPassed('gl.getSyncParameter(sync, gl.SYNC_CONDITION) is gl.UNSIGNALED');
        break;
      case gl.SIGNALED:
        testPassed('gl.getSyncParameter(sync, gl.SYNC_CONDITION) is gl.SIGNALED');
        break;
      default:
        testFailed('gl.getSyncParameter(sync, gl.SYNC_CONDITION) was ' + sync_status +
                   ', expected gl.UNSIGNALED or gl.SIGNALED');
        break;
    }
    shouldBe('gl.getSyncParameter(sync, gl.SYNC_CONDITION)', 'gl.SYNC_GPU_COMMANDS_COMPLETE');
    shouldBe('gl.getSyncParameter(sync, gl.SYNC_FLAGS)', '0');
    var validArrayForSyncParameter = new Array(
        gl.OBJECT_TYPE,
        gl.SYNC_STATUS,
        gl.SYNC_CONDITION,
        gl.SYNC_FLAGS
    );
    testInvalidArgument(
        "getSyncParameter",
        "pname",
        validArrayForSyncParameter,
        function(pname) {
            return gl.getSyncParameter(sync, pname);
    });

    debug("");
    debug("Test getQueryParameter");
    var query = gl.createQuery();
    gl.beginQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, query);
    gl.endQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN);
    shouldBe('gl.getQueryParameter(query, gl.QUERY_RESULT_AVAILABLE)', 'false');
    // Queries' results are tested elsewhere in the conformance suite. It's complicated
    // to wait for this query's result to become available and verify it.
    var validArrayForPname = new Array(
        gl.QUERY_RESULT,
        gl.QUERY_RESULT_AVAILABLE
    );
    testInvalidArgument(
        "getQueryParameter",
        "pname",
        validArrayForPname,
        function(pname) {
            return gl.getQueryParameter(query, pname);
        }
    );

    debug("");
    debug("Test getFragDataLocation");
    var baseVertShader = '' +
    '#version 300 es\n' +
    'uniform mat4 modelViewMatrix;\n' +
    'uniform mat4 projectionMatrix;\n' +
    'in vec4 vertex;\n' +
    'out vec4 position;\n' +
    'void main (void)\n' +
    '{\n' +
    '       position = modelViewMatrix * vertex;\n' +
    '       gl_Position = projectionMatrix * position;\n' +
    '}\n';
    var baseFragShader = '' +
    '#version 300 es\n' +
    'in lowp vec4 position;\n' +
    'layout(location = 0) out mediump vec4 fragColor;\n' +
    'void main (void)\n' +
    '{\n' +
    '       fragColor = position;\n' +
    '}\n';
    var vertShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vertShader, baseVertShader);
    gl.compileShader(vertShader);
    shouldBe('gl.getShaderParameter(vertShader, gl.COMPILE_STATUS)', 'true');
    var fragShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fragShader, baseFragShader);
    gl.compileShader(fragShader);
    shouldBe('gl.getShaderParameter(fragShader, gl.COMPILE_STATUS)', 'true');
    var program = gl.createProgram();
    gl.attachShader(program, vertShader);
    gl.attachShader(program, fragShader);
    gl.linkProgram(program);
    shouldBe('gl.getProgramParameter(program, gl.LINK_STATUS)','true');
    shouldBe('gl.getFragDataLocation(program, "vertexColor")', '-1');
    shouldBe('gl.getFragDataLocation(program, "modelViewMatrix")', '-1');
    shouldBe('gl.getFragDataLocation(program, "projectionMatrix")', '-1');
    shouldBe('gl.getFragDataLocation(program, "position")', '-1');
    shouldBe('gl.getFragDataLocation(program, "fragColor")', '0');

    debug("");
    debug("Test getActiveUniforms");
    var program = wtu.loadUniformBlockProgram(gl);
    gl.linkProgram(program);
    shouldBe('gl.getProgramParameter(program, gl.LINK_STATUS)', 'true');
    shouldBe('gl.getError()', 'gl.NO_ERROR');

    var numActiveUniforms = gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS);
    var blockIndex = gl.getUniformBlockIndex(program, "Transform");
    var uniformIndices = [];
    for (var i = 0; i < numActiveUniforms; i++)
      uniformIndices.push(i);
    var types = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_TYPE);
    var sizes = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_SIZE);
    var blockIndices = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_BLOCK_INDEX);
    var offsets = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_OFFSET);
    var arrayStrides = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_ARRAY_STRIDE);
    var matrixStrides = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_MATRIX_STRIDE);
    var rowMajors = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_IS_ROW_MAJOR);
    for (var i = 0; i < numActiveUniforms; i++) {
      if (types[i] != gl.FLOAT_MAT4 && types[i] != gl.FLOAT_MAT3)
        testFailed("expected value: GL_FLOAT_MAT4 or GL_FLOAT_MAT3" + " actual value for UNIFORM_TYPE for uniform index[" + i + "]:" + wtu.glEnumToString(gl, types[i]));
      if (sizes[i] != 1)
        testFailed("expected value: 1" + " actual value for UNIFORM_SIZE for uniform index[" + i + "]:" + sizes[i]);
      if (blockIndices[i] != blockIndex)
        testFailed("expected value: 0" + " actual value for UNIFORM_BLOCK_INDEX for uniform index[" + i + "]:" + blockIndices[i]);
      if (offsets[i] < 0)
        testFailed("expected value >= 0" + " actual value for UNIFORM_OFFSET for uniform index[" + i + "]:" + offsets[i]);
      if (arrayStrides[i] != 0)
        testFailed("expected value: 0" + " actual value for UNIFORM_ARRAY_STRIDE for uniform index[" + i + "]:" + arrayStrides[i]);
      if (matrixStrides[i] < 0)
        testFailed("expected value >= 0" + " actual value for UNIFORM_MATRIX_STRIDE for uniform index[" + i + "]:" + matrixStrides[i]);
      shouldBe('typeof rowMajors[i]', '"boolean"');
      if (rowMajors[i] != false)
        testFailed("expected value: 0" + " actual value for UNIFORM_IS_ROW_MAJOR for uniform index[" + i + "]:" + rowMajors[i]);
    }

    var validArrayForPname = new Array(
        gl.UNIFORM_TYPE,
        gl.UNIFORM_SIZE,
        gl.UNIFORM_BLOCK_INDEX,
        gl.UNIFORM_OFFSET,
        gl.UNIFORM_ARRAY_STRIDE,
        gl.UNIFORM_MATRIX_STRIDE,
        gl.UNIFORM_IS_ROW_MAJOR
    );
    testInvalidArgument(
        "getActiveUniforms",
        "pname",
        validArrayForPname,
        function(pname) {
            return gl.getActiveUniforms(program, uniformIndices, pname);
        }
    );

    debug("");
    debug("Test getUniformBlockIndex");
    var program = wtu.loadUniformBlockProgram(gl);
    gl.linkProgram(program);
    shouldBeTrue('gl.getProgramParameter(program, gl.LINK_STATUS)');
    shouldBe('gl.getUniformBlockIndex(program, "Transform")', '0');
    shouldBe('gl.getUniformBlockIndex(program, "u_modelViewMatrix")', 'gl.INVALID_INDEX');
    shouldBe('gl.getUniformBlockIndex(program, "normal")', 'gl.INVALID_INDEX');
    shouldBe('gl.getUniformBlockIndex(program, "u_normal")', 'gl.INVALID_INDEX');
    var noUniformProgram = wtu.loadStandardProgram(gl);
    gl.linkProgram(noUniformProgram);
    shouldBeTrue('gl.getProgramParameter(noUniformProgram, gl.LINK_STATUS)');
    shouldBe('gl.getUniformBlockIndex(noUniformProgram, "u_modelViewProjMatrix")', 'gl.INVALID_INDEX');
    shouldBe('gl.getUniformBlockIndex(noUniformProgram, "u_normal")', 'gl.INVALID_INDEX');

    debug("");
    debug("Test getActiveUniformBlockName");
    var program = wtu.loadUniformBlockProgram(gl);
    gl.linkProgram(program);
    shouldBeTrue('gl.getProgramParameter(program, gl.LINK_STATUS)');
    shouldBeEqualToString('gl.getActiveUniformBlockName(program, 0)', 'Transform');
    shouldBeNull('gl.getActiveUniformBlockName(program, -1)');
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE);
    shouldBeNull('gl.getActiveUniformBlockName(program, 1)');
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE);
    shouldBeNull('gl.getActiveUniformBlockName(program, gl.INVALID_INDEX)');
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE);
    var noLinkProgram = gl.createProgram();
    shouldBeFalse('gl.getProgramParameter(noLinkProgram, gl.LINK_STATUS)');
    wtu.shouldGenerateGLError(gl, gl.INVALID_OPERATION, 'gl.getActiveUniformBlockName(noLinkProgram, 0)');
    var noUniformProgram = wtu.loadStandardProgram(gl);
    gl.linkProgram(noUniformProgram);
    shouldBeTrue('gl.getProgramParameter(noUniformProgram, gl.LINK_STATUS)');
    shouldBeNull('gl.getActiveUniformBlockName(noUniformProgram, -1)');
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE);
    shouldBeNull('gl.getActiveUniformBlockName(noUniformProgram, 0)');
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE);
    shouldBeNull('gl.getActiveUniformBlockName(noUniformProgram, gl.INVALID_INDEX)');
    wtu.glErrorShouldBe(gl, gl.INVALID_VALUE);

    debug("");
    debug("Test getActiveUniformBlockParameter");
    var program = wtu.loadUniformBlockProgram(gl);
    gl.linkProgram(program);
    shouldBeTrue('gl.getProgramParameter(program, gl.LINK_STATUS)');
    shouldBe('gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_BINDING)', '0');
    gl.uniformBlockBinding(program, 0, 1);
    shouldBe('gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_BINDING)', '1');
    // The actual block data size can be bigger than 164, depending on the uniform block layout.
    shouldBeTrue('gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_DATA_SIZE) >= 164');
    shouldBe('gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_ACTIVE_UNIFORMS)', '3');
    shouldBeTrue('gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER)');
    shouldBeFalse('gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER)');
    var indices = gl.getActiveUniformBlockParameter(program, 0, gl.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES);
    for (var i = 0; i < 3; i++) {
      if (indices[i] < 0)
        testFailed("expected value >= 0" + " actual value for UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES for uniform index[" + i + "]:" + indices[i]);
    }
    var validArrayForPname = new Array(
        gl.UNIFORM_BLOCK_BINDING,
        gl.UNIFORM_BLOCK_DATA_SIZE,
        gl.UNIFORM_BLOCK_ACTIVE_UNIFORMS,
        gl.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
        gl.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER,
        gl.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER
    );
    testInvalidArgument(
        "getActiveUniformBlockParameter",
        "pname",
        validArrayForPname,
        function(pname) {
            return gl.getActiveUniformBlockParameter(program, 0, pname);
        }
    );
}

wtu.glErrorShouldBe(gl, gl.NO_ERROR);

var successfullyParsed = true;

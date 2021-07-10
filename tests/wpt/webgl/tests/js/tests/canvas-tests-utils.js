/*
** Copyright (c) 2016 The Khronos Group Inc.
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

// Some variables that will be used in this file
var canvas;
var gl;
var OES_vertex_array_object;
var uniformLocation;
var extension;
var buffer;
var framebuffer;
var program;
var renderbuffer;
var shader;
var texture;
var arrayBuffer;
var arrayBufferView;
var vertexArrayObject;
var imageData;
var float32array;
var int32array;

var OES_texture_float;
var new_WEBGL_lose_context;
var allowRestore;
var contextLostEventFired;
var contextRestoredEventFired;
var newExtension;

function compareGLError(glError, evalStr) {
    var exception;
    try {
        eval(evalStr);
    } catch (e) {
        exception = e;
    }
    if (exception) {
        return false;
    } else {
        if (gl.getError() == glError)
            return true;
        return false;
    }
}

function contextCreation(contextType) {
    canvas = new OffscreenCanvas(10, 10);
    gl = canvas.getContext(contextType);

    if (contextType == 'webgl') {
        if (gl instanceof WebGLRenderingContext)
            return true;
        return false;
    } else if (contextType == 'webgl2') {
        if (gl instanceof WebGL2RenderingContext)
            return true;
        return false;
    } else {
        return false;
    }
}

function transferredOffscreenCanvasCreation(placeholder, width, height) {
  placeholder.width = width;
  placeholder.height = height;
  return placeholder.transferControlToOffscreen();
}

function assertWidthAndHeight(entity, entityName, width, height) {
 if (entity.width == width && entity.height == height) {
   testPassed("The width and height of " + entityName + " are correct.");
   return;
 }
 var errMsg = "";
 if (entity.width != width) {
   errMsg += "The width of " + entityName + " is " + entity.width + " while expected value is " + width + ". ";
 }
 if (entity.height != height) {
   errMsg += "The height of " + entityName + " is " + entity.height + " while expected value is " + height + ". ";
 }
 testFailed(errMsg);
}

var webgl1Methods = [
  "getContextAttributes",
  "activeTexture",
  "attachShader",
  "bindAttribLocation",
  "bindBuffer",
  "bindFramebuffer",
  "bindRenderbuffer",
  "bindTexture",
  "blendColor",
  "blendEquation",
  "blendEquationSeparate",
  "blendFunc",
  "blendFuncSeparate",
  "bufferData",
  "bufferSubData",
  "checkFramebufferStatus",
  "clear",
  "clearColor",
  "clearDepth",
  "clearStencil",
  "colorMask",
  "compileShader",
  "compressedTexImage2D",
  "compressedTexSubImage2D",
  "copyTexImage2D",
  "copyTexSubImage2D",
  "createBuffer",
  "createFramebuffer",
  "createProgram",
  "createRenderbuffer",
  "createShader",
  "createTexture",
  "cullFace",
  "deleteBuffer",
  "deleteFramebuffer",
  "deleteProgram",
  "deleteRenderbuffer",
  "deleteShader",
  "deleteTexture",
  "depthFunc",
  "depthMask",
  "depthRange",
  "detachShader",
  "disable",
  "disableVertexAttribArray",
  "drawArrays",
  "drawElements",
  "enable",
  "enableVertexAttribArray",
  "finish",
  "flush",
  "framebufferRenderbuffer",
  "framebufferTexture2D",
  "frontFace",
  "generateMipmap",
  "getActiveAttrib",
  "getActiveUniform",
  "getAttachedShaders",
  "getAttribLocation",
  "getParameter",
  "getBufferParameter",
  "getError",
  "getExtension",
  "getFramebufferAttachmentParameter",
  "getProgramParameter",
  "getProgramInfoLog",
  "getRenderbufferParameter",
  "getShaderParameter",
  "getShaderInfoLog",
  "getShaderPrecisionFormat",
  "getShaderSource",
  "getSupportedExtensions",
  "getTexParameter",
  "getUniform",
  "getUniformLocation",
  "getVertexAttrib",
  "getVertexAttribOffset",
  "hint",
  "isBuffer",
  "isContextLost",
  "isEnabled",
  "isFramebuffer",
  "isProgram",
  "isRenderbuffer",
  "isShader",
  "isTexture",
  "lineWidth",
  "linkProgram",
  "pixelStorei",
  "polygonOffset",
  "readPixels",
  "renderbufferStorage",
  "sampleCoverage",
  "scissor",
  "shaderSource",
  "stencilFunc",
  "stencilFuncSeparate",
  "stencilMask",
  "stencilMaskSeparate",
  "stencilOp",
  "stencilOpSeparate",
  "texImage2D",
  "texParameterf",
  "texParameteri",
  "texSubImage2D",
  "uniform1f",
  "uniform1fv",
  "uniform1i",
  "uniform1iv",
  "uniform2f",
  "uniform2fv",
  "uniform2i",
  "uniform2iv",
  "uniform3f",
  "uniform3fv",
  "uniform3i",
  "uniform3iv",
  "uniform4f",
  "uniform4fv",
  "uniform4i",
  "uniform4iv",
  "uniformMatrix2fv",
  "uniformMatrix3fv",
  "uniformMatrix4fv",
  "useProgram",
  "validateProgram",
  "vertexAttrib1f",
  "vertexAttrib1fv",
  "vertexAttrib2f",
  "vertexAttrib2fv",
  "vertexAttrib3f",
  "vertexAttrib3fv",
  "vertexAttrib4f",
  "vertexAttrib4fv",
  "vertexAttribPointer",
  "viewport",
  "commit"
];

var webgl2Methods = [
  "getBufferSubData",
  "copyBufferSubData",
  "blitFramebuffer",
  "framebufferTextureLayer",
  "getInternalformatParameter",
  "invalidateFramebuffer",
  "invalidateSubFramebuffer",
  "readBuffer",
  "renderbufferStorageMultisample",
  "texImage3D",
  "texStorage2D",
  "texStorage3D",
  "texSubImage3D",
  "copyTexSubImage3D",
  "compressedTexImage3D",
  "compressedTexSubImage3D",
  "getFragDataLocation",
  "uniform1ui",
  "uniform2ui",
  "uniform3ui",
  "uniform4ui",
  "uniform1uiv",
  "uniform2uiv",
  "uniform3uiv",
  "uniform4uiv",
  "uniformMatrix2x3fv",
  "uniformMatrix3x2fv",
  "uniformMatrix2x4fv",
  "uniformMatrix4x2fv",
  "uniformMatrix3x4fv",
  "uniformMatrix4x3fv",
  "vertexAttribI4i",
  "vertexAttribI4iv",
  "vertexAttribI4ui",
  "vertexAttribI4uiv",
  "vertexAttribIPointer",
  "vertexAttribDivisor",
  "drawArraysInstanced",
  "drawElementsInstanced",
  "drawRangeElements",
  "drawBuffers",
  "clearBufferiv",
  "clearBufferuiv",
  "clearBufferfv",
  "clearBufferfi",
  "createQuery",
  "deleteQuery",
  "isQuery",
  "beginQuery",
  "endQuery",
  "getQuery",
  "getQueryParameter",
  "createSampler",
  "deleteSampler",
  "isSampler",
  "bindSampler",
  "samplerParameteri",
  "samplerParameterf",
  "getSamplerParameter",
  "fenceSync",
  "isSync",
  "deleteSync",
  "clientWaitSync",
  "waitSync",
  "getSyncParameter",
  "createTransformFeedback",
  "deleteTransformFeedback",
  "isTransformFeedback",
  "bindTransformFeedback",
  "beginTransformFeedback",
  "endTransformFeedback",
  "transformFeedbackVaryings",
  "getTransformFeedbackVarying",
  "pauseTransformFeedback",
  "resumeTransformFeedback",
  "bindBufferBase",
  "bindBufferRange",
  "getIndexedParameter",
  "getUniformIndices",
  "getActiveUniforms",
  "getUniformBlockIndex",
  "getActiveUniformBlockParameter",
  "getActiveUniformBlockName",
  "uniformBlockBinding",
  "createVertexArray",
  "deleteVertexArray",
  "isVertexArray",
  "bindVertexArray",
];

function assertFunction(v, f) {
    try {
        if (typeof v[f] != "function") {
            return false;
        } else {
            return true;
        }
    } catch(e) {
        return false;
    }
}

function testAPIs(contextType) {
    canvas = new OffscreenCanvas(10, 10);
    gl = canvas.getContext(contextType);
    var passed = true;
    var methods;
    if (contextType == 'webgl')
        methods = webgl1Methods;
    else
        methods = webgl1Methods.concat(webgl2Methods);
    for (var i=0; i<methods.length; i++) {
      var r = assertFunction(gl, methods[i]);
      passed = passed && r;
    }

    var extended = false;
    for (var i in gl) {
      if (typeof gl[i] == "function" && methods.indexOf(i) == -1) {
        if (!extended) {
          extended = true;
        }
      }
    }

    if (!passed || extended)
        return false;
    return true;
}

var simpleTextureVertexShader = [
  'attribute vec4 vPosition;',
  'attribute vec2 texCoord0;',
  'varying vec2 texCoord;',
  'void main() {',
  '    gl_Position = vPosition;',
  '    texCoord = texCoord0;',
  '}'].join('\n');

var simpleTextureFragmentShader = [
  'precision mediump float;',
  'uniform sampler2D tex;',
  'varying vec2 texCoord;',
  'void main() {',
  '    gl_FragData[0] = texture2D(tex, texCoord);',
  '}'].join('\n');

function getShader(gl, shaderStr, type)
{
  var shader = gl.createShader(type);
  gl.shaderSource(shader, shaderStr);
  gl.compileShader(shader);

  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS))
    return null;
  return shader;
}

function setupProgram(gl, shaders, opt_attribs, opt_locations)
{
  var vertexShader = getShader(gl, simpleTextureVertexShader, gl.VERTEX_SHADER);
  var fragmentShader = getShader(gl, simpleTextureFragmentShader, gl.FRAGMENT_SHADER);
  var program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);

  if (opt_attribs) {
    for (var ii = 0; ii < opt_attribs.length; ++ii) {
      gl.bindAttribLocation(
          program,
          opt_locations ? opt_locations[ii] : ii,
          opt_attribs[ii]);
    }
  }
  gl.linkProgram(program);

  // Check the link status
  var linked = gl.getProgramParameter(program, gl.LINK_STATUS);
  if (!linked) {
      // something went wrong with the link
      gl.deleteProgram(program);
      return null;
  }
  gl.useProgram(program);
  return program;
}

function setupSimpleTextureProgram(gl, opt_positionLocation, opt_texcoordLocation)
{
  opt_positionLocation = opt_positionLocation || 0;
  opt_texcoordLocation = opt_texcoordLocation || 1;
  return setupProgram(gl,
                      [simpleTextureVertexShader, simpleTextureFragmentShader],
                      ['vPosition', 'texCoord0'],
                      [opt_positionLocation, opt_texcoordLocation]);
}

function testLostContextWithoutRestore()
{
    // Functions with special return values.
    if (!gl.isContextLost())
        return false;

    if (gl.getError() != gl.CONTEXT_LOST_WEBGL)
        return false;
    if (gl.getError() != gl.NO_ERROR)
        return false;

    if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) != gl.FRAMEBUFFER_UNSUPPORTED ||
        gl.getAttribLocation(program, 'u_modelViewProjMatrix') != -1 ||
        gl.getVertexAttribOffset(0, gl.VERTEX_ATTRIB_ARRAY_POINTER) != 0)
        return false;

    // Test the extension itself.
    if (!compareGLError(gl.INVALID_OPERATION, "extension.loseContext()"))
        return false;

    imageData = new ImageData(1, 1);
    float32array = new Float32Array(1);
    int32array = new Int32Array(1);

    // Functions returning void should return immediately.
    // This is untestable, but we can at least be sure they cause no errors
    // and the codepaths are exercised.
    if (!compareGLError(gl.NO_ERROR, "gl.activeTexture(gl.TEXTURE0)") ||
        !compareGLError(gl.NO_ERROR, "gl.attachShader(program, shader)") ||
        !compareGLError(gl.NO_ERROR, "gl.bindBuffer(gl.ARRAY_BUFFER, buffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.bindTexture(gl.TEXTURE_2D, texture)") ||
        !compareGLError(gl.NO_ERROR, "gl.blendColor(1.0, 1.0, 1.0, 1.0)") ||
        !compareGLError(gl.NO_ERROR, "gl.blendEquation(gl.FUNC_ADD)") ||
        !compareGLError(gl.NO_ERROR, "gl.blendEquationSeparate(gl.FUNC_ADD, gl.FUNC_ADD)") ||
        !compareGLError(gl.NO_ERROR, "gl.blendFunc(gl.ONE, gl.ONE)") ||
        !compareGLError(gl.NO_ERROR, "gl.blendFuncSeparate(gl.ONE, gl.ONE, gl.ONE, gl.ONE)") ||
        !compareGLError(gl.NO_ERROR, "gl.bufferData(gl.ARRAY_BUFFER, 0, gl.STATIC_DRAW)") ||
        !compareGLError(gl.NO_ERROR, "gl.bufferData(gl.ARRAY_BUFFER, arrayBufferView, gl.STATIC_DRAW)") ||
        !compareGLError(gl.NO_ERROR, "gl.bufferData(gl.ARRAY_BUFFER, arrayBuffer, gl.STATIC_DRAW)") ||
        !compareGLError(gl.NO_ERROR, "gl.bufferSubData(gl.ARRAY_BUFFRE, 0, arrayBufferView)") ||
        !compareGLError(gl.NO_ERROR, "gl.bufferSubData(gl.ARRAY_BUFFRE, 0, arrayBuffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.clear(gl.COLOR_BUFFER_BIT)") ||
        !compareGLError(gl.NO_ERROR, "gl.clearColor(1, 1, 1, 1)") ||
        !compareGLError(gl.NO_ERROR, "gl.clearDepth(1)") ||
        !compareGLError(gl.NO_ERROR, "gl.clearStencil(0)") ||
        !compareGLError(gl.NO_ERROR, "gl.colorMask(1, 1, 1, 1)") ||
        !compareGLError(gl.NO_ERROR, "gl.compileShader(shader)") ||
        !compareGLError(gl.NO_ERROR, "gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 0, 0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.cullFace(gl.FRONT)") ||
        !compareGLError(gl.NO_ERROR, "gl.deleteBuffer(buffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.deleteFramebuffer(framebuffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.deleteProgram(program)") ||
        !compareGLError(gl.NO_ERROR, "gl.deleteRenderbuffer(renderbuffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.deleteShader(shader)") ||
        !compareGLError(gl.NO_ERROR, "gl.deleteTexture(texture)") ||
        !compareGLError(gl.NO_ERROR, "gl.depthFunc(gl.NEVER)") ||
        !compareGLError(gl.NO_ERROR, "gl.depthMask(0)") ||
        !compareGLError(gl.NO_ERROR, "gl.depthRange(0, 1)") ||
        !compareGLError(gl.NO_ERROR, "gl.detachShader(program, shader)") ||
        !compareGLError(gl.NO_ERROR, "gl.disable(gl.BLEND)") ||
        !compareGLError(gl.NO_ERROR, "gl.disableVertexAttribArray(0)") ||
        !compareGLError(gl.NO_ERROR, "gl.drawArrays(gl.POINTS, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.drawElements(gl.POINTS, 0, gl.UNSIGNED_SHORT, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.enable(gl.BLEND)") ||
        !compareGLError(gl.NO_ERROR, "gl.enableVertexAttribArray(0)") ||
        !compareGLError(gl.NO_ERROR, "gl.finish()") ||
        !compareGLError(gl.NO_ERROR, "gl.flush()") ||
        !compareGLError(gl.NO_ERROR, "gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbuffer)") ||
        !compareGLError(gl.NO_ERROR, "gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.frontFace(gl.CW)") ||
        !compareGLError(gl.NO_ERROR, "gl.generateMipmap(gl.TEXTURE_2D)") ||
        !compareGLError(gl.NO_ERROR, "gl.hint(gl.GENERATE_MIPMAP_HINT, gl.FASTEST)") ||
        !compareGLError(gl.NO_ERROR, "gl.lineWidth(0)") ||
        !compareGLError(gl.NO_ERROR, "gl.linkProgram(program)") ||
        !compareGLError(gl.NO_ERROR, "gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.polygonOffset(0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.readPixels(0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, arrayBufferView)") ||
        !compareGLError(gl.NO_ERROR, "gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.sampleCoverage(0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.scissor(0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.shaderSource(shader, '')") ||
        !compareGLError(gl.NO_ERROR, "gl.stencilFunc(gl.NEVER, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.stencilFuncSeparate(gl.FRONT, gl.NEVER, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.stencilMask(0)") ||
        !compareGLError(gl.NO_ERROR, "gl.stencilMaskSeparate(gl.FRONT, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP)") ||
        !compareGLError(gl.NO_ERROR, "gl.stencilOpSeparate(gl.FRONT, gl.KEEP, gl.KEEP, gl.KEEP)") ||
        !compareGLError(gl.NO_ERROR, "gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, arrayBufferView)") ||
        !compareGLError(gl.NO_ERROR, "gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, imageData)") ||
        !compareGLError(gl.NO_ERROR, "gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST)") ||
        !compareGLError(gl.NO_ERROR, "gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST)") ||
        !compareGLError(gl.NO_ERROR, "gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, arrayBufferView)") ||
        !compareGLError(gl.NO_ERROR, "gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, imageData)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform1f(uniformLocation, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform1fv(uniformLocation, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform1fv(uniformLocation, [0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform1i(uniformLocation, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform1iv(uniformLocation, int32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform1iv(uniformLocation, [0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform2f(uniformLocation, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform2fv(uniformLocation, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform2fv(uniformLocation, [0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform2i(uniformLocation, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform2iv(uniformLocation, int32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform2iv(uniformLocation, [0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform3f(uniformLocation, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform3fv(uniformLocation, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform3fv(uniformLocation, [0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform3i(uniformLocation, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform3iv(uniformLocation, int32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform3iv(uniformLocation, [0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform4f(uniformLocation, 0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform4fv(uniformLocation, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform4fv(uniformLocation, [0, 0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform4i(uniformLocation, 0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform4iv(uniformLocation, int32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniform4iv(uniformLocation, [0, 0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniformMatrix2fv(uniformLocation, false, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniformMatrix2fv(uniformLocation, false, [0, 0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniformMatrix3fv(uniformLocation, false, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniformMatrix3fv(uniformLocation, false, [0, 0, 0, 0, 0, 0, 0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.uniformMatrix4fv(uniformLocation, false, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.uniformMatrix4fv(uniformLocation, false, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.useProgram(program)") ||
        !compareGLError(gl.NO_ERROR, "gl.validateProgram(program)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib1f(0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib1fv(0, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib1fv(0, [0])") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib2f(0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib2fv(0, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib2fv(0, [0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib3f(0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib3fv(0, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib3fv(0, [0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib4f(0, 0, 0, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib4fv(0, float32array)") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttrib4fv(0, [0, 0, 0, 0])") ||
        !compareGLError(gl.NO_ERROR, "gl.vertexAttribPointer(0, 0, gl.FLOAT, false, 0, 0)") ||
        !compareGLError(gl.NO_ERROR, "gl.viewport(0, 0, 0, 0)"))
        return false;

    // Functions return nullable values should all return null.
    if (gl.createBuffer() != null ||
        gl.createFramebuffer() != null ||
        gl.createProgram() != null ||
        gl.createRenderbuffer() != null ||
        gl.createShader(gl.GL_VERTEX_SHADER) != null ||
        gl.createTexture() != null ||
        gl.getActiveAttrib(program, 0) != null ||
        gl.getActiveUniform(program, 0) != null ||
        gl.getAttachedShaders(program) != null ||
        gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE) != null ||
        gl.getContextAttributes() != null ||
        gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME) != null ||
        gl.getParameter(gl.CURRENT_PROGRAM) != null ||
        gl.getProgramInfoLog(program) != null ||
        gl.getProgramParameter(program, gl.LINK_STATUS) != null ||
        gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_WIDTH) != null ||
        gl.getShaderInfoLog(shader) != null ||
        gl.getShaderParameter(shader, gl.SHADER_TYPE) != null ||
        gl.getShaderSource(shader) != null ||
        gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S) != null ||
        gl.getUniform(program, uniformLocation) != null ||
        gl.getUniformLocation(program, 'vPosition') != null ||
        gl.getVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING) != null ||
        gl.getSupportedExtensions() != null ||
        gl.getExtension("WEBGL_lose_context") != null)
        return false;

    // "Is" queries should all return false.
    if (gl.isBuffer(buffer) || gl.isEnabled(gl.BLEND) || gl.isFramebuffer(framebuffer) ||
        gl.isProgram(program) || gl.isRenderbuffer(renderbuffer) || gl.isShader(shader) ||
        gl.isTexture(texture))
        return false;

    if (gl.getError() != gl.NO_ERROR)
        return false;

    // test extensions
    if (OES_vertex_array_object) {
        if (!compareGLError(gl.NO_ERROR, "OES_vertex_array_object.bindVertexArrayOES(vertexArrayObject)") ||
            !compareGLError(gl.NO_ERROR, "OES_vertex_array_object.isVertexArrayOES(vertexArrayObject)") ||
            !compareGLError(gl.NO_ERROR, "OES_vertex_array_object.deleteVertexArrayOES(vertexArrayObject)"))
            return false;
        if (OES_vertex_array_object.createVertexArrayOES() != null)
            return false;
    }
    return true;
}
function testValidContext()
{
    if (gl.isContextLost())
        return false;

    arrayBuffer = new ArrayBuffer(4);
    arrayBufferView = new Int8Array(arrayBuffer);

    // Generate resources for testing.
    buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    framebuffer = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
    program = setupSimpleTextureProgram(gl);
    renderbuffer = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
    shader = gl.createShader(gl.VERTEX_SHADER);
    texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    if (gl.getError() != gl.NO_ERROR)
        return false;

    // Test is queries that will later be false
    if (!compareGLError(gl.NO_ERROR, "gl.enable(gl.BLEND)"))
        return false;
    if (!gl.isBuffer(buffer) || !gl.isEnabled(gl.BLEND) || !gl.isFramebuffer(framebuffer) ||
        !gl.isProgram(program) || !gl.isRenderbuffer(renderbuffer) || !gl.isShader(shader) ||
        !gl.isTexture(texture))
        return false;

    if (OES_vertex_array_object) {
        vertexArrayObject = OES_vertex_array_object.createVertexArrayOES();
        if (gl.getError() != gl.NO_ERROR)
            return false;
        if (!OES_vertex_array_object.isVertexArrayOES(vertexArrayObject))
            return false;
    }
    return true;
}

function setupTest()
{
    canvas = new OffscreenCanvas(10, 10);
    gl = canvas.getContext('webgl');
    WEBGL_lose_context = gl.getExtension("WEBGL_lose_context");
    if (!WEBGL_lose_context)
        return false;

    // Try to get a few extensions
    OES_vertex_array_object = gl.getExtension("OES_vertex_array_object");
    OES_texture_float = gl.getExtension("OES_texture_float");

    return true;
}

function testOriginalContext()
{
    if (gl.isContextLost())
        return false;
    if (gl.getError() != gl.NO_ERROR)
        return false;
    return true;
}

function testLostContext(e)
{
    if (contextLostEventFired)
        return false;
    contextLostEventFired = true;
    if (!gl.isContextLost())
        return false;
    if (gl.getError() != gl.NO_ERROR)
        return false;
    if (allowRestore)
      e.preventDefault();
    return true;
}

function testLosingAndRestoringContext()
{
    return new Promise(function(resolve, reject) {
        if (!setupTest())
            reject("Test failed");

        canvas.addEventListener("webglcontextlost", function(e) {
            if (!testLostContext(e))
              reject("Test failed");
            // restore the context after this event has exited.
            setTimeout(function() {
                if (!compareGLError(gl.NO_ERROR, "WEBGL_lose_context.restoreContext()"))
                    reject("Test failed");
                // The context should still be lost. It will not get restored until the
                // webglrestorecontext event is fired.
                if (!gl.isContextLost())
                    reject("Test failed");
                if (gl.getError() != gl.NO_ERROR)
                    reject("Test failed");
                // gl methods should still be no-ops
                if (!compareGLError(gl.NO_ERROR, "gl.blendFunc(gl.TEXTURE_2D, gl.TEXTURE_CUBE_MAP)"))
                    reject("Test failed");
            }, 0);
        });
        canvas.addEventListener("webglcontextrestored", function() {
            if (!testRestoredContext())
                reject("Test failed");
            else
                resolve("Test passed");
        });
        allowRestore = true;
        contextLostEventFired = false;
        contextRestoredEventFired = false;

        if (!testOriginalContext())
            reject("Test failed");
        WEBGL_lose_context.loseContext();
        // The context should be lost immediately.
        if (!gl.isContextLost())
            reject("Test failed");
        if (gl.getError() != gl.CONTEXT_LOST_WEBGL)
            reject("Test failed");
        if (gl.getError() != gl.NO_ERROR)
            reject("Test failed");
        // gl methods should be no-ops
        if (!compareGLError(gl.NO_ERROR, "gl.blendFunc(gl.TEXTURE_2D, gl.TEXTURE_CUBE_MAP)"))
            reject("Test failed");
        // but the event should not have been fired.
        if (contextLostEventFired)
            reject("Test failed");
    });
}

function reGetExtensionAndTestForProperty(gl, name, expectProperty) {
    var newExtension = gl.getExtension(name);
    // NOTE: while getting a extension after context lost/restored is allowed to fail
    // for the purpose the conformance tests it is not.
    //
    // Hypothetically the user can switch GPUs live. For example on Windows, install 2 GPUs,
    // then in the control panen enable 1, disable the others and visa versa. Since the GPUs
    // have different capabilities one or the other may not support a particlar extension.
    //
    // But, for the purpose of the conformance tests the context is expected to restore
    // on the same GPU and therefore the extensions that succeeded previously should
    // succeed on restore.
    if (newExtension == null)
        return false;
    if (expectProperty) {
        if (!(newExtension.webglTestProperty === true))
            return false;
    } else {
        if (!(newExtension.webglTestProperty === undefined))
            return false;
    }
    return newExtension;
}


function testOESTextureFloat() {
  if (OES_texture_float) {
    // Extension must still be lost.
    var tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    if (!compareGLError(gl.INVALID_ENUM, "gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.FLOAT, null)"))
        return false;
    // Try re-enabling extension
    OES_texture_float = reGetExtensionAndTestForProperty(gl, "OES_texture_float", false);
    if (!compareGLError(gl.NO_ERROR, "gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.FLOAT, null)"))
        return false;
    return true;
  }
}

function testOESVertexArrayObject() {
  if (OES_vertex_array_object) {
    // Extension must still be lost.
    if (OES_vertex_array_object.createVertexArrayOES() != null)
        return false;
    // Try re-enabling extension

    var old_OES_vertex_array_object = OES_vertex_array_object;
    OES_vertex_array_object = reGetExtensionAndTestForProperty(gl, "OES_vertex_array_object", false);
    if (OES_vertex_array_object.createVertexArrayOES() == null)
        return false;
    if (old_OES_vertex_array_object.createVertexArrayOES() != null)
        return false;
    return true;
  }
}

function testExtensions() {
  if (!testOESTextureFloat() || !testOESVertexArrayObject())
    return false;
  return true;
}

function testRestoredContext()
{
    if (contextRestoredEventFired)
        return false;
    contextRestoredEventFired = true;
    if (gl.isContextLost())
        return false;
    if (gl.getError() != gl.NO_ERROR)
        return false;

    if (!testExtensions())
        return false;
    return true;
}


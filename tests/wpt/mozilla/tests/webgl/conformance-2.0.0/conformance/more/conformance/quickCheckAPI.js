/*
** Copyright (c) 2012 The Khronos Group Inc.
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

/*
  QuickCheck tests for WebGL:

    1. Write a valid arg generator for each function
      1.1. Write valid arg predicates to use with random generator:
            if value passes generator, accept it as valid.
      1.2. Often needs initializing and cleanup:
            setup - generate - cleanup
            gl.createBuffer - test(bindBufferGenerator) - gl.deleteBuffer

    2. Write an invalid arg generator
      2.1. Take valid args, modify an arg until the args no longer pass
            checkArgValidity.
      2.2. Repeat for all args.

    3. Test functions using the generators
      3.1. Args generated with the valid arg generator should pass
            assertOk(f(args))
      3.2. Args generated with the invalid arg generator should pass
            assertFail(f(args))
*/
var GLcanvas = document.createElement('canvas');
var canvas2D = document.createElement('canvas');
GLcanvas.width = GLcanvas.height = 256;
GL = getGLContext(GLcanvas);
Array.from = function(o) {
  var a = [];
  for (var i=0; i<o.length; i++)
    a.push(o[i]);
  return a;
}
Array.prototype.has = function(v) { return this.indexOf(v) != -1; }
Array.prototype.random = function() { return this[randomInt(this.length)]; }

castToInt = function(o) {
  if (typeof o == 'number')
    return isNaN(o) ? 0 : Math.floor(o);
  if (o == true) return 1;
  return 0;
};

// Creates a constant checker / generator from its arguments.
//
// E.g. if you want a constant checker for the constants 1, 2, and 3, you
// would do the following:
//
//   var cc = constCheck(1,2,3);
//   var randomConst = cc.random();
//   if (cc.has(randomConst))
//     console.log("randomConst is included in cc's constants");
//
constCheck = function() {
  var a = Array.from(arguments);
  a.has = function(v) { return this.indexOf(castToInt(v)) != -1; };
  return a;
}

bindTextureTarget = constCheck(GL.TEXTURE_2D, GL.TEXTURE_CUBE_MAP);
blendEquationMode = constCheck(GL.FUNC_ADD, GL.FUNC_SUBTRACT, GL.FUNC_REVERSE_SUBTRACT);
blendFuncSfactor = constCheck(
  GL.ZERO, GL.ONE, GL.SRC_COLOR, GL.ONE_MINUS_SRC_COLOR, GL.DST_COLOR,
  GL.ONE_MINUS_DST_COLOR, GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA, GL.DST_ALPHA,
  GL.ONE_MINUS_DST_ALPHA, GL.CONSTANT_COLOR, GL.ONE_MINUS_CONSTANT_COLOR,
  GL.CONSTANT_ALPHA, GL.ONE_MINUS_CONSTANT_ALPHA, GL.SRC_ALPHA_SATURATE
);
blendFuncDfactor = constCheck(
  GL.ZERO, GL.ONE, GL.SRC_COLOR, GL.ONE_MINUS_SRC_COLOR, GL.DST_COLOR,
  GL.ONE_MINUS_DST_COLOR, GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA, GL.DST_ALPHA,
  GL.ONE_MINUS_DST_ALPHA, GL.CONSTANT_COLOR, GL.ONE_MINUS_CONSTANT_COLOR,
  GL.CONSTANT_ALPHA, GL.ONE_MINUS_CONSTANT_ALPHA
);
bufferTarget = constCheck(GL.ARRAY_BUFFER, GL.ELEMENT_ARRAY_BUFFER);
bufferMode = constCheck(GL.STREAM_DRAW, GL.STATIC_DRAW, GL.DYNAMIC_DRAW);
clearMask = constCheck(
  GL.COLOR_BUFFER_BIT | GL.DEPTH_BUFFER_BIT | GL.STENCIL_BUFFER_BIT,
  GL.COLOR_BUFFER_BIT | GL.DEPTH_BUFFER_BIT,
  GL.COLOR_BUFFER_BIT | GL.STENCIL_BUFFER_BIT,
  GL.DEPTH_BUFFER_BIT | GL.STENCIL_BUFFER_BIT,
  GL.COLOR_BUFFER_BIT, GL.DEPTH_BUFFER_BIT, GL.STENCIL_BUFFER_BIT, 0
);
cullFace = constCheck(GL.FRONT, GL.BACK, GL.FRONT_AND_BACK);
depthFuncFunc = constCheck(
  GL.NEVER, GL.LESS, GL.EQUAL, GL.LEQUAL, GL.GREATER, GL.NOTEQUAL,
  GL.GEQUAL, GL.ALWAYS
);
stencilFuncFunc = depthFuncFunc;
enableCap = constCheck(
  GL.BLEND, GL.CULL_FACE, GL.DEPTH_TEST, GL.DITHER, GL.POLYGON_OFFSET_FILL,
  GL.SAMPLE_ALPHA_TO_COVERAGE, GL.SAMPLE_COVERAGE, GL.SCISSOR_TEST,
  GL.STENCIL_TEST
);
frontFaceMode = constCheck(GL.CCW, GL.CW);
getParameterPname = constCheck(
  GL.ACTIVE_TEXTURE || "GL.ACTIVE_TEXTURE",
  GL.ALIASED_LINE_WIDTH_RANGE || "GL.ALIASED_LINE_WIDTH_RANGE",
  GL.ALIASED_POINT_SIZE_RANGE || "GL.ALIASED_POINT_SIZE_RANGE",
  GL.ALPHA_BITS || "GL.ALPHA_BITS",
  GL.ARRAY_BUFFER_BINDING || "GL.ARRAY_BUFFER_BINDING",
  GL.BLEND || "GL.BLEND",
  GL.BLEND_COLOR || "GL.BLEND_COLOR",
  GL.BLEND_DST_ALPHA || "GL.BLEND_DST_ALPHA",
  GL.BLEND_DST_RGB || "GL.BLEND_DST_RGB",
  GL.BLEND_EQUATION_ALPHA || "GL.BLEND_EQUATION_ALPHA",
  GL.BLEND_EQUATION_RGB || "GL.BLEND_EQUATION_RGB",
  GL.BLEND_SRC_ALPHA || "GL.BLEND_SRC_ALPHA",
  GL.BLEND_SRC_RGB || "GL.BLEND_SRC_RGB",
  GL.BLUE_BITS || "GL.BLUE_BITS",
  GL.COLOR_CLEAR_VALUE || "GL.COLOR_CLEAR_VALUE",
  GL.COLOR_WRITEMASK || "GL.COLOR_WRITEMASK",
  GL.COMPRESSED_TEXTURE_FORMATS || "GL.COMPRESSED_TEXTURE_FORMATS",
  GL.CULL_FACE || "GL.CULL_FACE",
  GL.CULL_FACE_MODE || "GL.CULL_FACE_MODE",
  GL.CURRENT_PROGRAM || "GL.CURRENT_PROGRAM",
  GL.DEPTH_BITS || "GL.DEPTH_BITS",
  GL.DEPTH_CLEAR_VALUE || "GL.DEPTH_CLEAR_VALUE",
  GL.DEPTH_FUNC || "GL.DEPTH_FUNC",
  GL.DEPTH_RANGE || "GL.DEPTH_RANGE",
  GL.DEPTH_TEST || "GL.DEPTH_TEST",
  GL.DEPTH_WRITEMASK || "GL.DEPTH_WRITEMASK",
  GL.DITHER || "GL.DITHER",
  GL.ELEMENT_ARRAY_BUFFER_BINDING || "GL.ELEMENT_ARRAY_BUFFER_BINDING",
  GL.FRAMEBUFFER_BINDING || "GL.FRAMEBUFFER_BINDING",
  GL.FRONT_FACE || "GL.FRONT_FACE",
  GL.GENERATE_MIPMAP_HINT || "GL.GENERATE_MIPMAP_HINT",
  GL.GREEN_BITS || "GL.GREEN_BITS",
  GL.LINE_WIDTH || "GL.LINE_WIDTH",
  GL.MAX_COMBINED_TEXTURE_IMAGE_UNITS || "GL.MAX_COMBINED_TEXTURE_IMAGE_UNITS",
  GL.MAX_CUBE_MAP_TEXTURE_SIZE || "GL.MAX_CUBE_MAP_TEXTURE_SIZE",
  GL.MAX_FRAGMENT_UNIFORM_VECTORS || "GL.MAX_FRAGMENT_UNIFORM_VECTORS",
  GL.MAX_RENDERBUFFER_SIZE || "GL.MAX_RENDERBUFFER_SIZE",
  GL.MAX_TEXTURE_IMAGE_UNITS || "GL.MAX_TEXTURE_IMAGE_UNITS",
  GL.MAX_TEXTURE_SIZE || "GL.MAX_TEXTURE_SIZE",
  GL.MAX_VARYING_VECTORS || "GL.MAX_VARYING_VECTORS",
  GL.MAX_VERTEX_ATTRIBS || "GL.MAX_VERTEX_ATTRIBS",
  GL.MAX_VERTEX_TEXTURE_IMAGE_UNITS || "GL.MAX_VERTEX_TEXTURE_IMAGE_UNITS",
  GL.MAX_VERTEX_UNIFORM_VECTORS || "GL.MAX_VERTEX_UNIFORM_VECTORS",
  GL.MAX_VIEWPORT_DIMS || "GL.MAX_VIEWPORT_DIMS",
  GL.PACK_ALIGNMENT || "GL.PACK_ALIGNMENT",
  GL.POLYGON_OFFSET_FACTOR || "GL.POLYGON_OFFSET_FACTOR",
  GL.POLYGON_OFFSET_FILL || "GL.POLYGON_OFFSET_FILL",
  GL.POLYGON_OFFSET_UNITS || "GL.POLYGON_OFFSET_UNITS",
  GL.RED_BITS || "GL.RED_BITS",
  GL.RENDERBUFFER_BINDING || "GL.RENDERBUFFER_BINDING",
  GL.SAMPLE_BUFFERS || "GL.SAMPLE_BUFFERS",
  GL.SAMPLE_COVERAGE_INVERT || "GL.SAMPLE_COVERAGE_INVERT",
  GL.SAMPLE_COVERAGE_VALUE || "GL.SAMPLE_COVERAGE_VALUE",
  GL.SAMPLES || "GL.SAMPLES",
  GL.SCISSOR_BOX || "GL.SCISSOR_BOX",
  GL.SCISSOR_TEST || "GL.SCISSOR_TEST",
  GL.STENCIL_BACK_FAIL || "GL.STENCIL_BACK_FAIL",
  GL.STENCIL_BACK_FUNC || "GL.STENCIL_BACK_FUNC",
  GL.STENCIL_BACK_PASS_DEPTH_FAIL || "GL.STENCIL_BACK_PASS_DEPTH_FAIL",
  GL.STENCIL_BACK_PASS_DEPTH_PASS || "GL.STENCIL_BACK_PASS_DEPTH_PASS",
  GL.STENCIL_BACK_REF || "GL.STENCIL_BACK_REF",
  GL.STENCIL_BACK_VALUE_MASK || "GL.STENCIL_BACK_VALUE_MASK",
  GL.STENCIL_BACK_WRITEMASK || "GL.STENCIL_BACK_WRITEMASK",
  GL.STENCIL_BITS || "GL.STENCIL_BITS",
  GL.STENCIL_CLEAR_VALUE || "GL.STENCIL_CLEAR_VALUE",
  GL.STENCIL_FAIL || "GL.STENCIL_FAIL",
  GL.STENCIL_FUNC || "GL.STENCIL_FUNC",
  GL.STENCIL_PASS_DEPTH_FAIL || "GL.STENCIL_PASS_DEPTH_FAIL",
  GL.STENCIL_PASS_DEPTH_PASS || "GL.STENCIL_PASS_DEPTH_PASS",
  GL.STENCIL_REF || "GL.STENCIL_REF",
  GL.STENCIL_TEST || "GL.STENCIL_TEST",
  GL.STENCIL_VALUE_MASK || "GL.STENCIL_VALUE_MASK",
  GL.STENCIL_WRITEMASK || "GL.STENCIL_WRITEMASK",
  GL.SUBPIXEL_BITS || "GL.SUBPIXEL_BITS",
  GL.TEXTURE_BINDING_2D || "GL.TEXTURE_BINDING_2D",
  GL.TEXTURE_BINDING_CUBE_MAP || "GL.TEXTURE_BINDING_CUBE_MAP",
  GL.UNPACK_ALIGNMENT || "GL.UNPACK_ALIGNMENT",
  GL.VIEWPORT || "GL.VIEWPORT"
);
mipmapHint = constCheck(GL.FASTEST, GL.NICEST, GL.DONT_CARE);
pixelStoreiPname = constCheck(GL.PACK_ALIGNMENT, GL.UNPACK_ALIGNMENT);
pixelStoreiParam = constCheck(1,2,4,8);
shaderType = constCheck(GL.VERTEX_SHADER, GL.FRAGMENT_SHADER);
stencilOp = constCheck(GL.KEEP, GL.ZERO, GL.REPLACE, GL.INCR, GL.INCR_WRAP,
                        GL.DECR, GL.DECR_WRAP, GL.INVERT);
texImageTarget = constCheck(
  GL.TEXTURE_2D,
  GL.TEXTURE_CUBE_MAP_POSITIVE_X,
  GL.TEXTURE_CUBE_MAP_NEGATIVE_X,
  GL.TEXTURE_CUBE_MAP_POSITIVE_Y,
  GL.TEXTURE_CUBE_MAP_NEGATIVE_Y,
  GL.TEXTURE_CUBE_MAP_POSITIVE_Z,
  GL.TEXTURE_CUBE_MAP_NEGATIVE_Z
);
texImageInternalFormat = constCheck(
  GL.ALPHA,
  GL.LUMINANCE,
  GL.LUMINANCE_ALPHA,
  GL.RGB,
  GL.RGBA
);
texImageFormat = constCheck(
  GL.ALPHA,
  GL.LUMINANCE,
  GL.LUMINANCE_ALPHA,
  GL.RGB,
  GL.RGBA
);
texImageType = constCheck(GL.UNSIGNED_BYTE);
texParameterPname = constCheck(
  GL.TEXTURE_MIN_FILTER, GL.TEXTURE_MAG_FILTER,
  GL.TEXTURE_WRAP_S, GL.TEXTURE_WRAP_T);
texParameterParam = {};
texParameterParam[GL.TEXTURE_MIN_FILTER] = constCheck(
  GL.NEAREST, GL.LINEAR, GL.NEAREST_MIPMAP_NEAREST, GL.LINEAR_MIPMAP_NEAREST,
  GL.NEAREST_MIPMAP_LINEAR, GL.LINEAR_MIPMAP_LINEAR);
texParameterParam[GL.TEXTURE_MAG_FILTER] = constCheck(GL.NEAREST, GL.LINEAR);
texParameterParam[GL.TEXTURE_WRAP_S] = constCheck(
  GL.CLAMP_TO_EDGE, GL.MIRRORED_REPEAT, GL.REPEAT);
texParameterParam[GL.TEXTURE_WRAP_T] = texParameterParam[GL.TEXTURE_WRAP_S];
textureUnit = constCheck.apply(this, (function(){
  var textureUnits = [];
  var texUnits = GL.getParameter(GL.MAX_TEXTURE_IMAGE_UNITS);
  for (var i=0; i<texUnits; i++) textureUnits.push(GL['TEXTURE'+i]);
  return textureUnits;
})());

var StencilBits = GL.getParameter(GL.STENCIL_BITS);
var MaxStencilValue = 1 << StencilBits;

var MaxVertexAttribs = GL.getParameter(GL.MAX_VERTEX_ATTRIBS);
var LineWidthRange = GL.getParameter(GL.ALIASED_LINE_WIDTH_RANGE);

// Returns true if bufData can be passed to GL.bufferData
isBufferData = function(bufData) {
  if (typeof bufData == 'number')
    return bufData >= 0;
  if (bufData instanceof ArrayBuffer)
    return true;
  return WebGLArrayTypes.some(function(t) {
    return bufData instanceof t;
  });
};

isVertexAttribute = function(idx) {
  if (typeof idx != 'number') return false;
  return idx >= 0 && idx < MaxVertexAttribs;
};

isValidName = function(name) {
  if (typeof name != 'string') return false;
  for (var i=0; i<name.length; i++) {
    var c = name.charCodeAt(i);
    if (c & 0x00FF == 0 || c & 0xFF00 == 0) {
      return false;
    }
  }
  return true;
};

WebGLArrayTypes = [
  Float32Array,
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array
];
webGLArrayContentGenerators = [randomLength, randomSmallIntArray];
randomWebGLArray = function() {
  var t = WebGLArrayTypes.random();
  return new t(webGLArrayContentGenerators.random()());
};

randomArrayBuffer = function(buflen) {
  if (buflen == null) buflen = 256;
  var len = randomInt(buflen)+1;
  var rv;
  try {
    rv = new ArrayBuffer(len);
  } catch(e) {
    log("Error creating ArrayBuffer with length " + len);
    throw(e);
  }
  return rv;
};

bufferDataGenerators = [randomLength, randomWebGLArray, randomArrayBuffer];
randomBufferData = function() {
  return bufferDataGenerators.random()();
};

randomSmallWebGLArray = function(buflen) {
  var t = WebGLArrayTypes.random();
  return new t(randomInt(buflen/4)+1);
};

bufferSubDataGenerators = [randomSmallWebGLArray, randomArrayBuffer];
randomBufferSubData = function(buflen) {
  var data = bufferSubDataGenerators.random()(buflen);
  var offset = randomInt(buflen - data.byteLength);
  return {data:data, offset:offset};
};

randomColor = function() {
  return [Math.random(), Math.random(), Math.random(), Math.random()];
};

randomName = function() {
  var arr = [];
  var len = randomLength()+1;
  for (var i=0; i<len; i++) {
    var l = randomInt(255)+1;
    var h = randomInt(255)+1;
    var c = (h << 8) | l;
    arr.push(String.fromCharCode(c));
  }
  return arr.join('');
};
randomVertexAttribute = function() {
  return randomInt(MaxVertexAttribs);
};

randomBool = function() { return Math.random() > 0.5; };

randomStencil = function() {
  return randomInt(MaxStencilValue);
};

randomLineWidth = function() {
  var lo = LineWidthRange[0],
      hi = LineWidthRange[1];
  return randomFloatFromRange(lo, hi);
};

randomImage = function(w,h) {
  var img;
  var r = Math.random();
  if (r < 0.25) {
    img = document.createElement('canvas');
    img.width = w; img.height = h;
    img.getContext('2d').fillRect(0,0,w,h);
  } else if (r < 0.5) {
    img = document.createElement('video');
    img.width = w; img.height = h;
  } else if (r < 0.75) {
    img = document.createElement('img');
    img.width = w; img.height = h;
  } else {
    img = canvas2D.getContext('2d').createImageData(w,h);
  }
  return img
};

mutateArgs = function(args) {
  var mutateCount = randomIntFromRange(1, args.length);
  var newArgs = Array.from(args);
  for (var i=0; i<mutateCount; i++) {
    var idx = randomInt(args.length);
    newArgs[idx] = generateRandomArg(idx, args.length);
  }
  return newArgs;
};

// Calls testFunction numberOfTests times with arguments generated by
// argGen.generate() (or empty arguments if no generate present).
//
// The arguments testFunction is called with are the generated args,
// the argGen, and what argGen.setup() returned or [] if argGen has not setup
// method. I.e. testFunction(generatedArgs, argGen, setupVars).
//
argGeneratorTestRunner = function(argGen, testFunction, numberOfTests) {
  // do argument generator setup if needed
  var setupVars = argGen.setup ? argGen.setup() : [];
  var error;
  for (var i=0; i<numberOfTests; i++) {
    var failed = false;
    // generate arguments if argGen has a generate method
    var generatedArgs = argGen.generate ? argGen.generate.apply(argGen, setupVars) : [];
    try {
      // call testFunction with the generated args
      testFunction.call(this, generatedArgs, argGen, setupVars);
    } catch (e) {
      failed = true;
      error = e;
    }
    // if argGen needs cleanup for generated args, do it here
    if (argGen.cleanup)
      argGen.cleanup.apply(argGen, generatedArgs);
    if (failed) break;
  }
  // if argGen needs to do a final cleanup for setupVars, do it here
  if (argGen.teardown)
    argGen.teardown.apply(argGen, setupVars);
  if (error) throw(error);
};

// TODO: Remove this
// WebKit or at least Chrome is really slow at laying out strings with
// unprintable characters. Without this tests can take 30-90 seconds.
// With this they're instant.
sanitize = function(str) {
  var newStr = [];
  for (var ii = 0; ii < str.length; ++ii) {
    var c = str.charCodeAt(ii);
    newStr.push((c > 31 && c < 128) ? str[ii] : "?");
  }
  return newStr.join('');
};

argsToString = function(args) {
  return sanitize(args.map(function(a){return Object.toSource(a)}).join(","));
};

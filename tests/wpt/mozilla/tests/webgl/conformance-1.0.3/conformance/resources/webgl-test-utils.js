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
var WebGLTestUtils = (function() {
"use strict";

/**
 * Wrapped logging function.
 * @param {string} msg The message to log.
 */
var log = function(msg) {
  if (window.console && window.console.log) {
    window.console.log(msg);
  }
};

/**
 * Wrapped logging function.
 * @param {string} msg The message to log.
 */
var error = function(msg) {
  if (window.console) {
    if (window.console.error) {
      window.console.error(msg);
    }
    else if (window.console.log) {
      window.console.log(msg);
    }
  }
};

/**
 * Turn off all logging.
 */
var loggingOff = function() {
  log = function() {};
  error = function() {};
};

/**
 * Converts a WebGL enum to a string.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} value The enum value.
 * @return {string} The enum as a string.
 */
var glEnumToString = function(gl, value) {
  // Optimization for the most common enum:
  if (value === gl.NO_ERROR) {
    return "NO_ERROR";
  }
  for (var p in gl) {
    if (gl[p] == value) {
      return p;
    }
  }
  return "0x" + value.toString(16);
};

var lastError = "";

/**
 * Returns the last compiler/linker error.
 * @return {string} The last compiler/linker error.
 */
var getLastError = function() {
  return lastError;
};

/**
 * Whether a haystack ends with a needle.
 * @param {string} haystack String to search
 * @param {string} needle String to search for.
 * @param {boolean} True if haystack ends with needle.
 */
var endsWith = function(haystack, needle) {
  return haystack.substr(haystack.length - needle.length) === needle;
};

/**
 * Whether a haystack starts with a needle.
 * @param {string} haystack String to search
 * @param {string} needle String to search for.
 * @param {boolean} True if haystack starts with needle.
 */
var startsWith = function(haystack, needle) {
  return haystack.substr(0, needle.length) === needle;
};

/**
 * A vertex shader for a single texture.
 * @type {string}
 */
var simpleTextureVertexShader = [
  'attribute vec4 vPosition;',
  'attribute vec2 texCoord0;',
  'varying vec2 texCoord;',
  'void main() {',
  '    gl_Position = vPosition;',
  '    texCoord = texCoord0;',
  '}'].join('\n');

/**
 * A fragment shader for a single texture.
 * @type {string}
 */
var simpleTextureFragmentShader = [
  'precision mediump float;',
  'uniform sampler2D tex;',
  'varying vec2 texCoord;',
  'void main() {',
  '    gl_FragData[0] = texture2D(tex, texCoord);',
  '}'].join('\n');

/**
 * A vertex shader for a single texture.
 * @type {string}
 */
var noTexCoordTextureVertexShader = [
  'attribute vec4 vPosition;',
  'varying vec2 texCoord;',
  'void main() {',
  '    gl_Position = vPosition;',
  '    texCoord = vPosition.xy * 0.5 + 0.5;',
  '}'].join('\n');

/**
 * A vertex shader for a uniform color.
 * @type {string}
 */
var simpleColorVertexShader = [
  'attribute vec4 vPosition;',
  'void main() {',
  '    gl_Position = vPosition;',
  '}'].join('\n');

/**
 * A fragment shader for a uniform color.
 * @type {string}
 */
var simpleColorFragmentShader = [
  'precision mediump float;',
  'uniform vec4 u_color;',
  'void main() {',
  '    gl_FragData[0] = u_color;',
  '}'].join('\n');

/**
 * A vertex shader for vertex colors.
 * @type {string}
 */
var simpleVertexColorVertexShader = [
  'attribute vec4 vPosition;',
  'attribute vec4 a_color;',
  'varying vec4 v_color;',
  'void main() {',
  '    gl_Position = vPosition;',
  '    v_color = a_color;',
  '}'].join('\n');

/**
 * A fragment shader for vertex colors.
 * @type {string}
 */
var simpleVertexColorFragmentShader = [
  'precision mediump float;',
  'varying vec4 v_color;',
  'void main() {',
  '    gl_FragData[0] = v_color;',
  '}'].join('\n');

/**
 * Creates a simple texture vertex shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupSimpleTextureVertexShader = function(gl) {
    return loadShader(gl, simpleTextureVertexShader, gl.VERTEX_SHADER);
};

/**
 * Creates a simple texture fragment shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupSimpleTextureFragmentShader = function(gl) {
    return loadShader(
        gl, simpleTextureFragmentShader, gl.FRAGMENT_SHADER);
};

/**
 * Creates a texture vertex shader that doesn't need texcoords.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupNoTexCoordTextureVertexShader = function(gl) {
    return loadShader(gl, noTexCoordTextureVertexShader, gl.VERTEX_SHADER);
};

/**
 * Creates a simple vertex color vertex shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupSimpleVertexColorVertexShader = function(gl) {
    return loadShader(gl, simpleVertexColorVertexShader, gl.VERTEX_SHADER);
};

/**
 * Creates a simple vertex color fragment shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupSimpleVertexColorFragmentShader = function(gl) {
    return loadShader(
        gl, simpleVertexColorFragmentShader, gl.FRAGMENT_SHADER);
};

/**
 * Creates a simple color vertex shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupSimpleColorVertexShader = function(gl) {
    return loadShader(gl, simpleColorVertexShader, gl.VERTEX_SHADER);
};

/**
 * Creates a simple color fragment shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {!WebGLShader}
 */
var setupSimpleColorFragmentShader = function(gl) {
    return loadShader(
        gl, simpleColorFragmentShader, gl.FRAGMENT_SHADER);
};

/**
 * Creates a program, attaches shaders, binds attrib locations, links the
 * program and calls useProgram.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<!WebGLShader|string>} shaders The shaders to
 *        attach, or the source, or the id of a script to get
 *        the source from.
 * @param {!Array.<string>} opt_attribs The attribs names.
 * @param {!Array.<number>} opt_locations The locations for the attribs.
 * @param {boolean} opt_logShaders Whether to log shader source.
 */
var setupProgram = function(
    gl, shaders, opt_attribs, opt_locations, opt_logShaders) {
  var realShaders = [];
  var program = gl.createProgram();
  var shaderCount = 0;
  for (var ii = 0; ii < shaders.length; ++ii) {
    var shader = shaders[ii];
    var shaderType = undefined;
    if (typeof shader == 'string') {
      var element = document.getElementById(shader);
      if (element) {
        if (element.type != "x-shader/x-vertex" && element.type != "x-shader/x-fragment")
          shaderType = ii ? gl.FRAGMENT_SHADER : gl.VERTEX_SHADER;
        shader = loadShaderFromScript(gl, shader, shaderType, undefined, opt_logShaders);
      } else if (endsWith(shader, ".vert")) {
        shader = loadShaderFromFile(gl, shader, gl.VERTEX_SHADER, undefined, opt_logShaders);
      } else if (endsWith(shader, ".frag")) {
        shader = loadShaderFromFile(gl, shader, gl.FRAGMENT_SHADER, undefined, opt_logShaders);
      } else {
        shader = loadShader(gl, shader, ii ? gl.FRAGMENT_SHADER : gl.VERTEX_SHADER, undefined, opt_logShaders);
      }
    } else if (opt_logShaders) {
      throw 'Shader source logging requested but no shader source provided';
    }
    if (shader) {
      ++shaderCount;
      gl.attachShader(program, shader);
    }
  }
  if (shaderCount != 2) {
    error("Error in compiling shader");
    return null;
  }
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
      lastError = gl.getProgramInfoLog (program);
      error("Error in program linking:" + lastError);

      gl.deleteProgram(program);
      return null;
  }

  gl.useProgram(program);
  return program;
};

/**
 * Creates a simple texture program.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @return {WebGLProgram}
 */
var setupNoTexCoordTextureProgram = function(gl) {
  var vs = setupNoTexCoordTextureVertexShader(gl);
  var fs = setupSimpleTextureFragmentShader(gl);
  if (!vs || !fs) {
    return null;
  }
  var program = setupProgram(
      gl,
      [vs, fs],
      ['vPosition'],
      [0]);
  if (!program) {
    gl.deleteShader(fs);
    gl.deleteShader(vs);
  }
  gl.useProgram(program);
  return program;
};

/**
 * Creates a simple texture program.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} opt_positionLocation The attrib location for position.
 * @param {number} opt_texcoordLocation The attrib location for texture coords.
 * @return {WebGLProgram}
 */
var setupSimpleTextureProgram = function(
    gl, opt_positionLocation, opt_texcoordLocation) {
  opt_positionLocation = opt_positionLocation || 0;
  opt_texcoordLocation = opt_texcoordLocation || 1;
  var vs = setupSimpleTextureVertexShader(gl);
  var fs = setupSimpleTextureFragmentShader(gl);
  if (!vs || !fs) {
    return null;
  }
  var program = setupProgram(
      gl,
      [vs, fs],
      ['vPosition', 'texCoord0'],
      [opt_positionLocation, opt_texcoordLocation]);
  if (!program) {
    gl.deleteShader(fs);
    gl.deleteShader(vs);
  }
  gl.useProgram(program);
  return program;
};

/**
 * Creates a simple vertex color program.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} opt_positionLocation The attrib location for position.
 * @param {number} opt_vertexColorLocation The attrib location
 *        for vertex colors.
 * @return {WebGLProgram}
 */
var setupSimpleVertexColorProgram = function(
    gl, opt_positionLocation, opt_vertexColorLocation) {
  opt_positionLocation = opt_positionLocation || 0;
  opt_vertexColorLocation = opt_vertexColorLocation || 1;
  var vs = setupSimpleVertexColorVertexShader(gl);
  var fs = setupSimpleVertexColorFragmentShader(gl);
  if (!vs || !fs) {
    return null;
  }
  var program = setupProgram(
      gl,
      [vs, fs],
      ['vPosition', 'a_color'],
      [opt_positionLocation, opt_vertexColorLocation]);
  if (!program) {
    gl.deleteShader(fs);
    gl.deleteShader(vs);
  }
  gl.useProgram(program);
  return program;
};

/**
 * Creates a simple color program.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} opt_positionLocation The attrib location for position.
 * @return {WebGLProgram}
 */
var setupSimpleColorProgram = function(gl, opt_positionLocation) {
  opt_positionLocation = opt_positionLocation || 0;
  var vs = setupSimpleColorVertexShader(gl);
  var fs = setupSimpleColorFragmentShader(gl);
  if (!vs || !fs) {
    return null;
  }
  var program = setupProgram(
      gl,
      [vs, fs],
      ['vPosition'],
      [opt_positionLocation]);
  if (!program) {
    gl.deleteShader(fs);
    gl.deleteShader(vs);
  }
  gl.useProgram(program);
  return program;
};

/**
 * Creates buffers for a textured unit quad and attaches them to vertex attribs.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} opt_positionLocation The attrib location for position.
 * @param {number} opt_texcoordLocation The attrib location for texture coords.
 * @return {!Array.<WebGLBuffer>} The buffer objects that were
 *      created.
 */
var setupUnitQuad = function(gl, opt_positionLocation, opt_texcoordLocation) {
  return setupUnitQuadWithTexCoords(gl, [ 0.0, 0.0 ], [ 1.0, 1.0 ],
                                    opt_positionLocation, opt_texcoordLocation);
};

/**
 * Creates buffers for a textured unit quad with specified lower left
 * and upper right texture coordinates, and attaches them to vertex
 * attribs.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number>} lowerLeftTexCoords The texture coordinates for the lower left corner.
 * @param {!Array.<number>} upperRightTexCoords The texture coordinates for the upper right corner.
 * @param {number} opt_positionLocation The attrib location for position.
 * @param {number} opt_texcoordLocation The attrib location for texture coords.
 * @return {!Array.<WebGLBuffer>} The buffer objects that were
 *      created.
 */
var setupUnitQuadWithTexCoords = function(
    gl, lowerLeftTexCoords, upperRightTexCoords,
    opt_positionLocation, opt_texcoordLocation) {
  return setupQuad(gl, {
    positionLocation: opt_positionLocation || 0,
    texcoordLocation: opt_texcoordLocation || 1,
    lowerLeftTexCoords: lowerLeftTexCoords,
    upperRightTexCoords: upperRightTexCoords,
  });
};

/**
 * Makes a quad with various options.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Object} options.
 *
 * scale: scale to multiple unit quad values by. default 1.0.
 * positionLocation: attribute location for position.
 * texcoordLocation: attribute location for texcoords.
 *     If this does not exist no texture coords are created.
 * lowerLeftTexCoords: an array of 2 values for the
 *     lowerLeftTexCoords.
 * upperRightTexCoords: an array of 2 values for the
 *     upperRightTexCoords.
 */
var setupQuad = function(gl, options) {
  var positionLocation = options.positionLocation || 0;
  var scale = options.scale || 1;

  var objects = [];

  var vertexObject = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, vertexObject);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
       1.0 * scale ,  1.0 * scale,
      -1.0 * scale ,  1.0 * scale,
      -1.0 * scale , -1.0 * scale,
       1.0 * scale ,  1.0 * scale,
      -1.0 * scale , -1.0 * scale,
       1.0 * scale , -1.0 * scale,]), gl.STATIC_DRAW);
  gl.enableVertexAttribArray(positionLocation);
  gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);
  objects.push(vertexObject);

  if (options.texcoordLocation !== undefined) {
    var llx = options.lowerLeftTexCoords[0];
    var lly = options.lowerLeftTexCoords[1];
    var urx = options.upperRightTexCoords[0];
    var ury = options.upperRightTexCoords[1];

    var vertexObject = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexObject);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
        urx, ury,
        llx, ury,
        llx, lly,
        urx, ury,
        llx, lly,
        urx, lly]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(options.texcoordLocation);
    gl.vertexAttribPointer(options.texcoordLocation, 2, gl.FLOAT, false, 0, 0);
    objects.push(vertexObject);
  }

  return objects;
};

/**
 * Creates a program and buffers for rendering a textured quad.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} opt_positionLocation The attrib location for
 *        position. Default = 0.
 * @param {number} opt_texcoordLocation The attrib location for
 *        texture coords. Default = 1.
 * @return {!WebGLProgram}
 */
var setupTexturedQuad = function(
    gl, opt_positionLocation, opt_texcoordLocation) {
  var program = setupSimpleTextureProgram(
      gl, opt_positionLocation, opt_texcoordLocation);
  setupUnitQuad(gl, opt_positionLocation, opt_texcoordLocation);
  return program;
};

/**
 * Creates a program and buffers for rendering a color quad.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} opt_positionLocation The attrib location for position.
 * @return {!WebGLProgram}
 */
var setupColorQuad = function(gl, opt_positionLocation) {
  opt_positionLocation = opt_positionLocation || 0;
  var program = setupSimpleColorProgram(gl);
  setupUnitQuad(gl, opt_positionLocation);
  return program;
};

/**
 * Creates a program and buffers for rendering a textured quad with
 * specified lower left and upper right texture coordinates.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number>} lowerLeftTexCoords The texture coordinates for the lower left corner.
 * @param {!Array.<number>} upperRightTexCoords The texture coordinates for the upper right corner.
 * @param {number} opt_positionLocation The attrib location for position.
 * @param {number} opt_texcoordLocation The attrib location for texture coords.
 * @return {!WebGLProgram}
 */
var setupTexturedQuadWithTexCoords = function(
    gl, lowerLeftTexCoords, upperRightTexCoords,
    opt_positionLocation, opt_texcoordLocation) {
  var program = setupSimpleTextureProgram(
      gl, opt_positionLocation, opt_texcoordLocation);
  setupUnitQuadWithTexCoords(gl, lowerLeftTexCoords, upperRightTexCoords,
                             opt_positionLocation, opt_texcoordLocation);
  return program;
};

/**
 * Creates a unit quad with only positions of a given resolution.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} gridRes The resolution of the mesh grid,
 *     expressed in the number of quads across and down.
 * @param {number} opt_positionLocation The attrib location for position.
 */
var setupIndexedQuad = function (
    gl, gridRes, opt_positionLocation, opt_flipOddTriangles) {
  return setupIndexedQuadWithOptions(gl,
    { gridRes: gridRes,
      positionLocation: opt_positionLocation,
      flipOddTriangles: opt_flipOddTriangles
    });
};

/**
 * Creates a quad with various options.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Object) options The options. See below.
 * @return {!Array.<WebGLBuffer>} The created buffers.
 *     [positions, <colors>, indices]
 *
 * Options:
 *   gridRes: number of quads across and down grid.
 *   positionLocation: attrib location for position
 *   flipOddTriangles: reverse order of vertices of every other
 *       triangle
 *   positionOffset: offset added to each vertex
 *   positionMult: multipier for each vertex
 *   colorLocation: attrib location for vertex colors. If
 *      undefined no vertex colors will be created.
 */
var setupIndexedQuadWithOptions = function (gl, options) {
  var positionLocation = options.positionLocation || 0;
  var objects = [];

  var gridRes = options.gridRes || 1;
  var positionOffset = options.positionOffset || 0;
  var positionMult = options.positionMult || 1;
  var vertsAcross = gridRes + 1;
  var numVerts = vertsAcross * vertsAcross;
  var positions = new Float32Array(numVerts * 3);
  var indices = new Uint16Array(6 * gridRes * gridRes);
  var poffset = 0;

  for (var yy = 0; yy <= gridRes; ++yy) {
    for (var xx = 0; xx <= gridRes; ++xx) {
      positions[poffset + 0] = (-1 + 2 * xx / gridRes) * positionMult + positionOffset;
      positions[poffset + 1] = (-1 + 2 * yy / gridRes) * positionMult + positionOffset;
      positions[poffset + 2] = 0;

      poffset += 3;
    }
  }

  var buf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buf);
  gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
  gl.enableVertexAttribArray(positionLocation);
  gl.vertexAttribPointer(positionLocation, 3, gl.FLOAT, false, 0, 0);
  objects.push(buf);

  if (options.colorLocation !== undefined) {
    var colors = new Float32Array(numVerts * 4);
    for (var yy = 0; yy <= gridRes; ++yy) {
      for (var xx = 0; xx <= gridRes; ++xx) {
        if (options.color !== undefined) {
          colors[poffset + 0] = options.color[0];
          colors[poffset + 1] = options.color[1];
          colors[poffset + 2] = options.color[2];
          colors[poffset + 3] = options.color[3];
        } else {
          colors[poffset + 0] = xx / gridRes;
          colors[poffset + 1] = yy / gridRes;
          colors[poffset + 2] = (xx / gridRes) * (yy / gridRes);
          colors[poffset + 3] = (yy % 2) * 0.5 + 0.5;
        }
        poffset += 4;
      }
    }

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, colors, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(options.colorLocation);
    gl.vertexAttribPointer(options.colorLocation, 4, gl.FLOAT, false, 0, 0);
    objects.push(buf);
  }

  var tbase = 0;
  for (var yy = 0; yy < gridRes; ++yy) {
    var index = yy * vertsAcross;
    for (var xx = 0; xx < gridRes; ++xx) {
      indices[tbase + 0] = index + 0;
      indices[tbase + 1] = index + 1;
      indices[tbase + 2] = index + vertsAcross;
      indices[tbase + 3] = index + vertsAcross;
      indices[tbase + 4] = index + 1;
      indices[tbase + 5] = index + vertsAcross + 1;

      if (options.flipOddTriangles) {
        indices[tbase + 4] = index + vertsAcross + 1;
        indices[tbase + 5] = index + 1;
      }

      index += 1;
      tbase += 6;
    }
  }

  var buf = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buf);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
  objects.push(buf);

  return objects;
};

/**
 * Returns the constructor for a typed array that corresponds to the given
 * WebGL type.
 * @param {!WebGLRenderingContext} gl A WebGLRenderingContext.
 * @param {number} type The WebGL type (eg, gl.UNSIGNED_BYTE)
 * @return {!Constructor} The typed array constructor that
 *      corresponds to the given type.
 */
var glTypeToTypedArrayType = function(gl, type) {
  switch (type) {
    case gl.BYTE:
      return window.Int8Array;
    case gl.UNSIGNED_BYTE:
      return window.Uint8Array;
    case gl.SHORT:
      return window.Int16Array;
    case gl.UNSIGNED_SHORT:
    case gl.UNSIGNED_SHORT_5_6_5:
    case gl.UNSIGNED_SHORT_4_4_4_4:
    case gl.UNSIGNED_SHORT_5_5_5_1:
      return window.Uint16Array;
    case gl.INT:
      return window.Int32Array;
    case gl.UNSIGNED_INT:
      return window.Uint32Array;
    default:
      throw 'unknown gl type ' + glEnumToString(gl, type);
  }
};

/**
 * Returns the number of bytes per component for a given WebGL type.
 * @param {!WebGLRenderingContext} gl A WebGLRenderingContext.
 * @param {GLenum} type The WebGL type (eg, gl.UNSIGNED_BYTE)
 * @return {number} The number of bytes per component.
 */
var getBytesPerComponent = function(gl, type) {
  switch (type) {
    case gl.BYTE:
    case gl.UNSIGNED_BYTE:
      return 1;
    case gl.SHORT:
    case gl.UNSIGNED_SHORT:
    case gl.UNSIGNED_SHORT_5_6_5:
    case gl.UNSIGNED_SHORT_4_4_4_4:
    case gl.UNSIGNED_SHORT_5_5_5_1:
      return 2;
    case gl.INT:
    case gl.UNSIGNED_INT:
      return 4;
    default:
      throw 'unknown gl type ' + glEnumToString(gl, type);
  }
};

/**
 * Returns the number of typed array elements per pixel for a given WebGL
 * format/type combination. The corresponding typed array type can be determined
 * by calling glTypeToTypedArrayType.
 * @param {!WebGLRenderingContext} gl A WebGLRenderingContext.
 * @param {GLenum} format The WebGL format (eg, gl.RGBA)
 * @param {GLenum} type The WebGL type (eg, gl.UNSIGNED_BYTE)
 * @return {number} The number of typed array elements per pixel.
 */
var getTypedArrayElementsPerPixel = function(gl, format, type) {
  switch (type) {
    case gl.UNSIGNED_SHORT_5_6_5:
    case gl.UNSIGNED_SHORT_4_4_4_4:
    case gl.UNSIGNED_SHORT_5_5_5_1:
      return 1;
    case gl.UNSIGNED_BYTE:
      break;
    default:
      throw 'not a gl type for color information ' + glEnumToString(gl, type);
  }

  switch (format) {
    case gl.RGBA:
      return 4;
    case gl.RGB:
      return 3;
    case gl.LUMINANCE_ALPHA:
      return 2;
    case gl.LUMINANCE:
    case gl.ALPHA:
      return 1;
    default:
      throw 'unknown gl format ' + glEnumToString(gl, format);
  }
};

/**
 * Fills the given texture with a solid color.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!WebGLTexture} tex The texture to fill.
 * @param {number} width The width of the texture to create.
 * @param {number} height The height of the texture to create.
 * @param {!Array.<number>} color The color to fill with.
 *        where each element is in the range 0 to 255.
 * @param {number} opt_level The level of the texture to fill. Default = 0.
 * @param {number} opt_format The format for the texture.
 */
var fillTexture = function(gl, tex, width, height, color, opt_level, opt_format, opt_type) {
  opt_level = opt_level || 0;
  opt_format = opt_format || gl.RGBA;
  opt_type = opt_type || gl.UNSIGNED_BYTE;
  var pack = gl.getParameter(gl.UNPACK_ALIGNMENT);
  var numComponents = color.length;
  var bytesPerComponent = getBytesPerComponent(gl, opt_type);
  var rowSize = numComponents * width * bytesPerComponent;
  var paddedRowSize = Math.floor((rowSize + pack - 1) / pack) * pack;
  var size = rowSize + (height - 1) * paddedRowSize;
  size = Math.floor((size + bytesPerComponent - 1) / bytesPerComponent) * bytesPerComponent;
  var buf = new (glTypeToTypedArrayType(gl, opt_type))(size);
  for (var yy = 0; yy < height; ++yy) {
    var off = yy * paddedRowSize;
    for (var xx = 0; xx < width; ++xx) {
      for (var jj = 0; jj < numComponents; ++jj) {
        buf[off++] = color[jj];
      }
    }
  }
  gl.bindTexture(gl.TEXTURE_2D, tex);
  gl.texImage2D(
      gl.TEXTURE_2D, opt_level, opt_format, width, height, 0,
      opt_format, opt_type, buf);
};

/**
 * Creates a texture and fills it with a solid color.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} width The width of the texture to create.
 * @param {number} height The height of the texture to create.
 * @param {!Array.<number>} color The color to fill with. A 4 element array
 *        where each element is in the range 0 to 255.
 * @return {!WebGLTexture}
 */
var createColoredTexture = function(gl, width, height, color) {
  var tex = gl.createTexture();
  fillTexture(gl, tex, width, height, color);
  return tex;
};

var ubyteToFloat = function(c) {
  return c / 255;
};

var ubyteColorToFloatColor = function(color) {
  var floatColor = [];
  for (var ii = 0; ii < color.length; ++ii) {
    floatColor[ii] = ubyteToFloat(color[ii]);
  }
  return floatColor;
};

/**
 * Sets the "u_color" uniform of the current program to color.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number> color 4 element array of 0-1 color
 *      components.
 */
var setFloatDrawColor = function(gl, color) {
  var program = gl.getParameter(gl.CURRENT_PROGRAM);
  var colorLocation = gl.getUniformLocation(program, "u_color");
  gl.uniform4fv(colorLocation, color);
};

/**
 * Sets the "u_color" uniform of the current program to color.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number> color 4 element array of 0-255 color
 *      components.
 */
var setUByteDrawColor = function(gl, color) {
  setFloatDrawColor(gl, ubyteColorToFloatColor(color));
};

/**
 * Draws a previously setup quad in the given color.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number>} color The color to draw with. A 4
 *        element array where each element is in the range 0 to
 *        1.
 */
var drawFloatColorQuad = function(gl, color) {
  var program = gl.getParameter(gl.CURRENT_PROGRAM);
  var colorLocation = gl.getUniformLocation(program, "u_color");
  gl.uniform4fv(colorLocation, color);
  gl.drawArrays(gl.TRIANGLES, 0, 6);
};


/**
 * Draws a previously setup quad in the given color.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number>} color The color to draw with. A 4
 *        element array where each element is in the range 0 to
 *        255.
 */
var drawUByteColorQuad = function(gl, color) {
  drawFloatColorQuad(gl, ubyteColorToFloatColor(color));
};

/**
 * Draws a previously setupUnitQuad.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 */
var drawUnitQuad = function(gl) {
  gl.drawArrays(gl.TRIANGLES, 0, 6);
};

/**
 * Clears then Draws a previously setupUnitQuad.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!Array.<number>} opt_color The color to fill clear with before
 *        drawing. A 4 element array where each element is in the range 0 to
 *        255. Default [255, 255, 255, 255]
 */
var clearAndDrawUnitQuad = function(gl, opt_color) {
  opt_color = opt_color || [255, 255, 255, 255];
  gl.clearColor(
      opt_color[0] / 255,
      opt_color[1] / 255,
      opt_color[2] / 255,
      opt_color[3] / 255);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  drawUnitQuad(gl);
};

/**
 * Draws a quad previously setup with setupIndexedQuad.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} gridRes Resolution of grid.
 */
var drawIndexedQuad = function(gl, gridRes) {
  gl.drawElements(gl.TRIANGLES, gridRes * gridRes * 6, gl.UNSIGNED_SHORT, 0);
};

/**
 * Draws a previously setupIndexedQuad
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number} gridRes Resolution of grid.
 * @param {!Array.<number>} opt_color The color to fill clear with before
 *        drawing. A 4 element array where each element is in the range 0 to
 *        255. Default [255, 255, 255, 255]
 */
var clearAndDrawIndexedQuad = function(gl, gridRes, opt_color) {
  opt_color = opt_color || [255, 255, 255, 255];
  gl.clearColor(
      opt_color[0] / 255,
      opt_color[1] / 255,
      opt_color[2] / 255,
      opt_color[3] / 255);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  drawIndexedQuad(gl, gridRes);
};

/**
 * Clips a range to min, max
 * (Eg. clipToRange(-5,7,0,20) would return {value:0,extent:2}
 * @param {number} value start of range
 * @param {number} extent extent of range
 * @param {number} min min.
 * @param {number} max max.
 * @return {!{value:number,extent:number} The clipped value.
 */
var clipToRange = function(value, extent, min, max) {
  if (value < min) {
    extent -= min - value;
    value = min;
  }
  var end = value + extent;
  if (end > max) {
    extent -= end - max;
  }
  if (extent < 0) {
    value = max;
    extent = 0;
  }
  return {value:value, extent: extent};
};

/**
 * Determines if the passed context is an instance of a WebGLRenderingContext
 * or later variant (like WebGL2RenderingContext)
 * @param {CanvasRenderingContext} ctx The context to check.
 */
var isWebGLContext = function(ctx) {
  if (ctx instanceof WebGLRenderingContext)
    return true;

  if ('WebGL2RenderingContext' in window && ctx instanceof WebGL2RenderingContext)
    return true;

  return false;
};

/**
 * Checks that a portion of a canvas or the currently attached framebuffer is 1 color.
 * @param {!WebGLRenderingContext|CanvasRenderingContext2D} gl The
 *         WebGLRenderingContext or 2D context to use.
 * @param {number} x left corner of region to check.
 * @param {number} y bottom corner of region to check in case of checking from
 *        a GL context or top corner in case of checking from a 2D context.
 * @param {number} width width of region to check.
 * @param {number} height width of region to check.
 * @param {!Array.<number>} color The color expected. A 4 element array where
 *        each element is in the range 0 to 255.
 * @param {number} opt_errorRange Optional. Acceptable error in
 *        color checking. 0 by default.
 * @param {!function()} sameFn Function to call if all pixels
 *        are the same as color.
 * @param {!function()} differentFn Function to call if a pixel
 *        is different than color
 * @param {!function()} logFn Function to call for logging.
 * @param {Uint8Array} opt_readBackBuf typically passed to reuse existing
 *        buffer while reading back pixels.
 */
var checkCanvasRectColor = function(gl, x, y, width, height, color, opt_errorRange, sameFn, differentFn, logFn, opt_readBackBuf) {
  if (isWebGLContext(gl) && !gl.getParameter(gl.FRAMEBUFFER_BINDING)) {
    // We're reading the backbuffer so clip.
    var xr = clipToRange(x, width, 0, gl.canvas.width);
    var yr = clipToRange(y, height, 0, gl.canvas.height);
    if (!xr.extent || !yr.extent) {
      logFn("checking rect: effective width or height is zero");
      sameFn();
      return;
    }
    x = xr.value;
    y = yr.value;
    width = xr.extent;
    height = yr.extent;
  }
  var errorRange = opt_errorRange || 0;
  if (!errorRange.length) {
    errorRange = [errorRange, errorRange, errorRange, errorRange]
  }
  var buf;
  if (isWebGLContext(gl)) {
    buf = opt_readBackBuf ? opt_readBackBuf : new Uint8Array(width * height * 4);
    gl.readPixels(x, y, width, height, gl.RGBA, gl.UNSIGNED_BYTE, buf);
  } else {
    buf = gl.getImageData(x, y, width, height).data;
  }
  for (var i = 0; i < width * height; ++i) {
    var offset = i * 4;
    for (var j = 0; j < color.length; ++j) {
      if (Math.abs(buf[offset + j] - color[j]) > errorRange[j]) {
        var was = buf[offset + 0].toString();
        for (j = 1; j < color.length; ++j) {
          was += "," + buf[offset + j];
        }
        differentFn('at (' + (x + (i % width)) + ', ' + (y + Math.floor(i / width)) +
                    ') expected: ' + color + ' was ' + was);
        return;
      }
    }
  }
  sameFn();
};

/**
 * Checks that a portion of a canvas or the currently attached framebuffer is 1 color.
 * @param {!WebGLRenderingContext|CanvasRenderingContext2D} gl The
 *         WebGLRenderingContext or 2D context to use.
 * @param {number} x left corner of region to check.
 * @param {number} y bottom corner of region to check in case of checking from
 *        a GL context or top corner in case of checking from a 2D context.
 * @param {number} width width of region to check.
 * @param {number} height width of region to check.
 * @param {!Array.<number>} color The color expected. A 4 element array where
 *        each element is in the range 0 to 255.
 * @param {string} opt_msg Message to associate with success. Eg
 *        ("should be red").
 * @param {number} opt_errorRange Optional. Acceptable error in
 *        color checking. 0 by default.
 */
var checkCanvasRect = function(gl, x, y, width, height, color, opt_msg, opt_errorRange) {
  checkCanvasRectColor(
      gl, x, y, width, height, color, opt_errorRange,
      function() {
        var msg = opt_msg;
        if (msg === undefined)
          msg = "should be " + color.toString();
        testPassed(msg);
      },
      testFailed,
      debug);
};

/**
 * Checks that an entire canvas or the currently attached framebuffer is 1 color.
 * @param {!WebGLRenderingContext|CanvasRenderingContext2D} gl The
 *         WebGLRenderingContext or 2D context to use.
 * @param {!Array.<number>} color The color expected. A 4 element array where
 *        each element is in the range 0 to 255.
 * @param {string} msg Message to associate with success. Eg ("should be red").
 * @param {number} errorRange Optional. Acceptable error in
 *        color checking. 0 by default.
 */
var checkCanvas = function(gl, color, msg, errorRange) {
  checkCanvasRect(gl, 0, 0, gl.canvas.width, gl.canvas.height, color, msg, errorRange);
};

/**
 * Checks a rectangular area both inside the area and outside
 * the area.
 * @param {!WebGLRenderingContext|CanvasRenderingContext2D} gl The
 *         WebGLRenderingContext or 2D context to use.
 * @param {number} x left corner of region to check.
 * @param {number} y bottom corner of region to check in case of checking from
 *        a GL context or top corner in case of checking from a 2D context.
 * @param {number} width width of region to check.
 * @param {number} height width of region to check.
 * @param {!Array.<number>} innerColor The color expected inside
 *     the area. A 4 element array where each element is in the
 *     range 0 to 255.
 * @param {!Array.<number>} outerColor The color expected
 *     outside. A 4 element array where each element is in the
 *     range 0 to 255.
 * @param {!number} opt_edgeSize: The number of pixels to skip
 *     around the edges of the area. Defaut 0.
 * @param {!{width:number, height:number}} opt_outerDimensions
 *     The outer dimensions. Default the size of gl.canvas.
 */
var checkAreaInAndOut = function(gl, x, y, width, height, innerColor, outerColor, opt_edgeSize, opt_outerDimensions) {
  var outerDimensions = opt_outerDimensions || { width: gl.canvas.width, height: gl.canvas.height };
  var edgeSize = opt_edgeSize || 0;
  checkCanvasRect(gl, x + edgeSize, y + edgeSize, width - edgeSize * 2, height - edgeSize * 2, innerColor);
  checkCanvasRect(gl, 0, 0, x - edgeSize, outerDimensions.height, outerColor);
  checkCanvasRect(gl, x + width + edgeSize, 0, outerDimensions.width - x - width - edgeSize, outerDimensions.height, outerColor);
  checkCanvasRect(gl, 0, 0, outerDimensions.width, y - edgeSize, outerColor);
  checkCanvasRect(gl, 0, y + height + edgeSize, outerDimensions.width, outerDimensions.height - y - height - edgeSize, outerColor);
};

/**
 * Loads a texture, calls callback when finished.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} url URL of image to load
 * @param {function(!Image): void} callback Function that gets called after
 *        image has loaded
 * @return {!WebGLTexture} The created texture.
 */
var loadTexture = function(gl, url, callback) {
    var texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    var image = new Image();
    image.onload = function() {
        gl.bindTexture(gl.TEXTURE_2D, texture);
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
        callback(image);
    };
    image.src = url;
    return texture;
};

/**
 * Checks whether the bound texture has expected dimensions. One corner pixel
 * of the texture will be changed as a side effect.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!WebGLTexture} texture The texture to check.
 * @param {number} width Expected width.
 * @param {number} height Expected height.
 * @param {GLenum} opt_format The texture's format. Defaults to RGBA.
 * @param {GLenum} opt_type The texture's type. Defaults to UNSIGNED_BYTE.
 */
var checkTextureSize = function(gl, width, height, opt_format, opt_type) {
  opt_format = opt_format || gl.RGBA;
  opt_type = opt_type || gl.UNSIGNED_BYTE;

  var numElements = getTypedArrayElementsPerPixel(gl, opt_format, opt_type);
  var buf = new (glTypeToTypedArrayType(gl, opt_type))(numElements);

  var errors = 0;
  gl.texSubImage2D(gl.TEXTURE_2D, 0, width - 1, height - 1, 1, 1, opt_format, opt_type, buf);
  if (gl.getError() != gl.NO_ERROR) {
    testFailed("Texture was smaller than the expected size " + width + "x" + height);
    ++errors;
  }
  gl.texSubImage2D(gl.TEXTURE_2D, 0, width - 1, height, 1, 1, opt_format, opt_type, buf);
  if (gl.getError() == gl.NO_ERROR) {
    testFailed("Texture was taller than " + height);
    ++errors;
  }
  gl.texSubImage2D(gl.TEXTURE_2D, 0, width, height - 1, 1, 1, opt_format, opt_type, buf);
  if (gl.getError() == gl.NO_ERROR) {
    testFailed("Texture was wider than " + width);
    ++errors;
  }
  if (errors == 0) {
    testPassed("Texture had the expected size " + width + "x" + height);
  }
};

/**
 * Makes a shallow copy of an object.
 * @param {!Object) src Object to copy
 * @return {!Object} The copy of src.
 */
var shallowCopyObject = function(src) {
  var dst = {};
  for (var attr in src) {
    if (src.hasOwnProperty(attr)) {
      dst[attr] = src[attr];
    }
  }
  return dst;
};

/**
 * Checks if an attribute exists on an object case insensitive.
 * @param {!Object) obj Object to check
 * @param {string} attr Name of attribute to look for.
 * @return {string?} The name of the attribute if it exists,
 *         undefined if not.
 */
var hasAttributeCaseInsensitive = function(obj, attr) {
  var lower = attr.toLowerCase();
  for (var key in obj) {
    if (obj.hasOwnProperty(key) && key.toLowerCase() == lower) {
      return key;
    }
  }
};

/**
 * Returns a map of URL querystring options
 * @return {Object?} Object containing all the values in the URL querystring
 */
var getUrlOptions = function() {
  var options = {};
  var s = window.location.href;
  var q = s.indexOf("?");
  var e = s.indexOf("#");
  if (e < 0) {
    e = s.length;
  }
  var query = s.substring(q + 1, e);
  var pairs = query.split("&");
  for (var ii = 0; ii < pairs.length; ++ii) {
    var keyValue = pairs[ii].split("=");
    var key = keyValue[0];
    var value = decodeURIComponent(keyValue[1]);
    options[key] = value;
  }

  return options;
};

/**
 * Creates a webgl context.
 * @param {!Canvas|string} opt_canvas The canvas tag to get
 *     context from. If one is not passed in one will be
 *     created. If it's a string it's assumed to be the id of a
 *     canvas.
 * @param {Object} opt_attributes Context attributes.
 * @param {!number} opt_version Version of WebGL context to create
 * @return {!WebGLRenderingContext} The created context.
 */
var create3DContext = function(opt_canvas, opt_attributes, opt_version) {
  if (window.initTestingHarness) {
    window.initTestingHarness();
  }
  var attributes = shallowCopyObject(opt_attributes || {});
  if (!hasAttributeCaseInsensitive(attributes, "antialias")) {
    attributes.antialias = false;
  }
  if (!opt_version) {
    opt_version = parseInt(getUrlOptions().webglVersion, 10) || 1;
  }
  opt_canvas = opt_canvas || document.createElement("canvas");
  if (typeof opt_canvas == 'string') {
    opt_canvas = document.getElementById(opt_canvas);
  }
  var context = null;

  var names;
  switch (opt_version) {
    case 2:
      names = ["webgl2", "experimental-webgl2"]; break;
    default:
      names = ["webgl", "experimental-webgl"]; break;
  }

  for (var i = 0; i < names.length; ++i) {
    try {
      context = opt_canvas.getContext(names[i], attributes);
    } catch (e) {
    }
    if (context) {
      break;
    }
  }
  if (!context) {
    testFailed("Unable to fetch WebGL rendering context for Canvas");
  }
  return context;
}

/**
 * Wraps a WebGL function with a function that throws an exception if there is
 * an error.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} fname Name of function to wrap.
 * @return {function} The wrapped function.
 */
var createGLErrorWrapper = function(context, fname) {
  return function() {
    var rv = context[fname].apply(context, arguments);
    var err = context.getError();
    if (err != context.NO_ERROR)
      throw "GL error " + glEnumToString(context, err) + " in " + fname;
    return rv;
  };
};

/**
 * Creates a WebGL context where all functions are wrapped to throw an exception
 * if there is an error.
 * @param {!Canvas} canvas The HTML canvas to get a context from.
 * @return {!Object} The wrapped context.
 */
function create3DContextWithWrapperThatThrowsOnGLError(canvas) {
  var context = create3DContext(canvas);
  var wrap = {};
  for (var i in context) {
    try {
      if (typeof context[i] == 'function') {
        wrap[i] = createGLErrorWrapper(context, i);
      } else {
        wrap[i] = context[i];
      }
    } catch (e) {
      error("createContextWrapperThatThrowsOnGLError: Error accessing " + i);
    }
  }
  wrap.getError = function() {
      return context.getError();
  };
  return wrap;
};

/**
 * Tests that an evaluated expression generates a specific GL error.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number|Array.<number>} glErrors The expected gl error or an array of expected errors.
 * @param {string} evalStr The string to evaluate.
 */
var shouldGenerateGLError = function(gl, glErrors, evalStr) {
  var exception;
  try {
    eval(evalStr);
  } catch (e) {
    exception = e;
  }
  if (exception) {
    testFailed(evalStr + " threw exception " + exception);
  } else {
    glErrorShouldBe(gl, glErrors, "after evaluating: " + evalStr);
  }
};

/**
 * Tests that the first error GL returns is the specified error.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {number|Array.<number>} glErrors The expected gl error or an array of expected errors.
 * @param {string} opt_msg Optional additional message.
 */
var glErrorShouldBe = function(gl, glErrors, opt_msg) {
  if (!glErrors.length) {
    glErrors = [glErrors];
  }
  opt_msg = opt_msg || "";
  var err = gl.getError();
  var ndx = glErrors.indexOf(err);
  var errStrs = [];
  for (var ii = 0; ii < glErrors.length; ++ii) {
    errStrs.push(glEnumToString(gl, glErrors[ii]));
  }
  var expected = errStrs.join(" or ");
  if (ndx < 0) {
    var msg = "getError expected" + ((glErrors.length > 1) ? " one of: " : ": ");
    testFailed(msg + expected +  ". Was " + glEnumToString(gl, err) + " : " + opt_msg);
  } else {
    var msg = "getError was " + ((glErrors.length > 1) ? "one of: " : "expected value: ");
    testPassed(msg + expected + " : " + opt_msg);
  }
};

/**
 * Links a WebGL program, throws if there are errors.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!WebGLProgram} program The WebGLProgram to link.
 * @param {function(string): void) opt_errorCallback callback for errors. 
 */
var linkProgram = function(gl, program, opt_errorCallback) {
  var errFn = opt_errorCallback || testFailed;
  // Link the program
  gl.linkProgram(program);

  // Check the link status
  var linked = gl.getProgramParameter(program, gl.LINK_STATUS);
  if (!linked) {
    // something went wrong with the link
    var error = gl.getProgramInfoLog (program);

    errFn("Error in program linking:" + error);

    gl.deleteProgram(program);
  }
};

/**
 * Loads text from an external file. This function is synchronous.
 * @param {string} url The url of the external file.
 * @param {!function(bool, string): void} callback that is sent a bool for
 *     success and the string.
 */
var loadTextFileAsync = function(url, callback) {
  log ("loading: " + url);
  var error = 'loadTextFileSynchronous failed to load url "' + url + '"';
  var request;
  if (window.XMLHttpRequest) {
    request = new XMLHttpRequest();
    if (request.overrideMimeType) {
      request.overrideMimeType('text/plain');
    }
  } else {
    throw 'XMLHttpRequest is disabled';
  }
  try {
    request.open('GET', url, true);
    request.onreadystatechange = function() {
      if (request.readyState == 4) {
        var text = '';
        // HTTP reports success with a 200 status. The file protocol reports
        // success with zero. HTTP does not use zero as a status code (they
        // start at 100).
        // https://developer.mozilla.org/En/Using_XMLHttpRequest
        var success = request.status == 200 || request.status == 0;
        if (success) {
          text = request.responseText;
        }
        log("loaded: " + url);
        callback(success, text);
      }
    };
    request.send(null);
  } catch (e) {
    log("failed to load: " + url);
    callback(false, '');
  }
};

/**
 * Recursively loads a file as a list. Each line is parsed for a relative
 * path. If the file ends in .txt the contents of that file is inserted in
 * the list.
 *
 * @param {string} url The url of the external file.
 * @param {!function(bool, Array<string>): void} callback that is sent a bool
 *     for success and the array of strings.
 */
var getFileListAsync = function(url, callback) {
  var files = [];

  var getFileListImpl = function(url, callback) {
    var files = [];
    if (url.substr(url.length - 4) == '.txt') {
      loadTextFileAsync(url, function() {
        return function(success, text) {
          if (!success) {
            callback(false, '');
            return;
          }
          var lines = text.split('\n');
          var prefix = '';
          var lastSlash = url.lastIndexOf('/');
          if (lastSlash >= 0) {
            prefix = url.substr(0, lastSlash + 1);
          }
          var fail = false;
          var count = 1;
          var index = 0;
          for (var ii = 0; ii < lines.length; ++ii) {
            var str = lines[ii].replace(/^\s\s*/, '').replace(/\s\s*$/, '');
            if (str.length > 4 &&
                str[0] != '#' &&
                str[0] != ";" &&
                str.substr(0, 2) != "//") {
              var names = str.split(/ +/);
              new_url = prefix + str;
              if (names.length == 1) {
                new_url = prefix + str;
                ++count;
                getFileListImpl(new_url, function(index) {
                  return function(success, new_files) {
                    log("got files: " + new_files.length);
                    if (success) {
                      files[index] = new_files;
                    }
                    finish(success);
                  };
                }(index++));
              } else {
                var s = "";
                var p = "";
                for (var jj = 0; jj < names.length; ++jj) {
                  s += p + prefix + names[jj];
                  p = " ";
                }
                files[index++] = s;
              }
            }
          }
          finish(true);

          function finish(success) {
            if (!success) {
              fail = true;
            }
            --count;
            log("count: " + count);
            if (!count) {
              callback(!fail, files);
            }
          }
        }
      }());

    } else {
      files.push(url);
      callback(true, files);
    }
  };

  getFileListImpl(url, function(success, files) {
    // flatten
    var flat = [];
    flatten(files);
    function flatten(files) {
      for (var ii = 0; ii < files.length; ++ii) {
        var value = files[ii];
        if (typeof(value) == "string") {
          flat.push(value);
        } else {
          flatten(value);
        }
      }
    }
    callback(success, flat);
  });
};

/**
 * Gets a file from a file/URL.
 * @param {string} file the URL of the file to get.
 * @return {string} The contents of the file.
 */
var readFile = function(file) {
  var xhr = new XMLHttpRequest();
  xhr.open("GET", file, false);
  xhr.send();
  return xhr.responseText.replace(/\r/g, "");
};

var readFileList = function(url) {
  var files = [];
  if (url.substr(url.length - 4) == '.txt') {
    var lines = readFile(url).split('\n');
    var prefix = '';
    var lastSlash = url.lastIndexOf('/');
    if (lastSlash >= 0) {
      prefix = url.substr(0, lastSlash + 1);
    }
    for (var ii = 0; ii < lines.length; ++ii) {
      var str = lines[ii].replace(/^\s\s*/, '').replace(/\s\s*$/, '');
      if (str.length > 4 &&
          str[0] != '#' &&
          str[0] != ";" &&
          str.substr(0, 2) != "//") {
        var names = str.split(/ +/);
        if (names.length == 1) {
          new_url = prefix + str;
          files = files.concat(readFileList(new_url));
        } else {
          var s = "";
          var p = "";
          for (var jj = 0; jj < names.length; ++jj) {
            s += p + prefix + names[jj];
            p = " ";
          }
          files.push(s);
        }
      }
    }
  } else {
    files.push(url);
  }
  return files;
};

/**
 * Loads a shader.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} shaderSource The shader source.
 * @param {number} shaderType The type of shader. 
 * @param {function(string): void) opt_errorCallback callback for errors. 
 * @param {boolean} opt_logShaders Whether to log shader source.
 * @param {string} opt_shaderLabel Label that identifies the shader source in
 *     the log.
 * @param {string} opt_url URL from where the shader source was loaded from.
 *     If opt_logShaders is set, then a link to the source file will also be
 *     added.
 * @return {!WebGLShader} The created shader.
 */
var loadShader = function(
    gl, shaderSource, shaderType, opt_errorCallback, opt_logShaders,
    opt_shaderLabel, opt_url) {
  var errFn = opt_errorCallback || error;
  // Create the shader object
  var shader = gl.createShader(shaderType);
  if (shader == null) {
    errFn("*** Error: unable to create shader '"+shaderSource+"'");
    return null;
  }

  // Load the shader source
  gl.shaderSource(shader, shaderSource);
  var err = gl.getError();
  if (err != gl.NO_ERROR) {
    errFn("*** Error loading shader '" + shader + "':" + glEnumToString(gl, err));
    return null;
  }

  // Compile the shader
  gl.compileShader(shader);

  if (opt_logShaders) {
    var label = shaderType == gl.VERTEX_SHADER ? 'vertex shader' : 'fragment_shader';
    if (opt_shaderLabel) {
      label = opt_shaderLabel + ' ' + label;
    }
    addShaderSources(
        gl, document.getElementById('console'), label, shader, shaderSource, opt_url);
  }

  // Check the compile status
  var compiled = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
  if (!compiled) {
    // Something went wrong during compilation; get the error
    lastError = gl.getShaderInfoLog(shader);
    errFn("*** Error compiling " + glEnumToString(gl, shaderType) + " '" + shader + "':" + lastError);
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}

/**
 * Loads a shader from a URL.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {file} file The URL of the shader source.
 * @param {number} type The type of shader.
 * @param {function(string): void) opt_errorCallback callback for errors. 
 * @param {boolean} opt_logShaders Whether to log shader source.
 * @return {!WebGLShader} The created shader.
 */
var loadShaderFromFile = function(
    gl, file, type, opt_errorCallback, opt_logShaders) {
  var shaderSource = readFile(file);
  return loadShader(gl, shaderSource, type, opt_errorCallback,
      opt_logShaders, undefined, file);
};

/**
 * Gets the content of script.
 * @param {string} scriptId The id of the script tag.
 * @return {string} The content of the script.
 */
var getScript = function(scriptId) {
  var shaderScript = document.getElementById(scriptId);
  if (!shaderScript) {
    throw("*** Error: unknown script element " + scriptId);
  }
  return shaderScript.text;
};

/**
 * Loads a shader from a script tag.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} scriptId The id of the script tag.
 * @param {number} opt_shaderType The type of shader. If not passed in it will
 *     be derived from the type of the script tag.
 * @param {function(string): void) opt_errorCallback callback for errors. 
 * @param {boolean} opt_logShaders Whether to log shader source.
 * @return {!WebGLShader} The created shader.
 */
var loadShaderFromScript = function(
    gl, scriptId, opt_shaderType, opt_errorCallback, opt_logShaders) {
  var shaderSource = "";
  var shaderScript = document.getElementById(scriptId);
  if (!shaderScript) {
    throw("*** Error: unknown script element " + scriptId);
  }
  shaderSource = shaderScript.text;

  if (!opt_shaderType) {
    if (shaderScript.type == "x-shader/x-vertex") {
      opt_shaderType = gl.VERTEX_SHADER;
    } else if (shaderScript.type == "x-shader/x-fragment") {
      opt_shaderType = gl.FRAGMENT_SHADER;
    } else {
      throw("*** Error: unknown shader type");
      return null;
    }
  }

  return loadShader(gl, shaderSource, opt_shaderType, opt_errorCallback,
      opt_logShaders);
};

var loadStandardProgram = function(gl) {
  var program = gl.createProgram();
  gl.attachShader(program, loadStandardVertexShader(gl));
  gl.attachShader(program, loadStandardFragmentShader(gl));
  gl.bindAttribLocation(program, 0, "a_vertex");
  gl.bindAttribLocation(program, 1, "a_normal");
  linkProgram(gl, program);
  return program;
};

/**
 * Loads shaders from files, creates a program, attaches the shaders and links.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} vertexShaderPath The URL of the vertex shader.
 * @param {string} fragmentShaderPath The URL of the fragment shader.
 * @param {function(string): void) opt_errorCallback callback for errors. 
 * @return {!WebGLProgram} The created program.
 */
var loadProgramFromFile = function(
    gl, vertexShaderPath, fragmentShaderPath, opt_errorCallback) {
  var program = gl.createProgram();
  var vs = loadShaderFromFile(
      gl, vertexShaderPath, gl.VERTEX_SHADER, opt_errorCallback);
  var fs = loadShaderFromFile(
      gl, fragmentShaderPath, gl.FRAGMENT_SHADER, opt_errorCallback);
  if (vs && fs) {
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    linkProgram(gl, program, opt_errorCallback);
  }
  if (vs) {
    gl.deleteShader(vs);
  }
  if (fs) {
    gl.deleteShader(fs);
  }
  return program;
};

/**
 * Loads shaders from script tags, creates a program, attaches the shaders and
 * links.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} vertexScriptId The id of the script tag that contains the
 *        vertex shader.
 * @param {string} fragmentScriptId The id of the script tag that contains the
 *        fragment shader.
 * @param {function(string): void) opt_errorCallback callback for errors. 
 * @return {!WebGLProgram} The created program.
 */
var loadProgramFromScript = function loadProgramFromScript(
    gl, vertexScriptId, fragmentScriptId, opt_errorCallback) {
  var program = gl.createProgram();
  gl.attachShader(
      program,
      loadShaderFromScript(
          gl, vertexScriptId, gl.VERTEX_SHADER, opt_errorCallback));
  gl.attachShader(
      program,
      loadShaderFromScript(
          gl, fragmentScriptId,  gl.FRAGMENT_SHADER, opt_errorCallback));
  linkProgram(gl, program, opt_errorCallback);
  return program;
};

/**
 * Loads shaders from source, creates a program, attaches the shaders and
 * links.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!WebGLShader} vertexShader The vertex shader.
 * @param {!WebGLShader} fragmentShader The fragment shader.
 * @param {function(string): void) opt_errorCallback callback for errors.
 * @return {!WebGLProgram} The created program.
 */
var createProgram = function(gl, vertexShader, fragmentShader, opt_errorCallback) {
  var program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  linkProgram(gl, program, opt_errorCallback);
  return program;
};

/**
 * Loads shaders from source, creates a program, attaches the shaders and
 * links.
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} vertexShader The vertex shader source.
 * @param {string} fragmentShader The fragment shader source.
 * @param {function(string): void) opt_errorCallback callback for errors. 
 * @param {boolean} opt_logShaders Whether to log shader source.
 * @return {!WebGLProgram} The created program.
 */
var loadProgram = function(
    gl, vertexShader, fragmentShader, opt_errorCallback, opt_logShaders) {
  var program;
  var vs = loadShader(
      gl, vertexShader, gl.VERTEX_SHADER, opt_errorCallback, opt_logShaders);
  var fs = loadShader(
      gl, fragmentShader, gl.FRAGMENT_SHADER, opt_errorCallback, opt_logShaders);
  if (vs && fs) {
    program = createProgram(gl, vs, fs, opt_errorCallback)
  }
  if (vs) {
    gl.deleteShader(vs);
  }
  if (fs) {
    gl.deleteShader(fs);
  }
  return program;
};

/**
 * Loads shaders from source, creates a program, attaches the shaders and
 * links but expects error.
 *
 * GLSL 1.0.17 10.27 effectively says that compileShader can
 * always succeed as long as linkProgram fails so we can't
 * rely on compileShader failing. This function expects
 * one of the shader to fail OR linking to fail.
 *
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} vertexShaderScriptId The vertex shader.
 * @param {string} fragmentShaderScriptId The fragment shader.
 * @return {WebGLProgram} The created program.
 */
var loadProgramFromScriptExpectError = function(
    gl, vertexShaderScriptId, fragmentShaderScriptId) {
  var vertexShader = loadShaderFromScript(gl, vertexShaderScriptId);
  if (!vertexShader) {
    return null;
  }
  var fragmentShader = loadShaderFromScript(gl, fragmentShaderScriptId);
  if (!fragmentShader) {
    return null;
  }
  var linkSuccess = true;
  var program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  linkSuccess = true;
  linkProgram(gl, program, function() {
      linkSuccess = false;
    });
  return linkSuccess ? program : null;
};


var getActiveMap = function(gl, program, typeInfo) {
  var numVariables = gl.getProgramParameter(program, gl[typeInfo.param]);
  var variables = {};
  for (var ii = 0; ii < numVariables; ++ii) {
    var info = gl[typeInfo.activeFn](program, ii);
    variables[info.name] = {
      name: info.name,
      size: info.size,
      type: info.type,
      location: gl[typeInfo.locFn](program, info.name)
    };
  }
  return variables;
};

/**
 * Returns a map of attrib names to info about those
 * attribs.
 *
 * eg:
 *    { "attrib1Name":
 *      {
 *        name: "attrib1Name",
 *        size: 1,
 *        type: gl.FLOAT_MAT2,
 *        location: 0
 *      },
 *      "attrib2Name[0]":
 *      {
 *         name: "attrib2Name[0]",
 *         size: 4,
 *         type: gl.FLOAT,
 *         location: 1
 *      },
 *    }
 *
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {WebGLProgram} The program to query for attribs.
 * @return the map.
 */
var getAttribMap = function(gl, program) {
  return getActiveMap(gl, program, {
      param: "ACTIVE_ATTRIBUTES",
      activeFn: "getActiveAttrib",
      locFn: "getAttribLocation"
  });
};

/**
 * Returns a map of uniform names to info about those uniforms.
 *
 * eg:
 *    { "uniform1Name":
 *      {
 *        name: "uniform1Name",
 *        size: 1,
 *        type: gl.FLOAT_MAT2,
 *        location: WebGLUniformLocation
 *      },
 *      "uniform2Name[0]":
 *      {
 *         name: "uniform2Name[0]",
 *         size: 4,
 *         type: gl.FLOAT,
 *         location: WebGLUniformLocation
 *      },
 *    }
 *
 * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {WebGLProgram} The program to query for uniforms.
 * @return the map.
 */
var getUniformMap = function(gl, program) {
  return getActiveMap(gl, program, {
      param: "ACTIVE_UNIFORMS",
      activeFn: "getActiveUniform",
      locFn: "getUniformLocation"
  });
};

var basePath;
var getBasePath = function() {
  if (!basePath) {
    var expectedBase = "webgl-test-utils.js";
    var scripts = document.getElementsByTagName('script');
    for (var script, i = 0; script = scripts[i]; i++) {
      var src = script.src;
      var l = src.length;
      if (src.substr(l - expectedBase.length) == expectedBase) {
        basePath = src.substr(0, l - expectedBase.length);
      }
    }
  }
  return basePath;
};

var loadStandardVertexShader = function(gl) {
  return loadShaderFromFile(
      gl, getBasePath() + "vertexShader.vert", gl.VERTEX_SHADER);
};

var loadStandardFragmentShader = function(gl) {
  return loadShaderFromFile(
      gl, getBasePath() + "fragmentShader.frag", gl.FRAGMENT_SHADER);
};

/**
 * Loads an image asynchronously.
 * @param {string} url URL of image to load.
 * @param {!function(!Element): void} callback Function to call
 *     with loaded image.
 */
var loadImageAsync = function(url, callback) {
  var img = document.createElement('img');
  img.onload = function() {
    callback(img);
  };
  img.src = url;
};

/**
 * Loads an array of images.
 * @param {!Array.<string>} urls URLs of images to load.
 * @param {!function(!{string, img}): void} callback. Callback
 *     that gets passed map of urls to img tags.
 */
var loadImagesAsync = function(urls, callback) {
  var count = 1;
  var images = { };
  function countDown() {
    --count;
    if (count == 0) {
      log("loadImagesAsync: all images loaded");
      callback(images);
    }
  }
  function imageLoaded(url) {
    return function(img) {
      images[url] = img;
      log("loadImagesAsync: loaded " + url);
      countDown();
    }
  }
  for (var ii = 0; ii < urls.length; ++ii) {
    ++count;
    loadImageAsync(urls[ii], imageLoaded(urls[ii]));
  }
  countDown();
};

/**
 * Returns a map of key=value values from url.
 * @return {!Object.<string, number>} map of keys to values.
 */
var getUrlArguments = function() {
  var args = {};
  try {
    var s = window.location.href;
    var q = s.indexOf("?");
    var e = s.indexOf("#");
    if (e < 0) {
      e = s.length;
    }
    var query = s.substring(q + 1, e);
    var pairs = query.split("&");
    for (var ii = 0; ii < pairs.length; ++ii) {
      var keyValue = pairs[ii].split("=");
      var key = keyValue[0];
      var value = decodeURIComponent(keyValue[1]);
      args[key] = value;
    }
  } catch (e) {
    throw "could not parse url";
  }
  return args;
};

/**
 * Makes an image from a src.
 * @param {string} src Image source URL.
 * @param {function} onload Callback to call when the image has finised loading.
 * @param {function} onerror Callback to call when an error occurs.
 * @return {!Image} The created image.
 */
var makeImage = function(src, onload, onerror) {
  var img = document.createElement('img');
  if (onload) {
    img.onload = onload;
  }
  if (onerror) {
    img.onerror = onerror;
  } else {
    img.onerror = function() {
      log("WARNING: creating image failed; src: " + this.src);
    };
  }
  if (src) {
    img.src = src;
  }
  return img;
}

/**
 * Makes an image element from a canvas.
 * @param {!HTMLCanvas} canvas Canvas to make image from.
 * @param {function} onload Callback to call when the image has finised loading.
 * @param {string} imageFormat Image format to be passed to toDataUrl().
 * @return {!Image} The created image.
 */
var makeImageFromCanvas = function(canvas, onload, imageFormat) {
  return makeImage(canvas.toDataURL(imageFormat), onload);
};

/**
 * Makes a video element from a src.
 * @param {string} src Video source URL.
 * @param {function} onerror Callback to call when an error occurs.
 * @return {!Video} The created video.
 */
var makeVideo = function(src, onerror) {
  var vid = document.createElement('video');
  if (onerror) {
    vid.onerror = onerror;
  } else {
    vid.onerror = function() {
      log("WARNING: creating video failed; src: " + this.src);
    };
  }
  if (src) {
    vid.src = src;
  }
  return vid;
}

/**
 * Inserts an image with a caption into 'element'.
 * @param {!HTMLElement} element Element to append image to.
 * @param {string} caption caption to associate with image.
 * @param {!Image) img image to insert.
 */
var insertImage = function(element, caption, img) {
  var div = document.createElement("div");
  div.appendChild(img);
  var label = document.createElement("div");
  label.appendChild(document.createTextNode(caption));
  div.appendChild(label);
   element.appendChild(div);
};

/**
 * Inserts a 'label' that when clicked expands to the pre formatted text
 * supplied by 'source'.
 * @param {!HTMLElement} element element to append label to.
 * @param {string} label label for anchor.
 * @param {string} source preformatted text to expand to.
 * @param {string} opt_url URL of source. If provided a link to the source file
 *     will also be added.
 */
var addShaderSource = function(element, label, source, opt_url) {
  var div = document.createElement("div");
  var s = document.createElement("pre");
  s.className = "shader-source";
  s.style.display = "none";
  var ol = document.createElement("ol");
  //s.appendChild(document.createTextNode(source));
  var lines = source.split("\n");
  for (var ii = 0; ii < lines.length; ++ii) {
    var line = lines[ii];
    var li = document.createElement("li");
    li.appendChild(document.createTextNode(line));
    ol.appendChild(li);
  }
  s.appendChild(ol);
  var l = document.createElement("a");
  l.href = "show-shader-source";
  l.appendChild(document.createTextNode(label));
  l.addEventListener('click', function(event) {
      if (event.preventDefault) {
        event.preventDefault();
      }
      s.style.display = (s.style.display == 'none') ? 'block' : 'none';
      return false;
    }, false);
  div.appendChild(l);
  if (opt_url) {
    var u = document.createElement("a");
    u.href = opt_url;
    div.appendChild(document.createTextNode(" "));
    u.appendChild(document.createTextNode("(" + opt_url + ")"));
    div.appendChild(u);
  }
  div.appendChild(s);
  element.appendChild(div);
};

/**
 * Inserts labels that when clicked expand to show the original source of the
 * shader and also translated source of the shader, if that is available.
 * @param {WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {!HTMLElement} element element to append label to.
 * @param {string} label label for anchor.
 * @param {WebGLShader} shader Shader to show the sources for.
 * @param {string} shaderSource Original shader source.
 * @param {string} opt_url URL of source. If provided a link to the source file
 *     will also be added.
 */
var addShaderSources = function(
    gl, element, label, shader, shaderSource, opt_url) {
  addShaderSource(element, label, shaderSource, opt_url);

  var debugShaders = gl.getExtension('WEBGL_debug_shaders');
  if (debugShaders && shader) {
    var translatedSource = debugShaders.getTranslatedShaderSource(shader);
    if (translatedSource != '') {
      addShaderSource(element, label + ' translated for driver', translatedSource);
    }
  }
};

/**
 * Sends shader information to the server to be dumped into text files
 * when tests are run from within the test-runner harness.
 * @param {WebGLRenderingContext} gl The WebGLRenderingContext to use.
 * @param {string} url URL of current.
 * @param {string} passMsg Test description.
 * @param {object} vInfo Object containing vertex shader information.
 * @param {object} fInfo Object containing fragment shader information.
 */
var dumpShadersInfo = function(gl, url, passMsg, vInfo, fInfo) {
  var shaderInfo = {};
  shaderInfo.url = url;
  shaderInfo.testDescription = passMsg;
  shaderInfo.vLabel = vInfo.label;
  shaderInfo.vShouldCompile = vInfo.shaderSuccess;
  shaderInfo.vSource = vInfo.source;
  shaderInfo.fLabel = fInfo.label;
  shaderInfo.fShouldCompile = fInfo.shaderSuccess;
  shaderInfo.fSource = fInfo.source;
  shaderInfo.vTranslatedSource = null;
  shaderInfo.fTranslatedSource = null;
  var debugShaders = gl.getExtension('WEBGL_debug_shaders');
  if (debugShaders) {
    if (vInfo.shader)
      shaderInfo.vTranslatedSource = debugShaders.getTranslatedShaderSource(vInfo.shader);
    if (fInfo.shader)
      shaderInfo.fTranslatedSource = debugShaders.getTranslatedShaderSource(fInfo.shader);
  }

  var dumpShaderInfoRequest = new XMLHttpRequest();
  dumpShaderInfoRequest.open('POST', "/dumpShaderInfo", true);
  dumpShaderInfoRequest.setRequestHeader("Content-Type", "text/plain");
  dumpShaderInfoRequest.send(JSON.stringify(shaderInfo));
};

// Add your prefix here.
var browserPrefixes = [
  "",
  "MOZ_",
  "OP_",
  "WEBKIT_"
];

/**
 * Given an extension name like WEBGL_compressed_texture_s3tc
 * returns the name of the supported version extension, like
 * WEBKIT_WEBGL_compressed_teture_s3tc
 * @param {string} name Name of extension to look for.
 * @return {string} name of extension found or undefined if not
 *     found.
 */
var getSupportedExtensionWithKnownPrefixes = function(gl, name) {
  var supported = gl.getSupportedExtensions();
  for (var ii = 0; ii < browserPrefixes.length; ++ii) {
    var prefixedName = browserPrefixes[ii] + name;
    if (supported.indexOf(prefixedName) >= 0) {
      return prefixedName;
    }
  }
};

/**
 * Given an extension name like WEBGL_compressed_texture_s3tc
 * returns the supported version extension, like
 * WEBKIT_WEBGL_compressed_teture_s3tc
 * @param {string} name Name of extension to look for.
 * @return {WebGLExtension} The extension or undefined if not
 *     found.
 */
var getExtensionWithKnownPrefixes = function(gl, name) {
  for (var ii = 0; ii < browserPrefixes.length; ++ii) {
    var prefixedName = browserPrefixes[ii] + name;
    var ext = gl.getExtension(prefixedName);
    if (ext) {
      return ext;
    }
  }
};

/**
 * Returns possible prefixed versions of an extension's name.
 * @param {string} name Name of extension. May already include a prefix.
 * @return {Array.<string>} Variations of the extension name with known
 *     browser prefixes.
 */
var getExtensionPrefixedNames = function(name) {
  var unprefix = function(name) {
    for (var ii = 0; ii < browserPrefixes.length; ++ii) {
      if (browserPrefixes[ii].length > 0 &&
          name.substring(0, browserPrefixes[ii].length).toLowerCase() ===
          browserPrefixes[ii].toLowerCase()) {
        return name.substring(browserPrefixes[ii].length);
      }
    }
    return name;
  }

  var unprefixed = unprefix(name);

  var variations = [];
  for (var ii = 0; ii < browserPrefixes.length; ++ii) {
    variations.push(browserPrefixes[ii] + unprefixed);
  }

  return variations;
};

var replaceRE = /\$\((\w+)\)/g;

/**
 * Replaces strings with property values.
 * Given a string like "hello $(first) $(last)" and an object
 * like {first:"John", last:"Smith"} will return
 * "hello John Smith".
 * @param {string} str String to do replacements in.
 * @param {...} 1 or more objects containing properties.
 */
var replaceParams = function(str) {
  var args = arguments;
  return str.replace(replaceRE, function(str, p1, offset, s) {
    for (var ii = 1; ii < args.length; ++ii) {
      if (args[ii][p1] !== undefined) {
        return args[ii][p1];
      }
    }
    throw "unknown string param '" + p1 + "'";
  });
};

var upperCaseFirstLetter = function(str) {
  return str.substring(0, 1).toUpperCase() + str.substring(1);
};

/**
 * Gets a prefixed property. For example,
 *
 *     var fn = getPrefixedProperty(
 *        window,
 *        "requestAnimationFrame");
 *
 * Will return either:
 *    "window.requestAnimationFrame",
 *    "window.oRequestAnimationFrame",
 *    "window.msRequestAnimationFrame",
 *    "window.mozRequestAnimationFrame",
 *    "window.webKitRequestAnimationFrame",
 *    undefined
 *
 * the non-prefixed function is tried first.
 */
var propertyPrefixes = ["", "moz", "ms", "o", "webkit"];
var getPrefixedProperty = function(obj, propertyName) {
  for (var ii = 0; ii < propertyPrefixes.length; ++ii) {
    var prefix = propertyPrefixes[ii];
    var name = prefix + propertyName;
    log(name);
    var property = obj[name];
    if (property) {
      return property;
    }
    if (ii == 0) {
      propertyName = upperCaseFirstLetter(propertyName);
    }
  }
  return undefined;
};

var _requestAnimFrame;

/**
 * Provides requestAnimationFrame in a cross browser way.
 */
var requestAnimFrame = function(callback) {
  if (!_requestAnimFrame) {
    _requestAnimFrame = getPrefixedProperty(window, "requestAnimationFrame") || 
      function(callback, element) {
        return window.setTimeout(callback, 1000 / 70);
      };
  }
  log("requestAnimFrame: document.hidden = " + document.hidden);
  _requestAnimFrame.call(window, callback);
};

var _cancelAnimFrame;

/**
 * Provides cancelAnimationFrame in a cross browser way.
 */
var cancelAnimFrame = function(request) {
  if (!_cancelAnimFrame) {
    _cancelAnimFrame = getPrefixedProperty(window, "cancelAnimationFrame") ||
      window.clearTimeout;
  }
  _cancelAnimFrame.call(window, request);
};

/**
 * Provides requestFullScreen in a cross browser way.
 */
var requestFullScreen = function(element) {
  var fn = getPrefixedProperty(element, "requestFullScreen");
  if (fn) {
    fn.call(element);
  }
};

/**
 * Provides cancelFullScreen in a cross browser way.
 */
var cancelFullScreen = function() {
  var fn = getPrefixedProperty(document, "cancelFullScreen");
  if (fn) {
    fn.call(document);
  }
};

var fullScreenStateName;
(function() {
  var fullScreenStateNames = [
    "isFullScreen",
    "fullScreen",
  ];
  for (var ii = 0; ii < fullScreenStateNames.length; ++ii) {
    var propertyName = fullScreenStateNames[ii];
    for (var jj = 0; jj < propertyPrefixes.length; ++jj) {
      var prefix = propertyPrefixes[jj];
      if (prefix.length) {
        propertyName = upperCaseFirstLetter(propertyName);
        fullScreenStateName = prefix + propertyName;
        if (document[fullScreenStateName] !== undefined) {
          return;
        }
      }
    }
    fullScreenStateName = undefined;
  }
}());

/**
 * @return {boolean} True if fullscreen mode is active.
 */
var getFullScreenState = function() {
  log("fullscreenstatename:" + fullScreenStateName);
  log(document[fullScreenStateName]);
  return document[fullScreenStateName];
};

/**
 * @param {!HTMLElement} element The element to go fullscreen.
 * @param {!function(boolean)} callback A function that will be called
 *        when entering/exiting fullscreen. It is passed true if
 *        entering fullscreen, false if exiting.
 */
var onFullScreenChange = function(element, callback) {
  propertyPrefixes.forEach(function(prefix) {
    var eventName = prefix + "fullscreenchange";
    log("addevent: " + eventName);
    document.addEventListener(eventName, function(event) {
      log("event: " + eventName);
      callback(getFullScreenState());
    });
  });
};

/**
 * @param {!string} buttonId The id of the button that will toggle fullscreen
 *        mode.
 * @param {!string} fullscreenId The id of the element to go fullscreen.
 * @param {!function(boolean)} callback A function that will be called
 *        when entering/exiting fullscreen. It is passed true if
 *        entering fullscreen, false if exiting.
 * @return {boolean} True if fullscreen mode is supported.
 */
var setupFullscreen = function(buttonId, fullscreenId, callback) {
  if (!fullScreenStateName) {
    return false;
  }

  var fullscreenElement = document.getElementById(fullscreenId);
  onFullScreenChange(fullscreenElement, callback);

  var toggleFullScreen = function(event) {
    if (getFullScreenState()) {
      cancelFullScreen(fullscreenElement);
    } else {
      requestFullScreen(fullscreenElement);
    }
    event.preventDefault();
    return false;
  };

  var buttonElement = document.getElementById(buttonId);
  buttonElement.addEventListener('click', toggleFullScreen);

  return true;
};

/**
 * Waits for the browser to composite the web page.
 * @param {function()} callback A function to call after compositing has taken
 *        place.
 */
var waitForComposite = function(callback) {
  var frames = 5;
  var countDown = function() {
    if (frames == 0) {
      log("waitForComposite: callback");
      callback();
    } else {
      log("waitForComposite: countdown(" + frames + ")");
      --frames;
      requestAnimFrame.call(window, countDown);
    }
  };
  countDown();
};

/**
 * Runs an array of functions, yielding to the browser between each step.
 * If you want to know when all the steps are finished add a last step.
 * @param {!Array.<function(): void>} steps. Array of functions.
 */
var runSteps = function(steps) {
  if (!steps.length) {
    return;
  }

  // copy steps so they can't be modifed.
  var stepsToRun = steps.slice();
  var currentStep = 0;
  var runNextStep = function() {
    stepsToRun[currentStep++]();
    if (currentStep < stepsToRun.length) {
      setTimeout(runNextStep, 1);
    }
  };
  runNextStep();
};

/**
 * Starts playing a video and waits for it to be consumable.
 * @param {!HTMLVideoElement} video An HTML5 Video element.
 * @param {!function(!HTMLVideoElement): void>} callback Function to call when
 *        video is ready.
 */
var startPlayingAndWaitForVideo = function(video, callback) {
  var gotPlaying = false;
  var gotTimeUpdate = false;

  var maybeCallCallback = function() {
    if (gotPlaying && gotTimeUpdate && callback) {
      callback(video);
      callback = undefined;
      video.removeEventListener('playing', playingListener, true);
      video.removeEventListener('timeupdate', timeupdateListener, true);
    }
  };

  var playingListener = function() {
    gotPlaying = true;
    maybeCallCallback();
  };

  var timeupdateListener = function() {
    // Checking to make sure the current time has advanced beyond
    // the start time seems to be a reliable heuristic that the
    // video element has data that can be consumed.
    if (video.currentTime > 0.0) {
      gotTimeUpdate = true;
      maybeCallCallback();
    }
  };

  video.addEventListener('playing', playingListener, true);
  video.addEventListener('timeupdate', timeupdateListener, true);
  video.loop = true;
  video.play();
};

var getHost = function(url) {
  url = url.replace("\\", "/");
  var pos = url.indexOf("://");
  if (pos >= 0) {
    url = url.substr(pos + 3);
  }
  var parts = url.split('/');
  return parts[0];
}

// This function returns the last 2 words of the domain of a URL
// This is probably not the correct check but it will do for now.
var getBaseDomain = function(host) {
  var parts = host.split(":");
  var hostname = parts[0];
  var port = parts[1] || "80";
  parts = hostname.split(".");
  if(parts.length < 2)
    return hostname + ":" + port;
  var tld = parts[parts.length-1];
  var domain = parts[parts.length-2];
  return domain + "." + tld + ":" + port;
}

var runningOnLocalhost = function() {
  return window.location.hostname.indexOf("localhost") != -1 ||
      window.location.hostname.indexOf("127.0.0.1") != -1;
}

var getLocalCrossOrigin = function() {
  var domain;
  if (window.location.host.indexOf("localhost") != -1) {
    domain = "127.0.0.1";
  } else {
    domain = "localhost";
  }

  var port = window.location.port || "80";
  return window.location.protocol + "//" + domain + ":" + port
}

var getRelativePath = function(path) {
  var relparts = window.location.pathname.split("/");
  relparts.pop(); // Pop off filename
  var pathparts = path.split("/");

  var i;
  for (i = 0; i < pathparts.length; ++i) {
    switch (pathparts[i]) {
      case "": break;
      case ".": break;
      case "..":
        relparts.pop();
        break;
      default:
        relparts.push(pathparts[i]);
        break;
    }
  }

  return relparts.join("/");
}

var setupImageForCrossOriginTest = function(img, imgUrl, localUrl, callback) {
  window.addEventListener("load", function() {
    if (typeof(img) == "string")
      img = document.querySelector(img);
    if (!img)
      img = new Image();

    img.addEventListener("load", callback, false);
    img.addEventListener("error", callback, false);

    if (runningOnLocalhost())
      img.src = getLocalCrossOrigin() + getRelativePath(localUrl);
    else
      img.src = getUrlOptions().imgUrl || imgUrl;
  }, false);
}

return {
  addShaderSource: addShaderSource,
  addShaderSources: addShaderSources,
  cancelAnimFrame: cancelAnimFrame,
  create3DContext: create3DContext,
  create3DContextWithWrapperThatThrowsOnGLError:
      create3DContextWithWrapperThatThrowsOnGLError,
  checkAreaInAndOut: checkAreaInAndOut,
  checkCanvas: checkCanvas,
  checkCanvasRect: checkCanvasRect,
  checkCanvasRectColor: checkCanvasRectColor,
  checkTextureSize: checkTextureSize,
  clipToRange: clipToRange,
  createColoredTexture: createColoredTexture,
  createProgram: createProgram,
  clearAndDrawUnitQuad: clearAndDrawUnitQuad,
  clearAndDrawIndexedQuad: clearAndDrawIndexedQuad,
  drawUnitQuad: drawUnitQuad,
  drawIndexedQuad: drawIndexedQuad,
  drawUByteColorQuad: drawUByteColorQuad,
  drawFloatColorQuad: drawFloatColorQuad,
  dumpShadersInfo: dumpShadersInfo,
  endsWith: endsWith,
  fillTexture: fillTexture,
  getBytesPerComponent: getBytesPerComponent,
  getExtensionPrefixedNames: getExtensionPrefixedNames,
  getExtensionWithKnownPrefixes: getExtensionWithKnownPrefixes,
  getFileListAsync: getFileListAsync,
  getLastError: getLastError,
  getPrefixedProperty: getPrefixedProperty,
  getScript: getScript,
  getSupportedExtensionWithKnownPrefixes: getSupportedExtensionWithKnownPrefixes,
  getTypedArrayElementsPerPixel: getTypedArrayElementsPerPixel,
  getUrlArguments: getUrlArguments,
  getUrlOptions: getUrlOptions,
  getAttribMap: getAttribMap,
  getUniformMap: getUniformMap,
  glEnumToString: glEnumToString,
  glErrorShouldBe: glErrorShouldBe,
  glTypeToTypedArrayType: glTypeToTypedArrayType,
  hasAttributeCaseInsensitive: hasAttributeCaseInsensitive,
  insertImage: insertImage,
  loadImageAsync: loadImageAsync,
  loadImagesAsync: loadImagesAsync,
  loadProgram: loadProgram,
  loadProgramFromFile: loadProgramFromFile,
  loadProgramFromScript: loadProgramFromScript,
  loadProgramFromScriptExpectError: loadProgramFromScriptExpectError,
  loadShader: loadShader,
  loadShaderFromFile: loadShaderFromFile,
  loadShaderFromScript: loadShaderFromScript,
  loadStandardProgram: loadStandardProgram,
  loadStandardVertexShader: loadStandardVertexShader,
  loadStandardFragmentShader: loadStandardFragmentShader,
  loadTextFileAsync: loadTextFileAsync,
  loadTexture: loadTexture,
  log: log,
  loggingOff: loggingOff,
  makeImage: makeImage,
  makeImageFromCanvas: makeImageFromCanvas,
  makeVideo: makeVideo,
  error: error,
  shallowCopyObject: shallowCopyObject,
  setupColorQuad: setupColorQuad,
  setupProgram: setupProgram,
  setupQuad: setupQuad,
  setupIndexedQuad: setupIndexedQuad,
  setupIndexedQuadWithOptions: setupIndexedQuadWithOptions,
  setupSimpleColorFragmentShader: setupSimpleColorFragmentShader,
  setupSimpleColorVertexShader: setupSimpleColorVertexShader,
  setupSimpleColorProgram: setupSimpleColorProgram,
  setupSimpleTextureFragmentShader: setupSimpleTextureFragmentShader,
  setupSimpleTextureProgram: setupSimpleTextureProgram,
  setupSimpleTextureVertexShader: setupSimpleTextureVertexShader,
  setupSimpleVertexColorFragmentShader: setupSimpleVertexColorFragmentShader,
  setupSimpleVertexColorProgram: setupSimpleVertexColorProgram,
  setupSimpleVertexColorVertexShader: setupSimpleVertexColorVertexShader,
  setupNoTexCoordTextureProgram: setupNoTexCoordTextureProgram,
  setupNoTexCoordTextureVertexShader: setupNoTexCoordTextureVertexShader,
  setupTexturedQuad: setupTexturedQuad,
  setupTexturedQuadWithTexCoords: setupTexturedQuadWithTexCoords,
  setupUnitQuad: setupUnitQuad,
  setupUnitQuadWithTexCoords: setupUnitQuadWithTexCoords,
  setFloatDrawColor: setFloatDrawColor,
  setUByteDrawColor: setUByteDrawColor,
  startPlayingAndWaitForVideo: startPlayingAndWaitForVideo,
  startsWith: startsWith,
  shouldGenerateGLError: shouldGenerateGLError,
  readFile: readFile,
  readFileList: readFileList,
  replaceParams: replaceParams,
  requestAnimFrame: requestAnimFrame,
  runSteps: runSteps,
  waitForComposite: waitForComposite,

  // fullscreen api
  setupFullscreen: setupFullscreen,

  getHost: getHost,
  getBaseDomain: getBaseDomain,
  runningOnLocalhost: runningOnLocalhost,
  getLocalCrossOrigin: getLocalCrossOrigin,
  getRelativePath: getRelativePath,
  setupImageForCrossOriginTest: setupImageForCrossOriginTest,

  none: false
};

}());

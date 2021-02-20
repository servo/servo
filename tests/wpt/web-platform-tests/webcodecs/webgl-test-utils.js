// Copyright (c) 2011 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

WebGLTestUtils = (function() {
  /**
   * Converts a WebGL enum to a string
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {number} value The enum value.
   * @return {string} The enum as a string.
   */
  var glEnumToString = function(gl, value) {
    for (var p in gl) {
      if (gl[p] == value) {
        return p;
      }
    }
    return '0x' + value.toString(16);
  };

  var lastError = '';

  /**
   * Returns the last compiler/linker error.
   * @return {string} The last compiler/linker error.
   */
  var getLastError = function() {
    return lastError;
  };

  // clang-format off

  /**
   * A vertex shader for a single texture.
   * @type {string}
   */
  var simpleTextureVertexShader = [
    'attribute vec4 vPosition;',  //
    'attribute vec2 texCoord0;',
    'varying vec2 texCoord;',
    'void main() {',
    '    gl_Position = vPosition;',
    '    texCoord = texCoord0;',
    '}'
  ].join('\n');

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
    '}'
  ].join('\n');

  // clang-format on

  /**
   * Creates a simple texture vertex shader.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @return {!WebGLShader}
   */
  var setupSimpleTextureVertexShader = function(gl) {
    return loadShader(gl, simpleTextureVertexShader, gl.VERTEX_SHADER);
  };

  /**
   * Creates a simple texture fragment shader.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @return {!WebGLShader}
   */
  var setupSimpleTextureFragmentShader = function(gl) {
    return loadShader(gl, simpleTextureFragmentShader, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a program, attaches shaders, binds attrib locations, links the
   * program and calls useProgram.
   * @param {!Array.<!WebGLShader>} shaders The shaders to attach .
   * @param {!Array.<string>} opt_attribs The attribs names.
   * @param {!Array.<number>} opt_locations The locations for the attribs.
   */
  var setupProgram = function(gl, shaders, opt_attribs, opt_locations) {
    var realShaders = [];
    var program = gl.createProgram();
    for (var ii = 0; ii < shaders.length; ++ii) {
      var shader = shaders[ii];
      if (typeof shader == 'string') {
        var element = document.getElementById(shader);
        if (element) {
          shader = loadShaderFromScript(gl, shader);
        } else {
          shader = loadShader(
              gl, shader, ii ? gl.FRAGMENT_SHADER : gl.VERTEX_SHADER);
        }
      }
      gl.attachShader(program, shader);
    }
    if (opt_attribs) {
      for (var ii = 0; ii < opt_attribs.length; ++ii) {
        gl.bindAttribLocation(
            program, opt_locations ? opt_locations[ii] : ii, opt_attribs[ii]);
      }
    }
    gl.linkProgram(program);

    // Check the link status
    var linked = gl.getProgramParameter(program, gl.LINK_STATUS);
    if (!linked) {
      gl.deleteProgram(program);
      return null;
    }

    gl.useProgram(program);
    return program;
  };

  /**
   * Creates a simple texture program.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture
   *     coords.
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
        gl, [vs, fs], ['vPosition', 'texCoord0'],
        [opt_positionLocation, opt_texcoordLocation]);
    if (!program) {
      gl.deleteShader(fs);
      gl.deleteShader(vs);
    }
    gl.useProgram(program);
    return program;
  };

  /**
   * Creates buffers for a textured unit quad and attaches them to vertex
   * attribs.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture
   *     coords.
   * @return {!Array.<WebGLBuffer>} The buffer objects that were
   *      created.
   */
  var setupUnitQuad = function(gl, opt_positionLocation, opt_texcoordLocation) {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var objects = [];

    var vertexObject = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexObject);
    gl.bufferData(
        gl.ARRAY_BUFFER, new Float32Array([
          1.0, 1.0, 0.0, -1.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0,
          -1.0, 0.0, 1.0, -1.0, 0.0
        ]),
        gl.STATIC_DRAW);
    gl.enableVertexAttribArray(opt_positionLocation);
    gl.vertexAttribPointer(opt_positionLocation, 3, gl.FLOAT, false, 0, 0);
    objects.push(vertexObject);

    var vertexObject = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexObject);
    gl.bufferData(
        gl.ARRAY_BUFFER,
        new Float32Array(
            [1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0]),
        gl.STATIC_DRAW);
    gl.enableVertexAttribArray(opt_texcoordLocation);
    gl.vertexAttribPointer(opt_texcoordLocation, 2, gl.FLOAT, false, 0, 0);
    objects.push(vertexObject);
    return objects;
  };

  /**
   * Creates a program and buffers for rendering a textured quad.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture
   *     coords.
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
   * Draws a previously setup quad.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {!Array.<number>} opt_color The color to fill clear with before
   *        drawing. A 4 element array where each element is in the range 0 to
   *        255. Default [255, 255, 255, 255]
   */
  var drawQuad = function(gl, opt_color) {
    opt_color = opt_color || [255, 255, 255, 255];
    gl.clearColor(
        opt_color[0] / 255, opt_color[1] / 255, opt_color[2] / 255,
        opt_color[3] / 255);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  };

  /**
   * Links a WebGL program, throws if there are errors.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {!WebGLProgram} program The WebGLProgram to link.
   * @param {function(string): void) opt_errorCallback callback for errors.
   */
  var linkProgram = function(gl, program, opt_errorCallback) {
    // Link the program
    gl.linkProgram(program);

    // Check the link status
    var linked = gl.getProgramParameter(program, gl.LINK_STATUS);
    if (!linked) {
      // something went wrong with the link
      gl.deleteProgram(program);
      return false;
    }

    return true;
  };

  /**
   * Loads a shader.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {string} shaderSource The shader source.
   * @param {number} shaderType The type of shader.
   * @param {function(string): void) opt_errorCallback callback for errors.
   * @return {!WebGLShader} The created shader.
   */
  var loadShader =
      function(gl, shaderSource, shaderType, opt_errorCallback) {
    var errFn = opt_errorCallback || (_ => {});
    // Create the shader object
    var shader = gl.createShader(shaderType);
    if (shader == null) {
      errFn('*** Error: unable to create shader \'' + shaderSource + '\'');
      return null;
    }

    // Load the shader source
    gl.shaderSource(shader, shaderSource);
    var err = gl.getError();
    if (err != gl.NO_ERROR) {
      errFn(
          '*** Error loading shader \'' + shader +
          '\':' + glEnumToString(gl, err));
      return null;
    }

    // Compile the shader
    gl.compileShader(shader);

    // Check the compile status
    var compiled = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
    if (!compiled) {
      // Something went wrong during compilation; get the error
      lastError = gl.getShaderInfoLog(shader);
      errFn('*** Error compiling shader \'' + shader + '\':' + lastError);
      gl.deleteShader(shader);
      return null;
    }

    return shader;
  }

  /**
   * Loads shaders from source, creates a program, attaches the shaders and
   * links.
   * @param {!WebGLContext} gl The WebGLContext to use.
   * @param {string} vertexShader The vertex shader.
   * @param {string} fragmentShader The fragment shader.
   * @param {function(string): void) opt_errorCallback callback for errors.
   * @return {!WebGLProgram} The created program.
   */
  var loadProgram = function(
      gl, vertexShader, fragmentShader, opt_errorCallback) {
    var program = gl.createProgram();
    gl.attachShader(
        program,
        loadShader(gl, vertexShader, gl.VERTEX_SHADER, opt_errorCallback));
    gl.attachShader(
        program,
        loadShader(gl, fragmentShader, gl.FRAGMENT_SHADER, opt_errorCallback));
    return linkProgram(gl, program, opt_errorCallback) ? program : null;
  };

  return {
    drawQuad: drawQuad,
    getLastError: getLastError,
    glEnumToString: glEnumToString,
    loadProgram: loadProgram,
    loadShader: loadShader,
    setupProgram: setupProgram,
    setupSimpleTextureFragmentShader: setupSimpleTextureFragmentShader,
    setupSimpleTextureProgram: setupSimpleTextureProgram,
    setupSimpleTextureVertexShader: setupSimpleTextureVertexShader,
    setupTexturedQuad: setupTexturedQuad,
    setupUnitQuad: setupUnitQuad,
  };
}());

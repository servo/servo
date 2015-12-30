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
var TexImageUtils = (function() {

  "use strict";

  var wtu = WebGLTestUtils;

  /**
   * A vertex shader for a single texture.
   * @type {string}
   */
  var simpleTextureVertexShaderES3 = [
    '#version 300 es',
    'in vec4 vPosition;',
    'in vec2 texCoord0;',
    'out vec2 texCoord;',
    'void main() {',
    '    gl_Position = vPosition;',
    '    texCoord = texCoord0;',
    '}'].join('\n');

  /**
   * A fragment shader for a single integer texture.
   * @type {string}
   */
  // Note we always output 1.0 for alpha because if the texture does not contain
  // alpha channel, sampling returns 1; for RGBA textures, sampling returns [0,255].
  var simpleUintTextureFragmentShaderES3 = [
    '#version 300 es',
    'precision mediump float;',
    'uniform mediump usampler2D tex;',
    'in vec2 texCoord;',
    'out vec4 fragData;',
    'void main() {',
    '    uvec4 data = texture(tex, texCoord);',
    '    fragData = vec4(float(data[0])/255.0,',
    '                    float(data[1])/255.0,',
    '                    float(data[2])/255.0,',
    '                    1.0);',
    '}'].join('\n');

  /**
   * A fragment shader for a single cube map integer texture.
   * @type {string}
   */
  // Note we always output 1.0 for alpha because if the texture does not contain
  // alpha channel, sampling returns 1; for RGBA textures, sampling returns [0,255].
  var simpleCubeMapUintTextureFragmentShaderES3 = [
    '#version 300 es',
    'precision mediump float;',
    'uniform mediump usamplerCube tex;',
    'uniform int face;',
    'in vec2 texCoord;',
    'out vec4 fragData;',
    'void main() {',
    // Transform [0, 1] -> [-1, 1]
    '    vec2 texC2 = (texCoord * 2.) - 1.;',
    // Transform 2d tex coord. to each face of TEXTURE_CUBE_MAP coord.
    '    vec3 texCube = vec3(0., 0., 0.);',
    '    if (face == 34069) {',         // TEXTURE_CUBE_MAP_POSITIVE_X
    '        texCube = vec3(1., -texC2.y, -texC2.x);',
    '    } else if (face == 34070) {',  // TEXTURE_CUBE_MAP_NEGATIVE_X
    '        texCube = vec3(-1., -texC2.y, texC2.x);',
    '    } else if (face == 34071) {',  // TEXTURE_CUBE_MAP_POSITIVE_Y
    '        texCube = vec3(texC2.x, 1., texC2.y);',
    '    } else if (face == 34072) {',  // TEXTURE_CUBE_MAP_NEGATIVE_Y
    '        texCube = vec3(texC2.x, -1., -texC2.y);',
    '    } else if (face == 34073) {',  // TEXTURE_CUBE_MAP_POSITIVE_Z
    '        texCube = vec3(texC2.x, -texC2.y, 1.);',
    '    } else if (face == 34074) {',  // TEXTURE_CUBE_MAP_NEGATIVE_Z
    '        texCube = vec3(-texC2.x, -texC2.y, -1.);',
    '    }',
    '    uvec4 data = texture(tex, texCube);',
    '    fragData = vec4(float(data[0])/255.0,',
    '                    float(data[1])/255.0,',
    '                    float(data[2])/255.0,',
    '                    1.0);',
    '}'].join('\n');

  /**
   * A fragment shader for a single 3D texture.
   * @type {string}
   */
  // Note we always set the tex coordinate t to 0.
  var simple3DTextureFragmentShaderES3 = [
    '#version 300 es',
    'precision mediump float;',
    'uniform mediump sampler3D tex;',
    'in vec2 texCoord;',
    'out vec4 fragData;',
    'void main() {',
    '    fragData = vec4(texture(tex, vec3(texCoord, 0.0)).rgb, 1.0);',
    '}'].join('\n');

  /**
   * A fragment shader for a single 3D integer texture.
   * @type {string}
   */
  // Note we always set the tex coordinate t to 0.
  // Note we always output 1.0 for alpha because if the texture does not contain
  // alpha channel, sampling returns 1; for RGBA textures, sampling returns [0,255].
  var simple3DUintTextureFragmentShaderES3 = [
    '#version 300 es',
    'precision mediump float;',
    'uniform mediump usampler3D tex;',
    'in vec2 texCoord;',
    'out vec4 fragData;',
    'void main() {',
    '    vec4 data = vec4(texture(tex, vec3(texCoord, 0.0)).rgb, 1.0);',
    '    fragData = vec4(float(data[0])/255.0,',
    '                    float(data[1])/255.0,',
    '                    float(data[2])/255.0,',
    '                    1.0);',
    '}'].join('\n');

  /**
   * A fragment shader for a single 2D_ARRAY texture.
   * @type {string}
   */
  // Note we always use the first image in the array.
  var simple2DArrayTextureFragmentShaderES3 = [
    '#version 300 es',
    'precision mediump float;',
    'uniform mediump sampler2DArray tex;',
    'in vec2 texCoord;',
    'out vec4 fragData;',
    'void main() {',
    '    fragData = vec4(texture(tex, vec3(texCoord, 0.0)).rgb, 1.0);',
    '}'].join('\n');

  /**
   * A fragment shader for a single 2D_ARRAY unsigned integer texture.
   * @type {string}
   */
  // Note we always use the first image in the array.
  // Note we always output 1.0 for alpha because if the texture does not contain
  // alpha channel, sampling returns 1; for RGBA textures, sampling returns [0,255].
  var simple2DArrayUintTextureFragmentShaderES3 = [
    '#version 300 es',
    'precision mediump float;',
    'uniform mediump usampler2DArray tex;',
    'in vec2 texCoord;',
    'out vec4 fragData;',
    'void main() {',
    '    vec4 data = vec4(texture(tex, vec3(texCoord, 0.0)).rgb, 1.0);',
    '    fragData = vec4(float(data[0])/255.0,',
    '                    float(data[1])/255.0,',
    '                    float(data[2])/255.0,',
    '                    1.0);',
    '}'].join('\n');


  /**
   * Creates a simple texture vertex shader.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimpleTextureVertexShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simpleTextureVertexShaderES3, gl.VERTEX_SHADER);
  };

  /**
   * Creates a simple unsigned integer texture fragment shader.
   * Output is scaled by 1/255 to bring the result into normalized float range.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimpleUintTextureFragmentShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simpleUintTextureFragmentShaderES3, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a simple cube map unsigned integer texture fragment shader.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimpleCubeMapUintTextureFragmentShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simpleCubeMapUintTextureFragmentShaderES3, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a simple 3D texture fragment shader.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimple3DTextureFragmentShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simple3DTextureFragmentShaderES3, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a simple 3D integer texture fragment shader.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimple3DUintTextureFragmentShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simple3DUintTextureFragmentShaderES3, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a simple 2D_ARRAY texture fragment shader.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimple2DArrayTextureFragmentShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simple2DArrayTextureFragmentShaderES3, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a simple 2D_ARRAY integer texture fragment shader.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLShader}
   */
  var setupSimple2DArrayUintTextureFragmentShader = function(gl) {
    return WebGLTestUtils.loadShader(gl, simple2DArrayUintTextureFragmentShaderES3, gl.FRAGMENT_SHADER);
  };

  /**
   * Creates a simple unsigned integer texture program.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture coords.
   * @return {WebGLProgram}
   */
  var setupSimpleUintTextureProgram = function(gl, opt_positionLocation, opt_texcoordLocation)
  {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var vs = setupSimpleTextureVertexShader(gl),
        fs = setupSimpleUintTextureFragmentShader(gl);
    if (!vs || !fs) {
      return null;
    }
    var program = WebGLTestUtils.setupProgram(
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
   * Creates a simple cube map unsigned integer texture program.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture coords.
   * @return {WebGLProgram}
   */
  var setupSimpleCubeMapUintTextureProgram = function(gl, opt_positionLocation, opt_texcoordLocation) {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var vs = setupSimpleTextureVertexShader(gl);
    var fs = setupSimpleCubeMapUintTextureFragmentShader(gl);
    if (!vs || !fs) {
      return null;
    }
    var program = WebGLTestUtils.setupProgram(
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
   * Creates a simple 3D texture program.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture coords.
   * @return {WebGLProgram}
   */
  var setupSimple3DTextureProgram = function(gl, opt_positionLocation, opt_texcoordLocation)
  {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var vs = setupSimpleTextureVertexShader(gl),
        fs = setupSimple3DTextureFragmentShader(gl);
    if (!vs || !fs) {
      return null;
    }
    var program = WebGLTestUtils.setupProgram(
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
   * Creates a simple 3D unsigned integer texture program.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture coords.
   * @return {WebGLProgram}
   */
  var setupSimple3DUintTextureProgram = function(gl, opt_positionLocation, opt_texcoordLocation)
  {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var vs = setupSimpleTextureVertexShader(gl),
        fs = setupSimple3DUintTextureFragmentShader(gl);
    if (!vs || !fs) {
      return null;
    }
    var program = WebGLTestUtils.setupProgram(
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
   * Creates a simple 2D_ARRAY texture program.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture coords.
   * @return {WebGLProgram}
   */
  var setupSimple2DArrayTextureProgram = function(gl, opt_positionLocation, opt_texcoordLocation)
  {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var vs = setupSimpleTextureVertexShader(gl),
        fs = setupSimple2DArrayTextureFragmentShader(gl);
    if (!vs || !fs) {
      return null;
    }
    var program = WebGLTestUtils.setupProgram(
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
   * Creates a simple 2D_ARRAY unsigned integer texture program.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {number} opt_positionLocation The attrib location for position.
   * @param {number} opt_texcoordLocation The attrib location for texture coords.
   * @return {WebGLProgram}
   */
  var setupSimple2DArrayUintTextureProgram = function(gl, opt_positionLocation, opt_texcoordLocation)
  {
    opt_positionLocation = opt_positionLocation || 0;
    opt_texcoordLocation = opt_texcoordLocation || 1;
    var vs = setupSimpleTextureVertexShader(gl),
        fs = setupSimple2DArrayUintTextureFragmentShader(gl);
    if (!vs || !fs) {
      return null;
    }
    var program = WebGLTestUtils.setupProgram(
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
   * Creates a program and buffers for rendering a unsigned integer textured quad.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLProgram}
   */
  var setupUintTexturedQuad = function(gl) {
    var program = setupSimpleUintTextureProgram(gl);
    wtu.setupUnitQuad(gl);
    return program;
  };

  /**
   * Creates a program and buffers for rendering a textured quad with
   * a cube map unsigned integer texture.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @return {!WebGLProgram}
   */
  var setupUintTexturedQuadWithCubeMap = function(gl)
  {
    var program = setupSimpleCubeMapUintTextureProgram(gl);
    wtu.setupUnitQuad(gl);
    return program;
  };

  /**
   * Does the GL internal format represent an unsigned integer format
   * texture?
   * @return {boolean}
   */
  var isUintFormat = function(internalFormat)
  {
    return (internalFormat == "R8UI" ||
            internalFormat == "RG8UI" ||
            internalFormat == "RGB8UI" ||
            internalFormat == "RGBA8UI");
  };

  /**
   * Createa a program and buffers for rendering a textured quad for
   * tex-image-and-sub-image tests. Handle selection of correct
   * program to handle texture format.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {string} internalFormat The internal format for texture to be tested.
   */
  var setupTexturedQuad = function(gl, internalFormat)
  {
    if (isUintFormat(internalFormat))
      return setupUintTexturedQuad(gl);

    return wtu.setupTexturedQuad(gl);
  };

  /**
   * Createa a program and buffers for rendering a textured quad with
   * a cube map for tex-image-and-sub-image tests. Handle selection of
   * correct program to handle texture format.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {string} internalFormat The internal format for texture to be tested.
   */
  function setupTexturedQuadWithCubeMap(gl, internalFormat)
  {
    if (isUintFormat(internalFormat))
      return setupUintTexturedQuadWithCubeMap(gl);

    return wtu.setupTexturedQuadWithCubeMap(gl);
  }

  /**
   * Createa a program and buffers for rendering a textured quad with a 3D texture
   * for tex-image-and-sub-image tests. Handle selection of correct
   * program to handle texture format.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {string} internalFormat The internal format for texture to be tested.
   */
  var setupTexturedQuadWith3D = function(gl, internalFormat)
  {
    var program;
    if (isUintFormat(internalFormat))
      program = setupSimple3DUintTextureProgram(gl);
    else
      program = setupSimple3DTextureProgram(gl);
    wtu.setupUnitQuad(gl);
    return program;
  };

  /**
   * Createa a program and buffers for rendering a textured quad with a 2D_ARRAY
   * texture for tex-image-and-sub-image tests. Handle selection of correct
   * program to handle texture format.
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use.
   * @param {string} internalFormat The internal format for texture to be tested.
   */
  var setupTexturedQuadWith2DArray = function(gl, internalFormat)
  {
    var program;
    if (isUintFormat(internalFormat))
      program = setupSimple2DArrayUintTextureProgram(gl);
    else
      program = setupSimple2DArrayTextureProgram(gl);
    wtu.setupUnitQuad(gl);
    return program;
  };

  return {
    setupTexturedQuad: setupTexturedQuad,
    setupTexturedQuadWithCubeMap: setupTexturedQuadWithCubeMap,
    setupTexturedQuadWith3D: setupTexturedQuadWith3D,
    setupTexturedQuadWith2DArray: setupTexturedQuadWith2DArray
  };

}());

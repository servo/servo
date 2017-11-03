/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('modules.shared.glsTextureTestUtil');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuStringTemplate');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTexLookupVerifier');
goog.require('framework.common.tcuTexCompareVerifier');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.delibs.debase.deRandom');

goog.scope(function() {
var tcuTexLookupVerifier = framework.common.tcuTexLookupVerifier;
var tcuTexCompareVerifier = framework.common.tcuTexCompareVerifier;
var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
var gluDrawUtil = framework.opengl.gluDrawUtil;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuTexture = framework.common.tcuTexture;
var tcuSurface = framework.common.tcuSurface;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var tcuStringTemplate = framework.common.tcuStringTemplate;
var deMath = framework.delibs.debase.deMath;
var tcuImageCompare = framework.common.tcuImageCompare;
var tcuPixelFormat = framework.common.tcuPixelFormat;
var tcuRGBA = framework.common.tcuRGBA;
var deRandom = framework.delibs.debase.deRandom;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

var MIN_SUBPIXEL_BITS = 4;

/**
 * @enum
 */
glsTextureTestUtil.textureType = {
    TEXTURETYPE_2D: 0,
    TEXTURETYPE_CUBE: 1,
    TEXTURETYPE_2D_ARRAY: 2,
    TEXTURETYPE_3D: 3,
    TEXTURETYPE_CUBE_ARRAY: 4,
    TEXTURETYPE_1D: 5,
    TEXTURETYPE_1D_ARRAY: 6,
    TEXTURETYPE_BUFFER: 7
};

/**
 * @enum
 */
glsTextureTestUtil.samplerType = {
    SAMPLERTYPE_FLOAT: 0,
    SAMPLERTYPE_INT: 1,
    SAMPLERTYPE_UINT: 2,
    SAMPLERTYPE_SHADOW: 3,

    SAMPLERTYPE_FETCH_FLOAT: 4,
    SAMPLERTYPE_FETCH_INT: 5,
    SAMPLERTYPE_FETCH_UINT: 6
};

/**
 * @param {tcuTexture.TextureFormat} format
 * @return {glsTextureTestUtil.samplerType}
 */
glsTextureTestUtil.getSamplerType = function(format) {
    if (format == null)
        throw new Error('Missing format information');

    switch (format.type) {
        case tcuTexture.ChannelType.SIGNED_INT8:
        case tcuTexture.ChannelType.SIGNED_INT16:
        case tcuTexture.ChannelType.SIGNED_INT32:
            return glsTextureTestUtil.samplerType.SAMPLERTYPE_INT;

        case tcuTexture.ChannelType.UNSIGNED_INT8:
        case tcuTexture.ChannelType.UNSIGNED_INT32:
        case tcuTexture.ChannelType.UNSIGNED_INT_1010102_REV:
            return glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT;

        // Texture formats used in depth/stencil textures.
        case tcuTexture.ChannelType.UNSIGNED_INT16:
        case tcuTexture.ChannelType.UNSIGNED_INT_24_8:
            return (format.order == tcuTexture.ChannelOrder.D || format.order == tcuTexture.ChannelOrder.DS) ? glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT : glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT;

        default:
            return glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT;
    }
};

/**
 * @constructor
 * @param {HTMLElement} canvas
 * @param {number} preferredWidth
 * @param {number} preferredHeight
 * @param {number=} seed
 */
glsTextureTestUtil.RandomViewport = function(canvas, preferredWidth, preferredHeight, seed) {
    this.width = Math.min(canvas.width, preferredWidth);
    this.height = Math.min(canvas.height, preferredHeight);

    if (typeof seed === 'undefined')
        seed = preferredWidth + preferredHeight;

    var rnd = new deRandom.Random(seed);
    this.x = rnd.getInt(0, canvas.width - this.width);
    this.y = rnd.getInt(0, canvas.height - this.height);
};

/**
 * @constructor
 * @param {glsTextureTestUtil.textureType} texType
 */
glsTextureTestUtil.RenderParams = function(texType) {
    this.flags = {
        projected: false,
        use_bias: false,
        log_programs: false,
        log_uniforms: false
    };
    this.texType = texType;
    this.w = [1, 1, 1, 1];
    this.bias = 0;
    this.ref = 0;
    this.colorScale = [1, 1, 1, 1];
    this.colorBias = [0, 0, 0, 0];
    this.samplerType = glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT;
};

/**
 * @enum
 */
glsTextureTestUtil.lodMode = {
    EXACT: 0, //!< Ideal lod computation.
    MIN_BOUND: 1, //!< Use estimation range minimum bound.
    MAX_BOUND: 2 //!< Use estimation range maximum bound.

};

/**
 * @constructor
 * @extends {glsTextureTestUtil.RenderParams}
 * @param {glsTextureTestUtil.textureType} texType
 * @param {tcuTexture.Sampler=} sampler
 * @param {glsTextureTestUtil.lodMode=} lodMode_
 */
glsTextureTestUtil.ReferenceParams = function(texType, sampler, lodMode_) {
    glsTextureTestUtil.RenderParams.call(this, texType);
    if (sampler)
        this.sampler = sampler;
    if (lodMode_)
        this.lodMode = lodMode_;
    else
        this.lodMode = glsTextureTestUtil.lodMode.EXACT;
    this.minLod = -1000;
    this.maxLod = 1000;
    this.baseLevel = 0;
    this.maxLevel = 1000;
};

glsTextureTestUtil.ReferenceParams.prototype = Object.create(glsTextureTestUtil.RenderParams.prototype);

/** Copy constructor */
glsTextureTestUtil.ReferenceParams.prototype.constructor = glsTextureTestUtil.ReferenceParams;

/**
 * @param {Array<number>} bottomLeft
 * @param {Array<number>} topRight
 * @return {Array<number>}
 */
glsTextureTestUtil.computeQuadTexCoord2D = function(bottomLeft, topRight) {
    var dst = [];
    dst.length = 4 * 2;

    dst[0] = bottomLeft[0]; dst[1] = bottomLeft[1];
    dst[2] = bottomLeft[0]; dst[3] = topRight[1];
    dst[4] = topRight[0]; dst[5] = bottomLeft[1];
    dst[6] = topRight[0]; dst[7] = topRight[1];

    return dst;
};

/**
 * @param {tcuTexture.CubeFace} face
 * @return {Array<number>}
 */
glsTextureTestUtil.computeQuadTexCoordCube = function(face) {
    var texCoordNegX = [
        -1, 1, -1,
        -1, -1, -1,
        -1, 1, 1,
        -1, -1, 1
    ];
    var texCoordPosX = [
        +1, 1, 1,
        +1, -1, 1,
        +1, 1, -1,
        +1, -1, -1
    ];
    var texCoordNegY = [
        -1, -1, 1,
        -1, -1, -1,
         1, -1, 1,
         1, -1, -1
    ];
    var texCoordPosY = [
        -1, +1, -1,
        -1, +1, 1,
         1, +1, -1,
         1, +1, 1
    ];
    var texCoordNegZ = [
         1, 1, -1,
         1, -1, -1,
        -1, 1, -1,
        -1, -1, -1
    ];
    var texCoordPosZ = [
        -1, 1, +1,
        -1, -1, +1,
         1, 1, +1,
         1, -1, +1
    ];

    switch (face) {
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X: return texCoordNegX;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X: return texCoordPosX;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y: return texCoordNegY;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y: return texCoordPosY;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z: return texCoordNegZ;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z: return texCoordPosZ;
    }
    throw new Error('Unrecognized face ' + face);
};

/**
 * @param {tcuTexture.CubeFace} face
 * @param {Array<number>} bottomLeft
 * @param {Array<number>} topRight
 * @return {Array<number>}
 */
glsTextureTestUtil.computeQuadTexCoordCubeFace = function(face, bottomLeft, topRight) {
    var dst = [];
    /** @type {number} */ var sRow = 0;
    /** @type {number} */ var tRow = 0;
    /** @type {number} */ var mRow = 0;
    /** @type {number} */ var sSign = 1.0;
    /** @type {number} */ var tSign = 1.0;
    /** @type {number} */ var mSign = 1.0;

    switch (face) {
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X: mRow = 0; sRow = 2; tRow = 1; mSign = -1.0; tSign = -1.0; break;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X: mRow = 0; sRow = 2; tRow = 1; sSign = -1.0; tSign = -1.0; break;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y: mRow = 1; sRow = 0; tRow = 2; mSign = -1.0; tSign = -1.0; break;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y: mRow = 1; sRow = 0; tRow = 2; break;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z: mRow = 2; sRow = 0; tRow = 1; mSign = -1.0; sSign = -1.0; tSign = -1.0; break;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z: mRow = 2; sRow = 0; tRow = 1; tSign = -1.0; break;
        default:
            throw new Error('Invalid cube face specified.');
    }

    dst[0 + mRow] = mSign;
    dst[3 + mRow] = mSign;
    dst[6 + mRow] = mSign;
    dst[9 + mRow] = mSign;

    dst[0 + sRow] = sSign * bottomLeft[0];
    dst[3 + sRow] = sSign * bottomLeft[0];
    dst[6 + sRow] = sSign * topRight[0];
    dst[9 + sRow] = sSign * topRight[0];

    dst[0 + tRow] = tSign * bottomLeft[1];
    dst[3 + tRow] = tSign * topRight[1];
    dst[6 + tRow] = tSign * bottomLeft[1];
    dst[9 + tRow] = tSign * topRight[1];

    return dst;
};

/**
 * @param {number} layerNdx
 * @param {Array<number>} bottomLeft
 * @param {Array<number>} topRight
 * @return {Array<number>}
 */
glsTextureTestUtil.computeQuadTexCoord2DArray = function(layerNdx, bottomLeft, topRight) {
    var dst = [];
    dst.length = 4 * 3;

    dst[0] = bottomLeft[0]; dst[1] = bottomLeft[1]; dst[2] = layerNdx;
    dst[3] = bottomLeft[0]; dst[4] = topRight[1]; dst[5] = layerNdx;
    dst[6] = topRight[0]; dst[7] = bottomLeft[1]; dst[8] = layerNdx;
    dst[9] = topRight[0]; dst[10] = topRight[1]; dst[11] = layerNdx;

    return dst;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @param {Array<number>} c
 * @return {Array<number>} a + (b - a) * c
 */
glsTextureTestUtil.selectCoords = function(a, b, c) {
    var x1 = deMath.subtract(b, a);
    var x2 = deMath.multiply(x1, c);
    var x3 = deMath.add(a, x2);
    return x3;
};

/**
 * @param {Array<number>} p0
 * @param {Array<number>} p1
 * @param {Array<number>} dirSwz
 * @return {Array<number>}
 */
glsTextureTestUtil.computeQuadTexCoord3D = function(p0, p1, dirSwz) {
    var dst = [];
    dst.length = 4 * 3;

    var f0 = deMath.swizzle(([0, 0, 0]), [dirSwz[0], dirSwz[1], dirSwz[2]]);
    var f1 = deMath.swizzle(([0, 1, 0]), [dirSwz[0], dirSwz[1], dirSwz[2]]);
    var f2 = deMath.swizzle(([1, 0, 0]), [dirSwz[0], dirSwz[1], dirSwz[2]]);
    var f3 = deMath.swizzle(([1, 1, 0]), [dirSwz[0], dirSwz[1], dirSwz[2]]);

    var v0 = glsTextureTestUtil.selectCoords(p0, p1, f0);
    var v1 = glsTextureTestUtil.selectCoords(p0, p1, f1);
    var v2 = glsTextureTestUtil.selectCoords(p0, p1, f2);
    var v3 = glsTextureTestUtil.selectCoords(p0, p1, f3);

    dst[0] = v0[0]; dst[1] = v0[1]; dst[2] = v0[2];
    dst[3] = v1[0]; dst[4] = v1[1]; dst[5] = v1[2];
    dst[6] = v2[0]; dst[7] = v2[1]; dst[8] = v2[2];
    dst[9] = v3[0]; dst[10] = v3[1]; dst[11] = v3[2];

    return dst;
};

/**
 * @enum
 */
glsTextureTestUtil.programType = {
    PROGRAM_2D_FLOAT: 0,
    PROGRAM_2D_INT: 1,
    PROGRAM_2D_UINT: 2,
    PROGRAM_2D_SHADOW: 3,

    PROGRAM_2D_FLOAT_BIAS: 4,
    PROGRAM_2D_INT_BIAS: 5,
    PROGRAM_2D_UINT_BIAS: 6,
    PROGRAM_2D_SHADOW_BIAS: 7,

    PROGRAM_1D_FLOAT: 8,
    PROGRAM_1D_INT: 9,
    PROGRAM_1D_UINT: 10,
    PROGRAM_1D_SHADOW: 11,

    PROGRAM_1D_FLOAT_BIAS: 12,
    PROGRAM_1D_INT_BIAS: 13,
    PROGRAM_1D_UINT_BIAS: 14,
    PROGRAM_1D_SHADOW_BIAS: 15,

    PROGRAM_CUBE_FLOAT: 16,
    PROGRAM_CUBE_INT: 17,
    PROGRAM_CUBE_UINT: 18,
    PROGRAM_CUBE_SHADOW: 19,

    PROGRAM_CUBE_FLOAT_BIAS: 20,
    PROGRAM_CUBE_INT_BIAS: 21,
    PROGRAM_CUBE_UINT_BIAS: 22,
    PROGRAM_CUBE_SHADOW_BIAS: 23,

    PROGRAM_1D_ARRAY_FLOAT: 24,
    PROGRAM_1D_ARRAY_INT: 25,
    PROGRAM_1D_ARRAY_UINT: 26,
    PROGRAM_1D_ARRAY_SHADOW: 27,

    PROGRAM_2D_ARRAY_FLOAT: 28,
    PROGRAM_2D_ARRAY_INT: 29,
    PROGRAM_2D_ARRAY_UINT: 30,
    PROGRAM_2D_ARRAY_SHADOW: 31,

    PROGRAM_3D_FLOAT: 32,
    PROGRAM_3D_INT: 33,
    PROGRAM_3D_UINT: 34,

    PROGRAM_3D_FLOAT_BIAS: 35,
    PROGRAM_3D_INT_BIAS: 36,
    PROGRAM_3D_UINT_BIAS: 37,

    PROGRAM_CUBE_ARRAY_FLOAT: 38,
    PROGRAM_CUBE_ARRAY_INT: 39,
    PROGRAM_CUBE_ARRAY_UINT: 40,
    PROGRAM_CUBE_ARRAY_SHADOW: 41,

    PROGRAM_BUFFER_FLOAT: 42,
    PROGRAM_BUFFER_INT: 43,
    PROGRAM_BUFFER_UINT: 44
};

/**
 * @constructor
 * @param {string} version GL version
 * @param {gluShaderUtil.precision} precision
 */
glsTextureTestUtil.ProgramLibrary = function(version, precision) {
    this.m_glslVersion = version;
    this.m_texCoordPrecision = precision;
};

/**
 * @param {glsTextureTestUtil.programType} program
 * @return {gluShaderProgram.ShaderProgram}
 */
glsTextureTestUtil.ProgramLibrary.prototype.getProgram = function(program) {
    /* TODO: Implement */
    // if (m_programs.find(program) != m_programs.end())
    //     return m_programs[program]; // Return from cache.

    var vertShaderTemplate =
        '${VTX_HEADER}' +
        '${VTX_IN} highp vec4 a_position;\n' +
        '${VTX_IN} ${PRECISION} ${TEXCOORD_TYPE} a_texCoord;\n' +
        '${VTX_OUT} ${PRECISION} ${TEXCOORD_TYPE} v_texCoord;\n' +
        '\n' +
        'void main (void)\n' +
        ' {\n' +
        ' gl_Position = a_position;\n' +
        ' v_texCoord = a_texCoord;\n' +
        '}\n';
    var fragShaderTemplate =
        '${FRAG_HEADER}' +
        '${FRAG_IN} ${PRECISION} ${TEXCOORD_TYPE} v_texCoord;\n' +
        'uniform ${PRECISION} float u_bias;\n' +
        'uniform ${PRECISION} float u_ref;\n' +
        'uniform ${PRECISION} vec4 u_colorScale;\n' +
        'uniform ${PRECISION} vec4 u_colorBias;\n' +
        'uniform ${PRECISION} ${SAMPLER_TYPE} u_sampler;\n' +
        '\n' +
        'void main (void)\n' +
        ' {\n' +
        ' ${FRAG_COLOR} = ${LOOKUP} * u_colorScale + u_colorBias;\n' +
        '}\n';

    var params = [];

    var isCube = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT, glsTextureTestUtil.programType.PROGRAM_CUBE_SHADOW_BIAS);
    var isArray = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_FLOAT, glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_SHADOW) ||
                            deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_FLOAT, glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_SHADOW);

    var is1D = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_1D_FLOAT, glsTextureTestUtil.programType.PROGRAM_1D_UINT_BIAS) ||
                            deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_FLOAT, glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_SHADOW) ||
                            deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_BUFFER_FLOAT, glsTextureTestUtil.programType.PROGRAM_BUFFER_UINT);

    var is2D = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_2D_FLOAT, glsTextureTestUtil.programType.PROGRAM_2D_UINT_BIAS) ||
                            deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_FLOAT, glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_SHADOW);

    var is3D = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_3D_FLOAT, glsTextureTestUtil.programType.PROGRAM_3D_UINT_BIAS);
    var isCubeArray = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_FLOAT, glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_SHADOW);
    var isBuffer = deMath.deInRange32(program, glsTextureTestUtil.programType.PROGRAM_BUFFER_FLOAT, glsTextureTestUtil.programType.PROGRAM_BUFFER_UINT);

    if (this.m_glslVersion === '100 es') {
        params['FRAG_HEADER'] = '';
        params['VTX_HEADER'] = '';
        params['VTX_IN'] = 'attribute';
        params['VTX_OUT'] = 'varying';
        params['FRAG_IN'] = 'varying';
        params['FRAG_COLOR'] = 'gl_FragColor';
    } else if (this.m_glslVersion === '300 es' || this.m_glslVersion === '310 es' || this.m_glslVersion === '330 es') {
        var ext = null;

        // if (isCubeArray && glu::glslVersionIsES(m_glslVersion))
        //     ext = "gl.EXT_texture_cube_map_array";
        // else if (isBuffer && glu::glslVersionIsES(m_glslVersion))
        //     ext = "gl.EXT_texture_buffer";

        var extension = '';
        if (ext)
            extension = '\n#extension ' + ext + ' : require';

        params['FRAG_HEADER'] = '#version ' + this.m_glslVersion + extension + '\nlayout(location = 0) out mediump vec4 dEQP_FragColor;\n';
        params['VTX_HEADER'] = '#version ' + this.m_glslVersion + '\n';
        params['VTX_IN'] = 'in';
        params['VTX_OUT'] = 'out';
        params['FRAG_IN'] = 'in';
        params['FRAG_COLOR'] = 'dEQP_FragColor';
    } else
        throw new Error('Unsupported version: ' + this.m_glslVersion);

    params['PRECISION'] = gluShaderUtil.getPrecisionName(this.m_texCoordPrecision);

    if (isCubeArray)
        params['TEXCOORD_TYPE'] = 'vec4';
    else if (isCube || (is2D && isArray) || is3D)
        params['TEXCOORD_TYPE'] = 'vec3';
    else if ((is1D && isArray) || is2D)
        params['TEXCOORD_TYPE'] = 'vec2';
    else if (is1D)
        params['TEXCOORD_TYPE'] = 'float';
    else
        DE_ASSERT(false);

    var sampler = null;
    var lookup = null;

    if (this.m_glslVersion === '300 es' || this.m_glslVersion === '310 es' || this.m_glslVersion === '330 es') {
        switch (program) {
            case glsTextureTestUtil.programType.PROGRAM_2D_FLOAT: sampler = 'sampler2D'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_INT: sampler = 'isampler2D'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_UINT: sampler = 'usampler2D'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_SHADOW: sampler = 'sampler2DShadow'; lookup = 'vec4(texture(u_sampler, vec3(v_texCoord, u_ref)), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_FLOAT_BIAS: sampler = 'sampler2D'; lookup = 'texture(u_sampler, v_texCoord, u_bias)'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_INT_BIAS: sampler = 'isampler2D'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_UINT_BIAS: sampler = 'usampler2D'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_SHADOW_BIAS: sampler = 'sampler2DShadow'; lookup = 'vec4(texture(u_sampler, vec3(v_texCoord, u_ref), u_bias), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_FLOAT: sampler = 'sampler1D'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_INT: sampler = 'isampler1D'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_UINT: sampler = 'usampler1D'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_SHADOW: sampler = 'sampler1DShadow'; lookup = 'vec4(texture(u_sampler, vec3(v_texCoord, u_ref)), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_FLOAT_BIAS: sampler = 'sampler1D'; lookup = 'texture(u_sampler, v_texCoord, u_bias)'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_INT_BIAS: sampler = 'isampler1D'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_UINT_BIAS: sampler = 'usampler1D'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_SHADOW_BIAS: sampler = 'sampler1DShadow'; lookup = 'vec4(texture(u_sampler, vec3(v_texCoord, u_ref), u_bias), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT: sampler = 'samplerCube'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_INT: sampler = 'isamplerCube'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_UINT: sampler = 'usamplerCube'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_SHADOW: sampler = 'samplerCubeShadow'; lookup = 'vec4(texture(u_sampler, vec4(v_texCoord, u_ref)), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT_BIAS: sampler = 'samplerCube'; lookup = 'texture(u_sampler, v_texCoord, u_bias)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_INT_BIAS: sampler = 'isamplerCube'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_UINT_BIAS: sampler = 'usamplerCube'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_SHADOW_BIAS: sampler = 'samplerCubeShadow'; lookup = 'vec4(texture(u_sampler, vec4(v_texCoord, u_ref), u_bias), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_FLOAT: sampler = 'sampler2DArray'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_INT: sampler = 'isampler2DArray'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_UINT: sampler = 'usampler2DArray'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_SHADOW: sampler = 'sampler2DArrayShadow'; lookup = 'vec4(texture(u_sampler, vec4(v_texCoord, u_ref)), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_3D_FLOAT: sampler = 'sampler3D'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_3D_INT: sampler = 'isampler3D'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_3D_UINT: sampler = ' usampler3D'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_3D_FLOAT_BIAS: sampler = 'sampler3D'; lookup = 'texture(u_sampler, v_texCoord, u_bias)'; break;
            case glsTextureTestUtil.programType.PROGRAM_3D_INT_BIAS: sampler = 'isampler3D'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_3D_UINT_BIAS: sampler = ' usampler3D'; lookup = 'vec4(texture(u_sampler, v_texCoord, u_bias))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_FLOAT: sampler = 'samplerCubeArray'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_INT: sampler = 'isamplerCubeArray'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_UINT: sampler = 'usamplerCubeArray'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_SHADOW: sampler = 'samplerCubeArrayShadow'; lookup = 'vec4(texture(u_sampler, vec4(v_texCoord, u_ref)), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_FLOAT: sampler = 'sampler1DArray'; lookup = 'texture(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_INT: sampler = 'isampler1DArray'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_UINT: sampler = 'usampler1DArray'; lookup = 'vec4(texture(u_sampler, v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_SHADOW: sampler = 'sampler1DArrayShadow'; lookup = 'vec4(texture(u_sampler, vec4(v_texCoord, u_ref)), 0.0, 0.0, 1.0)'; break;
            case glsTextureTestUtil.programType.PROGRAM_BUFFER_FLOAT: sampler = 'samplerBuffer'; lookup = 'texelFetch(u_sampler, int(v_texCoord))'; break;
            case glsTextureTestUtil.programType.PROGRAM_BUFFER_INT: sampler = 'isamplerBuffer'; lookup = 'vec4(texelFetch(u_sampler, int(v_texCoord)))'; break;
            case glsTextureTestUtil.programType.PROGRAM_BUFFER_UINT: sampler = 'usamplerBuffer'; lookup = 'vec4(texelFetch(u_sampler, int(v_texCoord)))'; break;
            default:
                DE_ASSERT(false);
        }
    } else if (this.m_glslVersion === '100 es') {
        sampler = isCube ? 'samplerCube' : 'sampler2D';

        switch (program) {
            case glsTextureTestUtil.programType.PROGRAM_2D_FLOAT: lookup = 'texture2D(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_2D_FLOAT_BIAS: lookup = 'texture2D(u_sampler, v_texCoord, u_bias)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT: lookup = 'textureCube(u_sampler, v_texCoord)'; break;
            case glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT_BIAS: lookup = 'textureCube(u_sampler, v_texCoord, u_bias)'; break;
            default:
                DE_ASSERT(false);
        }
    } else
        DE_ASSERT(!'Unsupported version');

    params['SAMPLER_TYPE'] = sampler;
    params['LOOKUP'] = lookup;

    var vertSrc = tcuStringTemplate.specialize(vertShaderTemplate, params);
    var fragSrc = tcuStringTemplate.specialize(fragShaderTemplate, params);
    var progObj = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertSrc, fragSrc));
    // if (!progObj.isOk()) {
    //     // log << *progObj;
    //     testFailedOptions("Failed to create shader", true);
    // }

    // try
    // {
    //     m_programs[program] = progObj;
    // }
    // catch (...)
    // {
    //     delete progObj;
    //     throw;
    // }

    return progObj;
};

// public:
//                                             glsTextureTestUtil.ProgramLibrary (const glu::RenderContext& context, tcu::TestContext& testCtx, glu::GLSLVersion glslVersion, glu::Precision texCoordPrecision);
//                                             ~glsTextureTestUtil.ProgramLibrary (void);

//     glu::ShaderProgram*                        getProgram (Program program);
//     void clear (void);

// private:
//                                             glsTextureTestUtil.ProgramLibrary (const glsTextureTestUtil.ProgramLibrary& other);
//     glsTextureTestUtil.ProgramLibrary& operator= (const glsTextureTestUtil.ProgramLibrary& other);

//     const glu::RenderContext& m_context;
//     tcu::TestContext& m_testCtx;
//     glu::GLSLVersion m_glslVersion;
//     glu::Precision m_texCoordPrecision;
//     std::map<Program, glu::ShaderProgram*> m_programs;
// };

/**
 * @constructor
 * @param {string} version GL version
 * @param {gluShaderUtil.precision} precision
 */
glsTextureTestUtil.TextureRenderer = function(version, precision) {
    this.m_programLibrary = new glsTextureTestUtil.ProgramLibrary(version, precision);
};

/**
 * @param {tcuPixelFormat.PixelFormat} format
 * @return {Array<boolean>}
 */
glsTextureTestUtil.getCompareMask = function(format) {
    return [
        format.redBits > 0,
        format.greenBits > 0,
        format.blueBits > 0,
        format.alphaBits > 0
    ];
};

/**
 * @param {tcuPixelFormat.PixelFormat} format
 * @return {Array<number>}
 */
glsTextureTestUtil.getBitsVec = function(format) {
    return [
        format.redBits,
        format.greenBits,
        format.blueBits,
        format.alphaBits
    ];
};

/**
 * @param {number} texUnit
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.RenderParams} params
 */
glsTextureTestUtil.TextureRenderer.prototype.renderQuad = function(texUnit, texCoord, params) {
    var wCoord = params.flags.projected ? params.w : [1, 1, 1, 1];
    var useBias = params.flags.use_bias;
    var logUniforms = params.flags.log_uniforms;

    // Render quad with texture.
    var position = [
        -1 * wCoord[0], -1 * wCoord[0], 0, wCoord[0],
        -1 * wCoord[1], +1 * wCoord[1], 0, wCoord[1],
        +1 * wCoord[2], -1 * wCoord[2], 0, wCoord[2],
        +1 * wCoord[3], +1 * wCoord[3], 0, wCoord[3]
    ];
    /** @const */ var indices = [0, 1, 2, 2, 1, 3];

    /** @type {?glsTextureTestUtil.programType} */ var progSpec = null;
    var numComps = 0;
    if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_2D) {
        numComps = 2;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_2D_FLOAT_BIAS : glsTextureTestUtil.programType.PROGRAM_2D_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_2D_INT_BIAS : glsTextureTestUtil.programType.PROGRAM_2D_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_2D_UINT_BIAS : glsTextureTestUtil.programType.PROGRAM_2D_UINT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_2D_SHADOW_BIAS : glsTextureTestUtil.programType.PROGRAM_2D_SHADOW; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_1D) {
        numComps = 1;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_1D_FLOAT_BIAS : glsTextureTestUtil.programType.PROGRAM_1D_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_1D_INT_BIAS : glsTextureTestUtil.programType.PROGRAM_1D_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_1D_UINT_BIAS : glsTextureTestUtil.programType.PROGRAM_1D_UINT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_1D_SHADOW_BIAS : glsTextureTestUtil.programType.PROGRAM_1D_SHADOW; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_CUBE) {
        numComps = 3;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT_BIAS : glsTextureTestUtil.programType.PROGRAM_CUBE_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_CUBE_INT_BIAS : glsTextureTestUtil.programType.PROGRAM_CUBE_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_CUBE_UINT_BIAS : glsTextureTestUtil.programType.PROGRAM_CUBE_UINT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_CUBE_SHADOW_BIAS : glsTextureTestUtil.programType.PROGRAM_CUBE_SHADOW; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_3D) {
        numComps = 3;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_3D_FLOAT_BIAS : glsTextureTestUtil.programType.PROGRAM_3D_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_3D_INT_BIAS : glsTextureTestUtil.programType.PROGRAM_3D_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = useBias ? glsTextureTestUtil.programType.PROGRAM_3D_UINT_BIAS : glsTextureTestUtil.programType.PROGRAM_3D_UINT; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_2D_ARRAY) {
        DE_ASSERT(!useBias); // \todo [2012-02-17 pyry] Support bias.

        numComps = 3;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_UINT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW: progSpec = glsTextureTestUtil.programType.PROGRAM_2D_ARRAY_SHADOW; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_CUBE_ARRAY) {
        DE_ASSERT(!useBias);

        numComps = 4;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_UINT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW: progSpec = glsTextureTestUtil.programType.PROGRAM_CUBE_ARRAY_SHADOW; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_1D_ARRAY) {
        DE_ASSERT(!useBias); // \todo [2012-02-17 pyry] Support bias.

        numComps = 2;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FLOAT: progSpec = glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_INT: progSpec = glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_UINT: progSpec = glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_UINT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW: progSpec = glsTextureTestUtil.programType.PROGRAM_1D_ARRAY_SHADOW; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else if (params.texType == glsTextureTestUtil.textureType.TEXTURETYPE_BUFFER) {
        numComps = 1;

        switch (params.samplerType) {
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FETCH_FLOAT: progSpec = glsTextureTestUtil.programType.PROGRAM_BUFFER_FLOAT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FETCH_INT: progSpec = glsTextureTestUtil.programType.PROGRAM_BUFFER_INT; break;
            case glsTextureTestUtil.samplerType.SAMPLERTYPE_FETCH_UINT: progSpec = glsTextureTestUtil.programType.PROGRAM_BUFFER_UINT; break;
            default: throw new Error('Unrecognized sampler type:' + params.samplerType);
        }
    } else
       throw new Error('Unrecognized texture type:' + params.texType);

    if (progSpec === null)
        throw new Error('Could not find program specification');

    var program = this.m_programLibrary.getProgram(progSpec);

    // \todo [2012-09-26 pyry] Move to glsTextureTestUtil.ProgramLibrary and log unique programs only(?)
    /* TODO: Port logging
    if (params.flags.log_programs)
        log << *program;
    */

    // Program and uniforms.
    var prog = program.getProgram();
    gl.useProgram(prog);

    var loc = gl.getUniformLocation(prog, 'u_sampler');
    gl.uniform1i(loc, texUnit);
    // if (logUniforms)
    //     log << TestLog::Message << "u_sampler = " << texUnit << TestLog::EndMessage;

    if (useBias) {
        gl.uniform1f(gl.getUniformLocation(prog, 'u_bias'), params.bias);
        // if (logUniforms)
        //     log << TestLog::Message << "u_bias = " << params.bias << TestLog::EndMessage;
    }

    if (params.samplerType == glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW) {
        gl.uniform1f(gl.getUniformLocation(prog, 'u_ref'), params.ref);
        // if (logUniforms)
        //     log << TestLog::Message << "u_ref = " << params.ref << TestLog::EndMessage;
    }

    gl.uniform4fv(gl.getUniformLocation(prog, 'u_colorScale'), params.colorScale);
    gl.uniform4fv(gl.getUniformLocation(prog, 'u_colorBias'), params.colorBias);

    // if (logUniforms)
    // {
    //     log << TestLog::Message << "u_colorScale = " << params.colorScale << TestLog::EndMessage;
    //     log << TestLog::Message << "u_colorBias = " << params.colorBias << TestLog::EndMessage;
    // }
    var vertexArrays = [];

    var posLoc = gl.getAttribLocation(prog, 'a_position');
    if (posLoc === -1) {
        testFailedOptions("no location found for attribute 'a_position'", true);
    }
    var texLoc = gl.getAttribLocation(prog, 'a_texCoord');
    if (texLoc === -1) {
        testFailedOptions("no location found for attribute 'a_texCoord'", true);
    }

    vertexArrays.push(new gluDrawUtil.VertexArrayBinding(gl.FLOAT, posLoc, 4, 4, position));
    vertexArrays.push(new gluDrawUtil.VertexArrayBinding(gl.FLOAT, texLoc, numComps, 4, texCoord));
    gluDrawUtil.draw(gl, prog, vertexArrays, gluDrawUtil.triangles(indices));
};

// public:
//                                 glsTextureTestUtil.TextureRenderer (const glu::RenderContext& context, tcu::TestContext& testCtx, glu::GLSLVersion glslVersion, glu::Precision texCoordPrecision);
//                                 ~glsTextureTestUtil.TextureRenderer (void);

//     void clear (void); //!< Frees allocated resources. Destructor will call clear() as well.

//     void renderQuad (int texUnit, const float* texCoord, TextureType texType);
//     void renderQuad (int texUnit, const float* texCoord, const glsTextureTestUtil.RenderParams& params);

// private:
//                                 glsTextureTestUtil.TextureRenderer (const glsTextureTestUtil.TextureRenderer& other);
//     glsTextureTestUtil.TextureRenderer& operator= (const glsTextureTestUtil.TextureRenderer& other);

//     const glu::RenderContext& m_renderCtx;
//     tcu::TestContext& m_testCtx;
//     glsTextureTestUtil.ProgramLibrary m_programLibrary;
// };

/**
 * @constructor
 * @param {tcuSurface.Surface} surface
 * @param {tcuPixelFormat.PixelFormat=} colorFmt
 * @param {number=} x
 * @param {number=} y
 * @param {number=} width
 * @param {number=} height
 */
glsTextureTestUtil.SurfaceAccess = function(surface, colorFmt, x, y, width, height) {
    this.m_surface = surface;
    this.colorMask = undefined; /*TODO*/
    this.m_x = x || 0;
    this.m_y = y || 0;
    this.m_width = width || surface.getWidth();
    this.m_height = height || surface.getHeight();
};

/** @return {number} */
glsTextureTestUtil.SurfaceAccess.prototype.getWidth = function() { return this.m_width; };
/** @return {number} */
glsTextureTestUtil.SurfaceAccess.prototype.getHeight = function() { return this.m_height; };

/**
 * @param {Array<number>} color
 * @param {number} x
 * @param {number} y
 */
glsTextureTestUtil.SurfaceAccess.prototype.setPixel = function(color, x, y) {
    /* TODO: Apply color mask */
    var c = color;
    for (var i = 0; i < c.length; i++)
        c[i] = deMath.clamp(Math.round(color[i] * 255), 0, 255);
    this.m_surface.setPixel(x, y, c);
};

/**
 * @param {glsTextureTestUtil.lodMode} mode
 * @param {number} dudx
 * @param {number} dvdx
 * @param {number} dwdx
 * @param {number} dudy
 * @param {number} dvdy
 * @param {number} dwdy
 * @return {number}
 */
glsTextureTestUtil.computeLodFromDerivates3D = function(mode, dudx, dvdx, dwdx, dudy, dvdy, dwdy) {
    var p = 0;
    switch (mode) {
        case glsTextureTestUtil.lodMode.EXACT:
            p = Math.max(Math.sqrt(dudx * dudx + dvdx * dvdx + dwdx * dwdx), Math.sqrt(dudy * dudy + dvdy * dvdy + dwdy * dwdy));
            break;

        case glsTextureTestUtil.lodMode.MIN_BOUND:
        case glsTextureTestUtil.lodMode.MAX_BOUND: {
            var mu = Math.max(Math.abs(dudx), Math.abs(dudy));
            var mv = Math.max(Math.abs(dvdx), Math.abs(dvdy));
            var mw = Math.max(Math.abs(dwdx), Math.abs(dwdy));

            p = (mode == glsTextureTestUtil.lodMode.MIN_BOUND) ? Math.max(mu, mv, mw) : mu + mv + mw;
            break;
        }

        default:
            DE_ASSERT(false);
    }

    // Native dEQP uses 32-bit numbers. So here 64-bit floating numbers should be transformed into 32-bit ones to ensure the correctness of the result.
    return deMath.toFloat32(Math.log(p)) * deMath.INV_LOG_2_FLOAT32;
};

/**
 * @param {glsTextureTestUtil.lodMode} mode
 * @param {Array<number>} dstSize
 * @param {Array<number>} srcSize
 * @param {Array<number>} sq
 * @param {Array<number>} tq
 * @param {Array<number>=} rq
 * @return {number}
 */
glsTextureTestUtil.computeNonProjectedTriLod = function(mode, dstSize, srcSize, sq, tq, rq) {
    var dux = (sq[2] - sq[0]) * srcSize[0];
    var duy = (sq[1] - sq[0]) * srcSize[0];
    var dvx = (tq[2] - tq[0]) * srcSize[1];
    var dvy = (tq[1] - tq[0]) * srcSize[1];
    var dwx = 0;
    var dwy = 0;
    if (rq) {
        dwx = (rq[2] - rq[0]) * srcSize[2];
        dwy = (rq[1] - rq[0]) * srcSize[2];
    }
    var dx = dstSize[0];
    var dy = dstSize[1];

    return glsTextureTestUtil.computeLodFromDerivates3D(mode, dux / dx, dvx / dx, dwx / dx, duy / dy, dvy / dy, dwy / dy);
};

/**
 * @param {Array<number>} v
 * @param {number} x
 * @param {number} y
 * @return {number}
 */
glsTextureTestUtil.triangleInterpolate = function(v, x, y) {
    return v[0] + (v[2] - v[0]) * x + (v[1] - v[0]) * y;
};

/**
 * @param {Array<number>} s
 * @param {Array<number>} w
 * @param {number} wx
 * @param {number} width
 * @param {number} ny
 * @return {number}
 */
glsTextureTestUtil.triDerivateX = function(s, w, wx, width, ny) {
    var d = w[1] * w[2] * (width * (ny - 1) + wx) - w[0] * (w[2] * width * ny + w[1] * wx);
    return (w[0] * w[1] * w[2] * width * (w[1] * (s[0] - s[2]) * (ny - 1) + ny * (w[2] * (s[1] - s[0]) + w[0] * (s[2] - s[1])))) / (d * d);
};

/**
 * @param {Array<number>} s
 * @param {Array<number>} w
 * @param {number} wy
 * @param {number} height
 * @param {number} nx
 * @return {number}
 */
glsTextureTestUtil.triDerivateY = function(s, w, wy, height, nx) {
    var d = w[1] * w[2] * (height * (nx - 1) + wy) - w[0] * (w[1] * height * nx + w[2] * wy);
    return (w[0] * w[1] * w[2] * height * (w[2] * (s[0] - s[1]) * (nx - 1) + nx * (w[0] * (s[1] - s[2]) + w[1] * (s[2] - s[0])))) / (d * d);
};

/**
 * @param {(tcuTexture.Texture2DView|tcuTexture.Texture2DArrayView|tcuTexture.TextureCubeView)} src
 * @param {glsTextureTestUtil.ReferenceParams} params
 * @param {Array<number>} texCoord Texture coordinates
 * @param {number} lod
 * @return {Array<number>} sample
 */
glsTextureTestUtil.execSample = function(src, params, texCoord, lod) {
    if (params.samplerType == glsTextureTestUtil.samplerType.SAMPLERTYPE_SHADOW)
        return [src.sampleCompare(params.sampler, params.ref, texCoord, lod), 0, 0, 1];
    else
        return src.sample(params.sampler, texCoord, lod);
};

/**
 * @param {Array<number>} pixel
 * @param {Array<number>} scale
 * @param {Array<number>} bias
 */
glsTextureTestUtil.applyScaleAndBias = function(pixel, scale, bias) {
    pixel[0] = pixel[0] * scale[0] + bias[0];
    pixel[1] = pixel[1] * scale[1] + bias[1];
    pixel[2] = pixel[2] * scale[2] + bias[2];
    pixel[3] = pixel[3] * scale[3] + bias[3];
};

/**
 * @param {Array<number>} pixel
 * @param {Array<number>} scale
 * @param {Array<number>} bias
 */
glsTextureTestUtil.deapplyScaleAndBias = function(pixel, scale, bias) {
    pixel[0] = (pixel[0] - bias[0]) / scale[0];
    pixel[1] = (pixel[1] - bias[1]) / scale[1];
    pixel[2] = (pixel[2] - bias[2]) / scale[2];
    pixel[3] = (pixel[3] - bias[3]) / scale[3];
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture2DView} src
 * @param {Array<number>} sq
 * @param {Array<number>} tq
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTextureProjected2D = function(dst, src, sq, tq, params) {
    /** @type {number} */ var lodBias = params.flags.use_bias ? params.bias : 0.0;
    /** @type {number} */ var dstW = dst.getWidth();
    /** @type {number} */ var dstH = dst.getHeight();

    /** @type {Array<number>} */ var uq = deMath.scale(sq, src.getWidth());
    /** @type {Array<number>} */ var vq = deMath.scale(tq, src.getHeight());

    /** @type {Array<Array<number>>} */ var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triU = [deMath.swizzle(uq, [0, 1, 2]), deMath.swizzle(uq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triV = [deMath.swizzle(vq, [0, 1, 2]), deMath.swizzle(vq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triW = [deMath.swizzle(params.w, [0, 1, 2]), deMath.swizzle(params.w, [3, 2, 1])];

    for (var py = 0; py < dst.getHeight(); py++) {
        for (var px = 0; px < dst.getWidth(); px++) {
            /** @type {number} */ var wx = px + 0.5;
            /** @type {number} */ var wy = py + 0.5;
            /** @type {number} */ var nx = wx / dstW;
            /** @type {number} */ var ny = wy / dstH;

            /** @type {number} */ var triNdx = nx + ny >= 1.0 ? 1 : 0;
            /** @type {number} */ var triWx = triNdx ? dstW - wx : wx;
            /** @type {number} */ var triWy = triNdx ? dstH - wy : wy;
            /** @type {number} */ var triNx = triNdx ? 1.0 - nx : nx;
            /** @type {number} */ var triNy = triNdx ? 1.0 - ny : ny;

            /** @type {number} */ var s = glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], triNx, triNy);
            /** @type {number} */ var t = glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], triNx, triNy);
            /** @type {number} */ var lod = glsTextureTestUtil.computeProjectedTriLod2D(params.lodMode, triU[triNdx], triV[triNdx], triW[triNdx], triWx, triWy, dst.getWidth(), dst.getHeight()) + lodBias;

            var pixel = glsTextureTestUtil.execSample(src, params, [s, t], lod);
            glsTextureTestUtil.applyScaleAndBias(pixel, params.colorScale, params.colorBias);
            dst.setPixel(pixel, px, py);
        }
    }
};

/**
 * @param {glsTextureTestUtil.lodMode} mode
 * @param {Array<number>} u
 * @param {Array<number>} v
 * @param {Array<number>} projection
 * @param {number} wx
 * @param {number} wy
 * @param {number} width
 * @param {number} height
 * @return {number}
 */
glsTextureTestUtil.computeProjectedTriLod2D = function(mode, u, v, projection, wx, wy, width, height) {
    // Exact derivatives.
    /** @type {number} */ var dudx = glsTextureTestUtil.triDerivateX(u, projection, wx, width, wy / height);
    /** @type {number} */ var dvdx = glsTextureTestUtil.triDerivateX(v, projection, wx, width, wy / height);
    /** @type {number} */ var dudy = glsTextureTestUtil.triDerivateY(u, projection, wy, height, wx / width);
    /** @type {number} */ var dvdy = glsTextureTestUtil.triDerivateY(v, projection, wy, height, wx / width);

    return glsTextureTestUtil.computeLodFromDerivates2D(mode, dudx, dvdx, dudy, dvdy);
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture2DView} src
 * @param {Array<number>} sq
 * @param {Array<number>} tq
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTextureNonProjected2D = function(dst, src, sq, tq, params) {
    var lodBias = params.flags.use_bias ? params.bias : 0;

    var dstSize = [dst.getWidth(), dst.getHeight()];
    var srcSize = [src.getWidth(), src.getHeight()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triLod = [deMath.clamp((glsTextureTestUtil.computeNonProjectedTriLod(params.lodMode, dstSize, srcSize, triS[0], triT[0]) + lodBias), params.minLod, params.maxLod),
                    deMath.clamp((glsTextureTestUtil.computeNonProjectedTriLod(params.lodMode, dstSize, srcSize, triS[1], triT[1]) + lodBias), params.minLod, params.maxLod)];


    for (var y = 0; y < dst.getHeight(); y++) {
        for (var x = 0; x < dst.getWidth(); x++) {
            var yf = (y + 0.5) / dst.getHeight();
            var xf = (x + 0.5) / dst.getWidth();

            var triNdx = xf + yf >= 1 ? 1 : 0; // Top left fill rule.
            var triX = triNdx ? 1 - xf : xf;
            var triY = triNdx ? 1 - yf : yf;

            var s = glsTextureTestUtil.triangleInterpolate(triS[triNdx], triX, triY);
            var t = glsTextureTestUtil.triangleInterpolate(triT[triNdx], triX, triY);
            var lod = triLod[triNdx];

            var pixel = glsTextureTestUtil.execSample(src, params, [s, t], lod);
            glsTextureTestUtil.applyScaleAndBias(pixel, params.colorScale, params.colorBias);
            dst.setPixel(pixel, x, y);
        }
    }
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture2DArrayView} src
 * @param {Array<number>} sq
 * @param {Array<number>} tq
 * @param {Array<number>} rq
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTextureNonProjected2DArray = function(dst, src, sq, tq, rq, params) {
    var lodBias = (params.flags.use_bias) ? params.bias : 0;

    var dstSize = [dst.getWidth(), dst.getHeight()];
    var srcSize = [src.getWidth(), src.getHeight()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triLod = [glsTextureTestUtil.computeNonProjectedTriLod(params.lodMode, dstSize, srcSize, triS[0], triT[0]) + lodBias,
                                glsTextureTestUtil.computeNonProjectedTriLod(params.lodMode, dstSize, srcSize, triS[1], triT[1]) + lodBias];

    for (var y = 0; y < dst.getHeight(); y++) {
        for (var x = 0; x < dst.getWidth(); x++) {
            var yf = (y + 0.5) / dst.getHeight();
            var xf = (x + 0.5) / dst.getWidth();

            var triNdx = xf + yf >= 1 ? 1 : 0; // Top left fill rule.
            var triX = triNdx ? 1 - xf : xf;
            var triY = triNdx ? 1 - yf : yf;

            var s = glsTextureTestUtil.triangleInterpolate(triS[triNdx], triX, triY);
            var t = glsTextureTestUtil.triangleInterpolate(triT[triNdx], triX, triY);
            var r = glsTextureTestUtil.triangleInterpolate(triR[triNdx], triX, triY);
            var lod = triLod[triNdx];

            var pixel = glsTextureTestUtil.execSample(src, params, [s, t, r], lod);
            glsTextureTestUtil.applyScaleAndBias(pixel, params.colorScale, params.colorBias);
            dst.setPixel(pixel, x, y);
        }
    }
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture2DView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTexture2D = function(dst, src, texCoord, params) {
    var view = src.getSubView(params.baseLevel, params.maxLevel);
    var sq = [texCoord[0 + 0], texCoord[2 + 0], texCoord[4 + 0], texCoord[6 + 0]];
    var tq = [texCoord[0 + 1], texCoord[2 + 1], texCoord[4 + 1], texCoord[6 + 1]];

    if (params.flags.projected)
        glsTextureTestUtil.sampleTextureProjected2D(dst, view, sq, tq, params);
    else
        glsTextureTestUtil.sampleTextureNonProjected2D(dst, view, sq, tq, params);
};

/**
 * @param {glsTextureTestUtil.lodMode} mode
 * @param {number} dudx
 * @param {number} dvdx
 * @param {number} dudy
 * @param {number} dvdy
 * @return {number}
 */
glsTextureTestUtil.computeLodFromDerivates2D = function(mode, dudx, dvdx, dudy, dvdy) {
    var p = 0;
    switch (mode) {
        case glsTextureTestUtil.lodMode.EXACT:
            p = Math.max(Math.sqrt(dudx * dudx + dvdx * dvdx), Math.sqrt(dudy * dudy + dvdy * dvdy));
            break;

        case glsTextureTestUtil.lodMode.MIN_BOUND:
        case glsTextureTestUtil.lodMode.MAX_BOUND: {
            var mu = Math.max(Math.abs(dudx), Math.abs(dudy));
            var mv = Math.max(Math.abs(dvdx), Math.abs(dvdy));

            p = (mode == glsTextureTestUtil.lodMode.MIN_BOUND) ? Math.max(mu, mv) : mu + mv;
            break;
        }

        default:
            throw new Error('Unrecognized mode:' + mode);
    }

    // Native dEQP uses 32-bit numbers. So here 64-bit floating numbers should be transformed into 32-bit ones to ensure the correctness of the result.
    return deMath.toFloat32(Math.log(p)) * deMath.INV_LOG_2_FLOAT32;
};

/**
 * @param {glsTextureTestUtil.lodMode} lodModeParm
 * @param {Array<number>} coord
 * @param {Array<number>} coordDx
 * @param {Array<number>} coordDy
 * @param {number} faceSize
 * @return {number}
 */
glsTextureTestUtil.computeCubeLodFromDerivates = function(lodModeParm, coord, coordDx, coordDy, faceSize) {
    var face = tcuTexture.selectCubeFace(coord);
    var maNdx = 0;
    var sNdx = 0;
    var tNdx = 0;

    // \note Derivate signs don't matter when computing lod
    switch (face) {
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X:
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X: maNdx = 0; sNdx = 2; tNdx = 1; break;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y:
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y: maNdx = 1; sNdx = 0; tNdx = 2; break;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z:
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z: maNdx = 2; sNdx = 0; tNdx = 1; break;
        default:
            throw new Error('Unrecognized face ' + face);
    } {
        var sc = coord[sNdx];
        var tc = coord[tNdx];
        var ma = Math.abs(coord[maNdx]);
        var scdx = coordDx[sNdx];
        var tcdx = coordDx[tNdx];
        var madx = Math.abs(coordDx[maNdx]);
        var scdy = coordDy[sNdx];
        var tcdy = coordDy[tNdx];
        var mady = Math.abs(coordDy[maNdx]);
        var dudx = faceSize * 0.5 * (scdx * ma - sc * madx) / (ma * ma);
        var dvdx = faceSize * 0.5 * (tcdx * ma - tc * madx) / (ma * ma);
        var dudy = faceSize * 0.5 * (scdy * ma - sc * mady) / (ma * ma);
        var dvdy = faceSize * 0.5 * (tcdy * ma - tc * mady) / (ma * ma);
        return glsTextureTestUtil.computeLodFromDerivates2D(lodModeParm, dudx, dvdx, dudy, dvdy);
    }
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.TextureCubeView} src
 * @param {Array<number>} sq
 * @param {Array<number>} tq
 * @param {Array<number>} rq
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTextureCube_str = function(dst, src, sq, tq, rq, params) {
    var dstSize = [dst.getWidth(), dst.getHeight()];
    var dstW = dstSize[0];
    var dstH = dstSize[1];
    var srcSize = src.getSize();

    // Coordinates per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triW = [deMath.swizzle(params.w, [0, 1, 2]), deMath.swizzle(params.w, [3, 2, 1])];

    var lodBias = (params.flags.use_bias ? params.bias : 0);

    for (var py = 0; py < dst.getHeight(); py++) {
        for (var px = 0; px < dst.getWidth(); px++) {
            var wx = px + 0.5;
            var wy = py + 0.5;
            var nx = wx / dstW;
            var ny = wy / dstH;
            var triNdx = nx + ny >= 1 ? 1 : 0;
            var triNx = triNdx ? 1 - nx : nx;
            var triNy = triNdx ? 1 - ny : ny;

            var coord = [glsTextureTestUtil.triangleInterpolate(triS[triNdx], triNx, triNy),
                                         glsTextureTestUtil.triangleInterpolate(triT[triNdx], triNx, triNy),
                                         glsTextureTestUtil.triangleInterpolate(triR[triNdx], triNx, triNy)];
            var coordDx = [glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wx, dstW, triNy),
                                         glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wx, dstW, triNy),
                                         glsTextureTestUtil.triDerivateX(triR[triNdx], triW[triNdx], wx, dstW, triNy)];
            var coordDy = [glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wy, dstH, triNx),
                                         glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wy, dstH, triNx),
                                         glsTextureTestUtil.triDerivateY(triR[triNdx], triW[triNdx], wy, dstH, triNx)];

            var lod = deMath.clamp((glsTextureTestUtil.computeCubeLodFromDerivates(params.lodMode, coord, coordDx, coordDy, srcSize) + lodBias), params.minLod, params.maxLod);

            var pixel = glsTextureTestUtil.execSample(src, params, coord, lod);
            glsTextureTestUtil.applyScaleAndBias(pixel, params.colorScale, params.colorBias);
            dst.setPixel(pixel, px, py);
        }
    }
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.TextureCubeView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTextureCube = function(dst, src, texCoord, params) {
    /*const tcu::TextureCubeView*/ var view = src.getSubView(params.baseLevel, params.maxLevel);
    var sq = [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq = [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq = [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    glsTextureTestUtil.sampleTextureCube_str(dst, view, sq, tq, rq, params);
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture2DArrayView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTexture2DArray = function(dst, src, texCoord, params) {
    var sq = [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq = [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq = [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    DE_ASSERT(!(params.flags.projected)); // \todo [2012-02-17 pyry] Support projected lookups.
    glsTextureTestUtil.sampleTextureNonProjected2DArray(dst, src, sq, tq, rq, params);
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture3DView} src
 * @param {Array<number>} sq
 * @param {Array<number>} tq
 * @param {Array<number>} rq
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTextureNonProjected3D = function(dst, src, sq, tq, rq, params) {
    var lodBias = params.flags.use_bias ? params.bias : 0;

    var dstSize = [dst.getWidth(), dst.getHeight()];
    var srcSize = [src.getWidth(), src.getHeight(), src.getDepth()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triLod = [deMath.clamp((glsTextureTestUtil.computeNonProjectedTriLod(params.lodMode, dstSize, srcSize, triS[0], triT[0], triR[0]) + lodBias), params.minLod, params.maxLod),
                  deMath.clamp((glsTextureTestUtil.computeNonProjectedTriLod(params.lodMode, dstSize, srcSize, triS[1], triT[1], triR[1]) + lodBias), params.minLod, params.maxLod)];

    for (var y = 0; y < dst.getHeight(); y++) {
        for (var x = 0; x < dst.getWidth(); x++) {
            var yf = (y + 0.5) / dst.getHeight();
            var xf = (x + 0.5) / dst.getWidth();

            var triNdx = xf + yf >= 1 ? 1 : 0; // Top left fill rule.
            var triX = triNdx ? 1 - xf : xf;
            var triY = triNdx ? 1 - yf : yf;

            var s = glsTextureTestUtil.triangleInterpolate(triS[triNdx], triX, triY);
            var t = glsTextureTestUtil.triangleInterpolate(triT[triNdx], triX, triY);
            var r = glsTextureTestUtil.triangleInterpolate(triR[triNdx], triX, triY);
            var lod = triLod[triNdx];

            var pixel = src.sample(params.sampler, [s, t, r], lod);
            glsTextureTestUtil.applyScaleAndBias(pixel, params.colorScale, params.colorBias);
            dst.setPixel(pixel, x, y);
        }
    }
};

/**
 * @param {glsTextureTestUtil.SurfaceAccess} dst
 * @param {tcuTexture.Texture3DView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} params
 */
glsTextureTestUtil.sampleTexture3D = function(dst, src, texCoord, params) {
    /*const tcu::TextureCubeView*/ var view = src.getSubView(params.baseLevel, params.maxLevel);
    var sq = [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq = [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq = [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    glsTextureTestUtil.sampleTextureNonProjected3D(dst, view, sq, tq, rq, params);
};

/**
 * @param {tcuSurface.Surface} reference
 * @param {tcuSurface.Surface} rendered
 * @param {Array<number>} threshold
 * @param {Array< Array<number> >} skipPixels
 *
 * @return {boolean}
 */
glsTextureTestUtil.compareImages = function(reference, rendered, threshold, skipPixels) {
    return tcuImageCompare.pixelThresholdCompare('Result', 'Image comparison result', reference, rendered, threshold, undefined /*tcu::COMPARE_LOG_RESULT*/, skipPixels);
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.Texture2DView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {tcuPixelFormat.PixelFormat} pixelFormat
 * @return {boolean}
 */
glsTextureTestUtil.verifyTexture2DResult = function(result, src, texCoord, sampleParams, lookupPrec, lodPrec, pixelFormat) {
    DE_ASSERT(deMath.equal(glsTextureTestUtil.getCompareMask(pixelFormat), lookupPrec.colorMask));
    /** @type {tcuSurface.Surface} */ var reference = new tcuSurface.Surface(result.getWidth(), result.getHeight());
    /** @type {tcuSurface.Surface} */ var errorMask = new tcuSurface.Surface(result.getWidth(), result.getHeight());
    /** @type {number} */ var numFailedPixels;

    /** @type {glsTextureTestUtil.SurfaceAccess} */ var surface = new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat);

    glsTextureTestUtil.sampleTexture2D(surface, src, texCoord, sampleParams);
    numFailedPixels = glsTextureTestUtil.computeTextureLookupDiff2D(result, reference.getAccess(), errorMask.getAccess(), src, texCoord, sampleParams, lookupPrec, lodPrec/*, testCtx.getWatchDog()*/);

    if (numFailedPixels > 0)
        tcuImageCompare.displayImages(result, reference.getAccess(), errorMask.getAccess());

    return numFailedPixels == 0;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.Texture2DView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexCompareVerifier.TexComparePrecision} comparePrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {Array<number>} nonShadowThreshold
 * @return {number}
 */
glsTextureTestUtil.computeTextureCompareDiff2D = function(result, reference, errorMask, src, texCoord, sampleParams, comparePrec, lodPrec, nonShadowThreshold) {
    DE_ASSERT(result.getWidth() == reference.getWidth() && result.getHeight() == reference.getHeight());
    DE_ASSERT(result.getWidth() == errorMask.getWidth() && result.getHeight() == errorMask.getHeight());

    var sq = [texCoord[0 + 0], texCoord[2 + 0], texCoord[4 + 0], texCoord[6 + 0]];
    var tq = [texCoord[0 + 1], texCoord[2 + 1], texCoord[4 + 1], texCoord[6 + 1]];

    var dstSize = [result.getWidth(), result.getHeight()];
    var dstW = dstSize[0];
    var dstH = dstSize[1];
    var srcSize = [src.getWidth(), src.getHeight()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triW = [deMath.swizzle(sampleParams.w, [0, 1, 2]), deMath.swizzle(sampleParams.w, [3, 2, 1])];

    var lodBias = sampleParams.flags.use_bias ? [sampleParams.bias, sampleParams.bias] : [0, 0];
    var numFailed = 0;

    var lodOffsets = [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1]
    ];

    /** @type {Array<number>} */ var green = [0, 255, 0, 255];
    errorMask.clear(green);

    /** @type {Array<number>} */ var red = [];
    for (var py = 0; py < result.getHeight(); py++) {
        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);

            if (!deMath.boolAll(deMath.lessThanEqual(deMath.absDiff(deMath.swizzle(refPix, [1, 2, 3]), deMath.swizzle(resPix, [1, 2, 3])), nonShadowThreshold))) {
                red = [255, 0, 0, 255];
                errorMask.setPixel(red, px, py);
                numFailed += 1;
                continue;
            }

            if (resPix[0] != refPix[0]) {
                var wx = px + 0.5;
                var wy = py + 0.5;
                var nx = wx / dstW;
                var ny = wy / dstH;

                var triNdx = nx + ny >= 1.0 ? 1 : 0;
                var triWx = triNdx ? dstW - wx : wx;
                var triWy = triNdx ? dstH - wy : wy;
                var triNx = triNdx ? 1.0 - nx : nx;
                var triNy = triNdx ? 1.0 - ny : ny;

                var coord = [glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], triNx, triNy),
                             glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], triNx, triNy)];
                var coordDx = deMath.multiply([glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wx, dstW, triNy),
                                            glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wx, dstW, triNy)], srcSize);
                var coordDy = deMath.multiply([glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wy, dstH, triNx),
                                            glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wy, dstH, triNx)], srcSize);

                var lodBounds = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDx[0], coordDx[1], coordDy[0], coordDy[1], lodPrec);

                // Compute lod bounds across lodOffsets range.
                for (var lodOffsNdx = 0; lodOffsNdx < lodOffsets.length; lodOffsNdx++) {
                    var wxo = triWx + lodOffsets[lodOffsNdx][0];
                    var wyo = triWy + lodOffsets[lodOffsNdx][1];
                    var nxo = wxo / dstW;
                    var nyo = wyo / dstH;

                    var coordO = [glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], nxo, nyo),
                                  glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], nxo, nyo)];
                    var coordDxo = deMath.multiply([glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wxo, dstW, nyo),
                                                 glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wxo, dstW, nyo)], srcSize);
                    var coordDyo = deMath.multiply([glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wyo, dstH, nxo),
                                                 glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wyo, dstH, nxo)], srcSize);
                    var lodO = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDxo[0], coordDxo[1], coordDyo[0], coordDyo[1], lodPrec);

                    lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                    lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                }

                var clampedLod = tcuTexLookupVerifier.clampLodBounds(deMath.add(lodBounds, lodBias), [sampleParams.minLod, sampleParams.maxLod], lodPrec);
                var isOk = tcuTexCompareVerifier.isTexCompareResultValid2D(src, sampleParams.sampler, comparePrec, coord, clampedLod, sampleParams.ref, resPix[0]);

                if (!isOk) {
                    red = [255, 0, 0, 255];
                    errorMask.setPixel(red, px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.Texture3DView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {tcuPixelFormat.PixelFormat} pixelFormat
 * @return {boolean}
 */
glsTextureTestUtil.verifyTexture3DResult = function(
    result, src, texCoord, sampleParams, lookupPrec, lodPrec, pixelFormat
) {
    /** @type {tcuSurface.Surface} */ var reference = new tcuSurface.Surface(result.getWidth(), result.getHeight());
    /** @type {tcuSurface.Surface} */ var errorMask = new tcuSurface.Surface(result.getWidth(), result.getHeight());
    var numFailedPixels = 0;

    assertMsgOptions(
        deMath.equal(glsTextureTestUtil.getCompareMask(pixelFormat), lookupPrec.colorMask),
        'Compare color masks do not match', false, true
    );

    /** @type {glsTextureTestUtil.SurfaceAccess} */ var surface = new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat);
    glsTextureTestUtil.sampleTexture3D(surface, src, texCoord, sampleParams);
    numFailedPixels = glsTextureTestUtil.computeTextureLookupDiff3D(result, reference.getAccess(), errorMask.getAccess(), src, texCoord, sampleParams, lookupPrec, lodPrec);

    if (numFailedPixels > 0)
        tcuImageCompare.displayImages(result, reference.getAccess(), errorMask.getAccess());

    return numFailedPixels == 0;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.Texture3DView} baseView
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @return {number}
 */
glsTextureTestUtil.computeTextureLookupDiff3D = function(
    result, reference, errorMask, baseView, texCoord,
    sampleParams, lookupPrec, lodPrec
) {
    assertMsgOptions(
        result.getWidth() == reference.getWidth() &&
        result.getHeight() == reference.getHeight(),
        'Result and reference images are not the same size', false, true
    );
    assertMsgOptions(
        result.getWidth() == errorMask.getWidth() &&
        result.getHeight() == errorMask.getHeight(),
        'Result and error mask images are not the same size', false, true
    );

    /** @type {tcuTexture.Texture3DView} */
    var src = baseView.getSubView(
        sampleParams.baseLevel, sampleParams.maxLevel
    );

    var sq =
        [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq =
        [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq =
        [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    var dstSize = [result.getWidth(), result.getHeight()];
    var dstW = dstSize[0];
    var dstH = dstSize[1];
    var srcSize = [src.getWidth(), src.getHeight(), src.getDepth()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triW = [
        deMath.swizzle(sampleParams.w, [0, 1, 2]),
        deMath.swizzle(sampleParams.w, [3, 2, 1])
    ];

    var lodBias = sampleParams.flags.useBias ? sampleParams.bias : 0.0;

    var posEps = 1.0 / ((1 << MIN_SUBPIXEL_BITS) + 1);

    var numFailed = 0;

    var lodOffsets = [
        [-1, 0],
        [+1, 0],
        [0, -1],
        [0, +1]
    ];

    var green = [0, 255, 0, 255];
    errorMask.clear(new tcuRGBA.RGBA(green).toVec());

    for (var py = 0; py < result.getHeight(); py++) {
        // Ugly hack, validation can take way too long at the moment.
        /*TODO: if (watchDog)
            qpWatchDog_touch(watchDog);*/

        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(resPix, sampleParams.colorScale, sampleParams.colorBias);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(refPix, sampleParams.colorScale, sampleParams.colorBias);

            // Try comparison to ideal reference first,
            // and if that fails use slower verificator.
            if (!deMath.boolAll(deMath.lessThanEqual(
                deMath.absDiff(resPix, refPix),
                lookupPrec.colorThreshold))
            ) {
                /** @type {number} */ var wx = px + 0.5;
                /** @type {number} */ var wy = py + 0.5;
                /** @type {number} */ var nx = wx / dstW;
                /** @type {number} */ var ny = wy / dstH;

                /** @type {boolean} */ var tri0 = nx + ny - posEps <= 1.0;
                /** @type {boolean} */ var tri1 = nx + ny + posEps >= 1.0;

                var isOk = false;

                assertMsgOptions(
                    tri0 || tri1,
                    'Pixel should belong at least to one triangle',
                    false, true
                );

                // Pixel can belong to either of the triangles
                // if it lies close enough to the edge.
                for (var triNdx = (tri0 ? 0 : 1);
                    triNdx <= (tri1 ? 1 : 0);
                    triNdx++) {
                    var triWx = triNdx ? dstW - wx : wx;
                    var triWy = triNdx ? dstH - wy : wy;
                    var triNx = triNdx ? 1.0 - nx : nx;
                    var triNy = triNdx ? 1.0 - ny : ny;

                    var coord = [
                        glsTextureTestUtil.projectedTriInterpolate(
                            triS[triNdx], triW[triNdx], triNx, triNy
                        ),
                        glsTextureTestUtil.projectedTriInterpolate(
                            triT[triNdx], triW[triNdx], triNx, triNy
                        ),
                        glsTextureTestUtil.projectedTriInterpolate(
                            triR[triNdx], triW[triNdx], triNx, triNy
                        )
                    ];
                    var coordDx = deMath.multiply([
                        glsTextureTestUtil.triDerivateX(
                            triS[triNdx], triW[triNdx], wx, dstW, triNy
                        ),
                        glsTextureTestUtil.triDerivateX(
                            triT[triNdx], triW[triNdx], wx, dstW, triNy
                        ),
                        glsTextureTestUtil.triDerivateX(
                            triR[triNdx], triW[triNdx], wx, dstW, triNy
                        )
                    ], srcSize);
                    var coordDy = deMath.multiply([
                        glsTextureTestUtil.triDerivateY(
                            triS[triNdx], triW[triNdx], wy, dstH, triNx
                        ),
                        glsTextureTestUtil.triDerivateY(
                            triT[triNdx], triW[triNdx], wy, dstH, triNx
                        ),
                        glsTextureTestUtil.triDerivateY(
                            triR[triNdx], triW[triNdx], wy, dstH, triNx
                        )
                    ], srcSize);

                    var lodBounds =
                        tcuTexLookupVerifier.computeLodBoundsFromDerivates(
                            coordDx[0], coordDx[1], coordDx[2],
                            coordDy[0], coordDy[1], coordDy[2], lodPrec
                        );

                    // Compute lod bounds across lodOffsets range.
                    for (var lodOffsNdx = 0;
                        lodOffsNdx < lodOffsets.length;
                        lodOffsNdx++) {
                        var wxo = triWx + lodOffsets[lodOffsNdx][0];
                        var wyo = triWy + lodOffsets[lodOffsNdx][1];
                        var nxo = wxo / dstW;
                        var nyo = wyo / dstH;

                        var coordO = [
                            glsTextureTestUtil.projectedTriInterpolate(
                                triS[triNdx], triW[triNdx], nxo, nyo
                            ),
                            glsTextureTestUtil.projectedTriInterpolate(
                                triT[triNdx], triW[triNdx], nxo, nyo
                            ),
                            glsTextureTestUtil.projectedTriInterpolate(
                                triR[triNdx], triW[triNdx], nxo, nyo
                            )
                        ];
                        var coordDxo = deMath.multiply([
                            glsTextureTestUtil.triDerivateX(
                                triS[triNdx], triW[triNdx], wxo, dstW, nyo
                            ),
                            glsTextureTestUtil.triDerivateX(
                                triT[triNdx], triW[triNdx], wxo, dstW, nyo
                            ),
                            glsTextureTestUtil.triDerivateX(
                                triR[triNdx], triW[triNdx], wxo, dstW, nyo
                            )
                        ], srcSize);
                        var coordDyo = deMath.multiply([
                            glsTextureTestUtil.triDerivateY(
                                triS[triNdx], triW[triNdx], wyo, dstH, nxo
                            ),
                            glsTextureTestUtil.triDerivateY(
                                triT[triNdx], triW[triNdx], wyo, dstH, nxo
                            ),
                            glsTextureTestUtil.triDerivateY(
                                triR[triNdx], triW[triNdx], wyo, dstH, nxo
                            )
                        ], srcSize);
                        var lodO =
                            tcuTexLookupVerifier.computeLodBoundsFromDerivates(
                                coordDxo[0], coordDxo[1], coordDxo[2],
                                coordDyo[0], coordDyo[1], coordDyo[2], lodPrec
                            );

                        lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                        lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                    }

                    var clampedLod = tcuTexLookupVerifier.clampLodBounds(
                        deMath.addScalar(lodBounds, lodBias),
                        [sampleParams.minLod, sampleParams.maxLod],
                        lodPrec
                    );

                    if (
                        tcuTexLookupVerifier.isLookupResultValid(
                            src, sampleParams.sampler, lookupPrec,
                            coord, clampedLod, resPix
                        )
                    ) {
                        isOk = true;
                        break;
                    }
                }

                if (!isOk) {
                    var red = [255, 0, 0, 255];
                    errorMask.setPixel(new tcuRGBA.RGBA(red).toVec(), px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.TextureCubeView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {tcuPixelFormat.PixelFormat} pixelFormat
 * @return {boolean}
 */
glsTextureTestUtil.verifyTextureCubeResult = function(
    result, src, texCoord, sampleParams, lookupPrec, lodPrec, pixelFormat
) {
    /** @type {tcuSurface.Surface} */
    var reference = new tcuSurface.Surface(
        result.getWidth(), result.getHeight()
    );
    /** @type {tcuSurface.Surface} */
    var errorMask = new tcuSurface.Surface(
        result.getWidth(), result.getHeight()
    );
    /** @type {number} */ var numFailedPixels = 0;

    assertMsgOptions(
        deMath.equal(glsTextureTestUtil.getCompareMask(pixelFormat), lookupPrec.colorMask),
        'Compare color masks do not match', false, true
    );

    /** @type {glsTextureTestUtil.SurfaceAccess} */
    var surface = new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat);

    glsTextureTestUtil.sampleTextureCube(
        surface, src, texCoord, sampleParams
    );

    numFailedPixels = glsTextureTestUtil.computeTextureLookupDiffCube(
        result, reference.getAccess(), errorMask.getAccess(),
        src, texCoord, sampleParams, lookupPrec, lodPrec
        /*, testCtx.getWatchDog()*/
    );

    if (numFailedPixels > 0)
        tcuImageCompare.displayImages(result, reference.getAccess(), errorMask.getAccess());

    return numFailedPixels == 0;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.TextureCubeView} baseView
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @return {number}
 */
glsTextureTestUtil.computeTextureLookupDiffCube = function(
    result, reference, errorMask, baseView, texCoord,
    sampleParams, lookupPrec, lodPrec
) {
    assertMsgOptions(
        result.getWidth() == reference.getWidth() &&
        result.getHeight() == reference.getHeight(),
        'Result and reference images are not the same size', false, true
    );
    assertMsgOptions(
        result.getWidth() == errorMask.getWidth() &&
        result.getHeight() == errorMask.getHeight(),
        'Result and error mask images are not the same size', false, true
    );

    /** @type {tcuTexture.TextureCubeView} */
    var src = baseView.getSubView(
        sampleParams.baseLevel, sampleParams.maxLevel
    );

    var sq =
        [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq =
        [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq =
        [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    var dstSize = [result.getWidth(), result.getHeight()];
    var dstW = dstSize[0];
    var dstH = dstSize[1];
    var srcSize = [src.getSize(), src.getSize()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triW = [
        deMath.swizzle(sampleParams.w, [0, 1, 2]),
        deMath.swizzle(sampleParams.w, [3, 2, 1])
    ];

    var lodBias = sampleParams.flags.useBias ? sampleParams.bias : 0.0;

    var posEps = 1.0 / ((1 << MIN_SUBPIXEL_BITS) + 1);

    var numFailed = 0;

    var lodOffsets = [
        [-1, 0],
        [+1, 0],
        [0, -1],
        [0, +1],

        // \note Not strictly allowed by spec,
        // but implementations do this in practice.
        [-1, -1],
        [-1, 1],
        [1, -1],
        [1, 1]
    ];

    var green = [0, 255, 0, 255];
    errorMask.clear(new tcuRGBA.RGBA(green).toVec());

    for (var py = 0; py < result.getHeight(); py++) {
        // Ugly hack, validation can take way too long at the moment.
        /*TODO: if (watchDog)
            qpWatchDog_touch(watchDog);*/

        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(resPix, sampleParams.colorScale, sampleParams.colorBias);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(refPix, sampleParams.colorScale, sampleParams.colorBias);

            // Try comparison to ideal reference first,
            // and if that fails use slower verificator.
            if (!deMath.boolAll(deMath.lessThanEqual(
                deMath.absDiff(resPix, refPix),
                lookupPrec.colorThreshold))
            ) {
                /** @type {number} */ var wx = px + 0.5;
                /** @type {number} */ var wy = py + 0.5;
                /** @type {number} */ var nx = wx / dstW;
                /** @type {number} */ var ny = wy / dstH;

                /** @type {boolean} */ var tri0 = nx + ny - posEps <= 1.0;
                /** @type {boolean} */ var tri1 = nx + ny + posEps >= 1.0;

                var isOk = false;

                assertMsgOptions(
                    tri0 || tri1,
                    'Pixel should belong at least to one triangle',
                    false, true
                );

                // Pixel can belong to either of the triangles
                // if it lies close enough to the edge.
                for (var triNdx = (tri0 ? 0 : 1);
                    triNdx <= (tri1 ? 1 : 0);
                    triNdx++) {
                    var triWx = triNdx ? dstW - wx : wx;
                    var triWy = triNdx ? dstH - wy : wy;
                    var triNx = triNdx ? 1.0 - nx : nx;
                    var triNy = triNdx ? 1.0 - ny : ny;

                    var coord = [
                        glsTextureTestUtil.projectedTriInterpolate(
                            triS[triNdx], triW[triNdx], triNx, triNy
                        ),
                        glsTextureTestUtil.projectedTriInterpolate(
                            triT[triNdx], triW[triNdx], triNx, triNy
                        ),
                        glsTextureTestUtil.projectedTriInterpolate(
                            triR[triNdx], triW[triNdx], triNx, triNy
                        )
                    ];
                    var coordDx = [
                        glsTextureTestUtil.triDerivateX(
                            triS[triNdx], triW[triNdx], wx, dstW, triNy
                        ),
                        glsTextureTestUtil.triDerivateX(
                            triT[triNdx], triW[triNdx], wx, dstW, triNy
                        ),
                        glsTextureTestUtil.triDerivateX(
                            triR[triNdx], triW[triNdx], wx, dstW, triNy
                        )
                    ];
                    var coordDy = [
                        glsTextureTestUtil.triDerivateY(
                            triS[triNdx], triW[triNdx], wy, dstH, triNx
                        ),
                        glsTextureTestUtil.triDerivateY(
                            triT[triNdx], triW[triNdx], wy, dstH, triNx
                        ),
                        glsTextureTestUtil.triDerivateY(
                            triR[triNdx], triW[triNdx], wy, dstH, triNx
                        )
                    ];

                    var lodBounds =
                        tcuTexLookupVerifier.computeCubeLodBoundsFromDerivates(
                            coord, coordDx, coordDy, src.getSize(), lodPrec
                        );

                    // Compute lod bounds across lodOffsets range.
                    for (var lodOffsNdx = 0;
                        lodOffsNdx < lodOffsets.length;
                        lodOffsNdx++) {
                        var wxo = triWx + lodOffsets[lodOffsNdx][0];
                        var wyo = triWy + lodOffsets[lodOffsNdx][1];
                        var nxo = wxo / dstW;
                        var nyo = wyo / dstH;

                        var coordO = [
                            glsTextureTestUtil.projectedTriInterpolate(
                                triS[triNdx], triW[triNdx], nxo, nyo
                            ),
                            glsTextureTestUtil.projectedTriInterpolate(
                                triT[triNdx], triW[triNdx], nxo, nyo
                            ),
                            glsTextureTestUtil.projectedTriInterpolate(
                                triR[triNdx], triW[triNdx], nxo, nyo
                            )
                        ];
                        var coordDxo = [
                            glsTextureTestUtil.triDerivateX(
                                triS[triNdx], triW[triNdx], wxo, dstW, nyo
                            ),
                            glsTextureTestUtil.triDerivateX(
                                triT[triNdx], triW[triNdx], wxo, dstW, nyo
                            ),
                            glsTextureTestUtil.triDerivateX(
                                triR[triNdx], triW[triNdx], wxo, dstW, nyo
                            )
                        ];
                        var coordDyo = [
                            glsTextureTestUtil.triDerivateY(
                                triS[triNdx], triW[triNdx], wyo, dstH, nxo
                            ),
                            glsTextureTestUtil.triDerivateY(
                                triT[triNdx], triW[triNdx], wyo, dstH, nxo
                            ),
                            glsTextureTestUtil.triDerivateY(
                                triR[triNdx], triW[triNdx], wyo, dstH, nxo
                            )
                        ];
                        var lodO =
                            tcuTexLookupVerifier.
                            computeCubeLodBoundsFromDerivates(
                                coordO, coordDxo, coordDyo,
                                src.getSize(), lodPrec
                            );

                        lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                        lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                    }

                    var clampedLod = tcuTexLookupVerifier.clampLodBounds(
                        deMath.addScalar(lodBounds, lodBias),
                        [sampleParams.minLod, sampleParams.maxLod],
                        lodPrec
                    );

                    if (tcuTexLookupVerifier.
                        isLookupResultValid_TextureCubeView(
                            src, sampleParams.sampler, lookupPrec, coord, clampedLod, resPix
                        )
                    ) {
                        isOk = true;
                        break;
                    }
                }

                if (!isOk) {
                    var red = [255, 0, 0, 255];
                    errorMask.setPixel(new tcuRGBA.RGBA(red).toVec(), px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.Texture2DArrayView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {tcuPixelFormat.PixelFormat} pixelFormat
 * @return {boolean}
 */
glsTextureTestUtil.verifyTexture2DArrayResult = function(result, src, texCoord, sampleParams, lookupPrec, lodPrec, pixelFormat) {
    DE_ASSERT(deMath.equal(glsTextureTestUtil.getCompareMask(pixelFormat), lookupPrec.colorMask));
    /** @type {tcuSurface.Surface} */ var reference = new tcuSurface.Surface(result.getWidth(), result.getHeight());
    /** @type {tcuSurface.Surface} */ var errorMask = new tcuSurface.Surface(result.getWidth(), result.getHeight());
    /** @type {number} */ var numFailedPixels;

    /** @type {glsTextureTestUtil.SurfaceAccess} */ var surface = new glsTextureTestUtil.SurfaceAccess(reference, pixelFormat);

    glsTextureTestUtil.sampleTexture2DArray(surface, src, texCoord, sampleParams);
    numFailedPixels = glsTextureTestUtil.computeTextureLookupDiff2DArray(result, reference.getAccess(), errorMask.getAccess(), src, texCoord, sampleParams, lookupPrec, lodPrec/*, testCtx.getWatchDog()*/);

    if (numFailedPixels > 0)
        tcuImageCompare.displayImages(result, reference.getAccess(), errorMask.getAccess());

    return numFailedPixels == 0;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.Texture2DArrayView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexCompareVerifier.TexComparePrecision} comparePrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {Array<number>} nonShadowThreshold
 * @return {number}
 */
glsTextureTestUtil.computeTextureCompareDiff2DArray = function(result, reference, errorMask, src, texCoord, sampleParams, comparePrec, lodPrec, nonShadowThreshold) {
    DE_ASSERT(result.getWidth() == reference.getWidth() && result.getHeight() == reference.getHeight());
    DE_ASSERT(result.getWidth() == errorMask.getWidth() && result.getHeight() == errorMask.getHeight());

    var sq = [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq = [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq = [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    var dstSize = [result.getWidth(), result.getHeight()];
    var dstW = dstSize[0];
    var dstH = dstSize[1];
    var srcSize = [src.getWidth(), src.getHeight()];

    // Coordinates and lod per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triW = [deMath.swizzle(sampleParams.w, [0, 1, 2]), deMath.swizzle(sampleParams.w, [3, 2, 1])];

    var lodBias = sampleParams.flags.use_bias ? [sampleParams.bias, sampleParams.bias] : [0, 0];
    var numFailed = 0;

    var lodOffsets = [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1]
    ];

    /** @type {Array<number>} */ var green = [0, 255, 0, 255];
    errorMask.clear(green);

    /** @type {Array<number>} */ var red = [];
    for (var py = 0; py < result.getHeight(); py++) {
        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);

            if (!deMath.boolAll(deMath.lessThanEqual(deMath.absDiff(deMath.swizzle(refPix, [1, 2, 3]), deMath.swizzle(resPix, [1, 2, 3])), nonShadowThreshold))) {
                red = [255, 0, 0, 255];
                errorMask.setPixel(red, px, py);
                numFailed += 1;
                continue;
            }

            if (resPix[0] != refPix[0]) {
                var wx = px + 0.5;
                var wy = py + 0.5;
                var nx = wx / dstW;
                var ny = wy / dstH;

                var triNdx = nx + ny >= 1.0 ? 1 : 0;
                var triWx = triNdx ? dstW - wx : wx;
                var triWy = triNdx ? dstH - wy : wy;
                var triNx = triNdx ? 1.0 - nx : nx;
                var triNy = triNdx ? 1.0 - ny : ny;

                var coord = [glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], triNx, triNy),
                             glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], triNx, triNy),
                             glsTextureTestUtil.projectedTriInterpolate(triR[triNdx], triW[triNdx], triNx, triNy)];
                var coordDx = deMath.multiply([glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wx, dstW, triNy),
                               glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wx, dstW, triNy)], srcSize);
                var coordDy = deMath.multiply([glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wy, dstH, triNx),
                               glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wy, dstH, triNx)], srcSize);

                var lodBounds = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDx[0], coordDx[1], coordDy[0], coordDy[1], lodPrec);

                // Compute lod bounds across lodOffsets range.
                for (var lodOffsNdx = 0; lodOffsNdx < lodOffsets.length; lodOffsNdx++) {
                    var wxo = triWx + lodOffsets[lodOffsNdx][0];
                    var wyo = triWy + lodOffsets[lodOffsNdx][1];
                    var nxo = wxo / dstW;
                    var nyo = wyo / dstH;

                    var coordO = [glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], nxo, nyo),
                                  glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], nxo, nyo)];
                    var coordDxo = deMath.multiply([glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wxo, dstW, nyo),
                                                 glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wxo, dstW, nyo)], srcSize);
                    var coordDyo = deMath.multiply([glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wyo, dstH, nxo),
                                                 glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wyo, dstH, nxo)], srcSize);
                    var lodO = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDxo[0], coordDxo[1], coordDyo[0], coordDyo[1], lodPrec);

                    lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                    lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                }

                var clampedLod = tcuTexLookupVerifier.clampLodBounds(deMath.add(lodBounds, lodBias), [sampleParams.minLod, sampleParams.maxLod], lodPrec);
                var isOk = tcuTexCompareVerifier.isTexCompareResultValid2DArray(src, sampleParams.sampler, comparePrec, coord, clampedLod, sampleParams.ref, resPix[0]);

                if (!isOk) {
                    red = [255, 0, 0, 255];
                    errorMask.setPixel(red, px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.TextureCubeView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexCompareVerifier.TexComparePrecision} comparePrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {Array<number>} nonShadowThreshold
 * @return {number}
 */
glsTextureTestUtil.computeTextureCompareDiffCube = function(result, reference, errorMask, src, texCoord, sampleParams, comparePrec, lodPrec, nonShadowThreshold) {
    DE_ASSERT(result.getWidth() == reference.getWidth() && result.getHeight() == reference.getHeight());
    DE_ASSERT(result.getWidth() == errorMask.getWidth() && result.getHeight() == errorMask.getHeight());

    var sq = [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    var tq = [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    var rq = [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    var dstSize = [result.getWidth(), result.getHeight()];
    var dstW = dstSize[0];
    var dstH = dstSize[1];
    var srcSize = src.getSize();

    // Coordinates per triangle.
    var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    var triW = [deMath.swizzle(sampleParams.w, [0, 1, 2]), deMath.swizzle(sampleParams.w, [3, 2, 1])];

    var lodBias = sampleParams.flags.use_bias ? [sampleParams.bias, sampleParams.bias] : [0, 0];
    var numFailed = 0;

    var lodOffsets = [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1]
    ];

    /** @type {Array<number>} */ var green = [0, 255, 0, 255];
    errorMask.clear(new tcuRGBA.RGBA(green).toVec());

    /** @type {Array<number>} */ var red = [];
    for (var py = 0; py < result.getHeight(); py++) {
        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);

            if (!deMath.boolAll(deMath.lessThanEqual(deMath.absDiff(deMath.swizzle(resPix, [1, 2, 3]), deMath.swizzle(refPix, [1, 2, 3])), nonShadowThreshold))) {
                red = [255, 0, 0, 255];
                errorMask.setPixel(red, px, py);
                numFailed += 1;
                continue;
            }

            if (resPix[0] != refPix[0]) {
                var wx = px + 0.5;
                var wy = py + 0.5;
                var nx = wx / dstW;
                var ny = wy / dstH;

                var triNdx = nx + ny >= 1.0 ? 1 : 0;
                var triWx = triNdx ? dstW - wx : wx;
                var triWy = triNdx ? dstH - wy : wy;
                var triNx = triNdx ? 1.0 - nx : nx;
                var triNy = triNdx ? 1.0 - ny : ny;

                var coord = [glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], triNx, triNy),
                             glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], triNx, triNy),
                             glsTextureTestUtil.projectedTriInterpolate(triR[triNdx], triW[triNdx], triNx, triNy)];
                var coordDx = [glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wx, dstW, triNy),
                               glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wx, dstW, triNy),
                               glsTextureTestUtil.triDerivateX(triR[triNdx], triW[triNdx], wx, dstW, triNy)];
                var coordDy = [glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wy, dstH, triNx),
                               glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wy, dstH, triNx),
                               glsTextureTestUtil.triDerivateY(triR[triNdx], triW[triNdx], wy, dstH, triNx)];

                var lodBounds = tcuTexLookupVerifier.computeCubeLodBoundsFromDerivates(coord, coordDx, coordDy, srcSize, lodPrec);

                // Compute lod bounds across lodOffsets range.
                for (var lodOffsNdx = 0; lodOffsNdx < lodOffsets.length; lodOffsNdx++) {
                    var wxo = triWx + lodOffsets[lodOffsNdx][0];
                    var wyo = triWy + lodOffsets[lodOffsNdx][1];
                    var nxo = wxo / dstW;
                    var nyo = wyo / dstH;

                    var coordO = [glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], nxo, nyo),
                                  glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], nxo, nyo),
                                  glsTextureTestUtil.projectedTriInterpolate(triR[triNdx], triW[triNdx], nxo, nyo)];
                    var coordDxo = [glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wxo, dstW, nyo),
                                    glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wxo, dstW, nyo),
                                    glsTextureTestUtil.triDerivateX(triR[triNdx], triW[triNdx], wxo, dstW, nyo)];
                    var coordDyo = [glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wyo, dstH, nxo),
                                    glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wyo, dstH, nxo),
                                    glsTextureTestUtil.triDerivateY(triR[triNdx], triW[triNdx], wyo, dstH, nxo)];
                    var lodO = tcuTexLookupVerifier.computeCubeLodBoundsFromDerivates(coordO, coordDxo, coordDyo, srcSize, lodPrec);

                    lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                    lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                }

                var clampedLod = tcuTexLookupVerifier.clampLodBounds(deMath.add(lodBounds, lodBias), [sampleParams.minLod, sampleParams.maxLod], lodPrec);
                var isOk = tcuTexCompareVerifier.isTexCompareResultValidCube(src, sampleParams.sampler, comparePrec, coord, clampedLod, sampleParams.ref, resPix[0]);

                if (!isOk) {
                    red = [255, 0, 0, 255];
                    errorMask.setPixel(red, px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

/**
 * @param {Array<number>} s
 * @param {Array<number>} w
 * @param {number} nx
 * @param {number} ny
 * @return {number}
 */
glsTextureTestUtil.projectedTriInterpolate = function(s, w, nx, ny) {
    return (s[0] * (1.0 - nx - ny) / w[0] + s[1] * ny / w[1] + s[2] * nx / w[2]) / ((1.0 - nx - ny) / w[0] + ny / w[1] + nx / w[2]);
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.Texture2DView} baseView
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {*=} watchDog - TODO: ??
 * @return {number}
 */
glsTextureTestUtil.computeTextureLookupDiff2D = function(result, reference, errorMask, baseView, texCoord, sampleParams, lookupPrec, lodPrec, watchDog) {
    DE_ASSERT(result.getWidth() == reference.getWidth() && result.getHeight() == reference.getHeight());
    DE_ASSERT(result.getWidth() == errorMask.getWidth() && result.getHeight() == errorMask.getHeight());

    /** @type {tcuTexture.Texture2DView} */ var src = baseView.getSubView(sampleParams.baseLevel, sampleParams.maxLevel);

    /** @type {Array<number>} */ var sq = [texCoord[0 + 0], texCoord[2 + 0], texCoord[4 + 0], texCoord[6 + 0]];
    /** @type {Array<number>} */ var tq = [texCoord[0 + 1], texCoord[2 + 1], texCoord[4 + 1], texCoord[6 + 1]];

    /** @type {Array<number>} */ var dstSize = [result.getWidth(), result.getHeight()];
    /** @type {number} */ var dstW = dstSize[0];
    /** @type {number} */ var dstH = dstSize[1];
    /** @type {Array<number>} */ var srcSize = [src.getWidth(), src.getHeight()];

    // Coordinates and lod per triangle.
    /** @type {Array<Array<number>>} */ var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triW = [deMath.swizzle(sampleParams.w, [0, 1, 2]), deMath.swizzle(sampleParams.w, [3, 2, 1])];

    /** @type {Array<number>} */ var lodBias = sampleParams.flags.use_bias ? [sampleParams.bias, sampleParams.bias] : [0.0, 0.0];

    /** @type {number} */ var numFailed = 0;

    /** @type {Array<Array<number>>} */ var lodOffsets = [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1]
    ];

    /** @type {Array<number>} */ var green = [0, 255, 0, 255];
    errorMask.clear(new tcuRGBA.RGBA(green).toVec());

    for (var py = 0; py < result.getHeight(); py++) {
        // Ugly hack, validation can take way too long at the moment.

        // TODO:are we implementing qpWatchDog? skipping in the meantime
        // if (watchDog)
        //     qpWatchDog_touch(watchDog);

        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(resPix, sampleParams.colorScale, sampleParams.colorBias);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(refPix, sampleParams.colorScale, sampleParams.colorBias);

            // Try comparison to ideal reference first, and if that fails use slower verificator.
            if (!deMath.boolAll(deMath.lessThanEqual(deMath.absDiff(resPix, refPix), lookupPrec.colorThreshold))) {
                /** @type {number} */ var wx = px + 0.5;
                /** @type {number} */ var wy = py + 0.5;
                /** @type {number} */ var nx = wx / dstW;
                /** @type {number} */ var ny = wy / dstH;

                /** @type {number} */ var triNdx = nx + ny >= 1.0 ? 1 : 0;
                /** @type {number} */ var triWx = triNdx ? dstW - wx : wx;
                /** @type {number} */ var triWy = triNdx ? dstH - wy : wy;
                /** @type {number} */ var triNx = triNdx ? 1.0 - nx : nx;
                /** @type {number} */ var triNy = triNdx ? 1.0 - ny : ny;

                /** @type {Array<number>} */ var coord = [
                    glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], triNx, triNy),
                    glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], triNx, triNy)
                ];
                /** @type {Array<number>} */ var coordDx = deMath.multiply([
                    glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wx, dstW, triNy),
                    glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wx, dstW, triNy)], srcSize);
                /** @type {Array<number>} */ var coordDy = deMath.multiply([
                    glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wy, dstH, triNx),
                    glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wy, dstH, triNx)], srcSize);

                /** @type {Array<number>} */
                var lodBounds = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDx[0], coordDx[1], coordDy[0], coordDy[1], lodPrec);

                // Compute lod bounds across lodOffsets range.
                for (var lodOffsNdx = 0; lodOffsNdx < lodOffsets.length; lodOffsNdx++) {
                    /** @type {number} */ var wxo = triWx + lodOffsets[lodOffsNdx][0];
                    /** @type {number} */ var wyo = triWy + lodOffsets[lodOffsNdx][1];
                    /** @type {number} */ var nxo = wxo / dstW;
                    /** @type {number} */ var nyo = wyo / dstH;

                    /** @type {Array<number>} */ var coordO = [
                        glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], nxo, nyo),
                        glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], nxo, nyo)];
                    /** @type {Array<number>} */ var coordDxo = deMath.multiply([
                        glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wxo, dstW, nyo),
                        glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wxo, dstW, nyo)], srcSize);
                    /** @type {Array<number>} */ var coordDyo = deMath.multiply([
                        glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wyo, dstH, nxo),
                        glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wyo, dstH, nxo)], srcSize);
                    /** @type {Array<number>} */
                    var lodO = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDxo[0], coordDxo[1], coordDyo[0], coordDyo[1], lodPrec);

                    lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                    lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                }

                /** @type {Array<number>} */ var clampedLod = tcuTexLookupVerifier.clampLodBounds(
                    deMath.add(lodBounds, lodBias), [sampleParams.minLod, sampleParams.maxLod], lodPrec);
                /** @type {boolean} */
                var isOk = tcuTexLookupVerifier.isLookupResultValid_Texture2DView(src, sampleParams.sampler, lookupPrec, coord, clampedLod, resPix);

                if (!isOk) {
                    /** @type {tcuRGBA.RGBA} */ var red = tcuRGBA.newRGBAComponents(255, 0, 0, 255);
                    errorMask.setPixel(red.toVec(), px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

// Verifies texture lookup results and returns number of failed pixels.

/**
 * @param {tcuTexture.ConstPixelBufferAccess} result
 * @param {tcuTexture.ConstPixelBufferAccess} reference
 * @param {tcuTexture.PixelBufferAccess} errorMask
 * @param {tcuTexture.Texture2DArrayView} src
 * @param {Array<number>} texCoord
 * @param {glsTextureTestUtil.ReferenceParams} sampleParams
 * @param {tcuTexLookupVerifier.LookupPrecision} lookupPrec
 * @param {tcuTexLookupVerifier.LodPrecision} lodPrec
 * @param {*=} watchDog - TODO: ??
 * @return {number}
 */
glsTextureTestUtil.computeTextureLookupDiff2DArray = function(result, reference, errorMask, src, texCoord, sampleParams, lookupPrec, lodPrec, watchDog) {
    DE_ASSERT(result.getWidth() == reference.getWidth() && result.getHeight() == reference.getHeight());
    DE_ASSERT(result.getWidth() == errorMask.getWidth() && result.getHeight() == errorMask.getHeight());

    /** @type {Array<number>} */ var sq = [texCoord[0 + 0], texCoord[3 + 0], texCoord[6 + 0], texCoord[9 + 0]];
    /** @type {Array<number>} */ var tq = [texCoord[0 + 1], texCoord[3 + 1], texCoord[6 + 1], texCoord[9 + 1]];
    /** @type {Array<number>} */ var rq = [texCoord[0 + 2], texCoord[3 + 2], texCoord[6 + 2], texCoord[9 + 2]];

    /** @type {Array<number>} */ var dstSize = [result.getWidth(), result.getHeight()];
    /** @type {number} */ var dstW = dstSize[0];
    /** @type {number} */ var dstH = dstSize[1];
    /** @type {Array<number>} */ var srcSize = [src.getWidth(), src.getHeight()];

    // Coordinates and lod per triangle.
    /** @type {Array<Array<number>>} */ var triS = [deMath.swizzle(sq, [0, 1, 2]), deMath.swizzle(sq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triT = [deMath.swizzle(tq, [0, 1, 2]), deMath.swizzle(tq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triR = [deMath.swizzle(rq, [0, 1, 2]), deMath.swizzle(rq, [3, 2, 1])];
    /** @type {Array<Array<number>>} */ var triW = [deMath.swizzle(sampleParams.w, [0, 1, 2]), deMath.swizzle(sampleParams.w, [3, 2, 1])];

    /** @type {Array<number>} */ var lodBias = sampleParams.flags.use_bias ? [sampleParams.bias, sampleParams.bias] : [0.0, 0.0];

    /** @type {number} */ var numFailed = 0;

    /** @type {Array<Array<number>>} */ var lodOffsets = [
        [-1, 0],
        [1, 0],
        [0, -1],
        [0, 1]
    ];

    /** @type {Array<number>} */ var green = [0, 255, 0, 255];
    errorMask.clear(new tcuRGBA.RGBA(green).toVec());

    for (var py = 0; py < result.getHeight(); py++) {
        // Ugly hack, validation can take way too long at the moment.

        // TODO:are we implementing qpWatchDog? skipping in the meantime
        // if (watchDog)
        //     qpWatchDog_touch(watchDog);

        for (var px = 0; px < result.getWidth(); px++) {
            /** @type {Array<number>} */
            var resPix = result.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(resPix, sampleParams.colorScale, sampleParams.colorBias);
            /** @type {Array<number>} */
            var refPix = reference.getPixel(px, py);
            glsTextureTestUtil.deapplyScaleAndBias(refPix, sampleParams.colorScale, sampleParams.colorBias);


            // Try comparison to ideal reference first, and if that fails use slower verificator.
            if (!deMath.boolAll(deMath.lessThanEqual(deMath.absDiff(resPix, refPix), lookupPrec.colorThreshold))) {
                /** @type {number} */ var wx = px + 0.5;
                /** @type {number} */ var wy = py + 0.5;
                /** @type {number} */ var nx = wx / dstW;
                /** @type {number} */ var ny = wy / dstH;

                /** @type {number} */ var triNdx = nx + ny >= 1.0 ? 1 : 0;
                /** @type {number} */ var triWx = triNdx ? dstW - wx : wx;
                /** @type {number} */ var triWy = triNdx ? dstH - wy : wy;
                /** @type {number} */ var triNx = triNdx ? 1.0 - nx : nx;
                /** @type {number} */ var triNy = triNdx ? 1.0 - ny : ny;

                /** @type {Array<number>} */ var coord = [
                    glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], triNx, triNy),
                    glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], triNx, triNy),
                    glsTextureTestUtil.projectedTriInterpolate(triR[triNdx], triW[triNdx], triNx, triNy)
                ];
                /** @type {Array<number>} */ var coordDx = deMath.multiply([
                    glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wx, dstW, triNy),
                    glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wx, dstW, triNy)], srcSize);
                /** @type {Array<number>} */ var coordDy = deMath.multiply([
                    glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wy, dstH, triNx),
                    glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wy, dstH, triNx)], srcSize);

                /** @type {Array<number>} */
                var lodBounds = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDx[0], coordDx[1], coordDy[0], coordDy[1], lodPrec);

                // Compute lod bounds across lodOffsets range.
                for (var lodOffsNdx = 0; lodOffsNdx < lodOffsets.length; lodOffsNdx++) {
                    /** @type {number} */ var wxo = triWx + lodOffsets[lodOffsNdx][0];
                    /** @type {number} */ var wyo = triWy + lodOffsets[lodOffsNdx][1];
                    /** @type {number} */ var nxo = wxo / dstW;
                    /** @type {number} */ var nyo = wyo / dstH;

                    /** @type {Array<number>} */ var coordO = [
                        glsTextureTestUtil.projectedTriInterpolate(triS[triNdx], triW[triNdx], nxo, nyo),
                        glsTextureTestUtil.projectedTriInterpolate(triT[triNdx], triW[triNdx], nxo, nyo),
                        glsTextureTestUtil.projectedTriInterpolate(triR[triNdx], triW[triNdx], nxo, nyo)
                    ];
                    /** @type {Array<number>} */ var coordDxo = deMath.multiply([
                        glsTextureTestUtil.triDerivateX(triS[triNdx], triW[triNdx], wxo, dstW, nyo),
                        glsTextureTestUtil.triDerivateX(triT[triNdx], triW[triNdx], wxo, dstW, nyo)], srcSize
                    );
                    /** @type {Array<number>} */ var coordDyo = deMath.multiply([
                        glsTextureTestUtil.triDerivateY(triS[triNdx], triW[triNdx], wyo, dstH, nxo),
                        glsTextureTestUtil.triDerivateY(triT[triNdx], triW[triNdx], wyo, dstH, nxo)], srcSize
                    );
                    /** @type {Array<number>} */
                    var lodO = tcuTexLookupVerifier.computeLodBoundsFromDerivatesUV(coordDxo[0], coordDxo[1], coordDyo[0], coordDyo[1], lodPrec);

                    lodBounds[0] = Math.min(lodBounds[0], lodO[0]);
                    lodBounds[1] = Math.max(lodBounds[1], lodO[1]);
                }

                /** @type {Array<number>} */ var clampedLod = tcuTexLookupVerifier.clampLodBounds(
                    deMath.add(lodBounds, lodBias), [sampleParams.minLod, sampleParams.maxLod], lodPrec);
                /** @type {boolean} */
                var isOk = tcuTexLookupVerifier.isLookupResultValid_Texture2DArrayView(src, sampleParams.sampler, lookupPrec, coord, clampedLod, resPix);

                if (!isOk) {
                    /** @type {tcuRGBA.RGBA} */ var red = tcuRGBA.newRGBAComponents(255, 0, 0, 255);
                    errorMask.setPixel(red.toVec(), px, py);
                    numFailed += 1;
                }
            }
        }
    }

    return numFailed;
};

});

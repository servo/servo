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
goog.provide('framework.opengl.gluShaderUtil');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var gluShaderUtil = framework.opengl.gluShaderUtil;
var deMath = framework.delibs.debase.deMath;

/**
 * ShadingLanguageVersion
 * @enum
 */
gluShaderUtil.GLSLVersion = {
    V100_ES: 0, //!< GLSL ES 1.0
    V300_ES: 1 //!< GLSL ES 3.0
};

/**
 * gluShaderUtil.glslVersionUsesInOutQualifiers
 * @param {gluShaderUtil.GLSLVersion} version
 * @return {boolean}
 */
gluShaderUtil.glslVersionUsesInOutQualifiers = function(version) {
    return version == gluShaderUtil.GLSLVersion.V300_ES;
};

/**
 * gluShaderUtil.isGLSLVersionSupported
 * @param {WebGL2RenderingContext|WebGLRenderingContextBase} ctx
 * @param {gluShaderUtil.GLSLVersion} version
 * @return {boolean}
 */
gluShaderUtil.isGLSLVersionSupported = function(ctx, version) {
    return version <= gluShaderUtil.getGLSLVersion(ctx);
};

/**
 * gluShaderUtil.getGLSLVersion - Returns a gluShaderUtil.GLSLVersion based on a given webgl context.
 * @param {WebGL2RenderingContext|WebGLRenderingContextBase} gl
 * @return {gluShaderUtil.GLSLVersion}
 */
gluShaderUtil.getGLSLVersion = function(gl) {
    var glslversion = gl.getParameter(gl.SHADING_LANGUAGE_VERSION);

    // TODO: Versions are not yet well implemented... Firefox returns GLSL ES 1.0 in some cases,
    // and Chromium returns GLSL ES 2.0 in some cases. Returning the right version for
    // testing.
    // return gluShaderUtil.GLSLVersion.V300_ES;

    if (glslversion.indexOf('WebGL GLSL ES 1.0') != -1) return gluShaderUtil.GLSLVersion.V100_ES;
    if (glslversion.indexOf('WebGL GLSL ES 3.0') != -1) return gluShaderUtil.GLSLVersion.V300_ES;

    throw new Error('Invalid WebGL version');
};

/**
 * gluShaderUtil.getGLSLVersionDeclaration - Returns a string declaration for the glsl version in a shader.
 * @param {gluShaderUtil.GLSLVersion} version
 * @return {string}
 */
gluShaderUtil.getGLSLVersionDeclaration = function(version) {
    /** @type {Array<string>} */ var s_decl =
    [
        '#version 100',
        '#version 300 es'
    ];

    if (version > s_decl.length - 1)
        throw new Error('Unsupported GLSL version.');

    return s_decl[version];
};

/**
 * gluShaderUtil.getGLSLVersionString - Returns the same thing as
 * getGLSLVersionDeclaration() but without the substring '#version'
 * @param {gluShaderUtil.GLSLVersion} version
 * @return {string}
 */
gluShaderUtil.getGLSLVersionString = function(version) {
    /** @type {Array<string>} */ var s_decl =
    [
        '100',
        '300 es'
    ];

    if (version > s_decl.length - 1)
        throw new Error('Unsupported GLSL version.');

    return s_decl[version];
};

/**
 * @enum
 */
gluShaderUtil.precision = {
    PRECISION_LOWP: 0,
    PRECISION_MEDIUMP: 1,
    PRECISION_HIGHP: 2
};

gluShaderUtil.getPrecisionName = function(prec) {
    var s_names = [
        'lowp',
        'mediump',
        'highp'
    ];

    return s_names[prec];
};

/**
 * The Type constants
 * @enum {number}
 */
gluShaderUtil.DataType = {
    INVALID: 0,

    FLOAT: 1,
    FLOAT_VEC2: 2,
    FLOAT_VEC3: 3,
    FLOAT_VEC4: 4,
    FLOAT_MAT2: 5,
    FLOAT_MAT2X3: 6,
    FLOAT_MAT2X4: 7,
    FLOAT_MAT3X2: 8,
    FLOAT_MAT3: 9,
    FLOAT_MAT3X4: 10,
    FLOAT_MAT4X2: 11,
    FLOAT_MAT4X3: 12,
    FLOAT_MAT4: 13,

    INT: 14,
    INT_VEC2: 15,
    INT_VEC3: 16,
    INT_VEC4: 17,

    UINT: 18,
    UINT_VEC2: 19,
    UINT_VEC3: 20,
    UINT_VEC4: 21,

    BOOL: 22,
    BOOL_VEC2: 23,
    BOOL_VEC3: 24,
    BOOL_VEC4: 25,

    SAMPLER_2D: 26,
    SAMPLER_CUBE: 27,
    SAMPLER_2D_ARRAY: 28,
    SAMPLER_3D: 29,

    SAMPLER_2D_SHADOW: 30,
    SAMPLER_CUBE_SHADOW: 31,
    SAMPLER_2D_ARRAY_SHADOW: 32,

    INT_SAMPLER_2D: 33,
    INT_SAMPLER_CUBE: 34,
    INT_SAMPLER_2D_ARRAY: 35,
    INT_SAMPLER_3D: 36,

    UINT_SAMPLER_2D: 37,
    UINT_SAMPLER_CUBE: 38,
    UINT_SAMPLER_2D_ARRAY: 39,
    UINT_SAMPLER_3D: 40
};

/**
 * Returns type of float scalars
 * @param {gluShaderUtil.DataType} dataType
 * @return {string} type of float scalar
 */
gluShaderUtil.getDataTypeFloatScalars = function(dataType) {

    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT: return 'float';
        case gluShaderUtil.DataType.FLOAT_VEC2: return 'vec2';
        case gluShaderUtil.DataType.FLOAT_VEC3: return 'vec3';
        case gluShaderUtil.DataType.FLOAT_VEC4: return 'vec4';
        case gluShaderUtil.DataType.FLOAT_MAT2: return 'mat2';
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 'mat2x3';
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 'mat2x4';
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 'mat3x2';
        case gluShaderUtil.DataType.FLOAT_MAT3: return 'mat3';
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 'mat3x4';
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 'mat4x2';
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 'mat4x3';
        case gluShaderUtil.DataType.FLOAT_MAT4: return 'mat4';
        case gluShaderUtil.DataType.INT: return 'float';
        case gluShaderUtil.DataType.INT_VEC2: return 'vec2';
        case gluShaderUtil.DataType.INT_VEC3: return 'vec3';
        case gluShaderUtil.DataType.INT_VEC4: return 'vec4';
        case gluShaderUtil.DataType.UINT: return 'float';
        case gluShaderUtil.DataType.UINT_VEC2: return 'vec2';
        case gluShaderUtil.DataType.UINT_VEC3: return 'vec3';
        case gluShaderUtil.DataType.UINT_VEC4: return 'vec4';
        case gluShaderUtil.DataType.BOOL: return 'float';
        case gluShaderUtil.DataType.BOOL_VEC2: return 'vec2';
        case gluShaderUtil.DataType.BOOL_VEC3: return 'vec3';
        case gluShaderUtil.DataType.BOOL_VEC4: return 'vec4';
    }
    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * gluShaderUtil.getDataTypeVector
 * @param {gluShaderUtil.DataType} scalarType
 * @param {number} size
 * @return {gluShaderUtil.DataType}
 */
gluShaderUtil.getDataTypeVector = function(scalarType, size) {
    var floats = [gluShaderUtil.DataType.FLOAT,
                  gluShaderUtil.DataType.FLOAT_VEC2,
                  gluShaderUtil.DataType.FLOAT_VEC3,
                  gluShaderUtil.DataType.FLOAT_VEC4];
    var ints = [gluShaderUtil.DataType.INT,
                  gluShaderUtil.DataType.INT_VEC2,
                  gluShaderUtil.DataType.INT_VEC3,
                  gluShaderUtil.DataType.INT_VEC4];
    var uints = [gluShaderUtil.DataType.UINT,
                  gluShaderUtil.DataType.UINT_VEC2,
                  gluShaderUtil.DataType.UINT_VEC3,
                  gluShaderUtil.DataType.UINT_VEC4];
    var bools = [gluShaderUtil.DataType.BOOL,
                  gluShaderUtil.DataType.BOOL_VEC2,
                  gluShaderUtil.DataType.BOOL_VEC3,
                  gluShaderUtil.DataType.BOOL_VEC4];

    switch (scalarType) {
        case gluShaderUtil.DataType.FLOAT: return floats[size - 1];
        case gluShaderUtil.DataType.INT: return ints[size - 1];
        case gluShaderUtil.DataType.UINT: return uints[size - 1];
        case gluShaderUtil.DataType.BOOL: return bools[size - 1];
        default:
            throw new Error('Scalar type is not a vectoe:' + scalarType);
    }
};

/**
 * gluShaderUtil.getDataTypeFloatVec
 * @param {number} vecSize
 * @return {gluShaderUtil.DataType}
 */
gluShaderUtil.getDataTypeFloatVec = function(vecSize) {
    return gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.FLOAT, vecSize);
};

/**
 * gluShaderUtil.isDataTypeBoolOrBVec
 * @param {gluShaderUtil.DataType} dataType
 * @return {boolean}
 */
gluShaderUtil.isDataTypeBoolOrBVec = function(dataType) {
    return (dataType >= gluShaderUtil.DataType.BOOL) && (dataType <= gluShaderUtil.DataType.BOOL_VEC4);
};

/**
 * Returns type of scalar
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {string} type of scalar type
 */
gluShaderUtil.getDataTypeScalarType = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT: return 'float';
        case gluShaderUtil.DataType.FLOAT_VEC2: return 'float';
        case gluShaderUtil.DataType.FLOAT_VEC3: return 'float';
        case gluShaderUtil.DataType.FLOAT_VEC4: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT2: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT3: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 'float';
        case gluShaderUtil.DataType.FLOAT_MAT4: return 'float';
        case gluShaderUtil.DataType.INT: return 'int';
        case gluShaderUtil.DataType.INT_VEC2: return 'int';
        case gluShaderUtil.DataType.INT_VEC3: return 'int';
        case gluShaderUtil.DataType.INT_VEC4: return 'int';
        case gluShaderUtil.DataType.UINT: return 'uint';
        case gluShaderUtil.DataType.UINT_VEC2: return 'uint';
        case gluShaderUtil.DataType.UINT_VEC3: return 'uint';
        case gluShaderUtil.DataType.UINT_VEC4: return 'uint';
        case gluShaderUtil.DataType.BOOL: return 'bool';
        case gluShaderUtil.DataType.BOOL_VEC2: return 'bool';
        case gluShaderUtil.DataType.BOOL_VEC3: return 'bool';
        case gluShaderUtil.DataType.BOOL_VEC4: return 'bool';
        case gluShaderUtil.DataType.SAMPLER_2D: return 'sampler2D';
        case gluShaderUtil.DataType.SAMPLER_CUBE: return 'samplerCube';
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY: return 'sampler2DArray';
        case gluShaderUtil.DataType.SAMPLER_3D: return 'sampler3D';
        case gluShaderUtil.DataType.SAMPLER_2D_SHADOW: return 'sampler2DShadow';
        case gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW: return 'samplerCubeShadow';
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW: return 'sampler2DArrayShadow';
        case gluShaderUtil.DataType.INT_SAMPLER_2D: return 'isampler2D';
        case gluShaderUtil.DataType.INT_SAMPLER_CUBE: return 'isamplerCube';
        case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY: return 'isampler2DArray';
        case gluShaderUtil.DataType.INT_SAMPLER_3D: return 'isampler3D';
        case gluShaderUtil.DataType.UINT_SAMPLER_2D: return 'usampler2D';
        case gluShaderUtil.DataType.UINT_SAMPLER_CUBE: return 'usamplerCube';
        case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY: return 'usampler2DArray';
        case gluShaderUtil.DataType.UINT_SAMPLER_3D: return 'usampler3D';
    }
   throw new Error('Unrecognized datatype:' + dataType);
};

/**
 * Returns type of scalar
 * @param {?gluShaderUtil.DataType} dataType shader
 * @return {gluShaderUtil.DataType} type of scalar type
 */
gluShaderUtil.getDataTypeScalarTypeAsDataType = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_VEC2: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_VEC3: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_VEC4: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT2: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT3: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.FLOAT_MAT4: return gluShaderUtil.DataType.FLOAT;
        case gluShaderUtil.DataType.INT: return gluShaderUtil.DataType.INT;
        case gluShaderUtil.DataType.INT_VEC2: return gluShaderUtil.DataType.INT;
        case gluShaderUtil.DataType.INT_VEC3: return gluShaderUtil.DataType.INT;
        case gluShaderUtil.DataType.INT_VEC4: return gluShaderUtil.DataType.INT;
        case gluShaderUtil.DataType.UINT: return gluShaderUtil.DataType.UINT;
        case gluShaderUtil.DataType.UINT_VEC2: return gluShaderUtil.DataType.UINT;
        case gluShaderUtil.DataType.UINT_VEC3: return gluShaderUtil.DataType.UINT;
        case gluShaderUtil.DataType.UINT_VEC4: return gluShaderUtil.DataType.UINT;
        case gluShaderUtil.DataType.BOOL: return gluShaderUtil.DataType.BOOL;
        case gluShaderUtil.DataType.BOOL_VEC2: return gluShaderUtil.DataType.BOOL;
        case gluShaderUtil.DataType.BOOL_VEC3: return gluShaderUtil.DataType.BOOL;
        case gluShaderUtil.DataType.BOOL_VEC4: return gluShaderUtil.DataType.BOOL;
        case gluShaderUtil.DataType.SAMPLER_2D:
        case gluShaderUtil.DataType.SAMPLER_CUBE:
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY:
        case gluShaderUtil.DataType.SAMPLER_3D:
        case gluShaderUtil.DataType.SAMPLER_2D_SHADOW:
        case gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW:
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW:
        case gluShaderUtil.DataType.INT_SAMPLER_2D:
        case gluShaderUtil.DataType.INT_SAMPLER_CUBE:
        case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY:
        case gluShaderUtil.DataType.INT_SAMPLER_3D:
        case gluShaderUtil.DataType.UINT_SAMPLER_2D:
        case gluShaderUtil.DataType.UINT_SAMPLER_CUBE:
        case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY:
        case gluShaderUtil.DataType.UINT_SAMPLER_3D:
            return dataType;
    }
    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * Checks if dataType is integer or vectors of integers
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType integer or integer vector
 */
gluShaderUtil.isDataTypeIntOrIVec = function(dataType) {
    /** @type {boolean} */ var retVal = false;
    switch (dataType) {
        case gluShaderUtil.DataType.INT:
        case gluShaderUtil.DataType.INT_VEC2:
        case gluShaderUtil.DataType.INT_VEC3:
        case gluShaderUtil.DataType.INT_VEC4:
            retVal = true;
    }

    return retVal;
};

/**
 * Checks if dataType is unsigned integer or vectors of unsigned integers
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType unsigned integer or unsigned integer vector
 */
gluShaderUtil.isDataTypeUintOrUVec = function(dataType) {
    /** @type {boolean} */ var retVal = false;
    switch (dataType) {
        case gluShaderUtil.DataType.UINT:
        case gluShaderUtil.DataType.UINT_VEC2:
        case gluShaderUtil.DataType.UINT_VEC3:
        case gluShaderUtil.DataType.UINT_VEC4:
            retVal = true;
    }

    return retVal;
};

/**
* Returns type of scalar size
* @param {gluShaderUtil.DataType} dataType shader
* @return {number} with size of the type of scalar
*/
gluShaderUtil.getDataTypeScalarSize = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT: return 1;
        case gluShaderUtil.DataType.FLOAT_VEC2: return 2;
        case gluShaderUtil.DataType.FLOAT_VEC3: return 3;
        case gluShaderUtil.DataType.FLOAT_VEC4: return 4;
        case gluShaderUtil.DataType.FLOAT_MAT2: return 4;
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 6;
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 8;
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 6;
        case gluShaderUtil.DataType.FLOAT_MAT3: return 9;
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 12;
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 8;
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 12;
        case gluShaderUtil.DataType.FLOAT_MAT4: return 16;
        case gluShaderUtil.DataType.INT: return 1;
        case gluShaderUtil.DataType.INT_VEC2: return 2;
        case gluShaderUtil.DataType.INT_VEC3: return 3;
        case gluShaderUtil.DataType.INT_VEC4: return 4;
        case gluShaderUtil.DataType.UINT: return 1;
        case gluShaderUtil.DataType.UINT_VEC2: return 2;
        case gluShaderUtil.DataType.UINT_VEC3: return 3;
        case gluShaderUtil.DataType.UINT_VEC4: return 4;
        case gluShaderUtil.DataType.BOOL: return 1;
        case gluShaderUtil.DataType.BOOL_VEC2: return 2;
        case gluShaderUtil.DataType.BOOL_VEC3: return 3;
        case gluShaderUtil.DataType.BOOL_VEC4: return 4;
        case gluShaderUtil.DataType.SAMPLER_2D: return 1;
        case gluShaderUtil.DataType.SAMPLER_CUBE: return 1;
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY: return 1;
        case gluShaderUtil.DataType.SAMPLER_3D: return 1;
        case gluShaderUtil.DataType.SAMPLER_2D_SHADOW: return 1;
        case gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW: return 1;
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW: return 1;
        case gluShaderUtil.DataType.INT_SAMPLER_2D: return 1;
        case gluShaderUtil.DataType.INT_SAMPLER_CUBE: return 1;
        case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY: return 1;
        case gluShaderUtil.DataType.INT_SAMPLER_3D: return 1;
        case gluShaderUtil.DataType.UINT_SAMPLER_2D: return 1;
        case gluShaderUtil.DataType.UINT_SAMPLER_CUBE: return 1;
        case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY: return 1;
        case gluShaderUtil.DataType.UINT_SAMPLER_3D: return 1;
    }
    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * Checks if dataType is float or vector
 * @param {?gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType float or vector
 */
gluShaderUtil.isDataTypeFloatOrVec = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT:
        case gluShaderUtil.DataType.FLOAT_VEC2:
        case gluShaderUtil.DataType.FLOAT_VEC3:
        case gluShaderUtil.DataType.FLOAT_VEC4:
            return true;
    }
    return false;
};

/**
 * Checks if dataType is a matrix
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType matrix or not
 */
gluShaderUtil.isDataTypeMatrix = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT_MAT2:
        case gluShaderUtil.DataType.FLOAT_MAT2X3:
        case gluShaderUtil.DataType.FLOAT_MAT2X4:
        case gluShaderUtil.DataType.FLOAT_MAT3X2:
        case gluShaderUtil.DataType.FLOAT_MAT3:
        case gluShaderUtil.DataType.FLOAT_MAT3X4:
        case gluShaderUtil.DataType.FLOAT_MAT4X2:
        case gluShaderUtil.DataType.FLOAT_MAT4X3:
        case gluShaderUtil.DataType.FLOAT_MAT4:
            return true;
    }
    return false;
};

/**
 * Checks if dataType is a vector
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType vector or not
 */
gluShaderUtil.isDataTypeScalar = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT:
        case gluShaderUtil.DataType.INT:
        case gluShaderUtil.DataType.UINT:
        case gluShaderUtil.DataType.BOOL:
            return true;
    }
    return false;
};

/**
 * Checks if dataType is a vector
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType vector or not
 */
gluShaderUtil.isDataTypeVector = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT_VEC2:
        case gluShaderUtil.DataType.FLOAT_VEC3:
        case gluShaderUtil.DataType.FLOAT_VEC4:
        case gluShaderUtil.DataType.INT_VEC2:
        case gluShaderUtil.DataType.INT_VEC3:
        case gluShaderUtil.DataType.INT_VEC4:
        case gluShaderUtil.DataType.UINT_VEC2:
        case gluShaderUtil.DataType.UINT_VEC3:
        case gluShaderUtil.DataType.UINT_VEC4:
        case gluShaderUtil.DataType.BOOL_VEC2:
        case gluShaderUtil.DataType.BOOL_VEC3:
        case gluShaderUtil.DataType.BOOL_VEC4:
            return true;
    }
    return false;
};

/**
 * Checks if dataType is a vector or a scalar type
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType vector or scalar or not
 */
gluShaderUtil.isDataTypeScalarOrVector = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT:
        case gluShaderUtil.DataType.FLOAT_VEC2:
        case gluShaderUtil.DataType.FLOAT_VEC3:
        case gluShaderUtil.DataType.FLOAT_VEC4:
        case gluShaderUtil.DataType.INT:
        case gluShaderUtil.DataType.INT_VEC2:
        case gluShaderUtil.DataType.INT_VEC3:
        case gluShaderUtil.DataType.INT_VEC4:
        case gluShaderUtil.DataType.UINT:
        case gluShaderUtil.DataType.UINT_VEC2:
        case gluShaderUtil.DataType.UINT_VEC3:
        case gluShaderUtil.DataType.UINT_VEC4:
        case gluShaderUtil.DataType.BOOL:
        case gluShaderUtil.DataType.BOOL_VEC2:
        case gluShaderUtil.DataType.BOOL_VEC3:
        case gluShaderUtil.DataType.BOOL_VEC4:
            return true;
    }
    return false;
};

/**
 * Checks if dataType is a sampler
 * @param {gluShaderUtil.DataType} dataType shader
 * @return {boolean} Is dataType vector or scalar or not
 */
gluShaderUtil.isDataTypeSampler = function(dataType) {
    return (dataType >= gluShaderUtil.DataType.SAMPLER_2D) && (dataType <= gluShaderUtil.DataType.UINT_SAMPLER_3D);
};

/**
 * Returns a gluShaderUtil.DataType based on given rows and columns
 * @param {number} numCols
 * @param {number} numRows
 * @return {gluShaderUtil.DataType}
 */
gluShaderUtil.getDataTypeMatrix = function(numCols, numRows) {
    if (!(deMath.deInRange32(numCols, 2, 4) && deMath.deInRange32(numRows, 2, 4)))
        throw new Error('Out of bounds: (' + numCols + ',' + numRows + ')');

    var size = numCols.toString() + 'x' + numRows.toString();
    var datatypes = {
        '2x2': gluShaderUtil.DataType.FLOAT_MAT2,
        '2x3': gluShaderUtil.DataType.FLOAT_MAT2X3,
        '2x4': gluShaderUtil.DataType.FLOAT_MAT2X4,
        '3x2': gluShaderUtil.DataType.FLOAT_MAT3X2,
        '3x3': gluShaderUtil.DataType.FLOAT_MAT3,
        '3x4': gluShaderUtil.DataType.FLOAT_MAT3X4,
        '4x2': gluShaderUtil.DataType.FLOAT_MAT4X2,
        '4x3': gluShaderUtil.DataType.FLOAT_MAT4X3,
        '4x4': gluShaderUtil.DataType.FLOAT_MAT4
    };
    return datatypes[size];
};

/**
* Returns number of rows of a gluShaderUtil.DataType Matrix
* @param {gluShaderUtil.DataType} dataType shader
* @return {number} with number of rows depending on gluShaderUtil.DataType Matrix
*/
gluShaderUtil.getDataTypeMatrixNumRows = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT_MAT2: return 2;
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 3;
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 4;
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 2;
        case gluShaderUtil.DataType.FLOAT_MAT3: return 3;
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 4;
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 2;
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 3;
        case gluShaderUtil.DataType.FLOAT_MAT4: return 4;
    }
    throw Error('Unrecognized dataType ' + dataType);
};

/**
* Returns number of columns of a gluShaderUtil.DataType Matrix
* @param {gluShaderUtil.DataType} dataType shader
* @return {number} with number of columns depending on gluShaderUtil.DataType Matrix
*/
gluShaderUtil.getDataTypeMatrixNumColumns = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.FLOAT_MAT2: return 2;
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 2;
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 2;
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 3;
        case gluShaderUtil.DataType.FLOAT_MAT3: return 3;
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 3;
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 4;
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 4;
        case gluShaderUtil.DataType.FLOAT_MAT4: return 4;
    }
    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * @param {gluShaderUtil.DataType} dataType
 * @return {number}
 */
gluShaderUtil.getDataTypeNumLocations = function(dataType) {
    if (gluShaderUtil.isDataTypeScalarOrVector(dataType))
        return 1;
    else if (gluShaderUtil.isDataTypeMatrix(dataType))
        return gluShaderUtil.getDataTypeMatrixNumColumns(dataType);
    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * @param {gluShaderUtil.DataType} dataType
 * @return {number}
 */
gluShaderUtil.getDataTypeNumComponents = function(dataType) {
    if (gluShaderUtil.isDataTypeScalarOrVector(dataType))
        return gluShaderUtil.getDataTypeScalarSize(dataType);
    else if (gluShaderUtil.isDataTypeMatrix(dataType))
        return gluShaderUtil.getDataTypeMatrixNumRows(dataType);

    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * Returns name of the dataType
 * @param {?gluShaderUtil.DataType} dataType shader
 * @return {string} dataType name
 */
gluShaderUtil.getDataTypeName = function(dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.INVALID: return 'invalid';

        case gluShaderUtil.DataType.FLOAT: return 'float';
        case gluShaderUtil.DataType.FLOAT_VEC2: return 'vec2';
        case gluShaderUtil.DataType.FLOAT_VEC3: return 'vec3';
        case gluShaderUtil.DataType.FLOAT_VEC4: return 'vec4';
        case gluShaderUtil.DataType.FLOAT_MAT2: return 'mat2';
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 'mat2x3';
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 'mat2x4';
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 'mat3x2';
        case gluShaderUtil.DataType.FLOAT_MAT3: return 'mat3';
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 'mat3x4';
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 'mat4x2';
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 'mat4x3';
        case gluShaderUtil.DataType.FLOAT_MAT4: return 'mat4';

        case gluShaderUtil.DataType.INT: return 'int';
        case gluShaderUtil.DataType.INT_VEC2: return 'ivec2';
        case gluShaderUtil.DataType.INT_VEC3: return 'ivec3';
        case gluShaderUtil.DataType.INT_VEC4: return 'ivec4';

        case gluShaderUtil.DataType.UINT: return 'uint';
        case gluShaderUtil.DataType.UINT_VEC2: return 'uvec2';
        case gluShaderUtil.DataType.UINT_VEC3: return 'uvec3';
        case gluShaderUtil.DataType.UINT_VEC4: return 'uvec4';

        case gluShaderUtil.DataType.BOOL: return 'bool';
        case gluShaderUtil.DataType.BOOL_VEC2: return 'bvec2';
        case gluShaderUtil.DataType.BOOL_VEC3: return 'bvec3';
        case gluShaderUtil.DataType.BOOL_VEC4: return 'bvec4';

        case gluShaderUtil.DataType.SAMPLER_2D: return 'sampler2D';
        case gluShaderUtil.DataType.SAMPLER_CUBE: return 'samplerCube';
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY: return 'sampler2DArray';
        case gluShaderUtil.DataType.SAMPLER_3D: return 'sampler3D';

        case gluShaderUtil.DataType.SAMPLER_2D_SHADOW: return 'sampler2DShadow';
        case gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW: return 'samplerCubeShadow';
        case gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW: return 'sampler2DArrayShadow';

        case gluShaderUtil.DataType.INT_SAMPLER_2D: return 'isampler2D';
        case gluShaderUtil.DataType.INT_SAMPLER_CUBE: return 'isamplerCube';
        case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY: return 'isampler2DArray';
        case gluShaderUtil.DataType.INT_SAMPLER_3D: return 'isampler3D';

        case gluShaderUtil.DataType.UINT_SAMPLER_2D: return 'usampler2D';
        case gluShaderUtil.DataType.UINT_SAMPLER_CUBE: return 'usamplerCube';
        case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY: return 'usampler2DArray';
        case gluShaderUtil.DataType.UINT_SAMPLER_3D: return 'usampler3D';
    }
    throw Error('Unrecognized dataType ' + dataType);
};

/**
 * Returns the gluShaderUtil.DataType from the GL type
 * @param {number} glType
 * @return {gluShaderUtil.DataType}
 */
gluShaderUtil.getDataTypeFromGLType = function(glType) {
    switch (glType) {
        case gl.FLOAT: return gluShaderUtil.DataType.FLOAT;
        case gl.FLOAT_VEC2: return gluShaderUtil.DataType.FLOAT_VEC2;
        case gl.FLOAT_VEC3: return gluShaderUtil.DataType.FLOAT_VEC3;
        case gl.FLOAT_VEC4: return gluShaderUtil.DataType.FLOAT_VEC4;

        case gl.FLOAT_MAT2: return gluShaderUtil.DataType.FLOAT_MAT2;
        case gl.FLOAT_MAT2x3: return gluShaderUtil.DataType.FLOAT_MAT2X3;
        case gl.FLOAT_MAT2x4: return gluShaderUtil.DataType.FLOAT_MAT2X4;

        case gl.FLOAT_MAT3x2: return gluShaderUtil.DataType.FLOAT_MAT3X2;
        case gl.FLOAT_MAT3: return gluShaderUtil.DataType.FLOAT_MAT3;
        case gl.FLOAT_MAT3x4: return gluShaderUtil.DataType.FLOAT_MAT3X4;

        case gl.FLOAT_MAT4x2: return gluShaderUtil.DataType.FLOAT_MAT4X2;
        case gl.FLOAT_MAT4x3: return gluShaderUtil.DataType.FLOAT_MAT4X3;
        case gl.FLOAT_MAT4: return gluShaderUtil.DataType.FLOAT_MAT4;

        case gl.INT: return gluShaderUtil.DataType.INT;
        case gl.INT_VEC2: return gluShaderUtil.DataType.INT_VEC2;
        case gl.INT_VEC3: return gluShaderUtil.DataType.INT_VEC3;
        case gl.INT_VEC4: return gluShaderUtil.DataType.INT_VEC4;

        case gl.UNSIGNED_INT: return gluShaderUtil.DataType.UINT;
        case gl.UNSIGNED_INT_VEC2: return gluShaderUtil.DataType.UINT_VEC2;
        case gl.UNSIGNED_INT_VEC3: return gluShaderUtil.DataType.UINT_VEC3;
        case gl.UNSIGNED_INT_VEC4: return gluShaderUtil.DataType.UINT_VEC4;

        case gl.BOOL: return gluShaderUtil.DataType.BOOL;
        case gl.BOOL_VEC2: return gluShaderUtil.DataType.BOOL_VEC2;
        case gl.BOOL_VEC3: return gluShaderUtil.DataType.BOOL_VEC3;
        case gl.BOOL_VEC4: return gluShaderUtil.DataType.BOOL_VEC4;

        case gl.SAMPLER_2D: return gluShaderUtil.DataType.SAMPLER_2D;
        case gl.SAMPLER_CUBE: return gluShaderUtil.DataType.SAMPLER_CUBE;
        case gl.SAMPLER_2D_ARRAY: return gluShaderUtil.DataType.SAMPLER_2D_ARRAY;
        case gl.SAMPLER_3D: return gluShaderUtil.DataType.SAMPLER_3D;

        case gl.SAMPLER_2D_SHADOW: return gluShaderUtil.DataType.SAMPLER_2D_SHADOW;
        case gl.SAMPLER_CUBE_SHADOW: return gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW;
        case gl.SAMPLER_2D_ARRAY_SHADOW: return gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW;

        case gl.INT_SAMPLER_2D: return gluShaderUtil.DataType.INT_SAMPLER_2D;
        case gl.INT_SAMPLER_CUBE: return gluShaderUtil.DataType.INT_SAMPLER_CUBE;
        case gl.INT_SAMPLER_2D_ARRAY: return gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY;
        case gl.INT_SAMPLER_3D: return gluShaderUtil.DataType.INT_SAMPLER_3D;

        case gl.UNSIGNED_INT_SAMPLER_2D: return gluShaderUtil.DataType.UINT_SAMPLER_2D;
        case gl.UNSIGNED_INT_SAMPLER_CUBE: return gluShaderUtil.DataType.UINT_SAMPLER_CUBE;
        case gl.UNSIGNED_INT_SAMPLER_2D_ARRAY: return gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY;
        case gl.UNSIGNED_INT_SAMPLER_3D: return gluShaderUtil.DataType.UINT_SAMPLER_3D;

        default:
            throw new Error('Unrecognized GL type:' + glType);
    }
};

});

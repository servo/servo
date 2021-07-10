/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the 'License');
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an 'AS IS' BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fShaderStateQueryTests');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fShaderStateQueryTests = functional.gles3.es3fShaderStateQueryTests;
var tcuTestCase = framework.common.tcuTestCase;
var glsStateQuery = modules.shared.glsStateQuery;
var es3fApiCase = functional.gles3.es3fApiCase;
var deRandom = framework.delibs.debase.deRandom;
var tcuMatrix = framework.common.tcuMatrix;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

var commonTestVertSource = '#version 300 es\n' +
                            'void main (void)\n' +
                            '{\n' +
                            ' gl_Position = vec4(0.0);\n' +
                            '}\n';
var commonTestFragSource = '#version 300 es\n' +
                            'layout(location = 0) out mediump vec4 fragColor;\n' +
                            'void main (void)\n' +
                            '{\n' +
                            ' fragColor = vec4(0.0);\n' +
                            '}\n';

var brokenShader = '#version 300 es\n' +
                            'broken, this should not compile!\n' +
                            '\n';

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ShaderTypeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ShaderTypeCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ShaderTypeCase.prototype.test = function() {
    var shaderTypes = [gl.VERTEX_SHADER, gl.FRAGMENT_SHADER];
    for (var ndx = 0; ndx < shaderTypes.length; ++ndx) {
        var shader = gl.createShader(shaderTypes[ndx]);
        var result = glsStateQuery.verifyShader(shader, gl.SHADER_TYPE, shaderTypes[ndx]);
        this.check(result, 'Incorrect shader type');
        gl.deleteShader(shader);
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ShaderCompileStatusCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ShaderCompileStatusCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ShaderCompileStatusCase.prototype.test = function() {
    var result;
    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    result = glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, false);
    this.check(result, 'Vertex shader compilation status should be false');
    result = glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, false);
    this.check(result, 'Fragment shader compilation status should be false');

    gl.shaderSource(shaderVert, commonTestVertSource);
    gl.shaderSource(shaderFrag, commonTestFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    result = glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true);
    this.check(result, 'Vertex shader compilation status should be true');
    result = glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, true);
    this.check(result, 'Fragment shader compilation status should be true');

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ShaderInfoLogCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ShaderInfoLogCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ShaderInfoLogCase.prototype.test = function() {
    var shader = gl.createShader(gl.VERTEX_SHADER);
    var log = gl.getShaderInfoLog(shader);
    this.check(log === '');

    gl.shaderSource(shader, brokenShader);
    gl.compileShader(shader);

    log = gl.getShaderInfoLog(shader);
    this.check(log === null || typeof log === 'string');

    gl.deleteShader(shader);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ShaderSourceCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ShaderSourceCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ShaderSourceCase.prototype.test = function() {
    var shader = gl.createShader(gl.VERTEX_SHADER);
    this.check(gl.getShaderSource(shader) === '');

    gl.shaderSource(shader, brokenShader);
    this.check(gl.getShaderSource(shader) === brokenShader);

    gl.deleteShader(shader);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.DeleteStatusCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.DeleteStatusCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.DeleteStatusCase.prototype.test = function() {
    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, commonTestVertSource);
    gl.shaderSource(shaderFrag, commonTestFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    this.check(glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, true));

    var shaderProg = gl.createProgram();
    gl.attachShader(shaderProg, shaderVert);
    gl.attachShader(shaderProg, shaderFrag);
    gl.linkProgram(shaderProg);

    this.check(glsStateQuery.verifyProgram(shaderProg, gl.LINK_STATUS, true));

    this.check(glsStateQuery.verifyShader(shaderVert, gl.DELETE_STATUS, false));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.DELETE_STATUS, false));
    this.check(glsStateQuery.verifyProgram(shaderProg, gl.DELETE_STATUS, false));

    gl.useProgram(shaderProg);

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(shaderProg);

    this.check(glsStateQuery.verifyShader(shaderVert, gl.DELETE_STATUS, true));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.DELETE_STATUS, true));
    this.check(glsStateQuery.verifyProgram(shaderProg, gl.DELETE_STATUS, true));

    gl.useProgram(null);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.CurrentVertexAttribInitialCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.CurrentVertexAttribInitialCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.CurrentVertexAttribInitialCase.prototype.test = function() {
    var attribute_count = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
    var initial = new Float32Array([0, 0, 0, 1]);
    // initial

    for (var index = 0; index < attribute_count; ++index) {
        var attrib = gl.getVertexAttrib(index, gl.CURRENT_VERTEX_ATTRIB);
        this.check(glsStateQuery.compare(attrib, initial), 'Initial attrib value should be [0, 0, 0, 1]');
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.CurrentVertexAttribFloatCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.CurrentVertexAttribFloatCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.CurrentVertexAttribFloatCase.prototype.test = function() {
    var rnd = new deRandom.Random(0xabcdef);

    var attribute_count = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));

    // test write float/read float

    for (var index = 0; index < attribute_count; ++index) {
        var x = rnd.getFloat(-64000, 64000);
        var y = rnd.getFloat(-64000, 64000);
        var z = rnd.getFloat(-64000, 64000);
        var w = rnd.getFloat(-64000, 64000);

        gl.vertexAttrib4f(index, x, y, z, w);
        this.check(glsStateQuery.verifyCurrentVertexAttrib(index, new Float32Array([x, y, z, w])));
    }
    for (var index = 0; index < attribute_count; ++index) {
        var x = rnd.getFloat(-64000, 64000);
        var y = rnd.getFloat(-64000, 64000);
        var z = rnd.getFloat(-64000, 64000);
        var w = 1.0;

        gl.vertexAttrib3f(index, x, y, z);
        this.check(glsStateQuery.verifyCurrentVertexAttrib(index, new Float32Array([x, y, z, w])));
    }
    for (var index = 0; index < attribute_count; ++index) {
        var x = rnd.getFloat(-64000, 64000);
        var y = rnd.getFloat(-64000, 64000);
        var z = 0.0;
        var w = 1.0;

        gl.vertexAttrib2f(index, x, y);
        this.check(glsStateQuery.verifyCurrentVertexAttrib(index, new Float32Array([x, y, z, w])));
    }
    for (var index = 0; index < attribute_count; ++index) {
        var x = rnd.getFloat(-64000, 64000);
        var y = 0.0;
        var z = 0.0;
        var w = 1.0;

        gl.vertexAttrib1f(index, x);
        this.check(glsStateQuery.verifyCurrentVertexAttrib(index, new Float32Array([x, y, z, w])));
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.CurrentVertexAttribIntCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.CurrentVertexAttribIntCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.CurrentVertexAttribIntCase.prototype.test = function() {
    var rnd = new deRandom.Random(0xabcdef);

    var attribute_count = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));

    // test write float/read float

    for (var index = 0; index < attribute_count; ++index) {
        var x = rnd.getInt(-64000, 64000);
        var y = rnd.getInt(-64000, 64000);
        var z = rnd.getInt(-64000, 64000);
        var w = rnd.getInt(-64000, 64000);

        gl.vertexAttribI4i(index, x, y, z, w);
        this.check(glsStateQuery.verifyCurrentVertexAttrib(index, new Int32Array([x, y, z, w])));
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.CurrentVertexAttribUintCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.CurrentVertexAttribUintCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.CurrentVertexAttribUintCase.prototype.test = function() {
    var rnd = new deRandom.Random(0xabcdef);

    var attribute_count = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));

    // test write float/read float

    for (var index = 0; index < attribute_count; ++index) {
        var x = rnd.getInt(0, 64000);
        var y = rnd.getInt(0, 64000);
        var z = rnd.getInt(0, 64000);
        var w = rnd.getInt(0, 64000);

        gl.vertexAttribI4ui(index, x, y, z, w);
        this.check(glsStateQuery.verifyCurrentVertexAttrib(index, new Uint32Array([x, y, z, w])));
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ProgramInfoLogCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ProgramInfoLogCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ProgramInfoLogCase.prototype.test = function() {
    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, brokenShader);
    gl.compileShader(shaderVert);
    gl.shaderSource(shaderFrag, brokenShader);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);

    var log = gl.getProgramInfoLog(program);
    this.check(log === null || typeof log === 'string');

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ProgramValidateStatusCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ProgramValidateStatusCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ProgramValidateStatusCase.prototype.test = function() {
    // test validate ok
    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, commonTestVertSource);
    gl.shaderSource(shaderFrag, commonTestFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);

    this.check(glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyProgram(program, gl.LINK_STATUS, true));

    gl.validateProgram(program);
    this.check(glsStateQuery.verifyProgram(program, gl.VALIDATE_STATUS, true));

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);

    // test with broken shader
    shaderVert = gl.createShader(gl.VERTEX_SHADER);
    shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, commonTestVertSource);
    gl.shaderSource(shaderFrag, brokenShader);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);

    this.check(glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, false));
    this.check(glsStateQuery.verifyProgram(program, gl.LINK_STATUS, false));

    gl.validateProgram(program);
    this.check(glsStateQuery.verifyProgram(program, gl.VALIDATE_STATUS, false));

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ProgramAttachedShadersCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ProgramAttachedShadersCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ProgramAttachedShadersCase.prototype.test = function() {
    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, commonTestVertSource);
    gl.shaderSource(shaderFrag, commonTestFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    // check ATTACHED_SHADERS

    var program = gl.createProgram();
    this.check(glsStateQuery.verifyProgram(program, gl.ATTACHED_SHADERS, 0));

    gl.attachShader(program, shaderVert);
    this.check(glsStateQuery.verifyProgram(program, gl.ATTACHED_SHADERS, 1));

    gl.attachShader(program, shaderFrag);
    this.check(glsStateQuery.verifyProgram(program, gl.ATTACHED_SHADERS, 2));

    // check GetAttachedShaders
    var shaders = gl.getAttachedShaders(program);
    this.check(glsStateQuery.compare(shaders, [shaderVert, shaderFrag]));

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ProgramActiveUniformNameCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ProgramActiveUniformNameCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ProgramActiveUniformNameCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform highp float uniformNameWithLength23;\n' +
        'uniform highp vec2 uniformVec2;\n' +
        'uniform highp mat4 uniformMat4;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(0.0) + vec4(uniformNameWithLength23) + vec4(uniformVec2.x) + vec4(uniformMat4[2][3]);\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);

    this.check(glsStateQuery.verifyProgram(program, gl.ACTIVE_UNIFORMS, 3));

    var uniformNames = [
        'uniformNameWithLength23',
        'uniformVec2',
        'uniformMat4'
    ];

    var indices = gl.getUniformIndices(program, uniformNames);

    // check names
    for (var ndx = 0; ndx < uniformNames.length; ++ndx) {
        var index = indices[ndx];
        var uniform = gl.getActiveUniform(program, index);

        this.check(glsStateQuery.compare(uniform.name, uniformNames[ndx]));
    }

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);

};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ProgramUniformCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ProgramUniformCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ProgramUniformCase.prototype.test = function() {
    var uniformTypes = [
        ['float', '', 'highp', '', 'uniformValue', gl.FLOAT, 1, false],
        ['float[2]', '', 'highp', '', 'uniformValue[1]', gl.FLOAT, 2, false],
        ['vec2', '', 'highp', '', 'uniformValue.x', gl.FLOAT_VEC2, 1, false],
        ['vec3', '', 'highp', '', 'uniformValue.x', gl.FLOAT_VEC3, 1, false],
        ['vec4', '', 'highp', '', 'uniformValue.x', gl.FLOAT_VEC4, 1, false],
        ['int', '', 'highp', '', 'float(uniformValue)', gl.INT, 1, false],
        ['ivec2', '', 'highp', '', 'float(uniformValue.x)', gl.INT_VEC2, 1, false],
        ['ivec3', '', 'highp', '', 'float(uniformValue.x)', gl.INT_VEC3, 1, false],
        ['ivec4', '', 'highp', '', 'float(uniformValue.x)', gl.INT_VEC4, 1, false],
        ['uint', '', 'highp', '', 'float(uniformValue)', gl.UNSIGNED_INT, 1, false],
        ['uvec2', '', 'highp', '', 'float(uniformValue.x)', gl.UNSIGNED_INT_VEC2, 1, false],
        ['uvec3', '', 'highp', '', 'float(uniformValue.x)', gl.UNSIGNED_INT_VEC3, 1, false],
        ['uvec4', '', 'highp', '', 'float(uniformValue.x)', gl.UNSIGNED_INT_VEC4, 1, false],
        ['bool', '', '', '', 'float(uniformValue)', gl.BOOL, 1, false],
        ['bvec2', '', '', '', 'float(uniformValue.x)', gl.BOOL_VEC2, 1, false],
        ['bvec3', '', '', '', 'float(uniformValue.x)', gl.BOOL_VEC3, 1, false],
        ['bvec4', '', '', '', 'float(uniformValue.x)', gl.BOOL_VEC4, 1, false],
        ['mat2', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT2, 1, false],
        ['mat3', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT3, 1, false],
        ['mat4', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT4, 1, false],
        ['mat2x3', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT2x3, 1, false],
        ['mat2x4', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT2x4, 1, false],
        ['mat3x2', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT3x2, 1, false],
        ['mat3x4', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT3x4, 1, false],
        ['mat4x2', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT4x2, 1, false],
        ['mat4x3', '', 'highp', '', 'float(uniformValue[0][0])', gl.FLOAT_MAT4x3, 1, false],
        ['sampler2D', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_2D, 1, false],
        ['sampler3D', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_3D, 1, false],
        ['samplerCube', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_CUBE, 1, false],
        ['sampler2DShadow', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_2D_SHADOW, 1, false],
        ['sampler2DArray', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_2D_ARRAY, 1, false],
        ['sampler2DArrayShadow', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_2D_ARRAY_SHADOW, 1, false],
        ['samplerCubeShadow', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.SAMPLER_CUBE_SHADOW, 1, false],
        ['isampler2D', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.INT_SAMPLER_2D, 1, false],
        ['isampler3D', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.INT_SAMPLER_3D, 1, false],
        ['isamplerCube', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.INT_SAMPLER_CUBE, 1, false],
        ['isampler2DArray', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.INT_SAMPLER_2D_ARRAY, 1, false],
        ['usampler2D', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.UNSIGNED_INT_SAMPLER_2D, 1, false],
        ['usampler3D', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.UNSIGNED_INT_SAMPLER_3D, 1, false],
        ['usamplerCube', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.UNSIGNED_INT_SAMPLER_CUBE, 1, false],
        ['usampler2DArray', '', 'highp', '', 'float(textureSize(uniformValue,0).r)', gl.UNSIGNED_INT_SAMPLER_2D_ARRAY, 1, false]
    ];

    var vertSource =
        '#version 300 es\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);
    var program = gl.createProgram();

    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);

    gl.shaderSource(shaderVert, vertSource);
    gl.compileShader(shaderVert);

    for (var ndx = 0; ndx < uniformTypes.length; ++ndx) {
        var declaration = uniformTypes[ndx][0];
        var postDeclaration = uniformTypes[ndx][1];
        var precision = uniformTypes[ndx][2];
        var layout = uniformTypes[ndx][3];
        var getter = uniformTypes[ndx][4];
        var type = uniformTypes[ndx][5];
        var size = uniformTypes[ndx][6];
        var isRowMajor = uniformTypes[ndx][7];
        bufferedLogToConsole('Verify type of ' + declaration + ' variable' + postDeclaration);

        // gen fragment shader

        var frag = '';
        frag += '#version 300 es\n';
        frag += layout + 'uniform ' + precision + ' ' + declaration + ' uniformValue' + postDeclaration + ';\n';
        frag += 'layout(location = 0) out mediump vec4 fragColor;\n';
        frag += 'void main (void)\n';
        frag += '{\n';
        frag += ' fragColor = vec4(' + getter + ');\n';
        frag += '}\n';

        gl.shaderSource(shaderFrag, frag);

        // compile & link

        gl.compileShader(shaderFrag);
        gl.linkProgram(program);

        // test
        if (this.check(glsStateQuery.verifyProgram(program, gl.LINK_STATUS, true), 'Program link fail' + gl.getProgramInfoLog(program))) {
            var indices = gl.getUniformIndices(program, ['uniformValue']);
            var info_type = gl.getActiveUniforms(program, indices, gl.UNIFORM_TYPE)[0];
            var info_size = gl.getActiveUniforms(program, indices, gl.UNIFORM_SIZE)[0];
            var info_is_row_major = gl.getActiveUniforms(program, indices, gl.UNIFORM_IS_ROW_MAJOR)[0];
            this.check(glsStateQuery.compare(info_size, size));
            this.check(glsStateQuery.compare(info_type, type));
            this.check(glsStateQuery.compare(info_is_row_major, isRowMajor));
        }
    }

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ProgramActiveUniformBlocksCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ProgramActiveUniformBlocksCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ProgramActiveUniformBlocksCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform longlongUniformBlockName {highp vec2 vector2;} longlongUniformInstanceName;\n' +
        'uniform shortUniformBlockName {highp vec2 vector2;highp vec4 vector4;} shortUniformInstanceName;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = shortUniformInstanceName.vector4 + vec4(longlongUniformInstanceName.vector2.x) + vec4(shortUniformInstanceName.vector2.x);\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'uniform longlongUniformBlockName {highp vec2 vector2;} longlongUniformInstanceName;\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(longlongUniformInstanceName.vector2.y);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);

    this.check(glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyProgram(program, gl.LINK_STATUS, true));

    this.check(glsStateQuery.verifyProgram(program, gl.ACTIVE_UNIFORM_BLOCKS, 2));

    var longlongUniformBlockIndex = gl.getUniformBlockIndex(program, 'longlongUniformBlockName');
    var shortUniformBlockIndex = gl.getUniformBlockIndex(program, 'shortUniformBlockName');

    var uniformNames = [
        'longlongUniformBlockName.vector2',
        'shortUniformBlockName.vector2',
        'shortUniformBlockName.vector4'
    ];

    // test UNIFORM_BLOCK_INDEX

    var uniformIndices = gl.getUniformIndices(program, uniformNames);

    var uniformsBlockIndices = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_BLOCK_INDEX);
    this.check(uniformsBlockIndices[0] == longlongUniformBlockIndex &&
        uniformsBlockIndices[1] == shortUniformBlockIndex &&
        uniformsBlockIndices[2] == shortUniformBlockIndex,
        'Expected [' + longlongUniformBlockIndex + ", " + shortUniformBlockIndex + ", " + shortUniformBlockIndex + ']; got ' +
        uniformsBlockIndices[0] + ", " + uniformsBlockIndices[1] + ", " + uniformsBlockIndices[2] + "]");

    // test UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER & UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER

    this.check(glsStateQuery.verifyActiveUniformBlock(program, longlongUniformBlockIndex, gl.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER, true));
    this.check(glsStateQuery.verifyActiveUniformBlock(program, longlongUniformBlockIndex, gl.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER, true));
    this.check(glsStateQuery.verifyActiveUniformBlock(program, shortUniformBlockIndex, gl.UNIFORM_BLOCK_REFERENCED_BY_VERTEX_SHADER, true));
    this.check(glsStateQuery.verifyActiveUniformBlock(program, shortUniformBlockIndex, gl.UNIFORM_BLOCK_REFERENCED_BY_FRAGMENT_SHADER, false));

    // test UNIFORM_BLOCK_ACTIVE_UNIFORMS

    this.check(glsStateQuery.verifyActiveUniformBlock(program, longlongUniformBlockIndex, gl.UNIFORM_BLOCK_ACTIVE_UNIFORMS, 1));
    this.check(glsStateQuery.verifyActiveUniformBlock(program, shortUniformBlockIndex, gl.UNIFORM_BLOCK_ACTIVE_UNIFORMS, 2));

    // test UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES

    var shortUniformBlockIndices = gl.getActiveUniformBlockParameter(program, shortUniformBlockIndex, gl.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES);
    this.check(shortUniformBlockIndices.length == 2, 'Expected 2 indices; got ' + shortUniformBlockIndices.length);

    this.check(glsStateQuery.compare(shortUniformBlockIndices, new Uint32Array([uniformIndices[1], uniformIndices[2]])) ||
               glsStateQuery.compare(shortUniformBlockIndices, new Uint32Array([uniformIndices[2], uniformIndices[1]])),
                'Expected { ' + uniformIndices[1] +', ' + uniformIndices[2] +
                '}; got {' + shortUniformBlockIndices[0] + ', ' + shortUniformBlockIndices[1] + '}');

    // check block names

    var name = gl.getActiveUniformBlockName(program, longlongUniformBlockIndex);
    this.check(name == "longlongUniformBlockName", 'Wrong uniform block name, expected longlongUniformBlockName; got ' + name);
    name = gl.getActiveUniformBlockName(program, shortUniformBlockIndex)
    this.check(name == "shortUniformBlockName", 'Wrong uniform block name, expected shortUniformBlockName; got ' + name);

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.TransformFeedbackCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.TransformFeedbackCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.TransformFeedbackCase.prototype.test = function() {
    var transformFeedbackTestVertSource =
        '#version 300 es\n' +
        'out highp vec4 tfOutput2withLongName;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(0.0);\n' +
        ' tfOutput2withLongName = vec4(0.0);\n' +
        '}\n';
    var transformFeedbackTestFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out highp vec4 fragColor;\n' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);
    var shaderProg = gl.createProgram();

    this.check(glsStateQuery.verifyProgram(shaderProg, gl.TRANSFORM_FEEDBACK_BUFFER_MODE, gl.INTERLEAVED_ATTRIBS));

    gl.shaderSource(shaderVert, transformFeedbackTestVertSource);
    gl.shaderSource(shaderFrag, transformFeedbackTestFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    this.check(glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true));
    this.check(glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, true));

    gl.attachShader(shaderProg, shaderVert);
    gl.attachShader(shaderProg, shaderFrag);

    // check TRANSFORM_FEEDBACK_BUFFER_MODE

    var transform_feedback_outputs = ['gl_Position', 'tfOutput2withLongName'];
    var bufferModes = [gl.SEPARATE_ATTRIBS, gl.INTERLEAVED_ATTRIBS];

    for (var ndx = 0; ndx < bufferModes.length; ++ndx) {
        gl.transformFeedbackVaryings(shaderProg, transform_feedback_outputs, bufferModes[ndx]);
        gl.linkProgram(shaderProg);

        this.check(glsStateQuery.verifyProgram(shaderProg, gl.LINK_STATUS, true));
        this.check(glsStateQuery.verifyProgram(shaderProg, gl.TRANSFORM_FEEDBACK_BUFFER_MODE, bufferModes[ndx]));
    }

    // check varyings
    var varyings = /** @type {number} */ (gl.getProgramParameter(shaderProg, gl.TRANSFORM_FEEDBACK_VARYINGS));
    this.check(varyings === 2);

    for (var index = 0; index < varyings; ++index) {
        var info = gl.getTransformFeedbackVarying(shaderProg, index);
        this.check(glsStateQuery.compare(info.type, gl.FLOAT_VEC4));
        this.check(glsStateQuery.compare(info.size, 1));
        this.check(glsStateQuery.compare(info.name, transform_feedback_outputs[index]));
    }

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(shaderProg);

    // TODO(kbr): this test is failing and leaving an error in the GL
    // state, causing later tests to fail. Clear the error state for
    // the time being.
    while (gl.getError() != gl.NO_ERROR) {}
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.ActiveAttributesCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.ActiveAttributesCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.ActiveAttributesCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'in highp vec2 longInputAttributeName;\n' +
        'in highp vec2 shortName;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = longInputAttributeName.yxxy + shortName.xyxy;\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);

    this.check(glsStateQuery.verifyProgram(program, gl.ACTIVE_ATTRIBUTES, 2));

    var attribNames = [
        'longInputAttributeName',
        'shortName'
    ];
    // check names
    for (var attributeNdx = 0; attributeNdx < 2; ++attributeNdx) {
        var info = gl.getActiveAttrib(program, attributeNdx);
        this.check(glsStateQuery.compare(info.name, attribNames[0]) || glsStateQuery.compare(info.name, attribNames[1]));
    }

    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeSizeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeSizeCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeSizeCase.prototype.test = function() {
    var pointers = [
        // size test
        [4, gl.FLOAT, 0, false, 0],
        [3, gl.FLOAT, 0, false, 0],
        [2, gl.FLOAT, 0, false, 0],
        [1, gl.FLOAT, 0, false, 0],
        [4, gl.INT, 0, false, 0],
        [3, gl.INT, 0, false, 0],
        [2, gl.INT, 0, false, 0],
        [1, gl.INT, 0, false, 0]
    ];

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);

    // Test with default VAO

    for (var ndx = 0; ndx < pointers.length; ++ndx) {
        gl.vertexAttribPointer(0, pointers[ndx][0], pointers[ndx][1], pointers[ndx][3], pointers[ndx][2], pointers[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_SIZE, pointers[ndx][0]));
    }

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_SIZE, 4));

    // set vao 0 to some value
    gl.vertexAttribPointer(0, pointers[0][0], pointers[0][1], pointers[0][3], pointers[0][2], 0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribPointer(0, pointers[1][0], pointers[1][1], pointers[1][3], pointers[1][2], 0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_SIZE, pointers[1][0]));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_SIZE, pointers[0][0]));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buf);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeTypeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeTypeCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeTypeCase.prototype.test = function() {
    var pointers = [
        // type test
        [1, gl.BYTE, 0, false, 0],
        [1, gl.SHORT, 0, false, 0],
        [1, gl.INT, 0, false, 0],
        [1, gl.FLOAT, 0, false, 0],
        [1, gl.HALF_FLOAT, 0, false, 0],
        [1, gl.UNSIGNED_BYTE, 0, false, 0],
        [1, gl.UNSIGNED_SHORT, 0, false, 0],
        [1, gl.UNSIGNED_INT, 0, false, 0],
        [4, gl.INT_2_10_10_10_REV, 0, false, 0],
        [4, gl.UNSIGNED_INT_2_10_10_10_REV, 0, false, 0]
    ];

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);

    // Test with default VAO

    for (var ndx = 0; ndx < pointers.length; ++ndx) {
        gl.vertexAttribPointer(0, pointers[ndx][0], pointers[ndx][1], pointers[ndx][3], pointers[ndx][2], pointers[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_TYPE, pointers[ndx][1]));
    }

    var pointersI = [
        [1, gl.BYTE, 0, false, 0],
        [1, gl.SHORT, 0, false, 0],
        [1, gl.INT, 0, false, 0],
        [1, gl.UNSIGNED_BYTE, 0, false, 0],
        [1, gl.UNSIGNED_SHORT, 0, false, 0],
        [1, gl.UNSIGNED_INT, 0, false, 0]
    ];

    for (var ndx = 0; ndx < pointersI.length; ++ndx) {
        gl.vertexAttribIPointer(0, pointersI[ndx][0], pointersI[ndx][1], pointersI[ndx][2], pointersI[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_TYPE, pointersI[ndx][1]));
    }

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_TYPE, gl.FLOAT));

    // set vao 0 to some value
    gl.vertexAttribPointer(0, 1, gl.FLOAT, false, 0, 0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribPointer(0, 1, gl.SHORT, false, 0, 0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_TYPE, gl.SHORT));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_TYPE, gl.FLOAT));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buf);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeStrideCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeStrideCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeStrideCase.prototype.test = function() {
    var pointers = [
        [1, gl.FLOAT, 0, 0, gl.NO_ERROR],
        [1, gl.FLOAT, 1, 0, gl.INVALID_OPERATION],
        [1, gl.FLOAT, 4, 0, gl.NO_ERROR],
        [1, gl.HALF_FLOAT, 0, 0, gl.NO_ERROR],
        [1, gl.HALF_FLOAT, 1, 0, gl.INVALID_OPERATION],
        [1, gl.HALF_FLOAT, 4, 0, gl.NO_ERROR]
    ];

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);

    // Test with default VAO

    for (var ndx = 0; ndx < pointers.length; ++ndx) {
        gl.vertexAttribPointer(0, pointers[ndx][0], pointers[ndx][1], false, pointers[ndx][2], pointers[ndx][3]);
        this.expectError(pointers[ndx][4]);
        if (pointers[ndx][4] == gl.NO_ERROR) {
            this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_STRIDE, pointers[ndx][2]));
        }
    }

    var pointersI = [
        [1, gl.INT, 0, 0, gl.NO_ERROR],
        [1, gl.INT, 1, 0, gl.INVALID_OPERATION],
        [1, gl.INT, 4, 0, gl.NO_ERROR],
        [4, gl.UNSIGNED_BYTE, 0, 0, gl.NO_ERROR],
        [4, gl.UNSIGNED_BYTE, 1, 0, gl.NO_ERROR],
        [4, gl.UNSIGNED_BYTE, 4, 0, gl.NO_ERROR],
        [2, gl.SHORT, 0, 0, gl.NO_ERROR],
        [2, gl.SHORT, 1, 0, gl.INVALID_OPERATION],
        [2, gl.SHORT, 4, 0, gl.NO_ERROR]
    ];

    for (var ndx = 0; ndx < pointersI.length; ++ndx) {
        gl.vertexAttribIPointer(0, pointersI[ndx][0], pointersI[ndx][1], pointersI[ndx][2], pointersI[ndx][3]);
        this.expectError(pointersI[ndx][4]);
        if (pointersI[ndx][4] == gl.NO_ERROR) {
            this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_STRIDE, pointersI[ndx][2]));
        }
    }

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_STRIDE, 0));

    // set vao 0 to some value
    gl.vertexAttribPointer(0, 1, gl.FLOAT, false, 4, 0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribPointer(0, 1, gl.SHORT, false, 8, 0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_STRIDE, 8));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_STRIDE, 4));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buf);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeNormalizedCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeNormalizedCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeNormalizedCase.prototype.test = function() {
    var pointers = [
        // type test
        [1, gl.BYTE, 0, false, 0],
        [1, gl.SHORT, 0, false, 0],
        [1, gl.INT, 0, false, 0],
        [1, gl.FLOAT, 0, false, 0],
        [1, gl.HALF_FLOAT, 0, false, 0],
        [1, gl.UNSIGNED_BYTE, 0, false, 0],
        [1, gl.UNSIGNED_SHORT, 0, false, 0],
        [1, gl.UNSIGNED_INT, 0, false, 0],
        [4, gl.INT_2_10_10_10_REV, 0, false, 0],
        [4, gl.UNSIGNED_INT_2_10_10_10_REV, 0, false, 0],
        [1, gl.BYTE, 0, true, 0],
        [1, gl.SHORT, 0, true, 0],
        [1, gl.INT, 0, true, 0],
        [1, gl.FLOAT, 0, true, 0],
        [1, gl.HALF_FLOAT, 0, true, 0],
        [1, gl.UNSIGNED_BYTE, 0, true, 0],
        [1, gl.UNSIGNED_SHORT, 0, true, 0],
        [1, gl.UNSIGNED_INT, 0, true, 0],
        [4, gl.INT_2_10_10_10_REV, 0, true, 0],
        [4, gl.UNSIGNED_INT_2_10_10_10_REV, 0, true, 0]
    ];

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);

    // Test with default VAO

    for (var ndx = 0; ndx < pointers.length; ++ndx) {
        gl.vertexAttribPointer(0, pointers[ndx][0], pointers[ndx][1], pointers[ndx][3], pointers[ndx][2], pointers[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_NORMALIZED, pointers[ndx][3]));
    }

    var pointersI = [
        [1, gl.BYTE, 0, false, 0],
        [1, gl.SHORT, 0, false, 0],
        [1, gl.INT, 0, false, 0],
        [1, gl.UNSIGNED_BYTE, 0, false, 0],
        [1, gl.UNSIGNED_SHORT, 0, false, 0],
        [1, gl.UNSIGNED_INT, 0, false, 0]
    ];

    for (var ndx = 0; ndx < pointersI.length; ++ndx) {
        gl.vertexAttribIPointer(0, pointersI[ndx][0], pointersI[ndx][1], pointersI[ndx][2], pointersI[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_NORMALIZED, false));
    }

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_NORMALIZED, false));

    // set vao 0 to some value
    gl.vertexAttribPointer(0, 1, gl.INT, true, 0, 0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribPointer(0, 1, gl.INT, false, 0, 0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_NORMALIZED, false));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_NORMALIZED, true));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buf);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeIntegerCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeIntegerCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeIntegerCase.prototype.test = function() {
    var pointers = [
        // type test
        [1, gl.BYTE, 0, false, 0],
        [1, gl.SHORT, 0, false, 0],
        [1, gl.INT, 0, false, 0],
        [1, gl.FLOAT, 0, false, 0],
        [1, gl.HALF_FLOAT, 0, false, 0],
        [1, gl.UNSIGNED_BYTE, 0, false, 0],
        [1, gl.UNSIGNED_SHORT, 0, false, 0],
        [1, gl.UNSIGNED_INT, 0, false, 0],
        [4, gl.INT_2_10_10_10_REV, 0, false, 0],
        [4, gl.UNSIGNED_INT_2_10_10_10_REV, 0, false, 0]
    ];

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);

    // Test with default VAO

    for (var ndx = 0; ndx < pointers.length; ++ndx) {
        gl.vertexAttribPointer(0, pointers[ndx][0], pointers[ndx][1], pointers[ndx][3], pointers[ndx][2], pointers[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_INTEGER, false));
    }

    var pointersI = [
        [1, gl.BYTE, 0, false, 0],
        [1, gl.SHORT, 0, false, 0],
        [1, gl.INT, 0, false, 0],
        [1, gl.UNSIGNED_BYTE, 0, false, 0],
        [1, gl.UNSIGNED_SHORT, 0, false, 0],
        [1, gl.UNSIGNED_INT, 0, false, 0]
    ];

    for (var ndx = 0; ndx < pointersI.length; ++ndx) {
        gl.vertexAttribIPointer(0, pointersI[ndx][0], pointersI[ndx][1], pointersI[ndx][2], pointersI[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_INTEGER, true));
    }

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_INTEGER, false));

    // set vao 0 to some value
    gl.vertexAttribIPointer(0, 1, gl.INT, 0, 0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribPointer(0, 1, gl.FLOAT, false, 0, 0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_INTEGER, false));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_INTEGER, true));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buf);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeEnabledCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeEnabledCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeEnabledCase.prototype.test = function() {
    // Test with default VAO

    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_ENABLED, false));
    gl.enableVertexAttribArray(0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_ENABLED, true));
    gl.disableVertexAttribArray(0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_ENABLED, false));

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    // set vao 0 to some value
    gl.enableVertexAttribArray(0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.disableVertexAttribArray(0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_ENABLED, false));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_ENABLED, true));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeDivisorCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeDivisorCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeDivisorCase.prototype.test = function() {
    // Test with default VAO

    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_DIVISOR, 0));
    gl.vertexAttribDivisor(0, 1);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_DIVISOR, 1));
    gl.vertexAttribDivisor(0, 5);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_DIVISOR, 5));

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    // set vao 0 to some value
    gl.vertexAttribDivisor(0, 1);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribDivisor(0, 5);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_DIVISOR, 5));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_DIVISOR, 1));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeBufferBindingCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeBufferBindingCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeBufferBindingCase.prototype.test = function() {
    // Test with default VAO

    var buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, buffer));

    gl.deleteBuffer(buffer);

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();
    var buffer0 = gl.createBuffer();
    var buffer1 = gl.createBuffer();

    // initial
    gl.bindVertexArray(vao0);
    // set vao 0 to some value
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer0);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer1);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, buffer1));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING, buffer0));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buffer0);
    gl.deleteBuffer(buffer1);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.VertexAttributeOffsetCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.VertexAttributeOffsetCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.VertexAttributeOffsetCase.prototype.test = function() {
    var pointers = [
        [1, gl.BYTE, 0, false, 2 * 4],
        [1, gl.SHORT, 0, false, 1 * 4],
        [1, gl.INT, 0, false, 2 * 4],
        [1, gl.FLOAT, 0, false, 0 * 4],
        [1, gl.FLOAT, 0, false, 3 * 4],
        [1, gl.FLOAT, 0, false, 2 * 4],
        [1, gl.HALF_FLOAT, 0, false, 0 * 4],
        [4, gl.HALF_FLOAT, 0, false, 1 * 4],
        [4, gl.HALF_FLOAT, 0, false, 2 * 4]
    ];

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);

    // Test with default VAO

    for (var ndx = 0; ndx < pointers.length; ++ndx) {
        gl.vertexAttribPointer(0, pointers[ndx][0], pointers[ndx][1], pointers[ndx][3], pointers[ndx][2], pointers[ndx][4]);
        this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_POINTER, pointers[ndx][4]));
    }

    // Test with multiple VAOs
    var vao0 = gl.createVertexArray();
    var vao1 = gl.createVertexArray();

    // initial
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_POINTER, 0));

    // set vao 0 to some value
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 8);

    // set vao 1 to some other value
    gl.bindVertexArray(vao1);
    gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 4);

    // verify vao 1 state
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_POINTER, 4));

    // verify vao 0 state
    gl.bindVertexArray(vao0);
    this.check(glsStateQuery.verifyVertexAttrib(0, gl.VERTEX_ATTRIB_ARRAY_POINTER, 8));

    gl.deleteVertexArray(vao0);
    gl.deleteVertexArray(vao1);
    gl.deleteBuffer(buf);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueFloatCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueFloatCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueFloatCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform highp float floatUniform;\n' +
        'uniform highp vec2 float2Uniform;\n' +
        'uniform highp vec3 float3Uniform;\n' +
        'uniform highp vec4 float4Uniform;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(floatUniform + float2Uniform.x + float3Uniform.x + float4Uniform.x);\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);
    gl.useProgram(program);

    var location;

    location = gl.getUniformLocation(program, 'floatUniform');
    gl.uniform1f(location, 1);
    this.check(glsStateQuery.verifyUniform(program, location, 1));

    location = gl.getUniformLocation(program, 'float2Uniform');
    gl.uniform2f(location, 1, 2);
    this.check(glsStateQuery.verifyUniform(program, location, new Float32Array([1, 2])));

    location = gl.getUniformLocation(program, 'float3Uniform');
    gl.uniform3f(location, 1, 2, 3);
    this.check(glsStateQuery.verifyUniform(program, location, new Float32Array([1, 2, 3])));

    location = gl.getUniformLocation(program, 'float4Uniform');
    gl.uniform4f(location, 1, 2, 3, 4);
    this.check(glsStateQuery.verifyUniform(program, location, new Float32Array([1, 2, 3, 4])));

    gl.useProgram(null);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueIntCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueIntCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueIntCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform highp int intUniform;\n' +
        'uniform highp ivec2 int2Uniform;\n' +
        'uniform highp ivec3 int3Uniform;\n' +
        'uniform highp ivec4 int4Uniform;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(float(intUniform + int2Uniform.x + int3Uniform.x + int4Uniform.x));\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);
    gl.useProgram(program);

    var location;

    location = gl.getUniformLocation(program, 'intUniform');
    gl.uniform1i(location, 1);
    this.check(glsStateQuery.verifyUniform(program, location, 1));

    location = gl.getUniformLocation(program, 'int2Uniform');
    gl.uniform2i(location, 1, 2);
    this.check(glsStateQuery.verifyUniform(program, location, new Int32Array([1, 2])));

    location = gl.getUniformLocation(program, 'int3Uniform');
    gl.uniform3i(location, 1, 2, 3);
    this.check(glsStateQuery.verifyUniform(program, location, new Int32Array([1, 2, 3])));

    location = gl.getUniformLocation(program, 'int4Uniform');
    gl.uniform4i(location, 1, 2, 3, 4);
    this.check(glsStateQuery.verifyUniform(program, location, new Int32Array([1, 2, 3, 4])));

    gl.useProgram(null);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueUintCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueUintCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueUintCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform highp uint uintUniform;\n' +
        'uniform highp uvec2 uint2Uniform;\n' +
        'uniform highp uvec3 uint3Uniform;\n' +
        'uniform highp uvec4 uint4Uniform;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(float(uintUniform + uint2Uniform.x + uint3Uniform.x + uint4Uniform.x));\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);
    gl.useProgram(program);

    var location;

    location = gl.getUniformLocation(program, 'uintUniform');
    gl.uniform1ui(location, 1);
    this.check(glsStateQuery.verifyUniform(program, location, 1));

    location = gl.getUniformLocation(program, 'uint2Uniform');
    gl.uniform2ui(location, 1, 2);
    this.check(glsStateQuery.verifyUniform(program, location, new Uint32Array([1, 2])));

    location = gl.getUniformLocation(program, 'uint3Uniform');
    gl.uniform3ui(location, 1, 2, 3);
    this.check(glsStateQuery.verifyUniform(program, location, new Uint32Array([1, 2, 3])));

    location = gl.getUniformLocation(program, 'uint4Uniform');
    gl.uniform4ui(location, 1, 2, 3, 4);
    this.check(glsStateQuery.verifyUniform(program, location, new Uint32Array([1, 2, 3, 4])));

    gl.useProgram(null);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueBooleanCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueBooleanCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueBooleanCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform bool boolUniform;\n' +
        'uniform bvec2 bool2Uniform;\n' +
        'uniform bvec3 bool3Uniform;\n' +
        'uniform bvec4 bool4Uniform;\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(float(boolUniform) + float(bool2Uniform.x) + float(bool3Uniform.x) + float(bool4Uniform.x));\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);
    gl.useProgram(program);

    var location;

    location = gl.getUniformLocation(program, 'boolUniform');
    gl.uniform1i(location, 1);
    this.check(glsStateQuery.verifyUniform(program, location, true));

    location = gl.getUniformLocation(program, 'bool2Uniform');
    gl.uniform2i(location, 1, 0);
    this.check(glsStateQuery.verifyUniform(program, location, [true, false]));

    location = gl.getUniformLocation(program, 'bool3Uniform');
    gl.uniform3i(location, 1, 0, 1);
    this.check(glsStateQuery.verifyUniform(program, location, [true, false, true]));

    location = gl.getUniformLocation(program, 'bool4Uniform');
    gl.uniform4i(location, 1, 0, 1, 0);
    this.check(glsStateQuery.verifyUniform(program, location, [true, false, true, false]));

    gl.useProgram(null);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueSamplerCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueSamplerCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueSamplerCase.prototype.test = function() {
        var testVertSource =
            '#version 300 es\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = vec4(0.0);\n' +
            '}\n';
        var testFragSource =
            '#version 300 es\n' +
            'uniform highp sampler2D uniformSampler;\n' +
            'layout(location = 0) out mediump vec4 fragColor;' +
            'void main (void)\n' +
            '{\n' +
            ' fragColor = vec4(textureSize(uniformSampler, 0).x);\n' +
            '}\n';

        var shaderVert = gl.createShader(gl.VERTEX_SHADER);
        var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

        gl.shaderSource(shaderVert, testVertSource);
        gl.shaderSource(shaderFrag, testFragSource);

        gl.compileShader(shaderVert);
        gl.compileShader(shaderFrag);

        var program = gl.createProgram();
        gl.attachShader(program, shaderVert);
        gl.attachShader(program, shaderFrag);
        gl.linkProgram(program);
        gl.useProgram(program);

        var location;

        location = gl.getUniformLocation(program, 'uniformSampler');
        gl.uniform1i(location, 1);
        this.check(glsStateQuery.verifyUniform(program, location, 1));

        gl.useProgram(null);
        gl.deleteShader(shaderVert);
        gl.deleteShader(shaderFrag);
        gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueArrayCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueArrayCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueArrayCase.prototype.test = function() {
    var testVertSource =
        '#version 300 es\n' +
        'uniform highp float arrayUniform[5];' +
        'uniform highp vec2 array2Uniform[5];' +
        'uniform highp vec3 array3Uniform[5];' +
        'uniform highp vec4 array4Uniform[5];' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = \n' +
        ' + vec4(arrayUniform[0] + arrayUniform[1] + arrayUniform[2] + arrayUniform[3] + arrayUniform[4])\n' +
        ' + vec4(array2Uniform[0].x + array2Uniform[1].x + array2Uniform[2].x + array2Uniform[3].x + array2Uniform[4].x)\n' +
        ' + vec4(array3Uniform[0].x + array3Uniform[1].x + array3Uniform[2].x + array3Uniform[3].x + array3Uniform[4].x)\n' +
        ' + vec4(array4Uniform[0].x + array4Uniform[1].x + array4Uniform[2].x + array4Uniform[3].x + array4Uniform[4].x);\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);
    gl.useProgram(program);

    var location;

    var uniformValue = [
        -1.0, 0.1, 4.0, 800.0,
        13.0, 55.0, 12.0, 91.0,
        -55.1, 1.1, 98.0, 19.0,
        41.0, 65.0, 4.0, 12.2,
        95.0, 77.0, 32.0, 48.0
    ];

    location = gl.getUniformLocation(program, 'arrayUniform');
    gl.uniform1fv(location, new Float32Array(uniformValue.slice(0, 5)));

    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'arrayUniform[0]'), uniformValue[0]));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'arrayUniform[1]'), uniformValue[1]));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'arrayUniform[2]'), uniformValue[2]));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'arrayUniform[3]'), uniformValue[3]));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'arrayUniform[4]'), uniformValue[4]));

    location = gl.getUniformLocation(program, 'array2Uniform');
    gl.uniform2fv(location, new Float32Array(uniformValue.slice(0, 10)));

    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array2Uniform[0]'), new Float32Array([uniformValue[2 * 0], uniformValue[(2 * 0) + 1]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array2Uniform[1]'), new Float32Array([uniformValue[2 * 1], uniformValue[(2 * 1) + 1]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array2Uniform[2]'), new Float32Array([uniformValue[2 * 2], uniformValue[(2 * 2) + 1]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array2Uniform[3]'), new Float32Array([uniformValue[2 * 3], uniformValue[(2 * 3) + 1]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array2Uniform[4]'), new Float32Array([uniformValue[2 * 4], uniformValue[(2 * 4) + 1]])));

    location = gl.getUniformLocation(program, 'array3Uniform');
    gl.uniform3fv(location, new Float32Array(uniformValue.slice(0, 15)));

    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array3Uniform[0]'), new Float32Array([uniformValue[3 * 0], uniformValue[(3 * 0) + 1], uniformValue[(3 * 0) + 2]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array3Uniform[1]'), new Float32Array([uniformValue[3 * 1], uniformValue[(3 * 1) + 1], uniformValue[(3 * 1) + 2]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array3Uniform[2]'), new Float32Array([uniformValue[3 * 2], uniformValue[(3 * 2) + 1], uniformValue[(3 * 2) + 2]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array3Uniform[3]'), new Float32Array([uniformValue[3 * 3], uniformValue[(3 * 3) + 1], uniformValue[(3 * 3) + 2]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array3Uniform[4]'), new Float32Array([uniformValue[3 * 4], uniformValue[(3 * 4) + 1], uniformValue[(3 * 4) + 2]])));

    location = gl.getUniformLocation(program, 'array4Uniform');
    gl.uniform4fv(location, new Float32Array(uniformValue));

    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array4Uniform[0]'), new Float32Array([uniformValue[4 * 0], uniformValue[(4 * 0) + 1], uniformValue[(4 * 0) + 2], uniformValue[(4 * 0) + 3]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array4Uniform[1]'), new Float32Array([uniformValue[4 * 1], uniformValue[(4 * 1) + 1], uniformValue[(4 * 1) + 2], uniformValue[(4 * 1) + 3]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array4Uniform[2]'), new Float32Array([uniformValue[4 * 2], uniformValue[(4 * 2) + 1], uniformValue[(4 * 2) + 2], uniformValue[(4 * 2) + 3]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array4Uniform[3]'), new Float32Array([uniformValue[4 * 3], uniformValue[(4 * 3) + 1], uniformValue[(4 * 3) + 2], uniformValue[(4 * 3) + 3]])));
    this.check(glsStateQuery.verifyUniform(program, gl.getUniformLocation(program, 'array4Uniform[4]'), new Float32Array([uniformValue[4 * 4], uniformValue[(4 * 4) + 1], uniformValue[(4 * 4) + 2], uniformValue[(4 * 4) + 3]])));

    gl.useProgram(null);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderStateQueryTests.UniformValueMatrixCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fShaderStateQueryTests.UniformValueMatrixCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.UniformValueMatrixCase.prototype.test = function() {
    var transpose = function(rows, cols, data) {
        var matrix = tcuMatrix.matrixFromDataArray(rows, cols, data);
        var result = [];
        for (var col = 0; col < cols; col++)
            result.push(matrix.getColumn(col));
        return new Float32Array([].concat.apply([], result));
    };

    var testVertSource =
        '#version 300 es\n' +
        'uniform highp mat2 mat2Uniform;' +
        'uniform highp mat3 mat3Uniform;' +
        'uniform highp mat4 mat4Uniform;' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(mat2Uniform[0][0] + mat3Uniform[0][0] + mat4Uniform[0][0]);\n' +
        '}\n';
    var testFragSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 fragColor;' +
        'void main (void)\n' +
        '{\n' +
        ' fragColor = vec4(0.0);\n' +
        '}\n';

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

    gl.shaderSource(shaderVert, testVertSource);
    gl.shaderSource(shaderFrag, testFragSource);

    gl.compileShader(shaderVert);
    gl.compileShader(shaderFrag);

    var program = gl.createProgram();
    gl.attachShader(program, shaderVert);
    gl.attachShader(program, shaderFrag);
    gl.linkProgram(program);
    gl.useProgram(program);

    var location;

    var matrixValues = [
        -1.0, 0.1, 4.0, 800.0,
        13.0, 55.0, 12.0, 91.0,
        -55.1, 1.1, 98.0, 19.0,
        41.0, 65.0, 4.0, 12.0
    ];

    // the values of the matrix are returned in column major order but they can be given in either order

    location = gl.getUniformLocation(program, 'mat2Uniform');
    var m2 = new Float32Array(matrixValues.slice(0, 2 * 2));
    gl.uniformMatrix2fv(location, false, m2);
    this.check(glsStateQuery.verifyUniform(program, location, m2));
    gl.uniformMatrix2fv(location, true, m2);
    this.check(glsStateQuery.verifyUniform(program, location, transpose(2, 2, m2)));

    location = gl.getUniformLocation(program, 'mat3Uniform');
    var m3 = new Float32Array(matrixValues.slice(0, 3 * 3));
    gl.uniformMatrix3fv(location, false, m3);
    this.check(glsStateQuery.verifyUniform(program, location, m3));
    gl.uniformMatrix3fv(location, true, m3);
    this.check(glsStateQuery.verifyUniform(program, location, transpose(3, 3, m3)));

    location = gl.getUniformLocation(program, 'mat4Uniform');
    var m4 = new Float32Array(matrixValues.slice(0, 4 * 4));
    gl.uniformMatrix4fv(location, false, m4);
    this.check(glsStateQuery.verifyUniform(program, location, m4));
    gl.uniformMatrix4fv(location, true, m4);
    this.check(glsStateQuery.verifyUniform(program, location, transpose(4, 4, m4)));

    gl.useProgram(null);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(program);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {number} shaderType
 * @param {number} precisionType
 */
es3fShaderStateQueryTests.PrecisionFormatCase = function(name, description, shaderType, precisionType) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_shaderType = shaderType;
    this.m_precisionType = precisionType;
};

setParentClass(es3fShaderStateQueryTests.PrecisionFormatCase, es3fApiCase.ApiCase);

es3fShaderStateQueryTests.PrecisionFormatCase.prototype.test = function() {
    var requirements = {};
    requirements[gl.LOW_FLOAT] = [0, 0, 8];
    requirements[gl.MEDIUM_FLOAT] = [13, 13, 10];
    requirements[gl.HIGH_FLOAT] = [127, 127, 23];
    requirements[gl.LOW_INT] = [8, 7, 0];
    requirements[gl.MEDIUM_INT] = [15, 14, 0];
    requirements[gl.HIGH_INT] = [31, 30, 0];


    var expected = requirements[this.m_precisionType];
    var result = gl.getShaderPrecisionFormat(this.m_shaderType, this.m_precisionType);

    bufferedLogToConsole('Precision:' +
                ' range min = ' + result.rangeMin +
                ' range max = ' + result.rangeMax +
                ' precision = ' + result.precision);

    if (this.m_precisionType == gl.HIGH_FLOAT) {
        // highp float must be IEEE 754 single

        this.check(result.rangeMin == expected[0] ||
            result.rangeMax == expected[1] ||
            result.precision == expected[2],
                'Invalid precision format, expected:' +
                ' range min = ' + expected[0] +
                ' range max = ' + expected[1] +
                ' precision = ' + expected[2]);
    } else{
        this.check(result.rangeMin >= expected[0] ||
            result.rangeMax >= expected[1] ||
            result.precision >= expected[2],
                'Invalid precision format, expected:' +
                ' range min >= ' + expected[0] +
                ' range max >= ' + expected[1] +
                ' precision >= ' + expected[2]);
    }
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fShaderStateQueryTests.ShaderStateQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'shader', 'Shader State Query tests');
};

es3fShaderStateQueryTests.ShaderStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fShaderStateQueryTests.ShaderStateQueryTests.prototype.constructor = es3fShaderStateQueryTests.ShaderStateQueryTests;

es3fShaderStateQueryTests.ShaderStateQueryTests.prototype.init = function() {
    // shader
    this.addChild(new es3fShaderStateQueryTests.ShaderTypeCase('shader_type', 'SHADER_TYPE'));
    this.addChild(new es3fShaderStateQueryTests.ShaderCompileStatusCase('shader_compile_status', 'COMPILE_STATUS'));
    this.addChild(new es3fShaderStateQueryTests.ShaderInfoLogCase('shader_info_log', 'INFO_LOG'));
    this.addChild(new es3fShaderStateQueryTests.ShaderSourceCase('shader_source', 'SHADER_SOURCE'));

    // shader and program
    this.addChild(new es3fShaderStateQueryTests.DeleteStatusCase('delete_status', 'DELETE_STATUS'));

    // // vertex-attrib
    this.addChild(new es3fShaderStateQueryTests.CurrentVertexAttribInitialCase('current_vertex_attrib_initial', 'CURRENT_VERTEX_ATTRIB'));
    this.addChild(new es3fShaderStateQueryTests.CurrentVertexAttribFloatCase('current_vertex_attrib_float', 'CURRENT_VERTEX_ATTRIB'));
    this.addChild(new es3fShaderStateQueryTests.CurrentVertexAttribIntCase('current_vertex_attrib_int', 'CURRENT_VERTEX_ATTRIB'));
    this.addChild(new es3fShaderStateQueryTests.CurrentVertexAttribUintCase('current_vertex_attrib_uint', 'CURRENT_VERTEX_ATTRIB'));

    // // program
    this.addChild(new es3fShaderStateQueryTests.ProgramInfoLogCase('program_info_log', 'INFO_LOG'));
    this.addChild(new es3fShaderStateQueryTests.ProgramValidateStatusCase('program_validate_status', 'VALIDATE_STATUS'));
    this.addChild(new es3fShaderStateQueryTests.ProgramAttachedShadersCase('program_attached_shaders', 'ATTACHED_SHADERS'));

    this.addChild(new es3fShaderStateQueryTests.ProgramActiveUniformNameCase('program_active_uniform_name', 'ACTIVE_UNIFORMS'));
    this.addChild(new es3fShaderStateQueryTests.ProgramUniformCase('program_active_uniform_types', 'UNIFORM_TYPE, UNIFORM_SIZE, and UNIFORM_IS_ROW_MAJOR'));
    this.addChild(new es3fShaderStateQueryTests.ProgramActiveUniformBlocksCase ("program_active_uniform_blocks", "ACTIVE_UNIFORM_BLOCK_x"));

    // transform feedback
    this.addChild(new es3fShaderStateQueryTests.TransformFeedbackCase('transform_feedback', 'TRANSFORM_FEEDBACK_BUFFER_MODE, TRANSFORM_FEEDBACK_VARYINGS, TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH'));

    // attribute related
    this.addChild(new es3fShaderStateQueryTests.ActiveAttributesCase('active_attributes', 'ACTIVE_ATTRIBUTES and ACTIVE_ATTRIBUTE_MAX_LENGTH'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeSizeCase('vertex_attrib_size', 'VERTEX_ATTRIB_ARRAY_SIZE'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeTypeCase('vertex_attrib_type', 'VERTEX_ATTRIB_ARRAY_TYPE'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeStrideCase('vertex_attrib_stride', 'VERTEX_ATTRIB_ARRAY_STRIDE'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeNormalizedCase('vertex_attrib_normalized', 'VERTEX_ATTRIB_ARRAY_NORMALIZED'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeIntegerCase('vertex_attrib_integer', 'VERTEX_ATTRIB_ARRAY_INTEGER'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeEnabledCase('vertex_attrib_array_enabled', 'VERTEX_ATTRIB_ARRAY_ENABLED'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeDivisorCase('vertex_attrib_array_divisor', 'VERTEX_ATTRIB_ARRAY_DIVISOR'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeBufferBindingCase('vertex_attrib_array_buffer_binding', 'VERTEX_ATTRIB_ARRAY_BUFFER_BINDING'));
    this.addChild(new es3fShaderStateQueryTests.VertexAttributeOffsetCase('vertex_attrib_offset', 'VERTEX_ATTRIB_ARRAY_POINTER'));

    // uniform values
    this.addChild(new es3fShaderStateQueryTests.UniformValueFloatCase('uniform_value_float', 'GetUniform*'));
    this.addChild(new es3fShaderStateQueryTests.UniformValueIntCase('uniform_value_int', 'GetUniform*'));
    this.addChild(new es3fShaderStateQueryTests.UniformValueUintCase('uniform_value_uint', 'GetUniform*'));
    this.addChild(new es3fShaderStateQueryTests.UniformValueBooleanCase('uniform_value_boolean', 'GetUniform*'));
    this.addChild(new es3fShaderStateQueryTests.UniformValueSamplerCase('uniform_value_sampler', 'GetUniform*'));
    this.addChild(new es3fShaderStateQueryTests.UniformValueArrayCase('uniform_value_array', 'GetUniform*'));
    this.addChild(new es3fShaderStateQueryTests.UniformValueMatrixCase('uniform_value_matrix', 'GetUniform*'));

    // precision format query
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_vertex_lowp_float', 'GetShaderPrecisionFormat', gl.VERTEX_SHADER, gl.LOW_FLOAT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_vertex_mediump_float', 'GetShaderPrecisionFormat', gl.VERTEX_SHADER, gl.MEDIUM_FLOAT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_vertex_highp_float', 'GetShaderPrecisionFormat', gl.VERTEX_SHADER, gl.HIGH_FLOAT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_vertex_lowp_int', 'GetShaderPrecisionFormat', gl.VERTEX_SHADER, gl.LOW_INT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_vertex_mediump_int', 'GetShaderPrecisionFormat', gl.VERTEX_SHADER, gl.MEDIUM_INT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_vertex_highp_int', 'GetShaderPrecisionFormat', gl.VERTEX_SHADER, gl.HIGH_INT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_fragment_lowp_float', 'GetShaderPrecisionFormat', gl.FRAGMENT_SHADER, gl.LOW_FLOAT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_fragment_mediump_float', 'GetShaderPrecisionFormat', gl.FRAGMENT_SHADER, gl.MEDIUM_FLOAT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_fragment_highp_float', 'GetShaderPrecisionFormat', gl.FRAGMENT_SHADER, gl.HIGH_FLOAT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_fragment_lowp_int', 'GetShaderPrecisionFormat', gl.FRAGMENT_SHADER, gl.LOW_INT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_fragment_mediump_int', 'GetShaderPrecisionFormat', gl.FRAGMENT_SHADER, gl.MEDIUM_INT));
    this.addChild(new es3fShaderStateQueryTests.PrecisionFormatCase('precision_fragment_highp_int', 'GetShaderPrecisionFormat', gl.FRAGMENT_SHADER, gl.HIGH_INT));
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fShaderStateQueryTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fShaderStateQueryTests.ShaderStateQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fShaderStateQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

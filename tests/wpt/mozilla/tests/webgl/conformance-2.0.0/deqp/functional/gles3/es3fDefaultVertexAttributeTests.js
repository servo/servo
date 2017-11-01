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
goog.provide('functional.gles3.es3fDefaultVertexAttributeTests');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {
var es3fDefaultVertexAttributeTests = functional.gles3.es3fDefaultVertexAttributeTests;
var tcuTestCase = framework.common.tcuTestCase;
var tcuSurface = framework.common.tcuSurface;
var deMath = framework.delibs.debase.deMath;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var tcuLogImage = framework.common.tcuLogImage;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib1f = function() {
    this.caseName = 'vertex_attrib_1f';
    this.name = 'VertexAttrib1f';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib1f(index, value[0]);
        return [value[0], 0, 0, 1];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib2f = function() {
    this.caseName = 'vertex_attrib_2f';
    this.name = 'VertexAttrib2f';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib2f(index, value[0], value[1]);
        return [value[0], value[1], 0, 1];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib3f = function() {
    this.caseName = 'vertex_attrib_3f';
    this.name = 'VertexAttrib3f';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib3f(index, value[0], value[1], value[2]);
        return [value[0], value[1], value[2], 1];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib4f = function() {
    this.caseName = 'vertex_attrib_4f';
    this.name = 'VertexAttrib4f';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib4f(index, value[0], value[1], value[2], value[3]);
        return [value[0], value[1], value[2], value[3]];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib1fv = function() {
    this.caseName = 'vertex_attrib_1fv';
    this.name = 'VertexAttrib1fv';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib1fv(index, value.slice(0, 1));
        return [value[0], 0, 0, 1];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib2fv = function() {
    this.caseName = 'vertex_attrib_2fv';
    this.name = 'VertexAttrib2fv';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib2fv(index, value.slice(0, 2));
        return [value[0], value[1], 0, 1];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib3fv = function() {
    this.caseName = 'vertex_attrib_3fv';
    this.name = 'VertexAttrib3fv';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib3fv(index, value.slice(0, 3));
        return [value[0], value[1], value[2], 1];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttrib4fv = function() {
    this.caseName = 'vertex_attrib_4fv';
    this.name = 'VertexAttrib4fv';
    this.signed = true;
    this.load = function(index, value) {
        gl.vertexAttrib4fv(index, value.slice(0, 4));
        return [value[0], value[1], value[2], value[3]];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttribI4i = function() {
    this.caseName = 'vertex_attrib_4i';
    this.name = 'VertexAttribI4i';
    this.signed = true;
    this.load = function(index, value) {
        var v = new Int32Array(value);
        gl.vertexAttribI4i(index, v[0], v[1], v[2], v[3]);
        return [v[0], v[1], v[2], v[3]];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttribI4iv = function() {
    this.caseName = 'vertex_attrib_4iv';
    this.name = 'VertexAttribI4iv';
    this.signed = true;
    this.load = function(index, value) {
        var v = new Int32Array(value);
        gl.vertexAttribI4iv(index, v);
        return [v[0], v[1], v[2], v[3]];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttribI4ui = function() {
    this.caseName = 'vertex_attrib_4ui';
    this.name = 'VertexAttribI4ui';
    this.signed = false;
    this.load = function(index, value) {
        var v = new Uint32Array(value);
        gl.vertexAttribI4ui(index, v[0], v[1], v[2], v[3]);
        return [v[0], v[1], v[2], v[3]];
    };
};

/**
 * @constructor
 */
es3fDefaultVertexAttributeTests.LoaderVertexAttribI4uiv = function() {
    this.caseName = 'vertex_attrib_4uiv';
    this.name = 'VertexAttribI4uiv';
    this.signed = false;
    this.load = function(index, value) {
        var v = new Uint32Array(value);
        gl.vertexAttribI4uiv(index, v);
        return [v[0], v[1], v[2], v[3]];
    };
};

/** @const */ var RENDER_SIZE = 32;
/** @const */ var s_valueRange = 10;
/** @const */ var s_passThroughFragmentShaderSource = '#version 300 es\n' +
                                                        'layout(location = 0) out mediump vec4 fragColor;\n' +
                                                        'in mediump vec4 v_color;\n' +
                                                        'void main (void)\n' +
                                                        '{\n' +
                                                        ' fragColor = v_color;\n' +
                                                        '}\n';

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fDefaultVertexAttributeTests.AttributeCase = function(loaderType, dataType) {
    var loader = new loaderType();
    var name = loader.caseName;
    var description = 'Test ' + loader.name;
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_funcName = loader.name;
    this.m_useNegativeValues = loader.signed;
    this.m_dataType = dataType;
    this.m_allIterationsPassed = true;
    this.m_loader = loader;
    this.m_iteration = 0;
};

setParentClass(es3fDefaultVertexAttributeTests.AttributeCase, tcuTestCase.DeqpTest);

es3fDefaultVertexAttributeTests.AttributeCase.prototype.init = function() {
    // log test info

    var maxRange = s_valueRange;
    var minRange = (this.m_useNegativeValues) ? (-maxRange) : (0.0);

    bufferedLogToConsole(
        'Loading attribute values using ' + this.m_funcName + '\n' +
        'Attribute type: ' + gluShaderUtil.getDataTypeName(this.m_dataType) + '\n' +
        'Attribute value range: [' + minRange + ', ' + maxRange + ']');

    // gen shader and base quad

    this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(this.genVertexSource(), s_passThroughFragmentShaderSource));
    if (!this.m_program.isOk())
        testFailedOptions('could not build program', true);

    var fullscreenQuad = [
         1.0, 1.0, 0.0, 1.0,
         1.0, -1.0, 0.0, 1.0,
        -1.0, 1.0, 0.0, 1.0,
        -1.0, -1.0, 0.0, 1.0
    ];

    this.m_bufID = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.m_bufID);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(fullscreenQuad), gl.STATIC_DRAW);
};

es3fDefaultVertexAttributeTests.AttributeCase.prototype.deinit = function() {
    this.m_loader = null;

    gl.useProgram(null);
    this.m_program = null;

    if (this.m_bufID) {
        gl.deleteBuffer(this.m_bufID);
        this.m_bufID = null;
    }
};

es3fDefaultVertexAttributeTests.AttributeCase.prototype.iterate = function() {
    var testValues = [
        [0.0, 0.5, 0.2, 1.0],
        [0.1, 0.7, 1.0, 0.6],
        [0.4, 0.2, 0.0, 0.5],
        [0.5, 0.0, 0.9, 0.1],
        [0.6, 0.2, 0.2, 0.9],
        [0.9, 1.0, 0.0, 0.0],
        [1.0, 0.5, 0.3, 0.8]
    ];

    bufferedLogToConsole('Iteration ' + (this.m_iteration + 1) + '/' + testValues.length);

    var testValue = this.m_useNegativeValues ?
        deMath.subScalar(deMath.scale(testValues[this.m_iteration], 2), 1) :
        deMath.scale(testValues[this.m_iteration], s_valueRange);

    if (!this.renderWithValue(testValue))
        this.m_allIterationsPassed = false;

    // continue

    if (++this.m_iteration < testValues.length)
        return tcuTestCase.IterateResult.CONTINUE;

    if (this.m_allIterationsPassed)
        testPassed();
    else
        testFailed('Got unexpected values');

    return tcuTestCase.IterateResult.STOP;
};

es3fDefaultVertexAttributeTests.AttributeCase.prototype.genVertexSource = function() {
    var vectorSize = (gluShaderUtil.isDataTypeMatrix(this.m_dataType)) ? (gluShaderUtil.getDataTypeMatrixNumRows(this.m_dataType)) : (gluShaderUtil.isDataTypeVector(this.m_dataType)) ? (gluShaderUtil.getDataTypeScalarSize(this.m_dataType)) : (-1);
    var vectorType = gluShaderUtil.getDataTypeName((gluShaderUtil.isDataTypeMatrix(this.m_dataType)) ? (gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.FLOAT, vectorSize)) : (gluShaderUtil.isDataTypeVector(this.m_dataType)) ? (gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.FLOAT, vectorSize)) : (gluShaderUtil.DataType.FLOAT));
    var components = (gluShaderUtil.isDataTypeMatrix(this.m_dataType)) ? (gluShaderUtil.getDataTypeMatrixNumRows(this.m_dataType)) : (gluShaderUtil.getDataTypeScalarSize(this.m_dataType));

    var buf = '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in highp ' + gluShaderUtil.getDataTypeName(this.m_dataType) + ' a_value;\n' +
            'out highp vec4 v_color;\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            '\n';

    buf += ' highp ' + vectorType + ' normalizedValue = ' + ((gluShaderUtil.getDataTypeScalarType(this.m_dataType) == gluShaderUtil.DataType.FLOAT) ? ('') : (vectorType)) + '(a_value' + ((gluShaderUtil.isDataTypeMatrix(this.m_dataType)) ? ('[1]') : ('')) + ') / float(' + s_valueRange + ');\n';

    if (this.m_useNegativeValues)
        buf += ' highp ' + vectorType + ' positiveNormalizedValue = (normalizedValue + ' + vectorType + '(1.0)) / 2.0;\n';
    else
        buf += ' highp ' + vectorType + ' positiveNormalizedValue = normalizedValue;\n';

    if (components == 1)
        buf += ' v_color = vec4(positiveNormalizedValue, 0.0, 0.0, 1.0);\n';
    else if (components == 2)
        buf += ' v_color = vec4(positiveNormalizedValue.xy, 0.0, 1.0);\n';
    else if (components == 3)
        buf += ' v_color = vec4(positiveNormalizedValue.xyz, 1.0);\n';
    else if (components == 4)
        buf += ' v_color = vec4((positiveNormalizedValue.xy + positiveNormalizedValue.zz) / 2.0, positiveNormalizedValue.w, 1.0);\n';
    else
       throw new Error('Wrong component size: ' + components);

    buf += '}\n';

    return buf;
};

es3fDefaultVertexAttributeTests.AttributeCase.prototype.renderWithValue = function(v) {
    var positionIndex = gl.getAttribLocation(this.m_program.getProgram(), 'a_position');
    var valueIndex = gl.getAttribLocation(this.m_program.getProgram(), 'a_value');
    var dest = new tcuSurface.Surface(RENDER_SIZE, RENDER_SIZE);

    gl.clearColor(0.0, 0.0, 0.0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.viewport(0, 0, RENDER_SIZE, RENDER_SIZE);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.m_bufID);
    gl.vertexAttribPointer(positionIndex, 4, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(positionIndex);

    // transfer test value. Load to the second column in the matrix case
    var loadedValue = this.m_loader.load((gluShaderUtil.isDataTypeMatrix(this.m_dataType)) ? (valueIndex + 1) : (valueIndex), v);

    gl.useProgram(this.m_program.getProgram());
    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    gl.useProgram(null);
    // The original c++ test does not disable vertex attrib array, which is wrong.
    // On most drivers all tests pass because a_position is assigned location 0.
    // On MacOSX some tests fail because a_value is assigned location 0 and vertex
    // attrib array is left enabled and affects later tests.
    gl.disableVertexAttribArray(positionIndex);
    dest.readViewport(gl);

    // check whole result is colored correctly
    return this.verifyUnicoloredBuffer(dest, this.computeColor(loadedValue));
};

es3fDefaultVertexAttributeTests.AttributeCase.prototype.computeColor = function(value) {
    var normalizedValue = deMath.scale(value, 1 / s_valueRange);
    var positiveNormalizedValue = this.m_useNegativeValues ?
        deMath.scale(deMath.addScalar(normalizedValue, 1), 0.5) :
        normalizedValue;
    var components = (gluShaderUtil.isDataTypeMatrix(this.m_dataType)) ? (gluShaderUtil.getDataTypeMatrixNumRows(this.m_dataType)) : (gluShaderUtil.getDataTypeScalarSize(this.m_dataType));

    if (components == 1)
        return [positiveNormalizedValue[0], 0.0, 0.0, 1.0];
    else if (components == 2)
        return [positiveNormalizedValue[0], positiveNormalizedValue[1], 0.0, 1.0];
    else if (components == 3)
        return [positiveNormalizedValue[0], positiveNormalizedValue[1], positiveNormalizedValue[2], 1.0];
    else if (components == 4)
        return [(positiveNormalizedValue[0] + positiveNormalizedValue[2]) / 2.0, (positiveNormalizedValue[1] + positiveNormalizedValue[2]) / 2.0, positiveNormalizedValue[3], 1.0];
    else
       throw new Error('Wrong component size: ' + components);
};

/**
 * @param {tcuSurface.Surface} scene
 * @param {Array<number>} refColor
 * @return {boolean}
 */
es3fDefaultVertexAttributeTests.AttributeCase.prototype.verifyUnicoloredBuffer = function(scene, refColor) {
    var access = scene.getAccess();
    var errorMask = new tcuSurface.Surface(RENDER_SIZE, RENDER_SIZE);
    var colorThreshold = [6, 6, 6, 6];
    var error = false;

    errorMask.getAccess().clear([0, 1, 0, 1]);

    bufferedLogToConsole('Verifying rendered image. Expecting color ' + refColor + ', threshold ' + colorThreshold);

    for (var y = 0; y < RENDER_SIZE; ++y)
    for (var x = 0; x < RENDER_SIZE; ++x) {
        var color = access.getPixel(x, y);

        if (Math.abs(color[0] - refColor[0]) > colorThreshold[0] ||
            Math.abs(color[1] - refColor[1]) > colorThreshold[1] ||
            Math.abs(color[2] - refColor[2]) > colorThreshold[2]) {

            // first error
            if (!error)
                debug('Found invalid pixel(s). Pixel at (' + x + ', ' + y + ') color: ' + color);

            error = true;
            errorMask.setPixel(x, y, [1, 0, 0, 1]);
        }
    }

    if (!error)
        bufferedLogToConsole('Rendered image is valid.');
    else {
        tcuLogImage.logImage('Result', '', access);
        tcuLogImage.logImage('Error mask', '', errorMask.getAccess());
    }

    return !error;
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fDefaultVertexAttributeTests.DefaultVertexAttributeTests = function() {
    tcuTestCase.DeqpTest.call(this, 'default_vertex_attrib', 'Test default vertex attributes');
};

es3fDefaultVertexAttributeTests.DefaultVertexAttributeTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fDefaultVertexAttributeTests.DefaultVertexAttributeTests.prototype.constructor = es3fDefaultVertexAttributeTests.DefaultVertexAttributeTests;

es3fDefaultVertexAttributeTests.DefaultVertexAttributeTests.prototype.init = function() {
    var tests = this;
    var floatTargets = [
        ['float', gluShaderUtil.DataType.FLOAT, false],
        ['vec2', gluShaderUtil.DataType.FLOAT_VEC2, true],
        ['vec3', gluShaderUtil.DataType.FLOAT_VEC3, true],
        ['vec4', gluShaderUtil.DataType.FLOAT_VEC4, false],
        ['mat2', gluShaderUtil.DataType.FLOAT_MAT2, true],
        ['mat2x3', gluShaderUtil.DataType.FLOAT_MAT2X3, true],
        ['mat2x4', gluShaderUtil.DataType.FLOAT_MAT2X4, true],
        ['mat3', gluShaderUtil.DataType.FLOAT_MAT3, true],
        ['mat3x2', gluShaderUtil.DataType.FLOAT_MAT3X2, true],
        ['mat3x4', gluShaderUtil.DataType.FLOAT_MAT3X4, true],
        ['mat4', gluShaderUtil.DataType.FLOAT_MAT4, false],
        ['mat4x2', gluShaderUtil.DataType.FLOAT_MAT4X2, true],
        ['mat4x3', gluShaderUtil.DataType.FLOAT_MAT4X3, true]
    ];

    floatTargets.forEach(function(elem) {
        var name = elem[0];
        var dataType = elem[1];
        var reduced = elem[2];
        var group = new tcuTestCase.DeqpTest(name, 'test with ' + name);
        tests.addChild(group);
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib1f, dataType));
        if (!reduced) {
            group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib2f, dataType));
            group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib3f, dataType));
        }
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib4f, dataType));

        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib1fv, dataType));
        if (!reduced) {
            group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib2fv, dataType));
            group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib3fv, dataType));
        }
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttrib4fv, dataType));

    });

    var intTargets = [
        ['int', gluShaderUtil.DataType.INT, false],
        ['ivec2', gluShaderUtil.DataType.INT_VEC2, true],
        ['ivec3', gluShaderUtil.DataType.INT_VEC3, true],
        ['ivec4', gluShaderUtil.DataType.INT_VEC4, false]
    ];

   intTargets.forEach(function(elem) {
        var name = elem[0];
        var dataType = elem[1];
        var reduced = elem[2];
        var group = new tcuTestCase.DeqpTest(name, 'test with ' + name);
        tests.addChild(group);
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttribI4i, dataType));
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttribI4iv, dataType));
    });

    var uintTargets = [
        ['uint', gluShaderUtil.DataType.UINT, false],
        ['uvec2', gluShaderUtil.DataType.UINT_VEC2, true],
        ['uvec3', gluShaderUtil.DataType.UINT_VEC3, true],
        ['uvec4', gluShaderUtil.DataType.UINT_VEC4, false]
    ];

   uintTargets.forEach(function(elem) {
        var name = elem[0];
        var dataType = elem[1];
        var reduced = elem[2];
        var group = new tcuTestCase.DeqpTest(name, 'test with ' + name);
        tests.addChild(group);
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttribI4ui, dataType));
        group.addChild(new es3fDefaultVertexAttributeTests.AttributeCase(es3fDefaultVertexAttributeTests.LoaderVertexAttribI4uiv, dataType));
    });

};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fDefaultVertexAttributeTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fDefaultVertexAttributeTests.DefaultVertexAttributeTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fDefaultVertexAttributeTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

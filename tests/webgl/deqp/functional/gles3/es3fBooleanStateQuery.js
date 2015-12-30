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
goog.provide('functional.gles3.es3fBooleanStateQuery');
goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fBooleanStateQuery = functional.gles3.es3fBooleanStateQuery;
var tcuTestCase = framework.common.tcuTestCase;
var glsStateQuery = modules.shared.glsStateQuery;
var es3fApiCase = functional.gles3.es3fApiCase;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {number} targetName
 * @param {boolean} value
 */
es3fBooleanStateQuery.IsEnabledStateTestCase = function(name, description, targetName, value) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_targetName = targetName;
    this.m_initial = value;
};

setParentClass(es3fBooleanStateQuery.IsEnabledStateTestCase, es3fApiCase.ApiCase);

es3fBooleanStateQuery.IsEnabledStateTestCase.prototype.test = function() {
    // check inital value
    this.m_pass &= glsStateQuery.verify(this.m_targetName, this.m_initial);

    // check toggle

    gl.enable(this.m_targetName);

    this.m_pass &= glsStateQuery.verify(this.m_targetName, true);

    gl.disable(this.m_targetName);

    this.m_pass &= glsStateQuery.verify(this.m_targetName, false);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fBooleanStateQuery.DepthWriteMaskTestCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fBooleanStateQuery.DepthWriteMaskTestCase, es3fApiCase.ApiCase);

es3fBooleanStateQuery.DepthWriteMaskTestCase.prototype.test = function() {
    this.m_pass &= glsStateQuery.verify(gl.DEPTH_WRITEMASK, true);

    gl.depthMask(false);
    this.m_pass &= glsStateQuery.verify(gl.DEPTH_WRITEMASK, false);

    gl.depthMask(true);
    this.m_pass &= glsStateQuery.verify(gl.DEPTH_WRITEMASK, true);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fBooleanStateQuery.SampleCoverageInvertTestCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fBooleanStateQuery.SampleCoverageInvertTestCase, es3fApiCase.ApiCase);

es3fBooleanStateQuery.SampleCoverageInvertTestCase.prototype.test = function() {
    this.m_pass &= glsStateQuery.verify(gl.SAMPLE_COVERAGE_INVERT, false);

    gl.sampleCoverage(1, true);
    this.m_pass &= glsStateQuery.verify(gl.SAMPLE_COVERAGE_INVERT, true);

    gl.sampleCoverage(1, false);
    this.m_pass &= glsStateQuery.verify(gl.SAMPLE_COVERAGE_INVERT, false);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {number} targetName
 * @param {boolean} value
 */
es3fBooleanStateQuery.InitialBooleanTestCase = function(name, description, targetName, value) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_targetName = targetName;
    this.m_initial = value;
};

setParentClass(es3fBooleanStateQuery.InitialBooleanTestCase, es3fApiCase.ApiCase);

es3fBooleanStateQuery.InitialBooleanTestCase.prototype.test = function() {
    // check inital value
    this.m_pass &= glsStateQuery.verify(this.m_targetName, this.m_initial);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fBooleanStateQuery.ColorMaskTestCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fBooleanStateQuery.ColorMaskTestCase, es3fApiCase.ApiCase);

es3fBooleanStateQuery.ColorMaskTestCase.prototype.test = function() {
    this.m_pass &= glsStateQuery.verify(gl.COLOR_WRITEMASK, [true, true, true, true]);

    var testMasks = [
        [true, true, true, true],
        [true, true, true, false],
        [true, true, false, true],
        [true, true, false, false],
        [true, false, true, true],
        [true, false, true, false],
        [true, false, false, true],
        [true, false, false, false],
        [false, true, true, true],
        [false, true, true, false],
        [false, true, false, true],
        [false, true, false, false],
        [false, false, true, true],
        [false, false, true, false],
        [false, false, false, true],
        [false, false, false, false]
    ];

    for (var ndx = 0; ndx < testMasks.length; ndx++) {
        var mask = testMasks[ndx];
        gl.colorMask(mask[0], mask[1], mask[2], mask[3]);
        this.m_pass &= glsStateQuery.verify(gl.COLOR_WRITEMASK, mask);
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fBooleanStateQuery.TransformFeedbackTestCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    /** @type {WebGLTransformFeedback} */ this.m_transformfeedback = null;
};

setParentClass(es3fBooleanStateQuery.TransformFeedbackTestCase, es3fApiCase.ApiCase);

es3fBooleanStateQuery.TransformFeedbackTestCase.prototype.testTransformFeedback = function() {
    throw new Error('Virtual function.');
};

es3fBooleanStateQuery.TransformFeedbackTestCase.prototype.test = function() {
    var transformFeedbackTestVertSource = '#version 300 es\n' +
                                                'void main (void)\n' +
                                                '{\n' +
                                                ' gl_Position = vec4(0.0);\n' +
                                                '}\n';
    var transformFeedbackTestFragSource = '#version 300 es\n' +
                                                'layout(location = 0) out mediump vec4 fragColor;' +
                                                'void main (void)\n' +
                                                '{\n' +
                                                ' fragColor = vec4(0.0);\n' +
                                                '}\n';

    this.m_transformfeedback = gl.createTransformFeedback();

    var shaderVert = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(shaderVert, transformFeedbackTestVertSource);
    gl.compileShader(shaderVert);
    this.m_pass &= glsStateQuery.verifyShader(shaderVert, gl.COMPILE_STATUS, true);

    var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(shaderFrag, transformFeedbackTestFragSource);
    gl.compileShader(shaderFrag);
    this.m_pass &= glsStateQuery.verifyShader(shaderFrag, gl.COMPILE_STATUS, true);

    var shaderProg = gl.createProgram();
    gl.attachShader(shaderProg, shaderVert);
    gl.attachShader(shaderProg, shaderFrag);
    var transform_feedback_outputs = ['gl_Position'];
    gl.transformFeedbackVaryings(shaderProg, transform_feedback_outputs, gl.INTERLEAVED_ATTRIBS);
    gl.linkProgram(shaderProg);
    this.m_pass &= glsStateQuery.verifyProgram(shaderProg, gl.LINK_STATUS, true);

    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, this.m_transformfeedback);

    var buffer = gl.createBuffer();
    gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buffer);
    gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 16, gl.DYNAMIC_READ);
    gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buffer);

    gl.useProgram(shaderProg);

    this.testTransformFeedback();

    gl.useProgram(null);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
    gl.deleteTransformFeedback(this.m_transformfeedback);
    gl.deleteBuffer(buffer);
    gl.deleteShader(shaderVert);
    gl.deleteShader(shaderFrag);
    gl.deleteProgram(shaderProg);
};

/**
 * @constructor
 * @extends {es3fBooleanStateQuery.TransformFeedbackTestCase}
 * @param {string} name
 */
es3fBooleanStateQuery.TransformFeedbackBasicTestCase = function(name) {
    es3fBooleanStateQuery.TransformFeedbackTestCase.call(this, name, 'Test TRANSFORM_FEEDBACK_ACTIVE and TRANSFORM_FEEDBACK_PAUSED');
};

setParentClass(es3fBooleanStateQuery.TransformFeedbackBasicTestCase, es3fBooleanStateQuery.TransformFeedbackTestCase);

es3fBooleanStateQuery.TransformFeedbackBasicTestCase.prototype.testTransformFeedback = function() {
    gl.beginTransformFeedback(gl.POINTS);

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, true);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, false);

    gl.pauseTransformFeedback();

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, true);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, true);

    gl.resumeTransformFeedback();

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, true);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, false);

    gl.endTransformFeedback();

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, false);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, false);
};

/**
 * @constructor
 * @extends {es3fBooleanStateQuery.TransformFeedbackTestCase}
 * @param {string} name
 */
es3fBooleanStateQuery.TransformFeedbackImplicitResumeTestCase = function(name) {
    es3fBooleanStateQuery.TransformFeedbackTestCase.call(this, name, 'EndTransformFeedback performs an implicit ResumeTransformFeedback.');
};

setParentClass(es3fBooleanStateQuery.TransformFeedbackImplicitResumeTestCase, es3fBooleanStateQuery.TransformFeedbackTestCase);

es3fBooleanStateQuery.TransformFeedbackImplicitResumeTestCase.prototype.testTransformFeedback = function() {
    gl.beginTransformFeedback(gl.POINTS);

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, true);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, false);

    gl.pauseTransformFeedback();

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, true);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, true);

    gl.endTransformFeedback();

    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_ACTIVE, false);
    this.m_pass &= glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_PAUSED, false);
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fBooleanStateQuery.BooleanStateQuery = function() {
    tcuTestCase.DeqpTest.call(this, 'boolean', 'Boolean State Query tests');
};

es3fBooleanStateQuery.BooleanStateQuery.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fBooleanStateQuery.BooleanStateQuery.prototype.constructor = es3fBooleanStateQuery.BooleanStateQuery;

es3fBooleanStateQuery.BooleanStateQuery.prototype.init = function() {
    var testRoot = this;
    var isEnableds = [
        ['rasterizer_discard', 'RASTERIZER_DISCARD', gl.RASTERIZER_DISCARD, false],
        ['cull_face', 'CULL_FACE', gl.CULL_FACE, false],
        ['polygon_offset_fill', 'POLYGON_OFFSET_FILL', gl.POLYGON_OFFSET_FILL, false],
        ['sample_alpha_to_coverage', 'SAMPLE_ALPHA_TO_COVERAGE', gl.SAMPLE_ALPHA_TO_COVERAGE, false],
        ['sample_coverage', 'SAMPLE_COVERAGE', gl.SAMPLE_COVERAGE, false],
        ['scissor_test', 'SCISSOR_TEST', gl.SCISSOR_TEST, false],
        ['stencil_test', 'STENCIL_TEST', gl.STENCIL_TEST, false],
        ['depth_test', 'DEPTH_TEST', gl.DEPTH_TEST, false],
        ['blend', 'BLEND', gl.BLEND, false],
        ['dither', 'DITHER', gl.DITHER, true]
    ];
    isEnableds.forEach(function(elem) {
        var name = elem[0];
        var description = elem[1];
        var targetName = elem[2];
        var value = elem[3];
        testRoot.addChild(new es3fBooleanStateQuery.IsEnabledStateTestCase(name, description, targetName, value));
    });

    testRoot.addChild(new es3fBooleanStateQuery.ColorMaskTestCase('color_writemask', 'COLOR_WRITEMASK'));
    testRoot.addChild(new es3fBooleanStateQuery.DepthWriteMaskTestCase('depth_writemask', 'DEPTH_WRITEMASK'));
    testRoot.addChild(new es3fBooleanStateQuery.SampleCoverageInvertTestCase('sample_coverage_invert', 'SAMPLE_COVERAGE_INVERT'));
    testRoot.addChild(new es3fBooleanStateQuery.InitialBooleanTestCase('transform_feedback_active_initial', 'initial TRANSFORM_FEEDBACK_ACTIVE', gl.TRANSFORM_FEEDBACK_ACTIVE, false));
    testRoot.addChild(new es3fBooleanStateQuery.InitialBooleanTestCase('transform_feedback_paused_initial', 'initial TRANSFORM_FEEDBACK_PAUSED', gl.TRANSFORM_FEEDBACK_PAUSED, false));
    testRoot.addChild(new es3fBooleanStateQuery.TransformFeedbackBasicTestCase('transform_feedback'));
    testRoot.addChild(new es3fBooleanStateQuery.TransformFeedbackImplicitResumeTestCase('transform_feedback_implicit_resume'));
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fBooleanStateQuery.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fBooleanStateQuery.BooleanStateQuery());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fBooleanStateQuery.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

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
goog.provide('functional.gles3.es3fSamplerStateQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fSamplerStateQueryTests = functional.gles3.es3fSamplerStateQueryTests;
var tcuTestCase = framework.common.tcuTestCase;
var glsStateQuery = modules.shared.glsStateQuery;
var es3fApiCase = functional.gles3.es3fApiCase;
var deRandom = framework.delibs.debase.deRandom;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fSamplerStateQueryTests.SamplerCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    /** @type {WebGLSampler} */ this.m_sampler;
};

setParentClass(es3fSamplerStateQueryTests.SamplerCase, es3fApiCase.ApiCase);

es3fSamplerStateQueryTests.SamplerCase.prototype.testSampler = function() {
    throw new Error('Virtual function. Please override.');
};

es3fSamplerStateQueryTests.SamplerCase.prototype.test = function() {
    this.m_sampler = gl.createSampler();

    this.testSampler();

    gl.deleteSampler(this.m_sampler);
};

/**
 * @constructor
 * @extends {es3fSamplerStateQueryTests.SamplerCase}
 * @param {string} name
 * @param {string} description
 * @param {number} valueName
 * @param {number} initialValue
 * @param {Array<number>} valueRange
 */
es3fSamplerStateQueryTests.SamplerModeCase = function(name, description, valueName, initialValue, valueRange) {
    es3fSamplerStateQueryTests.SamplerCase.call(this, name, description);
    this.m_valueName = valueName;
    this.m_initialValue = initialValue;
    this.m_valueRange = valueRange;
};

setParentClass(es3fSamplerStateQueryTests.SamplerModeCase, es3fSamplerStateQueryTests.SamplerCase);

es3fSamplerStateQueryTests.SamplerModeCase.prototype.testSampler = function() {
    this.check(glsStateQuery.verifySampler(this.m_sampler, this.m_valueName, this.m_initialValue));

    for (var ndx = 0; ndx < this.m_valueRange.length; ++ndx) {
        gl.samplerParameteri(this.m_sampler, this.m_valueName, this.m_valueRange[ndx]);

        this.check(glsStateQuery.verifySampler(this.m_sampler, this.m_valueName, this.m_valueRange[ndx]));
    }

    //check unit conversions with float

    for (var ndx = 0; ndx < this.m_valueRange.length; ++ndx) {
        gl.samplerParameterf(this.m_sampler, this.m_valueName, this.m_valueRange[ndx]);

        this.check(glsStateQuery.verifySampler(this.m_sampler, this.m_valueName, this.m_valueRange[ndx]));
    }
};

/**
 * @constructor
 * @extends {es3fSamplerStateQueryTests.SamplerCase}
 * @param {string} name
 * @param {string} description
 * @param {number} lodTarget
 * @param {number} initialValue
 */
es3fSamplerStateQueryTests.SamplerLODCase = function(name, description, lodTarget, initialValue) {
    es3fSamplerStateQueryTests.SamplerCase.call(this, name, description);
    this.m_lodTarget = lodTarget;
    this.m_initialValue = initialValue;
};

setParentClass(es3fSamplerStateQueryTests.SamplerLODCase, es3fSamplerStateQueryTests.SamplerCase);

es3fSamplerStateQueryTests.SamplerLODCase.prototype.testSampler = function() {
    var rnd = new deRandom.Random(0xabcdef);

    this.check(glsStateQuery.verifySampler(this.m_sampler, this.m_lodTarget, this.m_initialValue));
    var numIterations = 60;
    for (var ndx = 0; ndx < numIterations; ++ndx) {
        var ref = rnd.getFloat(-64000, 64000);

        gl.samplerParameterf(this.m_sampler, this.m_lodTarget, ref);

        this.check(glsStateQuery.verifySampler(this.m_sampler, this.m_lodTarget, ref));
    }

    // check unit conversions with int

    for (var ndx = 0; ndx < numIterations; ++ndx) {
        var ref = rnd.getInt(-64000, 64000);

        gl.samplerParameteri(this.m_sampler, this.m_lodTarget, ref);

        this.check(glsStateQuery.verifySampler(this.m_sampler, this.m_lodTarget, ref));
    }
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fSamplerStateQueryTests.SamplerStateQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'sampler', 'Sampler State Query tests');
};

es3fSamplerStateQueryTests.SamplerStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fSamplerStateQueryTests.SamplerStateQueryTests.prototype.constructor = es3fSamplerStateQueryTests.SamplerStateQueryTests;

es3fSamplerStateQueryTests.SamplerStateQueryTests.prototype.init = function() {
    var wrapValues = [gl.CLAMP_TO_EDGE, gl.REPEAT, gl.MIRRORED_REPEAT];
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_wrap_s' , 'TEXTURE_WRAP_S',
        gl.TEXTURE_WRAP_S, gl.REPEAT, wrapValues));
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_wrap_t' , 'TEXTURE_WRAP_T',
        gl.TEXTURE_WRAP_T, gl.REPEAT, wrapValues));
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_wrap_r' , 'TEXTURE_WRAP_R',
        gl.TEXTURE_WRAP_R, gl.REPEAT, wrapValues));

    var magValues = [gl.NEAREST, gl.LINEAR];
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_mag_filter' , 'TEXTURE_MAG_FILTER',
        gl.TEXTURE_MAG_FILTER, gl.LINEAR, magValues));

    var minValues = [gl.NEAREST, gl.LINEAR, gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST_MIPMAP_LINEAR, gl.LINEAR_MIPMAP_NEAREST, gl.LINEAR_MIPMAP_LINEAR];
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_min_filter' , 'TEXTURE_MIN_FILTER',
        gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_LINEAR, minValues));

    this.addChild(new es3fSamplerStateQueryTests.SamplerLODCase('sampler_texture_min_lod' , 'TEXTURE_MIN_LOD', gl.TEXTURE_MIN_LOD, -1000));
    this.addChild(new es3fSamplerStateQueryTests.SamplerLODCase('sampler_texture_max_lod' , 'TEXTURE_MAX_LOD', gl.TEXTURE_MAX_LOD, 1000));

    var modes = [gl.COMPARE_REF_TO_TEXTURE, gl.NONE];
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_compare_mode' , 'TEXTURE_COMPARE_MODE',
        gl.TEXTURE_COMPARE_MODE, gl.NONE, modes));

    var compareFuncs = [gl.LEQUAL, gl.GEQUAL, gl.LESS, gl.GREATER, gl.EQUAL, gl.NOTEQUAL, gl.ALWAYS, gl.NEVER];
    this.addChild(new es3fSamplerStateQueryTests.SamplerModeCase('sampler_texture_compare_func' , 'TEXTURE_COMPARE_FUNC',
        gl.TEXTURE_COMPARE_FUNC, gl.LEQUAL, compareFuncs));
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fSamplerStateQueryTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fSamplerStateQueryTests.SamplerStateQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fSamplerStateQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

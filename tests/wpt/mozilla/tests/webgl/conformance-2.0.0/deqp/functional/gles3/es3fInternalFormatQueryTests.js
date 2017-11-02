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
goog.provide('functional.gles3.es3fInternalFormatQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fInternalFormatQueryTests = functional.gles3.es3fInternalFormatQueryTests;
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
 * @param {number} internalFormat
 * @param {boolean} isIntegerInternalFormat
 */
es3fInternalFormatQueryTests.SamplesCase = function(name, description, internalFormat, isIntegerInternalFormat) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_internalFormat = internalFormat;
    this.m_isIntegerInternalFormat = isIntegerInternalFormat;
};

setParentClass(es3fInternalFormatQueryTests.SamplesCase, es3fApiCase.ApiCase);

es3fInternalFormatQueryTests.SamplesCase.prototype.test = function() {
    var samples = gl.getInternalformatParameter(gl.RENDERBUFFER, this.m_internalFormat, gl.SAMPLES);

    this.check(!this.m_isIntegerInternalFormat || samples.length == 0, 'integer internal format should have 0 samples, got ' + samples.length);

    if (samples.length == 0)
        return;

    var prevSampleCount = 0;
    var sampleCount = 0;
    for (var ndx = 0; ndx < samples.length; ++ndx, prevSampleCount = sampleCount) {
        sampleCount = samples[ndx];

        // sample count must be > 0
        this.check(sampleCount > 0, 'Expected sample count to be at least one; got ' + sampleCount);

        // samples must be ordered descending
        this.check(ndx == 0 || sampleCount < prevSampleCount, 'Expected sample count to be ordered in descending order; got ' + prevSampleCount + ' at index ' + (ndx - 1) + ', and ' + sampleCount + ' at index ' + ndx);
    }

    // the maximum value in SAMPLES is guaranteed to be at least the value of MAX_SAMPLES
    var maxSamples = /** @type {number} */ (gl.getParameter(gl.MAX_SAMPLES));
    var maximumFormatSampleCount = samples[0];
    this.check(maximumFormatSampleCount >= maxSamples, 'Expected maximum value in SAMPLES (' + maximumFormatSampleCount + ') to be at least the value of MAX_SAMPLES (' + maxSamples + ')');
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fInternalFormatQueryTests.InternalFormatQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'internal_format', 'Internal Format Query tests');
};

es3fInternalFormatQueryTests.InternalFormatQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fInternalFormatQueryTests.InternalFormatQueryTests.prototype.constructor = es3fInternalFormatQueryTests.InternalFormatQueryTests;

es3fInternalFormatQueryTests.InternalFormatQueryTests.prototype.init = function() {
    var internalFormats = [
        //name, format, is_integer
        // color renderable and unsized
        // \note These unsized formats seem to allowed by the spec, but they are not useful in any way. (You can't create a renderbuffer with such internalFormat)
        ['rgba', gl.RGBA, false],
        ['rgb', gl.RGB, false],

        // color renderable
        ['r8', gl.R8, false],
        ['rg8', gl.RG8, false],
        ['rgb8', gl.RGB8, false],
        ['rgb565', gl.RGB565, false],
        ['rgba4', gl.RGBA4, false],
        ['rgb5_a1', gl.RGB5_A1, false],
        ['rgba8', gl.RGBA8, false],
        ['rgb10_a2', gl.RGB10_A2, false],
        ['rgb10_a2ui', gl.RGB10_A2UI, true],
        ['srgb8_alpha8', gl.SRGB8_ALPHA8, false],
        ['r8i', gl.R8I, true],
        ['r8ui', gl.R8UI, true],
        ['r16i', gl.R16I, true],
        ['r16ui', gl.R16UI, true],
        ['r32i', gl.R32I, true],
        ['r32ui', gl.R32UI, true],
        ['rg8i', gl.RG8I, true],
        ['rg8ui', gl.RG8UI, true],
        ['rg16i', gl.RG16I, true],
        ['rg16ui', gl.RG16UI, true],
        ['rg32i', gl.RG32I, true],
        ['rg32ui', gl.RG32UI, true],
        ['rgba8i', gl.RGBA8I, true],
        ['rgba8ui', gl.RGBA8UI, true],
        ['rgba16i', gl.RGBA16I, true],
        ['rgba16ui', gl.RGBA16UI, true],
        ['rgba32i', gl.RGBA32I, true],
        ['rgba32ui', gl.RGBA32UI, true],

        // depth renderable
        ['depth_component16', gl.DEPTH_COMPONENT16, false],
        ['depth_component24', gl.DEPTH_COMPONENT24, false],
        ['depth_component32f', gl.DEPTH_COMPONENT32F, false],
        ['depth24_stencil8', gl.DEPTH24_STENCIL8, false],
        ['depth32f_stencil8', gl.DEPTH32F_STENCIL8, false],

        // stencil renderable
        ['stencil_index8', gl.STENCIL_INDEX8, false]
        // DEPTH24_STENCIL8, duplicate
        // DEPTH32F_STENCIL8 duplicate
    ];

    for (var ndx = 0; ndx < internalFormats.length; ++ndx) {
        var internalFormat = internalFormats[ndx];

        this.addChild(new es3fInternalFormatQueryTests.SamplesCase(internalFormat[0] + '_samples', 'SAMPLES and NUM_SAMPLE_COUNTS', internalFormat[1], internalFormat[2]));
    }
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fInternalFormatQueryTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fInternalFormatQueryTests.InternalFormatQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fInternalFormatQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

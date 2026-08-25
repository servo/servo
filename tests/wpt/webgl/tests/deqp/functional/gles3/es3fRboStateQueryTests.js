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
goog.provide('functional.gles3.es3fRboStateQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fRboStateQueryTests = functional.gles3.es3fRboStateQueryTests;
var tcuTestCase = framework.common.tcuTestCase;
var glsStateQuery = modules.shared.glsStateQuery;
var es3fApiCase = functional.gles3.es3fApiCase;
var deRandom = framework.delibs.debase.deRandom;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @this {es3fApiCase.ApiCase}
 */
var checkRenderbufferComponentSize = function(r, g, b, a, d, s) {
    var referenceSizes = [r, g, b, a, d, s];
    var paramNames = [
        gl.RENDERBUFFER_RED_SIZE,
        gl.RENDERBUFFER_GREEN_SIZE,
        gl.RENDERBUFFER_BLUE_SIZE,
        gl.RENDERBUFFER_ALPHA_SIZE,
        gl.RENDERBUFFER_DEPTH_SIZE,
        gl.RENDERBUFFER_STENCIL_SIZE
    ];

    for (var ndx = 0; ndx < referenceSizes.length; ++ndx) {
        if (referenceSizes[ndx] == -1)
            continue;
        var value = /** @type {number} */ (gl.getRenderbufferParameter(gl.RENDERBUFFER, paramNames[ndx]));

        this.check(value >= referenceSizes[ndx], 'Expected greater or equal to ' + referenceSizes[ndx] + ' got ' + value);
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fRboStateQueryTests.RboSizeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fRboStateQueryTests.RboSizeCase, es3fApiCase.ApiCase);

es3fRboStateQueryTests.RboSizeCase.prototype.test = function() {
    var rnd = new deRandom.Random(0xabcdef);

    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_WIDTH, 0));
    this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_HEIGHT, 0));

    var numIterations = 60;
    for (var i = 0; i < numIterations; ++i) {
        var w = rnd.getInt(0, 128);
        var h = rnd.getInt(0, 128);

        gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB8, w, h);

        this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_WIDTH, w));
        this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_HEIGHT, h));
    }
    gl.deleteRenderbuffer(renderbufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fRboStateQueryTests.RboInternalFormatCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fRboStateQueryTests.RboInternalFormatCase, es3fApiCase.ApiCase);

es3fRboStateQueryTests.RboInternalFormatCase.prototype.test = function() {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_INTERNAL_FORMAT, gl.RGBA4));

    var requiredColorformats = [
        gl.R8, gl.RG8, gl.RGB8, gl.RGB565, gl.RGBA4, gl.RGB5_A1, gl.RGBA8, gl.RGB10_A2,
        gl.RGB10_A2UI, gl.SRGB8_ALPHA8, gl.R8I, gl.R8UI, gl.R16I, gl.R16UI, gl.R32I, gl.R32UI,
        gl.RG8I, gl.RG8UI, gl.RG16I, gl.RG16UI, gl.RG32I, gl.RG32UI, gl.RGBA8I, gl.RGBA8UI,
        gl.RGBA16I, gl.RGBA16UI, gl.RGBA32I, gl.RGBA32UI
    ];

    for (var ndx = 0; ndx < requiredColorformats.length; ++ndx) {
        gl.renderbufferStorage(gl.RENDERBUFFER, requiredColorformats[ndx], 128, 128);

        this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_INTERNAL_FORMAT, requiredColorformats[ndx]));
    }
    gl.deleteRenderbuffer(renderbufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fRboStateQueryTests.RboComponentSizeColorCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fRboStateQueryTests.RboComponentSizeColorCase, es3fApiCase.ApiCase);

es3fRboStateQueryTests.RboComponentSizeColorCase.prototype.test = function() {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    checkRenderbufferComponentSize.bind(this, 0, 0, 0, 0, 0, 0);

    var requiredColorFormats = [
        // format, r, g, b, a
        [gl.R8, 8, 0, 0, 0],
        [gl.RG8, 8, 8, 0, 0],
        [gl.RGB8, 8, 8, 8, 0],
        [gl.RGB565, 5, 6, 5, 0],
        [gl.RGBA4, 4, 4, 4, 4],
        [gl.RGB5_A1, 5, 5, 5, 1],
        [gl.RGBA8, 8, 8, 8, 8],
        [gl.RGB10_A2, 10, 10, 10, 2],
        [gl.RGB10_A2UI, 10, 10, 10, 2],
        [gl.SRGB8_ALPHA8, 8, 8, 8, 8],
        [gl.R8I, 8, 0, 0, 0],
        [gl.R8UI, 8, 0, 0, 0],
        [gl.R16I, 16, 0, 0, 0],
        [gl.R16UI, 16, 0, 0, 0],
        [gl.R32I, 32, 0, 0, 0],
        [gl.R32UI, 32, 0, 0, 0],
        [gl.RG8I, 8, 8, 0, 0],
        [gl.RG8UI, 8, 8, 0, 0],
        [gl.RG16I, 16, 16, 0, 0],
        [gl.RG16UI, 16, 16, 0, 0],
        [gl.RG32I, 32, 32, 0, 0],
        [gl.RG32UI, 32, 32, 0, 0],
        [gl.RGBA8I, 8, 8, 8, 8],
        [gl.RGBA8UI, 8, 8, 8, 8],
        [gl.RGBA16I, 16, 16, 16, 16],
        [gl.RGBA16UI, 16, 16, 16, 16],
        [gl.RGBA32I, 32, 32, 32, 32],
        [gl.RGBA32UI, 32, 32, 32, 32]
    ];

    for (var ndx = 0; ndx < requiredColorFormats.length; ++ndx) {
        gl.renderbufferStorage(gl.RENDERBUFFER, requiredColorFormats[ndx][0], 128, 128);

        checkRenderbufferComponentSize.bind(this, requiredColorFormats[ndx][1], requiredColorFormats[ndx][2], requiredColorFormats[ndx][3], requiredColorFormats[ndx][4], -1, -1);
    }
    gl.deleteRenderbuffer(renderbufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fRboStateQueryTests.RboComponentSizeDepthCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fRboStateQueryTests.RboComponentSizeDepthCase, es3fApiCase.ApiCase);

es3fRboStateQueryTests.RboComponentSizeDepthCase.prototype.test = function() {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    var requiredDepthFormats = [
        // format, depth, stencil
        [gl.DEPTH_COMPONENT16, 16, 0],
        [gl.DEPTH_COMPONENT24, 24, 0],
        [gl.DEPTH_COMPONENT32F, 32, 0],
        [gl.DEPTH24_STENCIL8, 24, 8],
        [gl.DEPTH32F_STENCIL8, 32, 8]
    ];

    for (var ndx = 0; ndx < requiredDepthFormats.length; ++ndx) {
        gl.renderbufferStorage(gl.RENDERBUFFER, requiredDepthFormats[ndx][0], 128, 128);

        checkRenderbufferComponentSize.bind(this, -1, -1, -1, -1, requiredDepthFormats[ndx][1], requiredDepthFormats[ndx][2]);
    }

    // STENCIL_INDEX8 is required, in that case sBits >= 8
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.STENCIL_INDEX8, 128, 128);

    var value = /** @type {number} */ (gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_STENCIL_SIZE));
    this.check(value >= 8, 'Expected greater or equal to 8; got ' + value);

    gl.deleteRenderbuffer(renderbufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fRboStateQueryTests.RboSamplesCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fRboStateQueryTests.RboSamplesCase, es3fApiCase.ApiCase);

es3fRboStateQueryTests.RboSamplesCase.prototype.test = function() {
    var rnd = new deRandom.Random(0xabcdef);

    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_SAMPLES, 0));

    var max_samples = /** @type {number} */ (gl.getParameter(gl.MAX_SAMPLES));

    // 0 samples is a special case
    gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 0, gl.RGBA8, 128, 128);

    this.check(glsStateQuery.verifyRenderbuffer(gl.RENDERBUFFER_SAMPLES, 0));

    // test [1, n] samples
    for (var samples = 1; samples <= max_samples; ++samples) {
        gl.renderbufferStorageMultisample(gl.RENDERBUFFER, samples, gl.RGBA8, 128, 128);
        var value = /** @type {number} */ (gl.getRenderbufferParameter(gl.RENDERBUFFER, gl.RENDERBUFFER_SAMPLES));
        this.check(value >= samples, 'Expected greater or equal to ' + samples + ' got ' + value);
    }

    gl.deleteRenderbuffer(renderbufferID);
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fRboStateQueryTests.RboStateQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'rbo', 'Rbo State Query tests');
};

es3fRboStateQueryTests.RboStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fRboStateQueryTests.RboStateQueryTests.prototype.constructor = es3fRboStateQueryTests.RboStateQueryTests;

es3fRboStateQueryTests.RboStateQueryTests.prototype.init = function() {
    this.addChild(new es3fRboStateQueryTests.RboSizeCase('renderbuffer_size', 'RENDERBUFFER_WIDTH and RENDERBUFFER_HEIGHT'));
    this.addChild(new es3fRboStateQueryTests.RboInternalFormatCase('renderbuffer_internal_format', 'RENDERBUFFER_INTERNAL_FORMAT'));
    this.addChild(new es3fRboStateQueryTests.RboComponentSizeColorCase('renderbuffer_component_size_color', 'RENDERBUFFER_x_SIZE'));
    this.addChild(new es3fRboStateQueryTests.RboComponentSizeDepthCase('renderbuffer_component_size_depth', 'RENDERBUFFER_x_SIZE'));
    this.addChild(new es3fRboStateQueryTests.RboSamplesCase('renderbuffer_samples', 'RENDERBUFFER_SAMPLES'));
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fRboStateQueryTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fRboStateQueryTests.RboStateQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fRboStateQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

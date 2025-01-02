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
goog.provide('functional.gles3.es3fBufferObjectQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {
var es3fBufferObjectQueryTests = functional.gles3.es3fBufferObjectQueryTests;
var tcuTestCase = framework.common.tcuTestCase;
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
es3fBufferObjectQueryTests.BufferCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fBufferObjectQueryTests.BufferCase, es3fApiCase.ApiCase);

es3fBufferObjectQueryTests.BufferCase.prototype.test = function() {
    var bufferTargets = [
        gl.ARRAY_BUFFER, gl.COPY_READ_BUFFER,
        gl.TRANSFORM_FEEDBACK_BUFFER, gl.UNIFORM_BUFFER,

        gl.COPY_WRITE_BUFFER, gl.ELEMENT_ARRAY_BUFFER,
        gl.PIXEL_PACK_BUFFER, gl.PIXEL_UNPACK_BUFFER
    ];

    // most test need only to be run with a subset of targets
   var targets = this.m_testAllTargets ? bufferTargets.length : 4;

    for (var ndx = 0; ndx < targets; ++ndx) {
        this.m_bufferTarget = bufferTargets[ndx];

        var bufferId = gl.createBuffer();
        gl.bindBuffer(this.m_bufferTarget, bufferId);

        this.testBuffer();

        gl.bindBuffer(this.m_bufferTarget, null);
        gl.deleteBuffer(bufferId);
    }
};

/**
 * @constructor
 * @extends {es3fBufferObjectQueryTests.BufferCase}
 * @param {string} name
 * @param {string} description
 */
es3fBufferObjectQueryTests.BufferSizeCase = function(name, description) {
    es3fBufferObjectQueryTests.BufferCase.call(this, name, description);
    this.m_testAllTargets = true;
};

setParentClass(es3fBufferObjectQueryTests.BufferSizeCase, es3fBufferObjectQueryTests.BufferCase);

es3fBufferObjectQueryTests.BufferSizeCase.prototype.testBuffer = function() {
    var rnd = new deRandom.Random(0xabcdef);

    var size = /** type {number} */ (gl.getBufferParameter(this.m_bufferTarget, gl.BUFFER_SIZE));
    this.check(size == 0, 'Initial size should be 0; got ' + size);

    var numIterations = 16;
    for (var i = 0; i < numIterations; ++i) {
        var len = rnd.getInt(0, 1024);
        gl.bufferData(this.m_bufferTarget, len, gl.STREAM_DRAW);

        size = /** type {number} */ (gl.getBufferParameter(this.m_bufferTarget, gl.BUFFER_SIZE));
        this.check(size == len, 'Buffer size should be ' + len + ' ; got ' + size);
    }
};

/**
 * @constructor
 * @extends {es3fBufferObjectQueryTests.BufferCase}
 * @param {string} name
 * @param {string} description
 */
es3fBufferObjectQueryTests.BufferUsageCase = function(name, description) {
    es3fBufferObjectQueryTests.BufferCase.call(this, name, description);
    this.m_testAllTargets = false;
};

setParentClass(es3fBufferObjectQueryTests.BufferUsageCase, es3fBufferObjectQueryTests.BufferCase);

es3fBufferObjectQueryTests.BufferUsageCase.prototype.testBuffer = function() {
    var usage = /** type {number} */ (gl.getBufferParameter(this.m_bufferTarget, gl.BUFFER_USAGE));
    this.check(usage == gl.STATIC_DRAW, 'Initial usage should be STATIC_DRAW; got ' + wtu.glEnumToString(gl, usage));

    var usages = [
        gl.STREAM_DRAW, gl.STREAM_READ,
        gl.STREAM_COPY, gl.STATIC_DRAW,
        gl.STATIC_READ, gl.STATIC_COPY,
        gl.DYNAMIC_DRAW, gl.DYNAMIC_READ,
        gl.DYNAMIC_COPY
    ];

    for (var ndx = 0; ndx < usages.length; ++ndx) {
        gl.bufferData(this.m_bufferTarget, 16, usages[ndx]);

        usage = /** type {number} */ (gl.getBufferParameter(this.m_bufferTarget, gl.BUFFER_USAGE));
        this.check(usage == usages[ndx], 'Buffer usage should be ' + wtu.glEnumToString(gl, usages[ndx]) + ' ; got ' + wtu.glEnumToString(gl, usage));
    }
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fBufferObjectQueryTests.BufferObjectQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'buffer_object', 'Buffer Object Query tests');
};

es3fBufferObjectQueryTests.BufferObjectQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fBufferObjectQueryTests.BufferObjectQueryTests.prototype.constructor = es3fBufferObjectQueryTests.BufferObjectQueryTests;

es3fBufferObjectQueryTests.BufferObjectQueryTests.prototype.init = function() {
    this.addChild(new es3fBufferObjectQueryTests.BufferSizeCase('buffer_size' , 'BUFFER_SIZE'));
    this.addChild(new es3fBufferObjectQueryTests.BufferUsageCase('buffer_usage' , 'BUFFER_USAGE'));
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fBufferObjectQueryTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fBufferObjectQueryTests.BufferObjectQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fBufferObjectQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

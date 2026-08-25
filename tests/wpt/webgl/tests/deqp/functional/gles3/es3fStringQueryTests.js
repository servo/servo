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
goog.provide('functional.gles3.es3fStringQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {
    var es3fStringQueryTests = functional.gles3.es3fStringQueryTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var es3fApiCase = functional.gles3.es3fApiCase;

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fStringQueryTests.StringQueryTests = function() {
        tcuTestCase.DeqpTest.call(this, 'string', 'String Query tests');
    };

    es3fStringQueryTests.StringQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fStringQueryTests.StringQueryTests.prototype.constructor = es3fStringQueryTests.StringQueryTests;

    es3fStringQueryTests.StringQueryTests.prototype.init = function() {
        this.addChild(new es3fApiCase.ApiCaseCallback('renderer', 'RENDERER', gl, function() {
            var string = /** @type {string} */ (gl.getParameter(gl.RENDERER));
            this.check(string !== null,
                'Got invalid string: ' + string);
        }));

        this.addChild(new es3fApiCase.ApiCaseCallback('vendor', 'VENDOR', gl, function() {
            var string = /** @type {string} */ (gl.getParameter(gl.VENDOR));
            this.check(string !== null,
                'Got invalid string: ' + string);
        }));

        this.addChild(new es3fApiCase.ApiCaseCallback('version', 'VERSION', gl, function() {
            var string = /** @type {string} */ (gl.getParameter(gl.VERSION));
            /** @type {string} */ var referenceString = 'WebGL 2.0';

            this.check(string !== null && string.indexOf(referenceString) === 0,
                'Got invalid string prefix: ' + string + ' expected: ' + referenceString);
        }));

        this.addChild(new es3fApiCase.ApiCaseCallback('shading_language_version', 'SHADING_LANGUAGE_VERSION', gl, function() {
            var string = /** @type {string} */ (gl.getParameter(gl.SHADING_LANGUAGE_VERSION));
            /** @type {string} */ var referenceString = 'WebGL GLSL ES 3.00';

            this.check(string !== null, 'Got invalid string');
            this.check(string.indexOf(referenceString) === 0, 'Got invalid string prefix');
        }));

        this.addChild(new es3fApiCase.ApiCaseCallback('extensions', 'EXTENSIONS', gl, function() {
            /** @type {Array<string>} */ var extensions = gl.getSupportedExtensions();
            this.check(extensions !== null, 'Got invalid string');

            // [dag] check that all extensions from gl.getSupportedExtensions() are found using gl.getExtension()
            for (var i in extensions) {
                /** @type {Object} */ var extension = gl.getExtension(extensions[i]);
                this.check(extension !== null,  'Advertised extension ' + extensions[i] + ' not found');
            }

            // [dag] check that gl.getExtension() returns null for items not in gl.getSupportedExtensions()
            this.check(gl.getExtension('Random_String') === null, 'Extension query methods are not consistent.');
        }));

    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fStringQueryTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fStringQueryTests.StringQueryTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fStringQueryTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

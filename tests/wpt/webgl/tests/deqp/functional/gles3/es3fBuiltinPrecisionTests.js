/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES 3.0 Module
 * -------------------------------------------------
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
 *//*!
 * \file
 * \brief Tests for precision and range of GLSL builtins and types.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('functional.gles3.es3fBuiltinPrecisionTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluShaderProgram');
goog.require('modules.shared.glsBuiltinPrecisionTests');

goog.scope(function() {

    var es3fBuiltinPrecisionTests = functional.gles3.es3fBuiltinPrecisionTests;
    var glsBuiltinPrecisionTests = modules.shared.glsBuiltinPrecisionTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderProgram = framework.opengl.gluShaderProgram;

    /**
     * @param {*} context
     * @param {number} caseId test case Id
     * @return {tcuTestCase.DeqpTest}
     */
    es3fBuiltinPrecisionTests.createBuiltinPrecisionTests = function(context, caseId) {
        /** @type {tcuTestCase.DeqpTest} */
        var group = tcuTestCase.newTest('precision', 'Builtin precision tests');

        /** @type {Array<gluShaderProgram.shaderType>} */ var shaderTypes = [];
        var es3Cases = glsBuiltinPrecisionTests.createES3BuiltinCases(caseId);

        shaderTypes.push(gluShaderProgram.shaderType.VERTEX);
        shaderTypes.push(gluShaderProgram.shaderType.FRAGMENT);

        glsBuiltinPrecisionTests.addBuiltinPrecisionTests(es3Cases, shaderTypes, group);
        return group;
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     * @param {number} caseId test case Id
     */
    es3fBuiltinPrecisionTests.run = function(context, caseId) {
        gl = context;
        // Set up root Test
        var state = tcuTestCase.runner;

        var test = es3fBuiltinPrecisionTests.createBuiltinPrecisionTests(context, caseId);
        var testName = test.fullName();
        var testDescription = test.getDescription() === undefined ? '' : test.getDescription();

        state.testName = testName;
        state.setRoot(test);
        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            test.init();
            //Run test cases
            tcuTestCase.runTestCases();
        } catch (err) {
            bufferedLogToConsole('Exception: ' + err);
            testFailedOptions('Failed to es3fAttribLocationTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

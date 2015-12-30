/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES 3.0 Module
 * -------------------------------------------------
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
 *//*!
 * \file
 * \brief Negative Fragment Pipe API tests.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('functional.gles3.es3fNegativeFragmentApiTests');
goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {

    var es3fNegativeFragmentApiTests = functional.gles3.es3fNegativeFragmentApiTests;
    var es3fApiCase = functional.gles3.es3fApiCase;
    var tcuTestCase = framework.common.tcuTestCase;

    /**
     * @param {WebGL2RenderingContext} gl
     */
    es3fNegativeFragmentApiTests.init = function(gl) {

        var testGroup = tcuTestCase.runner.testCases;

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('scissor', 'Invalid gl.scissor() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if either width or height is negative.');
            gl.scissor(0, 0, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.scissor(0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.scissor(0, 0, -1, -1);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('depth_func', 'Invalid gl.depthFunc() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if func is not an accepted value.');
            gl.depthFunc(-1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('viewport', 'Invalid gl.viewport() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if either width or height is negative.');
            gl.viewport(0, 0, -1, 1);
            this.expectError(gl.INVALID_VALUE);
            gl.viewport(0, 0, 1, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.viewport(0, 0, -1, -1);
            this.expectError(gl.INVALID_VALUE);

        }));

        // Stencil functions

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('stencil_func', 'Invalid gl.stencilFunc() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if func is not one of the eight accepted values.');
            gl.stencilFunc(-1, 0, 1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('stencil_func_separate', 'Invalid gl.stencilFuncSeparate() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if face is not gl.FRONT, gl.BACK, or gl.FRONT_AND_BACK.');
            gl.stencilFuncSeparate(-1, gl.NEVER, 0, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if func is not one of the eight accepted values.');
            gl.stencilFuncSeparate(gl.FRONT, -1, 0, 1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('stencil_op', 'Invalid gl.stencilOp() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if sfail, dpfail, or dppass is any value other than the defined symbolic constant values.');
            gl.stencilOp(-1, gl.ZERO, gl.REPLACE);
            this.expectError(gl.INVALID_ENUM);
            gl.stencilOp(gl.KEEP, -1, gl.REPLACE);
            this.expectError(gl.INVALID_ENUM);
            gl.stencilOp(gl.KEEP, gl.ZERO, -1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('stencil_op_separate', 'Invalid gl.stencilOpSeparate() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if face is any value other than gl.FRONT, gl.BACK, or gl.FRONT_AND_BACK.');
            gl.stencilOpSeparate(-1, gl.KEEP, gl.ZERO, gl.REPLACE);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if sfail, dpfail, or dppass is any value other than the eight defined symbolic constant values.');
            gl.stencilOpSeparate(gl.FRONT, -1, gl.ZERO, gl.REPLACE);
            this.expectError(gl.INVALID_ENUM);
            gl.stencilOpSeparate(gl.FRONT, gl.KEEP, -1, gl.REPLACE);
            this.expectError(gl.INVALID_ENUM);
            gl.stencilOpSeparate(gl.FRONT, gl.KEEP, gl.ZERO, -1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('stencil_mask_separate', 'Invalid gl.stencilMaskSeparate() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if face is not gl.FRONT, gl.BACK, or gl.FRONT_AND_BACK.');
            gl.stencilMaskSeparate(-1, 0);
            this.expectError(gl.INVALID_ENUM);

        }));

        // Blend functions

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('blend_equation', 'Invalid gl.blendEquation() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not gl.FUNC_ADD, gl.FUNC_SUBTRACT, gl.FUNC_REVERSE_SUBTRACT, gl.MAX or gl.MIN.');
            gl.blendEquation(-1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('blend_equation_separate', 'Invalid gl.blendEquationSeparate() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if modeRGB is not gl.FUNC_ADD, gl.FUNC_SUBTRACT, gl.FUNC_REVERSE_SUBTRACT, gl.MAX or gl.MIN.');
            gl.blendEquationSeparate(-1, gl.FUNC_ADD);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if modeAlpha is not gl.FUNC_ADD, gl.FUNC_SUBTRACT, gl.FUNC_REVERSE_SUBTRACT, gl.MAX or gl.MIN.');
            gl.blendEquationSeparate(gl.FUNC_ADD, -1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('blend_func', 'Invalid gl.blendFunc() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if either sfactor or dfactor is not an accepted value.');
            gl.blendFunc(-1, gl.ONE);
            this.expectError(gl.INVALID_ENUM);
            gl.blendFunc(gl.ONE, -1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('blend_func_separate', 'Invalid gl.blendFuncSeparate() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if srcRGB, dstRGB, srcAlpha, or dstAlpha is not an accepted value.');
            gl.blendFuncSeparate(-1, gl.ONE, gl.SRC_COLOR, gl.ONE_MINUS_SRC_COLOR);
            this.expectError(gl.INVALID_ENUM);
            gl.blendFuncSeparate(gl.ZERO, -1, gl.SRC_COLOR, gl.ONE_MINUS_SRC_COLOR);
            this.expectError(gl.INVALID_ENUM);
            gl.blendFuncSeparate(gl.ZERO, gl.ONE, -1, gl.ONE_MINUS_SRC_COLOR);
            this.expectError(gl.INVALID_ENUM);
            gl.blendFuncSeparate(gl.ZERO, gl.ONE, gl.SRC_COLOR, -1);
            this.expectError(gl.INVALID_ENUM);

        }));

        // Rasterization API functions

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('cull_face', 'Invalid gl.cullFace() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.cullFace(-1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('front_face', 'Invalid gl.frontFace() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.frontFace(-1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('line_width', 'Invalid gl.lineWidth() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if width is less than or equal to 0.');
            gl.lineWidth(0);
            this.expectError(gl.INVALID_VALUE);
            gl.lineWidth(-1);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('begin_query', 'Invalid gl.beginQuery() usage', gl, function() {
            /** @type{Array<WebGLQuery>} */ var ids = [];
            ids[0] = gl.createQuery();
            ids[1] = gl.createQuery();
            ids[2] = gl.createQuery();

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted tokens.');
            gl.beginQuery(-1, ids[0]);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.beginQuery is executed while a query object of the same target is already active.');
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, ids[0]);
            this.expectError(gl.NO_ERROR);
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, ids[1]);
            this.expectError(gl.INVALID_OPERATION);
            // \note gl.ANY_SAMPLES_PASSED and gl.ANY_SAMPLES_PASSED_CONSERVATIVE alias to the same target for the purposes of this error.
            gl.beginQuery(gl.ANY_SAMPLES_PASSED_CONSERVATIVE, ids[1]);
            this.expectError(gl.INVALID_OPERATION);
            gl.beginQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, ids[1]);
            this.expectError(gl.NO_ERROR);
            gl.beginQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, ids[2]);
            this.expectError(gl.INVALID_OPERATION);
            gl.endQuery(gl.ANY_SAMPLES_PASSED);
            gl.endQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if id is 0.');
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, null);
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if id not a name returned from a previous call to glGenQueries, or if such a name has since been deleted with gl.deleteQuery.');
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.deleteQuery(ids[2]);
            this.expectError(gl.NO_ERROR);
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, ids[2]);
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if id is the name of an already active query object.');
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, ids[0]);
            this.expectError(gl.NO_ERROR);
            gl.beginQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, ids[0]);
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if id refers to an existing query object whose type does not does not match target.');
            gl.endQuery(gl.ANY_SAMPLES_PASSED);
            this.expectError(gl.NO_ERROR);
            gl.beginQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, ids[0]);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteQuery(ids[0]);
            gl.deleteQuery(ids[1]);
            gl.deleteQuery(ids[2]);
            this.expectError(gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('end_query', 'Invalid gl.endQuery() usage', gl, function() {
            /** @type{WebGLQuery} */ var id;
            id = gl.createQuery();

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted tokens.');
            gl.endQuery(-1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.endQuery is executed when a query object of the same target is not active.');
            gl.endQuery(gl.ANY_SAMPLES_PASSED);
            this.expectError(gl.INVALID_OPERATION);
            gl.beginQuery(gl.ANY_SAMPLES_PASSED, id);
            this.expectError(gl.NO_ERROR);
            gl.endQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN);
            this.expectError(gl.INVALID_OPERATION);
            gl.endQuery(gl.ANY_SAMPLES_PASSED);
            this.expectError(gl.NO_ERROR);

            gl.deleteQuery(id);
            this.expectError(gl.NO_ERROR);
        }));

        // Sync objects

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('fence_sync', 'Invalid gl.fenceSync() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if condition is not gl.SYNC_GPU_COMMANDS_COMPLETE.');
            gl.fenceSync(-1, 0);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if flags is not zero.');
            gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0x0010);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('wait_sync', 'Invalid gl.waitSync() usage', gl, function() {
            /** @type{WebGLSync} */ var sync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if sync is not the name of a sync object.');
            gl.waitSync(null, 0, gl.TIMEOUT_IGNORED);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if flags is not zero.');
            gl.waitSync(sync, 0x0010, gl.TIMEOUT_IGNORED);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if timeout is not gl.TIMEOUT_IGNORED.');
            gl.waitSync(sync, 0, 0);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteSync(sync);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('client_wait_sync', 'Invalid gl.clientWaitSync() usage', gl, function() {
            /** @type{WebGLSync} */ var sync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if sync is not the name of an existing sync object.');
            gl.clientWaitSync (null, 0, 10000);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if flags contains any unsupported flag.');
            gl.clientWaitSync(sync, 0x00000004, 10000);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteSync(sync);
        }));

    };

    /**
     * @param {WebGL2RenderingContext} gl
     */
    es3fNegativeFragmentApiTests.run = function(gl) {
        var testName = 'negativeFragmentApi';
        var testDescription = 'Negative Fragment API tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fNegativeFragmentApiTests.init(gl);
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };

});

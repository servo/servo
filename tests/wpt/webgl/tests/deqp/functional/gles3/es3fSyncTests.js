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
goog.provide('functional.gles3.es3fSyncTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {
    var es3fSyncTests = functional.gles3.es3fSyncTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var deRandom = framework.delibs.debase.deRandom;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var deString = framework.delibs.debase.deString;

    /** @const {number} */ es3fSyncTests.NUM_CASE_ITERATIONS = 5;
    /** @const {number} */ es3fSyncTests.MAX_VERIFY_WAIT = 5;

    /**
     * @enum
     */
    es3fSyncTests.WaitCommand = {
        WAIT_SYNC: 1,
        CLIENT_WAIT_SYNC: 2
    };

    /** @enum
     */
    es3fSyncTests.CaseOptions = {
        FLUSH_BEFORE_WAIT: 1,
        FINISH_BEFORE_WAIT: 2
    };

    /** @enum
     */
    es3fSyncTests.State = {
        DRAW: 0,
        VERIFY: 1,
        FINISH: 2
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     * @param {number} numPrimitives
     * @param {number} waitCommand
     * @param {number} waitFlags
     * @param {number} timeout
     * @param {number} options
     */
    es3fSyncTests.FenceSyncCase = function(name, description, numPrimitives, waitCommand, waitFlags, timeout, options) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {number} */ this.m_numPrimitives = numPrimitives;
        /** @type {number} */ this.m_waitCommand = waitCommand;
        /** @type {number} */ this.m_waitFlags = waitFlags;
        /** @type {number} */ this.m_timeout = timeout;
        /** @type {number} */ this.m_caseOptions = options;

        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {WebGLSync} */ this.m_syncObject = null;
        /** @type {number} */ this.m_iterNdx = 0;
        /** @type {deRandom.Random} */ this.m_rnd = new deRandom.Random(deString.deStringHash(this.name));
        /** @type {es3fSyncTests.State} */ this.m_state = es3fSyncTests.State.DRAW;
    };

    es3fSyncTests.FenceSyncCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fSyncTests.FenceSyncCase.prototype.constructor = es3fSyncTests.FenceSyncCase;

    /**
     * @param {number} numPrimitives
     * @param {deRandom.Random} rnd
     * @return {Array<number>}
     */
    es3fSyncTests.generateVertices = function(numPrimitives, rnd) {
        /** @type {Array<number>} */ var dst = [];
        /** @type {number} */ var numVertices = 3 * numPrimitives;

        for (var i = 0; i < numVertices; i++) {
            dst.push(rnd.getFloat(-1.0, 1.0)); // x
            dst.push(rnd.getFloat(-1.0, 1.0)); // y
            dst.push(rnd.getFloat(0.0, 1.0)); // z
            dst.push(1.0); // w
        }
        return dst;
    };

    es3fSyncTests.FenceSyncCase.prototype.init = function() {
        /** @type {string} */ var vertShaderSource = '#version 300 es\n' +
                'layout(location = 0) in mediump vec4 a_position;\n' +
                '\n' +
                'void main (void)\n' +
                '{\n' +
                '    gl_Position = a_position;\n' +
                '}\n';

        /** @type {string} */ var fragShaderSource = '#version 300 es\n' +
                    'layout(location = 0) out mediump vec4 o_color;\n' +
                    '\n' +
                    'void main (void)\n' +
                    '{\n' +
                    '    o_color = vec4(0.25, 0.5, 0.75, 1.0);\n' +
                    '}\n';

        assertMsgOptions(!this.m_program, 'Program should be null.', false, true);
        this.m_program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertShaderSource, fragShaderSource));

        if (!this.m_program.isOk())
            throw new Error('Failed to compile shader program');
    };

    es3fSyncTests.FenceSyncCase.prototype.deinit = function() {
        if (this.m_program)
            this.m_program = null;

        if (this.m_syncObject) {
            gl.deleteSync(this.m_syncObject);
            this.m_syncObject = null;
        }
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fSyncTests.FenceSyncCase.prototype.draw = function() {
        /** @type {Array<number>} */ var vertices = [];

        /** @type {string} */ var header = 'Case iteration ' + (this.m_iterNdx + 1) + ' / ' + es3fSyncTests.NUM_CASE_ITERATIONS;
        bufferedLogToConsole(header);

        assertMsgOptions(this.m_program !== null, 'Expected program', false, true);
        gl.useProgram(this.m_program.getProgram());
        gl.enable(gl.DEPTH_TEST);
        gl.clearColor(0.3, 0.3, 0.3, 1.0);
        gl.clearDepth(1.0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        // Generate vertices

        gl.enableVertexAttribArray(0);
        vertices = es3fSyncTests.generateVertices(this.m_numPrimitives, this.m_rnd);

        /** @type {WebGLBuffer} */ var vertexGLBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, vertexGLBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STATIC_DRAW);
        gl.vertexAttribPointer(0, 4, gl.FLOAT, false, 0, 0);

        // Draw

        gl.drawArrays(gl.TRIANGLES, 0, vertices.length / 4);
        bufferedLogToConsole('Primitives drawn.');

        // Create sync object

        this.m_syncObject = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
        bufferedLogToConsole('Sync object created');

        if (this.m_caseOptions & es3fSyncTests.CaseOptions.FLUSH_BEFORE_WAIT)
            gl.flush();
        if (this.m_caseOptions & es3fSyncTests.CaseOptions.FINISH_BEFORE_WAIT)
            gl.finish();
        this.m_state = es3fSyncTests.State.VERIFY;
    };


    es3fSyncTests.FenceSyncCase.prototype.verify = function() {
        /** @type {number} */ var waitValue = 0;
        /** @type {boolean} */ var testOk = true;

        // Wait for sync object
        if (this.m_waitCommand & es3fSyncTests.WaitCommand.WAIT_SYNC) {
            assertMsgOptions(this.m_timeout === gl.TIMEOUT_IGNORED, 'Expected TIMEOUT_IGNORED', false, true);
            assertMsgOptions(this.m_waitFlags === 0, 'Expected waitFlags = 0', false, true);
            gl.waitSync(this.m_syncObject, this.m_waitFlags, this.m_timeout);
            bufferedLogToConsole('Wait command glWaitSync called with GL_TIMEOUT_IGNORED.');
        }

        if (this.m_waitCommand & es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC) {
            waitValue = gl.clientWaitSync(this.m_syncObject, this.m_waitFlags, this.m_timeout);
            bufferedLogToConsole('glClientWaitSync return value:');
            switch (waitValue) {
                case gl.ALREADY_SIGNALED:
                    bufferedLogToConsole('gl.ALREADY_SIGNALED');
                    break;
                case gl.TIMEOUT_EXPIRED:
                    bufferedLogToConsole('gl.TIMEOUT_EXPIRED');
                    break;
                case gl.CONDITION_SATISFIED:
                    bufferedLogToConsole('gl.CONDITION_SATISFIED');
                    break;
                case gl.WAIT_FAILED:
                    bufferedLogToConsole('gl.WAIT_FAILED');
                    testOk = false;
                    break;
                default:
                    bufferedLogToConsole('Illegal return value!');
            }
        }

        gl.finish();

        // Delete sync object

        if (this.m_syncObject && testOk) {
            gl.deleteSync(this.m_syncObject);
            this.m_syncObject = null;
            bufferedLogToConsole('Sync object deleted.');
        }

        // Evaluate test result

        bufferedLogToConsole('Test result: ' + (testOk ? 'Passed!' : 'Failed!'));

        if (!testOk) {
            if (!this.m_verifyStart)
                this.m_verifyStart = new Date();
            else {
                var current = new Date();
                var elapsedTime = 0.001 * (current.getTime() - this.m_verifyStart.getTime());
                if (elapsedTime > es3fSyncTests.MAX_VERIFY_WAIT) {
                    testFailedOptions('Fail', false);
                    this.m_state = es3fSyncTests.State.FINISH;
                    if (this.m_syncObject) {
                        gl.deleteSync(this.m_syncObject);
                        this.m_syncObject = null;
                        bufferedLogToConsole('Sync object deleted.');
                    }
                }
            }
        } else {
            bufferedLogToConsole('Sync objects created and deleted successfully.');
            testPassedOptions('Pass', true);
            this.m_state = (++this.m_iterNdx < es3fSyncTests.NUM_CASE_ITERATIONS) ? es3fSyncTests.State.DRAW : es3fSyncTests.State.FINISH;
        }
    };

    es3fSyncTests.FenceSyncCase.prototype.iterate = function() {
        switch (this.m_state) {
            case es3fSyncTests.State.DRAW:
                this.draw();
                break;
             case es3fSyncTests.State.VERIFY:
                this.verify();
                break;
             case es3fSyncTests.State.FINISH:
                return tcuTestCase.IterateResult.STOP;
             default:
                throw new Error('Invalid state: ' + this.m_state);
        }

        return tcuTestCase.IterateResult.CONTINUE;
    };

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fSyncTests.SyncTests = function() {
        tcuTestCase.DeqpTest.call(this, 'fence_sync', 'Fence Sync Tests');
    };

    es3fSyncTests.SyncTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fSyncTests.SyncTests.prototype.constructor = es3fSyncTests.SyncTests;

    es3fSyncTests.SyncTests.prototype.init = function() {
        // Fence sync tests.

        this.addChild(new es3fSyncTests.FenceSyncCase('wait_sync_smalldraw', '', 10, es3fSyncTests.WaitCommand.WAIT_SYNC, 0, gl.TIMEOUT_IGNORED, 0));
        this.addChild(new es3fSyncTests.FenceSyncCase('wait_sync_largedraw', '', 100000, es3fSyncTests.WaitCommand.WAIT_SYNC, 0, gl.TIMEOUT_IGNORED, 0));

        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_smalldraw', '', 10, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, 0, 0));
        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_largedraw', '', 100000, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, 0, 0));

        // Originally the next two test cases' timeout is 10, but in WebGL2 that could be illegal.
        var max = gl.getParameter(gl.MAX_CLIENT_WAIT_TIMEOUT_WEBGL) || 0;
        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_timeout_smalldraw', '', 10, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, max, 0));
        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_timeout_largedraw', '', 100000, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, max, 0));

        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_flush_auto', '', 100000, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, gl.SYNC_FLUSH_COMMANDS_BIT, 0, 0));
        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_flush_manual', '', 100000, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, 0, es3fSyncTests.CaseOptions.FLUSH_BEFORE_WAIT));
        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_noflush', '', 100000, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, 0, 0));
        this.addChild(new es3fSyncTests.FenceSyncCase('client_wait_sync_finish', '', 100000, es3fSyncTests.WaitCommand.CLIENT_WAIT_SYNC, 0, 0, es3fSyncTests.CaseOptions.FINISH_BEFORE_WAIT));

    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fSyncTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fSyncTests.SyncTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fSyncTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

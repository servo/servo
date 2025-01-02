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
goog.provide('functional.gles3.es3fApiCase');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluStrUtil');

goog.scope(function() {

    var es3fApiCase = functional.gles3.es3fApiCase;
    var gluStrUtil = framework.opengl.gluStrUtil;
    var tcuTestCase = framework.common.tcuTestCase;

    // format numbers as they appear in gl.h
    var getHexStr = function(num) {
        var numstr = num.toString(16);
        var prefix = '0x';
        for (
            var padding = (num < 0x10000 ? 4 : 8) - numstr.length;
            padding-- > 0;
        ) prefix += '0';
        return prefix + numstr;
    };

    /**
    * Base class for all the API tests.
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    * @param {string} name
    * @param {string} desc
    */
    es3fApiCase.ApiCase = function(name, desc, gl) {
        gl = gl || window.gl;
        if (this.test === undefined) {
            throw new Error('Unimplemented virtual function: es3fApiCase.ApiCase.test');
        }
        tcuTestCase.DeqpTest.call(this, name, desc);

        this.m_gl = gl;
        this.m_pass = true;
        this.m_comment = '';

    };

    es3fApiCase.ApiCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fApiCase.ApiCase.prototype.constructor = es3fApiCase.ApiCase;

    /**
     * @param {boolean} condition
     * @param {string=} message
     */
    es3fApiCase.ApiCase.prototype.check = function(condition, message) {
        if (this.m_pass && !condition) {
            bufferedLogToConsole('Condition is false. Test failed.');
            if (message)
                this.m_comment += ' ' + message;
            this.m_pass = condition;
        }
        return condition;
    };

    es3fApiCase.ApiCase.prototype.iterate = function() {

        this.test();

        if (this.m_pass)
            testPassed(this.m_comment);
        else
            testFailedOptions(this.m_comment, true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
    * @param {Array<number>|number} expected
    * @return {boolean} returns true if gl.getError returns an expected error code and false otherwise.
    */
    es3fApiCase.ApiCase.prototype.expectError = function(expected) {
        if (expected.constructor === Number)
            expected = [expected];

        var err = this.m_gl.getError();
        var conformant = expected.indexOf(err) >= 0;

        if (!conformant) {

            var l = expected.length;
            var msg = 'Expected ';

            if (l > 1)
                msg += (l == 2 ? 'either ' : 'one of ');

            for (var i = 0; i < l; ++i) msg += (
                (gluStrUtil.getErrorName(expected[i]) || getHexStr(expected[i])) +
                (l - i == 2 ? ' or ' : ', ')
            );

            msg += 'but got ' + (gluStrUtil.getErrorName(err) || getHexStr(err)) + '.';

            this.testFailed(msg);

        }

        return conformant;
    };

    es3fApiCase.ApiCase.prototype.testFailed = function(comment) {
        bufferedLogToConsole(comment);
        if (this.m_pass) {
            this.m_comment = comment;
            this.m_pass = false;
        }
    };

    es3fApiCase.ApiCase.prototype.expectThrowNoError = function(f) {
        try {
            f();
            this.testFailed("should have thrown exception");
        } catch (e) {
            this.expectError(this.m_gl.NO_ERROR);
        }
    }

    /**
    * Base class for all the API tests.
    * @constructor
    * @extends {es3fApiCase.ApiCase}
    * @param {string} name
    * @param {string} desc
    * @param {function(this:es3fApiCase.ApiCaseCallback)} callback
    */
    es3fApiCase.ApiCaseCallback = function(name, desc, gl, callback) {
        this.test = callback;
        es3fApiCase.ApiCase.call(this, name, desc, gl);
    };
    es3fApiCase.ApiCaseCallback.prototype = Object.create(es3fApiCase.ApiCase.prototype);
    es3fApiCase.ApiCaseCallback.prototype.constructor = es3fApiCase.ApiCaseCallback;

/*
    es3fApiCase.ApiCase.prototype.expectError // (error) or (error0, error1)
    es3fApiCase.ApiCase.prototype.getSupportedExtensions // (number numSupportedValues, number extension, [number] values )
    es3fApiCase.ApiCase.prototype.checkBooleans // (char value, char expected);
//*/
});

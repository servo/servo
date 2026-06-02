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
goog.provide('functional.gles3.es3fBufferCopyTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('modules.shared.glsBufferTestUtil');

goog.scope(function() {

    var es3fBufferCopyTests = functional.gles3.es3fBufferCopyTests;
    var glsBufferTestUtil = modules.shared.glsBufferTestUtil;
    var tcuTestCase = framework.common.tcuTestCase;
    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var deRandom = framework.delibs.debase.deRandom;

    /** @type {WebGL2RenderingContext} */ var gl;

    /**
     * @constructor
     * @extends {glsBufferTestUtil.BufferCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} srcTarget
     * @param {number} srcSize
     * @param {number} srcHint
     * @param {number} dstTarget
     * @param {number} dstSize
     * @param {number} dstHint
     * @param {number} copySrcOffset
     * @param {number} copyDstOffset
     * @param {number} copySize
     * @param {glsBufferTestUtil.VerifyType} verifyType
     */
    es3fBufferCopyTests.BasicBufferCopyCase = function(name, desc, srcTarget, srcSize, srcHint, dstTarget, dstSize, dstHint, copySrcOffset, copyDstOffset, copySize, verifyType) {
        glsBufferTestUtil.BufferCase.call(this, name, desc);

        this.m_srcTarget = srcTarget;
        this.m_srcSize = srcSize;
        this.m_srcHint = srcHint;
        this.m_dstTarget = dstTarget;
        this.m_dstSize = dstSize;
        this.m_dstHint = dstHint;
        this.m_copySrcOffset = copySrcOffset;
        this.m_copyDstOffset = copyDstOffset;
        this.m_copySize = copySize;
        this.m_verifyType = verifyType;

        assertMsgOptions(deMath.deInBounds32(this.m_copySrcOffset, 0, this.m_srcSize) && deMath.deInRange32(this.m_copySrcOffset + this.m_copySize, this.m_copySrcOffset, this.m_srcSize), 'Copy parameters are out of buffer\'s range', false, true);
        assertMsgOptions(deMath.deInBounds32(this.m_copyDstOffset, 0, this.m_dstSize) && deMath.deInRange32(this.m_copyDstOffset + this.m_copySize, this.m_copyDstOffset, this.m_dstSize), 'Copy parameters are out of buffer\'s range', false, true);
    };

    es3fBufferCopyTests.BasicBufferCopyCase.prototype = Object.create(glsBufferTestUtil.BufferCase.prototype);
    es3fBufferCopyTests.BasicBufferCopyCase.prototype.constructor = es3fBufferCopyTests.BasicBufferCopyCase;

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fBufferCopyTests.BasicBufferCopyCase.prototype.iterate = function() {
        /** @type {glsBufferTestUtil.BufferVerifier} */ var verifier = new glsBufferTestUtil.BufferVerifier(this.m_verifyType);
        var srcRef = new glsBufferTestUtil.ReferenceBuffer();
        var dstRef = new glsBufferTestUtil.ReferenceBuffer();
        var srcBuf = 0;
        var dstBuf = 0;
        var srcSeed = deMath.binaryOp(deString.deStringHash(this.fullName()), 0xabcd, deMath.BinaryOp.XOR);
        var dstSeed = deMath.binaryOp(deString.deStringHash(this.fullName()), 0xef01, deMath.BinaryOp.XOR);
        var isOk = true;

        srcRef.setSize(this.m_srcSize);
        glsBufferTestUtil.fillWithRandomBytes(srcRef.getPtr(), this.m_srcSize, srcSeed);

        dstRef.setSize(this.m_dstSize);
        glsBufferTestUtil.fillWithRandomBytes(dstRef.getPtr(), this.m_dstSize, dstSeed);

        // Create source buffer and fill with data.
        srcBuf = this.genBuffer();
        gl.bindBuffer(this.m_srcTarget, srcBuf);
        gl.bufferData(this.m_srcTarget, srcRef.getPtr(), this.m_srcHint);

        // Create destination buffer and fill with data.
        dstBuf = this.genBuffer();
        gl.bindBuffer(this.m_dstTarget, dstBuf);
        gl.bufferData(this.m_dstTarget, dstRef.getPtr(), this.m_dstHint);

        // Verify both buffers before executing copy.
        isOk = verifier.verify(srcBuf, srcRef.getPtr(), 0, this.m_srcSize, this.m_srcTarget) && isOk;
        isOk = verifier.verify(dstBuf, dstRef.getPtr(), 0, this.m_dstSize, this.m_dstTarget) && isOk;

        // Execute copy.
        dstRef.getPtr().set(srcRef.getPtr().subarray(this.m_copySrcOffset, this.m_copySrcOffset + this.m_copySize), this.m_copyDstOffset);

        gl.bindBuffer(this.m_srcTarget, srcBuf);
        gl.bindBuffer(this.m_dstTarget, dstBuf);
        gl.copyBufferSubData(this.m_srcTarget, this.m_dstTarget, this.m_copySrcOffset, this.m_copyDstOffset, this.m_copySize);

        // Verify both buffers after copy.
        isOk = verifier.verify(srcBuf, srcRef.getPtr(), 0, this.m_srcSize, this.m_srcTarget) && isOk;
        isOk = verifier.verify(dstBuf, dstRef.getPtr(), 0, this.m_dstSize, this.m_dstTarget) && isOk;

        if (isOk)
            testPassed('');
        else
            testFailed('Buffer verification failed');

        return tcuTestCase.IterateResult.STOP;
    };

    // Case B: same buffer, take range as parameter

    /**
     * @constructor
     * @extends {glsBufferTestUtil.BufferCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} srcTarget
     * @param {number} dstTarget
     * @param {number} hint
     * @param {glsBufferTestUtil.VerifyType} verifyType
     */
    es3fBufferCopyTests.SingleBufferCopyCase = function(name, desc, srcTarget, dstTarget, hint, verifyType) {
        glsBufferTestUtil.BufferCase.call(this, name, desc);
        this.m_srcTarget = srcTarget;
        this.m_dstTarget = dstTarget;
        this.m_hint = hint;
        this.m_verifyType = verifyType;
    };

    es3fBufferCopyTests.SingleBufferCopyCase.prototype = Object.create(glsBufferTestUtil.BufferCase.prototype);
    es3fBufferCopyTests.SingleBufferCopyCase.prototype.constructor = es3fBufferCopyTests.SingleBufferCopyCase;

    /**
     * @return {tcuTestCase.IterateResult}
     */
     es3fBufferCopyTests.SingleBufferCopyCase.prototype.iterate = function() {
        var size = 1000;
        /** @type {glsBufferTestUtil.BufferVerifier} */ var verifier = new glsBufferTestUtil.BufferVerifier(this.m_verifyType);
        var ref = new glsBufferTestUtil.ReferenceBuffer();
        var baseSeed = deString.deStringHash(this.fullName());
        var isOk = true;

        ref.setSize(size);

        // Create buffer.
        var buf = this.genBuffer();
        gl.bindBuffer(this.m_srcTarget, buf);

        /** @type {Array<{srcOffset: number, dstOffset: number, copySize: number}>} */
        var copyRanges = [{
                srcOffset: 57, dstOffset: 701, copySize: 101 // Non-adjecent, from low to high.
            },{
                srcOffset: 640, dstOffset: 101, copySize: 101 // Non-adjecent, from high to low.
            },{
                srcOffset: 0, dstOffset: 500, copySize: 500 // Lower half to upper half.
            },{
                srcOffset: 500, dstOffset: 0, copySize: 500 // Upper half to lower half.
        }];

        for (var ndx = 0; ndx < copyRanges.length && isOk; ndx++) {
            var srcOffset = copyRanges[ndx].srcOffset;
            var dstOffset = copyRanges[ndx].dstOffset;
            var copySize = copyRanges[ndx].copySize;

            glsBufferTestUtil.fillWithRandomBytes(ref.getPtr(), size, deMath.binaryOp(baseSeed, deMath.deMathHash(ndx), deMath.BinaryOp.XOR));

            // Fill with data.
            gl.bindBuffer(this.m_srcTarget, buf);
            gl.bufferData(this.m_srcTarget, ref.getPtr(), this.m_hint);

            // Execute copy.
            ref.getPtr().set(ref.getPtr().subarray(srcOffset, srcOffset + copySize), dstOffset);

            gl.bindBuffer(this.m_dstTarget, buf);
            gl.copyBufferSubData(this.m_srcTarget, this.m_dstTarget, srcOffset, dstOffset, copySize);

            // Verify buffer after copy.
            isOk = verifier.verify(buf, ref.getPtr(), 0, size, this.m_dstTarget) && isOk;
        }

        if (isOk)
            testPassed('');
        else
            testFailed('Buffer verification failed');

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fBufferCopyTests.BufferCopyTests = function() {
        tcuTestCase.DeqpTest.call(this, 'copy', 'Buffer copy tests');
        this.makeExecutable();
    };

    es3fBufferCopyTests.BufferCopyTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fBufferCopyTests.BufferCopyTests.prototype.constructor = es3fBufferCopyTests.BufferCopyTests;

    es3fBufferCopyTests.BufferCopyTests.prototype.init = function() {
        /** @type {glsBufferTestUtil.VerifyType} */ var verify;

        var bufferTargets = [
            gl.ARRAY_BUFFER,
            gl.COPY_READ_BUFFER,
            gl.COPY_WRITE_BUFFER,
            gl.ELEMENT_ARRAY_BUFFER,
            gl.PIXEL_PACK_BUFFER,
            gl.PIXEL_UNPACK_BUFFER,
            gl.TRANSFORM_FEEDBACK_BUFFER,
            gl.UNIFORM_BUFFER
        ];

        // .basic

        var basicGroup = new tcuTestCase.DeqpTest('basic', 'Basic buffer copy cases');
        this.addChild(basicGroup);

        for (var srcTargetNdx = 0; srcTargetNdx < bufferTargets.length; srcTargetNdx++) {
            for (var dstTargetNdx = 0; dstTargetNdx < bufferTargets.length; dstTargetNdx++) {
                if (srcTargetNdx == dstTargetNdx)
                    continue;

                // In WebGL 2, a copy between an ELEMENT_ARRAY_BUFFER and other data buffer
                // (not COPY_WRITE_BUFFER nor COPY_READ_BUFFER nor ELEMENT_ARRAY_BUFFER)
                // cannot be made, so let's skip those cases.
                if (bufferTargets[srcTargetNdx] == gl.ELEMENT_ARRAY_BUFFER ||
                    bufferTargets[dstTargetNdx] == gl.ELEMENT_ARRAY_BUFFER)
                    continue;

                var srcTarget = bufferTargets[srcTargetNdx];
                var dstTarget = bufferTargets[dstTargetNdx];
                var size = 1017;
                var hint = gl.STATIC_DRAW;
                verify = glsBufferTestUtil.VerifyType.AS_VERTEX_ARRAY;
                var name = glsBufferTestUtil.getBufferTargetName(srcTarget) + '_' + glsBufferTestUtil.getBufferTargetName(dstTarget);

                basicGroup.addChild(new es3fBufferCopyTests.BasicBufferCopyCase(name, '', srcTarget, size, hint, dstTarget, size, hint, 0, 0, size, verify));
            }
        }

        // .subrange

        var subrangeGroup = new tcuTestCase.DeqpTest('subrange', 'Buffer subrange copy tests');
        this.addChild(subrangeGroup);

        /**
         * @type {Array<{name: string, srcSize: number, dstSize: number, srcOffset: number, dstOffset: number, copySize: number}>}
         */
        var cases = [{
                name: 'middle', srcSize: 1000, dstSize: 1000, srcOffset: 250, dstOffset: 250, copySize: 500
            },{
                name: 'small_to_large', srcSize: 100, dstSize: 1000, srcOffset: 0, dstOffset: 409, copySize: 100
            },{
                name: 'large_to_small', srcSize: 1000, dstSize: 100, srcOffset: 409, dstOffset: 0, copySize: 100
            },{
                name: 'low_to_high_1', srcSize: 1000, dstSize: 1000, srcOffset: 0, dstOffset: 500, copySize: 500
            },{
                name: 'low_to_high_2', srcSize: 997, dstSize: 1027, srcOffset: 0, dstOffset: 701, copySize: 111
            },{
                name: 'high_to_low_1', srcSize: 1000, dstSize: 1000, srcOffset: 500, dstOffset: 0, copySize: 500
            },{
                name: 'high_to_low_2', srcSize: 1027, dstSize: 997, srcOffset: 701, dstOffset: 17, copySize: 111
        }];

        for (var ndx = 0; ndx < cases.length; ndx++) {
            var srcTarget = gl.COPY_READ_BUFFER;
            var dstTarget = gl.COPY_WRITE_BUFFER;
            var hint = gl.STATIC_DRAW;
            verify = glsBufferTestUtil.VerifyType.AS_VERTEX_ARRAY;

            subrangeGroup.addChild(
                new es3fBufferCopyTests.BasicBufferCopyCase(
                    cases[ndx].name, '',
                    srcTarget, cases[ndx].srcSize, hint,
                    dstTarget, cases[ndx].dstSize, hint,
                    cases[ndx].srcOffset, cases[ndx].dstOffset, cases[ndx].copySize,
                    verify
                )
            );
        }

        // .single_buffer

        var singleBufGroup = new tcuTestCase.DeqpTest('single_buffer', 'Copies within single buffer');
        this.addChild(singleBufGroup);

        for (var srcTargetNdx = 0; srcTargetNdx < bufferTargets.length; srcTargetNdx++) {
            for (var dstTargetNdx = 0; dstTargetNdx < bufferTargets.length; dstTargetNdx++) {
                if (srcTargetNdx == dstTargetNdx)
                    continue;

                // In WebGL 2, we can't rebind an ELEMENT_ARRAY_BUFFER or TRANSFORM_FEEDBACK_BUFFER as a
                // different type of buffer, so we skip those cases.
                if (bufferTargets[srcTargetNdx] == gl.ELEMENT_ARRAY_BUFFER || bufferTargets[srcTargetNdx] == gl.TRANSFORM_FEEDBACK_BUFFER ||
                    bufferTargets[dstTargetNdx] == gl.ELEMENT_ARRAY_BUFFER || bufferTargets[dstTargetNdx] == gl.TRANSFORM_FEEDBACK_BUFFER)
                    continue;

                var srcTarget = bufferTargets[srcTargetNdx];
                var dstTarget = bufferTargets[dstTargetNdx];
                var hint = gl.STATIC_DRAW;
                verify = glsBufferTestUtil.VerifyType.AS_VERTEX_ARRAY;
                var name = glsBufferTestUtil.getBufferTargetName(srcTarget) + '_' + glsBufferTestUtil.getBufferTargetName(dstTarget);

                singleBufGroup.addChild(new es3fBufferCopyTests.SingleBufferCopyCase(name, '', srcTarget, dstTarget, hint, verify));
            }
        }
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     */
     es3fBufferCopyTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;

        state.setRoot(new es3fBufferCopyTests.BufferCopyTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

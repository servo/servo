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
goog.provide('functional.gles3.es3fShaderPrecisionTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuFloat');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {
    var es3fShaderPrecisionTests = functional.gles3.es3fShaderPrecisionTests;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;
    var deString = framework.delibs.debase.deString;
    var tcuFloat = framework.common.tcuFloat;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluShaderProgram = framework.opengl.gluShaderProgram;

    /** @const {number} */ es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH = 32;
    /** @const {number} */ es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT = 32;

    es3fShaderPrecisionTests.add = function(a, b) { return a + b; };
    es3fShaderPrecisionTests.sub = function(a, b) { return a - b; };
    es3fShaderPrecisionTests.mul = function(a, b) { return a * b; };
    // a * b = (a1 * 2^16 + a0) * (b1 * 2^16 + b0) = a1 * b1 * 2^32 + (a0 * b1 + a1 * b0) * 2^16 + a0 * b0
    // 32bit integer multiplication may overflow in JavaScript. Only return low 32bit of the result.
    es3fShaderPrecisionTests.mul32 = function(a, b) {
        var sign = Math.sign(a) * Math.sign(b);
        a = Math.abs(a);
        b = Math.abs(b);
        var a1 = deMath.split16(a)[1];
        var a0 = deMath.split16(a)[0];
        var b1 = deMath.split16(b)[1];
        var b0 = deMath.split16(b)[0];
        return sign * ((a0 * b1 + a1 * b0) * 0x10000 + a0 * b0);
    }
    es3fShaderPrecisionTests.div = function(a, b) { if (b !== 0) return a / b; else throw new Error('division by zero.')};

    /**
     * @param {gluShaderUtil.precision} precision
     * @param {string} evalOp
     * @param {boolean} isVertexCase
     * @return {gluShaderProgram.ShaderProgram}
     */
    es3fShaderPrecisionTests.createFloatPrecisionEvalProgram = function(precision, evalOp, isVertexCase) {
        /** @type {gluShaderUtil.DataType} */ var type = gluShaderUtil.DataType.FLOAT;
        /** @type {gluShaderUtil.DataType} */ var outType = gluShaderUtil.DataType.UINT;
        /** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(type);
        /** @type {string} */ var outTypeName = gluShaderUtil.getDataTypeName(outType);
        /** @type {string} */ var precName = gluShaderUtil.getPrecisionName(precision);
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        vtx += '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in ' + precName + ' ' + typeName + ' a_in0;\n' +
            'in ' + precName + ' ' + typeName + ' a_in1;\n';
        frag += '#version 300 es\n' +
            'layout(location = 0) out highp ' + outTypeName + ' o_out;\n';

        if (isVertexCase) {
            vtx += 'flat out ' + precName + ' ' + typeName + ' v_out;\n';
            frag += 'flat in ' + precName + ' ' + typeName + ' v_out;\n';
        } else {
            vtx += 'flat out ' + precName + ' ' + typeName + ' v_in0;\n' +
                'flat out ' + precName + ' ' + typeName + ' v_in1;\n';
            frag += 'flat in ' + precName + ' ' + typeName + ' v_in0;\n' +
                'flat in ' + precName + ' ' + typeName + ' v_in1;\n';
        }

        vtx += '\nvoid main (void)\n{\n' +
            '    gl_Position = a_position;\n';
        frag += '\nvoid main (void)\n{\n';

        op += '\t' + precName + ' ' + typeName + ' in0 = ' + (isVertexCase ? 'a_' : 'v_') + 'in0;\n' +
            '\t' + precName + ' ' + typeName + ' in1 = ' + (isVertexCase ? 'a_' : 'v_') + 'in1;\n';

        if (!isVertexCase)
            op += '\t' + precName + ' ' + typeName + ' res;\n';

        op += '\t' + (isVertexCase ? 'v_out' : 'res') + ' = ' + evalOp + ';\n';

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        if (isVertexCase) {
            frag += '    o_out = floatBitsToUint(v_out);\n';
        } else {
            vtx += '    v_in0 = a_in0;\n' +
                '    v_in1 = a_in1;\n';
            frag += '    o_out = floatBitsToUint(res);\n';
        }

        vtx += '}\n';
        frag += '}\n';

        return new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vtx, frag));
    };

    /**
     * @param {gluShaderUtil.DataType} type
     * @param {gluShaderUtil.precision} precision
     * @param {string} evalOp
     * @param {boolean} isVertexCase
     * @return {gluShaderProgram.ShaderProgram}
     */
    es3fShaderPrecisionTests.createIntUintPrecisionEvalProgram = function(type, precision, evalOp, isVertexCase) {
        /** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(type);
        /** @type {string} */ var precName = gluShaderUtil.getPrecisionName(precision);
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        vtx += '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in ' + precName + ' ' + typeName + ' a_in0;\n' +
            'in ' + precName + ' ' + typeName + ' a_in1;\n';
        frag += '#version 300 es\n' +
            'layout(location = 0) out ' + precName + ' ' + typeName + ' o_out;\n';

        if (isVertexCase) {
            vtx += 'flat out ' + precName + ' ' + typeName + ' v_out;\n';
            frag += 'flat in ' + precName + ' ' + typeName + ' v_out;\n';
        } else {
            vtx += 'flat out ' + precName + ' ' + typeName + ' v_in0;\n' +
                'flat out ' + precName + ' ' + typeName + ' v_in1;\n';
            frag += 'flat in ' + precName + ' ' + typeName + ' v_in0;\n' +
                'flat in ' + precName + ' ' + typeName + ' v_in1;\n';
        }

        vtx += '\nvoid main (void)\n{\n'+
            '    gl_Position = a_position;\n';
        frag += '\nvoid main (void)\n{\n';

        op += '\t' + precName + ' ' + typeName + ' in0 = ' + (isVertexCase ? 'a_' : 'v_') + 'in0;\n' +
            '\t' + precName + ' ' + typeName + ' in1 = ' + (isVertexCase ? 'a_' : 'v_') + 'in1;\n';

        op += '\t' + (isVertexCase ? 'v_' : 'o_') + 'out = ' + evalOp + ';\n';

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        if (isVertexCase) {
            frag += '    o_out = v_out;\n';
        } else {
            vtx += '    v_in0 = a_in0;\n' +
                '    v_in1 = a_in1;\n';
        }

        vtx += '}\n';
        frag += '}\n';

        return new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vtx, frag));
    };

    /** @typedef {function(number, number)} */ es3fShaderPrecisionTests.EvalFunc;


    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {string} op
     * @param {es3fShaderPrecisionTests.EvalFunc} evalFunc
     * @param {gluShaderUtil.precision} precision
     * @param {Array<number>} rangeA
     * @param {Array<number>} rangeB
     * @param {boolean} isVertexCase
     */
    es3fShaderPrecisionTests.ShaderFloatPrecisionCase = function(name, desc, op, evalFunc, precision, rangeA, rangeB, isVertexCase) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        // Case parameters.
        /** @type {string} */ this.m_op = op;
        /** @type {es3fShaderPrecisionTests.EvalFunc} */ this.m_evalFunc = evalFunc;
        /** @type {gluShaderUtil.precision} */ this.m_precision = precision;
        /** @type {Array<number>} */ this.m_rangeA = rangeA;
        /** @type {Array<number>} */ this.m_rangeB = rangeB;
        /** @type {boolean} */ this.m_isVertexCase = isVertexCase;

        /** @type {number} */ this.m_numTestsPerIter = 32;
        /** @type {number} */ this.m_numIters = 4;
        /** @type {deRandom.Random} */ this.m_rnd = new deRandom.Random(deString.deStringHash(this.name));

        // Iteration state.
        /** @type {?gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {?WebGLFramebuffer} */ this.m_framebuffer = null;
        /** @type {?WebGLRenderbuffer} */ this.m_renderbuffer = null;
        /** @type {number} */ this.m_iterNdx = 0;
        /** @type {Array<boolean>} */ this.m_iterPass = [];
    };

    es3fShaderPrecisionTests.ShaderFloatPrecisionCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderPrecisionTests.ShaderFloatPrecisionCase.prototype.constructor = es3fShaderPrecisionTests.ShaderFloatPrecisionCase;

    es3fShaderPrecisionTests.ShaderFloatPrecisionCase.prototype.init = function() {
        assertMsgOptions(!this.m_program && !this.m_framebuffer && !this.m_renderbuffer, 'Program/Framebuffer/Renderbuffer should be null at this point.', false, true);

        // Create program.
        this.m_program = es3fShaderPrecisionTests.createFloatPrecisionEvalProgram(this.m_precision, this.m_op, this.m_isVertexCase);

        if (!this.m_program.isOk())
            assertMsgOptions(false, 'Compile failed', false, true);

        // Create framebuffer.
        this.m_framebuffer = gl.createFramebuffer();
        this.m_renderbuffer = gl.createRenderbuffer();

        gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_renderbuffer);
        gl.renderbufferStorage(gl.RENDERBUFFER, gl.R32UI, es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH, es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT);

        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);
        gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, this.m_renderbuffer);

        assertMsgOptions(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE, 'Framebuffer is incomplete', false, true);

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        this.m_iterNdx = 0;
    };

    es3fShaderPrecisionTests.ShaderFloatPrecisionCase.prototype.deinit = function() {
        if(this.m_framebuffer)
            gl.deleteFramebuffer(this.m_framebuffer);
        if(this.m_renderbuffer)
            gl.deleteRenderbuffer(this.m_renderbuffer);
        this.m_program = null;
        this.m_framebuffer = null;
        this.m_renderbuffer = null;
    };

    /**
     * @param {number} in0
     * @param {number} in1
     * @param {number} reference
     * @param {number} result
     */

    es3fShaderPrecisionTests.ShaderFloatPrecisionCase.prototype.compare = function(in0, in1, reference, result) {
        // Comparison is done using 64-bit reference value to accurately evaluate rounding mode error.
        // If 32-bit reference value is used, 2 bits of rounding error must be allowed.

        // For mediump and lowp types the comparison currently allows 3 bits of rounding error:
        // two bits from conversions and one from actual operation.

        // \todo [2013-09-30 pyry] Make this more strict: determine if rounding can actually happen.

        /** @type {number} */ var  mantissaBits = this.m_precision == gluShaderUtil.precision.PRECISION_HIGHP ? 23 : 10;
        /** @type {number} */ var  numPrecBits = 52 - mantissaBits;

        /** @type {number} */ var  in0Exp = tcuFloat.newFloat32(in0).exponent();
        /** @type {number} */ var  in1Exp = tcuFloat.newFloat32(in1).exponent();
        /** @type {number} */ var  resExp = tcuFloat.newFloat32(result).exponent();
        /** @type {number} */ var  numLostBits = Math.max(in0Exp - resExp, in1Exp - resExp, 0); // Lost due to mantissa shift.

        /** @type {number} */ var  roundingUlpError = this.m_precision == gluShaderUtil.precision.PRECISION_HIGHP ? 1 : 3;
        /** @type {number} */ var  maskBits = numLostBits + numPrecBits;

        bufferedLogToConsole("Assuming " + mantissaBits + " mantissa bits, " + numLostBits + " bits lost in operation, and " + roundingUlpError + " ULP rounding error.")

        // These numbers should never be larger than 52 bits. An assertion in getBitRange verifies this.
        /** @type {number} */ var accurateRefBits = tcuFloat.newFloat64(reference).getBitRange(maskBits, 64);
        /** @type {number} */ var accurateResBits = tcuFloat.newFloat64(result).getBitRange(maskBits, 64);
        /** @type {number} */ var ulpDiff = Math.abs(accurateRefBits - accurateResBits);

        if (ulpDiff > roundingUlpError) {
            bufferedLogToConsole("ERROR: comparison failed! ULP diff (ignoring lost/undefined bits) = " + ulpDiff );
            return false;
        }
        else
            return true;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fShaderPrecisionTests.ShaderFloatPrecisionCase.prototype.iterate = function() {
        var testPassed = true;
        var testPassedMsg = 'Pass';

        // Constant data.
        /** @type {Array<number>} */ var position =[
        -1.0, -1.0, 0.0, 1.0,
            -1.0, 1.0, 0.0, 1.0,
            1.0, -1.0, 0.0, 1.0,
            1.0, 1.0, 0.0, 1.0
    ];

        /** @type {Array<number>} */ var indices = [0, 1, 2, 2, 1, 3];
        /** @type {number} */ var numVertices = 4;
        /** @type {Array<number>} */ var in0Arr = [0.0, 0.0, 0.0, 0.0];
        /** @type {Array<number>} */ var in1Arr = [0.0, 0.0, 0.0, 0.0];

        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays = [];

        // Image read from GL.
        /** @type {goog.TypedArray} */ var pixels_uint = new Uint32Array(es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH * es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT * 4);

        // \todo [2012-05-03 pyry] Could be cached.
        /** @type {WebGLProgram} */ var prog = this.m_program.getProgram();

        gl.useProgram(prog);
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);

        vertexArrays[0] = gluDrawUtil.newFloatVertexArrayBinding("a_position", 4, numVertices, 0, position);


        // Compute values and reference.
        for (var testNdx = 0; testNdx < this.m_numTestsPerIter; testNdx++) {
            /** @type {number} */ var in0 = this.m_rnd.getFloat(this.m_rangeA[0], this.m_rangeA[1]);
            /** @type {number} */ var in1 = this.m_rnd.getFloat(this.m_rangeB[0], this.m_rangeB[1]);

            // These random numbers are used in the reference computation. But
            // highp is only 32 bits, so these float64s must be rounded to
            // float32 first for correctness. This is needed for highp_mul_* on
            // one Linux/NVIDIA machine.
            in0 = tcuFloat.newFloat32(in0).getValue();
            in1 = tcuFloat.newFloat32(in1).getValue();

            /** @type {number} */ var refD = this.m_evalFunc(in0, in1);

            bufferedLogToConsole("iter " + this.m_iterNdx + ", test " + testNdx + ": "+
                "in0 = " + in0 + " / " + tcuFloat.newFloat32(in0).bits() +
                ", in1 = " + in1 + " / " + tcuFloat.newFloat32(in1).bits() +
                "  reference = " + refD + " / " + tcuFloat.newFloat32(refD).bits());

            in0Arr = [in0, in0, in0, in0];
            in1Arr = [in1, in1, in1, in1];
            vertexArrays[1] = gluDrawUtil.newFloatVertexArrayBinding("a_in0", 1, numVertices, 0, in0Arr);
            vertexArrays[2] = gluDrawUtil.newFloatVertexArrayBinding("a_in1", 1, numVertices, 0, in1Arr);

            gluDrawUtil.draw(gl, prog, vertexArrays, gluDrawUtil.triangles(indices));

            gl.readPixels(0, 0, es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH,
                es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT, gl.RGBA_INTEGER, gl.UNSIGNED_INT, pixels_uint);

            var pixels = new Float32Array(pixels_uint.buffer);
            bufferedLogToConsole("  result = " + pixels[0] + " / " + tcuFloat.newFloat32(pixels[0]).bits());

            // Verify results
            /** @type {boolean} */ var firstPixelOk = this.compare(in0, in1, refD, pixels[0]);

            if (firstPixelOk) {
                // Check that rest of pixels match to first one.
                /** @type {number} */ var firstPixelBits = tcuFloat.newFloat32(pixels[0]).bits();
                /** @type {boolean} */ var allPixelsOk = true;

                for (var y = 0; y < es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT; y++) {
                    for (var x = 0; x < es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH; x++) {
                        /** @type {number} */ var pixelBits = tcuFloat.newFloat32(pixels[(y * es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH + x) * 4]).bits();

                        if (pixelBits != firstPixelBits) {
                            bufferedLogToConsole("ERROR: Inconsistent results, got " + pixelBits + " at (" + x + ", " + y + ")")
                            allPixelsOk = false;
                        }
                    }

                    if (!allPixelsOk)
                        break;
                }

                if (!allPixelsOk){
                    bufferedLogToConsole("iter " + this.m_iterNdx + ", test " + testNdx + "Inconsistent values in framebuffer");
                    testPassed = false;
                    testPassedMsg = 'Inconsistent values in framebuffer';
                }
            }
            else{
                bufferedLogToConsole("iter " + this.m_iterNdx + ", test " + testNdx + "Result comparison failed");
                testPassed = false;
                testPassedMsg = 'Result comparison failed'
            }
        }

        // [dag] Aggregating test results to make the test less verbose.
        this.m_iterPass[this.m_iterNdx] = testPassed;

        // [dag] Show test results after the last iteration is done.
        if (this.m_iterPass.length === this.m_numIters) {
            if (!deMath.boolAll(this.m_iterPass))
                testFailedOptions(testPassedMsg, false);
            else
                testPassedOptions(testPassedMsg, true);
        }
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        this.m_iterNdx += 1;
        return (this.m_iterNdx < this.m_numIters) ? tcuTestCase.IterateResult.CONTINUE : tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {string} op
     * @param {es3fShaderPrecisionTests.EvalFunc} evalFunc
     * @param {gluShaderUtil.precision} precision
     * @param {number} bits
     * @param {Array<number>} rangeA
     * @param {Array<number>} rangeB
     * @param {boolean} isVertexCase
     */
    es3fShaderPrecisionTests.ShaderIntPrecisionCase = function(name, desc, op, evalFunc, precision, bits, rangeA, rangeB, isVertexCase) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        // Case parameters.
        /** @type {string} */ this.m_op = op;
        /** @type {es3fShaderPrecisionTests.EvalFunc} */ this.m_evalFunc = evalFunc;
        /** @type {gluShaderUtil.precision} */ this.m_precision = precision;
        /** @type {number} */ this.m_bits = bits;
        /** @type {Array<number>} */ this.m_rangeA = rangeA;
        /** @type {Array<number>} */ this.m_rangeB = rangeB;
        /** @type {boolean} */ this.m_isVertexCase = isVertexCase;

        /** @type {number} */ this.m_numTestsPerIter = 32;
        /** @type {number} */ this.m_numIters = 4;
        /** @type {deRandom.Random} */ this.m_rnd = new deRandom.Random(deString.deStringHash(this.name));

        // Iteration state.
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {WebGLFramebuffer} */ this.m_framebuffer = null;
        /** @type {WebGLRenderbuffer} */ this.m_renderbuffer = null;
        /** @type {number} */ this.m_iterNdx = 0;
    };

    es3fShaderPrecisionTests.ShaderIntPrecisionCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderPrecisionTests.ShaderIntPrecisionCase.prototype.constructor = es3fShaderPrecisionTests.ShaderIntPrecisionCase;

    es3fShaderPrecisionTests.ShaderIntPrecisionCase.prototype.init = function() {
        assertMsgOptions(!this.m_program && !this.m_framebuffer && !this.m_renderbuffer, 'Program/Framebuffer/Renderbuffer should be null at this point.', false, true);
        // Create program.
        this.m_program = es3fShaderPrecisionTests.createIntUintPrecisionEvalProgram(gluShaderUtil.DataType.INT, this.m_precision, this.m_op, this.m_isVertexCase);

        if (!this.m_program.isOk())
            assertMsgOptions(false, 'Compile failed', false, true);

        // Create framebuffer.
        this.m_framebuffer = gl.createFramebuffer();
        this.m_renderbuffer = gl.createRenderbuffer();

        gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_renderbuffer);
        gl.renderbufferStorage(gl.RENDERBUFFER, gl.R32I, es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH, es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT);

        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);
        gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, this.m_renderbuffer);

        assertMsgOptions(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE, 'Framebuffer is incomplete', false, true);

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        this.m_iterNdx = 0;

        bufferedLogToConsole("Number of accurate bits assumed = " + this.m_bits);
    };

    es3fShaderPrecisionTests.ShaderIntPrecisionCase.prototype.deinit = function() {
        if(this.m_framebuffer)
            gl.deleteFramebuffer(this.m_framebuffer);
        if(this.m_renderbuffer)
            gl.deleteRenderbuffer(this.m_renderbuffer);
        this.m_program = null;
        this.m_framebuffer = null;
        this.m_renderbuffer = null;
    };

    /**
     * @param {number} value
     * @param {number} bits
     * @return {number}
     */

    es3fShaderPrecisionTests.extendTo32Bit = function(value, bits) {
        return (value & ((1 << (bits - 1)) - 1)) | ((value & (1 << (bits - 1))) << (32 - bits)) >> (32 - bits);
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fShaderPrecisionTests.ShaderIntPrecisionCase.prototype.iterate = function() {
        var testPassed = true;
        var testPassedMsg = 'Pass';
        // Constant data.
        /** @type {Array<number>} */ var position = [
        -1.0, -1.0, 0.0, 1.0,
            -1.0, 1.0, 0.0, 1.0,
            1.0, -1.0, 0.0, 1.0,
            1.0, 1.0, 0.0, 1.0
    ]
        /** @type {Array<number>} */ var indices = [0, 1, 2, 2, 1, 3];

        /** @type {number} */ var numVertices    = 4;
        /** @type {Array<number>} */ var in0Arr = [0, 0, 0, 0];
        /** @type {Array<number>} */ var in1Arr = [0, 0, 0, 0];

        /** @type {number} */ var mask = this.m_bits === 32 ? 0xffffffff : ((1 << this.m_bits) - 1);
        /** @type {goog.TypedArray} */ var pixels = new Int32Array(es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH * es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT * 4);
        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays = [];

        /** @type {WebGLProgram} */ var prog = this.m_program.getProgram();

        // \todo [2012-05-03 pyry] A bit hacky. getInt() should work fine with ranges like this.
        /** @type {boolean} */ var isMaxRangeA = this.m_rangeA[0] === 0x80000000 && this.m_rangeA[1] === 0x7fffffff;
        /** @type {boolean} */ var isMaxRangeB = this.m_rangeB[0] === 0x80000000 && this.m_rangeB[1] === 0x7fffffff;

        gl.useProgram(prog);
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);

        vertexArrays[0] = gluDrawUtil.newFloatVertexArrayBinding("a_position", 4, numVertices, 0, position);

        // Compute values and reference.
        for (var testNdx = 0; testNdx < this.m_numTestsPerIter; testNdx++) {
            /** @type {number} */ var in0 = this.m_rnd.getInt(this.m_rangeA[0], this.m_rangeA[1]); //es3fShaderPrecisionTests.extendTo32Bit(((isMaxRangeA ? Math.abs(this.m_rnd.getInt()) : this.m_rnd.getInt(this.m_rangeA[0], this.m_rangeA[1])) & mask), this.m_bits);
            /** @type {number} */ var in1 = this.m_rnd.getInt(this.m_rangeB[0], this.m_rangeB[1]); //es3fShaderPrecisionTests.extendTo32Bit(((isMaxRangeB ? Math.abs(this.m_rnd.getInt()) : this.m_rnd.getInt(this.m_rangeB[0], this.m_rangeB[1])) & mask), this.m_bits);
            /** @type {number} */ var refMasked = this.m_evalFunc(in0, in1) & mask;
            /** @type {number} */ var refOut = es3fShaderPrecisionTests.extendTo32Bit(refMasked, this.m_bits);

            bufferedLogToConsole("iter " + this.m_iterNdx + ", test " + testNdx + ": " +
                "in0 = " + in0 + ", in1 = " + in1 + ", ref out = " + refOut + " / " + refMasked);

            in0Arr = [in0, in0, in0, in0];
            in1Arr = [in1, in1, in1, in1];

            vertexArrays[1] = gluDrawUtil.newInt32VertexArrayBinding("a_in0", 1, numVertices, 0, in0Arr);
            vertexArrays[2] = gluDrawUtil.newInt32VertexArrayBinding("a_in1", 1, numVertices, 0, in1Arr);

            gluDrawUtil.draw(gl, prog, vertexArrays, gluDrawUtil.triangles(indices));

            gl.readPixels(0, 0, es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH,
                es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT,
                gl.RGBA_INTEGER, gl.INT, pixels);

            // Compare pixels.
            for (var y = 0; y < es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT; y++) {
                for (var x = 0; x < es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH; x++) {
                    /** @type {number} */ var cmpOut = pixels[(y * es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH + x) * 4];
                    /** @type {number} */ var cmpMasked = cmpOut & mask;

                    if (cmpMasked != refMasked) {
                        bufferedLogToConsole("Comparison failed (at " + x + ", " + y + "): " +
                            + "got " + cmpOut + " / " + cmpOut);
                        testPassed = false;
                        testPassedMsg = 'Comparison failed';
                    }
                }
            }
        }

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        this.m_iterNdx += 1;
        if (!testPassed) {
            testFailedOptions(testPassedMsg, false);
            return tcuTestCase.IterateResult.STOP;
        } else if (testPassed && this.m_iterNdx < this.m_numIters) {
            return tcuTestCase.IterateResult.CONTINUE;
        } else {
            testPassedOptions(testPassedMsg, true);
            return tcuTestCase.IterateResult.STOP;
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {string} op
     * @param {es3fShaderPrecisionTests.EvalFunc} evalFunc
     * @param {gluShaderUtil.precision} precision
     * @param {number} bits
     * @param {Array<number>} rangeA
     * @param {Array<number>} rangeB
     * @param {boolean} isVertexCase
     */
    es3fShaderPrecisionTests.ShaderUintPrecisionCase = function(name, desc, op, evalFunc, precision, bits, rangeA, rangeB, isVertexCase) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        // Case parameters.
        /** @type {string} */ this.m_op = op;
        /** @type {es3fShaderPrecisionTests.EvalFunc} */ this.m_evalFunc = evalFunc;
        /** @type {gluShaderUtil.precision} */ this.m_precision = precision;
        /** @type {number} */ this.m_bits = bits;
        /** @type {Array<number>} */ this.m_rangeA = rangeA;
        /** @type {Array<number>} */ this.m_rangeB = rangeB;
        /** @type {boolean} */ this.m_isVertexCase = isVertexCase;

        /** @type {number} */ this.m_numTestsPerIter = 32;
        /** @type {number} */ this.m_numIters = 4;
        /** @type {deRandom.Random} */ this.m_rnd = new deRandom.Random(deString.deStringHash(this.name));

        // Iteration state.
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {WebGLFramebuffer} */ this.m_framebuffer = null;
        /** @type {WebGLRenderbuffer} */ this.m_renderbuffer = null;
        /** @type {number} */ this.m_iterNdx = 0;
    };

    es3fShaderPrecisionTests.ShaderUintPrecisionCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderPrecisionTests.ShaderUintPrecisionCase.prototype.constructor = es3fShaderPrecisionTests.ShaderUintPrecisionCase;

    es3fShaderPrecisionTests.ShaderUintPrecisionCase.prototype.init = function() {
        assertMsgOptions(!this.m_program && !this.m_framebuffer && !this.m_renderbuffer, 'Program/Framebuffer/Renderbuffer should be null at this point.', false, true);
        // Create program.
        this.m_program = es3fShaderPrecisionTests.createIntUintPrecisionEvalProgram(gluShaderUtil.DataType.UINT, this.m_precision, this.m_op, this.m_isVertexCase);

        if (!this.m_program.isOk())
            assertMsgOptions(false, 'Compile failed', false, true);

        // Create framebuffer.
        this.m_framebuffer = gl.createFramebuffer();
        this.m_renderbuffer = gl.createRenderbuffer();

        gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_renderbuffer);
        gl.renderbufferStorage(gl.RENDERBUFFER, gl.R32UI, es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH, es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT);

        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);
        gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, this.m_renderbuffer);

        assertMsgOptions(gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE, 'Framebuffer is incomplete', false, true);

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        this.m_iterNdx = 0;

        bufferedLogToConsole("Number of accurate bits assumed = " + this.m_bits);
    };

    es3fShaderPrecisionTests.ShaderUintPrecisionCase.prototype.deinit = function() {
        if(this.m_framebuffer)
            gl.deleteFramebuffer(this.m_framebuffer);
        if(this.m_renderbuffer)
            gl.deleteRenderbuffer(this.m_renderbuffer);
        this.m_program = null;
        this.m_framebuffer = null;
        this.m_renderbuffer = null;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fShaderPrecisionTests.ShaderUintPrecisionCase.prototype.iterate = function() {
        var testPassed = true;
        var testPassedMsg = 'Pass';

        // Constant data.
        /** @type {Array<number>} */ var position = [
        -1.0, -1.0, 0.0, 1.0,
            -1.0, 1.0, 0.0, 1.0,
            1.0, -1.0, 0.0, 1.0,
            1.0, 1.0, 0.0, 1.0
    ];
        /** @type {Array<number>} */ var indices = [0, 1, 2, 2, 1, 3];

        /** @type {number} */ var numVertices = 4;
        /** @type {Array<number>} */ var in0Arr = [0, 0, 0, 0];
        /** @type {Array<number>} */ var in1Arr = [0, 0, 0, 0];

        /** @type {number} */ var mask = this.m_bits === 32 ? 0xffffffff : ((1 << this.m_bits) - 1);
        /** @type {goog.TypedArray} */ var pixels = new Uint32Array(es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH * es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT * 4);
        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays = [];

        /** @type {WebGLProgram} */ var prog = this.m_program.getProgram();

        // \todo [2012-05-03 pyry] A bit hacky.
        /** @type {boolean} */ var isMaxRangeA = this.m_rangeA[0] === 0 && this.m_rangeA[1] === 0xffffffff;
        /** @type {boolean} */ var isMaxRangeB = this.m_rangeB[0] === 0 && this.m_rangeB[1] === 0xffffffff;

        gl.useProgram(prog);
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);

        vertexArrays[0] = gluDrawUtil.newFloatVertexArrayBinding("a_position", 4, numVertices, 0, position);

        // Compute values and reference.
        for (var testNdx = 0; testNdx < this.m_numTestsPerIter; testNdx++) {
            /** @type {number} */ var in0 = (isMaxRangeA ? Math.abs(this.m_rnd.getInt()) : (this.m_rangeA[0] + Math.abs(this.m_rnd.getInt()) % (this.m_rangeA[1] - this.m_rangeA[0] + 1))) & mask;
            /** @type {number} */ var in1 = (isMaxRangeB ? Math.abs(this.m_rnd.getInt()) : (this.m_rangeB[0] + Math.abs(this.m_rnd.getInt()) % (this.m_rangeB[1] - this.m_rangeB[0] + 1))) & mask;
            /** @type {number} */ var refOut = this.m_evalFunc(in0, in1) & mask;

            bufferedLogToConsole("iter " + this.m_iterNdx + ", test " + testNdx + ": " +
                + "in0 = " + in0 + ", in1 = " + in1 + ", ref out = " + refOut)

            in0Arr = [in0, in0, in0, in0];
            in1Arr = [in1, in1, in1, in1];
            vertexArrays[1] = gluDrawUtil.newUint32VertexArrayBinding("a_in0", 1, numVertices, 0, in0Arr);
            vertexArrays[2] = gluDrawUtil.newUint32VertexArrayBinding("a_in1", 1, numVertices, 0, in1Arr);

            gluDrawUtil.draw(gl, prog, vertexArrays, gluDrawUtil.triangles(indices));

            gl.readPixels(0, 0, es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH,
                es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT, gl.RGBA_INTEGER, gl.UNSIGNED_INT, pixels);

            // Compare pixels.
            for (var y = 0; y < es3fShaderPrecisionTests.FRAMEBUFFER_HEIGHT; y++) {
                for (var x = 0; x < es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH; x++) {
                    /** @type {number} */ var cmpOut = pixels[(y*es3fShaderPrecisionTests.FRAMEBUFFER_WIDTH + x) * 4];
                    /** @type {number} */ var cmpMasked = cmpOut & mask;

                    if (cmpMasked != refOut) {
                        bufferedLogToConsole("Comparison failed (at " + x + ", " + y + "): " + "got " + cmpOut)
                        testPassed = false;
                        testPassedMsg = 'Comparison failed';
                    }
                }
            }
        }


        gl.bindFramebuffer(gl.FRAMEBUFFER, null);

        this.m_iterNdx += 1;
        if (!testPassed) {
            testFailedOptions(testPassedMsg, false);
            return tcuTestCase.IterateResult.STOP;
        } else if (testPassed && this.m_iterNdx < this.m_numIters) {
            return tcuTestCase.IterateResult.CONTINUE;
        } else {
            testPassedOptions(testPassedMsg, true);
            return tcuTestCase.IterateResult.STOP;
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fShaderPrecisionTests.ShaderPrecisionTests = function() {
        tcuTestCase.DeqpTest.call(this, 'precision', 'Shader precision requirements validation tests');
    };

    es3fShaderPrecisionTests.ShaderPrecisionTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderPrecisionTests.ShaderPrecisionTests.prototype.constructor = es3fShaderPrecisionTests.ShaderPrecisionTests;

    es3fShaderPrecisionTests.ShaderPrecisionTests.prototype.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        // Exp = Emax-2, Mantissa = 0
        // /** @type {number} */ var minF32 = tcuFloat.newFloat32((1 << 31) | (0xfd << 23) | 0x0).getValue();
        // /** @type {number} */ var maxF32 = tcuFloat.newFloat32((0 << 31) | (0xfd << 23) | 0x0).getValue();
        // [dag] Workaround for float32 numbers
        /** @type {number} */ var minF32 = new Float32Array(new Uint32Array([1<<31|0xfd<<23|0x0]).buffer)[0];
        /** @type {number} */ var maxF32 = new Float32Array(new Uint32Array([0<<31|0xfd<<23|0x0]).buffer)[0];

        // /** @type {number} */ var minF16 = tcuFloat.newFloat16(((1 << 15) | (0x1d << 10) | 0x0)).getValue();
        // /** @type {number} */ var maxF16 = tcuFloat.newFloat16(((0 << 15) | (0x1d << 10) | 0x0)).getValue();
        /** @type {number} */ var minF16 = -16384; //-1 << 14; // 1 << 15 | 0x1d | 0x0 == 0b1111010000000000; -1 * (2**(29-15)) * 1
        /** @type {number} */ var maxF16 = 16384; //1 << 14; // 0 << 15 | 0x1d | 0x0 == 0b0111010000000000; +1 * (2**(29-15)) * 1

        /** @type {Array<number>} */ var fullRange32F = [minF32, maxF32];
        /** @type {Array<number>} */ var fullRange16F = [minF16, maxF16];
        /** @type {Array<number>} */ var fullRange32I = [-2147483648, 2147483647]; // [0x80000000|0, 0x7fffffff|0]; // |0 to force the number as a 32-bit integer
        /** @type {Array<number>} */ var fullRange16I = [minF16, maxF16 - 1]; //[-(1 << 15), (1 << 15) - 1]; // Added the negative sign to index 0
        /** @type {Array<number>} */ var fullRange8I = [-128, 127]; //[-(1 << 7), (1 << 7) - 1]; // Added the negative sign to index 0
        /** @type {Array<number>} */ var fullRange32U = [0, 0xffffffff];
        /** @type {Array<number>} */ var fullRange16U = [0, 0xffff];
        /** @type {Array<number>} */ var fullRange8U = [0, 0xff];

        // \note Right now it is not programmatically verified that the results shouldn't end up being inf/nan but
        // actual values used are ok.

        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {string} op
         * @param {es3fShaderPrecisionTests.EvalFunc} evalFunc
         * @param {gluShaderUtil.precision} precision
         * @param {Array<number>} rangeA
         * @param {Array<number>} rangeB
         */
        var FloatCase = function(name, op, evalFunc, precision, rangeA, rangeB) {
            /** @type {string} */ this.name = name;
            /** @type {string} */ this.op = op;
            /** @type {es3fShaderPrecisionTests.EvalFunc} */ this.evalFunc = evalFunc;
            /** @type {gluShaderUtil.precision} */ this.precision = precision;
            /** @type {Array<number>} */ this.rangeA = rangeA;
            /** @type {Array<number>} */ this.rangeB = rangeB;
        };

        /** @type {Array<FloatCase>} */ var floatCases = [
        new FloatCase('highp_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_HIGHP, fullRange32F, fullRange32F),
            new FloatCase('highp_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_HIGHP, fullRange32F, fullRange32F),
            new FloatCase('highp_mul', 'in0 * in1', es3fShaderPrecisionTests.mul, gluShaderUtil.precision.PRECISION_HIGHP, [-1e5, 1e5], [-1e5, 1e5]),
            new FloatCase('highp_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_HIGHP, [-1e5, 1e5], [-1e5, 1e5]),
            new FloatCase('mediump_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_MEDIUMP, fullRange16F, fullRange16F),
            new FloatCase('mediump_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_MEDIUMP, fullRange16F, fullRange16F),
            new FloatCase('mediump_mul', 'in0 * in1', es3fShaderPrecisionTests.mul, gluShaderUtil.precision.PRECISION_MEDIUMP, [-1e2, 1e2], [-1e2, 1e2]),
            new FloatCase('mediump_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_MEDIUMP, [-1e2, 1e2], [-1e2, 1e2])
    ];

        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {string} op
         * @param {es3fShaderPrecisionTests.EvalFunc} evalFunc
         * @param {gluShaderUtil.precision} precision
         * @param {number} bits
         * @param {Array<number>} rangeA
         * @param {Array<number>} rangeB
         */
        var IntCase = function(name, op, evalFunc, precision, bits, rangeA, rangeB) {
            /** @type {string} */ this.name = name;
            /** @type {string} */ this.op = op;
            /** @type {es3fShaderPrecisionTests.EvalFunc} */ this.evalFunc = evalFunc;
            /** @type {gluShaderUtil.precision} */ this.precision = precision;
            /** @type {number} */ this.bits = bits;
            /** @type {Array<number>} */ this.rangeA = rangeA;
            /** @type {Array<number>} */ this.rangeB = rangeB;
        };

        /** @type {Array<IntCase>} */ var intCases = [
        new IntCase('highp_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32I, fullRange32I),
            new IntCase('highp_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32I, fullRange32I),
            new IntCase('highp_mul', 'in0 * in1', es3fShaderPrecisionTests.mul32, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32I, fullRange32I),
            new IntCase('highp_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32I, [-10000, -1]),
            new IntCase('mediump_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16I, fullRange16I),
            new IntCase('mediump_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16I, fullRange16I),
            new IntCase('mediump_mul', 'in0 * in1', es3fShaderPrecisionTests.mul, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16I, fullRange16I),
            new IntCase('mediump_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16I, [1, 1000]),
            new IntCase('lowp_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8I, fullRange8I),
            new IntCase('lowp_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8I, fullRange8I),
            new IntCase('lowp_mul', 'in0 * in1', es3fShaderPrecisionTests.mul, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8I, fullRange8I),
            new IntCase('lowp_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8I, [-50, -1])
    ];

        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {string} op
         * @param {es3fShaderPrecisionTests.EvalFunc} evalFunc
         * @param {gluShaderUtil.precision} precision
         * @param {number} bits
         * @param {Array<number>} rangeA
         * @param {Array<number>} rangeB
         */
        var UintCase = function(name, op, evalFunc, precision, bits, rangeA, rangeB) {
            /** @type {string} */ this.name = name;
            /** @type {string} */ this.op = op;
            /** @type {es3fShaderPrecisionTests.EvalFunc} */ this.evalFunc = evalFunc;
            /** @type {gluShaderUtil.precision} */ this.precision = precision;
            /** @type {number} */ this.bits = bits;
            /** @type {Array<number>} */ this.rangeA = rangeA;
            /** @type {Array<number>} */ this.rangeB = rangeB;
        };

        /** @type {Array<UintCase>} */ var uintCases = [
        new UintCase('highp_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32U, fullRange32U),
            new UintCase('highp_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32U, fullRange32U),
            new UintCase('highp_mul', 'in0 * in1', es3fShaderPrecisionTests.mul32, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32U, fullRange32U),
            new UintCase('highp_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_HIGHP, 32, fullRange32U, [1, 10000]),
            new UintCase('mediump_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16U, fullRange16U),
            new UintCase('mediump_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16U, fullRange16U),
            new UintCase('mediump_mul', 'in0 * in1', es3fShaderPrecisionTests.mul, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16U, fullRange16U),
            new UintCase('mediump_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_MEDIUMP, 16, fullRange16U, [1, 1000]),
            new UintCase('lowp_add', 'in0 + in1', es3fShaderPrecisionTests.add, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8U, fullRange8U),
            new UintCase('lowp_sub', 'in0 - in1', es3fShaderPrecisionTests.sub, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8U, fullRange8U),
            new UintCase('lowp_mul', 'in0 * in1', es3fShaderPrecisionTests.mul, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8U, fullRange8U),
            new UintCase('lowp_div', 'in0 / in1', es3fShaderPrecisionTests.div, gluShaderUtil.precision.PRECISION_LOWP, 8, fullRange8U, [1, 50])
    ];

        /** @type {tcuTestCase.DeqpTest} */ var floatGroup = tcuTestCase.newTest('float', 'Floating-point precision tests');
        testGroup.addChild(floatGroup);
        for (var ndx = 0; ndx < floatCases.length; ndx++) {
            floatGroup.addChild(new es3fShaderPrecisionTests.ShaderFloatPrecisionCase(
                floatCases[ndx].name + '_vertex', '', floatCases[ndx].op, floatCases[ndx].evalFunc,
                floatCases[ndx].precision, floatCases[ndx].rangeA, floatCases[ndx].rangeB, true));
            floatGroup.addChild(new es3fShaderPrecisionTests.ShaderFloatPrecisionCase(
                floatCases[ndx].name + '_fragment', '', floatCases[ndx].op, floatCases[ndx].evalFunc,
                floatCases[ndx].precision, floatCases[ndx].rangeA, floatCases[ndx].rangeB, false));
        }

        /** @type {tcuTestCase.DeqpTest} */ var intGroup = tcuTestCase.newTest('int', 'Integer precision tests');
        testGroup.addChild(intGroup);
        for (var ndx = 0; ndx < intCases.length; ndx++) {
            intGroup.addChild(new es3fShaderPrecisionTests.ShaderIntPrecisionCase(
                intCases[ndx].name + '_vertex', '', intCases[ndx].op, intCases[ndx].evalFunc,
                intCases[ndx].precision, intCases[ndx].bits, intCases[ndx].rangeA, intCases[ndx].rangeB, true));
            intGroup.addChild(new es3fShaderPrecisionTests.ShaderIntPrecisionCase(
                intCases[ndx].name + '_fragment', '', intCases[ndx].op, intCases[ndx].evalFunc,
                intCases[ndx].precision, intCases[ndx].bits, intCases[ndx].rangeA, intCases[ndx].rangeB, false));
        }

        /** @type {tcuTestCase.DeqpTest} */ var uintGroup = tcuTestCase.newTest('uint', 'Unsigned integer precision tests');
        testGroup.addChild(uintGroup);
        for (var ndx = 0; ndx < uintCases.length; ndx++) {
            uintGroup.addChild(new es3fShaderPrecisionTests.ShaderUintPrecisionCase(
                uintCases[ndx].name + '_vertex', '', uintCases[ndx].op, uintCases[ndx].evalFunc,
                uintCases[ndx].precision, uintCases[ndx].bits, uintCases[ndx].rangeA, uintCases[ndx].rangeB, true));
            uintGroup.addChild(new es3fShaderPrecisionTests.ShaderUintPrecisionCase(
                uintCases[ndx].name + '_fragment', '', uintCases[ndx].op, uintCases[ndx].evalFunc,
                uintCases[ndx].precision, uintCases[ndx].bits, uintCases[ndx].rangeA, uintCases[ndx].rangeB, false));
        }
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
    es3fShaderPrecisionTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderPrecisionTests.ShaderPrecisionTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fShaderPrecisionTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

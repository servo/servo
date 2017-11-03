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
goog.provide('functional.gles3.es3fShaderCommonFunctionTests');
goog.require('framework.common.tcuFloat');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluVarType');
goog.require('modules.shared.glsShaderExecUtil');
goog.scope(function() {
    var es3fShaderCommonFunctionTests = functional.gles3.es3fShaderCommonFunctionTests;
    var tcuFloat = framework.common.tcuFloat;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluVarType = framework.opengl.gluVarType;
    var deRandom = framework.delibs.debase.deRandom;
    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var glsShaderExecUtil = modules.shared.glsShaderExecUtil;

    /** @typedef {function(new: es3fShaderCommonFunctionTests.CommonFunctionCase, gluShaderUtil.DataType, gluShaderUtil.precision, gluShaderProgram.shaderType)} */ es3fShaderCommonFunctionTests.TestClass;

    /**
     * @enum
     */
    es3fShaderCommonFunctionTests.Types = {
        FLOAT: 0,
        INT: 1,
        UINT: 2
    };

    /**
     * @param {Array<number>} values
     */
    es3fShaderCommonFunctionTests.vecToFloat16 = function(values) {
        for (var ndx = 0; ndx < values.length; ndx++)
            values[ndx] = tcuFloat.newFloat16(values[ndx]).getValue();
    };

    /**
     * @param {es3fShaderCommonFunctionTests.Types} type
     * @param {deRandom.Random} rnd
     * @param {number} minValue
     * @param {number} maxValue
     * @return {number}
     */
    es3fShaderCommonFunctionTests.randomScalar = function(type, rnd, minValue, maxValue) {
        switch (type) {
            case es3fShaderCommonFunctionTests.Types.FLOAT: return rnd.getFloat(minValue, maxValue);
            case es3fShaderCommonFunctionTests.Types.INT: return rnd.getInt(minValue, maxValue);
            case es3fShaderCommonFunctionTests.Types.UINT: return Math.abs(rnd.getInt(minValue, maxValue));
            default: throw new Error('Only FLOAT, INT, and UINT are supported.');
        }
    };

    /**
     * @param {es3fShaderCommonFunctionTests.Types} type
     * @param {Array<number>} size
     * @param {deRandom.Random} rnd
     * @param {Array<number>} minValue
     * @param {Array<number>} maxValue
     * @return {Array<number>}
     */
    es3fShaderCommonFunctionTests.randomVector = function(type, size, rnd, minValue, maxValue) {
        /** @type {Array<number>} */ var res = [];
        for (var ndx = 0; ndx < size; ndx++)
            res.push(es3fShaderCommonFunctionTests.randomScalar(type, rnd, minValue[ndx], maxValue[ndx]));
        return res;
    };

    /**
     * @param {es3fShaderCommonFunctionTests.Types} type
     * @param {Array<number>} size
     * @param {deRandom.Random} rnd
     * @param {Array<number>} minValue
     * @param {Array<number>} maxValue
     * @param {number} numValues
     * @param {number=} offset
     * @return {Array<Array<number>>}
     */
    es3fShaderCommonFunctionTests.fillRandomVectors = function(type, size, rnd, minValue, maxValue, numValues, offset) {
        offset = offset === undefined ? 0 : offset;
        /** @type {Array<Array<number>>} */ var access;
        for (var ndx = 0; ndx < numValues; ndx++)
            access[offset + ndx] = es3fShaderCommonFunctionTests.randomVector(type, size, rnd, minValue, maxValue);
        return access;
    };

    /**
     * @param {es3fShaderCommonFunctionTests.Types} type
     * @param {deRandom.Random} rnd
     * @param {number} minValue
     * @param {number} maxValue
     * @param {number} numValues
     * @param {number=} offset
     * @return {Array<number>}
     */
    es3fShaderCommonFunctionTests.fillRandomScalars = function(type, rnd, minValue, maxValue, numValues, offset) {
        offset = offset === undefined ? 0 : offset;
        /** @type {Array<number>} */ var access = [];
        for (var ndx = 0; ndx < numValues; ndx++)
            access[offset + ndx] = es3fShaderCommonFunctionTests.randomScalar(type, rnd, minValue, maxValue);
        return access;
    };

    /**
     * @param {number} input
     * @param {number} output
     * @return {number}
     */
    es3fShaderCommonFunctionTests.numBitsLostInOp = function(input, output) {
        /** @type {number} */ var inExp = tcuFloat.newFloat32(input).exponent();
        /** @type {number} */ var outExp = tcuFloat.newFloat32(output).exponent();
        return Math.max(0, inExp - outExp); // Lost due to mantissa shift.
    };

    /**
     * @param {number} a
     * @param {number} b
     * @return {number}
     */
    es3fShaderCommonFunctionTests.getUlpDiff = function(a, b) {
        /** @type {number} */ var aBits = tcuFloat.newFloat32(a).bits();
        /** @type {number} */ var bBits = tcuFloat.newFloat32(b).bits();
        return aBits > bBits ? aBits - bBits : bBits - aBits;
    };

    /**
     * @param {number} a
     * @param {number} b
     * @return {number}
     */
    es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign = function(a, b) {
        if (tcuFloat.newFloat32(a).isZero())
            return es3fShaderCommonFunctionTests.getUlpDiff(new tcuFloat.deFloat().construct(tcuFloat.newFloat32(b).sign(), 0, 0).getValue(), b);
        else if (tcuFloat.newFloat32(b).isZero())
            return es3fShaderCommonFunctionTests.getUlpDiff(a, new tcuFloat.deFloat().construct(tcuFloat.newFloat32(a).sign(), 0, 0).getValue());
        else
            return es3fShaderCommonFunctionTests.getUlpDiff(a, b);
    };

    /**
     * @param {gluShaderUtil.precision} precision
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.supportsSignedZero = function(precision) {
        // \note GLSL ES 3.0 doesn't really require support for -0, but we require it for highp
        //         as it is very widely supported.
        return precision == gluShaderUtil.precision.PRECISION_HIGHP;
    };

    /**
     * @param {number} value
     * @param {number} ulpDiff
     * @return {number}
     */
    es3fShaderCommonFunctionTests.getEpsFromMaxUlpDiff = function(value, ulpDiff) {
        /** @type {number} */ var exp = tcuFloat.newFloat32(value).exponent();
        return new tcuFloat.deFloat().construct(+1, exp, (1 << 23) | ulpDiff).getValue() - new tcuFloat.deFloat().construct(+1, exp, 1 << 23).getValue();
    };

    /**
     * @param {number} numAccurateBits
     * @return {number}
     */
    es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits = function(numAccurateBits) {
        /** @type {number} */ var numGarbageBits = 23 - numAccurateBits;
        /** @type {number} */ var mask = (1 << numGarbageBits) - 1;

        return mask;
    };

    /**
     * @param {number} value
     * @param {number} numAccurateBits
     * @return {number}
     */
    es3fShaderCommonFunctionTests.getEpsFromBits = function(value, numAccurateBits) {
        return es3fShaderCommonFunctionTests.getEpsFromMaxUlpDiff(value, es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(numAccurateBits));
    };

    /**
     * @param {gluShaderUtil.precision} precision
     * @return {number}
     */
    es3fShaderCommonFunctionTests.getMinMantissaBits = function(precision) {
        /** @type {Array<number>} */ var bits = [
            7, // lowp
            10, // mediump
            23 // highp
        ];

        assertMsgOptions(deMath.deInBounds32(precision, 0, bits.length), 'Unexpected precision option.', false, true);
        return bits[precision];
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.CommonFunctionCase = function(name, description, shaderType) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {gluShaderProgram.shaderType} */ this.m_shaderType = shaderType;
        /** @type {number} */ this.m_numValues = 100;
        /** @type {glsShaderExecUtil.ShaderExecutor} */ this.m_executor = null;
        /** @type {glsShaderExecUtil.ShaderSpec} */ this.m_spec = new glsShaderExecUtil.ShaderSpec();
        this.m_spec.version = gluShaderUtil.GLSLVersion.V300_ES;
        /** @type {string} */ this.m_failMsg; //!< Comparison failure help message.
    };

    es3fShaderCommonFunctionTests.CommonFunctionCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderCommonFunctionTests.CommonFunctionCase.prototype.constructor = es3fShaderCommonFunctionTests.CommonFunctionCase;

    es3fShaderCommonFunctionTests.CommonFunctionCase.prototype.init = function() {
        assertMsgOptions(!this.m_executor, 'Shader executor should be null at this point', false, true);
        this.m_executor = glsShaderExecUtil.createExecutor(this.m_shaderType, this.m_spec);
        if (!this.m_executor.isOk())
            throw new Error('Compile failed');
    };

    es3fShaderCommonFunctionTests.CommonFunctionCase.prototype.deinit = function() {
        this.m_executor = null;
    };

    /**
     * @param {Array<glsShaderExecUtil.Symbol>} symbols
     * @return {Array<number>}
     */
    es3fShaderCommonFunctionTests.getScalarSizes = function(symbols) {
        /** @type {Array<number>} */ var sizes = [];
        for (var ndx = 0; ndx < symbols.length; ++ndx)
            sizes.push(symbols[ndx].varType.getScalarSize());
        return sizes;
    };

    /**
     * @param {Array<glsShaderExecUtil.Symbol>} symbols
     * @return {number}
     */
    es3fShaderCommonFunctionTests.computeTotalScalarSize = function(symbols) {
        /** @type {number} */ var totalSize = 0;
        for (var sym in symbols)
            totalSize += symbols[sym].varType.getScalarSize();
        return totalSize;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fShaderCommonFunctionTests.CommonFunctionCase.prototype.iterate = function() {
        /** @type {number} */ var numInputScalars = es3fShaderCommonFunctionTests.computeTotalScalarSize(this.m_spec.inputs);
        /** @type {number} */ var numOutputScalars = es3fShaderCommonFunctionTests.computeTotalScalarSize(this.m_spec.outputs);
        /** @type {Array<goog.TypedArray>} */ var inputData = [];
        /** @type {Array<goog.TypedArray>} */ var outputData = [];
        /** @type {gluShaderUtil.DataType} */ var inputType = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.DataType} */ var outputType = this.m_spec.outputs[0].varType.getBasicType();
        /** @type {Array<Array<number>>} */ var inputValues;
        /** @type {ArrayBuffer} */ var outputValues;
        inputValues = this.getInputValues(this.m_numValues);

        for (var inNdx = 0; inNdx < inputValues.length; inNdx++) {
            var data = inputType >= gluShaderUtil.DataType.FLOAT && inputType <= gluShaderUtil.DataType.FLOAT_VEC4 ? new Float32Array(inputValues[inNdx]) :
                       inputType >= gluShaderUtil.DataType.INT && inputType <= gluShaderUtil.DataType.INT_VEC4 ? new Int32Array(inputValues[inNdx]) :
                       inputType >= gluShaderUtil.DataType.UINT && inputType <= gluShaderUtil.DataType.UINT_VEC4 ? new Uint32Array(inputValues[inNdx]) :
                       null;
            inputData.push(data);
        }

        // Execute shader.
        this.m_executor.useProgram();
        outputValues = this.m_executor.execute(this.m_numValues, inputData);
        for (var outNdx = 0; outNdx < outputValues.length; outNdx++) {
            var data = outputType >= gluShaderUtil.DataType.FLOAT && outputType <= gluShaderUtil.DataType.FLOAT_VEC4 ? new Float32Array(outputValues[outNdx].buffer) :
                        outputType >= gluShaderUtil.DataType.INT && outputType <= gluShaderUtil.DataType.INT_VEC4 ? new Int32Array(outputValues[outNdx].buffer) :
                        outputType >= gluShaderUtil.DataType.UINT && outputType <= gluShaderUtil.DataType.UINT_VEC4 ? new Uint32Array(outputValues[outNdx].buffer) :
                        outputType >= gluShaderUtil.DataType.BOOL && outputType <= gluShaderUtil.DataType.BOOL_VEC4 ? new Int32Array(outputValues[outNdx].buffer) :
                        null;
            outputData.push(data);
        }

        // TODO: verify proper TypedArray for BOOL types; defaulting to Int32Array in the mean time (outputValues returns 400 bytes, we need 100 elements)
        // Compare results.
        /** @type {Array<number>} */ var inScalarSizes = es3fShaderCommonFunctionTests.getScalarSizes(this.m_spec.inputs);
        /** @type {Array<number>} */ var outScalarSizes = es3fShaderCommonFunctionTests.getScalarSizes(this.m_spec.outputs);
        /** @type {Array<*>} */ var curInputPtr = [];
        /** @type {Array<*>} */ var curOutputPtr = [];
        /** @type {number} */ var numFailed = 0;

        for (var inNdx = 0; inNdx < inputData.length; inNdx++) {
            curInputPtr[inNdx] = [];
            for (var valNdx = 0; valNdx < inputData[inNdx].length; valNdx += inScalarSizes[inNdx])
                curInputPtr[inNdx].push(inputData[inNdx].slice(valNdx, valNdx + inScalarSizes[inNdx]));
        }

        for (var outNdx = 0; outNdx < outputData.length; outNdx++) {
            curOutputPtr[outNdx] = [];
            for (var valNdx = 0; valNdx < outputData[outNdx].length; valNdx += outScalarSizes[outNdx])
                curOutputPtr[outNdx].push(outputData[outNdx].slice(valNdx, valNdx + outScalarSizes[outNdx]));
        }

        for (var valNdx = 0; valNdx < this.m_numValues; valNdx++) {
            var curInputValues = [];
            var curOutputValues = [];
            for (var inNdx = 0; inNdx < inputData.length; inNdx++) {
                curInputValues.push(curInputPtr[inNdx][valNdx]);
            }
            for (var outNdx = 0; outNdx < outputData.length; outNdx++) {
                curOutputValues.push(curOutputPtr[outNdx][valNdx]);
            }
            if (!this.compare(curInputValues, curOutputValues)) {
                // \todo [2013-08-08 pyry] We probably want to log reference value as well?

                bufferedLogToConsole('ERROR: comparison failed for value ' + valNdx + ':\n ' + this.m_failMsg);
                bufferedLogToConsole(' inputs:');
                bufferedLogToConsole(' ' + this.m_spec.inputs[0].name + ' = ' + this.m_spec.inputs[0].varType.toString() + ' ' + curInputPtr[valNdx]);
                bufferedLogToConsole(' outputs:');
                bufferedLogToConsole(' ' + this.m_spec.outputs[0].name + ' = ' + this.m_spec.outputs[0].varType.toString() + ' ' + curOutputPtr[valNdx]);

                this.m_failMsg = '';
                numFailed += 1;
            }
        }

        bufferedLogToConsole((this.m_numValues - numFailed) + ' / ' + this.m_numValues + ' values passed');

        /** @type {boolean} */ var isOk = numFailed === 0;

        if (!isOk)
            testFailedOptions('Result comparison failed', false);
        else
            testPassedOptions('Pass', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @param {gluShaderUtil.precision} precision
     * @return {string}
     */
    es3fShaderCommonFunctionTests.getPrecisionPostfix = function(precision) {
        /** @type {Array<string>} */ var s_postfix = [
            '_lowp',
            '_mediump',
            '_highp'
        ];
        assertMsgOptions(0 <= precision && precision < s_postfix.length, 'Error: Out of range', false, true);
        return s_postfix[precision];
    };

    /**
     * @param {gluShaderProgram.shaderType} shaderType
     * @return {string}
     */
    es3fShaderCommonFunctionTests.getShaderTypePostfix = function(shaderType) {
        /** @type {Array<string>} */ var s_postfix = [
            '_vertex',
            '_fragment'
        ];
        assertMsgOptions(0 <= shaderType && shaderType < s_postfix.length, 'Error Out of range', false, true);
        return s_postfix[shaderType];
    };

    /**
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     * @return {string}
     */
    es3fShaderCommonFunctionTests.getCommonFuncCaseName = function(baseType, precision, shaderType) {
        return gluShaderUtil.getDataTypeName(baseType) +
            es3fShaderCommonFunctionTests.getPrecisionPostfix(precision) +
            es3fShaderCommonFunctionTests.getShaderTypePostfix(shaderType);
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.AbsCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'abs', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = abs(in0);';
    };

    es3fShaderCommonFunctionTests.AbsCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.AbsCase.prototype.constructor = es3fShaderCommonFunctionTests.AbsCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.AbsCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var floatRanges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {Array<Array<number>>} */ var intRanges = [
            [-(1 << 7) + 1, (1 << 7) - 1],
            [-(1 << 15) + 1, (1 << 15) - 1],
            [-0x80000000 + 1, 0x7fffffff]
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0x235fac);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];

        if (gluShaderUtil.isDataTypeFloatOrVec(type))
            values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, floatRanges[precision][0], floatRanges[precision][1], numValues * scalarSize);
        else
            values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.INT, rnd, intRanges[precision][0], intRanges[precision][1], numValues * scalarSize);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.AbsCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref0;

        if (gluShaderUtil.isDataTypeFloatOrVec(type)) {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var maxUlpDiff = (1 << (23 - mantissaBits)) - 1;

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref0 = Math.abs(in0);
                /** @type {number} */ var ulpDiff0 = es3fShaderCommonFunctionTests.getUlpDiff(out0, ref0);

                if (ulpDiff0 > maxUlpDiff) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref0 /*HexFloat(ref0)*/ + ' with ULP threshold ' + maxUlpDiff + ', got ULP diff ' + ulpDiff0;
                    return false;
                }
            }
        } else
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref0 = Math.abs(in0);

                if (out0 != ref0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref0;
                    return false;
                }
            }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.SignCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'sign', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = sign(in0);';
    };

    es3fShaderCommonFunctionTests.SignCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.SignCase.prototype.constructor = es3fShaderCommonFunctionTests.SignCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.SignCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var floatRanges = [
            [-2.0, 2.0], // lowp
            [-1e4, 1e4], // mediump
            [-1e8, 1e8] // highp
        ];

        /** @type {Array<Array<number>>} */ var intRanges = [
            [-(1 << 7), (1 << 7) - 1],
            [-(1 << 15), (1 << 15) - 1],
            [0x80000000, 0x7fffffff]
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0x324);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];

        if (gluShaderUtil.isDataTypeFloatOrVec(type)) {
            // Special cases.
            // [dag] The special cases are 1, -1, and 0
            var specialCases = [1.0, -1.0, 0.0];
            for (var caseNdx = 0; caseNdx < specialCases.length; caseNdx++)
                for (var scalarNdx = 0; scalarNdx < scalarSize; scalarNdx++) {
                    values[0].push(specialCases[caseNdx]);
                }
            values[0] = values[0].concat(es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, floatRanges[precision][0], floatRanges[precision][1], (numValues - 3) * scalarSize));
        } else {
            var specialCases = [1, -1, 0];
            for (var caseNdx = 0; caseNdx < specialCases.length; caseNdx++)
                for (var scalarNdx = 0; scalarNdx < scalarSize; scalarNdx++) {
                    values[0].push(specialCases[caseNdx]);
                }
            values[0] = values[0].concat(es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.INT, rnd, intRanges[precision][0], intRanges[precision][1], (numValues - 3) * scalarSize));
        }

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.SignCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref0;

        if (gluShaderUtil.isDataTypeFloatOrVec(type)) {
            // Both highp and mediump should be able to represent -1, 0, and +1 exactly
            /** @type {number} */ var maxUlpDiff = precision === gluShaderUtil.precision.PRECISION_LOWP ?
                es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(es3fShaderCommonFunctionTests.getMinMantissaBits(precision)) :
                0;

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref0 = in0 < 0.0 ? -1.0 :
                       in0 > 0.0 ? 1.0 : 0.0;
                /** @type {number} */ var ulpDiff0 = es3fShaderCommonFunctionTests.getUlpDiff(out0, ref0);

                if (ulpDiff0 > maxUlpDiff) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref0 /*HexFloat(ref0)*/ + ' with ULP threshold ' + maxUlpDiff + ', got ULP diff ' + ulpDiff0;
                    return false;
                }
            }
        } else {
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref0 = in0 < 0 ? -1 :
                       in0 > 0 ? 1 : 0;

                if (out0 != ref0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref0;
                    return false;
                }
            }
        }

        return true;
    };

    /**
     * @param {number} v
     * @return {number}
     */
    es3fShaderCommonFunctionTests.roundEven = function(v) {
        /** @type {number} */ var q = deMath.deFloatFrac(v);
        /** @type {number} */ var truncated = Math.trunc(v - q);
        /** @type {number} */ var rounded = (q > 0.5) ? (truncated + 1) : // Rounded up
            (q == 0.5 && (truncated % 2 != 0)) ? (truncated + 1) : // Round to nearest even at 0.5
            truncated; // Rounded down
        return rounded;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.RoundEvenCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'roundEven', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = roundEven(in0);';
    };

    es3fShaderCommonFunctionTests.RoundEvenCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.RoundEvenCase.prototype.constructor = es3fShaderCommonFunctionTests.RoundEvenCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.RoundEvenCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var numSpecialCases = 0;
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];
        // Special cases.
        if (precision !== gluShaderUtil.precision.PRECISION_LOWP) {
            assertMsgOptions(numValues >= 20, 'numValues should be greater or equal than 20', false, true);
            for (var ndx = 0; ndx < 20; ndx++) {
                /** @type {number} */ var v = deMath.clamp(ndx - 10.5, ranges[precision][0], ranges[precision][1]);
                for (var scalarNdx = 0; scalarNdx < scalarSize; scalarNdx++) {
                    values[0].push(v);
                }
                numSpecialCases += 1;
            }
        }

        // Random cases.
        values[0] = values[0].concat(es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], (numValues - numSpecialCases) * scalarSize));

        // If precision is mediump, make sure values can be represented in fp16 exactly
        if (precision === gluShaderUtil.precision.PRECISION_MEDIUMP)
            es3fShaderCommonFunctionTests.vecToFloat16(values[0]);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.RoundEvenCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var in0;
        /** @type {number} */ var out0;

        if (precision == gluShaderUtil.precision.PRECISION_HIGHP || precision == gluShaderUtil.precision.PRECISION_MEDIUMP) {
            // Require exact rounding result.
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {number} */ var ref = es3fShaderCommonFunctionTests.roundEven(in0);

                /** @type {number} */ var ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref);

                if (ulpDiff > 0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ', got ULP diff ' + ulpDiff;
                    return false;
                }
            }
        } else {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits); // ULP diff for rounded integer value.
            /** @type {number} */ var eps = es3fShaderCommonFunctionTests.getEpsFromBits(1.0, mantissaBits); // epsilon for rounding bounds

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {number} */ var minRes = Math.floor(es3fShaderCommonFunctionTests.roundEven(in0 - eps));
                /** @type {number} */ var maxRes = Math.floor(es3fShaderCommonFunctionTests.roundEven(in0 + eps));
                /** @type {boolean} */ var anyOk = false;

                for (var roundedVal = minRes; roundedVal <= maxRes; roundedVal++) {
                    ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, roundedVal);

                    if (ulpDiff <= maxUlpDiff) {
                        anyOk = true;
                        break;
                    }
                }

                if (!anyOk) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = [' + minRes + ', ' + maxRes + '] with ULP threshold ' + maxUlpDiff;
                    return false;
                }
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.ModfCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'modf', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out1', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = modf(in0, out1);';
    };

    es3fShaderCommonFunctionTests.ModfCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.ModfCase.prototype.constructor = es3fShaderCommonFunctionTests.ModfCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.ModfCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], numValues * scalarSize);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.ModfCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {boolean} */ var hasZeroSign = es3fShaderCommonFunctionTests.supportsSignedZero(precision);
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var out1;

        for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
            in0 = inputs[0][compNdx];
            out0 = outputs[0][compNdx];
            out1 = outputs[1][compNdx];

            /** @type {number} */ var refOut1 = Math.floor(in0);
            /** @type {number} */ var refOut0 = in0 - refOut1;

            /** @type {number} */ var bitsLost = precision != gluShaderUtil.precision.PRECISION_HIGHP ? es3fShaderCommonFunctionTests.numBitsLostInOp(in0, refOut0) : 0;
            /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(Math.max(mantissaBits - bitsLost, 0));

            /** @type {number} */ var resSum = out0 + out1;

            /** @type {number} */ var ulpDiff = hasZeroSign ? es3fShaderCommonFunctionTests.getUlpDiff(resSum, in0) : es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(resSum, in0);

            if (ulpDiff > maxUlpDiff) {
                this.m_failMsg += 'Expected [' + compNdx + '] = (' + refOut0 + ') + (' + refOut1 + ') = ' + in0 + ' with ULP threshold ' +
                            maxUlpDiff + ', got ULP diff ' + ulpDiff;
                return false;
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.IsnanCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'isnan', shaderType);
        assertMsgOptions(gluShaderUtil.isDataTypeFloatOrVec(baseType), 'Assert error.', false, true);

        /** @type {number} */ var vecSize = gluShaderUtil.getDataTypeScalarSize(baseType);
        /** @type {gluShaderUtil.DataType} */ var boolType = vecSize > 1 ?
            gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.BOOL, vecSize) :
            gluShaderUtil.DataType.BOOL;

        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(boolType)));
        this.m_spec.source = 'out0 = isnan(in0);';
    };

    es3fShaderCommonFunctionTests.IsnanCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.IsnanCase.prototype.constructor = es3fShaderCommonFunctionTests.IsnanCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.IsnanCase.prototype.getInputValues = function(numValues) {
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xc2a39f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
        /** @type {number} */ var mantissaMask = (~es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits)) & ((1 << 23) - 1);
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];

        for (var valNdx = 0; valNdx < numValues * scalarSize; valNdx++) {
            /** @type {boolean} */ var isNan = rnd.getFloat() > 0.3;
            /** @type {boolean} */ var isInf = !isNan && rnd.getFloat() > 0.4;
            /** @type {number} */ var mantissa = !isInf ? ((1 << 22) | (Math.abs(rnd.getInt()) & mantissaMask)) : 0;
            /** @type {number} */ var exp = !isNan && !isInf ? (Math.abs(rnd.getInt()) & 0x7f) : 0xff;
            /** @type {number} */ var sign = Math.abs(rnd.getInt()) & 0x1;
            /** @type {number} */ var value = (sign << 31) | (exp << 23) | mantissa;

            // Convert int to float.
            var view = new DataView(new ArrayBuffer(4));
            view.setInt32(0, value, true);
            value = view.getFloat32(0, true);

            assertMsgOptions(tcuFloat.newFloat32(value).isInf() === isInf && tcuFloat.newFloat32(value).isNaN() === isNan, 'Assert error.', false, true);

            values[0].push(value);
        }

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.IsnanCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP) {
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref = tcuFloat.newFloat32(in0).isNaN() ? 1 : 0;

                if (out0 !== ref) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref;
                    return false;
                }
            }
        } else {
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                out0 = outputs[0][compNdx];

                if (out0 !== 0 && out0 !== 1) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = 0 / 1';
                    return false;
                }
            }
        }
        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.IsinfCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'isinf', shaderType);
        assertMsgOptions(gluShaderUtil.isDataTypeFloatOrVec(baseType), 'Assert error.', false, true);

        /** @type {number} */ var vecSize = gluShaderUtil.getDataTypeScalarSize(baseType);
        /** @type {gluShaderUtil.DataType} */ var boolType = vecSize > 1 ?
            gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.BOOL, vecSize) :
            gluShaderUtil.DataType.BOOL;

        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(boolType)));
        this.m_spec.source = 'out0 = isinf(in0);';
    };

    es3fShaderCommonFunctionTests.IsinfCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.IsinfCase.prototype.constructor = es3fShaderCommonFunctionTests.IsinfCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.IsinfCase.prototype.getInputValues = function(numValues) {
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xc2a39f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
        /** @type {number} */ var mantissaMask = (~es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits)) & ((1 << 23) - 1);
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];

        for (var valNdx = 0; valNdx < numValues * scalarSize; valNdx++) {
            /** @type {boolean} */ var isInf = rnd.getFloat() > 0.3;
            /** @type {boolean} */ var isNan = !isInf && rnd.getFloat() > 0.4;
            /** @type {number} */ var mantissa = !isInf ? ((1 << 22) | (Math.abs(rnd.getInt()) & mantissaMask)) : 0;
            /** @type {number} */ var exp = !isNan && !isInf ? (Math.abs(rnd.getInt()) & 0x7f) : 0xff;
            /** @type {number} */ var sign = Math.abs(rnd.getInt()) & 0x1;
            /** @type {number} */ var value = (sign << 31) | (exp << 23) | mantissa;

            // Convert int to float.
            var view = new DataView(new ArrayBuffer(4));
            view.setInt32(0, value, true);
            value = view.getFloat32(0, true);

            assertMsgOptions(tcuFloat.newFloat32(value).isInf() === isInf && tcuFloat.newFloat32(value).isNaN() === isNan, 'Assert error.', false, true);

            values[0].push(value);
        }

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.IsinfCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP) {
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref = tcuFloat.newFloat32(in0).isInf() ? 1 : 0;

                if (out0 !== ref) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref;
                    return false;
                }
            }
        } else {
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                out0 = outputs[0][compNdx];

                if (out0 !== 0 && out0 !== 1) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = 0 / 1';
                    return false;
                }
            }
        }
        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     * @param {boolean} outIsSigned
     */
    es3fShaderCommonFunctionTests.FloatBitsToUintIntCase = function(baseType, precision, shaderType, outIsSigned) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            outIsSigned ? 'floatBitsToInt' : 'floatBitsToUint', shaderType);

        /** @type {number} */ var vecSize = gluShaderUtil.getDataTypeScalarSize(baseType);
        /** @type {gluShaderUtil.DataType} */ var intType = outIsSigned ?
            (vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.INT, vecSize) : gluShaderUtil.DataType.INT) :
            (vecSize > 1 ? gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.UINT, vecSize) : gluShaderUtil.DataType.UINT);

        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(intType, gluShaderUtil.precision.PRECISION_HIGHP)));
        this.m_spec.source = outIsSigned ? 'out0 = floatBitsToInt(in0);' : 'out0 = floatBitsToUint(in0);';
    };

    es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.prototype.constructor = es3fShaderCommonFunctionTests.FloatBitsToUintIntCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.prototype.getInputValues = function(numValues) {

        /** @type {Array<number>} */ var ranges = [
                [-2.0, 2.0], // lowp
                [-1e3, 1e3], // mediump
                [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0x2790a);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {Array<Array<number>>} */ var values = [];

        values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], numValues * scalarSize);
        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
        /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var refOut0;
        /** @type {number} */ var ulpDiff;

        for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
            in0 = inputs[0][compNdx];
            out0 = outputs[0][compNdx];

            // Convert int to uint because ref out is in uint format.
            var view = new DataView(new ArrayBuffer(4));
            view.setInt32(0, out0, true);
            out0 = view.getUint32(0, true);

            refOut0 = tcuFloat.newFloat32(in0).bits();
            ulpDiff = Math.abs(out0 - refOut0);
            if (ulpDiff > maxUlpDiff) {
                this.m_failMsg += 'Expected [' + compNdx + '] = ' + refOut0 + ' with threshold ' +
                            maxUlpDiff + ', got diff ' + ulpDiff;
                return false;
            }
        }
        return true;
    };

    /**
      * @constructor
      * @extends {es3fShaderCommonFunctionTests.FloatBitsToUintIntCase}
      * @param {gluShaderUtil.DataType} baseType
      * @param {gluShaderUtil.precision} precision
      * @param {gluShaderProgram.shaderType} shaderType
      */
    es3fShaderCommonFunctionTests.FloatBitsToIntCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.call(this, baseType, precision, shaderType, true);
    };

    es3fShaderCommonFunctionTests.FloatBitsToIntCase.prototype = Object.create(es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.prototype);
    es3fShaderCommonFunctionTests.FloatBitsToIntCase.prototype.constructor = es3fShaderCommonFunctionTests.FloatBitsToIntCase;

    /**
      * @constructor
      * @extends {es3fShaderCommonFunctionTests.FloatBitsToUintIntCase}
      * @param {gluShaderUtil.DataType} baseType
      * @param {gluShaderUtil.precision} precision
      * @param {gluShaderProgram.shaderType} shaderType
      */
    es3fShaderCommonFunctionTests.FloatBitsToUintCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.call(this, baseType, precision, shaderType, false);
    };

    es3fShaderCommonFunctionTests.FloatBitsToUintCase.prototype = Object.create(es3fShaderCommonFunctionTests.FloatBitsToUintIntCase.prototype);
    es3fShaderCommonFunctionTests.FloatBitsToUintCase.prototype.constructor = es3fShaderCommonFunctionTests.FloatBitsToUintCase;

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.BitsToFloatCase = function(baseType, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, gluShaderUtil.precision.PRECISION_HIGHP, shaderType),
            gluShaderUtil.isDataTypeIntOrIVec(baseType) ? 'intBitsToFloat' : 'uintBitsToFloat', shaderType);
        /** @type {boolean} */ var inIsSigned = gluShaderUtil.isDataTypeIntOrIVec(baseType);
        /** @type {number} */ var vecSize = gluShaderUtil.getDataTypeScalarSize(baseType);
        /** @type {gluShaderUtil.DataType} */ var floatType = vecSize > 1 ? gluShaderUtil.getDataTypeFloatVec(vecSize) : gluShaderUtil.DataType.FLOAT;

        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, gluShaderUtil.precision.PRECISION_HIGHP)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(floatType, gluShaderUtil.precision.PRECISION_HIGHP)));
        this.m_spec.source = inIsSigned ? 'out0 = intBitsToFloat(in0);' : 'out0 = uintBitsToFloat(in0);';
    };

    es3fShaderCommonFunctionTests.BitsToFloatCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.BitsToFloatCase.prototype.constructor = es3fShaderCommonFunctionTests.BitsToFloatCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.BitsToFloatCase.prototype.getInputValues = function(numValues) {
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xbbb225);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {Array<number>} */ var range = [-1e8, 1e8];
        /** @type {Array<Array<number>>} */ var values = [];

        values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, range[0], range[1], numValues * scalarSize);
        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.BitsToFloatCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var maxUlpDiff = 0;

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ulpDiff;
        /** @type {number} */ var refOut0;

        for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
            in0 = inputs[0][compNdx];
            out0 = outputs[0][compNdx];

            // Convert int to float
            var view = new DataView(new ArrayBuffer(4));
            view.setInt32(0, in0, true);
            in0 = view.getFloat32(0, true);

            ulpDiff = es3fShaderCommonFunctionTests.getUlpDiff(in0, out0);
            if (ulpDiff > maxUlpDiff) {
                this.m_failMsg += 'Expected [' + compNdx + '] = ' + in0 + ' with ULP threshold ' +
                            maxUlpDiff + ', got ULP diff ' + ulpDiff;
                return false;
            }
        }
        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.FloorCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'floor', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = floor(in0);';
    };

    es3fShaderCommonFunctionTests.FloorCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.FloorCase.prototype.constructor = es3fShaderCommonFunctionTests.FloorCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.FloorCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {Array<Array<number>>} */ var values = [];
        // Random cases.
        values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], numValues * scalarSize);

        // If precision is mediump, make sure values can be represented in fp16 exactly
        if (precision === gluShaderUtil.precision.PRECISION_MEDIUMP)
            es3fShaderCommonFunctionTests.vecToFloat16(values[0]);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.FloorCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref;
        /** @type {number} */ var ulpDiff;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP || precision === gluShaderUtil.precision.PRECISION_MEDIUMP) {
            // Require exact result.
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref = Math.floor(in0);

                ulpDiff = es3fShaderCommonFunctionTests.getUlpDiff(out0, ref);

                if (ulpDiff > 0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ', got ULP diff ' + ulpDiff;
                    return false;
                }
            }
        } else {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits); // ULP diff for rounded integer value.
            /** @type {number} */ var eps = es3fShaderCommonFunctionTests.getEpsFromBits(1.0, mantissaBits); // epsilon for rounding bounds

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {number} */ var minRes = Math.floor(in0 - eps);
                /** @type {number} */ var maxRes = Math.floor(in0 + eps);
                /** @type {boolean} */ var anyOk = false;

                for (var roundedVal = minRes; roundedVal <= maxRes; roundedVal++) {
                    ulpDiff = es3fShaderCommonFunctionTests.getUlpDiff(out0, roundedVal);

                    if (ulpDiff <= maxUlpDiff) {
                        anyOk = true;
                        break;
                    }
                }

                if (!anyOk) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = [' + minRes + ', ' + maxRes + '] with ULP threshold ' + maxUlpDiff;
                    return false;
                }
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.TruncCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'trunc', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = trunc(in0);';
    };

    es3fShaderCommonFunctionTests.TruncCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.TruncCase.prototype.constructor = es3fShaderCommonFunctionTests.TruncCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.TruncCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {Array<number>} */ var specialCases = [0.0, -0.0, -0.9, 0.9, 1.0, -1.0];
        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];

        // Special cases
        for (var caseNdx = 0; caseNdx < specialCases.length; caseNdx++)
        for (var scalarNdx = 0; scalarNdx < scalarSize; scalarNdx++)
            values[0].push(specialCases[caseNdx]);

        // Random cases.
        values[0] = values[0].concat(es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], (numValues - specialCases.length) * scalarSize));

        // If precision is mediump, make sure values can be represented in fp16 exactly
        if (precision === gluShaderUtil.precision.PRECISION_MEDIUMP)
            es3fShaderCommonFunctionTests.vecToFloat16(values[0]);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.TruncCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref;
        /** @type {number} */ var ulpDiff;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP || precision === gluShaderUtil.precision.PRECISION_MEDIUMP) {
            // Require exact result.
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {boolean} */ var isNeg = tcuFloat.newFloat32(in0).sign() < 0;
                ref = isNeg ? (-Math.floor(-in0)) : Math.floor(in0);

                // \note: trunc() function definition is a bit broad on negative zeros. Ignore result sign if zero.
                ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref);

                if (ulpDiff > 0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ', got ULP diff ' + ulpDiff;
                    return false;
                }
            }
        } else {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits); // ULP diff for rounded integer value.
            /** @type {number} */ var eps = es3fShaderCommonFunctionTests.getEpsFromBits(1.0, mantissaBits); // epsilon for rounding bounds

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {number} */ var minRes = Math.trunc(in0 - eps);
                /** @type {number} */ var maxRes = Math.trunc(in0 + eps);
                /** @type {boolean} */ var anyOk = false;

                for (var roundedVal = minRes; roundedVal <= maxRes; roundedVal++) {
                    ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, roundedVal);

                    if (ulpDiff <= maxUlpDiff) {
                        anyOk = true;
                        break;
                    }
                }

                if (!anyOk) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = [' + minRes + ', ' + maxRes + '] with ULP threshold ' + maxUlpDiff;
                    return false;
                }
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.RoundCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'round', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = round(in0);';
    };

    es3fShaderCommonFunctionTests.RoundCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.RoundCase.prototype.constructor = es3fShaderCommonFunctionTests.RoundCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.RoundCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var numSpecialCases = 0;

        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = []

        // Special cases.
        if (precision === gluShaderUtil.precision.PRECISION_LOWP) {
            assertMsgOptions(numValues >= 10, 'Sample too small.', false, true);
            for (var ndx = 0; ndx < 10; ndx++) {
                /** @type {number} */ var v = deMath.clamp(ndx - 5.5, ranges[precision][0], ranges[precision][1]);
                for (var iter = 1; iter <= scalarSize; iter++)
                    values[0].push(v);
                numSpecialCases += 1;
            }
        }

        // Random cases.
        values[0] = values[0].concat(es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], (numValues - numSpecialCases) * scalarSize));

        // If precision is mediump, make sure values can be represented in fp16 exactly
        if (precision === gluShaderUtil.precision.PRECISION_MEDIUMP)
            es3fShaderCommonFunctionTests.vecToFloat16(values[0]);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.RoundCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {boolean} */ var hasZeroSign = es3fShaderCommonFunctionTests.supportsSignedZero(precision);
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ulpDiff;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP || precision === gluShaderUtil.precision.PRECISION_MEDIUMP) {
            // Require exact result.
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];

                if ((in0 - Math.floor(in0)) === 0.5) {
                    /** @type {number} */ var ref0 = Math.floor(in0);
                    /** @type {number} */ var ref1 = Math.ceil(in0);
                    /** @type {number} */ var ulpDiff0 = hasZeroSign ? es3fShaderCommonFunctionTests.getUlpDiff(out0, ref0) : es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref0);
                    /** @type {number} */ var ulpDiff1 = hasZeroSign ? es3fShaderCommonFunctionTests.getUlpDiff(out0, ref1) : es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref1);
                    if (ulpDiff0 > 0 && ulpDiff1 > 0) {
                        this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref0 + ' or ' + ref1 + ', got ULP diff ' + Math.min(ulpDiff0, ulpDiff1);
                        return false;
                    }
                } else {
                    // Require exact result
                    /** @type {number} */ var ref = es3fShaderCommonFunctionTests.roundEven(in0);
                    ulpDiff = hasZeroSign ? es3fShaderCommonFunctionTests.getUlpDiff(out0, ref) : es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref);

                    if (ulpDiff > 0) {
                        this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ', got ULP diff ' + ulpDiff;
                        return false;
                    }
                }
            }
        } else {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits); // ULP diff for rounded integer value.
            /** @type {number} */ var eps = es3fShaderCommonFunctionTests.getEpsFromBits(1.0, mantissaBits); // epsilon for rounding bounds

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {number} */ var minRes = Math.floor(es3fShaderCommonFunctionTests.roundEven(in0 - eps));
                /** @type {number} */ var maxRes = Math.floor(es3fShaderCommonFunctionTests.roundEven(in0 + eps));
                /** @type {boolean} */ var anyOk = false;

                for (var roundedVal = minRes; roundedVal <= maxRes; roundedVal++) {
                    ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, roundedVal);

                    if (ulpDiff <= maxUlpDiff) {
                        anyOk = true;
                        break;
                    }
                }

                if (!anyOk) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = [' + minRes + ', ' + maxRes + '] with ULP threshold ' + maxUlpDiff;
                    return false;
                }
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.CeilCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'ceil', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = ceil(in0);';
    };

    es3fShaderCommonFunctionTests.CeilCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.CeilCase.prototype.constructor = es3fShaderCommonFunctionTests.CeilCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.CeilCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {Array<Array<number>>} */ var values = [];

        // Random cases.
        values[0] = es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], numValues * scalarSize);

        // If precision is mediump, make sure values can be represented in fp16 exactly
        if (precision === gluShaderUtil.precision.PRECISION_MEDIUMP)
            es3fShaderCommonFunctionTests.vecToFloat16(values[0]);

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.CeilCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {boolean} */ var hasZeroSign = es3fShaderCommonFunctionTests.supportsSignedZero(precision);
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref;
        /** @type {number} */ var ulpDiff;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP || precision === gluShaderUtil.precision.PRECISION_MEDIUMP) {
            // Require exact result.
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref = Math.ceil(in0);
                ulpDiff = hasZeroSign ? es3fShaderCommonFunctionTests.getUlpDiff(out0, ref) : es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref);

                if (ulpDiff > 0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ', got ULP diff ' + ulpDiff;
                    return false;
                }
            }
        } else {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(mantissaBits); // ULP diff for rounded integer value.
            /** @type {number} */ var eps = es3fShaderCommonFunctionTests.getEpsFromBits(1.0, mantissaBits); // epsilon for rounding bounds

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                /** @type {number} */ var minRes = Math.ceil(in0 - eps);
                /** @type {number} */ var maxRes = Math.ceil(in0 + eps);
                /** @type {boolean} */ var anyOk = false;

                for (var roundedVal = minRes; roundedVal <= maxRes; roundedVal++) {
                    ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, roundedVal);

                    if (ulpDiff <= maxUlpDiff) {
                        anyOk = true;
                        break;
                    }
                }

                if (!anyOk & deMath.deInRange32(0, minRes, maxRes)) {
                    ulpDiff = Math.abs(Math.floor(tcuFloat.newFloat32(out0).bits()) - 0x80000000);
                    anyOk = ulpDiff <= maxUlpDiff;
                }

                if (!anyOk) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = [' + minRes + ', ' + maxRes + '] with ULP threshold ' + maxUlpDiff;
                    return false;
                }
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {es3fShaderCommonFunctionTests.CommonFunctionCase}
     * @param {gluShaderUtil.DataType} baseType
     * @param {gluShaderUtil.precision} precision
     * @param {gluShaderProgram.shaderType} shaderType
     */
    es3fShaderCommonFunctionTests.FractCase = function(baseType, precision, shaderType) {
        es3fShaderCommonFunctionTests.CommonFunctionCase.call(this,
            es3fShaderCommonFunctionTests.getCommonFuncCaseName(baseType, precision, shaderType),
            'fract', shaderType);
        this.m_spec.inputs.push(new glsShaderExecUtil.Symbol('in0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.outputs.push(new glsShaderExecUtil.Symbol('out0', gluVarType.newTypeBasic(baseType, precision)));
        this.m_spec.source = 'out0 = fract(in0);';
    };

    es3fShaderCommonFunctionTests.FractCase.prototype = Object.create(es3fShaderCommonFunctionTests.CommonFunctionCase.prototype);
    es3fShaderCommonFunctionTests.FractCase.prototype.constructor = es3fShaderCommonFunctionTests.FractCase;

    /**
     * @param {number} numValues
     * @return {*}
     */
    es3fShaderCommonFunctionTests.FractCase.prototype.getInputValues = function(numValues) {
        /** @type {Array<Array<number>>} */ var ranges = [
            [-2.0, 2.0], // lowp
            [-1e3, 1e3], // mediump
            [-1e7, 1e7] // highp
        ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xac23f);

        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {number} */ var numSpecialCases = 0;

        /** @type {Array<Array<number>>} */ var values = [];
        values[0] = [];

        // Special cases.
        if (precision !== gluShaderUtil.precision.PRECISION_LOWP) {
            assertMsgOptions(numValues >= 10, 'Sample too small.', false, true);
            for (var ndx = 0; ndx < 10; ndx++) {
                /** @type {number} */ var v = deMath.clamp(ndx - 5.5, ranges[precision][0], ranges[precision][1]);
                for (var scalarNdx = 0; scalarNdx < scalarSize; scalarNdx++) {
                    values[0].push(v);
                }
                numSpecialCases += 1;
            }
        }

        // Random cases.
        values[0] = values[0].concat(es3fShaderCommonFunctionTests.fillRandomScalars(es3fShaderCommonFunctionTests.Types.FLOAT, rnd, ranges[precision][0], ranges[precision][1], (numValues - numSpecialCases) * scalarSize));

        // If precision is mediump, make sure values can be represented in fp16 exactly
        if (precision === gluShaderUtil.precision.PRECISION_MEDIUMP)
            es3fShaderCommonFunctionTests.vecToFloat16(values[0])

        return values;
    };

    /**
     * @param {*} inputs
     * @param {*} outputs
     * @return {boolean}
     */
    es3fShaderCommonFunctionTests.FractCase.prototype.compare = function(inputs, outputs) {
        /** @type {gluShaderUtil.DataType} */ var type = this.m_spec.inputs[0].varType.getBasicType();
        /** @type {gluShaderUtil.precision} */ var precision = this.m_spec.inputs[0].varType.getPrecision();
        /** @type {boolean} */ var hasZeroSign = es3fShaderCommonFunctionTests.supportsSignedZero(precision);
        /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {number} */ var in0;
        /** @type {number} */ var out0;
        /** @type {number} */ var ref;
        /** @type {number} */ var ulpDiff;

        if (precision === gluShaderUtil.precision.PRECISION_HIGHP || precision === gluShaderUtil.precision.PRECISION_MEDIUMP) {
            // Require exact result.
            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];
                ref = in0 - Math.floor(in0);
                ulpDiff = hasZeroSign ? es3fShaderCommonFunctionTests.getUlpDiff(out0, ref) : es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref);

                if (ulpDiff > 0) {
                    this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ', got ULP diff ' + ulpDiff;
                    return false;
                }
            }
        } else {
            /** @type {number} */ var mantissaBits = es3fShaderCommonFunctionTests.getMinMantissaBits(precision);
            /** @type {number} */ var eps = es3fShaderCommonFunctionTests.getEpsFromBits(1.0, mantissaBits); // epsilon for rounding bounds

            for (var compNdx = 0; compNdx < scalarSize; compNdx++) {
                in0 = inputs[0][compNdx];
                out0 = outputs[0][compNdx];

                if (Math.floor(in0 - eps) == Math.floor(in0 + eps)) {
                    ref = in0 - Math.floor(in0);
                    /** @type {number} */ var bitsLost = es3fShaderCommonFunctionTests.numBitsLostInOp(in0, ref);
                    /** @type {number} */ var maxUlpDiff = es3fShaderCommonFunctionTests.getMaxUlpDiffFromBits(Math.max(0, mantissaBits - bitsLost)); // ULP diff for rounded integer value.
                    ulpDiff = es3fShaderCommonFunctionTests.getUlpDiffIgnoreZeroSign(out0, ref);
                    if (ulpDiff > maxUlpDiff) {
                        this.m_failMsg += 'Expected [' + compNdx + '] = ' + ref + ' with ULP threshold ' + maxUlpDiff + ', got diff ' + ulpDiff;
                        return false;
                    }
                } else {
                    if (out0 >= 1.0) {
                        this.m_failMsg += 'Expected [' + compNdx + '] < 1.0';
                        return false;
                    }
                }
            }
        }

        return true;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fShaderCommonFunctionTests.ShaderCommonFunctionTests = function() {
        tcuTestCase.DeqpTest.call(this, 'common', 'Common function tests');
    };

    es3fShaderCommonFunctionTests.ShaderCommonFunctionTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderCommonFunctionTests.ShaderCommonFunctionTests.prototype.constructor = es3fShaderCommonFunctionTests.ShaderCommonFunctionTests;

    /**
     * @param {tcuTestCase.DeqpTest} parent
     * @param {es3fShaderCommonFunctionTests.TestClass} testClass
     * @param {string} functionName
     * @param {boolean} floatTypes
     * @param {boolean} intTypes
     * @param {boolean} uintTypes
     */
    es3fShaderCommonFunctionTests.addFunctionCases = function(parent, testClass, functionName, floatTypes, intTypes, uintTypes) {
        /** @type {tcuTestCase.DeqpTest} */ var group = tcuTestCase.newTest(functionName, functionName);
        parent.addChild(group);

        /** @type {Array<gluShaderUtil.DataType>} */ var scalarTypes = [
            gluShaderUtil.DataType.FLOAT,
            gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.UINT
        ];

        for (var scalarTypeNdx = 0; scalarTypeNdx < scalarTypes.length; scalarTypeNdx++) {
            /** @type {gluShaderUtil.DataType} */ var scalarType = scalarTypes[scalarTypeNdx];

            if ((!floatTypes && scalarType == gluShaderUtil.DataType.FLOAT) ||
                (!intTypes && scalarType == gluShaderUtil.DataType.INT) ||
                (!uintTypes && scalarType == gluShaderUtil.DataType.UINT))
                continue;

            for (var vecSize = 1; vecSize <= 4; vecSize++)
            for (var prec = gluShaderUtil.precision.PRECISION_LOWP; prec <= gluShaderUtil.precision.PRECISION_HIGHP; prec++)
            for (var shaderType = gluShaderProgram.shaderType.VERTEX; shaderType <= gluShaderProgram.shaderType.FRAGMENT; shaderType++)
                group.addChild(new testClass(/** @type {gluShaderUtil.DataType} */ (scalarType + vecSize - 1), prec, shaderType));
        }
    };

    es3fShaderCommonFunctionTests.ShaderCommonFunctionTests.prototype.init = function() {
        var testGroup = tcuTestCase.runner.testCases;

        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.AbsCase, 'abs', true, true, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.SignCase, 'sign', true, true, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.FloorCase, 'floor', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.TruncCase, 'trunc', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.RoundCase, 'round', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.RoundEvenCase, 'roundeven', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.CeilCase, 'ceil', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.FractCase, 'fract', true, false, false);
        // mod
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.ModfCase, 'modf', true, false, false);
        // min, max, clamp, mix, step, smoothstep
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.IsnanCase, 'isnan', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.IsinfCase, 'isinf', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.FloatBitsToIntCase, 'floatbitstoint', true, false, false);
        es3fShaderCommonFunctionTests.addFunctionCases(testGroup, es3fShaderCommonFunctionTests.FloatBitsToUintCase, 'floatbitstouint', true, false, false);

        // (u)intBitsToFloat()
        /** @type {tcuTestCase.DeqpTest} */ var intGroup = tcuTestCase.newTest('intbitstofloat', 'intBitsToFloat() Tests');
        /** @type {tcuTestCase.DeqpTest} */ var uintGroup = tcuTestCase.newTest('uintbitstofloat', 'uintBitsToFloat() Tests');

        testGroup.addChild(intGroup);
        testGroup.addChild(uintGroup);

        /** @type {Array<gluShaderProgram.shaderType>} */ var shaderTypes = [
            gluShaderProgram.shaderType.VERTEX,
            gluShaderProgram.shaderType.FRAGMENT
        ];

        for (var vecSize = 1; vecSize < 4; vecSize++) {
            /** @type {gluShaderUtil.DataType} */ var intType = vecSize > 1 ?
                gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.INT, vecSize) :
                gluShaderUtil.DataType.INT;

            /** @type {gluShaderUtil.DataType} */ var uintType = vecSize > 1 ?
                gluShaderUtil.getDataTypeVector(gluShaderUtil.DataType.UINT, vecSize) :
                gluShaderUtil.DataType.UINT;

            for (var shaderType in shaderTypes) {
                intGroup.addChild(new es3fShaderCommonFunctionTests.BitsToFloatCase(intType, shaderTypes[shaderType]));
                uintGroup.addChild(new es3fShaderCommonFunctionTests.BitsToFloatCase(uintType, shaderTypes[shaderType]));
            }
        }
    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fShaderCommonFunctionTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderCommonFunctionTests.ShaderCommonFunctionTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fShaderCommonFunctionTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

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
goog.provide('functional.gles3.es3fShaderDerivateTests');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluPixelTransfer');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.common.tcuInterval');
goog.require('framework.common.tcuFloat');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuStringTemplate');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.common.tcuTestCase');
goog.require('modules.shared.glsShaderRenderCase');

goog.scope(function() {
    var es3fShaderDerivateTests = functional.gles3.es3fShaderDerivateTests;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;
    var deString = framework.delibs.debase.deString;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var gluPixelTransfer = framework.opengl.gluPixelTransfer;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTexture = framework.opengl.gluTexture;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var tcuInterval = framework.common.tcuInterval;
    var tcuFloat = framework.common.tcuFloat;
    var tcuLogImage = framework.common.tcuLogImage;
    var tcuMatrix = framework.common.tcuMatrix;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuStringTemplate = framework.common.tcuStringTemplate;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var tcuTestCase = framework.common.tcuTestCase;
    var glsShaderRenderCase = modules.shared.glsShaderRenderCase;

    /** @const {number} */ es3fShaderDerivateTests.VIEWPORT_WIDTH = 167;
    /** @const {number} */ es3fShaderDerivateTests.VIEWPORT_HEIGHT = 103;
    /** @const {number} */ es3fShaderDerivateTests.FBO_WIDTH = 99;
    /** @const {number} */ es3fShaderDerivateTests.FBO_HEIGHT = 133;
    /** @const {number} */ es3fShaderDerivateTests.MAX_FAILED_MESSAGES = 10;
    /** @const {number} */ es3fShaderDerivateTests.INTERPOLATION_LOST_BITS = 3; // number mantissa of bits allowed to be lost in varying interpolation
    /**
     * @enum {number}
     */
    es3fShaderDerivateTests.DerivateFunc = {
        DFDX: 0,
        DFDY: 1,
        FWIDTH: 2
    };

    /**
     * @enum {number}
     */
    es3fShaderDerivateTests.SurfaceType = {
        DEFAULT_FRAMEBUFFER: 0,
        UNORM_FBO: 1,
        FLOAT_FBO: 2 // \note Uses RGBA32UI fbo actually, since FP rendertargets are not in core spec.
    };

    /**
     * @enum {number}
     */
    es3fShaderDerivateTests.VerificationLogging = {
        LOG_ALL: 0,
        LOG_NOTHING: 1
    };

    /**
     * @param {es3fShaderDerivateTests.DerivateFunc} func
     * @return {string}
     */
    es3fShaderDerivateTests.getDerivateFuncName = function(func) {
        switch (func) {
            case es3fShaderDerivateTests.DerivateFunc.DFDX: return 'dFdx';
            case es3fShaderDerivateTests.DerivateFunc.DFDY: return 'dFdy';
            case es3fShaderDerivateTests.DerivateFunc.FWIDTH: return 'fwidth';
            default: throw new Error('Derivate Func not supported.');
        }
    };

    /**
     * @param {es3fShaderDerivateTests.DerivateFunc} func
     * @return {string}
     */
    es3fShaderDerivateTests.getDerivateFuncCaseName = function(func) {
        switch (func) {
            case es3fShaderDerivateTests.DerivateFunc.DFDX: return 'dfdx';
            case es3fShaderDerivateTests.DerivateFunc.DFDY: return 'dfdy';
            case es3fShaderDerivateTests.DerivateFunc.FWIDTH: return 'fwidth';
            default: throw new Error('Derivate Func not supported.');
        }
    };

    /**
     * @param {?gluShaderUtil.DataType} type
     * @return {Array<boolean>}
     */
    es3fShaderDerivateTests.getDerivateMask = function(type) {
        switch (type) {
            case gluShaderUtil.DataType.FLOAT: return [true, false, false, false];
            case gluShaderUtil.DataType.FLOAT_VEC2: return [true, true, false, false];
            case gluShaderUtil.DataType.FLOAT_VEC3: return [true, true, true, false];
            case gluShaderUtil.DataType.FLOAT_VEC4: return [true, true, true, true];
            default: throw new Error('Data Type not supported.');
        }
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} surface
     * @param {Array<number>} derivScale
     * @param {Array<number>} derivBias
     * @param {number} x
     * @param {number} y
     * @return {Array<number>}
     */
    es3fShaderDerivateTests.readDerivate = function(surface, derivScale, derivBias, x, y) {
        return deMath.divide(deMath.subtract(surface.getPixel(x, y), derivBias), derivScale);
    };

    /**
     * @param {Array<number>} v
     * @return {Array<number>}
     */
    es3fShaderDerivateTests.getCompExpBits = function(v) {
        return [tcuFloat.newFloat32(v[0]).exponentBits(),
            tcuFloat.newFloat32(v[1]).exponentBits(),
            tcuFloat.newFloat32(v[2]).exponentBits(),
            tcuFloat.newFloat32(v[3]).exponentBits()];
    };

    /**
     * @param {number} value
     * @param {number} numAccurateBits
     * @return {number}
     */
    es3fShaderDerivateTests.computeFloatingPointError = function(value, numAccurateBits) {
        /** @type {number} */ var numGarbageBits = 23 - numAccurateBits;
        /** @type {number} */ var mask = (1 << numGarbageBits) - 1 ;
        /** @type {number} */ var exp = tcuFloat.newFloat32(value).exponent();

        return (new tcuFloat.deFloat()).construct(1, exp, (1 << 23) | mask).getValue() - (new tcuFloat.deFloat()).construct(1, exp, 1 << 23).getValue();
    };

    /**
       * @param {?gluShaderUtil.precision} precision
     * @return {number}
     */
    es3fShaderDerivateTests.getNumMantissaBits = function(precision) {
        switch (precision) {
            case gluShaderUtil.precision.PRECISION_HIGHP: return 23;
            case gluShaderUtil.precision.PRECISION_MEDIUMP: return 10;
            case gluShaderUtil.precision.PRECISION_LOWP: return 6;
            default:
                throw new Error('Precision not supported: ' + precision);
        }
    };

    /**
     * @param {?gluShaderUtil.precision} precision
     * @return {number}
     */
    es3fShaderDerivateTests.getMinExponent = function(precision) {
        switch (precision) {
            case gluShaderUtil.precision.PRECISION_HIGHP: return -126;
            case gluShaderUtil.precision.PRECISION_MEDIUMP: return -14;
            case gluShaderUtil.precision.PRECISION_LOWP: return -8;
            default:
                throw new Error('Precision not supported: ' + precision);
        }
    };

    /**
     * @param {number} exp
     * @param {number} numMantissaBits
     * @return {number}
     */
    es3fShaderDerivateTests.getSingleULPForExponent = function(exp, numMantissaBits) {
        if (numMantissaBits > 0) {
            assertMsgOptions(numMantissaBits <= 23, 'numMantissaBits must be less or equal than 23.', false, true);

            /** @type {number} */ var ulpBitNdx = 23 - numMantissaBits;

            return (new tcuFloat.deFloat()).construct(1, exp, (1 << 23) | (1 << ulpBitNdx)).getValue() - (new tcuFloat.deFloat()).construct(1, exp, 1 << 23).getValue();
        } else {
            assertMsgOptions(numMantissaBits === 0, 'numMantissaBits must equal to 0.', false, true);
            return (new tcuFloat.deFloat()).construct(1, exp, (1 << 23)).getValue()
        }
    };

    /**
     * @param {number} value
     * @param {number} numMantissaBits
     * @return {number}
     */
    es3fShaderDerivateTests.getSingleULPForValue = function(value, numMantissaBits) {
        /** @type {number} */ var exp = (new tcuFloat.deFloat().deFloatNumber(value)).exponent();
        return es3fShaderDerivateTests.getSingleULPForExponent(exp, numMantissaBits);
    };

    /**
     * @param {number} value
     * @param {number} minExponent
     * @param {number} numAccurateBits
     * @return {number}
     */
    es3fShaderDerivateTests.convertFloorFlushToZero = function(value, minExponent, numAccurateBits) {
        if (value === 0.0) {
            return 0.0;
        } else {
            /** @type {tcuFloat.deFloat} */ var inputFloat = new tcuFloat.deFloat().deFloatNumber(value);
            /** @type {number} */ var numTruncatedBits = 23 - numAccurateBits;
            /** @type {number} */ var truncMask = (1 << numTruncatedBits) - 1;

            if (value > 0.0) {
                if (value > 0.0 && (new tcuFloat.deFloat().deFloatNumber(value)).exponent() < minExponent) {
                    // flush to zero if possible
                    return 0.0;
                } else {
                    // just mask away non-representable bits
                    return (new tcuFloat.deFloat()).construct(1, inputFloat.exponent(), inputFloat.mantissa() & ~truncMask).getValue();
                }
            } else {
                if (inputFloat.mantissa() & truncMask) {
                    // decrement one ulp if truncated bits are non-zero (i.e. if value is not representable)
                    return (new tcuFloat.deFloat()).construct(-1, inputFloat.exponent(), inputFloat.mantissa() & ~truncMask).getValue() - es3fShaderDerivateTests.getSingleULPForExponent(inputFloat.exponent(), numAccurateBits);
                } else {
                    // value is representable, no need to do anything
                    return value;
                }
            }
        }
    };

    /**
     * @param {number} value
     * @param {number} minExponent
     * @param {number} numAccurateBits
     * @return {number}
     */
    es3fShaderDerivateTests.convertCeilFlushToZero = function(value, minExponent, numAccurateBits) {
        return -es3fShaderDerivateTests.convertFloorFlushToZero(-value, minExponent, numAccurateBits);
    };

    /**
     * @param {number} value
     * @param {number} numUlps
     * @param {number} numMantissaBits
     * @return {number}
     */
    es3fShaderDerivateTests.addErrorUlp = function(value, numUlps, numMantissaBits) {
        return value + numUlps * es3fShaderDerivateTests.getSingleULPForValue(value, numMantissaBits);
    };

    /**
     * @param {?gluShaderUtil.precision} precision
     * @param {Array<number>} valueMin
     * @param {Array<number>} valueMax
     * @param {Array<number>} expectedDerivate
     * @return {Array<number>}
     */
    es3fShaderDerivateTests.getDerivateThreshold = function(precision, valueMin, valueMax, expectedDerivate) {
        /** @type {number} */ var baseBits = es3fShaderDerivateTests.getNumMantissaBits(precision);
        /** @type {Array<number>} */ var derivExp = es3fShaderDerivateTests.getCompExpBits(expectedDerivate);
        /** @type {Array<number>} */ var maxValueExp = deMath.max(es3fShaderDerivateTests.getCompExpBits(valueMin), es3fShaderDerivateTests.getCompExpBits(valueMax));
        /** @type {Array<number>} */ var numBitsLost = deMath.subtract(maxValueExp, deMath.min(maxValueExp, derivExp));
        /** @type {Array<number>} */
        var numAccurateBits = deMath.max(
            deMath.addScalar(
                deMath.subtract(
                    [baseBits, baseBits, baseBits, baseBits],
                    numBitsLost),
                -es3fShaderDerivateTests.INTERPOLATION_LOST_BITS),
            [0, 0, 0, 0]);

        return [es3fShaderDerivateTests.computeFloatingPointError(expectedDerivate[0], numAccurateBits[0]),
                es3fShaderDerivateTests.computeFloatingPointError(expectedDerivate[1], numAccurateBits[1]),
                es3fShaderDerivateTests.computeFloatingPointError(expectedDerivate[2], numAccurateBits[2]),
                es3fShaderDerivateTests.computeFloatingPointError(expectedDerivate[3], numAccurateBits[3])];
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} result
     * @param {tcuTexture.PixelBufferAccess} errorMask
     * @param {?gluShaderUtil.DataType} dataType
     * @param {Array<number>} reference
     * @param {Array<number>} threshold
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     * @param {es3fShaderDerivateTests.VerificationLogging=} logPolicy
     * @return {boolean}
     */
    es3fShaderDerivateTests.verifyConstantDerivate = function(result, errorMask, dataType, reference, threshold, scale, bias, logPolicy) {
        logPolicy = logPolicy === undefined ? es3fShaderDerivateTests.VerificationLogging.LOG_ALL : logPolicy;
        /** @type {Array<boolean>} */ var mask = deMath.logicalNotBool(es3fShaderDerivateTests.getDerivateMask(dataType));
        /** @type {number} */ var numFailedPixels = 0;

        if (logPolicy === es3fShaderDerivateTests.VerificationLogging.LOG_ALL)
            bufferedLogToConsole('Expecting ' + reference + ' with threshold ' + threshold);

        for (var y = 0; y < result.getHeight(); y++) {
            for (var x = 0; x < result.getWidth(); x++) {
                /** @type {Array<number>} */ var resDerivate = es3fShaderDerivateTests.readDerivate(result, scale, bias, x, y);
                /** @type {boolean} */
                var isOk = deMath.boolAll(
                    deMath.logicalOrBool(
                        deMath.lessThanEqual(
                            deMath.abs(deMath.subtract(reference, resDerivate)),
                            threshold),
                        mask));

                if (!isOk) {
                    if (numFailedPixels < es3fShaderDerivateTests.MAX_FAILED_MESSAGES && logPolicy === es3fShaderDerivateTests.VerificationLogging.LOG_ALL)
                        bufferedLogToConsole('FAIL: got ' + resDerivate + ', diff = ' + deMath.abs(deMath.subtract(reference, resDerivate)) + ', at x = ' + x + ', y = ' + y);
                    numFailedPixels += 1;
                    errorMask.setPixel(tcuRGBA.RGBA.red.toVec(), x, y);
                }
            }
        }

        if (numFailedPixels >= es3fShaderDerivateTests.MAX_FAILED_MESSAGES && logPolicy === es3fShaderDerivateTests.VerificationLogging.LOG_ALL)
            bufferedLogToConsole('...');

        if (numFailedPixels > 0 && logPolicy === es3fShaderDerivateTests.VerificationLogging.LOG_ALL)
            bufferedLogToConsole('FAIL: found ' + numFailedPixels + ' failed pixels');

        return numFailedPixels === 0;
    };

    /**
     *      .-----.
     *      | s_x |
     *  M x | s_y |
     *      | 1.0 |
     *      '-----'
     * @struct
     * @constructor
     */
    es3fShaderDerivateTests.Linear2DFunctionEvaluator = function() {
        /** @type {tcuMatrix.Matrix} */ this.matrix = new tcuMatrix.Matrix(4, 3);
    };

    es3fShaderDerivateTests.Linear2DFunctionEvaluator.prototype.evaluateAt = function(screenX, screenY) {
        /** @type {Array<number>} */ var position = [screenX, screenY, 1.0];
        return tcuMatrix.multiplyMatVec(this.matrix, position);
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} result
     * @param {tcuTexture.PixelBufferAccess} errorMask
     * @param {?gluShaderUtil.DataType} dataType
     * @param {?gluShaderUtil.precision} precision
     * @param {Array<number>} derivScale
     * @param {Array<number>} derivBias
     * @param {Array<number>} surfaceThreshold
     * @param {es3fShaderDerivateTests.DerivateFunc} derivateFunc
     * @param {es3fShaderDerivateTests.Linear2DFunctionEvaluator} func
     * @return {boolean}
     */
    es3fShaderDerivateTests.reverifyConstantDerivateWithFlushRelaxations = function(result, errorMask, dataType, precision, derivScale, derivBias, surfaceThreshold, derivateFunc, func) {
        assertMsgOptions(result.getWidth() === errorMask.getWidth(), 'Dimensions of result and errorMask inconsistent.', false, true);
        assertMsgOptions(result.getHeight() === errorMask.getHeight(), 'Dimensions of result and errorMask inconsistent.', false, true);
        assertMsgOptions(derivateFunc === es3fShaderDerivateTests.DerivateFunc.DFDX || derivateFunc === es3fShaderDerivateTests.DerivateFunc.DFDY, 'Derivate Function should be DFDX or DFDY.', false, true);

        /** @type {Array<number>} */ var red = [255, 0, 0, 255];
        /** @type {Array<number>} */ var green = [0, 255, 0, 255];
        /** @type {number} */ var divisionErrorUlps = 2.5;

        /** @type {number} */ var numComponents = gluShaderUtil.getDataTypeScalarTypeAsDataType(dataType);
        /** @type {number} */ var numBits = es3fShaderDerivateTests.getNumMantissaBits(precision);
        /** @type {number} */ var minExponent = es3fShaderDerivateTests.getMinExponent(precision);

        /** @type {number} */ var numVaryingSampleBits = numBits - es3fShaderDerivateTests.INTERPOLATION_LOST_BITS;
        /** @type {number} */ var numFailedPixels = 0;

        errorMask.clear(green);

        // search for failed pixels
        for (var y = 0; y < result.getHeight(); ++y)
        for (var x = 0; x < result.getWidth(); ++x) {
            //                 flushToZero?(f2z?(functionValueCurrent) - f2z?(functionValueBefore))
            // flushToZero? ( ------------------------------------------------------------------------ +- 2.5 ULP )
            //                                                  dx

            /** @type {Array<number>} */ var resultDerivative = es3fShaderDerivateTests.readDerivate(result, derivScale, derivBias, x, y);

            // sample at the front of the back pixel and the back of the front pixel to cover the whole area of
            // legal sample positions. In general case this is NOT OK, but we know that the target funtion is
            // (mostly*) linear which allows us to take the sample points at arbitrary points. This gets us the
            // maximum difference possible in exponents which are used in error bound calculations.
            // * non-linearity may happen around zero or with very high function values due to subnorms not
            //   behaving well.
            /** @type {Array<number>} */ var functionValueForward = (derivateFunc === es3fShaderDerivateTests.DerivateFunc.DFDX) ?
                                                        (func.evaluateAt(x + 2.0, y + 0.5)) :
                                                        (func.evaluateAt(x + 0.5, y + 2.0));
            /** @type {Array<number>} */ var functionValueBackward = (derivateFunc === es3fShaderDerivateTests.DerivateFunc.DFDX) ?
                                                        (func.evaluateAt(x - 1.0, y + 0.5)) :
                                                        (func.evaluateAt(x + 0.5, y - 1.0));

            /** @type {boolean} */ var anyComponentFailed = false;

            // check components separately
            for (var c = 0; c < numComponents; ++c) {
                // interpolation value range
                /** @type {tcuInterval.Interval} */ var forwardComponent = tcuInterval.withIntervals(
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertFloorFlushToZero(
                        es3fShaderDerivateTests.addErrorUlp(functionValueForward[c], -0.5, numVaryingSampleBits), minExponent, numBits)),
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertCeilFlushToZero(
                        es3fShaderDerivateTests.addErrorUlp(functionValueForward[c], +0.5, numVaryingSampleBits), minExponent, numBits))
                );

                /** @type {tcuInterval.Interval} */ var backwardComponent = tcuInterval.withIntervals(
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertFloorFlushToZero(
                        es3fShaderDerivateTests.addErrorUlp(functionValueBackward[c], -0.5, numVaryingSampleBits), minExponent, numBits)),
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertCeilFlushToZero(
                        es3fShaderDerivateTests.addErrorUlp(functionValueBackward[c], +0.5, numVaryingSampleBits), minExponent, numBits))
                );

                /** @type {number} */
                var maxValueExp = Math.max(
                        (new tcuFloat.deFloat().deFloatNumber(forwardComponent.lo())).exponent(),
                        (new tcuFloat.deFloat().deFloatNumber(forwardComponent.hi())).exponent(),
                        (new tcuFloat.deFloat().deFloatNumber(backwardComponent.lo())).exponent(),
                        (new tcuFloat.deFloat().deFloatNumber(backwardComponent.hi())).exponent());

                // subtraction in nominator will likely cause a cancellation of the most
                // significant bits. Apply error bounds.
                /** @type {tcuInterval.Interval} */ var nominator = tcuInterval.Interval.operatorSub(forwardComponent, backwardComponent);
                /** @type {number} */ var nominatorLoExp = (new tcuFloat.deFloat().deFloatNumber(nominator.lo())).exponent();
                /** @type {number} */ var nominatorHiExp = (new tcuFloat.deFloat().deFloatNumber(nominator.hi())).exponent();
                /** @type {number} */ var nominatorLoBitsLost = maxValueExp - nominatorLoExp;
                /** @type {number} */ var nominatorHiBitsLost = maxValueExp - nominatorHiExp;
                /** @type {number} */ var nominatorLoBits = Math.max(0, numBits - nominatorLoBitsLost);
                /** @type {number} */ var nominatorHiBits = Math.max(0, numBits - nominatorHiBitsLost);

                /** @type {tcuInterval.Interval} */ var nominatorRange = tcuInterval.withIntervals(
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertFloorFlushToZero(nominator.lo(), minExponent, nominatorLoBits)),
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertCeilFlushToZero(nominator.hi(), minExponent, nominatorHiBits)));
                //
                /** @type {tcuInterval.Interval} */ var divisionRange = tcuInterval.Interval.operatorDiv(nominatorRange, new tcuInterval.Interval(3.0)); // legal sample area is anywhere within this and neighboring pixels (i.e. size = 3)
                /** @type {tcuInterval.Interval} */ var divisionResultRange = tcuInterval.withIntervals(
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertFloorFlushToZero(es3fShaderDerivateTests.addErrorUlp(divisionRange.lo(), -divisionErrorUlps, numBits), minExponent, numBits)),
                    new tcuInterval.Interval(es3fShaderDerivateTests.convertCeilFlushToZero(es3fShaderDerivateTests.addErrorUlp(divisionRange.hi(), divisionErrorUlps, numBits), minExponent, numBits)));
                /** @type {tcuInterval.Interval} */ var finalResultRange = tcuInterval.withIntervals(
                    new tcuInterval.Interval(divisionResultRange.lo() - surfaceThreshold[c]),
                    new tcuInterval.Interval(divisionResultRange.hi() + surfaceThreshold[c]));

                if (resultDerivative[c] >= finalResultRange.lo() && resultDerivative[c] <= finalResultRange.hi()) {
                    // value ok
                } else {
                    if (numFailedPixels < es3fShaderDerivateTests.MAX_FAILED_MESSAGES)
                        bufferedLogToConsole('Error in pixel at ' + x + ', ' + y + ' with component ' + c + ' (channel ' + ('rgba'[c]) + ')\n' +
                            '\tGot pixel value ' + result.getPixelInt(x, y) + '\n' +
                            '\t\tdFd' + ((derivateFunc === es3fShaderDerivateTests.DerivateFunc.DFDX) ? 'x' : 'y') + ' ~= ' + resultDerivative[c] + '\n' +
                            '\t\tdifference to a valid range: ' +
                            ((resultDerivative[c] < finalResultRange.lo()) ? '-' : '+') +
                            ((resultDerivative[c] < finalResultRange.lo()) ? (finalResultRange.lo() - resultDerivative[c]) : (resultDerivative[c] - finalResultRange.hi())) +
                            '\n' +
                            '\tDerivative value range:\n' +
                            '\t\tMin: ' + finalResultRange.lo() + '\n' +
                            '\t\tMax: ' + finalResultRange.hi() + '\n');

                    ++numFailedPixels;
                    anyComponentFailed = true;
                }
            }

            if (anyComponentFailed)
                errorMask.setPixel(red, x, y);
        }

        if (numFailedPixels >= es3fShaderDerivateTests.MAX_FAILED_MESSAGES)
            bufferedLogToConsole('...');

        if (numFailedPixels > 0)
            bufferedLogToConsole('FAIL: found ' + numFailedPixels + ' failed pixels');

        return numFailedPixels === 0;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    es3fShaderDerivateTests.TriangleDerivateCase = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {?gluShaderUtil.DataType} */ this.m_dataType = null;
        /** @type {?gluShaderUtil.precision} */ this.m_precision = null;

        /** @type {?gluShaderUtil.DataType} */ this.m_coordDataType = null;
        /** @type {?gluShaderUtil.precision} */ this.m_coordPrecision = null;

        /** @type {string} */ this.m_fragmentSrc;

        /** @type {Array<number>} */ this.m_coordMin = [];
        /** @type {Array<number>} */ this.m_coordMax = [];
        /** @type {Array<number>} */ this.m_derivScale = [];
        /** @type {Array<number>} */ this.m_derivBias = [];

        /** @type {es3fShaderDerivateTests.SurfaceType} */ this.m_surfaceType = es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER;
        /** @type {number} */ this.m_numSamples = 0;
        /** @type {number} */ this.m_hint = gl.DONT_CARE;

        assertMsgOptions(this.m_surfaceType !== es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER || this.m_numSamples === 0, 'Did not expect surfaceType = DEFAULT_FRAMEBUFFER or numSamples = 0', false, true);
    };

    es3fShaderDerivateTests.TriangleDerivateCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderDerivateTests.TriangleDerivateCase.prototype.constructor = es3fShaderDerivateTests.TriangleDerivateCase;

    es3fShaderDerivateTests.TriangleDerivateCase.prototype.deinit = function() {};

    /** @param {WebGLProgram} program */
    es3fShaderDerivateTests.TriangleDerivateCase.prototype.setupRenderState = function(program) {};

    /**
     * @param {?gluShaderUtil.DataType} coordType
     * @param {?gluShaderUtil.precision} precision
     * @return {string}
     */
    es3fShaderDerivateTests.genVertexSource = function(coordType, precision) {
        assertMsgOptions(gluShaderUtil.isDataTypeFloatOrVec(coordType), 'Coord Type not supported', false, true);

        /** @type {string} */ var vertexTmpl = '' +
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in ${PRECISION} ${DATATYPE} a_coord;\n' +
            'out ${PRECISION} ${DATATYPE} v_coord;\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            ' v_coord = a_coord;\n' +
            '}\n';

        /** @type {Object} */ var vertexParams = {};

        vertexParams['PRECISION'] = gluShaderUtil.getPrecisionName(precision);
        vertexParams['DATATYPE'] = gluShaderUtil.getDataTypeName(coordType);

        return tcuStringTemplate.specialize(vertexTmpl, vertexParams);
    };

    /**
     * @return {Array<number>}
     */
    es3fShaderDerivateTests.TriangleDerivateCase.prototype.getViewportSize = function() {
        if (this.m_surfaceType === es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER) {
            /** @type {number} */ var width = Math.min(gl.drawingBufferWidth, es3fShaderDerivateTests.VIEWPORT_WIDTH);
            /** @type {number} */ var height = Math.min(gl.drawingBufferHeight, es3fShaderDerivateTests.VIEWPORT_HEIGHT);
            return [width, height];
        } else
            return [es3fShaderDerivateTests.FBO_WIDTH, es3fShaderDerivateTests.FBO_HEIGHT];
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fShaderDerivateTests.TriangleDerivateCase.prototype.iterate = function() {
        /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(es3fShaderDerivateTests.genVertexSource(this.m_coordDataType, this.m_coordPrecision), this.m_fragmentSrc));
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ 0xbbc24);
        /** @type {boolean} */ var useFbo = this.m_surfaceType != es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER;
        /** @type {number} */ var fboFormat = this.m_surfaceType === es3fShaderDerivateTests.SurfaceType.FLOAT_FBO ? gl.RGBA32UI : gl.RGBA8;
        /** @type {Array<number>} */ var viewportSize = this.getViewportSize();
        /** @type {number} */ var viewportX = useFbo ? 0 : rnd.getInt(0, gl.drawingBufferWidth - viewportSize[0]);
        /** @type {number} */ var viewportY = useFbo ? 0 : rnd.getInt(0, gl.drawingBufferHeight - viewportSize[1]);
        /** @type {?WebGLFramebuffer} */ var fbo = null;
        /** @type {?WebGLRenderbuffer} */ var rbo = null;
        /** @type {tcuTexture.TextureLevel} */ var result = null;

        bufferedLogToConsole(program.getProgramInfo().infoLog);

        if (!program.isOk())
            assertMsgOptions(false, 'Compile failed', false, true);

        if (useFbo) {
            bufferedLogToConsole('Rendering to FBO, format = ' + wtu.glEnumToString(gl, fboFormat) + ', samples = ' + this.m_numSamples);

            fbo = gl.createFramebuffer();
            rbo = gl.createRenderbuffer();

            gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
            gl.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, fboFormat, viewportSize[0], viewportSize[1]);
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo);
        } else {
            /** @type {tcuPixelFormat.PixelFormat} */ var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);

            bufferedLogToConsole('Rendering to default framebuffer\n' +
                '\tColor depth: R=' + pixelFormat.redBits + ', G=' + pixelFormat.greenBits + ', B=' + pixelFormat.blueBits + ', A=' + pixelFormat.alphaBits);
        }

        bufferedLogToConsole('in: ' + this.m_coordMin + ' ' + this.m_coordMax + '\n' +
            'v_coord.x = in.x * x\n' +
            'v_coord.y = in.y * y\n' +
            'v_coord.z = in.z * (x+y)/2\n' +
            'v_coord.w = in.w * (1 - (x+y)/2)\n' +
            '\n' +
            'u_scale: ' + this.m_derivScale + ', u_bias: ' + this.m_derivBias + ' (displayed values have scale/bias removed)' +
            'Viewport: ' + viewportSize[0] + 'x' + viewportSize[1] +
            'gl.FRAGMENT_SHADER_DERIVATE_HINT: ' + wtu.glEnumToString(gl, this.m_hint));
        // Draw
        /** @type {Array<number>} */ var positions = [
            -1.0, -1.0, 0.0, 1.0,
            -1.0, 1.0, 0.0, 1.0,
            1.0, -1.0, 0.0, 1.0,
            1.0, 1.0, 0.0, 1.0
        ];

        /** @type {Array<number>} */ var coords =[
            this.m_coordMin[0], this.m_coordMin[1], this.m_coordMin[2], this.m_coordMax[3],
            this.m_coordMin[0], this.m_coordMax[1], (this.m_coordMin[2] + this.m_coordMax[2]) * 0.5, (this.m_coordMin[3]+this.m_coordMax[3]) * 0.5,
            this.m_coordMax[0], this.m_coordMin[1], (this.m_coordMin[2] + this.m_coordMax[2]) * 0.5, (this.m_coordMin[3]+this.m_coordMax[3]) * 0.5,
            this.m_coordMax[0], this.m_coordMax[1], this.m_coordMax[2], this.m_coordMin[3]
        ];

        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays = [
            gluDrawUtil.newFloatVertexArrayBinding('a_position', 4, 4, 0, positions),
            gluDrawUtil.newFloatVertexArrayBinding('a_coord', 4, 4, 0, coords)
        ];

        /** @type {Array<number>} */ var indices = [0, 2, 1, 2, 3, 1];

        gl.clearColor(0.125, 0.25, 0.5, 1.0);
        // We can't really call clear() on gl.COLOR_BUFFER_BIT here as in c++ deqp.
        // The fbo format might be of integer type and WebGL2 requires an INVALID_OPERATION to be generated.
        var formatObj = gluTextureUtil.mapGLInternalFormat(fboFormat);
        var fmtClass = tcuTexture.getTextureChannelClass(formatObj.type);
        switch (fmtClass) {
            case tcuTexture.TextureChannelClass.FLOATING_POINT:
            case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
            case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
                gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
                break;
            case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
                gl.clearBufferuiv(gl.COLOR, 0, new Uint32Array([31, 63, 127, 255]));
                gl.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
                break;
            case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                gl.clearBufferiv(gl.COLOR, 0, new Int32Array([31, 63, 127, 255]));
                gl.clear(gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
                break;
            default:
                throw new Error('Invalid channelclass ' + fmtClass);
        }
        gl.disable(gl.DITHER);

        gl.useProgram(program.getProgram());

        /** @type {WebGLUniformLocation} */ var scaleLoc = gl.getUniformLocation(program.getProgram(), 'u_scale');
        /** @type {WebGLUniformLocation} */ var biasLoc = gl.getUniformLocation(program.getProgram(), 'u_bias');

        switch (this.m_dataType) {
            case gluShaderUtil.DataType.FLOAT:
                gl.uniform1f(scaleLoc, this.m_derivScale[0]);
                gl.uniform1f(biasLoc, this.m_derivBias[0]);
                break;

            case gluShaderUtil.DataType.FLOAT_VEC2:
                gl.uniform2fv(scaleLoc, this.m_derivScale.slice(0,2));
                gl.uniform2fv(biasLoc, this.m_derivBias.slice(0,2));
                break;

            case gluShaderUtil.DataType.FLOAT_VEC3:
                gl.uniform3fv(scaleLoc, this.m_derivScale.slice(0,3));
                gl.uniform3fv(biasLoc, this.m_derivBias.slice(0,3));
                break;

            case gluShaderUtil.DataType.FLOAT_VEC4:
                gl.uniform4fv(scaleLoc, this.m_derivScale);
                gl.uniform4fv(biasLoc, this.m_derivBias);
                break;

            default:
                throw new Error('Data Type not supported: ' + this.m_dataType);
        }

        glsShaderRenderCase.setupDefaultUniforms(program.getProgram());
        this.setupRenderState(program.getProgram());

        gl.hint(gl.FRAGMENT_SHADER_DERIVATIVE_HINT, this.m_hint);

        gl.viewport(viewportX, viewportY, viewportSize[0], viewportSize[1]);
        gluDrawUtil.draw(gl, program.getProgram(), vertexArrays, gluDrawUtil.triangles(indices));

        // Read back results

        /** @type {boolean} */ var isMSAA = useFbo && this.m_numSamples > 0;
        /** @type {?WebGLFramebuffer} */ var resFbo = null;
        /** @type {?WebGLRenderbuffer} */ var resRbo = null;

        // Resolve if necessary
        if (isMSAA) {
            resFbo = gl.createFramebuffer();
            resRbo = gl.createRenderbuffer();

            gl.bindRenderbuffer(gl.RENDERBUFFER, resRbo);
            gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 0, fboFormat, viewportSize[0], viewportSize[1]);
            gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, resFbo);
            gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, resRbo);

            gl.blitFramebuffer(0, 0, viewportSize[0], viewportSize[1], 0, 0, viewportSize[0], viewportSize[1], gl.COLOR_BUFFER_BIT, gl.NEAREST);

            gl.bindFramebuffer(gl.READ_FRAMEBUFFER, resFbo);
        }
        switch (this.m_surfaceType) {
            case es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER:
            case es3fShaderDerivateTests.SurfaceType.UNORM_FBO:
                var dataFormat = new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);
                result = new tcuTexture.TextureLevel(dataFormat, viewportSize[0], viewportSize[1]);
                gluPixelTransfer.readPixels(gl, viewportX, viewportY, dataFormat, result);
                break;

            case es3fShaderDerivateTests.SurfaceType.FLOAT_FBO:
                var dataFormat = new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.FLOAT);
                var transferFormat = new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT32);
                result = new tcuTexture.TextureLevel(dataFormat, viewportSize[0], viewportSize[1]);
                gluPixelTransfer.readPixels(gl, viewportX, viewportY, transferFormat, result);
                break;

            default:
                throw new Error('Surface Type not supported: ' + this.m_surfaceType);
        }

        // Verify
        /** @type {tcuSurface.Surface} */
        var errorMask = new tcuSurface.Surface(result.getWidth(), result.getHeight());

        errorMask.getAccess().clear(tcuRGBA.RGBA.green.toVec());

        /** @type {boolean} */ var isOk = this.verify(result.getAccess(), errorMask.getAccess());

        if (!isOk) {
            tcuLogImage.logImage('Rendered', 'Rendered image', result.getAccess());
            tcuLogImage.logImage('ErrorMask', 'Error mask', errorMask.getAccess());
            testFailedOptions('Fail', false);
        } else
            testPassedOptions('Pass', true);

        // Cleaning up buffers
        gl.deleteFramebuffer(fbo);
        gl.deleteRenderbuffer(rbo);
        gl.deleteFramebuffer(resFbo);
        gl.deleteRenderbuffer(resRbo);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @return {Array<number>}
     */
    es3fShaderDerivateTests.TriangleDerivateCase.prototype.getSurfaceThreshold = function() {
        switch (this.m_surfaceType) {
            case es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER:
                /** @type {tcuPixelFormat.PixelFormat} */ var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
                /** @type {Array<number>} */ var channelBits = [pixelFormat.redBits, pixelFormat.greenBits, pixelFormat.blueBits, pixelFormat.alphaBits];
                /** @type {Array<number>} */ var intThreshold = deMath.arrayShiftLeft([1, 1, 1, 1], deMath.subtract([8, 8, 8, 8], channelBits));
                /** @type {Array<number>} */ var normThreshold = deMath.scale(intThreshold, 1.0/255.0);

                return normThreshold;

            case es3fShaderDerivateTests.SurfaceType.UNORM_FBO: return deMath.scale([1, 1, 1, 1], 1.0/255.0);
            case es3fShaderDerivateTests.SurfaceType.FLOAT_FBO: return [0.0, 0.0, 0.0, 0.0];
            default:
                assertMsgOptions(false, 'Surface Type not supported. Falling back to default retun value [0.0, 0.0, 0.0, 0.0]', false, false);
                return [0.0, 0.0, 0.0, 0.0];
        }
    };

    /**
     * @constructor
     * @extends {es3fShaderDerivateTests.TriangleDerivateCase}
     * @param {string} name
     * @param {string} description
     * @param {es3fShaderDerivateTests.DerivateFunc} func
     * @param {gluShaderUtil.DataType} type
     */
    es3fShaderDerivateTests.ConstantDerivateCase = function(name, description, func, type) {
        es3fShaderDerivateTests.TriangleDerivateCase.call(this, name, description);
        /** @type {es3fShaderDerivateTests.DerivateFunc} */ this.m_func = func;
        this.m_dataType = type;
        this.m_precision = gluShaderUtil.precision.PRECISION_HIGHP;
        this.m_coordDataType = this.m_dataType;
        this.m_coordPrecision = this.m_precision;
    };

    es3fShaderDerivateTests.ConstantDerivateCase.prototype = Object.create(es3fShaderDerivateTests.TriangleDerivateCase.prototype);
    es3fShaderDerivateTests.ConstantDerivateCase.prototype.constructor = es3fShaderDerivateTests.ConstantDerivateCase;

    es3fShaderDerivateTests.ConstantDerivateCase.prototype.init = function() {
        /** @type {string} */ var fragmentTmpl = '' +
            '#version 300 es\n' +
            'layout(location = 0) out mediump vec4 o_color;\n' +
            'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
            'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
            'void main (void)\n' +
            '{\n' +
            ' ${PRECISION} ${DATATYPE} res = ${FUNC}(${VALUE}) * u_scale + u_bias;\n' +
            ' o_color = ${CAST_TO_OUTPUT};\n' +
            '}\n';

        /** @type {Object} */ var fragmentParams = {};
        fragmentParams['PRECISION'] = gluShaderUtil.getPrecisionName(this.m_precision);
        fragmentParams['DATATYPE'] = gluShaderUtil.getDataTypeName(this.m_dataType);
        fragmentParams['FUNC'] = es3fShaderDerivateTests.getDerivateFuncName(this.m_func);
        fragmentParams['VALUE'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? 'vec4(1.0, 7.2, -1e5, 0.0)' :
            this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? 'vec3(1e2, 8.0, 0.01)' :
            this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? 'vec2(-0.0, 2.7)' :
            '7.7';
        fragmentParams['CAST_TO_OUTPUT'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? 'res' :
            this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? 'vec4(res, 1.0)' :
            this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? 'vec4(res, 0.0, 1.0)' :
            'vec4(res, 0.0, 0.0, 1.0)';

        this.m_fragmentSrc = tcuStringTemplate.specialize(fragmentTmpl, fragmentParams);

        this.m_derivScale = [1e3, 1e3, 1e3, 1e3];
        this.m_derivBias = [0.5, 0.5, 0.5, 0.5];
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} result
     * @param {tcuTexture.PixelBufferAccess} errorMask
     * @return {boolean}
     */
    es3fShaderDerivateTests.ConstantDerivateCase.prototype.verify = function(result, errorMask) {
        /** @type {Array<number>} */ var reference = [0.0, 0.0, 0.0, 0.0]; // Derivate of constant argument should always be 0
        /** @type {Array<number>} */ var threshold = deMath.divide(this.getSurfaceThreshold(), deMath.abs(this.m_derivScale));
        return es3fShaderDerivateTests.verifyConstantDerivate(result, errorMask, this.m_dataType,
            reference, threshold, this.m_derivScale, this.m_derivBias);
    };

    /**
     * @constructor
     * @extends {es3fShaderDerivateTests.TriangleDerivateCase}
     * @param {string} name
     * @param {string} description
     * @param {es3fShaderDerivateTests.DerivateFunc} func
     * @param {gluShaderUtil.DataType} type
     * @param {gluShaderUtil.precision} precision
     * @param {number} hint
     * @param {es3fShaderDerivateTests.SurfaceType} surfaceType
     * @param {number} numSamples
     * @param {string} fragmentSrcTmpl
     */
    es3fShaderDerivateTests.LinearDerivateCase = function(name, description, func, type, precision, hint, surfaceType, numSamples, fragmentSrcTmpl) {
        es3fShaderDerivateTests.TriangleDerivateCase.call(this, name, description);
        /** @type {es3fShaderDerivateTests.DerivateFunc} */ this.m_func = func;
        /** @type {string} */ this.m_fragmentTmpl = fragmentSrcTmpl;
        this.m_dataType = type;
        this.m_precision = precision;
        this.m_coordDataType = this.m_dataType;
        this.m_coordPrecision = this.m_precision;
        this.m_hint = hint;
        this.m_surfaceType = surfaceType;
        this.m_numSamples = numSamples;
    };

    es3fShaderDerivateTests.LinearDerivateCase.prototype = Object.create(es3fShaderDerivateTests.TriangleDerivateCase.prototype);
    es3fShaderDerivateTests.LinearDerivateCase.prototype.constructor = es3fShaderDerivateTests.LinearDerivateCase;

    es3fShaderDerivateTests.LinearDerivateCase.prototype.init = function() {
        /** @type {Array<number>} */ var viewportSize = this.getViewportSize();
        /** @type {number} */ var w = viewportSize[0];
        /** @type {number} */ var h = viewportSize[1];
        /** @type {boolean} */ var packToInt = this.m_surfaceType === es3fShaderDerivateTests.SurfaceType.FLOAT_FBO;

        /** @type {Object} */ var fragmentParams = {};
        fragmentParams['OUTPUT_TYPE'] = gluShaderUtil.getDataTypeName(packToInt ? gluShaderUtil.DataType.UINT_VEC4 : gluShaderUtil.DataType.FLOAT_VEC4);
        fragmentParams['OUTPUT_PREC'] = gluShaderUtil.getPrecisionName(packToInt ? gluShaderUtil.precision.PRECISION_HIGHP : this.m_precision);
        fragmentParams['PRECISION'] = gluShaderUtil.getPrecisionName(this.m_precision);
        fragmentParams['DATATYPE'] = gluShaderUtil.getDataTypeName(this.m_dataType);
        fragmentParams['FUNC'] = es3fShaderDerivateTests.getDerivateFuncName(this.m_func);

        if (packToInt) {
            fragmentParams['CAST_TO_OUTPUT'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? 'floatBitsToUint(res)' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? 'floatBitsToUint(vec4(res, 1.0))' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? 'floatBitsToUint(vec4(res, 0.0, 1.0))' :
                'floatBitsToUint(vec4(res, 0.0, 0.0, 1.0))';
        } else {
            fragmentParams['CAST_TO_OUTPUT'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? 'res' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? 'vec4(res, 1.0)' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? 'vec4(res, 0.0, 1.0)' :
                'vec4(res, 0.0, 0.0, 1.0)';
        }

        this.m_fragmentSrc = tcuStringTemplate.specialize(this.m_fragmentTmpl, fragmentParams);

        switch (this.m_precision) {
            case gluShaderUtil.precision.PRECISION_HIGHP:
                this.m_coordMin = [-97., 0.2, 71., 74.];
                this.m_coordMax = [-13.2, -77., 44., 76.];
                break;

            case gluShaderUtil.precision.PRECISION_MEDIUMP:
                this.m_coordMin = [-37.0, 47., -7., 0.0];
                this.m_coordMax = [-1.0, 12., 7., 19.];
                break;

            case gluShaderUtil.precision.PRECISION_LOWP:
                this.m_coordMin = [0.0, -1.0, 0.0, 1.0];
                this.m_coordMax = [1.0, 1.0, -1.0, -1.0];
                break;

            default:
                throw new Error('Precision not supported: ' + this.m_precision);
        }

        if (this.m_surfaceType === es3fShaderDerivateTests.SurfaceType.FLOAT_FBO) {
            // No scale or bias used for accuracy.
            this.m_derivScale = [1.0, 1.0, 1.0, 1.0];
            this.m_derivBias = [0.0, 0.0, 0.0, 0.0];
        } else {
            // Compute scale - bias that normalizes to 0..1 range.
            /** @type {Array<number>} */ var dx = deMath.divide(deMath.subtract(this.m_coordMax, this.m_coordMin), [w, w, w * 0.5, -w * 0.5]);
            /** @type {Array<number>} */ var dy = deMath.divide(deMath.subtract(this.m_coordMax, this.m_coordMin), [h, h, h * 0.5, -h * 0.5]);

            switch (this.m_func) {
                case es3fShaderDerivateTests.DerivateFunc.DFDX:
                    this.m_derivScale = deMath.divide([0.5, 0.5, 0.5, 0.5], dx);
                    break;

                case es3fShaderDerivateTests.DerivateFunc.DFDY:
                    this.m_derivScale = deMath.divide([0.5, 0.5, 0.5, 0.5], dy);
                    break;

                case es3fShaderDerivateTests.DerivateFunc.FWIDTH:
                    this.m_derivScale = deMath.divide([0.5, 0.5, 0.5, 0.5], deMath.add(deMath.abs(dx), deMath.abs(dy)));
                    break;

                default:
                    throw new Error('Derivate Function not supported: ' + this.m_func);
            }

            this.m_derivBias = [0.0, 0.0, 0.0, 0.0];
        }
    };

    /**
     * @param {tcuTexture.ConstPixelBufferAccess} result
     * @param {tcuTexture.PixelBufferAccess} errorMask
     * @return {boolean}
     */
    es3fShaderDerivateTests.LinearDerivateCase.prototype.verify = function(result, errorMask) {
        /** @type {Array<number>} */ var xScale = [1.0, 0.0, 0.5, -0.5];
        /** @type {Array<number>} */ var yScale = [0.0, 1.0, 0.5, -0.5];
        /** @type {Array<number>} */ var surfaceThreshold = deMath.divide(this.getSurfaceThreshold(), deMath.abs(this.m_derivScale));

        /** @type {number} */ var w;
        /** @type {number} */ var h;
        /** @type {Array<number>} */ var reference;
        /** @type {Array<number>} */ var threshold;

        if (this.m_func === es3fShaderDerivateTests.DerivateFunc.DFDX || this.m_func === es3fShaderDerivateTests.DerivateFunc.DFDY) {
            /** @type {boolean} */ var isX = this.m_func === es3fShaderDerivateTests.DerivateFunc.DFDX;
            /** @type {number} */ var div = isX ? result.getWidth() : result.getHeight();
            /** @type {Array<number>} */ var scale = isX ? xScale : yScale;
            reference = deMath.multiply(deMath.scale(deMath.subtract(this.m_coordMax, this.m_coordMin), 1/div), scale);
            /** @type {Array<number>} */ var opThreshold = es3fShaderDerivateTests.getDerivateThreshold(this.m_precision, deMath.multiply(this.m_coordMin, scale), deMath.multiply(this.m_coordMax, scale), reference);
            threshold = deMath.max(surfaceThreshold, opThreshold);
            bufferedLogToConsole('Verifying result image.\n' +
                '\tValid derivative is ' + reference + ' with threshold ' + threshold);

            // short circuit if result is strictly within the normal value error bounds.
            // This improves performance significantly.
            if (es3fShaderDerivateTests.verifyConstantDerivate(result, errorMask,
                this.m_dataType, reference, threshold, this.m_derivScale,
                this.m_derivBias, es3fShaderDerivateTests.VerificationLogging.LOG_NOTHING)) {
                bufferedLogToConsole('No incorrect derivatives found, result valid.');
                return true;
            }

            // some pixels exceed error bounds calculated for normal values. Verify that these
            // potentially invalid pixels are in fact valid due to (for example) subnorm flushing.

            bufferedLogToConsole('Initial verification failed, verifying image by calculating accurate error bounds for each result pixel.\n' +
                '\tVerifying each result derivative is within its range of legal result values.');

            /** @type {Array<number>} */ var viewportSize = this.getViewportSize();
            /** @type {Array<number>} */ var valueRamp = deMath.subtract(this.m_coordMax, this.m_coordMin);
            /** @type {es3fShaderDerivateTests.Linear2DFunctionEvaluator} */ var function_ = new es3fShaderDerivateTests.Linear2DFunctionEvaluator();
            w = viewportSize[0];
            h = viewportSize[1];

            function_.matrix.setRow(0, [valueRamp[0] / w, 0.0, this.m_coordMin[0]]);
            function_.matrix.setRow(1, [0.0, valueRamp[1] / h, this.m_coordMin[1]]);
            function_.matrix.setRow(2, deMath.scale([valueRamp[2] / w, valueRamp[2] / h, this.m_coordMin[2] + this.m_coordMin[2]], 1 / 2.0));
            function_.matrix.setRow(3, deMath.scale([-valueRamp[3] / w, -valueRamp[3] / h, this.m_coordMax[3] + this.m_coordMax[3]], 1 / 2.0));

            return es3fShaderDerivateTests.reverifyConstantDerivateWithFlushRelaxations(
                result, errorMask, this.m_dataType, this.m_precision, this.m_derivScale,
                this.m_derivBias, surfaceThreshold, this.m_func, function_);
        } else {
            assertMsgOptions(this.m_func === es3fShaderDerivateTests.DerivateFunc.FWIDTH, 'Expected DerivateFunc.FWIDTH', false, true);
            w = result.getWidth();
            h = result.getHeight();

            /** @type {Array<number>} */ var dx = deMath.multiply(deMath.scale(deMath.subtract(this.m_coordMax, this.m_coordMin), 1 / w), xScale);
            /** @type {Array<number>} */ var dy = deMath.multiply(deMath.scale(deMath.subtract(this.m_coordMax, this.m_coordMin), 1 / h), yScale);
            reference = deMath.add(deMath.abs(dx), deMath.abs(dy));
            /** @type {Array<number>} */ var dxThreshold = es3fShaderDerivateTests.getDerivateThreshold(this.m_precision, deMath.multiply(this.m_coordMin, xScale), deMath.multiply(this.m_coordMax, xScale), dx);
            /** @type {Array<number>} */ var dyThreshold = es3fShaderDerivateTests.getDerivateThreshold(this.m_precision, deMath.multiply(this.m_coordMin, yScale), deMath.multiply(this.m_coordMax, yScale), dy);
            threshold = deMath.max(surfaceThreshold, deMath.max(dxThreshold, dyThreshold));

            return es3fShaderDerivateTests.verifyConstantDerivate(result, errorMask, this.m_dataType,
                                          reference, threshold, this.m_derivScale, this.m_derivBias);
        }
    };

    /**
     * @constructor
     * @extends {es3fShaderDerivateTests.TriangleDerivateCase}
     * @param {string} name
     * @param {string} description
     * @param {es3fShaderDerivateTests.DerivateFunc} func
     * @param {gluShaderUtil.DataType} type
     * @param {gluShaderUtil.precision} precision
     * @param {number} hint
     * @param {es3fShaderDerivateTests.SurfaceType} surfaceType
     * @param {number} numSamples
     */
    es3fShaderDerivateTests.TextureDerivateCase = function(name, description, func, type, precision, hint, surfaceType, numSamples) {
        es3fShaderDerivateTests.TriangleDerivateCase.call(this, name, description);
        /** @type {es3fShaderDerivateTests.DerivateFunc} */ this.m_func = func;
        /** @type {gluTexture.Texture2D} */ this.m_texture = null;
        /** @type {Array<number>} */ this.m_texValueMin = [];
        /** @type {Array<number>} */ this.m_texValueMax = [];
        this.m_dataType = type;
        this.m_precision = precision;
        this.m_coordDataType = gluShaderUtil.DataType.FLOAT_VEC2;
        this.m_coordPrecision = gluShaderUtil.precision.PRECISION_HIGHP;
        this.m_hint = hint;
        this.m_surfaceType = surfaceType;
        this.m_numSamples = numSamples;
    };

    es3fShaderDerivateTests.TextureDerivateCase.prototype = Object.create(es3fShaderDerivateTests.TriangleDerivateCase.prototype);
    es3fShaderDerivateTests.TextureDerivateCase.prototype.constructor = es3fShaderDerivateTests.TextureDerivateCase;

    es3fShaderDerivateTests.TextureDerivateCase.prototype.deinit = function() {
            this.m_texture = null;
    };

    es3fShaderDerivateTests.TextureDerivateCase.prototype.init = function() {
        // Generate shader
        /** @type {string} */ var fragmentTmpl = '' +
            '#version 300 es\n' +
            'in highp vec2 v_coord;\n' +
            'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
            'uniform ${PRECISION} sampler2D u_sampler;\n' +
            'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
            'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
            'void main (void)\n' +
            '{\n' +
            ' ${PRECISION} vec4 tex = texture(u_sampler, v_coord);\n' +
            ' ${PRECISION} ${DATATYPE} res = ${FUNC}(tex${SWIZZLE}) * u_scale + u_bias;\n' +
            ' o_color = ${CAST_TO_OUTPUT};\n' +
            '}\n';

        /** @type {boolean} */ var packToInt = this.m_surfaceType === es3fShaderDerivateTests.SurfaceType.FLOAT_FBO;
        /** @type {Object} */ var fragmentParams = {};
        /** @type {Array<number>} */ var viewportSize;
        fragmentParams['OUTPUT_TYPE'] = gluShaderUtil.getDataTypeName(packToInt ? gluShaderUtil.DataType.UINT_VEC4 : gluShaderUtil.DataType.FLOAT_VEC4);
        fragmentParams['OUTPUT_PREC'] = gluShaderUtil.getPrecisionName(packToInt ? gluShaderUtil.precision.PRECISION_HIGHP : this.m_precision);
        fragmentParams['PRECISION'] = gluShaderUtil.getPrecisionName(this.m_precision);
        fragmentParams['DATATYPE'] = gluShaderUtil.getDataTypeName(this.m_dataType);
        fragmentParams['FUNC'] = es3fShaderDerivateTests.getDerivateFuncName(this.m_func);
        fragmentParams['SWIZZLE'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? '' :
            this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? '.xyz' :
            this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? '.xy' :
            '.x';

        if (packToInt) {
            fragmentParams['CAST_TO_OUTPUT'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? 'floatBitsToUint(res)' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? 'floatBitsToUint(vec4(res, 1.0))' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? 'floatBitsToUint(vec4(res, 0.0, 1.0))' :
                'floatBitsToUint(vec4(res, 0.0, 0.0, 1.0))';
        } else {
            fragmentParams['CAST_TO_OUTPUT'] = this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC4 ? 'res' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC3 ? 'vec4(res, 1.0)' :
                this.m_dataType === gluShaderUtil.DataType.FLOAT_VEC2 ? 'vec4(res, 0.0, 1.0)' :
                'vec4(res, 0.0, 0.0, 1.0)';
        }

        this.m_fragmentSrc = tcuStringTemplate.specialize(fragmentTmpl, fragmentParams);

        // Texture size matches viewport and nearest sampling is used. Thus texture sampling
        // is equal to just interpolating the texture value range.

        // Determine value range for texture.

        switch (this.m_precision) {
            case gluShaderUtil.precision.PRECISION_HIGHP:
                this.m_texValueMin = [-97., 0.2, 71., 74.];
                this.m_texValueMax = [-13.2, -77., 44., 76.];
                break;

            case gluShaderUtil.precision.PRECISION_MEDIUMP:
                this.m_texValueMin = [-37.0, 47., -7., 0.0];
                this.m_texValueMax = [-1.0, 12., 7., 19.];
                break;

            case gluShaderUtil.precision.PRECISION_LOWP:
                this.m_texValueMin = [0.0, -1.0, 0.0, 1.0];
                this.m_texValueMax = [1.0, 1.0, -1.0, -1.0];
                break;

            default:
                throw new Error(false, 'Precision not supported:' + this.m_precision);
        }

        // Lowp and mediump cases use RGBA16F format, while highp uses RGBA32F.
        viewportSize = this.getViewportSize();
        assertMsgOptions(!this.m_texture, 'Texture not null', false, true);
        this.m_texture = gluTexture.texture2DFromInternalFormat(gl, this.m_precision === gluShaderUtil.precision.PRECISION_HIGHP ? gl.RGBA32F : gl.RGBA16F, viewportSize[0], viewportSize[1]);
        this.m_texture.getRefTexture().allocLevel(0);

        // Texture coordinates
        this.m_coordMin = [0.0, 0.0, 0.0, 0.0];
        this.m_coordMax = [1.0, 1.0, 1.0, 1.0];

        // Fill with gradients.
        /** @type {tcuTexture.PixelBufferAccess} */ var level0 = this.m_texture.getRefTexture().getLevel(0);
        for (var y = 0; y < level0.getHeight(); y++) {
            for (var x = 0; x < level0.getWidth(); x++) {
                /** @type {number} */ var xf = (x + 0.5) / level0.getWidth();
                /** @type {number} */ var yf = (y + 0.5) / level0.getHeight();
                /** @type {Array<number>} */ var s = [xf, yf, (xf + yf) / 2.0, 1.0 - (xf + yf) / 2.0];

                level0.setPixel(deMath.add(this.m_texValueMin, deMath.multiply(deMath.subtract(this.m_texValueMax, this.m_texValueMin), s)), x, y);
            }
        }

        this.m_texture.upload();

        if (this.m_surfaceType === es3fShaderDerivateTests.SurfaceType.FLOAT_FBO) {
            // No scale or bias used for accuracy.
            this.m_derivScale = [1.0, 1.0, 1.0, 1.0];
            this.m_derivBias = [0.0, 0.0, 0.0, 0.0];
        } else {
            // Compute scale - bias that normalizes to 0..1 range.
            viewportSize = this.getViewportSize();
            /** @type {number} */ var w = viewportSize[0];
            /** @type {number} */ var h = viewportSize[1];
            /** @type {Array<number>} */ var dx = deMath.divide(deMath.subtract(this.m_texValueMax, this.m_texValueMin), [w, w, w * 0.5, -w * 0.5]);
            /** @type {Array<number>} */ var dy = deMath.divide(deMath.subtract(this.m_texValueMax, this.m_texValueMin), [h, h, h * 0.5, -h * 0.5]);

            switch (this.m_func) {
                case es3fShaderDerivateTests.DerivateFunc.DFDX:
                    this.m_derivScale = deMath.divide([0.5, 0.5, 0.5, 0.5], dx);
                    break;

                case es3fShaderDerivateTests.DerivateFunc.DFDY:
                    this.m_derivScale = deMath.divide([0.5, 0.5, 0.5, 0.5], dy);
                    break;

                case es3fShaderDerivateTests.DerivateFunc.FWIDTH:
                    this.m_derivScale = deMath.divide([0.5, 0.5, 0.5, 0.5], deMath.add(deMath.abs(dx), deMath.abs(dy)));
                    break;

                default:
                    throw new Error('Derivate Function not supported: ' + this.m_func);
            }

            this.m_derivBias = [0.0, 0.0, 0.0, 0.0];
        }
    };

    /**
     * @param {WebGLProgram} program
     */
    es3fShaderDerivateTests.TextureDerivateCase.prototype.setupRenderState = function(program) {
        /** @type {number} */ var texUnit = 1;

        gl.activeTexture(gl.TEXTURE0 + texUnit);
        gl.bindTexture(gl.TEXTURE_2D, this.m_texture.getGLTexture());
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

        gl.uniform1i(gl.getUniformLocation(program, 'u_sampler'), texUnit);
    };

    /**
     * @param {tcuTexture.PixelBufferAccess} result
     * @param {tcuTexture.PixelBufferAccess} errorMask
     * @return {boolean}
     */
    es3fShaderDerivateTests.TextureDerivateCase.prototype.verify = function(result, errorMask) {
        // \note Edges are ignored in comparison
        if (result.getWidth() < 2 || result.getHeight() < 2)
            throw new Error('Too small viewport');

        /** @type {tcuTexture.PixelBufferAccess} */ var compareArea = tcuTextureUtil.getSubregion(result, 1, 1, 0, result.getWidth() - 2, result.getHeight() - 2, 1);
        /** @type {tcuTexture.PixelBufferAccess} */ var maskArea = tcuTextureUtil.getSubregion(errorMask, 1, 1, 0, errorMask.getWidth() - 2, errorMask.getHeight() - 2, 1);
        /** @type {Array<number>} */ var xScale = [1.0, 0.0, 0.5, -0.5];
        /** @type {Array<number>} */ var yScale = [0.0, 1.0, 0.5, -0.5];
        /** @type {number} */ var w = result.getWidth();
        /** @type {number} */ var h = result.getHeight();

        /** @type {Array<number>} */ var surfaceThreshold = deMath.divide(this.getSurfaceThreshold(), deMath.abs(this.m_derivScale));
        /** @type {Array<number>} */ var reference;
        /** @type {Array<number>} */ var threshold;
        if (this.m_func == es3fShaderDerivateTests.DerivateFunc.DFDX || this.m_func == es3fShaderDerivateTests.DerivateFunc.DFDY) {
            /** @type {boolean} */ var isX = this.m_func == es3fShaderDerivateTests.DerivateFunc.DFDX;
            /** @type {number} */ var div = isX ? w : h;
            /** @type {Array<number>} */ var scale = isX ? xScale : yScale;
            reference = deMath.multiply(deMath.scale(deMath.subtract(this.m_texValueMax, this.m_texValueMin), 1 / div), scale);
            /** @type {Array<number>} */ var opThreshold = es3fShaderDerivateTests.getDerivateThreshold(this.m_precision, deMath.multiply(this.m_texValueMin, scale), deMath.multiply(this.m_texValueMax, scale), reference);
            threshold = deMath.max(surfaceThreshold, opThreshold);

            bufferedLogToConsole('Verifying result image.\n'+
                '\tValid derivative is ' + reference + ' with threshold ' + threshold);

            // short circuit if result is strictly within the normal value error bounds.
            // This improves performance significantly.
            if (es3fShaderDerivateTests.verifyConstantDerivate(compareArea, maskArea, this.m_dataType,
                reference, threshold, this.m_derivScale, this.m_derivBias,
                es3fShaderDerivateTests.VerificationLogging.LOG_NOTHING)) {
                    bufferedLogToConsole('No incorrect derivatives found, result valid.');
                    return true;
            }
            // some pixels exceed error bounds calculated for normal values. Verify that these
            // potentially invalid pixels are in fact valid due to (for example) subnorm flushing.

            bufferedLogToConsole('Initial verification failed, verifying image by calculating accurate error bounds for each result pixel.\n' +
                '\tVerifying each result derivative is within its range of legal result values.');

            /** @type {Array<number>} */ var valueRamp = deMath.subtract(this.m_texValueMax, this.m_texValueMin);
            /** @type {es3fShaderDerivateTests.Linear2DFunctionEvaluator} */ var function_ = new es3fShaderDerivateTests.Linear2DFunctionEvaluator();

            function_.matrix.setRow(0, [valueRamp[0] / w, 0.0, this.m_texValueMin[0]]);
            function_.matrix.setRow(1, [0.0, valueRamp[1] / h, this.m_texValueMin[1]]);
            function_.matrix.setRow(2, deMath.scale([valueRamp[2] / w, valueRamp[2] / h, this.m_texValueMin[2] + this.m_texValueMin[2]], 1 / 2.0));
            function_.matrix.setRow(3, deMath.scale([-valueRamp[3] / w, -valueRamp[3] / h, this.m_texValueMax[3] + this.m_texValueMax[3]], 1 / 2.0));

            return es3fShaderDerivateTests.reverifyConstantDerivateWithFlushRelaxations(compareArea, maskArea, this.m_dataType, this.m_precision,
                this.m_derivScale, this.m_derivBias, surfaceThreshold, this.m_func, function_);
        } else {
            assertMsgOptions(this.m_func == es3fShaderDerivateTests.DerivateFunc.FWIDTH, 'Expected Derivate Function FWIDTH', false, true);
            /** @type {Array<number>} */ var dx = deMath.multiply(deMath.scale(deMath.subtract(this.m_texValueMax, this.m_texValueMin), 1 / w), xScale);
            /** @type {Array<number>} */ var dy = deMath.multiply(deMath.scale(deMath.subtract(this.m_texValueMax, this.m_texValueMin), 1 / h), yScale);
            reference = deMath.add(deMath.abs(dx), deMath.abs(dy));
            /** @type {Array<number>} */ var dxThreshold = es3fShaderDerivateTests.getDerivateThreshold(this.m_precision, deMath.multiply(this.m_texValueMin, xScale), deMath.multiply(this.m_texValueMax, xScale), dx);
            /** @type {Array<number>} */ var dyThreshold = es3fShaderDerivateTests.getDerivateThreshold(this.m_precision, deMath.multiply(this.m_texValueMin, yScale), deMath.multiply(this.m_texValueMax, yScale), dy);
            threshold = deMath.max(surfaceThreshold, deMath.max(dxThreshold, dyThreshold));

            return es3fShaderDerivateTests.verifyConstantDerivate(compareArea, maskArea, this.m_dataType,
                reference, threshold, this.m_derivScale, this.m_derivBias);
        };
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fShaderDerivateTests.ShaderDerivateTests = function() {
        tcuTestCase.DeqpTest.call(this, 'derivate', 'Derivate Function Tests');
    };

    es3fShaderDerivateTests.ShaderDerivateTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderDerivateTests.ShaderDerivateTests.prototype.constructor = es3fShaderDerivateTests.ShaderDerivateTests

    /**
     * @struct
     * @constructor
     * @param {string} name
     * @param {es3fShaderDerivateTests.DerivateFunc} func
     * @param {gluShaderUtil.DataType} dataType_
     * @param {gluShaderUtil.precision} precision_
     */
    es3fShaderDerivateTests.FunctionSpec = function(name, func, dataType_, precision_) {
        this.name = name;
        this.function_ = func;
        this.dataType = dataType_;
        this.precision = precision_;
    };

    es3fShaderDerivateTests.ShaderDerivateTests.prototype.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        /**
         * @struct
         * @constructor
         * @param {string} name
         * @param {string} description
         * @param {string} source
         */
        var LinearDerivateCase = function(name, description, source) {
            /** @type {string} */ this.name = name;
            /** @type {string} */ this.description = description;
            /** @type {string} */ this.source = source;
        };

        /** @type {Array<LinearDerivateCase>} */
        var s_linearDerivateCases = [
            new LinearDerivateCase(
                'linear',
                'Basic derivate of linearly interpolated argument',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res = ${FUNC}(v_coord) * u_scale + u_bias;\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'in_function',
                'Derivate of linear function argument',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                '\n' +
                '${PRECISION} ${DATATYPE} computeRes (${PRECISION} ${DATATYPE} value)\n' +
                '{\n' +
                ' return ${FUNC}(v_coord) * u_scale + u_bias;\n' +
                '}\n' +
                '\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res = computeRes(v_coord);\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'static_if',
                'Derivate of linearly interpolated value in static if',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res;\n' +
                ' if (false)\n' +
                ' res = ${FUNC}(-v_coord) * u_scale + u_bias;\n' +
                ' else\n' +
                ' res = ${FUNC}(v_coord) * u_scale + u_bias;\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'static_loop',
                'Derivate of linearly interpolated value in static loop',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res = ${DATATYPE}(0.0);\n' +
                ' for (int i = 0; i < 2; i++)\n' +
                ' res += ${FUNC}(v_coord * float(i));\n' +
                ' res = res * u_scale + u_bias;\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'static_switch',
                'Derivate of linearly interpolated value in static switch',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res;\n' +
                ' switch (1)\n' +
                ' {\n' +
                ' case 0: res = ${FUNC}(-v_coord) * u_scale + u_bias; break;\n' +
                ' case 1: res = ${FUNC}(v_coord) * u_scale + u_bias; break;\n' +
                ' }\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'uniform_if',
                'Derivate of linearly interpolated value in uniform if',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'uniform bool ub_true;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res;\n' +
                ' if (ub_true)\n' +
                ' res = ${FUNC}(v_coord) * u_scale + u_bias;\n' +
                ' else\n' +
                ' res = ${FUNC}(-v_coord) * u_scale + u_bias;\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'uniform_loop',
                'Derivate of linearly interpolated value in uniform loop',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'uniform int ui_two;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res = ${DATATYPE}(0.0);\n' +
                ' for (int i = 0; i < ui_two; i++)\n' +
                ' res += ${FUNC}(v_coord * float(i));\n' +
                ' res = res * u_scale + u_bias;\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n'),
            new LinearDerivateCase(
                'uniform_switch',
                'Derivate of linearly interpolated value in uniform switch',
                '#version 300 es\n' +
                'in ${PRECISION} ${DATATYPE} v_coord;\n' +
                'layout(location = 0) out ${OUTPUT_PREC} ${OUTPUT_TYPE} o_color;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_scale;\n' +
                'uniform ${PRECISION} ${DATATYPE} u_bias;\n' +
                'uniform int ui_one;\n' +
                'void main (void)\n' +
                '{\n' +
                ' ${PRECISION} ${DATATYPE} res;\n' +
                ' switch (ui_one)\n' +
                ' {\n' +
                ' case 0: res = ${FUNC}(-v_coord) * u_scale + u_bias; break;\n' +
                ' case 1: res = ${FUNC}(v_coord) * u_scale + u_bias; break;\n' +
                ' }\n' +
                ' o_color = ${CAST_TO_OUTPUT};\n' +
                '}\n')
        ];

        /**
         * @struct
         * @constructor
         * @param {string} name
         * @param {es3fShaderDerivateTests.SurfaceType} surfaceType
         * @param {number} numSamples
         */
        var FboConfig = function(name, surfaceType, numSamples) {
            /** @type {string} */ this.name = name;
            /** @type {es3fShaderDerivateTests.SurfaceType} */ this.surfaceType = surfaceType;
            /** @type {number} */ this.numSamples = numSamples;
        };

        /** @type {Array<FboConfig>} */ var s_fboConfigs = [
            new FboConfig('fbo', es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER, 0),
            new FboConfig('fbo_msaa2', es3fShaderDerivateTests.SurfaceType.UNORM_FBO, 2),
            new FboConfig('fbo_msaa4', es3fShaderDerivateTests.SurfaceType.UNORM_FBO, 4),
            new FboConfig('fbo_float', es3fShaderDerivateTests.SurfaceType.FLOAT_FBO, 0)
        ];

        /**
         * @struct
         * @constructor
         * @param {string} name
         * @param {number} hint
         */
        var Hint = function(name, hint) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.hint = hint;
        };

        /** @type {Array<Hint>} */ var s_hints = [
            new Hint('fastest', gl.FASTEST),
            new Hint('nicest', gl.NICEST)
        ];

        /**
         * @struct
         * @constructor
         * @param {string} name
         * @param {es3fShaderDerivateTests.SurfaceType} surfaceType
         * @param {number} numSamples
         */
        var HintFboConfig = function(name, surfaceType, numSamples) {
            /** @type {string} */ this.name = name;
            /** @type {es3fShaderDerivateTests.SurfaceType} */ this.surfaceType = surfaceType;
            /** @type {number} */ this.numSamples = numSamples;
        };

        /** @type {Array<HintFboConfig>} */ var s_hintFboConfigs = [
            new HintFboConfig('default', es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER, 0),
            new HintFboConfig('fbo_msaa4', es3fShaderDerivateTests.SurfaceType.UNORM_FBO, 4),
            new HintFboConfig('fbo_float', es3fShaderDerivateTests.SurfaceType.FLOAT_FBO, 0)
        ];

        /**
         * @struct
         * @constructor
         * @param {string} name
         * @param {es3fShaderDerivateTests.SurfaceType} surfaceType
         * @param {number} numSamples
         * @param {number} hint
         */
        var TextureConfig = function(name, surfaceType, numSamples, hint) {
            /** @type {string} */ this.name = name;
            /** @type {es3fShaderDerivateTests.SurfaceType} */ this.surfaceType = surfaceType;
            /** @type {number} */ this.numSamples = numSamples;
            /** @type {number} */ this.hint = hint;
        };

        /** @type {Array<TextureConfig>} */ var s_textureConfigs = [
            new TextureConfig('basic', es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER, 0, gl.DONT_CARE),
            new TextureConfig('msaa4', es3fShaderDerivateTests.SurfaceType.UNORM_FBO, 4, gl.DONT_CARE),
            new TextureConfig('float_fastest', es3fShaderDerivateTests.SurfaceType.FLOAT_FBO, 0, gl.FASTEST),
            new TextureConfig('float_nicest', es3fShaderDerivateTests.SurfaceType.FLOAT_FBO, 0, gl.NICEST)
        ];

        /** @type {gluShaderUtil.DataType} */ var dataType;
        /** @type {string} */ var source;
        /** @type {gluShaderUtil.precision} */ var precision;
        /** @type {es3fShaderDerivateTests.SurfaceType} */ var surfaceType;
        /** @type {number} */ var numSamples;
        /** @type {number} */ var hint;
        /** @type {string} */ var caseName;
        /** @type {tcuTestCase.DeqpTest} */ var fboGroup;

        // .dfdx, .dfdy, .fwidth
        for (var funcNdx in es3fShaderDerivateTests.DerivateFunc) {
            /** @type {es3fShaderDerivateTests.DerivateFunc} */ var function_ = es3fShaderDerivateTests.DerivateFunc[funcNdx];
            /** @type {tcuTestCase.DeqpTest} */ var functionGroup = tcuTestCase.newTest(es3fShaderDerivateTests.getDerivateFuncCaseName(function_), es3fShaderDerivateTests.getDerivateFuncName(function_));
            testGroup.addChild(functionGroup);

            // .constant - no precision variants, checks that derivate of constant arguments is 0
            /** @type {tcuTestCase.DeqpTest} */ var constantGroup = tcuTestCase.newTest('constant', 'Derivate of constant argument');
            functionGroup.addChild(constantGroup);

            for (var vecSize = 1; vecSize <= 4; vecSize++) {
                dataType = vecSize > 1 ? gluShaderUtil.getDataTypeFloatVec(vecSize) : gluShaderUtil.DataType.FLOAT;
                constantGroup.addChild(new es3fShaderDerivateTests.ConstantDerivateCase(gluShaderUtil.getDataTypeName(dataType), '', function_, dataType));
            }

            // Cases based on LinearDerivateCase
            for (var caseNdx = 0; caseNdx < s_linearDerivateCases.length; caseNdx++) {
                /** @type {tcuTestCase.DeqpTest} */ var linearCaseGroup = tcuTestCase.newTest(s_linearDerivateCases[caseNdx].name, s_linearDerivateCases[caseNdx].description);
                source = s_linearDerivateCases[caseNdx].source;
                functionGroup.addChild(linearCaseGroup);

                for (var vecSize = 1; vecSize <= 4; vecSize++)
                for (var precNdx in gluShaderUtil.precision) {
                    dataType = vecSize > 1 ? gluShaderUtil.getDataTypeFloatVec(vecSize) : gluShaderUtil.DataType.FLOAT;
                    precision = gluShaderUtil.precision[precNdx];
                    surfaceType = es3fShaderDerivateTests.SurfaceType.DEFAULT_FRAMEBUFFER;
                    numSamples = 0;
                    hint = gl.DONT_CARE;

                    if (caseNdx !== 0 && precision === gluShaderUtil.precision.PRECISION_LOWP)
                        continue; // Skip as lowp doesn't actually produce any bits when rendered to default FB.

                    caseName = gluShaderUtil.getDataTypeName(dataType) + '_' + gluShaderUtil.getPrecisionName(precision);

                    linearCaseGroup.addChild(new es3fShaderDerivateTests.LinearDerivateCase(caseName, '', function_, dataType, precision, hint, surfaceType, numSamples, source));
                }
            }

            // Fbo cases
            for (var caseNdx = 0; caseNdx < s_fboConfigs.length; caseNdx++) {
                fboGroup = tcuTestCase.newTest(s_fboConfigs[caseNdx].name, 'Derivate usage when rendering into FBO');
                source = s_linearDerivateCases[0].source; // use source from .linear group
                surfaceType = s_fboConfigs[caseNdx].surfaceType;
                numSamples = s_fboConfigs[caseNdx].numSamples;
                functionGroup.addChild(fboGroup);

                for (var vecSize = 1; vecSize <= 4; vecSize++)
                for (var precNdx in gluShaderUtil.precision) {
                    dataType = vecSize > 1 ? gluShaderUtil.getDataTypeFloatVec(vecSize) : gluShaderUtil.DataType.FLOAT;
                    precision = gluShaderUtil.precision[precNdx];
                    hint = gl.DONT_CARE;

                    if (surfaceType !== es3fShaderDerivateTests.SurfaceType.FLOAT_FBO && precision === gluShaderUtil.precision.PRECISION_LOWP)
                        continue; // Skip as lowp doesn't actually produce any bits when rendered to U8 RT.

                    caseName = gluShaderUtil.getDataTypeName(dataType) + '_' + gluShaderUtil.getPrecisionName(precision);

                    fboGroup.addChild(new es3fShaderDerivateTests.LinearDerivateCase(caseName, '', function_, dataType, precision, hint, surfaceType, numSamples, source));
                }
            }

            // .fastest, .nicest
            for (var hintCaseNdx = 0; hintCaseNdx < s_hints.length; hintCaseNdx++) {
                /** @type {tcuTestCase.DeqpTest} */ var hintGroup = tcuTestCase.newTest(s_hints[hintCaseNdx].name, 'Shader derivate hints');
                source = s_linearDerivateCases[0].source; // use source from .linear group
                hint = s_hints[hintCaseNdx].hint;
                functionGroup.addChild(hintGroup);

                for (var fboCaseNdx = 0; fboCaseNdx < s_hintFboConfigs.length; fboCaseNdx++) {
                    fboGroup = tcuTestCase.newTest(s_hintFboConfigs[fboCaseNdx].name, '');
                    surfaceType = s_hintFboConfigs[fboCaseNdx].surfaceType;
                    numSamples = s_hintFboConfigs[fboCaseNdx].numSamples;
                    hintGroup.addChild(fboGroup);

                    for (var vecSize = 1; vecSize <= 4; vecSize++)
                    for (var precNdx in gluShaderUtil.precision) {
                        dataType = vecSize > 1 ? gluShaderUtil.getDataTypeFloatVec(vecSize) : gluShaderUtil.DataType.FLOAT;
                        precision = gluShaderUtil.precision[precNdx];

                        if (surfaceType !== es3fShaderDerivateTests.SurfaceType.FLOAT_FBO && precision === gluShaderUtil.precision.PRECISION_LOWP)
                            continue; // Skip as lowp doesn't actually produce any bits when rendered to U8 RT.

                        caseName = gluShaderUtil.getDataTypeName(dataType) + '_' + gluShaderUtil.getPrecisionName(precision);

                        fboGroup.addChild(new es3fShaderDerivateTests.LinearDerivateCase(caseName, '', function_, dataType, precision, hint, surfaceType, numSamples, source));
                    }
                }
            }

            // .texture
            /** @type {tcuTestCase.DeqpTest} */ var textureGroup = tcuTestCase.newTest('texture', 'Derivate of texture lookup result');
            functionGroup.addChild(textureGroup);

            for (var texCaseNdx = 0; texCaseNdx < s_textureConfigs.length; texCaseNdx++) {
                /** @type {tcuTestCase.DeqpTest} */ var caseGroup = tcuTestCase.newTest(s_textureConfigs[texCaseNdx].name, '');
                surfaceType = s_textureConfigs[texCaseNdx].surfaceType;
                numSamples = s_textureConfigs[texCaseNdx].numSamples;
                hint = s_textureConfigs[texCaseNdx].hint;
                textureGroup.addChild(caseGroup);

                for (var vecSize = 1; vecSize <= 4; vecSize++)
                for (var precNdx in gluShaderUtil.precision) {
                    dataType = vecSize > 1 ? gluShaderUtil.getDataTypeFloatVec(vecSize) : gluShaderUtil.DataType.FLOAT;
                    precision = gluShaderUtil.precision[precNdx];

                    if (surfaceType !== es3fShaderDerivateTests.SurfaceType.FLOAT_FBO && precision === gluShaderUtil.precision.PRECISION_LOWP)
                        continue; // Skip as lowp doesn't actually produce any bits when rendered to U8 RT.

                    caseName = gluShaderUtil.getDataTypeName(dataType) + '_' + gluShaderUtil.getPrecisionName(precision);

                    caseGroup.addChild(new es3fShaderDerivateTests.TextureDerivateCase(caseName, '', function_, dataType, precision, hint, surfaceType, numSamples));
                }
            }
        }
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
    es3fShaderDerivateTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderDerivateTests.ShaderDerivateTests());

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
            testFailedOptions('Failed to es3fShaderDerivateTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});

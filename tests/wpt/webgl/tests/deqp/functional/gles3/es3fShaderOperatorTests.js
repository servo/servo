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
goog.provide('functional.gles3.es3fShaderOperatorTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.delibs.debase.deMath');
goog.require('modules.shared.glsShaderRenderCase');
goog.require('framework.common.tcuMatrix');

goog.scope(function() {
var es3fShaderOperatorTests = functional.gles3.es3fShaderOperatorTests;
var tcuTestCase = framework.common.tcuTestCase;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var deMath = framework.delibs.debase.deMath;
var glsShaderRenderCase = modules.shared.glsShaderRenderCase;
var tcuMatrix = framework.common.tcuMatrix;

/** @const */ es3fShaderOperatorTests.MAX_INPUTS = 3;

es3fShaderOperatorTests.stringJoin = function(elems, delim) {
    var result = '';
    for (var i = 0; i < elems.length; i++)
        result += (i > 0 ? delim : '') + elems[i];
    return result;
};

es3fShaderOperatorTests.twoValuedVec4 = function(first, second, firstMask) {
    var elems = [];
    for (var i = 0; i < 4; i++)
        elems[i] = firstMask[i] ? first : second;

    return 'vec4(' + es3fShaderOperatorTests.stringJoin(elems, ', ') + ')';
};

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

var negate = function(x) {
    return -x;
};

var addOne = function(x) {
    return x + 1;
};

var subOne = function(x) {
    return x - 1;
};

/**
 * @param {number} a
 * @param {number} b
 */
var add = function(a, b) {
    return a + b;
};

/**
 * @param {number} a
 * @param {number} b
 */
var sub = function(a, b) {
    return a - b;
};

/**
 * @param {number} a
 * @param {number} b
 */
var mul = function(a, b) {
    return a * b;
};

/**
 * @param {number} a
 * @param {number} b
 */
var div = function(a, b) {
    return a / b;
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
var lessThan = function(a, b) {
    return a < b ? 1 : 0;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var lessThanVec = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = a[i] < b[i];
    return res
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
var lessThanEqual = function(a, b) {
    return a <= b ? 1 : 0;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var lessThanEqualVec = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = a[i] <= b[i];
    return res;
};

/**
 * @param {number} a
 * @param {number} b
 */
var greaterThan = function(a, b) {
    return a > b ? 1 : 0;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var greaterThanVec = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = a[i] > b[i];
    return res;
};

/**
 * @param {number} a
 * @param {number} b
 */
var greaterThanEqual = function(a, b) {
    return a >= b ? 1 : 0;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var greaterThanEqualVec = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = a[i] >= b[i];
    return res;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {number}
 */
var allEqual = function(a, b) {
    return deMath.equal(a, b) ? 1 : 0;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var allEqualVec = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = a[i] == b[i];
    return res;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {number}
 */
var anyNotEqual = function(a, b) {
    return !deMath.equal(a, b) ? 1 : 0;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var anyNotEqualVec = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = a[i] != b[i];
    return res;
};

/**
 * @param {Array<number>} a
 * @return {Array<number>}
 */
var boolNotVec = function(a) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = !a[i];
    return res;
}

/**
 * @param {Array<number>} a
 * @return {boolean}
 */
var boolAny = function(a) {
    for (var i = 0; i < a.length; i++)
        if (a[i] == true)
            return true;
    return false;
}

/**
 * @param {Array<number>} a
 * @return {boolean}
 */
var boolAll = function(a) {
    for (var i = 0; i < a.length; i++)
        if (a[i] == false)
            return false;
    return true;
}

/**
 * @param {number} a
 * @param {number} b
 */
var logicalAnd = function(a, b) {
    return a && b ? 1 : 0;
};

/**
 * @param {number} a
 * @param {number} b
 */
var logicalOr = function(a, b) {
    return a || b ? 1 : 0;
};

/**
 * @param {number} a
 * @param {number} b
 */
var logicalXor = function(a, b) {
    return a != b ? 1 : 0;
};

/**
 * @param {number} a
 */
var exp2 = function(a) {
    return deFloatExp2(a);
};

/**
 * @param {number} a
 */
var inverseSqrt = function(a) {
    return deFloatRsq(a);
};

/**
 * @param {Array<number>} a
 * @param {number} b
 * @return {Array<number>}
 */
var minVecScalar = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = Math.min(a[i], b);
    return res;
};

/**
 * @param {Array<number>} a
 * @param {number} b
 * @return {Array<number>}
 */
var maxVecScalar = function(a, b) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = Math.max(a[i], b);
    return res;
};

/**
 * @param {number} a
 * @param {number} b
 * @param {number} c
 * @return {number}
 */
var mix = function(a, b, c) {
    return a * (1.0 - c) + b * c;
};
/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @param {number} c
 * @return {Array<number>}
 */
var mixVecVecScalar = function(a, b, c) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = mix(a[i], b[i], c);
    return res;
};

/**
 * @param {Array<number>} a
 * @param {number} b
 * @param {number} c
 * @return {Array<number>}
 */
var clampVecScalarScalar = function(a, b, c) {
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = deMath.clamp(a[i], b, c);
    return res;
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
var step = function(a, b) {
    return b < a ? 0.0 : 1.0;
};

/**
 * @param {number} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var stepScalarVec = function(a, b) {
    var res = [];
    for (var i = 0; i < b.length; i++)
        res[i] = step(a, b[i]);
    return res;
};

/**
 * @param {number} a
 * @param {number} b
 * @param {number} c
 * @return {number}
 */
var smoothStep = function(a, b, c) {
    if (c <= a) return 0.0;
    if (c >= b) return 1.0;
    var t = deMath.clamp((c - a) / (b - a), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
};

/**
 * @param {number} a
 * @param {number} b
 * @param {Array<number>} c
 * @return {Array<number>}
 */
var smoothStepScalarScalarVec = function(a, b, c) {
    var res = [];
    for (var i = 0; i < c.length; i++)
        res[i] = smoothStep(a, b, c[i]);
    return res;
};

/**
 * @param {number} a
 * @return {number}
 */
var roundToEven = function(a) {
    var q = deMath.deFloatFrac(a);
    var r = a - q;

    if (q > 0.5)
        r += 1.0;
    else if (q == 0.5 && r % 2 != 0)
        r += 1.0;

    return r;
};

/**
 * @param {number} a
 * @return {number}
 */
var fract = function(a) {
    return a - Math.floor(a);
};

/**
 * @param {number} a
 * @return {number}
 */
var radians = function(a) {
    return deFloatRadians(a);
}

/**
 * @param {number} a
 * @return {number}
 */
var degrees = function(a) {
    return deFloatDegrees(a);
}

/**
 * @param {number} a
 * @param {Array<number>} b
 */
var addScalarVec = function(a, b) {
    return deMath.addScalar(b, a);
};

/**
 * @param {number} a
 * @param {Array<number>} b
 */
var subScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(a - b[i]);
    return dst;
};

/**
 * @param {number} a
 * @param {Array<number>} b
 */
var mulScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(a * b[i]);
    return dst;
};

/**
 * @param {number} a
 * @param {Array<number>} b
 */
var divScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(a / b[i]);
    return dst;
};

/**
 * @param {number} a
 * @param {Array<number>} b
 */
var modScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(a % b[i]);
    return dst;
};

var bitwiseAndScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(deMath.binaryOp(a, b[i], deMath.BinaryOp.AND));
    return dst;
};

var bitwiseOrScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(deMath.binaryOp(a, b[i], deMath.BinaryOp.OR));
    return dst;
};

var bitwiseXorScalarVec = function(a, b) {
    var dst = [];
    for (var i = 0; i < b.length; i++)
        dst.push(deMath.binaryOp(a, b[i], deMath.BinaryOp.XOR));
    return dst;
};

/**
 * @param {Array<number>} a
 * @return {number}
 */
var length = function(a) {
    var squareSum = 0;
    for (var i = 0; i < a.length; i++)
        squareSum += a[i] * a[i];
    return Math.sqrt(squareSum);
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {number}
 */
var distance = function(a, b) {
    var res = deMath.subtract(a, b)
    return length(res);
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {number}
 */
var dot = function(a, b) {
    var res = deMath.multiply(a, b);
    var sum = 0;
    for (var i = 0; i < res.length; i++)
        sum += res[i];
    return sum;
};

/**
 * @param {Array<number>} a
 * @return {Array<number>}
 */
var normalize = function(a) {
    var ooLen = 1 / length(a);
    var res = [];
    for (var i = 0; i < a.length; i++)
        res[i] = ooLen * a[i];
    return res;
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @param {Array<number>} c
 * @return {Array<number>}
 */
var faceforward = function(a, b, c) {
    return dot(c, b) < 0 ? a : deMath.scale(a, -1);
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var reflect = function(a, b) {
    return deMath.subtract(a, deMath.scale(deMath.scale(b, dot(b, a)), 2));
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @param {number} c
 * @return {Array<number>}
 */
var refract = function(a, b, c) {
    var cosAngle = dot(b, a);
    var k = 1 - c * c * (1 - cosAngle * cosAngle);
    if (k < 0) {
        var res = new Array(a.length);
        return res.fill(0)
    } else
        return deMath.subtract(deMath.scale(a, c), deMath.scale(b, c * cosAngle + Math.sqrt(k)));
};

/**
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<number>}
 */
var cross = function(a, b) {
    if (a.length != 3 || b.length != 3)
        throw new Error('Arrays must have the size of 3');
    return [
        a[1] * b[2] - b[1] * a[2],
        a[2] * b[0] - b[2] * a[0],
        a[0] * b[1] - b[0] * a[1]];
};

var nop = function(v) {
    return v;
};

var selection = function(cond, a, b) {
    return cond ? a : b;
};

var boolNot = function(a) {
    return !a;
};

var bitwiseNot = function(a) {
    return ~a;
};

/**
 * @param {number} x
 * @return {number}
 */
var deFloatRadians = function(x) {
    return x * (Math.PI / 180.0);
};

/**
 * @param {number} x
 * @return {number}
 */
var deFloatDegrees = function(x) {
    return x * (180.0 / Math.PI);
};

/**
 * @param {number} x
 * @return {number}
 */
var deFloatExp2 = function(x) {
    return Math.exp(x * Math.LN2);
};

/**
 * @param {number} x
 * @return {number}
 */
var deFloatLog2 = function(x) {
    return Math.log(x) * Math.LOG2E;
};

/**
 * @param {number} x
 * @return {number}
 */
var deFloatRsq = function(x) {
    var s = Math.sqrt(x);
    return s == 0.0 ? 0.0 : (1.0 / s);
};

/**
 * @constructor
 * @param {boolean} low
 * @param {boolean} medium
 * @param {boolean} high
 */
es3fShaderOperatorTests.Precision = function(low, medium, high) {
    this.low = low;
    this.medium = medium;
    this.high = high;
};

/** @const */ es3fShaderOperatorTests.Precision.Low = new es3fShaderOperatorTests.Precision(true, false, false);
/** @const */ es3fShaderOperatorTests.Precision.Medium = new es3fShaderOperatorTests.Precision(false, true, false);
/** @const */ es3fShaderOperatorTests.Precision.High = new es3fShaderOperatorTests.Precision(false, false, true);
/** @const */ es3fShaderOperatorTests.Precision.LowMedium = new es3fShaderOperatorTests.Precision(true, true, false);
/** @const */ es3fShaderOperatorTests.Precision.MediumHigh = new es3fShaderOperatorTests.Precision(false, true, true);
/** @const */ es3fShaderOperatorTests.Precision.All = new es3fShaderOperatorTests.Precision(true, true, true);
/** @const */ es3fShaderOperatorTests.Precision.None = new es3fShaderOperatorTests.Precision(false, false, false);

/**
 * @enum
 */
es3fShaderOperatorTests.ValueType = {
    NONE: 0,
    FLOAT: (1 << 0), // float scalar
    FLOAT_VEC: (1 << 1), // float vector
    FLOAT_GENTYPE: (1 << 2), // float scalar/vector
    VEC3: (1 << 3), // vec3 only
    MATRIX: (1 << 4), // matrix
    BOOL: (1 << 5), // boolean scalar
    BOOL_VEC: (1 << 6), // boolean vector
    BOOL_GENTYPE: (1 << 7), // boolean scalar/vector
    INT: (1 << 8), // int scalar
    INT_VEC: (1 << 9), // int vector
    INT_GENTYPE: (1 << 10), // int scalar/vector
    UINT: (1 << 11), // uint scalar
    UINT_VEC: (1 << 12), // uint vector
    UINT_GENTYPE: (1 << 13) // uint scalar/vector
};

/**
 * @param {es3fShaderOperatorTests.ValueType} type
 * @return {boolean}
 */
es3fShaderOperatorTests.isBoolType = function(type) {
    return (type & (es3fShaderOperatorTests.ValueType.BOOL | es3fShaderOperatorTests.ValueType.BOOL_VEC | es3fShaderOperatorTests.ValueType.BOOL_GENTYPE)) != 0;
};

/**
 * @param {es3fShaderOperatorTests.ValueType} type
 * @return {boolean}
 */
es3fShaderOperatorTests.isIntType = function(type) {
    return (type & (es3fShaderOperatorTests.ValueType.INT | es3fShaderOperatorTests.ValueType.INT_VEC | es3fShaderOperatorTests.ValueType.INT_GENTYPE)) != 0;
};

/**
 * @param {es3fShaderOperatorTests.ValueType} type
 * @return {boolean}
 */
es3fShaderOperatorTests.isUintType = function(type) {
    return (type & (es3fShaderOperatorTests.ValueType.UINT | es3fShaderOperatorTests.ValueType.UINT_VEC | es3fShaderOperatorTests.ValueType.UINT_GENTYPE)) != 0;
};

/**
 * @param {es3fShaderOperatorTests.ValueType} type
 * @return {boolean}
 */
es3fShaderOperatorTests.isScalarType = function(type) {
    return type == es3fShaderOperatorTests.ValueType.FLOAT || type == es3fShaderOperatorTests.ValueType.BOOL || type == es3fShaderOperatorTests.ValueType.INT || type == es3fShaderOperatorTests.ValueType.UINT;
};

/**
 * @param {es3fShaderOperatorTests.ValueType} type
 * @return {boolean}
 */
es3fShaderOperatorTests.isFloatType = function(type) {
    return (type & (es3fShaderOperatorTests.ValueType.FLOAT | es3fShaderOperatorTests.ValueType.FLOAT_VEC | es3fShaderOperatorTests.ValueType.FLOAT_GENTYPE)) != 0;
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 * @param {gluShaderUtil.precision} uintPrecision
 * @return {number}
 */
es3fShaderOperatorTests.getGLSLUintMaxAsFloat = function(shaderType, uintPrecision) {
    switch (uintPrecision) {
        case gluShaderUtil.precision.PRECISION_LOWP:
            var intPrecisionGL = gl.LOW_INT;
            break;
        case gluShaderUtil.precision.PRECISION_MEDIUMP:
            var intPrecisionGL = gl.MEDIUM_INT;
            break;
        case gluShaderUtil.precision.PRECISION_HIGHP:
            var intPrecisionGL = gl.HIGH_INT;
            break;
        default:
            assertMsgOptions(false, 'Invalid shader type', false, false);
            var intPrecisionGL = 0;
    }

    switch (shaderType) {
        case gluShaderProgram.shaderType.VERTEX:
            var shaderTypeGL = gl.VERTEX_SHADER;
            break;
        case gluShaderProgram.shaderType.FRAGMENT:
            var shaderTypeGL = gl.FRAGMENT_SHADER;
            break;
        default:
            assertMsgOptions(false, 'Invalid shader type', false, false);
            var shaderTypeGL = 0;
    }

    /** @type {WebGLShaderPrecisionFormat } */ var sPrecision = gl.getShaderPrecisionFormat(shaderTypeGL, intPrecisionGL);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'glGetShaderPrecisionFormat failed', false, true);

    if (!deMath.deInBounds32(sPrecision.rangeMin, 8, 32))
        throw new Error('Out of range');

    var numBitsInType = sPrecision.rangeMin + 1;
    return Math.pow(2, numBitsInType) - 1;
};

/**
 * @enum
 */
es3fShaderOperatorTests.OperationType = {
    FUNCTION: 0,
    OPERATOR: 1,
    SIDE_EFFECT_OPERATOR: 2 // Test the side-effect (as opposed to the result) of a side-effect operator.
};

/**
 * swizzling indices for assigning the tested function output to the correct color channel
 */
es3fShaderOperatorTests.outIndices = [];
es3fShaderOperatorTests.outIndices[1] = [0];
es3fShaderOperatorTests.outIndices[2] = [1, 2];
es3fShaderOperatorTests.outIndices[3] = [0, 1, 2];
es3fShaderOperatorTests.outIndices[4] = [0, 1, 2, 3];

var convert = function(input, dataType) {
    switch (dataType) {
        case gluShaderUtil.DataType.INT:
            if (input instanceof Array) {
                var ret = [];
                for (var i = 0; i < input.length; i++)
                    ret[i] = deMath.intCast(input[i]);
                return ret;
            }
            return deMath.intCast(input);
        case gluShaderUtil.DataType.UINT:
            if (input instanceof Array) {
                var ret = [];
                for (var i = 0; i < input.length; i++)
                    ret[i] = deMath.uintCast(input[i]);
                return ret;
            }
            return deMath.uintCast(input);
        case gluShaderUtil.DataType.BOOL:
            if (input instanceof Array) {
                var ret = [];
                for (var i = 0; i < input.length; i++)
                    ret[i] = input[i] > 0 ? 1 : 0;
                return ret;
            }
            return input > 0 ? 1 : 0;

    }
    return input;
};

/**
 * Generate unary functions which have the same input and return type
 * @param {function(number): number} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.unaryGenTypeFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input) {
        if (input instanceof Array) {
            var len = input.length;
            var indices = es3fShaderOperatorTests.outIndices[len];
            for (var i = 0; i < input.length; i++)
                 output[indices[i]] = convert(func(convert(input[i], dataTypeIn)), dataTypeOut);
        } else
            output[0] = convert(func(convert(input, dataTypeIn)), dataTypeOut);
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, c.in_[0][2]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0])); };
    return functions;
};

/**
 * Generate unary functions which have the same input and return type
 * @param {function(Array<number>): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.unaryArrayFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input) {
        var len = input.length;
        var indices = es3fShaderOperatorTests.outIndices[len];
        var value = convert(func(convert(input, dataTypeIn)), dataTypeOut);
        for (var i = 0; i < input.length; i++)
             output[indices[i]] = value[i];
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, [c.in_[0][2]]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0])); };
    return functions;
};

/**
 * Generate unary functions which always have scalar return type
 * @param {function(Array<number>): number} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.unaryScalarGenTypeFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input) {
        output[0] = convert(func(convert(input, dataTypeIn)), dataTypeOut);
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, [c.in_[0][2]]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0])); };
    return functions;
};

/**
 * Generate unary functions which always have bolean return type
 * @param {function(Array<number>): boolean} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.unaryBooleanGenTypeFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input) {
        output[0] = convert(func(convert(input, dataTypeIn)), dataTypeOut);
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, [c.in_[0][2] > 0.0]); };
    functions.vec2 = function(c) { run(c.color, func, greaterThanVec(deMath.swizzle(c.in_[0], [3, 1]), [0, 0])); };
    functions.vec3 = function(c) { run(c.color, func, greaterThanVec(deMath.swizzle(c.in_[0], [2, 0, 1]), [0, 0, 0])); };
    functions.vec4 = function(c) { run(c.color, func, greaterThanVec(deMath.swizzle(c.in_[0], [1, 2, 3, 0]), [0, 0, 0, 0])); };
    return functions;
};

/**
 * Generate binary functions which have the same input and return type
 * @param {function(number, number): number} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.binaryGenTypeFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input1, input2) {
        if (input1 instanceof Array) {
            var len = input1.length;
            var indices = es3fShaderOperatorTests.outIndices[len];
            for (var i = 0; i < input1.length; i++)
                 output[indices[i]] = convert(func(convert(input1[i], dataTypeIn), convert(input2[i], dataTypeIn)), dataTypeOut);
        } else {
            var value = convert(func(convert(input1, dataTypeIn), convert(input2, dataTypeIn)), dataTypeOut);
            output[0] = value;
        }
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, c.in_[0][2], c.in_[1][0]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), deMath.swizzle(c.in_[1], [1, 0])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), deMath.swizzle(c.in_[1], [1, 2, 0])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), deMath.swizzle(c.in_[1], [3, 2, 1, 0])); };
    return functions;
};

/**
 * Generate binary functions which have the same input and return type
 * @param {function(number, number, number): number} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.ternaryGenTypeFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input1, input2, input3) {
        if (input1 instanceof Array) {
            var len = input1.length;
            var indices = es3fShaderOperatorTests.outIndices[len];
            for (var i = 0; i < input1.length; i++)
                 output[indices[i]] = convert(func(convert(input1[i], dataTypeIn), convert(input2[i], dataTypeIn), convert(input3[i], dataTypeIn)), dataTypeOut);
        } else {
            var value = convert(func(convert(input1, dataTypeIn), convert(input2, dataTypeIn), convert(input3, dataTypeIn)), dataTypeOut);
            output[0] = value;
        }
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, c.in_[0][2], c.in_[1][0], c.in_[2][1]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), deMath.swizzle(c.in_[1], [1, 0]), deMath.swizzle(c.in_[2], [2, 1])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), deMath.swizzle(c.in_[1], [1, 2, 0]), deMath.swizzle(c.in_[2], [3, 1, 2])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), deMath.swizzle(c.in_[1], [3, 2, 1, 0]), deMath.swizzle(c.in_[2], [0, 3, 2, 1])); };
    return functions;
};

/**
 * Generate binary functions which have the same input and return type
 * @param {function(Array<number>, Array<number>): number} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.binaryScalarGenTypeFuncs = function(func, dataTypeOut, dataTypeIn) {
    var run = function(output, func, input1, input2) {
        var value = convert(func(convert(input1, dataTypeIn), convert(input2, dataTypeIn)), dataTypeOut);
        output[0] = value;
    };

    var functions = {};
    functions.scalar = function(c) { run(c.color, func, [c.in_[0][2]], [c.in_[1][0]]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), deMath.swizzle(c.in_[1], [1, 0])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), deMath.swizzle(c.in_[1], [1, 2, 0])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), deMath.swizzle(c.in_[1], [3, 2, 1, 0])); };
    return functions;
};

/**
 * Generate (cond ? a : b) functions
 * @param {gluShaderUtil.DataType} dataType
 * Returns an array of functions, indexed by datatype size
 */
es3fShaderOperatorTests.selectionFuncs = function(dataType) {
    var run = function(output, input0, input1, input2) {
        var value = selection(input0, input1, input2);
        value = convert(value, dataType);
        if (value instanceof Array) {
            var len = value.length;
            var indices = es3fShaderOperatorTests.outIndices[len];
            for (var i = 0; i < len; i++)
                output[indices[i]] = value[i];
        } else
            output[0] = value;
    };

    var functions = [];
    functions[1] = function(c) { run(c.color, c.in_[0][2] > 0, c.in_[1][0], c.in_[2][1]); };
    functions[2] = function(c) { run(c.color, c.in_[0][2] > 0, deMath.swizzle(c.in_[1], [1, 0]), deMath.swizzle(c.in_[2], [2, 1])); };
    functions[3] = function(c) { run(c.color, c.in_[0][2] > 0, deMath.swizzle(c.in_[1], [1, 2, 0]), deMath.swizzle(c.in_[2], [3, 1, 2])); };
    functions[4] = function(c) { run(c.color, c.in_[0][2] > 0, deMath.swizzle(c.in_[1], [3, 2, 1, 0]), deMath.swizzle(c.in_[2], [0, 3, 2, 1])); };
    return functions;
};

var cp = function(dst, src) {
    var len = src.length;
    var indices = es3fShaderOperatorTests.outIndices[len];
    for (var i = 0; i < len; i++)
        dst[indices[i]] = src[i];
};

/**
 * Generate binary functions of form: vec = func(scalar, vec)
 * @param {function(number, Array<number>): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.binaryScalarVecFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(number, Array<number>): Array<number>} func
     * @param {number} input1
     * @param {Array<number>} input2
     */
    var run = function(output, func, input1, input2) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var value = func(in1, in2);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.vec2 = function(c) { run(c.color, func, c.in_[0][2], deMath.swizzle(c.in_[1], [1, 0])); };
    functions.vec3 = function(c) { run(c.color, func, c.in_[0][2], deMath.swizzle(c.in_[1], [1, 2, 0])); };
    functions.vec4 = function(c) { run(c.color, func, c.in_[0][2], deMath.swizzle(c.in_[1], [3, 2, 1, 0])); };
    return functions;
};

/**
 * Generate binary functions of form: vec = func(vec, scalar)
 * @param {function(Array<number>, number): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.binaryVecScalarFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(Array<number>, number): Array<number>} func
     * @param {Array<number>} input1
     * @param {number} input2
     */
    var run = function(output, func, input1, input2) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var value = func(in1, in2);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), c.in_[1][0]); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), c.in_[1][0]); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), c.in_[1][0]); };
    return functions;
};

/**
 * Generate binary functions of form: vec = func(vec, vec, scalar)
 * @param {function(Array<number>, Array<number>, number): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.ternaryVecVecScalarFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(Array<number>, Array<number>, number): Array<number>} func
     * @param {Array<number>} input1
     * @param {Array<number>} input2
     * @param {number} input3
     */
    var run = function(output, func, input1, input2, input3) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var in3 = convert(input3, dataTypeIn);
        var value = func(in1, in2, in3);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.scalar = function(c) { run(c.color, func, [c.in_[0][2]], [c.in_[1][0]], c.in_[2][1]); };
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), deMath.swizzle(c.in_[1], [1, 0]), c.in_[2][1]); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), deMath.swizzle(c.in_[1], [1, 2, 0]), c.in_[2][1]); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), deMath.swizzle(c.in_[1], [3, 2, 1, 0]), c.in_[2][1]); };
    return functions;
};

/**
 * Generate binary functions of form: vec = func(vec, scalar, scalar)
 * @param {function(Array<number>, number, number): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.ternaryVecScalarScalarFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(Array<number>, number, number): Array<number>} func
     * @param {Array<number>} input1
     * @param {number} input2
     * @param {number} input3
     */
    var run = function(output, func, input1, input2, input3) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var in3 = convert(input3, dataTypeIn);
        var value = func(in1, in2, in3);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), c.in_[1][0], c.in_[2][1]); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), c.in_[1][0], c.in_[2][1]); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), c.in_[1][0], c.in_[2][1]); };
    return functions;
};

/**
 * Generate binary functions of form: vec = func(scalar, scalar, vec)
 * @param {function(number, number, Array<number>): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.ternaryScalarScalarVecFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(number, number, Array<number>): Array<number>} func
     * @param {number} input1
     * @param {number} input2
     * @param {Array<number>} input3
     */
    var run = function(output, func, input1, input2, input3) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var in3 = convert(input3, dataTypeIn);
        var value = func(in1, in2, in3);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.vec2 = function(c) { run(c.color, func, c.in_[0][2], c.in_[1][0], deMath.swizzle(c.in_[2], [2, 1])); };
    functions.vec3 = function(c) { run(c.color, func, c.in_[0][2], c.in_[1][0], deMath.swizzle(c.in_[2], [3, 1, 2])); };
    functions.vec4 = function(c) { run(c.color, func, c.in_[0][2], c.in_[1][0], deMath.swizzle(c.in_[2], [0, 3, 2, 1])); };
    return functions;
};

/**
 * Generate binary functions of form: vec = func(vec, vec, vec)
 * @param {function(Array<number>, Array<number>, Array<number>): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.ternaryVecVecVecFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(Array<number>, Array<number>, Array<number>): Array<number>} func
     * @param {Array<number>} input1
     * @param {Array<number>} input2
     * @param {Array<number>} input3
     */
    var run = function(output, func, input1, input2, input3) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var in3 = convert(input3, dataTypeIn);
        var value = func(in1, in2, in3);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), deMath.swizzle(c.in_[1], [1, 0]), deMath.swizzle(c.in_[2], [2, 1])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), deMath.swizzle(c.in_[1], [1, 2, 0]), deMath.swizzle(c.in_[2], [3, 1, 2])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), deMath.swizzle(c.in_[1], [3, 2, 1, 0]), deMath.swizzle(c.in_[2], [0, 3, 2, 1])); };
    return functions;
};

/**
 * Generate binary functions of form: vec = func(vec, vec)
 * @param {function(Array<number>, Array<number>): Array<number>} func
 * @param {gluShaderUtil.DataType=} dataTypeIn
 * @param {gluShaderUtil.DataType=} dataTypeOut
 */
es3fShaderOperatorTests.binaryVecVecFuncs = function(func, dataTypeOut, dataTypeIn) {
    /**
     * @param {function(Array<number>, Array<number>): Array<number>} func
     * @param {Array<number>} input1
     * @param {Array<number>} input2
     */
    var run = function(output, func, input1, input2) {
        var in1 = convert(input1, dataTypeIn);
        var in2 = convert(input2, dataTypeIn);
        var value = func(in1, in2);
        value = convert(value, dataTypeOut);
        cp(output, value);
    };
    var functions = {};
    functions.vec2 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [3, 1]), deMath.swizzle(c.in_[1], [1, 0])); };
    functions.vec3 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [2, 0, 1]), deMath.swizzle(c.in_[1], [1, 2, 0])); };
    functions.vec4 = function(c) { run(c.color, func, deMath.swizzle(c.in_[0], [1, 2, 3, 0]), deMath.swizzle(c.in_[1], [3, 2, 1, 0])); };
    return functions;
};

/**
 * @constructor
 * @param {es3fShaderOperatorTests.ValueType} valueType
 * @param {es3fShaderOperatorTests.FloatScalar} rangeMin
 * @param {es3fShaderOperatorTests.FloatScalar} rangeMax
 */
es3fShaderOperatorTests.Value = function(valueType, rangeMin, rangeMax) {
    this.valueType = valueType;
    this.rangeMin = rangeMin;
    this.rangeMax = rangeMax;
};

/**
 * @enum
 */
es3fShaderOperatorTests.Symbol = {
    SYMBOL_LOWP_UINT_MAX: 0,
    SYMBOL_MEDIUMP_UINT_MAX: 1,

    SYMBOL_LOWP_UINT_MAX_RECIPROCAL: 2,
    SYMBOL_MEDIUMP_UINT_MAX_RECIPROCAL: 3,

    SYMBOL_ONE_MINUS_UINT32MAX_DIV_LOWP_UINT_MAX: 4,
    SYMBOL_ONE_MINUS_UINT32MAX_DIV_MEDIUMP_UINT_MAX: 5

};

/**
 * @constructor
 * @param {number|es3fShaderOperatorTests.Symbol} value
 * @param {boolean=} isSymbol
 */
es3fShaderOperatorTests.FloatScalar = function(value, isSymbol) {
    if (isSymbol)
        this.symbol = /** @type {es3fShaderOperatorTests.Symbol} */ (value);
    else
        this.constant = /** @type {number} */ (value);
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 * @return {number}
 */
es3fShaderOperatorTests.FloatScalar.prototype.getValue = function(shaderType) {
    if (this.constant !== undefined)
        return this.constant;
    else
        switch (this.symbol) {
            case es3fShaderOperatorTests.Symbol.SYMBOL_LOWP_UINT_MAX:
                return es3fShaderOperatorTests.getGLSLUintMaxAsFloat(shaderType, gluShaderUtil.precision.PRECISION_LOWP);
            case es3fShaderOperatorTests.Symbol.SYMBOL_MEDIUMP_UINT_MAX:
                return es3fShaderOperatorTests.getGLSLUintMaxAsFloat(shaderType, gluShaderUtil.precision.PRECISION_MEDIUMP);

            case es3fShaderOperatorTests.Symbol.SYMBOL_LOWP_UINT_MAX_RECIPROCAL:
                return 1.0 / es3fShaderOperatorTests.getGLSLUintMaxAsFloat(shaderType, gluShaderUtil.precision.PRECISION_LOWP);
            case es3fShaderOperatorTests.Symbol.SYMBOL_MEDIUMP_UINT_MAX_RECIPROCAL:
                return 1.0 / es3fShaderOperatorTests.getGLSLUintMaxAsFloat(shaderType, gluShaderUtil.precision.PRECISION_MEDIUMP);

            case es3fShaderOperatorTests.Symbol.SYMBOL_ONE_MINUS_UINT32MAX_DIV_LOWP_UINT_MAX:
                return 1.0 - 0xFFFFFFFF / es3fShaderOperatorTests.getGLSLUintMaxAsFloat(shaderType, gluShaderUtil.precision.PRECISION_LOWP);
            case es3fShaderOperatorTests.Symbol.SYMBOL_ONE_MINUS_UINT32MAX_DIV_MEDIUMP_UINT_MAX:
                return 1.0 - 0xFFFFFFFF / es3fShaderOperatorTests.getGLSLUintMaxAsFloat(shaderType, gluShaderUtil.precision.PRECISION_MEDIUMP);

            default:
                assertMsgOptions(false, 'Invalid shader type', false, false);
                return 0.0;
        }
};

/**
 * @constructor
 * @param {gluShaderUtil.DataType=} type
 * @param {es3fShaderOperatorTests.FloatScalar=} rangeMin
 * @param {es3fShaderOperatorTests.FloatScalar=} rangeMax
 */
es3fShaderOperatorTests.ShaderValue = function(type, rangeMin, rangeMax) {
    this.type = type || gluShaderUtil.DataType.INVALID;
    this.rangeMin = rangeMin || new es3fShaderOperatorTests.FloatScalar(0);
    this.rangeMax = rangeMax || new es3fShaderOperatorTests.FloatScalar(0);
};

/**
 * @constructor
 */
es3fShaderOperatorTests.ShaderDataSpec = function() {
    /** @type {es3fShaderOperatorTests.FloatScalar} */ this.resultScale = new es3fShaderOperatorTests.FloatScalar(1);
    /** @type {es3fShaderOperatorTests.FloatScalar} */ this.resultBias = new es3fShaderOperatorTests.FloatScalar(0);
    /** @type {es3fShaderOperatorTests.FloatScalar} */ this.referenceScale = new es3fShaderOperatorTests.FloatScalar(1);
    /** @type {es3fShaderOperatorTests.FloatScalar} */ this.referenceBias = new es3fShaderOperatorTests.FloatScalar(0);
    /** @type {es3fShaderOperatorTests.Precision} */ this.precision = es3fShaderOperatorTests.Precision.None;
    /** @type {gluShaderUtil.DataType} */ this.output;
    /** @type {number} */ this.numInputs = 0;
    /** @type {Array<es3fShaderOperatorTests.ShaderValue>}*/ this.inputs = [];
    for (var i = 0; i < es3fShaderOperatorTests.MAX_INPUTS; i++)
        this.inputs[i] = new es3fShaderOperatorTests.ShaderValue();
};

/**
 * @constructor
 * @struct
 * @param {string} caseName
 * @param {string} shaderFuncName
 * @param {es3fShaderOperatorTests.ValueType} outValue
 * @param {Array<es3fShaderOperatorTests.Value>} inputs
 * @param {es3fShaderOperatorTests.FloatScalar} resultScale
 * @param {es3fShaderOperatorTests.FloatScalar} resultBias
 * @param {es3fShaderOperatorTests.FloatScalar} referenceScale
 * @param {es3fShaderOperatorTests.FloatScalar} referenceBias
 * @param {gluShaderUtil.precision} precision
 * @param {*} functions
 * @param {es3fShaderOperatorTests.OperationType=} type
 * @param {boolean=} isUnaryPrefix
 */
es3fShaderOperatorTests.BuiltinFuncInfo = function(caseName, shaderFuncName, outValue, inputs, resultScale, resultBias, referenceScale, referenceBias, precision, functions, type, isUnaryPrefix) {
    this.caseName = caseName;
    this.shaderFuncName = shaderFuncName;
    this.outValue = outValue;
    this.inputs = inputs;
    this.resultScale = resultScale;
    this.resultBias = resultBias;
    this.referenceScale = referenceScale;
    this.referenceBias = referenceBias;
    this.precision = precision;
    this.evalFunctions = functions;
    this.type = type || es3fShaderOperatorTests.OperationType.FUNCTION;
    this.isUnaryPrefix = isUnaryPrefix === undefined ? true : isUnaryPrefix;
};

es3fShaderOperatorTests.builtinOperInfo = function(caseName, shaderFuncName, outValue, inputs, scale, bias, precision, functions) {
    return new es3fShaderOperatorTests.BuiltinFuncInfo(caseName,
                                                   shaderFuncName,
                                                   outValue,
                                                   inputs,
                                                   scale,
                                                   bias,
                                                   scale,
                                                   bias,
                                                   precision,
                                                   functions,
                                                   es3fShaderOperatorTests.OperationType.OPERATOR);
};

es3fShaderOperatorTests.builtinFunctionInfo = function(caseName, shaderFuncName, outValue, inputs, scale, bias, precision, functions) {
    return new es3fShaderOperatorTests.BuiltinFuncInfo(caseName,
                                                   shaderFuncName,
                                                   outValue,
                                                   inputs,
                                                   scale,
                                                   bias,
                                                   scale,
                                                   bias,
                                                   precision,
                                                   functions,
                                                   es3fShaderOperatorTests.OperationType.FUNCTION);
};

es3fShaderOperatorTests.builtinSideEffOperInfo = function(caseName, shaderFuncName, outValue, inputs, scale, bias, precision, functions) {
    return new es3fShaderOperatorTests.BuiltinFuncInfo(caseName,
                                                   shaderFuncName,
                                                   outValue,
                                                   inputs,
                                                   scale,
                                                   bias,
                                                   scale,
                                                   bias,
                                                   precision,
                                                   functions,
                                                   es3fShaderOperatorTests.OperationType.SIDE_EFFECT_OPERATOR);
};

es3fShaderOperatorTests.builtinOperInfoSeparateRefScaleBias = function(caseName, shaderFuncName, outValue, inputs, scale, bias, precision, functions, referenceScale, referenceBias) {
    return new es3fShaderOperatorTests.BuiltinFuncInfo(caseName,
                                                    shaderFuncName,
                                                    outValue,
                                                    inputs,
                                                    scale,
                                                    bias,
                                                    referenceScale,
                                                    referenceBias,
                                                    precision,
                                                    functions,
                                                    es3fShaderOperatorTests.OperationType.OPERATOR);
};

es3fShaderOperatorTests.BuiltinPostSideEffOperInfo = function(caseName, shaderFuncName, outValue, inputs, scale, bias, precision, functions) {
    return new es3fShaderOperatorTests.BuiltinFuncInfo(caseName,
                                                    shaderFuncName,
                                                    outValue,
                                                    inputs,
                                                    scale,
                                                    bias,
                                                    scale,
                                                    bias,
                                                    precision,
                                                    functions,
                                                    es3fShaderOperatorTests.OperationType.SIDE_EFFECT_OPERATOR,
                                                    false);
};
es3fShaderOperatorTests.BuiltinPostOperInfo = function(caseName, shaderFuncName, outValue, inputs, scale, bias, precision, functions) {
    return new es3fShaderOperatorTests.BuiltinFuncInfo(caseName,
                                                    shaderFuncName,
                                                    outValue,
                                                    inputs,
                                                    scale,
                                                    bias,
                                                    scale,
                                                    bias,
                                                    precision,
                                                    functions,
                                                    es3fShaderOperatorTests.OperationType.OPERATOR,
                                                    false);
};

/**
 * @constructor
 * @param {string} name
 * @param {string} description
 */
es3fShaderOperatorTests.BuiltinFuncGroup = function(name, description) {
    this.name = name;
    this.description = description;
    this.funcInfos = [];
};

es3fShaderOperatorTests.BuiltinFuncGroup.prototype.push = function(a) { this.funcInfos.push(a); };

var s_inSwizzles = [

    ['z', 'wy', 'zxy', 'yzwx'],
    ['x', 'yx', 'yzx', 'wzyx'],
    ['y', 'zy', 'wyz', 'xwzy']
];

var s_outSwizzles = ['x', 'yz', 'xyz', 'xyzw'];

var s_outSwizzleChannelMasks = [
    [true, false, false, false],
    [false, true, true, false],
    [true, true, true, false],
    [true, true, true, true]
];

/**
 * @constructor
 * @extends {glsShaderRenderCase.ShaderEvaluator}
 * @param {gluShaderProgram.shaderType} shaderType
 * @param {glsShaderRenderCase.ShaderEvalFunc} evalFunc
 * @param {es3fShaderOperatorTests.FloatScalar} scale
 * @param {es3fShaderOperatorTests.FloatScalar} bias
 * @param {number} resultScalarSize
 */
es3fShaderOperatorTests.OperatorShaderEvaluator = function(shaderType, evalFunc, scale, bias, resultScalarSize) {
    glsShaderRenderCase.ShaderEvaluator.call(this, evalFunc);
    this.m_shaderType = shaderType;
    this.m_scale = scale;
    this.m_bias = bias;
    this.m_resultScalarSize = resultScalarSize;
    this.m_areScaleAndBiasEvaluated = false;
};

setParentClass(es3fShaderOperatorTests.OperatorShaderEvaluator, glsShaderRenderCase.ShaderEvaluator);

es3fShaderOperatorTests.OperatorShaderEvaluator.prototype.evaluate = function(ctx) {
    this.m_evalFunc(ctx);

    if (!this.m_areScaleAndBiasEvaluated) {
        this.m_evaluatedScale = this.m_scale.getValue(this.m_shaderType);
        this.m_evaluatedBias = this.m_bias.getValue(this.m_shaderType);
        this.m_areScaleAndBiasEvaluated = true;
    }

    for (var i = 0; i < 4; i++)
        if (s_outSwizzleChannelMasks[this.m_resultScalarSize - 1][i])
            ctx.color[i] = ctx.color[i] * this.m_evaluatedScale + this.m_evaluatedBias;
};

/**
 * @constructor
 * @extends {glsShaderRenderCase.ShaderRenderCase}
 * @param {string} caseName
 * @param {string} description
 * @param {boolean} isVertexCase
 * @param {glsShaderRenderCase.ShaderEvalFunc} evalFunc
 * @param {string} shaderOp
 * @param {es3fShaderOperatorTests.ShaderDataSpec} spec
 */
es3fShaderOperatorTests.ShaderOperatorCase = function(caseName, description, isVertexCase, evalFunc, shaderOp, spec) {
    glsShaderRenderCase.ShaderRenderCase.call(this, caseName, description, isVertexCase, evalFunc);
    this.m_spec = spec;
    this.m_shaderOp = shaderOp;
    var shaderType = isVertexCase ? gluShaderProgram.shaderType.VERTEX : gluShaderProgram.shaderType.FRAGMENT;
    this.m_evaluator = new es3fShaderOperatorTests.OperatorShaderEvaluator(shaderType,
                                                                           evalFunc,
                                                                           spec.referenceScale,
                                                                           spec.referenceBias,
                                                                           gluShaderUtil.getDataTypeScalarSize(spec.output));
};

setParentClass(es3fShaderOperatorTests.ShaderOperatorCase, glsShaderRenderCase.ShaderRenderCase);

es3fShaderOperatorTests.ShaderOperatorCase.prototype.setupShaderData = function() {
    var shaderType = this.m_isVertexCase ? gluShaderProgram.shaderType.VERTEX : gluShaderProgram.shaderType.FRAGMENT;
    var precision = this.m_spec.precision !== undefined ? gluShaderUtil.getPrecisionName(this.m_spec.precision) : null;
    var inputPrecision = [];
    var sources = [];
    sources[0] = ''; //vertex
    sources[1] = ''; //fragment
    var vtx = 0;
    var frag = 1;
    var op = this.m_isVertexCase ? vtx : frag;

    sources[vtx] += '#version 300 es\n';
    sources[frag] += '#version 300 es\n';

    // Compute precision for inputs.
    for (var i = 0; i < this.m_spec.numInputs; i++) {
        var isBoolVal = gluShaderUtil.isDataTypeBoolOrBVec(this.m_spec.inputs[i].type);
        var isIntVal = gluShaderUtil.isDataTypeIntOrIVec(this.m_spec.inputs[i].type);
        var isUintVal = gluShaderUtil.isDataTypeUintOrUVec(this.m_spec.inputs[i].type);
        // \note Mediump interpolators are used for booleans, and highp for integers.
        var prec = isBoolVal ? gluShaderUtil.precision.PRECISION_MEDIUMP :
                                isIntVal || isUintVal ? gluShaderUtil.precision.PRECISION_HIGHP :
                                this.m_spec.precision;
        inputPrecision[i] = gluShaderUtil.getPrecisionName(prec);
    }

    // Attributes.
    sources[vtx] += 'in highp vec4 a_position;\n';
    for (var i = 0; i < this.m_spec.numInputs; i++)
        sources[vtx] += 'in ' + inputPrecision[i] + ' vec4 a_in' + i + ';\n';

    // Color output.
    sources[frag] += 'layout(location = 0) out mediump vec4 o_color;\n';

    if (this.m_isVertexCase) {
        sources[vtx] += 'out mediump vec4 v_color;\n';
        sources[frag] += 'in mediump vec4 v_color;\n';
    } else{
        for (var i = 0; i < this.m_spec.numInputs; i++) {
            sources[vtx] += 'out ' + inputPrecision[i] + ' vec4 v_in' + i + ';\n';
            sources[frag] += 'in ' + inputPrecision[i] + ' vec4 v_in' + i + ';\n';
        }
    }

    sources[vtx] += '\n';
    sources[vtx] += 'void main()\n';
    sources[vtx] += '{\n';
    sources[vtx] += ' gl_Position = a_position;\n';

    sources[frag] += '\n';
    sources[frag] += 'void main()\n';
    sources[frag] += '{\n';

    // Expression inputs.
    var prefix = this.m_isVertexCase ? 'a_' : 'v_';
    for (var i = 0; i < this.m_spec.numInputs; i++) {
        var inType = this.m_spec.inputs[i].type;
        var inSize = gluShaderUtil.getDataTypeScalarSize(inType);
        var isInt = gluShaderUtil.isDataTypeIntOrIVec(inType);
        var isUint = gluShaderUtil.isDataTypeUintOrUVec(inType);
        var isBool = gluShaderUtil.isDataTypeBoolOrBVec(inType);
        var typeName = gluShaderUtil.getDataTypeName(inType);
        var swizzle = s_inSwizzles[i][inSize - 1];

        sources[op] += '\t';
        if (precision && !isBool) sources[op] += precision + ' ';

        sources[op] += typeName + ' in' + i + ' = ';

        if (isBool) {
            if (inSize == 1) sources[op] += '(';
            else sources[op] += 'greaterThan(';
        } else if (isInt || isUint)
            sources[op] += typeName + '(';

        sources[op] += prefix + 'in' + i + '.' + swizzle;

        if (isBool) {
            if (inSize == 1) sources[op] += ' > 0.0)';
            else sources[op] += ', vec' + inSize + '(0.0))';
        } else if (isInt || isUint)
            sources[op] += ')';

        sources[op] += ';\n';
    }

    // Result variable.
    var outTypeName = gluShaderUtil.getDataTypeName(this.m_spec.output);
    var isBoolOut = gluShaderUtil.isDataTypeBoolOrBVec(this.m_spec.output);

    sources[op] += '\t';
    if (precision && !isBoolOut) sources[op] += precision + ' ';
    sources[op] += outTypeName + ' res = ' + outTypeName + '(0.0);\n\n';


    // Expression.
    sources[op] += '\t' + this.m_shaderOp + '\n\n';

    // Convert to color.
    var isResFloatVec = gluShaderUtil.isDataTypeFloatOrVec(this.m_spec.output);
    var outScalarSize = gluShaderUtil.getDataTypeScalarSize(this.m_spec.output);

    sources[op] += '\thighp vec4 color = vec4(0.0, 0.0, 0.0, 1.0);\n';
    sources[op] += '\tcolor.' + s_outSwizzles[outScalarSize - 1] + ' = ';

    if (!isResFloatVec && outScalarSize == 1)
        sources[op] += 'float(res)';
    else if (!isResFloatVec)
        sources[op] += 'vec' + outScalarSize + '(res)';
    else
        sources[op] += 'res';

    sources[op] += ';\n';

    // Scale & bias.
    var resultScale = this.m_spec.resultScale.getValue(shaderType);
    var resultBias = this.m_spec.resultBias.getValue(shaderType);
    if ((resultScale != 1.0) || (resultBias != 0.0)) {
        sources[op] += '\tcolor = color';
        if (resultScale != 1.0) sources[op] += ' * ' + es3fShaderOperatorTests.twoValuedVec4(resultScale.toString(10), '1.0', s_outSwizzleChannelMasks[outScalarSize - 1]);
        if (resultBias != 0.0) sources[op] += ' + ' + es3fShaderOperatorTests.twoValuedVec4(resultBias.toString(10), '0.0', s_outSwizzleChannelMasks[outScalarSize - 1]);
        sources[op] += ';\n';
    }

    // ..
    if (this.m_isVertexCase) {
        sources[vtx] += ' v_color = color;\n';
        sources[frag] += ' o_color = v_color;\n';
    } else{
        for (var i = 0; i < this.m_spec.numInputs; i++)
        sources[vtx] += ' v_in' + i + ' = a_in' + i + ';\n';
        sources[frag] += ' o_color = color;\n';
    }

    sources[vtx] += '}\n';
    sources[frag] += '}\n';

    this.m_vertShaderSource = sources[vtx];
    this.m_fragShaderSource = sources[frag];

    // Setup the user attributes.
    this.m_userAttribTransforms = [];
    for (var inputNdx = 0; inputNdx < this.m_spec.numInputs; inputNdx++) {
        var v = this.m_spec.inputs[inputNdx];

        var rangeMin = v.rangeMin.getValue(shaderType);
        var rangeMax = v.rangeMax.getValue(shaderType);
        var scale = rangeMax - rangeMin;
        var minBias = rangeMin;
        var maxBias = rangeMax;
        var attribMatrix = new tcuMatrix.Matrix(4, 4);

        for (var rowNdx = 0; rowNdx < 4; rowNdx++) {
            var row;

            switch ((rowNdx + inputNdx) % 4) {
                case 0: row = [scale, 0.0, 0.0, minBias]; break;
                case 1: row = [0.0, scale, 0.0, minBias]; break;
                case 2: row = [-scale, 0.0, 0.0, maxBias]; break;
                case 3: row = [0.0, -scale, 0.0, maxBias]; break;
                default: throw new Error('Invalid row index');
            }

            attribMatrix.setRow(rowNdx, row);
        }

        this.m_userAttribTransforms[inputNdx] = attribMatrix;
    }

};

// Reference functions for specific sequence operations for the sequence operator tests.
/**
 * Reference for expression "in0, in2 + in1, in1 + in0"
 * @param {Array<number>} in0 Vec4
 * @param {Array<number>} in1 Vec4
 * @param {Array<number>} in2 Vec4
 * @return {Array<number>} Vec4
 */
es3fShaderOperatorTests.sequenceNoSideEffCase0 = function(in0, in1, in2) {
    return deMath.add(in1, in0);
};

/**
 * Reference for expression "in0, in2 + in1, in1 + in0"
 * @param {number} in0 float
 * @param {number} in1 deUint32
 * @param {number} in2 float
 * @return {number} deUint32
 */
es3fShaderOperatorTests.sequenceNoSideEffCase1 = function(in0, in1, in2) {
    in1 = convert(in1, gluShaderUtil.DataType.UINT);
    return convert(in1 + in1, gluShaderUtil.DataType.UINT);
};

/**
 * Reference for expression "in0 && in1, in0, ivec2(vec2(in0) + in2)"
 * @param {boolean} in0
 * @param {boolean} in1
 * @param {Array<number>} in2 Vec2
 * @return {Array<number>} IVec2
 */
es3fShaderOperatorTests.sequenceNoSideEffCase2 = function(in0, in1, in2) {
    in0 = convert(in0, gluShaderUtil.DataType.BOOL);
    return convert([in0 + in2[0], in0 + in2[1]], gluShaderUtil.DataType.INT);
};

/**
 * Reference for expression "in0 + vec4(in1), in2, in1"
 * @param {Array<number>} in0 Vec4
 * @param {Array<number>} in1 IVec4
 * @param {Array<boolean>} in2 BVec4
 * @return {Array<number>} IVec4
 */
es3fShaderOperatorTests.sequenceNoSideEffCase3 = function(in0, in1, in2) {
    return convert(in1, gluShaderUtil.DataType.INT);
};

/**
 * Reference for expression "in0++, in1 = in0 + in2, in2 = in1"
 * @param {Array<number>} in0 Vec4
 * @param {Array<number>} in1 Vec4
 * @param {Array<number>} in2 Vec4
 * @return {Array<number>} Vec4
 */
es3fShaderOperatorTests.sequenceSideEffCase0 = function(in0, in1, in2) {
    return deMath.add(deMath.add(in0, [1.0, 1.0, 1.0, 1.0]), in2);
};

/**
 * Reference for expression "in1++, in0 = float(in1), in1 = uint(in0 + in2)"
 * @param {number} in0 float
 * @param {number} in1 deUint32
 * @param {number} in2 float
 * @return {number} deUint32
 */
es3fShaderOperatorTests.sequenceSideEffCase1 = function(in0, in1, in2) {
    in1 = convert(in1, gluShaderUtil.DataType.UINT);
    return convert(in1 + 1.0 + in2, gluShaderUtil.DataType.UINT);
};

/**
 * Reference for expression "in1 = in0, in2++, in2 = in2 + vec2(in1), ivec2(in2)"
 * @param {boolean} in0
 * @param {boolean} in1
 * @param {Array<number>} in2 Vec2
 * @return {Array<number>} IVec2
 */
es3fShaderOperatorTests.sequenceSideEffCase2 = function(in0, in1, in2) {
    in0 = convert(in0, gluShaderUtil.DataType.BOOL);
    return convert(deMath.add(deMath.add(in2, [1.0, 1.0]), [in0, in0]), gluShaderUtil.DataType.INT);
};

/**
 * Reference for expression "in0 = in0 + vec4(in2), in1 = in1 + ivec4(in0), in1++"
 * @param {Array<number>} in0 Vec4
 * @param {Array<number>} in1 IVec4
 * @param {Array<boolean>} in2 BVec4
 * @return {Array<number>} IVec4
 */
es3fShaderOperatorTests.sequenceSideEffCase3 = function(in0, in1, in2) {
    in1 = convert(in1, gluShaderUtil.DataType.INT);
    in2 = convert(in2, gluShaderUtil.DataType.BOOL);
    in0 = deMath.add(in0, in2);
    in1 = deMath.add(in1, convert(in0, gluShaderUtil.DataType.INT));
    return in1;
};

// ShaderEvalFunc-type wrappers for the above functions.

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceNoSideEffCase0 = function(ctx) {
    ctx.color = es3fShaderOperatorTests.sequenceNoSideEffCase0(
        deMath.swizzle(ctx.in_[0], [1, 2, 3, 0]),
        deMath.swizzle(ctx.in_[1], [3, 2, 1, 0]),
        deMath.swizzle(ctx.in_[2], [0, 3, 2, 1])
    );
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceNoSideEffCase1 = function(ctx) {
    ctx.color[0] = es3fShaderOperatorTests.sequenceNoSideEffCase1(
        ctx.in_[0][2],
        ctx.in_[1][0],
        ctx.in_[2][1]
    );
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceNoSideEffCase2 = function(ctx) {
    /** @type {Array<number>} */ var result = es3fShaderOperatorTests.sequenceNoSideEffCase2(
        ctx.in_[0][2] > 0.0,
        ctx.in_[1][0] > 0.0,
        deMath.swizzle(ctx.in_[2], [2, 1])
    );
    ctx.color[1] = result[0];
    ctx.color[2] = result[1];
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceNoSideEffCase3 = function(ctx) {
    ctx.color = es3fShaderOperatorTests.sequenceNoSideEffCase3(
        deMath.swizzle(ctx.in_[0], [1, 2, 3, 0]),
        deMath.swizzle(ctx.in_[1], [3, 2, 1, 0]),
        deMath.greaterThan(deMath.swizzle(ctx.in_[2], [0, 3, 2, 1]), [0.0, 0.0, 0.0, 0.0])
    );
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceSideEffCase0 = function(ctx) {
    ctx.color = es3fShaderOperatorTests.sequenceSideEffCase0(
        deMath.swizzle(ctx.in_[0], [1, 2, 3, 0]),
        deMath.swizzle(ctx.in_[1], [3, 2, 1, 0]),
        deMath.swizzle(ctx.in_[2], [0, 3, 2, 1])
    );
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceSideEffCase1 = function(ctx) {
    ctx.color[0] = es3fShaderOperatorTests.sequenceSideEffCase1(
        ctx.in_[0][2],
        ctx.in_[1][0],
        ctx.in_[2][1]
    );
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceSideEffCase2 = function(ctx) {
    /** @type {Array<number>} */ var result = es3fShaderOperatorTests.sequenceSideEffCase2(
        ctx.in_[0][2] > 0.0,
        ctx.in_[1][0] > 0.0,
        deMath.swizzle(ctx.in_[2], [2, 1])
    );
    ctx.color[1] = result[0];
    ctx.color[2] = result[1];
};

/** @param {glsShaderRenderCase.ShaderEvalContext} ctx */
es3fShaderOperatorTests.evalSequenceSideEffCase3 = function(ctx) {
    ctx.color = es3fShaderOperatorTests.sequenceSideEffCase3(
        deMath.swizzle(ctx.in_[0], [1, 2, 3, 0]),
        deMath.swizzle(ctx.in_[1], [3, 2, 1, 0]),
        deMath.greaterThan(deMath.swizzle(ctx.in_[2], [0, 3, 2, 1]), [0.0, 0.0, 0.0, 0.0])
    );
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fShaderOperatorTests.ShaderOperatorTests = function() {
    tcuTestCase.DeqpTest.call(this, 'shaderop', 'Shader operators tests');
};

setParentClass(es3fShaderOperatorTests.ShaderOperatorTests, tcuTestCase.DeqpTest);

es3fShaderOperatorTests.ShaderOperatorTests.prototype.init = function() {
    var op = es3fShaderOperatorTests.builtinOperInfo;
    var side = es3fShaderOperatorTests.builtinSideEffOperInfo;
    var separate = es3fShaderOperatorTests.builtinOperInfoSeparateRefScaleBias;
    var postSide = es3fShaderOperatorTests.BuiltinPostSideEffOperInfo;
    var postOp = es3fShaderOperatorTests.BuiltinPostOperInfo;
    var all = es3fShaderOperatorTests.Precision.All;
    var highp = es3fShaderOperatorTests.Precision.High;
    var mediump = es3fShaderOperatorTests.Precision.Medium;
    var mediumhighp = es3fShaderOperatorTests.Precision.MediumHigh;
    var lowp = es3fShaderOperatorTests.Precision.Low;
    var na = es3fShaderOperatorTests.Precision.None;
    var GT = es3fShaderOperatorTests.ValueType.FLOAT_GENTYPE;
    var UGT = es3fShaderOperatorTests.ValueType.UINT_GENTYPE;
    var IGT = es3fShaderOperatorTests.ValueType.INT_GENTYPE;
    var BGT = es3fShaderOperatorTests.ValueType.BOOL_GENTYPE;
    var F = es3fShaderOperatorTests.ValueType.FLOAT;
    var I = es3fShaderOperatorTests.ValueType.INT;
    var U = es3fShaderOperatorTests.ValueType.UINT;
    var BV = es3fShaderOperatorTests.ValueType.BOOL_VEC;
    var FV = es3fShaderOperatorTests.ValueType.FLOAT_VEC;
    var IV = es3fShaderOperatorTests.ValueType.INT_VEC;
    var UV = es3fShaderOperatorTests.ValueType.UINT_VEC;
    var B = es3fShaderOperatorTests.ValueType.BOOL;
    var V3 = es3fShaderOperatorTests.ValueType.VEC3;
    var lUMax = es3fShaderOperatorTests.Symbol.SYMBOL_LOWP_UINT_MAX;
    var mUMax = es3fShaderOperatorTests.Symbol.SYMBOL_MEDIUMP_UINT_MAX;
    var lUMaxR = es3fShaderOperatorTests.Symbol.SYMBOL_LOWP_UINT_MAX_RECIPROCAL;
    var mUMaxR = es3fShaderOperatorTests.Symbol.SYMBOL_MEDIUMP_UINT_MAX_RECIPROCAL;
    var f = function(value) {
        return new es3fShaderOperatorTests.FloatScalar(value);
    };
    var s = function(value) {
        return new es3fShaderOperatorTests.FloatScalar(value, true);
    };
    var v = function(type, a, b) {
        return new es3fShaderOperatorTests.Value(type, f(a), f(b));
    };
    var v2 = function(type, a, b) {
        return new es3fShaderOperatorTests.Value(type, f(a), s(b));
    };
    var funcInfoGroups = [];
    var unary = new es3fShaderOperatorTests.BuiltinFuncGroup('unary_operator', 'Unary operator tests');
    funcInfoGroups.push(unary);

    unary.push(op('plus', '+', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop)));
    unary.push(op('plus', '+', IGT, [v(IGT, -5.0, 5.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(op('plus', '+', UGT, [v(UGT, 0.0, 2e2)], f(5e-3), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(op('minus', '-', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(negate)));
    unary.push(op('minus', '-', IGT, [v(IGT, -5.0, 5.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(negate, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(separate('minus', '-', UGT, [v2(UGT, 0.0, lUMax)], s(lUMaxR), f(0.0), lowp,
        es3fShaderOperatorTests.unaryGenTypeFuncs(negate, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT), s(lUMaxR),
        s(es3fShaderOperatorTests.Symbol.SYMBOL_ONE_MINUS_UINT32MAX_DIV_LOWP_UINT_MAX)));
    unary.push(separate('minus', '-', UGT, [v2(UGT, 0.0, mUMax)], s(mUMaxR), f(0.0), mediump,
        es3fShaderOperatorTests.unaryGenTypeFuncs(negate, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT), s(mUMaxR),
        s(es3fShaderOperatorTests.Symbol.SYMBOL_ONE_MINUS_UINT32MAX_DIV_MEDIUMP_UINT_MAX)));
    unary.push(op('minus', '-', UGT, [v(UGT, 0.0, 4e9)], f(2e-10), f(0.0), highp,
        es3fShaderOperatorTests.unaryGenTypeFuncs(negate, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(op('not', '!', B, [v(B, -1.0, 1.0)], f(1.0), f(0.0), na,{'scalar': es3fShaderOperatorTests.unaryGenTypeFuncs(boolNot, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL).scalar}));
    unary.push(op('bitwise_not', '~', IGT, [v(IGT, -1e5, 1e5)], f(5e-5), f(0.5), highp,
        es3fShaderOperatorTests.unaryGenTypeFuncs(bitwiseNot, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(op('bitwise_not', '~', UGT, [v(UGT, 0.0, 2e9)], f(2e-10), f(0.0), highp,
        es3fShaderOperatorTests.unaryGenTypeFuncs(bitwiseNot, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));

    // Pre/post incr/decr side effect cases.
    unary = new es3fShaderOperatorTests.BuiltinFuncGroup('unary_operator', 'Unary operator tests');
    funcInfoGroups.push(unary);

    unary.push(side('pre_increment_effect', '++', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne)));
    unary.push(side('pre_increment_effect', '++', IGT, [v(IGT, -6.0, 4.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(side('pre_increment_effect', '++', UGT, [v(UGT, 0.0, 9.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(side('pre_decrement_effect', '--', GT, [v(GT, -1.0, 1.0)], f(0.5), f(1.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne)));
    unary.push(side('pre_decrement_effect', '--', IGT, [v(IGT, -4.0, 6.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(side('pre_decrement_effect', '--', UGT, [v(UGT, 0.0, 10.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(postSide('post_increment_result', '++', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne)));
    unary.push(postSide('post_increment_result', '++', IGT, [v(IGT, -6.0, 4.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(postSide('post_increment_result', '++', UGT, [v(UGT, 0.0, 9.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(postSide('post_decrement_result', '--', GT, [v(GT, -1.0, 1.0)], f(0.5), f(1.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne)));
    unary.push(postSide('post_decrement_result', '--', IGT, [v(IGT, -4.0, 6.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(postSide('post_decrement_result', '--', UGT, [v(UGT, 1.0, 10.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));

    // Pre/post incr/decr result cases.
    unary = new es3fShaderOperatorTests.BuiltinFuncGroup('unary_operator', 'Unary operator tests');
    funcInfoGroups.push(unary);

    unary.push(op('pre_increment_result', '++', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne)));
    unary.push(op('pre_increment_result', '++', IGT, [v(IGT, -6.0, 4.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(op('pre_increment_result', '++', UGT, [v(UGT, 0.0, 9.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(addOne, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(op('pre_dencrement_result', '--', GT, [v(GT, -1.0, 1.0)], f(0.5), f(1.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne)));
    unary.push(op('pre_decrement_result', '--', IGT, [v(IGT, -4.0, 6.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(op('pre_decrement_result', '--', UGT, [v(UGT, 0.0, 10.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(subOne, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(postOp('post_increment_result', '++', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop)));
    unary.push(postOp('post_increment_result', '++', IGT, [v(IGT, -5.0, 5.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(postOp('post_increment_result', '++', UGT, [v(UGT, 0.0, 9.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    unary.push(postOp('post_decrement_result', '--', GT, [v(GT, -1.0, 1.0)], f(0.5), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop)));
    unary.push(postOp('post_decrement_result', '--', IGT, [v(IGT, -5.0, 5.0)], f(0.1), f(0.5), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    unary.push(postOp('post_decrement_result', '--', UGT, [v(UGT, 1.0, 10.0)], f(0.1), f(0.0), all,
        es3fShaderOperatorTests.unaryGenTypeFuncs(nop, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));

    var binary;

    // Normal binary operations and their corresponding assignment operations have lots in common; generate both in the following loop.
    // 0: normal op test, 1: assignment op side-effect test, 2: assignment op result test
    for (var binaryOperatorType = 0; binaryOperatorType <= 2; binaryOperatorType++) {
        var isNormalOp = binaryOperatorType == 0;
        var isAssignEff = binaryOperatorType == 1;
        var isAssignRes = binaryOperatorType == 2;

        var addName = isNormalOp ? 'add' : isAssignEff ? 'add_assign_effect' : 'add_assign_result';
        var subName = isNormalOp ? 'sub' : isAssignEff ? 'sub_assign_effect' : 'sub_assign_result';
        var mulName = isNormalOp ? 'mul' : isAssignEff ? 'mul_assign_effect' : 'mul_assign_result';
        var divName = isNormalOp ? 'div' : isAssignEff ? 'div_assign_effect' : 'div_assign_result';
        var modName = isNormalOp ? 'mod' : isAssignEff ? 'mod_assign_effect' : 'mod_assign_result';
        var andName = isNormalOp ? 'bitwise_and' : isAssignEff ? 'bitwise_and_assign_effect' : 'bitwise_and_assign_result';
        var orName = isNormalOp ? 'bitwise_or' : isAssignEff ? 'bitwise_or_assign_effect' : 'bitwise_or_assign_result';
        var xorName = isNormalOp ? 'bitwise_xor' : isAssignEff ? 'bitwise_xor_assign_effect' : 'bitwise_xor_assign_result';
        var leftShiftName = isNormalOp ? 'left_shift' : isAssignEff ? 'left_shift_assign_effect' : 'left_shift_assign_result';
        var rightShiftName = isNormalOp ? 'right_shift' : isAssignEff ? 'right_shift_assign_effect' : 'right_shift_assign_result';
        var addOp = isNormalOp ? '+' : '+=';
        var subOp = isNormalOp ? '-' : '-=';
        var mulOp = isNormalOp ? '*' : '*=';
        var divOp = isNormalOp ? '/' : '/=';
        var modOp = isNormalOp ? '%' : '%=';
        var andOp = isNormalOp ? '&' : '&=';
        var orOp = isNormalOp ? '|' : '|=';
        var xorOp = isNormalOp ? '^' : '^=';
        var leftShiftOp = isNormalOp ? '<<' : '<<=';
        var rightShiftOp = isNormalOp ? '>>' : '>>=';

        op = isAssignEff ? es3fShaderOperatorTests.builtinSideEffOperInfo : es3fShaderOperatorTests.builtinOperInfo;

        binary = new es3fShaderOperatorTests.BuiltinFuncGroup('binary_operator', 'Binary operator tests');
        funcInfoGroups.push(binary);

        // The add operator.

        binary.push(op(addName, addOp, GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryGenTypeFuncs(add)));
        binary.push(op(addName, addOp, IGT, [v(IGT, -4.0, 6.0), v(IGT, -6.0, 5.0)], f(0.1), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(add, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(addName, addOp, IGT, [v(IGT, -2e9, 2e9), v(IGT, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(add, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(addName, addOp, UGT, [v(UGT, 0.0, 1e2), v(UGT, 0.0, 1e2)], f(5e-3), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(add, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(addName, addOp, UGT, [v(UGT, 0.0, 4e9), v(UGT, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(add, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        binary.push(op(addName, addOp, FV, [v(FV, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.addScalar)));
        binary.push(op(addName, addOp, IV, [v(IV, -4.0, 6.0), v(I, -6.0, 5.0)], f(0.1), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.addScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(addName, addOp, IV, [v(IV, -2e9, 2e9), v(I, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.addScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(addName, addOp, UV, [v(UV, 0.0, 1e2), v(U, 0.0, 1e2)], f(5e-3), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.addScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(addName, addOp, UV, [v(UV, 0.0, 4e9), v(U, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.addScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(addName, addOp, FV, [v(F, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
                all, es3fShaderOperatorTests.binaryScalarVecFuncs(addScalarVec)));
            binary.push(op(addName, addOp, IV, [v(I, -4.0, 6.0), v(IV, -6.0, 5.0)], f(0.1), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(addScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(addName, addOp, IV, [v(I, -2e9, 2e9), v(IV, -2e9, 2e9)], f(4e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(addScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(addName, addOp, UV, [v(U, 0.0, 1e2), v(UV, 0.0, 1e2)], f(5e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(addScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(addName, addOp, UV, [v(U, 0.0, 4e9), v(UV, 0.0, 4e9)], f(2e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(addScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        // The subtract operator.

        binary.push(op(subName, subOp, GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryGenTypeFuncs(sub)));
        binary.push(op(subName, subOp, IGT, [v(IGT, -4.0, 6.0), v(IGT, -6.0, 5.0)], f(0.1), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(sub, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(subName, subOp, IGT, [v(IGT, -2e9, 2e9), v(IGT, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(sub, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(subName, subOp, UGT, [v(UGT, 1e2, 2e2), v(UGT, 0.0, 1e2)], f(5e-3), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(sub, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(subName, subOp, UGT, [v(UGT, .5e9, 3.7e9), v(UGT, 0.0, 3.9e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(sub, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(subName, subOp, FV, [v(FV, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.subScalar)));
        binary.push(op(subName, subOp, IV, [v(IV, -4.0, 6.0), v(I, -6.0, 5.0)], f(0.1), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.subScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(subName, subOp, IV, [v(IV, -2e9, 2e9), v(I, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.subScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(subName, subOp, UV, [v(UV, 1e2, 2e2), v(U, 0.0, 1e2)], f(5e-3), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.subScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(subName, subOp, UV, [v(UV, 0.0, 4e9), v(U, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.subScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(subName, subOp, FV, [v(F, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
                all, es3fShaderOperatorTests.binaryScalarVecFuncs(subScalarVec)));
            binary.push(op(subName, subOp, IV, [v(I, -4.0, 6.0), v(IV, -6.0, 5.0)], f(0.1), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(subScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(subName, subOp, IV, [v(I, -2e9, 2e9), v(IV, -2e9, 2e9)], f(4e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(subScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(subName, subOp, UV, [v(U, 1e2, 2e2), v(UV, 0.0, 1e2)], f(5e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(subScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(subName, subOp, UV, [v(U, 0.0, 4e9), v(UV, 0.0, 4e9)], f(2e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(subScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        binary = new es3fShaderOperatorTests.BuiltinFuncGroup('binary_operator', 'Binary operator tests');
        funcInfoGroups.push(binary);

        // The multiply operator.

        binary.push(op(mulName, mulOp, GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryGenTypeFuncs(mul)));
        binary.push(op(mulName, mulOp, IGT, [v(IGT, -4.0, 6.0), v(IGT, -6.0, 5.0)], f(0.1), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(mul, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(mulName, mulOp, IGT, [v(IGT, -3e5, 3e5), v(IGT, -3e4, 3e4)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(mul, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(mulName, mulOp, UGT, [v(UGT, 0.0, 16.0), v(UGT, 0.0, 16.0)], f(4e-3), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(mul, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(mulName, mulOp, UGT, [v(UGT, 0.0, 6e5), v(UGT, 0.0, 6e4)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(mul, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(mulName, mulOp, FV, [v(FV, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.scale)));
        binary.push(op(mulName, mulOp, IV, [v(IV, -4.0, 6.0), v(I, -6.0, 5.0)], f(0.1), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.scale, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(mulName, mulOp, IV, [v(IV, -3e5, 3e5), v(I, -3e4, 3e4)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.scale, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(mulName, mulOp, UV, [v(UV, 0.0, 16.0), v(U, 0.0, 16.0)], f(4e-3), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.scale, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(mulName, mulOp, UV, [v(UV, 0.0, 6e5), v(U, 0.0, 6e4)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.scale, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(mulName, mulOp, FV, [v(F, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
                all, es3fShaderOperatorTests.binaryScalarVecFuncs(mulScalarVec)));
            binary.push(op(mulName, mulOp, IV, [v(I, -4.0, 6.0), v(IV, -6.0, 5.0)], f(0.1), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(mulScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(mulName, mulOp, IV, [v(I, -3e5, 3e5), v(IV, -3e4, 3e4)], f(4e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(mulScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(mulName, mulOp, UV, [v(U, 0.0, 16.0), v(UV, 0.0, 16.0)], f(4e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(mulScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(mulName, mulOp, UV, [v(U, 0.0, 6e5), v(UV, 0.0, 6e4)], f(2e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(mulScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        // The divide operator.

        binary.push(op(divName, divOp, GT, [v(GT, -1.0, 1.0), v(GT, -2.0, -0.5)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryGenTypeFuncs(div)));
        binary.push(op(divName, divOp, IGT, [v(IGT, 24.0, 24.0), v(IGT, -4.0, -1.0)], f(0.04), f(1.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(div, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(divName, divOp, IGT, [v(IGT, 40320.0, 40320.0), v(IGT, -8.0, -1.0)], f(1e-5), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(div, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(divName, divOp, UGT, [v(UGT, 0.0, 24.0), v(UGT, 1.0, 4.0)], f(0.04), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(div, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(divName, divOp, UGT, [v(UGT, 0.0, 40320.0), v(UGT, 1.0, 8.0)], f(1e-5), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(div, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(divName, divOp, FV, [v(FV, -1.0, 1.0), v(F, -2.0, -0.5)], f(1.0), f(0.0),
            all, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.divideScale)));
        binary.push(op(divName, divOp, IV, [v(IV, 24.0, 24.0), v(I, -4.0, -1.0)], f(0.04), f(1.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.divideScale, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(divName, divOp, IV, [v(IV, 40320.0, 40320.0), v(I, -8.0, -1.0)], f(1e-5), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.divideScale, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(divName, divOp, UV, [v(UV, 0.0, 24.0), v(U, 1.0, 4.0)], f(0.04), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.divideScale, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(divName, divOp, UV, [v(UV, 0.0, 40320.0), v(U, 1.0, 8.0)], f(1e-5), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.divideScale, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(divName, divOp, FV, [v(F, -1.0, 1.0), v(FV, -2.0, -0.5)], f(1.0), f(0.0),
                all, es3fShaderOperatorTests.binaryScalarVecFuncs(divScalarVec)));
            binary.push(op(divName, divOp, IV, [v(I, 24.0, 24.0), v(IV, -4.0, -1.0)], f(0.04), f(1.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(divScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(divName, divOp, IV, [v(I, 40320.0, 40320.0), v(IV, -8.0, -1.0)], f(1e-5), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(divScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(divName, divOp, UV, [v(U, 0.0, 24.0), v(UV, 1.0, 4.0)], f(0.04), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(divScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(divName, divOp, UV, [v(U, 0.0, 40320.0), v(UV, 1.0, 8.0)], f(1e-5), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(divScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        binary = new es3fShaderOperatorTests.BuiltinFuncGroup('binary_operator', 'Binary operator tests');
        funcInfoGroups.push(binary);

        // The modulus operator.

        binary.push(op(modName, modOp, IGT, [v(IGT, 0.0, 6.0), v(IGT, 1.1, 6.1)], f(0.25), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.mod, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(modName, modOp, IGT, [v(IGT, 0.0, 14.0), v(IGT, 1.1, 11.1)], f(0.1), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.mod, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(modName, modOp, UGT, [v(UGT, 0.0, 6.0), v(UGT, 1.1, 6.1)], f(0.25), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.mod, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(modName, modOp, UGT, [v(UGT, 0.0, 24.0), v(UGT, 1.1, 11.1)], f(0.1), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.mod, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(modName, modOp, IV, [v(IV, 0.0, 6.0), v(I, 1.1, 6.1)], f(0.25), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.modScale, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(modName, modOp, IV, [v(IV, 0.0, 6.0), v(I, 1.1, 11.1)], f(0.1), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.modScale, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(modName, modOp, UV, [v(UV, 0.0, 6.0), v(U, 1.1, 6.1)], f(0.25), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.modScale, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(modName, modOp, UV, [v(UV, 0.0, 24.0), v(U, 1.1, 11.1)], f(0.1), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.modScale, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(modName, modOp, IV, [v(I, 0.0, 6.0), v(IV, 1.1, 6.1)], f(0.25), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(modScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(modName, modOp, IV, [v(I, 0.0, 6.0), v(IV, 1.1, 11.1)], f(0.1), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(modScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(modName, modOp, UV, [v(U, 0.0, 6.0), v(UV, 1.1, 6.1)], f(0.25), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(modScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(modName, modOp, UV, [v(U, 0.0, 24.0), v(UV, 1.1, 11.1)], f(0.1), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(modScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        // The bitwise and operator.

        binary.push(op(andName, andOp, IGT, [v(IGT, -16.0, 16.0), v(IGT, -16.0, 16.0)], f(0.03), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryAnd, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(andName, andOp, IGT, [v(IGT, -2e9, 2e9), v(IGT, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryAnd, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(andName, andOp, UGT, [v(UGT, 0.0, 32.0), v(UGT, 0.0, 32.0)], f(0.03), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryAnd, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(andName, andOp, UGT, [v(UGT, 0.0, 4e9), v(UGT, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryAnd, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(andName, andOp, IV, [v(IV, -16.0, 16.0), v(I, -16.0, 16.0)], f(0.03), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryAndVecScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(andName, andOp, IV, [v(IV, -2e9, 2e9), v(I, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryAndVecScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(andName, andOp, UV, [v(UV, 0.0, 32.0), v(U, 0.0, 32.0)], f(0.03), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryAndVecScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(andName, andOp, UV, [v(UV, 0.0, 4e9), v(U, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryAndVecScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(andName, andOp, IV, [v(I, -16.0, 16.0), v(IV, -16.0, 16.0)], f(0.03), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseAndScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(andName, andOp, IV, [v(I, -2e9, 2e9), v(IV, -2e9, 2e9)], f(4e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseAndScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(andName, andOp, UV, [v(U, 0.0, 32.0), v(UV, 0.0, 32.0)], f(0.03), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseAndScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(andName, andOp, UV, [v(U, 0.0, 4e9), v(UV, 0.0, 4e9)], f(2e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseAndScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        binary = new es3fShaderOperatorTests.BuiltinFuncGroup('binary_operator', 'Binary operator tests');
        funcInfoGroups.push(binary);

        // The bitwise or operator.

        binary.push(op(orName, orOp, IGT, [v(IGT, -16.0, 16.0), v(IGT, -16.0, 16.0)], f(0.03), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryOr, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(orName, orOp, IGT, [v(IGT, -2e9, 2e9), v(IGT, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryOr, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(orName, orOp, UGT, [v(UGT, 0.0, 32.0), v(UGT, 0.0, 32.0)], f(0.03), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryOr, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(orName, orOp, UGT, [v(UGT, 0.0, 4e9), v(UGT, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryOr, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(orName, orOp, IV, [v(IV, -16.0, 16.0), v(I, -16.0, 16.0)], f(0.03), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryOrVecScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(orName, orOp, IV, [v(IV, -2e9, 2e9), v(I, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryOrVecScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(orName, orOp, UV, [v(UV, 0.0, 32.0), v(U, 0.0, 32.0)], f(0.03), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryOrVecScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(orName, orOp, UV, [v(UV, 0.0, 4e9), v(U, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryOrVecScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(orName, orOp, IV, [v(I, -16.0, 16.0), v(IV, -16.0, 16.0)], f(0.03), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseOrScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(orName, orOp, IV, [v(I, -2e9, 2e9), v(IV, -2e9, 2e9)], f(4e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseOrScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(orName, orOp, UV, [v(U, 0.0, 32.0), v(UV, 0.0, 32.0)], f(0.03), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseOrScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(orName, orOp, UV, [v(U, 0.0, 4e9), v(UV, 0.0, 4e9)], f(2e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseOrScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        // The bitwise xor operator.

        binary.push(op(xorName, xorOp, IGT, [v(IGT, -16.0, 16.0), v(IGT, -16.0, 16.0)], f(0.03), f(0.5),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryXor, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(xorName, xorOp, IGT, [v(IGT, -2e9, 2e9), v(IGT, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryXor, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(xorName, xorOp, UGT, [v(UGT, 0.0, 32.0), v(UGT, 0.0, 32.0)], f(0.03), f(0.0),
            mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryXor, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(xorName, xorOp, UGT, [v(UGT, 0.0, 4e9), v(UGT, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.binaryXor, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(xorName, xorOp, IV, [v(IV, -16.0, 16.0), v(I, -16.0, 16.0)], f(0.03), f(0.5),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryXorVecScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(xorName, xorOp, IV, [v(IV, -2e9, 2e9), v(I, -2e9, 2e9)], f(4e-10), f(0.5),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryXorVecScalar, gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT)));
        binary.push(op(xorName, xorOp, UV, [v(UV, 0.0, 32.0), v(U, 0.0, 32.0)], f(0.03), f(0.0),
            mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryXorVecScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));
        binary.push(op(xorName, xorOp, UV, [v(UV, 0.0, 4e9), v(U, 0.0, 4e9)], f(2e-10), f(0.0),
            highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.binaryXorVecScalar, gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT)));

        if (isNormalOp) {
            binary.push(op(xorName, xorOp, IV, [v(I, -16.0, 16.0), v(IV, -16.0, 16.0)], f(0.03), f(0.5),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseXorScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(xorName, xorOp, IV, [v(I, -2e9, 2e9), v(IV, -2e9, 2e9)], f(4e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseXorScalarVec, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(xorName, xorOp, UV, [v(U, 0.0, 32.0), v(UV, 0.0, 32.0)], f(0.03), f(0.0),
                mediump, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseXorScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(xorName, xorOp, UV, [v(U, 0.0, 4e9), v(UV, 0.0, 4e9)], f(2e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryScalarVecFuncs(bitwiseXorScalarVec, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        binary = new es3fShaderOperatorTests.BuiltinFuncGroup('binary_operator', 'Binary operator tests');
        funcInfoGroups.push(binary);

        // The left shift operator. Second operand (shift amount) can be either int or uint, even for uint and int first operand, respectively.
        for (var isSignedAmount = 0; isSignedAmount <= 1; isSignedAmount++) {
            var gType = isSignedAmount == 0 ? UGT : IGT;
            var sType = isSignedAmount == 0 ? U : I;
            binary.push(op(leftShiftName, leftShiftOp, IGT, [v(IGT, -7.0, 7.0), v(gType, 0.0, 4.0)], f(4e-3), f(0.5),
                mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftLeft, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(leftShiftName, leftShiftOp, IGT, [v(IGT, -7.0, 7.0), v(gType, 0.0, 27.0)], f(5e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftLeft, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(leftShiftName, leftShiftOp, UGT, [v(UGT, 0.0, 7.0), v(gType, 0.0, 5.0)], f(4e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftLeft, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(leftShiftName, leftShiftOp, UGT, [v(UGT, 0.0, 7.0), v(gType, 0.0, 28.0)], f(5e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftLeft, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(leftShiftName, leftShiftOp, IV, [v(IV, -7.0, 7.0), v(sType, 0.0, 4.0)], f(4e-3), f(0.5),
                mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftLeftVecScalar, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(leftShiftName, leftShiftOp, IV, [v(IV, -7.0, 7.0), v(sType, 0.0, 27.0)], f(5e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftLeftVecScalar, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(leftShiftName, leftShiftOp, UV, [v(UV, 0.0, 7.0), v(sType, 0.0, 5.0)], f(4e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftLeftVecScalar, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(leftShiftName, leftShiftOp, UV, [v(UV, 0.0, 7.0), v(sType, 0.0, 28.0)], f(5e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftLeftVecScalar, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }

        // The right shift operator. Second operand (shift amount) can be either int or uint, even for uint and int first operand, respectively.

        for (var isSignedAmount = 0; isSignedAmount <= 1; isSignedAmount++) {
            gType = isSignedAmount == 0 ? UGT : IGT;
            sType = isSignedAmount == 0 ? U : I;
            binary.push(op(rightShiftName, rightShiftOp, IGT, [v(IGT, -127.0, 127.0), v(gType, 0.0, 8.0)], f(4e-3), f(0.5),
                mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftRight, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(rightShiftName, rightShiftOp, IGT, [v(IGT, -2e9, 2e9), v(gType, 0.0, 31.0)], f(5e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftRight, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(rightShiftName, rightShiftOp, UGT, [v(UGT, 0.0, 255.0), v(gType, 0.0, 8.0)], f(4e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftRight, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(rightShiftName, rightShiftOp, UGT, [v(UGT, 0.0, 4e9), v(gType, 0.0, 31.0)], f(5e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.shiftRight, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(rightShiftName, rightShiftOp, IV, [v(IV, -127.0, 127.0), v(sType, 0.0, 8.0)], f(4e-3), f(0.5),
                mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftRightVecScalar, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(rightShiftName, rightShiftOp, IV, [v(IV, -2e9, 2e9), v(sType, 0.0, 31.0)], f(5e-10), f(0.5),
                highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftRightVecScalar, gluShaderUtil.DataType.INT,
                gluShaderUtil.DataType.INT)));
            binary.push(op(rightShiftName, rightShiftOp, UV, [v(UV, 0.0, 255.0), v(sType, 0.0, 8.0)], f(4e-3), f(0.0),
                mediump, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftRightVecScalar, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
            binary.push(op(rightShiftName, rightShiftOp, UV, [v(UV, 0.0, 4e9), v(sType, 0.0, 31.0)], f(5e-10), f(0.0),
                highp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.shiftRightVecScalar, gluShaderUtil.DataType.UINT,
                gluShaderUtil.DataType.UINT)));
        }
    }

    // Rest of binary operators.
    // Scalar relational operators.
    binary = new es3fShaderOperatorTests.BuiltinFuncGroup('binary_operator', 'Binary operator tests');
    funcInfoGroups.push(binary);

    binary.push(op('less', '<', B, [v(F, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(lessThan).scalar}));
    binary.push(op('less', '<', B, [v(I, -5.0, 5.0), v(I, -5.0, 5.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(lessThan, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT).scalar}));
    binary.push(op('less', '<', B, [v(U, 0.0, 16.0), v(U, 0.0, 16.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(lessThan, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT).scalar}));
    binary.push(op('less_or_equal', '<=', B, [v(F, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(lessThanEqual).scalar}));
    binary.push(op('less_or_equal', '<=', B, [v(I, -5.0, 5.0), v(I, -5.0, 5.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(lessThanEqual, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT).scalar}));
    binary.push(op('less_or_equal', '<=', B, [v(U, 0.0, 16.0), v(U, 0.0, 16.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(lessThanEqual, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT).scalar}));
    binary.push(op('greater', '>', B, [v(F, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(greaterThan).scalar}));
    binary.push(op('greater', '>', B, [v(I, -5.0, 5.0), v(I, -5.0, 5.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(greaterThan, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT).scalar}));
    binary.push(op('greater', '>', B, [v(U, 0.0, 16.0), v(U, 0.0, 16.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(greaterThan, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT).scalar}));
    binary.push(op('greater_or_equal', '>=', B, [v(F, -1.0, 1.0), v(F, -1.0, 1.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(greaterThanEqual).scalar}));
    binary.push(op('greater_or_equal', '>=', B, [v(I, -5.0, 5.0), v(I, -5.0, 5.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(greaterThanEqual, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT).scalar}));
    binary.push(op('greater_or_equal', '>=', B, [v(U, 0.0, 16.0), v(U, 0.0, 16.0)], f(1.0), f(0.0),
        all, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(greaterThanEqual, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT).scalar}));

    binary.push(op('equal', '==', B, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(allEqual)));
    binary.push(op('equal', '==', B, [v(IGT, -5.5, 4.7), v(IGT, -2.1, 0.1)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(allEqual, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    binary.push(op('equal', '==', B, [v(UGT, 0.0, 8.0), v(UGT, 3.5, 4.5)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(allEqual, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    binary.push(op('equal', '==', B, [v(BGT, -2.1, 2.1), v(BGT, -1.1, 3.0)], f(1.0), f(0.0),
        na, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(allEqual, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL)));
    binary.push(op('not_equal', '!=', B, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(anyNotEqual)));
    binary.push(op('not_equal', '!=', B, [v(IGT, -5.5, 4.7), v(IGT, -2.1, 0.1)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(anyNotEqual, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    binary.push(op('not_equal', '!=', B, [v(UGT, 0.0, 8.0), v(UGT, 3.5, 4.5)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(anyNotEqual, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    binary.push(op('not_equal', '!=', B, [v(BGT, -2.1, 2.1), v(BGT, -1.1, 3.0)], f(1.0), f(0.0),
        na, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(anyNotEqual, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL)));

    // Logical operators.
    binary.push(op('logical_and', '&&', B, [v(B, -1.0, 1.0), v(B, -1.0, 1.0)], f(1.0), f(0.0),
        na, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(logicalAnd, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL).scalar}));
    binary.push(op('logical_or', '||', B, [v(B, -1.0, 1.0), v(B, -1.0, 1.0)], f(1.0), f(0.0),
        na, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(logicalOr, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL).scalar}));
    binary.push(op('logical_xor', '^^', B, [v(B, -1.0, 1.0), v(B, -1.0, 1.0)], f(1.0), f(0.0),
        na, {scalar: es3fShaderOperatorTests.binaryGenTypeFuncs(logicalXor, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL).scalar}));

    // 8.1 Angle and Trigonometry Functions.
    var trig = new es3fShaderOperatorTests.BuiltinFuncGroup("angle_and_trigonometry", "Angle and trigonometry function tests.");
    funcInfoGroups.push(trig);
    op = es3fShaderOperatorTests.builtinFunctionInfo;
    trig.push(op("radians", "radians", GT, [v(GT, -1.0, 1.0)], f(25.0), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(radians)));
    trig.push(op("degrees", "degrees", GT, [v(GT, -1.0, 1.0)], f(0.04), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(degrees)));
    trig.push(op("sin", "sin", GT, [v(GT, -5.0, 5.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.sin)));
    trig.push(op("sin", "sin", GT, [v(GT, -1.5, 1.5)], f(0.5), f(0.5),
        lowp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.sin)));
    trig.push(op("cos", "cos", GT, [v(GT, -5.0, 5.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.cos)));
    trig.push(op("cos", "cos", GT, [v(GT, -1.5, 1.5)], f(0.5), f(0.5),
        lowp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.cos)));
    trig.push(op("tan", "tan", GT, [v(GT, -5.0, 5.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.tan)));
    trig.push(op("tan", "tan", GT, [v(GT, -1.5, 5.5)], f(0.5), f(0.5),
        lowp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.tan)));

    trig = new es3fShaderOperatorTests.BuiltinFuncGroup("angle_and_trigonometry", "Angle and trigonometry function tests.");
    funcInfoGroups.push(trig);
    trig.push(op("asin", "asin", GT, [v(GT, -1.0, 1.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.asin)));
    trig.push(op("acos", "acos", GT, [v(GT, -1.0, 1.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.acos)));
    trig.push(op("atan", "atan", GT, [v(GT, -4.0, 4.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.atan)));
    trig.push(op("atan2", "atan", GT, [v(GT, -4.0, 4.0), v(GT, 0.5, 2.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.atan2)));

    trig = new es3fShaderOperatorTests.BuiltinFuncGroup("angle_and_trigonometry", "Angle and trigonometry function tests.");
    funcInfoGroups.push(trig);
    trig.push(op("sinh", "sinh", GT, [v(GT, -5.0, 5.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.sinh)));
    trig.push(op("sinh", "sinh", GT, [v(GT, -1.5, 1.5)], f(0.5), f(0.5),
        lowp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.sinh)));
    trig.push(op("cosh", "cosh", GT, [v(GT, -5.0, 5.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.cosh)));
    trig.push(op("cosh", "cosh", GT, [v(GT, -1.5, 1.5)], f(0.5), f(0.5),
        lowp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.cosh)));
    trig.push(op("tanh", "tanh", GT, [v(GT, -5.0, 5.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.tanh)));
    trig.push(op("tanh", "tanh", GT, [v(GT, -1.5, 5.5)], f(0.5), f(0.5),
        lowp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.tanh)));

    trig = new es3fShaderOperatorTests.BuiltinFuncGroup("angle_and_trigonometry", "Angle and trigonometry function tests.");
    funcInfoGroups.push(trig);
    trig.push(op("asinh", "asinh", GT, [v(GT, -1.0, 1.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.asinh)));
    trig.push(op("acosh", "acosh", GT, [v(GT, 1.0, 2.2)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.acosh)));
    // Results are undefined if |x| >= 1, so it diverses from C++ version here.
    trig.push(op("atanh", "atanh", GT, [v(GT, -0.99, 0.99)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.atanh)));

    // 8.2 Exponential Functions.
    var exps = new es3fShaderOperatorTests.BuiltinFuncGroup("exponential", "Exponential function tests");
    exps.push(op("pow", "pow", GT, [v(GT, 0.1, 8.0), v(GT, -4.0, 2.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.pow)));
    exps.push(op("exp", "exp", GT, [v(GT, -6.0, 3.0)], f(0.5), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.exp)));
    exps.push(op("log", "log", GT, [v(GT, 0.1, 10.0)], f(0.5), f(0.3),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.log)));
    exps.push(op("exp2", "exp2", GT, [v(GT, -7.0, 2.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(exp2)));
    exps.push(op("log2", "log2", GT, [v(GT, 0.1, 10.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.log2)));
    exps.push(op("sqrt", "sqrt", GT, [v(GT, 0.0, 10.0)], f(0.3), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.sqrt)));
    exps.push(op("inversesqrt", "inversesqrt", GT, [v(GT, 0.5, 10.0)], f(1.0), f(0.0),
        mediumhighp, es3fShaderOperatorTests.unaryGenTypeFuncs(inverseSqrt)));

    funcInfoGroups.push(exps);

    // 8.3 Common Functions.
    var comm = new es3fShaderOperatorTests.BuiltinFuncGroup("common_functions", "Common function tests.");
    comm.push(op("abs", "abs", GT, [v(GT, -2.0, 2.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.abs)));
    comm.push(op("sign", "sign", GT, [v(GT, -1.5, 1.5)], f(0.3), f(0.5),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.sign)));
    comm.push(op("floor", "floor", GT, [v(GT, 2.5, 2.5)], f(0.2), f(0.7),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.floor)));
    comm.push(op("trunc", "trunc", GT, [v(GT, 2.5, 2.5)], f(0.2), f(0.7),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.trunc)));
    comm.push(op("round", "round", GT, [v(GT, 2.5, 2.5)], f(0.2), f(0.7),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(roundToEven)));
    comm.push(op("roundEven", "roundEven", GT, [v(GT, 2.5, 2.5)], f(0.2), f(0.7),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(roundToEven)));
    comm.push(op("ceil", "ceil", GT, [v(GT, 2.5, 2.5)], f(0.2), f(0.5),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(Math.ceil)));
    comm.push(op("fract", "fract", GT, [v(GT, -1.5, 1.5)], f(0.8), f(0.1),
        all, es3fShaderOperatorTests.unaryGenTypeFuncs(fract)));
    comm.push(op("mod", "mod", GT, [v(GT, -2.0, 2.0), v(GT, 0.9, 6.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.binaryGenTypeFuncs(deMath.mod)));
    comm.push(op("mod", "mod", GT, [v(FV, -2.0, 2.0), v(F, 0.9, 6.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.binaryVecScalarFuncs(deMath.modScale)));
    comm.push(op("min", "min", GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.min)));
    comm.push(op("min", "min", GT, [v(FV, -1.0, 1.0), v(F, -1.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.binaryVecScalarFuncs(minVecScalar)));
    comm.push(op("min", "min", IGT, [v(IGT, -4.0, 4.0), v(IGT, -4.0, 4.0)], f(0.125), f(0.5),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.min, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("min", "min", IGT, [v(IV, -4.0, 4.0), v(I, -4.0, 4.0)], f(0.125), f(0.5),
        all, es3fShaderOperatorTests.binaryVecScalarFuncs(minVecScalar, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("min", "min", UGT, [v(UGT, 0.0, 8.0), v(UGT, 0.0, 8.0)], f(0.125), f(0.0),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.min, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("min", "min", UGT, [v(UV, 0.0, 8.0), v(U, 0.0, 8.0)], f(0.125), f(0.0),
        all, es3fShaderOperatorTests.binaryVecScalarFuncs(minVecScalar, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    comm.push(op("max", "max", GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.max)));
    comm.push(op("max", "max", GT, [v(FV, -1.0, 1.0), v(F, -1.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.binaryVecScalarFuncs(maxVecScalar)));
    comm.push(op("max", "max", IGT, [v(IGT, -4.0, 4.0), v(IGT, -4.0, 4.0)], f(0.125), f(0.5),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.max, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("max", "max", IGT, [v(IV, -4.0, 4.0), v(I, -4.0, 4.0)], f(0.125), f(0.5),
        all, es3fShaderOperatorTests.binaryVecScalarFuncs(maxVecScalar, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("max", "max", UGT, [v(UGT, 0.0, 8.0), v(UGT, 0.0, 8.0)], f(0.125), f(0.0),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(Math.max, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("max", "max", UGT, [v(UV, 0.0, 8.0), v(U, 0.0, 8.0)], f(0.125), f(0.0),
        all, es3fShaderOperatorTests.binaryVecScalarFuncs(maxVecScalar, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    comm.push(op("clamp", "clamp", GT, [v(GT, -1.0, 1.0), v(GT, -0.5, 0.5), v(GT, 0.5, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.ternaryGenTypeFuncs(deMath.clamp)));
    comm.push(op("clamp", "clamp", GT, [v(FV, -1.0, 1.0), v(F, -0.5, 0.5), v(F, 0.5, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.ternaryVecScalarScalarFuncs(clampVecScalarScalar)));
    comm.push(op("clamp", "clamp", IGT, [v(IGT, -4.0, 4.0), v(IGT, -2.0, 2.0), v(IGT, 2.0, 4.0)], f(0.125), f(0.5),
        all, es3fShaderOperatorTests.ternaryGenTypeFuncs(deMath.clamp, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("clamp", "clamp", IGT, [v(IGT, -4.0, 4.0), v(I, -2.0, 2.0), v(I, 2.0, 4.0)], f(0.125), f(0.5),
        all, es3fShaderOperatorTests.ternaryVecScalarScalarFuncs(clampVecScalarScalar, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    comm.push(op("clamp", "clamp", UGT, [v(UGT, 0.0, 8.0), v(UGT, 2.0, 6.0), v(UGT, 6.0, 8.0)], f(0.125), f(0.0),
        all, es3fShaderOperatorTests.ternaryGenTypeFuncs(deMath.clamp, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    comm.push(op("clamp", "clamp", UGT, [v(UV, 0.0, 8.0), v(U, 2.0, 6.0), v(U, 6.0, 8.0)], f(0.125), f(0.0),
        all, es3fShaderOperatorTests.ternaryVecScalarScalarFuncs(clampVecScalarScalar, gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT)));
    comm.push(op("mix", "mix", GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 1.0), v(GT, 0.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.ternaryGenTypeFuncs(mix)));
    comm.push(op("mix", "mix", GT, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0), v(F, 0.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.ternaryVecVecScalarFuncs(mixVecVecScalar)));
    comm.push(op("step", "step", GT, [v(GT, -1.0, 1.0), v(GT, -1.0, 0.0)], f(0.5), f(0.25),
        all, es3fShaderOperatorTests.binaryGenTypeFuncs(step)));
    comm.push(op("step", "step", GT, [v(F, -1.0, 1.0), v(FV, -1.0, 0.0)], f(0.5), f(0.25),
        all, es3fShaderOperatorTests.binaryScalarVecFuncs(stepScalarVec)));
    comm.push(op("smoothstep", "smoothstep", GT, [v(GT, -0.5, 0.0), v(GT, 0.1, 1.0), v(GT, -1.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.ternaryGenTypeFuncs(smoothStep)));
    comm.push(op("smoothstep", "smoothstep", GT, [v(F, -0.5, 0.0), v(F, 0.1, 1.0), v(FV, -1.0, 1.0)], f(0.5), f(0.5),
        all, es3fShaderOperatorTests.ternaryScalarScalarVecFuncs(smoothStepScalarScalarVec)));

    funcInfoGroups.push(comm);

    // 8.4 Geometric Functions.
    var geom = new es3fShaderOperatorTests.BuiltinFuncGroup("geometric", "Geometric function tests.");
    geom.push(op("length", "length", F, [v(GT, -5.0, 5.0)], f(0.1), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryScalarGenTypeFuncs(length)));
    geom.push(op("distance", "distance", F, [v(GT, -5.0, 5.0), v(GT, -5.0, 5.0)], f(0.1), f(0.5),
        mediumhighp, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(distance)));
    geom.push(op("dot", "dot", F, [v(GT, -5.0, 5.0), v(GT, -5.0, 5.0)], f(0.1), f(0.5),
        mediumhighp, es3fShaderOperatorTests.binaryScalarGenTypeFuncs(dot)));
    geom.push(op("cross", "cross", V3, [v(GT, -5.0, 5.0), v(GT, -5.0, 5.0)], f(0.1), f(0.5),
        mediumhighp, {vec3: es3fShaderOperatorTests.binaryVecVecFuncs(cross).vec3}));
    geom.push(op("normalize", "normalize", GT, [v(GT, 0.1, 4.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.unaryArrayFuncs(normalize)));
    geom.push(op("faceforward", "faceforward", GT, [v(GT, -5.0, 5.0), v(GT, -5.0, 5.0), v(GT, -1.0, 1.0)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.ternaryVecVecVecFuncs(faceforward)));
    geom.push(op("reflect", "reflect", GT, [v(GT, -0.8, -0.5), v(GT, 0.5, 0.8)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.binaryVecVecFuncs(reflect)));
    geom.push(op("refract", "refract", GT, [v(GT, -0.8, 1.2), v(GT, -1.1, 0.5), v(F, 0.2, 1.5)], f(0.5), f(0.5),
        mediumhighp, es3fShaderOperatorTests.ternaryVecVecScalarFuncs(refract)));

    funcInfoGroups.push(geom);

    // 8.6 Vector Relational Functions.
    var floatComp = new es3fShaderOperatorTests.BuiltinFuncGroup("float_compare", "Floating point comparison tests.");
    floatComp.push(op("lessThan", "lessThan", BV, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(lessThanVec)));
    floatComp.push(op("lessThanEqual", "lessThanEqual", BV, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(lessThanEqualVec)));
    floatComp.push(op("greaterThan", "greaterThan", BV, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(greaterThanVec)));
    floatComp.push(op("greaterThanEqual", "greaterThanEqual", BV, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(greaterThanEqualVec)));
    floatComp.push(op("equal", "equal", BV, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(allEqualVec)));
    floatComp.push(op("notEqual", "notEqual", BV, [v(FV, -1.0, 1.0), v(FV, -1.0, 1.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(anyNotEqualVec)));

    funcInfoGroups.push(floatComp);

    var intComp = new es3fShaderOperatorTests.BuiltinFuncGroup("int_compare", "Integer comparison tests.");
    intComp.push(op("lessThan", "lessThan", BV, [v(IV, 5.2, 4.9), v(IV, -5.0, 5.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(lessThanVec, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    intComp.push(op("lessThanEqual", "lessThanEqual", BV, [v(IV, -5.2, 4.9), v(IV, -5.0, 5.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(lessThanEqualVec, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    intComp.push(op("greaterThan", "greaterThan", BV, [v(IV, -5.2, 4.9), v(IV, -5.0, 5.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(greaterThanVec, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    intComp.push(op("greaterThanEqual", "greaterThanEqual", BV, [v(IV, -5.2, 4.9), v(IV, -5.0, 5.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(greaterThanEqualVec, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    intComp.push(op("equal", "equal", BV, [v(IV, -5.2, 4.9), v(IV, -5.0, 5.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(allEqualVec, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));
    intComp.push(op("notEqual", "notEqual", BV, [v(IV, -5.2, 4.9), v(IV, -5.0, 5.0)], f(1.0), f(0.0),
        all, es3fShaderOperatorTests.binaryVecVecFuncs(anyNotEqualVec, gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT)));

    funcInfoGroups.push(intComp);

    var boolComp = new es3fShaderOperatorTests.BuiltinFuncGroup("bool_compare", "Boolean comparison tests.");
    var evalBoolEqual = es3fShaderOperatorTests.binaryVecVecFuncs(allEqualVec, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL);
    var evalBoolNotEqual = es3fShaderOperatorTests.binaryVecVecFuncs(anyNotEqualVec, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL);
    var evalBoolAny = es3fShaderOperatorTests.unaryBooleanGenTypeFuncs(boolAny, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL);
    evalBoolAny.scalar = null;
    var evalBoolAll = es3fShaderOperatorTests.unaryBooleanGenTypeFuncs(boolAll, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL);
    evalBoolAll.scalar = null;
    var evalBoolNot = es3fShaderOperatorTests.unaryArrayFuncs(boolNotVec, gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL);
    evalBoolNot.scalar = null;

    boolComp.push(op("equal", "equal", BV, [v(BV, -5.2, 4.9), v(BV, -5.0, 5.0)], f(1.0), f(0.0),
        na, evalBoolEqual));
    boolComp.push(op("notEqual", "notEqual", BV, [v(BV, -5.2, 4.9), v(BV, -5.0, 5.0)], f(1.0), f(0.0),
        na, evalBoolNotEqual));
    boolComp.push(op("any", "any", B, [v(BV, -1.0, 0.3)], f(1.0), f(0.0),
        na, evalBoolAny));
    boolComp.push(op("all", "all", B, [v(BV, -0.3, 1.0)], f(1.0), f(0.0),
        na, evalBoolAll));
    boolComp.push(op("not", "not", BV, [v(BV, -1.0, 1.0)], f(1.0), f(0.0),
        na, evalBoolNot));

    funcInfoGroups.push(boolComp);

    var s_shaderTypes = [
        gluShaderProgram.shaderType.VERTEX,
        gluShaderProgram.shaderType.FRAGMENT
    ];

    var s_floatTypes = [
        gluShaderUtil.DataType.FLOAT,
        gluShaderUtil.DataType.FLOAT_VEC2,
        gluShaderUtil.DataType.FLOAT_VEC3,
        gluShaderUtil.DataType.FLOAT_VEC4
    ];

    var s_intTypes = [
        gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT_VEC2,
        gluShaderUtil.DataType.INT_VEC3,
        gluShaderUtil.DataType.INT_VEC4
    ];

    var s_uintTypes = [
        gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT_VEC2,
        gluShaderUtil.DataType.UINT_VEC3,
        gluShaderUtil.DataType.UINT_VEC4
    ];

    var s_boolTypes = [
        gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL_VEC2,
        gluShaderUtil.DataType.BOOL_VEC3,
        gluShaderUtil.DataType.BOOL_VEC4
    ];

    for (var outerGroupNdx = 0; outerGroupNdx < funcInfoGroups.length; outerGroupNdx++) {
        // Create outer group.
        var outerGroupInfo = funcInfoGroups[outerGroupNdx];
        var outerGroup = new tcuTestCase.DeqpTest(outerGroupInfo.name, outerGroupInfo.description);
        this.addChild(outerGroup);

        // Only create new group if name differs from previous one.
        var innerGroup = null;

        for (var funcInfoNdx = 0; funcInfoNdx < outerGroupInfo.funcInfos.length; funcInfoNdx++) {
            var funcInfo = outerGroupInfo.funcInfos[funcInfoNdx];
            var shaderFuncName = funcInfo.shaderFuncName;
            var isBoolCase = (funcInfo.precision == es3fShaderOperatorTests.Precision.None);
            var isBoolOut = es3fShaderOperatorTests.isBoolType(funcInfo.outValue);
            var isIntOut = es3fShaderOperatorTests.isIntType(funcInfo.outValue);
            var isUintOut = es3fShaderOperatorTests.isUintType(funcInfo.outValue);
            var isFloatOut = !isBoolOut && !isIntOut && !isUintOut;

            if (!innerGroup || (innerGroup.name != funcInfo.caseName)) {
                var groupDesc = 'Built-in function ' + shaderFuncName + '() tests.';
                innerGroup = new tcuTestCase.DeqpTest(funcInfo.caseName, groupDesc);
                outerGroup.addChild(innerGroup);
            }

            for (var inScalarSize = 1; inScalarSize <= 4; inScalarSize++) {
                var outScalarSize = ((funcInfo.outValue == es3fShaderOperatorTests.ValueType.FLOAT) || (funcInfo.outValue == es3fShaderOperatorTests.ValueType.BOOL)) ? 1 : inScalarSize; // \todo [petri] Int.
                var outDataType = isFloatOut ? s_floatTypes[outScalarSize - 1] :
                                            isIntOut ? s_intTypes[outScalarSize - 1] :
                                            isUintOut ? s_uintTypes[outScalarSize - 1] :
                                            isBoolOut ? s_boolTypes[outScalarSize - 1] :
                                            undefined;

                var evalFunc = null;
                if (inScalarSize == 1) evalFunc = funcInfo.evalFunctions.scalar;
                else if (inScalarSize == 2) evalFunc = funcInfo.evalFunctions.vec2;
                else if (inScalarSize == 3) evalFunc = funcInfo.evalFunctions.vec3;
                else if (inScalarSize == 4) evalFunc = funcInfo.evalFunctions.vec4;
                else throw new Error('Invalid scalar size ' + inScalarSize);

                // Skip if no valid eval func.
                // \todo [petri] Better check for V3 only etc. cases?
                if (evalFunc == null)
                    continue;

                var precisions = ['low', 'medium', 'high'];
                for (var precId = 0; precId < precisions.length; precId++) {
                    var precision = precisions[precId];
                    if ((funcInfo.precision[precision]) ||
                        (funcInfo.precision == es3fShaderOperatorTests.Precision.None && precision === 'medium')) { // use mediump interpolators for booleans
                        var precisionPrefix = isBoolCase ? '' : precision + 'p_';

                        for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                            var shaderType = s_shaderTypes[shaderTypeNdx];
                            var shaderSpec = new es3fShaderOperatorTests.ShaderDataSpec();
                            var shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                            var isVertexCase = shaderType == gluShaderProgram.shaderType.VERTEX;
                            var isUnaryOp = (funcInfo.inputs.length == 1);

                            // \note Data type names will be added to description and name in a following loop.
                            var desc = 'Built-in function ' + shaderFuncName + '(';
                            var name = precisionPrefix;

                            // Generate shader op.
                            var shaderOp = 'res = ';

                            var precNames = [gluShaderUtil.precision.PRECISION_LOWP,
                                             gluShaderUtil.precision.PRECISION_MEDIUMP,
                                             gluShaderUtil.precision.PRECISION_HIGHP];
                            // Setup shader data info.
                            shaderSpec.numInputs = 0;
                            shaderSpec.precision = isBoolCase ? undefined : precNames[precId];
                            shaderSpec.output = outDataType;
                            shaderSpec.resultScale = funcInfo.resultScale;
                            shaderSpec.resultBias = funcInfo.resultBias;
                            shaderSpec.referenceScale = funcInfo.referenceScale;
                            shaderSpec.referenceBias = funcInfo.referenceBias;

                            if (funcInfo.type == es3fShaderOperatorTests.OperationType.OPERATOR) {
                                if (isUnaryOp && funcInfo.isUnaryPrefix)
                                    shaderOp += shaderFuncName;
                            } else if (funcInfo.type == es3fShaderOperatorTests.OperationType.FUNCTION)
                                shaderOp += shaderFuncName + '(';
                            else // SIDE_EFFECT_OPERATOR
                                shaderOp += 'in0;\n\t';

                            for (var inputNdx = 0; inputNdx < funcInfo.inputs.length; inputNdx++) {
                                var prevNdx = inputNdx > 0 ? inputNdx - 1 : funcInfo.inputs.length - 1;
                                var prevValue = funcInfo.inputs[prevNdx];
                                var value = funcInfo.inputs[inputNdx];

                                if (value.valueType == es3fShaderOperatorTests.ValueType.NONE)
                                    continue; // Skip unused input.

                                var prevInScalarSize = es3fShaderOperatorTests.isScalarType(prevValue.valueType) ? 1 : inScalarSize;
                                var prevInDataType = es3fShaderOperatorTests.isFloatType(prevValue.valueType) ? s_floatTypes[prevInScalarSize - 1] :
                                                                    es3fShaderOperatorTests.isIntType(prevValue.valueType) ? s_intTypes[prevInScalarSize - 1] :
                                                                    es3fShaderOperatorTests.isUintType(prevValue.valueType) ? s_uintTypes[prevInScalarSize - 1] :
                                                                    es3fShaderOperatorTests.isBoolType(prevValue.valueType) ? s_boolTypes[prevInScalarSize - 1] :
                                                                    undefined;

                                var curInScalarSize = es3fShaderOperatorTests.isScalarType(value.valueType) ? 1 : inScalarSize;
                                var curInDataType = es3fShaderOperatorTests.isFloatType(value.valueType) ? s_floatTypes[curInScalarSize - 1] :
                                                                    es3fShaderOperatorTests.isIntType(value.valueType) ? s_intTypes[curInScalarSize - 1] :
                                                                    es3fShaderOperatorTests.isUintType(value.valueType) ? s_uintTypes[curInScalarSize - 1] :
                                                                    es3fShaderOperatorTests.isBoolType(value.valueType) ? s_boolTypes[curInScalarSize - 1] :
                                                                    undefined;

                                // Write input type(s) to case description and name.

                                if (inputNdx > 0)
                                    desc += ', ';

                                desc += gluShaderUtil.getDataTypeName(curInDataType);

                                if (inputNdx == 0 || prevInDataType != curInDataType) // \note Only write input type to case name if different from previous input type (avoid overly long names).
                                    name += gluShaderUtil.getDataTypeName(curInDataType) + '_';

                                // Generate op input source.

                                if (funcInfo.type == es3fShaderOperatorTests.OperationType.OPERATOR || funcInfo.type == es3fShaderOperatorTests.OperationType.FUNCTION) {
                                    if (inputNdx != 0) {
                                        if (funcInfo.type == es3fShaderOperatorTests.OperationType.OPERATOR && !isUnaryOp)
                                            shaderOp += ' ' + shaderFuncName + ' ';
                                        else
                                            shaderOp += ', ';
                                    }

                                    shaderOp += 'in' + inputNdx.toString(10);

                                    if (funcInfo.type == es3fShaderOperatorTests.OperationType.OPERATOR && isUnaryOp && !funcInfo.isUnaryPrefix)
                                        shaderOp += shaderFuncName;
                                } else{
                                    if (inputNdx != 0 || (isUnaryOp && funcInfo.isUnaryPrefix))
                                        shaderOp += (isUnaryOp ? '' : ' ') + shaderFuncName + (isUnaryOp ? '' : ' ');

                                    shaderOp += inputNdx == 0 ? 'res' : 'in' + inputNdx.toString(10); // \note in0 has already been assigned to res, so start from in1.

                                    if (isUnaryOp && !funcInfo.isUnaryPrefix)
                                        shaderOp += shaderFuncName;
                                }

                                // Fill in shader info.
                                shaderSpec.inputs[shaderSpec.numInputs++] = new es3fShaderOperatorTests.ShaderValue(curInDataType, value.rangeMin, value.rangeMax);
                            }

                            if (funcInfo.type == es3fShaderOperatorTests.OperationType.FUNCTION)
                                shaderOp += ')';

                            shaderOp += ';';

                            desc += ').';
                            name += shaderTypeName;

                            // Create the test case.
                            innerGroup.addChild(new es3fShaderOperatorTests.ShaderOperatorCase(name, desc, isVertexCase, evalFunc, shaderOp, shaderSpec));
                        }
                    }
                }
            }
        }
    }

    // The ?: selection operator.

    var s_selectionInfo = [
        gluShaderUtil.DataType.FLOAT,
        gluShaderUtil.DataType.FLOAT_VEC2,
        gluShaderUtil.DataType.FLOAT_VEC3,
        gluShaderUtil.DataType.FLOAT_VEC4,
        gluShaderUtil.DataType.INT,
        gluShaderUtil.DataType.INT_VEC2,
        gluShaderUtil.DataType.INT_VEC3,
        gluShaderUtil.DataType.INT_VEC4,
        gluShaderUtil.DataType.UINT,
        gluShaderUtil.DataType.UINT_VEC2,
        gluShaderUtil.DataType.UINT_VEC3,
        gluShaderUtil.DataType.UINT_VEC4,
        gluShaderUtil.DataType.BOOL,
        gluShaderUtil.DataType.BOOL_VEC2,
        gluShaderUtil.DataType.BOOL_VEC3,
        gluShaderUtil.DataType.BOOL_VEC4
    ];

    var selectionEvalFuncsFloat = es3fShaderOperatorTests.selectionFuncs(gluShaderUtil.DataType.FLOAT);
    var selectionEvalFuncsInt = es3fShaderOperatorTests.selectionFuncs(gluShaderUtil.DataType.INT);
    var selectionEvalFuncsUint = es3fShaderOperatorTests.selectionFuncs(gluShaderUtil.DataType.UINT);
    var selectionEvalFuncsBool = es3fShaderOperatorTests.selectionFuncs(gluShaderUtil.DataType.BOOL);

    var selectionGroup = new tcuTestCase.DeqpTest('selection', 'Selection operator tests');
    this.addChild(selectionGroup);

    for (var typeNdx = 0; typeNdx < s_selectionInfo.length; typeNdx++) {
        var curType = s_selectionInfo[typeNdx];
        var scalarSize = gluShaderUtil.getDataTypeScalarSize(curType);
        var isBoolCase = gluShaderUtil.isDataTypeBoolOrBVec(curType);
        var isFloatCase = gluShaderUtil.isDataTypeFloatOrVec(curType);
        var isIntCase = gluShaderUtil.isDataTypeIntOrIVec(curType);
        var isUintCase = gluShaderUtil.isDataTypeUintOrUVec(curType);
        var dataTypeStr = gluShaderUtil.getDataTypeName(curType);

        var evalFuncs = selectionEvalFuncsFloat;
        if (isBoolCase)
            evalFuncs = selectionEvalFuncsBool;
        else if (isIntCase)
            evalFuncs = selectionEvalFuncsInt;
        else if (isUintCase)
            evalFuncs = selectionEvalFuncsUint;

        var evalFunc = evalFuncs[scalarSize];

        for (var prec in gluShaderUtil.precision) {
            var precision = gluShaderUtil.precision[prec];
            if (isBoolCase && precision != gluShaderUtil.precision.PRECISION_MEDIUMP) // Use mediump interpolators for booleans.
                continue;

            var precisionStr = gluShaderUtil.getPrecisionName(precision);
            var precisionPrefix = isBoolCase ? '' : (precisionStr + '_');

            for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                var shaderType = s_shaderTypes[shaderTypeNdx];
                var shaderSpec = new es3fShaderOperatorTests.ShaderDataSpec();
                var shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                var isVertexCase = shaderType == gluShaderProgram.shaderType.VERTEX;

                var name = precisionPrefix + dataTypeStr + '_' + shaderTypeName;

                shaderSpec.numInputs = 3;
                shaderSpec.precision = isBoolCase ? undefined : precision;
                shaderSpec.output = curType;
                shaderSpec.resultScale = isBoolCase ? f(1.0) : isFloatCase ? f(0.5) : isUintCase ? f(0.5) : f(0.1);
                shaderSpec.resultBias = isBoolCase ? f(0.0) : isFloatCase ? f(0.5) : isUintCase ? f(0.0) : f(0.5);
                shaderSpec.referenceScale = shaderSpec.resultScale;
                shaderSpec.referenceBias = shaderSpec.resultBias;

                var rangeMin = isBoolCase ? -1.0 : isFloatCase ? -1.0 : isUintCase ? 0.0 : -5.0;
                var rangeMax = isBoolCase ? 1.0 : isFloatCase ? 1.0 : isUintCase ? 2.0 : 5.0;

                shaderSpec.inputs[0] = new es3fShaderOperatorTests.ShaderValue(gluShaderUtil.DataType.BOOL, f(-1.0), f(1.0));
                shaderSpec.inputs[1] = new es3fShaderOperatorTests.ShaderValue(curType, f(rangeMin), f(rangeMax));
                shaderSpec.inputs[2] = new es3fShaderOperatorTests.ShaderValue(curType, f(rangeMin), f(rangeMax));

                selectionGroup.addChild(new es3fShaderOperatorTests.ShaderOperatorCase(name, '', isVertexCase, evalFunc, 'res = in0 ? in1 : in2;', shaderSpec));
            }
        }
    }

    // The sequence operator (comma).
    /** @type {tcuTestCase.DeqpTest} */ var sequenceGroup = new tcuTestCase.DeqpTest('sequence', 'sequence');
    this.addChild(sequenceGroup);

    /** @type {tcuTestCase.DeqpTest} */ var sequenceNoSideEffGroup = new tcuTestCase.DeqpTest('no_side_effects', 'Sequence tests without side-effects');
    /** @type {tcuTestCase.DeqpTest} */ var sequenceSideEffGroup = new tcuTestCase.DeqpTest('side_effects', 'Sequence tests with side-effects');
    sequenceGroup.addChild(sequenceNoSideEffGroup);
    sequenceGroup.addChild(sequenceSideEffGroup);

    /**
     * @struct
     * @constructor
     * @param {boolean} containsSideEffects
     * @param {string} caseName
     * @param {string} expressionStr
     * @param {number} numInputs
     * @param {Array<gluShaderUtil.DataType>} inputTypes
     * @param {gluShaderUtil.DataType} resultType
     * @param {glsShaderRenderCase.ShaderEvalFunc} evalFunc
     */
     var SequenceCase = function(containsSideEffects, caseName, expressionStr, numInputs, inputTypes, resultType, evalFunc) {
         /** @type {boolean} */ this.containsSideEffects = containsSideEffects;
         /** @type {string} */ this.caseName = caseName;
         /** @type {string} */ this.expressionStr = expressionStr;
         /** @type {number} */ this.numInputs = numInputs;
         /** @type {Array<gluShaderUtil.DataType>} */ this.inputTypes = inputTypes;
         /** @type {gluShaderUtil.DataType} */ this.resultType = resultType;
         /** @type {glsShaderRenderCase.ShaderEvalFunc} */ this.evalFunc = evalFunc;
    };

    /** @type {Array<SequenceCase>} */ var s_sequenceCases = [
        new SequenceCase(false, 'vec4', 'in0, in2 + in1, in1 + in0', 3, [gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.FLOAT_VEC4], gluShaderUtil.DataType.FLOAT_VEC4, es3fShaderOperatorTests.evalSequenceNoSideEffCase0),
        new SequenceCase(false, 'float_uint', 'in0 + in2, in1 + in1', 3, [gluShaderUtil.DataType.FLOAT, gluShaderUtil.DataType.UINT, gluShaderUtil.DataType.FLOAT], gluShaderUtil.DataType.UINT, es3fShaderOperatorTests.evalSequenceNoSideEffCase1),
        new SequenceCase(false, 'bool_vec2', 'in0 && in1, in0, ivec2(vec2(in0) + in2)', 3, [gluShaderUtil.DataType.BOOL, gluShaderUtil.DataType.BOOL, gluShaderUtil.DataType.FLOAT_VEC2], gluShaderUtil.DataType.INT_VEC2, es3fShaderOperatorTests.evalSequenceNoSideEffCase2),
        new SequenceCase(false, 'vec4_ivec4_bvec4', 'in0 + vec4(in1), in2, in1', 3, [gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.INT_VEC4, gluShaderUtil.DataType.BOOL_VEC4], gluShaderUtil.DataType.INT_VEC4, es3fShaderOperatorTests.evalSequenceNoSideEffCase3),

        new SequenceCase(true, 'vec4', 'in0++, in1 = in0 + in2, in2 = in1', 3, [gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.FLOAT_VEC4], gluShaderUtil.DataType.FLOAT_VEC4, es3fShaderOperatorTests.evalSequenceSideEffCase0),
        new SequenceCase(true, 'float_uint', 'in1++, in0 = float(in1), in1 = uint(in0 + in2)', 3, [gluShaderUtil.DataType.FLOAT, gluShaderUtil.DataType.UINT, gluShaderUtil.DataType.FLOAT], gluShaderUtil.DataType.UINT, es3fShaderOperatorTests.evalSequenceSideEffCase1),
        new SequenceCase(true, 'bool_vec2', 'in1 = in0, in2++, in2 = in2 + vec2(in1), ivec2(in2)', 3, [gluShaderUtil.DataType.BOOL, gluShaderUtil.DataType.BOOL, gluShaderUtil.DataType.FLOAT_VEC2], gluShaderUtil.DataType.INT_VEC2, es3fShaderOperatorTests.evalSequenceSideEffCase2),
        new SequenceCase(true, 'vec4_ivec4_bvec4', 'in0 = in0 + vec4(in2), in1 = in1 + ivec4(in0), in1++', 3, [gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.INT_VEC4, gluShaderUtil.DataType.BOOL_VEC4], gluShaderUtil.DataType.INT_VEC4, es3fShaderOperatorTests.evalSequenceSideEffCase3)
    ];

    for (var caseNdx = 0; caseNdx < s_sequenceCases.length; caseNdx++) {
        for (var precision in gluShaderUtil.precision) {
            for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                /** @type {gluShaderProgram.shaderType} */ var shaderType = s_shaderTypes[shaderTypeNdx];
                /** @type {es3fShaderOperatorTests.ShaderDataSpec} */ var shaderSpec = new es3fShaderOperatorTests.ShaderDataSpec();
                /** @type {string} */ var shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                /** @type {boolean} */ var isVertexCase = shaderType === gluShaderProgram.shaderType.VERTEX;

                /** @type {string} */ var name = gluShaderUtil.getPrecisionName(gluShaderUtil.precision[precision]) + '_' + s_sequenceCases[caseNdx].caseName + '_' + shaderTypeName;

                shaderSpec.numInputs = s_sequenceCases[caseNdx].numInputs;
                shaderSpec.precision = gluShaderUtil.precision[precision];
                shaderSpec.output = s_sequenceCases[caseNdx].resultType;
                shaderSpec.resultScale = f(0.5);
                shaderSpec.resultBias = f(0.0);
                shaderSpec.referenceScale = shaderSpec.resultScale;
                shaderSpec.referenceBias = shaderSpec.resultBias;

                for (var inputNdx = 0; inputNdx < s_sequenceCases[caseNdx].numInputs; inputNdx++) {
                    /** @type {gluShaderUtil.DataType} */ var type = s_sequenceCases[caseNdx].inputTypes[inputNdx];
                    /** @type {es3fShaderOperatorTests.FloatScalar} */ var rangeMin = gluShaderUtil.isDataTypeFloatOrVec(type) ?
                        f(-0.5) : gluShaderUtil.isDataTypeIntOrIVec(type) ?
                        f(-2.0) : gluShaderUtil.isDataTypeUintOrUVec(type) ?
                        f(0.0) : f(-1.0);

                    /** @type {es3fShaderOperatorTests.FloatScalar} */ var rangeMax = gluShaderUtil.isDataTypeFloatOrVec(type) ?
                        f(0.5) : gluShaderUtil.isDataTypeIntOrIVec(type) ?
                        f(2.0) : gluShaderUtil.isDataTypeUintOrUVec(type) ?
                        f(2.0) : f(1.0);

                    shaderSpec.inputs[inputNdx] = new es3fShaderOperatorTests.ShaderValue(type, rangeMin, rangeMax);
                }

                /** @type {string} */ var expression = 'res = (' + s_sequenceCases[caseNdx].expressionStr + ');';

                if (s_sequenceCases[caseNdx].containsSideEffects)
                    sequenceSideEffGroup.addChild(new es3fShaderOperatorTests.ShaderOperatorCase(name, '', isVertexCase, s_sequenceCases[caseNdx].evalFunc, expression, shaderSpec));
                else
                    sequenceNoSideEffGroup.addChild(new es3fShaderOperatorTests.ShaderOperatorCase(name, '', isVertexCase, s_sequenceCases[caseNdx].evalFunc, expression, shaderSpec));
            }
        }
    }

};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fShaderOperatorTests.run = function(context, range) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fShaderOperatorTests.ShaderOperatorTests());

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
        testFailedOptions('Failed to es3fShaderOperatorTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});

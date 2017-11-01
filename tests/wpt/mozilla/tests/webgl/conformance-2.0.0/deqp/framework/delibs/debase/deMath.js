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
goog.provide('framework.delibs.debase.deMath');

/** @typedef { (Int8Array|Uint8Array|Uint8ClampedArray|Int16Array|Uint16Array|Int32Array|Uint32Array|Float32Array|Float64Array) } */
goog.TypedArray;

/** @typedef { (Array<number>|Array<boolean>|goog.TypedArray) } */
goog.NumberArray;

goog.scope(function() {

var deMath = framework.delibs.debase.deMath;

/** @const */ deMath.INT32_SIZE = 4;

deMath.deInRange32 = function(a, mn, mx) {
    return (a >= mn) && (a <= mx);
};

deMath.deInBounds32 = function(a, mn, mx) {
    return (a >= mn) && (a < mx);
};

/**
 * @param {number} a
 * @return {number}
 */
deMath.deFloatFrac = function(a) { return a - Math.floor(a); };

/**
 * Transform a 64-bit float number into a 32-bit float number.
 * Native dEQP uses 32-bit numbers, so sometimes 64-bit floating numbers in JS should be transformed into 32-bit ones to ensure the correctness of the result.
 * @param {number} a
 * @return {number}
 */
deMath.toFloat32 = (function() {
    var FLOAT32ARRAY1 = new Float32Array(1);
    return function(a) {
        FLOAT32ARRAY1[0] = a;
        return FLOAT32ARRAY1[0];
    };
})();

/** @const */ deMath.INV_LOG_2_FLOAT32 = deMath.toFloat32(1.44269504089); /** 1.0 / log_e(2.0) */

/**
 * Check if a value is a power-of-two.
 * @param {number} a Input value.
 * @return {boolean} return True if input is a power-of-two value, false otherwise.
 * (Also returns true for zero).
 */
deMath.deIsPowerOfTwo32 = function(a) {
    return ((a & (a - 1)) == 0);
};

/**
 * Align an integer to given power-of-two size.
 * @param {number} val The number to align.
 * @param {number} align The size to align to.
 * @return {number} The aligned value
 */
deMath.deAlign32 = function(val, align) {
    if (!deMath.deIsPowerOfTwo32(align))
        throw new Error('Not a power of 2: ' + align);
    return ((val + align - 1) & ~(align - 1)) & 0xFFFFFFFF; //0xFFFFFFFF make sure it returns a 32 bit calculation in 64 bit browsers.
};

/**
 * Compute the bit population count of an integer.
 * @param {number} a
 * @return {number} The number of one bits in
 */
deMath.dePop32 = function(a) {
    /** @type {number} */ var mask0 = 0x55555555; /* 1-bit values. */
    /** @type {number} */ var mask1 = 0x33333333; /* 2-bit values. */
    /** @type {number} */ var mask2 = 0x0f0f0f0f; /* 4-bit values. */
    /** @type {number} */ var mask3 = 0x00ff00ff; /* 8-bit values. */
    /** @type {number} */ var mask4 = 0x0000ffff; /* 16-bit values. */
    /** @type {number} */ var t = a & 0xFFFFFFFF; /* Crop to 32-bit value */
    t = (t & mask0) + ((t >> 1) & mask0);
    t = (t & mask1) + ((t >> 2) & mask1);
    t = (t & mask2) + ((t >> 4) & mask2);
    t = (t & mask3) + ((t >> 8) & mask3);
    t = (t & mask4) + (t >> 16);
    return t;
};

deMath.clamp = function(val, minParm, maxParm) {
    return Math.min(Math.max(val, minParm), maxParm);
};

/**
 * @param {Array<number>} values
 * @param {number} minParm
 * @param {number} maxParm
 * @return {Array<number>}
 */
deMath.clampVector = function(values, minParm, maxParm) {
    var result = [];
    for (var i = 0; i < values.length; i++)
        result.push(deMath.clamp(values[i], minParm, maxParm));
    return result;
};

deMath.imod = function(a, b) {
    var m = a % b;
    return m < 0 ? m + b : m;
};

deMath.mirror = function(a) {
    return a >= 0 ? a : -(1 + a);
};

/**
 * @param {goog.NumberArray} a Source array
 * @param {goog.NumberArray} indices
 * @return {Array<number>} Swizzled array
 */
deMath.swizzle = function(a, indices) {
    if (!indices.length)
        throw new Error('Argument must be an array');
    var dst = [];
    for (var i = 0; i < indices.length; i++)
        dst.push(a[indices[i]]);
    return dst;
};

/**
 * Shift left elements of array a by elements of array b
 * dst[n] a[n] << b[n]
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>} Result array
 */
deMath.arrayShiftLeft = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] << b[i]);
    return dst;
};

/**
 * Multiply two vectors, element by element
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>} Result array
 */

deMath.multiply = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] * b[i]);
    return dst;
};

/**
 * Divide two vectors, element by element
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>} Result array
 * @throws {Error}
 */

deMath.divide = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++) {
        if (b[i] === 0)
            throw new Error('Division by 0');
        dst.push(a[i] / b[i]);
    }
    return dst;
};

/**
 * Divide vector by a scalar
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>} Result array
 */
deMath.divideScale = function(a, b) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] / b);
    return dst;
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
deMath.mod = function(a, b) {
    return a - b * Math.floor(a / b);
};

/**
 * Modulus vector by a scalar
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>} Result array
 */
deMath.modScale = function(a, b) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(deMath.mod(a[i], b));
    return dst;
};

/**
 * Multiply vector by a scalar
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>} Result array
 */
deMath.scale = function(a, b) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] * b);
    return dst;
};

/**
 * Add vector and scalar, element by element
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>} Result array
 */
deMath.addScalar = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('First argument must be an array.');
    if (typeof b !== 'number')
        throw new Error('Second argument must be a number.');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] + b);
    return dst;
};

/**
 * Add two vectors, element by element
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>} Result array
 */
deMath.add = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] + b[i]);
    return dst;
};

/**
 * Subtract two vectors, element by element
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>} Result array
 */

deMath.subtract = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] - b[i]);
    return dst;
};

/**
 * Subtract vector and scalar, element by element
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>} Result array
 */
deMath.subScalar = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('First argument must be an array.');
    if (typeof b !== 'number')
        throw new Error('Second argument must be a number.');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] - b);
    return dst;
};

/**
 * Calculate absolute difference between two vectors
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>} abs(diff(a - b))
 */
deMath.absDiff = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(Math.abs(a[i] - b[i]));
    return dst;
};

/**
 * Calculate absolute value of a vector
 * @param {goog.NumberArray} a
 * @return {Array<number>} abs(a)
 */
deMath.abs = function(a) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(Math.abs(a[i]));
    return dst;
};

/**
 * Is a <= b (element by element)?
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<boolean>} Result array of booleans
 */
deMath.lessThanEqual = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i] <= b[i]);
    return dst;
};

/**
 * Is a === b (element by element)?
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {boolean} Result
 */
deMath.equal = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    for (var i = 0; i < a.length; i++) {
        if (a[i] !== b[i])
            return false;
    }
    return true;
};

/**
 * Are all values in the array true?
 * @param {Array<boolean>} a
 * @return {boolean}
 */
deMath.boolAll = function(a) {
    for (var i = 0; i < a.length; i++)
        if (a[i] == false)
            return false;
    return true;
};

/**
 * deMath.assign(a, b) element by element
 * @param {goog.NumberArray} a
 * @return {Array<number>}
 */
deMath.assign = function(a) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(a[i]);
    return dst;
};

/**
 * deMath.max(a, b) element by element
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>}
 */
deMath.max = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(Math.max(a[i], b[i]));
    return dst;
};

/**
 * deMath.min(a, b) element by element
 * @param {goog.NumberArray} a
 * @param {goog.NumberArray} b
 * @return {Array<number>}
 */
deMath.min = function(a, b) {
    if (a.length != b.length)
        throw new Error('Arrays must have the same size');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(Math.min(a[i], b[i]));
    return dst;
};

// Nearest-even rounding in case of tie (fractional part 0.5), otherwise ordinary rounding.
deMath.rint = function(a) {
    var floorVal = Math.floor(a);
    var fracVal = a - floorVal;

    if (fracVal != 0.5)
        return Math.round(a); // Ordinary case.

    var roundUp = (floorVal % 2) != 0;

    return floorVal + (roundUp ? 1 : 0);
};

/**
 * wrap the number, so that it fits in the range [minValue, maxValue]
 * @param {number} v
 * @param {number} minValue
 * @param {number} maxValue
 * @return {number}
 */
deMath.wrap = function(v, minValue, maxValue) {
    var range = maxValue - minValue + 1;

    if (v < minValue) {
        v += range * (Math.floor((minValue - v) / range) + 1);
    }
    return minValue + Math.floor((v - minValue) % range);
};

/**
 * Round number to int by dropping fractional part
 * it is equivalent of GLSL int() constructor
 * @param {number} a
 * @return {number}
 */
deMath.intCast = function(a) {
    var v;
    if (a >= 0)
        v = Math.floor(a);
    else
        v = Math.ceil(a);
    return deMath.wrap(v, -0x80000000, 0x7FFFFFFF);
};

/**
 * Round number to uint by dropping fractional part
 * it is equivalent of GLSL uint() constructor
 * @param {number} a
 * @return {number}
 */
deMath.uintCast = function(a) {
    var v;
    if (a >= 0)
        v = Math.floor(a);
    else
        v = Math.ceil(a);
    return deMath.wrap(v, 0, 0xFFFFFFFF);
};

/**
 * @param {number} a
 * @return {number}
 */
deMath.logToFloor = function(a) {
    assertMsgOptions(a > 0, 'Value is less or equal than zero', false, true);
    return 31 - deMath.clz32(a);
};

/**
 * Find intersection of two rectangles
 * @param {goog.NumberArray} a Array [x, y, width, height]
 * @param {goog.NumberArray} b Array [x, y, width, height]
 * @return {Array<number>}
 */
deMath.intersect = function(a, b) {
    if (a.length != 4)
        throw new Error('Array "a" must have length 4 but has length: ' + a.length);
    if (b.length != 4)
        throw new Error('Array "b" must have length 4 but has length: ' + b.length);
    var x0 = Math.max(a[0], b[0]);
    var y0 = Math.max(a[1], b[1]);
    var x1 = Math.min(a[0] + a[2], b[0] + b[2]);
    var y1 = Math.min(a[1] + a[3], b[1] + b[3]);
    var w = Math.max(0, x1 - x0);
    var h = Math.max(0, y1 - y0);

    return [x0, y0, w, h];
};

/** deMath.deMathHash
 * @param {number} a
 * @return {number}
 */
deMath.deMathHash = function(a) {
    var key = a;
    key = (key ^ 61) ^ (key >> 16);
    key = key + (key << 3);
    key = key ^ (key >> 4);
    key = key * 0x27d4eb2d; /* prime/odd constant */
    key = key ^ (key >> 15);
    return key;
};

/**
 * Converts a byte array to a number
 * @param {Uint8Array} array
 * @return {number}
 */
deMath.arrayToNumber = function(array) {
    /** @type {number} */ var result = 0;

    for (var ndx = 0; ndx < array.length; ndx++) {
        result += array[ndx] * Math.pow(256, ndx);
    }

    return result;
};

/**
 * Fills a byte array with a number
 * @param {Uint8Array} array Output array (already resized)
 * @param {number} number
 */
deMath.numberToArray = function(array, number) {
    for (var byteNdx = 0; byteNdx < array.length; byteNdx++) {
        /** @type {number} */ var acumzndx = !byteNdx ? number : Math.floor(number / Math.pow(256, byteNdx));
        array[byteNdx] = acumzndx & 0xFF;
    }
};

/**
 * Obtains the bit fragment from a number
 * @param {number} x
 * @param {number} firstNdx
 * @param {number} lastNdx
 * @return {number}
 */
deMath.getBitRange = function(x, firstNdx, lastNdx) {
    var shifted = deMath.shiftRight(x, firstNdx);
    var bitSize = lastNdx - firstNdx;
    var mask;
    if (bitSize < 32)
        mask = (1 << bitSize) - 1;
    else
        mask = Math.pow(2, bitSize) - 1;
    var masked = deMath.binaryAnd(shifted, mask);
    return masked;
};

/**
 * Split a large signed number into low and high 32bit dwords.
 * @param {number} x
 * @return {Array<number>}
 */
deMath.split32 = function(x) {
    var ret = [];
    ret[1] = Math.floor(x / 0x100000000);
    ret[0] = x - ret[1] * 0x100000000;
    return ret;
};

/**
 * Split a signed number's low 32bit dwords into low and high 16bit dwords.
 * @param {number} x
 * @return {Array<number>}
 */
deMath.split16 = function(x) {
    var ret = [];
    x = x & 0xffffffff;
    ret[1] = Math.floor(x / 0x10000);
    ret[0] = x - ret[1] * 0x10000;
    return ret;
};

/**
 * Recontruct a number from high and low 32 bit dwords
 * @param {Array<number>} x
 * @return {number}
 */
deMath.join32 = function(x) {
    var v0 = x[0] >= 0 ? x[0] : 0x100000000 + x[0];
    var v1 = x[1];
    var val = v1 * 0x100000000 + v0;
    return val;
};

//Bit operations with the help of arrays

/**
 * @enum
 */
deMath.BinaryOp = {
    AND: 0,
    OR: 1,
    XOR: 2
};

/**
 * Performs a normal (native) binary operation
 * @param {number} valueA First operand
 * @param {number} valueB Second operand
 * @param {deMath.BinaryOp} operation The desired operation to perform
 * @return {number}
 */
deMath.doNativeBinaryOp = function(valueA, valueB, operation) {
    switch (operation) {
        case deMath.BinaryOp.AND:
            return valueA & valueB;
        case deMath.BinaryOp.OR:
            return valueA | valueB;
        case deMath.BinaryOp.XOR:
            return valueA ^ valueB;
        default:
            throw new Error('Unknown operation: ' + operation);
    }
};

/**
 * Performs a binary operation between two operands
 * with the help of arrays to avoid losing the internal binary representation.
 * @param {number} valueA First operand
 * @param {number} valueB Second operand
 * @param {deMath.BinaryOp} binaryOpParm The desired operation to perform
 * @return {number}
 */
deMath.binaryOp = function(valueA, valueB, binaryOpParm) {
    //quick path if values fit in signed 32 bit range
    if (deMath.deInRange32(valueA, -0x80000000, 0x7FFFFFFF) && deMath.deInRange32(valueB, -0x80000000, 0x7FFFFFFF))
        return deMath.doNativeBinaryOp(valueA, valueB, binaryOpParm);

    var x = deMath.split32(valueA);
    var y = deMath.split32(valueB);
    var z = [];
    for (var i = 0; i < 2; i++)
        z[i] = deMath.doNativeBinaryOp(x[i], y[i], binaryOpParm);
    var ret = deMath.join32(z);
    return ret;
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
deMath.binaryAnd = function(a, b) {
    return deMath.binaryOp(a, b, deMath.BinaryOp.AND);
};

/**
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>}
 */
deMath.binaryAndVecScalar = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('First argument must be an array.');
    if (typeof b !== 'number')
        throw new Error('Second argument must be a number.');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(deMath.binaryOp(a[i], b, deMath.BinaryOp.AND));
    return dst;
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
deMath.binaryOr = function(a, b) {
    return deMath.binaryOp(a, b, deMath.BinaryOp.OR);
};

/**
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>}
 */
deMath.binaryOrVecScalar = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('First argument must be an array.');
    if (typeof b !== 'number')
        throw new Error('Second argument must be a number.');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(deMath.binaryOp(a[i], b, deMath.BinaryOp.OR));
    return dst;
};

/**
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
deMath.binaryXor = function(a, b) {
    return deMath.binaryOp(a, b, deMath.BinaryOp.XOR);
};

/**
 * @param {goog.NumberArray} a
 * @param {number} b
 * @return {Array<number>}
 */
deMath.binaryXorVecScalar = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('First argument must be an array.');
    if (typeof b !== 'number')
        throw new Error('Second argument must be a number.');
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(deMath.binaryOp(a[i], b, deMath.BinaryOp.XOR));
    return dst;
};

/**
 * Performs a binary NOT operation on an operand
 * @param {number} value Operand
 * @return {number}
 */
deMath.binaryNot = function(value) {
    //quick path if value fits in signed 32 bit range
    if (deMath.deInRange32(value, -0x80000000, 0x7FFFFFFF))
        return ~value;

    var x = deMath.split32(value);
    x[0] = ~x[0];
    x[1] = ~x[1];
    var ret = deMath.join32(x);
    return ret;
};

/**
 * Shifts the given value 'steps' bits to the left. Replaces << operator
 * This function should be used if the expected value will be wider than 32-bits.
 * @param {number} value
 * @param {number} steps
 * @return {number}
 */
deMath.shiftLeft = function(value, steps) {
    //quick path
    if (steps < 31) {
        var v = value * (1 << steps);
        if (deMath.deInRange32(v, -0x80000000, 0x7FFFFFFF))
            return v;
    }

    if (steps == 0)
        return value;
    else if (steps < 32) {
        var mask = (1 << 32 - steps) - 1;
        var x = deMath.split32(value);
        var highBits = x[0] & (~mask);
        var y = highBits >> (32 - steps);
        if (highBits < 0) {
            var m = (1 << steps) - 1;
            y &= m;
        }
        var result = [];
        result[0] = x[0] << steps;
        result[1] = x[1] << steps;
        result[1] |= y;

        return deMath.join32(result);
    } else {
        var x = deMath.split32(value);
        var result = [];
        result[0] = 0;
        result[1] = x[0] << steps - 32;
        return deMath.join32(result);
    }
};

/**
 * @param {Array<number>} a
 * @param {number} b
 */
deMath.shiftLeftVecScalar = function(a, b) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(deMath.shiftLeft(a[i], b));
    return dst;
};

/**
 * Shifts the given value 'steps' bits to the right. Replaces >> operator
 * This function should be used if the value is wider than 32-bits
 * @param {number} value
 * @param {number} steps
 * @return {number}
 */
deMath.shiftRight = function(value, steps) {
    //quick path
    if (deMath.deInRange32(value, -0x80000000, 0x7FFFFFFF) && steps < 32)
        return value >> steps;

    if (steps == 0)
        return value;
    else if (steps < 32) {
        if (steps == 0)
            return value;
        var mask = (1 << steps) - 1;
        var x = deMath.split32(value);
        var lowBits = x[1] & mask;
        var result = [];
        var m = (1 << 32 - steps) - 1;
        result[0] = (x[0] >> steps) & m;
        result[1] = x[1] >> steps;
        result[0] |= lowBits << 32 - steps;
        return deMath.join32(result);
    } else {
        var x = deMath.split32(value);
        var result = [];
        result[0] = x[1] >> steps - 32;
        result[1] = value < 0 ? -1 : 0;
        return deMath.join32(result);
    }
};

/**
 * @param {Array<number>} a
 * @param {number} b
 */
deMath.shiftRightVecScalar = function(a, b) {
    var dst = [];
    for (var i = 0; i < a.length; i++)
        dst.push(deMath.shiftRight(a[i], b));
    return dst;
};

/** deMath.logicalAndBool over two arrays of booleans
 * @param {Array<boolean>} a
 * @param {Array<boolean>} b
 * @return {Array<boolean>}
 */
deMath.logicalAndBool = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('The first parameter is not an array: (' + typeof(a) + ')' + a);
    if (!Array.isArray(b))
        throw new Error('The second parameter is not an array: (' + typeof(b) + ')' + b);
    if (a.length != b.length)
        throw new Error('The lengths of the passed arrays are not equivalent. (' + a.length + ' != ' + b.length + ')');

    /** @type {Array<boolean>} */ var result = [];
    for (var i = 0; i < a.length; i++) {
        if (a[i] & b[i])
            result.push(true);
        else
            result.push(false);
    }
    return result;
};

/** deMath.logicalOrBool over two arrays of booleans
 * @param {Array<boolean>} a
 * @param {Array<boolean>} b
 * @return {Array<boolean>}
 */
deMath.logicalOrBool = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('The first parameter is not an array: (' + typeof(a) + ')' + a);
    if (!Array.isArray(b))
        throw new Error('The second parameter is not an array: (' + typeof(b) + ')' + b);
    if (a.length != b.length)
        throw new Error('The lengths of the passed arrays are not equivalent. (' + a.length + ' != ' + b.length + ')');

    /** @type {Array<boolean>} */ var result = [];
    for (var i = 0; i < a.length; i++) {
        if (a[i] | b[i])
            result.push(true);
        else
            result.push(false);
    }
    return result;
};

/** deMath.logicalNotBool over an array of booleans
 * @param {Array<boolean>} a
 * @return {Array<boolean>}
 */
deMath.logicalNotBool = function(a) {
    if (!Array.isArray(a))
        throw new Error('The passed value is not an array: (' + typeof(a) + ')' + a);

    /** @type {Array<boolean>} */ var result = [];
    for (var i = 0; i < a.length; i++)
        result.push(!a[i]);
    return result;
};

/** deMath.greaterThan over two arrays of booleans
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<boolean>}
 */
deMath.greaterThan = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('The first parameter is not an array: (' + typeof(a) + ')' + a);
    if (!Array.isArray(b))
        throw new Error('The second parameter is not an array: (' + typeof(b) + ')' + b);
    if (a.length != b.length)
        throw new Error('The lengths of the passed arrays are not equivalent. (' + a.length + ' != ' + b.length + ')');

    /** @type {Array<boolean>} */ var result = [];
    for (var i = 0; i < a.length; i++)
        result.push(a[i] > b[i]);
    return result;
};

/** deMath.greaterThan over two arrays of booleans
 * @param {Array<number>} a
 * @param {Array<number>} b
 * @return {Array<boolean>}
 */
deMath.greaterThanEqual = function(a, b) {
    if (!Array.isArray(a))
        throw new Error('The first parameter is not an array: (' + typeof(a) + ')' + a);
    if (!Array.isArray(b))
        throw new Error('The second parameter is not an array: (' + typeof(b) + ')' + b);
    if (a.length != b.length)
        throw new Error('The lengths of the passed arrays are not equivalent. (' + a.length + ' != ' + b.length + ')');

    /** @type {Array<boolean>} */ var result = [];
    for (var i = 0; i < a.length; i++)
        result.push(a[i] >= b[i]);
    return result;
};

/**
 * Array of float to array of int (0, 255)
 * @param {Array<number>} a
 * @return {Array<number>}
 */

deMath.toIVec = function(a) {
    /** @type {Array<number>} */ var res = [];
    for (var i = 0; i < a.length; i++)
        res.push(deMath.clamp(Math.floor(a[i] * 255), 0, 255));
    return res;
};

/**
 * @param {number} a
 * @return {number}
 */
 deMath.clz32 = function(a) {
   /** @type {number} */ var maxValue = 2147483648; // max 32 bit number
   /** @type {number} */ var leadingZeros = 0;
   while (a < maxValue) {
     maxValue = maxValue >>> 1;
     leadingZeros++;
   }
   return leadingZeros;
};

/**
 * @param {number} a
 * @param {number} exponent
 * @return {number}
 */
deMath.deLdExp = function(a, exponent) {
    return deMath.ldexp(a, exponent);
};

/**
 * @param {number} a
 * @param {number} exponent
 * @return {number}
 */
deMath.deFloatLdExp = function(a, exponent) {
    return deMath.ldexp(a, exponent);
};

/**
 * @param {number} value
 * @return {Array<number>}
 */
deMath.frexp = (function() {
   var data = new DataView(new ArrayBuffer(8));

   return function(value) {
       if (value === 0) return [value, 0];
       data.setFloat64(0, value);
       var bits = (data.getUint32(0) >>> 20) & 0x7FF;
       if (bits === 0) {
           data.setFloat64(0, value * Math.pow(2, 64));
           bits = ((data.getUint32(0) >>> 20) & 0x7FF) - 64;
       }
       var exponent = bits - 1022,
           mantissa = deMath.ldexp(value, -exponent);
       return [mantissa, exponent];
   }
})();

/**
 * @param {number} mantissa
 * @param {number} exponent
 * @return {number}
 */
deMath.ldexp = function(mantissa, exponent) {
    return exponent > 1023 ? // avoid multiplying by infinity
            mantissa * Math.pow(2, 1023) * Math.pow(2, exponent - 1023) :
            exponent < -1074 ? // avoid multiplying by zero
            mantissa * Math.pow(2, -1074) * Math.pow(2, exponent + 1074) :
            mantissa * Math.pow(2, exponent);
};

/**
 * @param {number} a
 * @return {number}
 */
deMath.deCbrt = function(a) {
    return deMath.deSign(a) * Math.pow(Math.abs(a), 1.0 / 3.0);
};

/**
 * @param {number} x
 * @return {number}
 */
deMath.deSign = function(x) {
    return isNaN(x) ? x : ((x > 0.0) - (x < 0.0));
};

deMath.deFractExp = function(x) {
    var result = {
        significand: x,
        exponent: 0
    };

    if (isFinite(x)) {
        var r = deMath.frexp(x);
        result.exponent = r[1] - 1;
        result.significand = r[0] * 2;
    }
    return result;
};

});

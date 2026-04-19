// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// Some tests regarding conversion to Float16
assert.sameValue(Math.f16round(), NaN);

// Special values
assert.sameValue(Math.f16round(NaN), NaN);
assert.sameValue(Math.f16round(-Infinity), -Infinity);
assert.sameValue(Math.f16round(Infinity), Infinity);
assert.sameValue(Math.f16round(-0), -0);
assert.sameValue(Math.f16round(+0), +0);

// Polyfill function for Float16 conversion, adapted from
// https://github.com/petamoriken/float16/.
// The original license is preseved below.
function toFloat16(num) {
  // MIT License

  // Copyright (c) 2017-2024 Kenta Moriuchi

  // Permission is hereby granted, free of charge, to any person obtaining a copy
  // of this software and associated documentation files (the "Software"), to deal
  // in the Software without restriction, including without limitation the rights
  // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  // copies of the Software, and to permit persons to whom the Software is
  // furnished to do so, subject to the following conditions:

  // The above copyright notice and this permission notice shall be included in all
  // copies or substantial portions of the Software.

  // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  // SOFTWARE.

  const INVERSE_OF_EPSILON = 1 / Number.EPSILON;

  /**
   * rounds to the nearest value;
   * if the number falls midway, it is rounded to the nearest value with an even least significant digit
   * @param {number} num
   * @returns {number}
   */
  function roundTiesToEven(num) {
    return (num + INVERSE_OF_EPSILON) - INVERSE_OF_EPSILON;
  }

  const FLOAT16_MIN_VALUE = 6.103515625e-05;
  const FLOAT16_MAX_VALUE = 65504;
  const FLOAT16_EPSILON = 0.0009765625;

  const FLOAT16_EPSILON_MULTIPLIED_BY_FLOAT16_MIN_VALUE = FLOAT16_EPSILON * FLOAT16_MIN_VALUE;
  const FLOAT16_EPSILON_DEVIDED_BY_EPSILON = FLOAT16_EPSILON * INVERSE_OF_EPSILON;

  function roundToFloat16(num) {
    const number = +num;

    // NaN, Infinity, -Infinity, 0, -0
    if (!isFinite(number) || number === 0) {
      return number;
    }

    // finite except 0, -0
    const sign = number > 0 ? 1 : -1;
    const absolute = Math.abs(number);

    // small number
    if (absolute < FLOAT16_MIN_VALUE) {
      return sign * roundTiesToEven(absolute / FLOAT16_EPSILON_MULTIPLIED_BY_FLOAT16_MIN_VALUE) * FLOAT16_EPSILON_MULTIPLIED_BY_FLOAT16_MIN_VALUE;
    }

    const temp = (1 + FLOAT16_EPSILON_DEVIDED_BY_EPSILON) * absolute;
    const result = temp - (temp - absolute);

    // large number
    if (result > FLOAT16_MAX_VALUE || isNaN(result)) {
      return sign * Infinity;
    }

    return sign * result;
  }

  return roundToFloat16(num);
};

// A test on a certain range of numbers, including big numbers, so that
// we get a loss in precision for some of them.
for (var i = 0; i < 64; ++i) {
    var p = Math.pow(2, i) + 1;
    assert.sameValue(Math.f16round(p), toFloat16(p));
    assert.sameValue(Math.f16round(-p), toFloat16(-p));
}

/********************************************
/* Tests on maximal Float16 / Double values *
/*******************************************/
function maxValue(exponentWidth, significandWidth) {
  var n = 0;
  var maxExp = Math.pow(2, exponentWidth - 1) - 1;
  for (var i = significandWidth; i >= 0; i--)
      n += Math.pow(2, maxExp - i);
  return n;
}

var DBL_MAX = maxValue(11, 52);
assert.sameValue(DBL_MAX, Number.MAX_VALUE); // sanity check

// Finite as a double, too big for a float16
assert.sameValue(Math.f16round(DBL_MAX), Infinity);

var FLT16_MAX = maxValue(5, 10);
assert.sameValue(Math.f16round(FLT16_MAX), FLT16_MAX);
assert.sameValue(Math.f16round(65519), FLT16_MAX); // round-nearest rounds down to FLT16_MAX
assert.sameValue(Math.f16round(65520), Infinity); // no longer rounds down to FLT16_MAX

/*********************************************************
/******* Tests on denormalizations and roundings *********
/********************************************************/
function minValue(exponentWidth, significandWidth) {
  return Math.pow(2, -(Math.pow(2, exponentWidth - 1) - 2) - significandWidth);
}

var DBL_MIN = Math.pow(2, -1074);
assert.sameValue(DBL_MIN, Number.MIN_VALUE); // sanity check

// Too small for a float16
assert.sameValue(Math.f16round(DBL_MIN), 0);

var FLT16_MIN = minValue(5, 10);
assert.sameValue(Math.f16round(FLT16_MIN), FLT16_MIN);

assert.sameValue(Math.f16round(FLT16_MIN / 2), 0); // halfway, round-nearest rounds down to 0 (even)

// FLT16_MIN + a small amount rounds up to FLT16_MIN
// Constant taken from https://github.com/petamoriken/float16/
assert.sameValue(Math.f16round(2.980232238769531911744490042422139897126953655970282852649688720703125e-8), FLT16_MIN);

assert.sameValue(Math.f16round(-FLT16_MIN), -FLT16_MIN);
assert.sameValue(Math.f16round(-FLT16_MIN / 2), -0); // halfway, round-nearest rounds up to -0 (even)
// -FLT16_MIN - a small amount rounds down to -FLT16_MIN
// Constant taken from https://github.com/petamoriken/float16/
assert.sameValue(Math.f16round(2.980232238769531911744490042422139897126953655970282852649688720703125e-8), FLT16_MIN);

// Test some constants from https://github.com/petamoriken/float16/
assert.sameValue(Math.f16round(0.499994), 0.5);
assert.sameValue(Math.f16round(1.337), 1.3369140625);

// This will round incorrectly if the implementation first rounds to Float32,
// then to Float16
assert.sameValue(Math.f16round(1.00048828125000022204), 1.0009765625);


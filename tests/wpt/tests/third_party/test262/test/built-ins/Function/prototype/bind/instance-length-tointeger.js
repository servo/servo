// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-function.prototype.bind
description: >
  "length" value of a bound function is non-negative integer.
  ToInteger is performed on "length" value of target function.
info: |
  Function.prototype.bind ( thisArg, ...args )

  [...]
  5. Let targetHasLength be ? HasOwnProperty(Target, "length").
  6. If targetHasLength is true, then
    a. Let targetLen be ? Get(Target, "length").
    b. If Type(targetLen) is Number, then
       i. If targetLen is +∞𝔽, set L to +∞.
       ii. Else if targetLen is -∞𝔽, set L to 0.
       iii. Else,
            1. Let targetLenAsInt be ! ToIntegerOrInfinity(targetLen).
            2. Assert: targetLenAsInt is finite.
            3. Let argCount be the number of elements in args.
            4. Set L to max(targetLenAsInt - argCount, 0).
  7. Perform ! SetFunctionLength(F, L).
  [...]

  ToInteger ( argument )

  1. Let number be ? ToNumber(argument).
  2. If number is NaN, +0, or -0, return +0.
  3. If number is +∞ or -∞, return number.
  4. Let integer be the Number value that is the same sign as number and whose magnitude is floor(abs(number)).
  5. If integer is -0, return +0.
  6. Return integer.
---*/

function fn() {}

Object.defineProperty(fn, "length", {value: NaN});
assert.sameValue(fn.bind().length, 0);

Object.defineProperty(fn, "length", {value: -0});
assert.sameValue(fn.bind().length, 0);

Object.defineProperty(fn, "length", {value: Infinity});
assert.sameValue(fn.bind().length, Infinity, "target length of infinity, zero bound arguments");
assert.sameValue(fn.bind(0, 0).length, Infinity, "target length of infinity, one bound argument");

Object.defineProperty(fn, "length", {value: -Infinity});
assert.sameValue(fn.bind().length, 0, "target length of negative infinity, zero bound arguments");
assert.sameValue(fn.bind(0, 0).length, 0, "target length of negative infinity, one bound argument");

Object.defineProperty(fn, "length", {value: 3.66});
assert.sameValue(fn.bind().length, 3);

Object.defineProperty(fn, "length", {value: -0.77});
assert.sameValue(fn.bind().length, 0);

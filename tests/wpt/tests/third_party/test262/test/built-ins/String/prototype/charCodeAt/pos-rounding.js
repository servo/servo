// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.charcodeat
description: Rounding of the provided "pos" number
info: |
  [...]
  3. Let position be ? ToInteger(pos).
  [...]

  7.1.4 ToInteger

  1. Let number be ? ToNumber(argument).
  2. If number is NaN, return +0.
  3. If number is +0, -0, +∞, or -∞, return number.
  4. Return the number value that is the same sign as number and whose
     magnitude is floor(abs(number)). 
---*/

var aCode = 97;
var bCode = 98;

assert.sameValue('abc'.charCodeAt(-0.99999), aCode, '-0.99999');
assert.sameValue('abc'.charCodeAt(-0.00001), aCode, '-0.00001');
assert.sameValue('abc'.charCodeAt(0.00001), aCode, '0.00001');
assert.sameValue('abc'.charCodeAt(0.99999), aCode, '0.99999');
assert.sameValue('abc'.charCodeAt(1.00001), bCode, '1.00001');
assert.sameValue('abc'.charCodeAt(1.99999), bCode, '1.99999');

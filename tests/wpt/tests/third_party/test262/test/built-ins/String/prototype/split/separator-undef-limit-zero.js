// Copyright (C) 2020 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: Separator is undefined, limit is zero, return a new empty array
info: |
  ...
  3. Let S be ? ToString(O).
  4. Let A be ! ArrayCreate(0).
  ...
  6. If limit is undefined, let lim be 2**32 - 1; else let lim be ? ToUint32(limit).
  7. Let R be ? ToString(separator).
  8. If lim = 0, return A.

  ToUint32 ( argument )

  1. Let number be ? ToNumber(argument).
  2. If number is NaN, +0, -0, +∞, or -∞, return +0.
  3. Let int be the Number value that is the same sign as number and whose magnitude is floor(abs(number)).
  4. Let int32bit be int modulo 2**32.
  5. Return int32bit.
---*/

var str = 'undefined is not a function';

var result = str.split(undefined, 0);

assert.sameValue(Array.isArray(result), true, 'result is array');
assert.sameValue(result.length, 0, 'result.length');

result = str.split(undefined, false);

assert.sameValue(Array.isArray(result), true, 'boolean, result is array');
assert.sameValue(result.length, 0, 'boolean, result.length');

result = str.split(undefined, null);

assert.sameValue(Array.isArray(result), true, 'null, result is array');
assert.sameValue(result.length, 0, 'null, result.length');

result = str.split(undefined, {valueOf() { return undefined; }});

assert.sameValue(Array.isArray(result), true, 'obj > undefined, result is array');
assert.sameValue(result.length, 0, 'obj > undefined, result.length');

result = str.split(undefined, {valueOf() { return 0; }});

assert.sameValue(Array.isArray(result), true, 'obj > 0, result is array');
assert.sameValue(result.length, 0, 'obj > 0, result.length');

result = str.split(undefined, NaN);

assert.sameValue(Array.isArray(result), true, 'NaN, result is array');
assert.sameValue(result.length, 0, 'NaN, result.length');

result = str.split(undefined, 2 ** 32);

assert.sameValue(Array.isArray(result), true, '2 ** 32, result is array');
assert.sameValue(result.length, 0, '2 ** 32, result.length');

result = str.split(undefined, 2 ** 33);

assert.sameValue(Array.isArray(result), true, '2 ** 33, result is array');
assert.sameValue(result.length, 0, '2 ** 33, result.length');

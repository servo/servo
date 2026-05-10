// Copyright (C) 2020 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: Separator is undefined, limit is a positive number, return a new array with the string 
info: |
  ...
  3. Let S be ? ToString(O).
  4. Let A be ! ArrayCreate(0).
  ...
  6. If limit is undefined, let lim be 232 - 1; else let lim be ? ToUint32(limit).
  7. Let R be ? ToString(separator).
  8. If lim = 0, return A.
  9. If separator is undefined, then
    a. Perform ! CreateDataPropertyOrThrow(A, "0", S).
    b. Return A.
---*/

var str = 'undefined is not a function';

var result = str.split(undefined, 1);

assert.sameValue(Array.isArray(result), true, '1, result is array');
assert.sameValue(result.length, 1, '1, result.length');
assert.sameValue(result[0], str, '1, [0] is the same string');

result = str.split(undefined, 2);

assert.sameValue(Array.isArray(result), true, '2, result is array');
assert.sameValue(result.length, 1, '2, result.length');
assert.sameValue(result[0], str, '2, [0] is the same string');

result = str.split(undefined, undefined);

assert.sameValue(Array.isArray(result), true, 'undefined, result is array');
assert.sameValue(result.length, 1, 'undefined, result.length');
assert.sameValue(result[0], str, 'undefined, [0] is the same string');

result = str.split(undefined, true);

assert.sameValue(Array.isArray(result), true, 'boolean, result is array');
assert.sameValue(result.length, 1, 'boolean, result.length');
assert.sameValue(result[0], str, 'boolean, [0] is the same string');

result = str.split(undefined, 2 ** 32 + 1);

assert.sameValue(Array.isArray(result), true, 'ToUint32 2 ** 32 + 1, result is array');
assert.sameValue(result.length, 1, 'ToUint32 2 ** 32 + 1, result.length');
assert.sameValue(result[0], str, 'ToUint32 2 ** 32 + 1, [0] is the same string');

result = str.split(undefined, 2 ** 31);

assert.sameValue(Array.isArray(result), true, 'ToUint32 2 ** 31, result is array');
assert.sameValue(result.length, 1, 'ToUint32 2 ** 31, result.length');
assert.sameValue(result[0], str, 'ToUint32 2 ** 31, [0] is the same string');

result = str.split(undefined, 2 ** 16);

assert.sameValue(Array.isArray(result), true, 'ToUint32 2 ** 16, result is array');
assert.sameValue(result.length, 1, 'ToUint32 2 ** 16, result.length');
assert.sameValue(result[0], str, 'ToUint32 2 ** 16, [0] is the same string');

result = str.split(undefined, {valueOf() { return 1; }});

assert.sameValue(Array.isArray(result), true, 'boolean, result is array');
assert.sameValue(result.length, 1, 'boolean, result.length');
assert.sameValue(result[0], str, 'boolean, [0] is the same string');

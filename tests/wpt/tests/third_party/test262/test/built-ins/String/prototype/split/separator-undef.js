// Copyright (C) 2020 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.split
description: Separator is undefined, return a new array with the string
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

var result = str.split();

assert.sameValue(Array.isArray(result), true, 'implicit separator, result is array');
assert.sameValue(result.length, 1, 'implicit separator, result.length');
assert.sameValue(result[0], str, 'implicit separator, [0] is the same string');

result = str.split(undefined);

assert.sameValue(Array.isArray(result), true, 'explicit separator, result is array');
assert.sameValue(result.length, 1, 'explicit separator, result.length');
assert.sameValue(result[0], str, 'explicit separator, [0] is the same string');

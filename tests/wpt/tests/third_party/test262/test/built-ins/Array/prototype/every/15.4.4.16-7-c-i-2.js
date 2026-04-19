// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - element to be retrieved is own data
    property on an Array
---*/

var called = 0;

function callbackfn(val, idx, obj) {
  called++;
  return val === 11;
}

assert([11].every(callbackfn), '[11].every(callbackfn) !== true');
assert.sameValue(called, 1, 'called');

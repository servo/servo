// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - value of 'length' is boundary value (2^32
    + 1)
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var obj = {
  0: 11,
  1: 9,
  length: 4294967297
};

assert.sameValue(Array.prototype.every.call(obj, callbackfn1), false, 'Array.prototype.every.call(obj, callbackfn1)');
assert.sameValue(Array.prototype.every.call(obj, callbackfn2), false, 'Array.prototype.every.call(obj, callbackfn2)');

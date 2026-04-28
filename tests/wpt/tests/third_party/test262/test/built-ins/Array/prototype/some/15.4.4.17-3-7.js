// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - value of 'length' is a number (value is
    negative)
---*/

function callbackfn1(val, idx, obj) {
  return val > 10;
}

function callbackfn2(val, idx, obj) {
  return val > 11;
}

var obj = {
  0: 9,
  1: 11,
  2: 12,
  length: -4294967294
};

assert.sameValue(Array.prototype.some.call(obj, callbackfn1), false, 'Array.prototype.some.call(obj, callbackfn1)');
assert.sameValue(Array.prototype.some.call(obj, callbackfn2), false, 'Array.prototype.some.call(obj, callbackfn2)');

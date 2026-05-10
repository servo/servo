// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - callbackfn called with correct parameters
    (kValue is correct)
---*/

var firstIndex = false;
var secondIndex = false;

function callbackfn(val, idx, obj) {
  if (idx === 0) {
    firstIndex = (val === 11);
    return false;
  }
  if (idx === 1) {
    secondIndex = (val === 12);
    return false;
  }
}

var obj = {
  0: 11,
  1: 12,
  length: 2
};

assert.sameValue(Array.prototype.some.call(obj, callbackfn), false, 'Array.prototype.some.call(obj, callbackfn)');
assert(firstIndex, 'firstIndex !== true');
assert(secondIndex, 'secondIndex !== true');

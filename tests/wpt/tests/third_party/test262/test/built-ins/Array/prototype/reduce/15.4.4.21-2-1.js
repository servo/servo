// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: >
    Array.prototype.reduce - 'length' is own data property on an
    Array-like object
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return (obj.length === 2);
}

var obj = {
  0: 12,
  1: 11,
  2: 9,
  length: 2
};

assert.sameValue(Array.prototype.reduce.call(obj, callbackfn, 1), true, 'Array.prototype.reduce.call(obj, callbackfn, 1)');

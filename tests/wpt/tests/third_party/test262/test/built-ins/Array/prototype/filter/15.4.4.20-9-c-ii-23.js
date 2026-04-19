// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - callbackfn called with correct parameters
    (this object O is correct)
---*/

var obj = {
  0: 11,
  length: 2
};

function callbackfn(val, idx, o) {
  return obj === o;
}

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 1, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');

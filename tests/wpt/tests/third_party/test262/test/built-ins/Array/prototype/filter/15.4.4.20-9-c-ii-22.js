// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - callbackfn called with correct parameters
    (the index k is correct)
---*/

function callbackfn(val, idx, obj) {
  if (val === 11) {
    return idx === 0;
  }

  if (val === 12) {
    return idx === 1;
  }

  return false;
}

var obj = {
  0: 11,
  1: 12,
  length: 2
};
var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 2, 'newArr.length');
assert.sameValue(newArr[0], 11, 'newArr[0]');
assert.sameValue(newArr[1], 12, 'newArr[1]');

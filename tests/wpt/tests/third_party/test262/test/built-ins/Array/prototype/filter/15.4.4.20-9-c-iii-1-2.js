// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: >
    Array.prototype.filter - value of returned array element can be
    overwritten
---*/

function callbackfn(val, idx, obj) {
  return true;
}

var obj = {
  0: 11,
  1: 9,
  length: 2
};
var newArr = Array.prototype.filter.call(obj, callbackfn);

var tempVal = newArr[1];
newArr[1] += 1;

assert.notSameValue(newArr[1], tempVal, 'newArr[1]');

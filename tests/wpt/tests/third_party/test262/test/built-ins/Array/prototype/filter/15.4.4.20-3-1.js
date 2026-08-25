// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter - value of 'length' is undefined
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return true;
}

var obj = {
  0: 0,
  1: 1,
  length: undefined
};
var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr.length, 0, 'newArr.length');
assert.sameValue(accessed, false, 'accessed');

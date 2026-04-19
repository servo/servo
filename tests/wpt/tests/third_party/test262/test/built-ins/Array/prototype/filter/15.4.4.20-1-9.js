// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to Function object
---*/

function callbackfn(val, idx, obj) {
  return obj instanceof Function;
}

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

var newArr = Array.prototype.filter.call(obj, callbackfn);

assert.sameValue(newArr[0], 11, 'newArr[0]');
assert.sameValue(newArr[1], 9, 'newArr[1]');

// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduce
description: Array.prototype.reduce applied to Function object
---*/

function callbackfn(prevVal, curVal, idx, obj) {
  return obj instanceof Function;
}

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

assert(Array.prototype.reduce.call(obj, callbackfn, 1), 'Array.prototype.reduce.call(obj, callbackfn, 1) !== true');

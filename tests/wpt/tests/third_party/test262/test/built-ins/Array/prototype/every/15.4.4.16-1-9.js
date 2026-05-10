// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: Array.prototype.every applied to Function object
---*/

function callbackfn(val, idx, obj) {
  return !(obj instanceof Function);
}

var obj = function(a, b) {
  return a + b;
};
obj[0] = 11;
obj[1] = 9;

assert.sameValue(Array.prototype.every.call(obj, callbackfn), false, 'Array.prototype.every.call(obj, callbackfn)');

// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - 'length' is an own accessor property
    without a get function on an Array-like object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

var obj = {
  0: 11,
  1: 12
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

assert.sameValue(Array.prototype.some.call(obj, callbackfn), false, 'Array.prototype.some.call(obj, callbackfn)');
assert.sameValue(accessed, false, 'accessed');

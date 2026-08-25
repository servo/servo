// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.some
description: >
    Array.prototype.some - 'length' is own accessor property without a
    get function that overrides an inherited accessor property on an
    Array-like object
---*/

var accessed = false;

function callbackfn(val, idx, obj) {
  accessed = true;
  return val > 10;
}

Object.defineProperty(Object.prototype, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

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

// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.every
description: >
    Array.prototype.every - 'length' is own accessor property without
    a get function that overrides an inherited accessor property
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
  0: 9,
  1: 8
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

assert(Array.prototype.every.call(obj, callbackfn), 'Array.prototype.every.call(obj, callbackfn) !== true');
assert.sameValue(accessed, false, 'accessed');

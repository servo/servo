// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.map
description: >
    Array.prototype.map - applied to the Array-like object when
    'length' is own accessor property without a get function that
    overrides an inherited accessor property
---*/

function callbackfn(val, idx, obj) {
  return val > 10;
}

Object.defineProperty(Object.prototype, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

var obj = {
  0: 12,
  1: 11
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

var testResult = Array.prototype.map.call(obj, callbackfn);

assert.sameValue(testResult.length, 0, 'testResult.length');

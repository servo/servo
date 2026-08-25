// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is own accessor property
    without a get function that overrides an inherited accessor
    property on an Array-like object
---*/

Object.defineProperty(Object.prototype, "length", {
  get: function() {
    return 20;
  },
  configurable: true
});

var obj = {
  1: 1
};
Object.defineProperty(obj, "length", {
  set: function() {},
  configurable: true
});

assert.sameValue(Array.prototype.lastIndexOf.call(obj, 1), -1, 'Array.prototype.lastIndexOf.call(obj, 1)');

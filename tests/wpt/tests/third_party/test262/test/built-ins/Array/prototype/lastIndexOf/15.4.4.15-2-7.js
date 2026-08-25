// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is own accessor property on
    an Array-like object
---*/

var obj = {
  1: true,
  2: false
};

Object.defineProperty(obj, "length", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(Array.prototype.lastIndexOf.call(obj, true), 1, 'Array.prototype.lastIndexOf.call(obj, true)');
assert.sameValue(Array.prototype.lastIndexOf.call(obj, false), -1, 'Array.prototype.lastIndexOf.call(obj, false)');

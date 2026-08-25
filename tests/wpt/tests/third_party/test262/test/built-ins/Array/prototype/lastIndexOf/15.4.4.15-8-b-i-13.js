// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - element to be retrieved is own
    accessor property that overrides an inherited accessor property on
    an Array
---*/

var arr = [];

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return false;
  },
  configurable: true
});

Object.defineProperty(arr, "0", {
  get: function() {
    return true;
  },
  configurable: true
});

assert.sameValue(arr.lastIndexOf(true), 0, 'arr.lastIndexOf(true)');

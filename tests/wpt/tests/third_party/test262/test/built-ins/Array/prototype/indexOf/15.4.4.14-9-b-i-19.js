// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - element to be retrieved is own accessor
    property without a get function that overrides an inherited
    accessor property on an Array
---*/

var arr = [];

Object.defineProperty(arr, "0", {
  set: function() {},
  configurable: true
});

Object.defineProperty(Array.prototype, "0", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(arr.indexOf(undefined), 0, 'arr.indexOf(undefined)');

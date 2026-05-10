// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - element to be retrieved is own
    accessor property on an Array
---*/

var arr = [, , , ];
Object.defineProperty(arr, "0", {
  get: function() {
    return 0;
  },
  configurable: true
});

Object.defineProperty(arr, "1", {
  get: function() {
    return 1;
  },
  configurable: true
});

Object.defineProperty(arr, "2", {
  get: function() {
    return 2;
  },
  configurable: true
});

assert.sameValue(arr.lastIndexOf(0), 0, 'arr.lastIndexOf(0)');
assert.sameValue(arr.lastIndexOf(1), 1, 'arr.lastIndexOf(1)');
assert.sameValue(arr.lastIndexOf(2), 2, 'arr.lastIndexOf(2)');

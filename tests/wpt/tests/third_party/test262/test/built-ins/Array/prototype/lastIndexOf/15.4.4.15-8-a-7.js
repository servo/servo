// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf -  properties added into own object
    after current position are visited on an Array-like object
---*/

var arr = {
  length: 8
};

Object.defineProperty(arr, "4", {
  get: function() {
    Object.defineProperty(arr, "1", {
      get: function() {
        return 1;
      },
      configurable: true
    });
    return 0;
  },
  configurable: true
});

assert.sameValue(Array.prototype.lastIndexOf.call(arr, 1), 1, 'Array.prototype.lastIndexOf.call(arr, 1)');

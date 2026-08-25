// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf -  properties added into own object
    after current position are visited on an Array
---*/

var arr = [0, , 2];

Object.defineProperty(arr, "2", {
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

assert.sameValue(arr.lastIndexOf(1), 1, 'arr.lastIndexOf(1)');

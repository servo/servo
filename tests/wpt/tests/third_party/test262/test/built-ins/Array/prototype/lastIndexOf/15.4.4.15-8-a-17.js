// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf -  decreasing length of array causes
    index property not to be visited
---*/

var arr = [0, 1, 2, "last", 4];

Object.defineProperty(arr, "4", {
  get: function() {
    arr.length = 3;
    return 0;
  },
  configurable: true
});

assert.sameValue(arr.lastIndexOf("last"), -1, 'arr.lastIndexOf("last")');

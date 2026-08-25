// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - deleting own property causes index
    property not to be visited on an Array
---*/

var arr = [1, 2, 3, 4];

Object.defineProperty(arr, "1", {
  get: function() {
    return "6.99";
  },
  configurable: true
});

Object.defineProperty(arr, "3", {
  get: function() {
    delete arr[1];
    return 0;
  },
  configurable: true
});

assert.sameValue(arr.lastIndexOf("6.99"), -1, 'arr.lastIndexOf("6.99")');

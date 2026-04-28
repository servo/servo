// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - added properties in step 2 are visible
    here
---*/

var arr = {};

Object.defineProperty(arr, "length", {
  get: function() {
    arr[2] = "length";
    return 3;
  },
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call(arr, "length"), 2, 'Array.prototype.indexOf.call(arr, "length")');

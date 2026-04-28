// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - deleted properties in step 2 are visible
    here
---*/

var arr = {
  2: 6.99
};

Object.defineProperty(arr, "length", {
  get: function() {
    delete arr[2];
    return 3;
  },
  configurable: true
});

assert.sameValue(Array.prototype.indexOf.call(arr, 6.99), -1, 'Array.prototype.indexOf.call(arr, 6.99)');

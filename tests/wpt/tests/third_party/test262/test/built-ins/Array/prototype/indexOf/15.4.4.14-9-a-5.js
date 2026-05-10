// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - deleted properties in step 5 are visible
    here on an Array-like object
---*/

var arr = {
  10: false,
  length: 30
};

var fromIndex = {
  valueOf: function() {
    delete arr[10];
    return 3;
  }
};

assert.sameValue(Array.prototype.indexOf.call(arr, false, fromIndex), -1, 'Array.prototype.indexOf.call(arr, false, fromIndex)');

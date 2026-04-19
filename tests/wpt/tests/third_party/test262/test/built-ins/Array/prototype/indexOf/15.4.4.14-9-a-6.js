// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - deleted properties in step 5 are visible
    here on an Array
---*/

var arr = [];
arr[10] = "10";
arr.length = 20;

var fromIndex = {
  valueOf: function() {
    delete arr[10];
    return 3;
  }
};

assert.sameValue(arr.indexOf("10", fromIndex), -1, 'arr.indexOf("10", fromIndex)');

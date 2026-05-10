// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf -  deleted properties of step 5 are
    visible here on an Array
---*/

var arr = [];
arr[10] = "10";
arr.length = 20;

var fromIndex = {
  valueOf: function() {
    delete arr[10];
    return 11;
  }
};

assert.sameValue(arr.lastIndexOf("10", fromIndex), -1, 'arr.lastIndexOf("10", fromIndex)');

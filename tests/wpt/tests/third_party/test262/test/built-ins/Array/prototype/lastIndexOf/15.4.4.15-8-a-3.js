// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf -  added properties in step 5 are
    visible here on an Array
---*/

var arr = [];
arr.length = 30;
var targetObj = function() {};

var fromIndex = {
  valueOf: function() {
    arr[4] = targetObj;
    return 11;
  }
};

assert.sameValue(arr.lastIndexOf(targetObj, fromIndex), 4, 'arr.lastIndexOf(targetObj, fromIndex)');

// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' is a string
    containing Infinity
---*/

var arr = [];
arr[Math.pow(2, 32) - 2] = true; // length is the max value of Uint type

assert.sameValue(arr.lastIndexOf(true, "Infinity"), Math.pow(2, 32) - 2, 'arr.lastIndexOf(true, "Infinity")');

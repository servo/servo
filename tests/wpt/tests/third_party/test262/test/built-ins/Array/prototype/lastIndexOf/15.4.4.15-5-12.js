// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' is a number
    (value is Infinity)
---*/

var arr = [];
arr[Math.pow(2, 32) - 2] = null; // length is the max value of Uint type

assert.sameValue(arr.lastIndexOf(null, Infinity), Math.pow(2, 32) - 2, 'arr.lastIndexOf(null, Infinity)');

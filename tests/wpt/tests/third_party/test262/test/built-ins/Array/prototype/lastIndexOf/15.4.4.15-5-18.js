// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' is a string
    containing an exponential number
---*/

var targetObj = {};

assert.sameValue([0, NaN, targetObj, 3, false].lastIndexOf(targetObj, "2E0"), 2, '[0, NaN, targetObj, 3, false].lastIndexOf(targetObj, "2E0")');
assert.sameValue([0, NaN, 3, targetObj, false].lastIndexOf(targetObj, "2E0"), -1, '[0, NaN, 3, targetObj, false].lastIndexOf(targetObj, "2E0")');

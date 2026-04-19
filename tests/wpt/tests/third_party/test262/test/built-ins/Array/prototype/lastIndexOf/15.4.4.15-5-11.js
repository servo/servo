// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' is a number
    (value is negative number)
---*/

var targetObj = {};

assert.sameValue([0, targetObj, true].lastIndexOf(targetObj, -2.5), 1, '[0, targetObj, true].lastIndexOf(targetObj, -2.5)');
assert.sameValue([0, true, targetObj].lastIndexOf(targetObj, -2.5), -1, '[0, true, targetObj].lastIndexOf(targetObj, -2.5)');

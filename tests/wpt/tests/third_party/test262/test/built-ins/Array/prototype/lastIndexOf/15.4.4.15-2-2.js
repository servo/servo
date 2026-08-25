// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - 'length' is own data property on an
    Array
---*/

var targetObj = {};

Array.prototype[2] = targetObj;

assert.sameValue([0, targetObj].lastIndexOf(targetObj), 1, '[0, targetObj].lastIndexOf(targetObj)');
assert.sameValue([0, 1].lastIndexOf(targetObj), -1, '[0, 1].lastIndexOf(targetObj)');

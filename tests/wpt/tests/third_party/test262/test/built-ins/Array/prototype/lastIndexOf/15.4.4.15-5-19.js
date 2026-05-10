// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf - value of 'fromIndex' is a string
    containing a hex number
---*/

var targetObj = {};

assert.sameValue([0, true, targetObj, 3, false].lastIndexOf(targetObj, "0x0002"), 2, '[0, true, targetObj, 3, false].lastIndexOf(targetObj, "0x0002")');
assert.sameValue([0, true, 3, targetObj, false].lastIndexOf(targetObj, "0x0002"), -1, '[0, true, 3, targetObj, false].lastIndexOf(targetObj, "0x0002")');

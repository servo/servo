// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf returns index of last one when more
    than two elements in array are eligible
---*/

assert.sameValue([2, 1, 2, 2, 1].lastIndexOf(2), 3, '[2, 1, 2, 2, 1].lastIndexOf(2)');

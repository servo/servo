// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: >
    Array.prototype.indexOf - returns index of last one when more than
    two elements in array are eligible
---*/

assert.sameValue([1, 2, 2, 1, 2].indexOf(2), 1, '[1, 2, 2, 1, 2].indexOf(2)');

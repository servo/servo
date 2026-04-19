// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf returns correct index when 'fromIndex'
    is -1
---*/

assert.sameValue([1, 2, 3, 4].lastIndexOf(4, -1), 3, '[1, 2, 3, 4].lastIndexOf(4, -1)');

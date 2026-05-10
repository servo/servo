// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf returns -1 if 'length' is 0 (empty
    array)
---*/

var i = [].lastIndexOf(42);

assert.sameValue(i, -1, 'i');

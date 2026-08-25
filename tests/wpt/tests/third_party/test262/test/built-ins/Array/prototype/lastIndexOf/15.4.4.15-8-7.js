// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf must return correct index (self
    reference)
---*/

var a = new Array(0, 1, 2, 3);
a[2] = a;

assert.sameValue(a.lastIndexOf(a), 2, 'a.lastIndexOf(a)');
assert.sameValue(a.lastIndexOf(3), 3, 'a.lastIndexOf(3)');

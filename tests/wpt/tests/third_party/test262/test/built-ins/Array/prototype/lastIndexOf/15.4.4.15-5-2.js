// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf when fromIndex is floating point number
---*/

var a = new Array(1, 2, 1);

assert.sameValue(a.lastIndexOf(2, 1.49), 1, '1.49 resolves to 1');
assert.sameValue(a.lastIndexOf(2, 0.51), -1, '0.51 resolves to 0');
assert.sameValue(a.lastIndexOf(1, 0.51), 0, '0.51 resolves to 0');

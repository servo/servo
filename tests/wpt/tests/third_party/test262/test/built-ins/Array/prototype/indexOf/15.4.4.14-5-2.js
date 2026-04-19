// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf when fromIndex is floating point number
---*/

var a = new Array(1, 2, 3);

assert.sameValue(a.indexOf(3, 0.49), 2, '0.49 resolves to 0');
assert.sameValue(a.indexOf(1, 0.51), 0, '0.51 resolves to 0');
assert.sameValue(a.indexOf(1, 1.51), -1, '1.51 resolves to 1');

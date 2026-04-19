// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf when fromIndex is boolean
---*/

var a = [1, 2, 3];

assert.sameValue(a.indexOf(1, true), -1, 'true resolves to 1');
assert.sameValue(a.indexOf(1, false), 0, 'false resolves to 0');

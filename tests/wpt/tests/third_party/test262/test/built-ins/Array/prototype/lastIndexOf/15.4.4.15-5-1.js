// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf when fromIndex is string
---*/

var a = new Array(0, 1, 1);

assert.sameValue(a.lastIndexOf(1, "1"), 1, '"1" resolves to 1');
assert.sameValue(a.lastIndexOf(1, "one"), -1, 'NaN string resolves to 01');

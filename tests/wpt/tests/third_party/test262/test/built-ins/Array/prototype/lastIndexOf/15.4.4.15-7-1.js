// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf with negative fromIndex
---*/

var a = new Array(1, 2, 3);


assert.sameValue(a.lastIndexOf(2, -2), 1, 'a.lastIndexOf(2,-2)');
assert.sameValue(a.lastIndexOf(2, -3), -1, 'a.lastIndexOf(2,-3)');
assert.sameValue(a.lastIndexOf(1, -5.3), -1, 'a.lastIndexOf(1,-5.3)');

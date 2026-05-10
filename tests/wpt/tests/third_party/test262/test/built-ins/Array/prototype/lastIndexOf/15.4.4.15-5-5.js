// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: Array.prototype.lastIndexOf when fromIndex is null
---*/

var a = new Array(1, 2, 1);

// null resolves to 0
assert.sameValue(a.lastIndexOf(2, null), -1, 'a.lastIndexOf(2,null)');
assert.sameValue(a.lastIndexOf(1, null), 0, 'a.lastIndexOf(1,null)');

// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.lastindexof
description: >
    Array.prototype.lastIndexOf must return correct index (Sparse
    Array)
---*/

var a = new Array(0, 1);
a[4294967294] = 2; // 2^32-2 - is max array element index
a[4294967295] = 3; // 2^32-1 added as non-array element property
a[4294967296] = 4; // 2^32   added as non-array element property
a[4294967297] = 5; // 2^32+1 added as non-array element property
// stop searching near the end in case implementation actually tries to test all missing elements!!
a[4294967200] = 3;
a[4294967201] = 4;
a[4294967202] = 5;


assert.sameValue(a.lastIndexOf(2), 4294967294, 'a.lastIndexOf(2)');
assert.sameValue(a.lastIndexOf(3), 4294967200, 'a.lastIndexOf(3)');
assert.sameValue(a.lastIndexOf(4), 4294967201, 'a.lastIndexOf(4)');
assert.sameValue(a.lastIndexOf(5), 4294967202, 'a.lastIndexOf(5)');

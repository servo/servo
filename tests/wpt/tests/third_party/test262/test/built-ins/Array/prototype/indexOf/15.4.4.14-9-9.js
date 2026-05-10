// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.indexof
description: Array.prototype.indexOf must return correct index (Sparse Array)
---*/

var a = new Array(0, 1);
a[4294967294] = 2; // 2^32-2 - is max array element
a[4294967295] = 3; // 2^32-1 added as non-array element property
a[4294967296] = 4; // 2^32   added as non-array element property
a[4294967297] = 5; // 2^32+1 added as non-array element property

// start searching near the end so in case implementation actually tries to test all missing elements!!

assert.sameValue(a.indexOf(2, 4294967290), 4294967294, 'a.indexOf(2,4294967290 )');
assert.sameValue(a.indexOf(3, 4294967290), -1, 'a.indexOf(3,4294967290)');
assert.sameValue(a.indexOf(4, 4294967290), -1, 'a.indexOf(4,4294967290)');
assert.sameValue(a.indexOf(5, 4294967290), -1, 'a.indexOf(5,4294967290)');

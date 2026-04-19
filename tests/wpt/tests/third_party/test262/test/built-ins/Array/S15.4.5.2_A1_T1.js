// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Every Array object has a length property whose value is
    always a nonnegative integer less than 2^32. The value of the length property is
    numerically greater than the name of every property whose name is an array index
es5id: 15.4.5.2_A1_T1
description: Checking boundary points
---*/

var x = [];
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');

x[0] = 1;
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

x[1] = 1;
assert.sameValue(x.length, 2, 'The value of x.length is expected to be 2');

x[2147483648] = 1;
assert.sameValue(x.length, 2147483649, 'The value of x.length is expected to be 2147483649');

x[4294967294] = 1;
assert.sameValue(x.length, 4294967295, 'The value of x.length is expected to be 4294967295');

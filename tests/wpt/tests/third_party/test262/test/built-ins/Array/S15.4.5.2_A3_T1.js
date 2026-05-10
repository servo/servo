// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If the length property is changed, every property whose name
    is an array index whose value is not smaller than the new length is automatically deleted
es5id: 15.4.5.2_A3_T1
description: >
    If new length greater than the name of every property whose name
    is an array index
---*/

var x = [];
x.length = 1;
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

x[5] = 1;
x.length = 10;
assert.sameValue(x.length, 10, 'The value of x.length is expected to be 10');
assert.sameValue(x[5], 1, 'The value of x[5] is expected to be 1');

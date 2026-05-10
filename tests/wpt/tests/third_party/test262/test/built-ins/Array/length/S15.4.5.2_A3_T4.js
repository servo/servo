// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-array-instances-length
info: |
    If the length property is changed, every property whose name
    is an array index whose value is not smaller than the new length is automatically deleted
es5id: 15.4.5.2_A3_T4
description: >
    If new length greater than the name of every property whose name
    is an array index
---*/

var x = [0, 1, 2];
x[4294967294] = 4294967294;
x.length = 2;

assert.sameValue(x[0], 0, 'The value of x[0] is expected to be 0');
assert.sameValue(x[1], 1, 'The value of x[1] is expected to be 1');
assert.sameValue(x[2], undefined, 'The value of x[2] is expected to equal undefined');
assert.sameValue(x[4294967294], undefined, 'The value of x[4294967294] is expected to equal undefined');

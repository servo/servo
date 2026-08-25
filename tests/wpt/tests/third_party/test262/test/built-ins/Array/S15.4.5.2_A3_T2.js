// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If the length property is changed, every property whose name
    is an array index whose value is not smaller than the new length is automatically deleted
es5id: 15.4.5.2_A3_T2
description: >
    If new length greater than the name of every property whose name
    is an array index
---*/

var x = [];
x[1] = 1;
x[3] = 3;
x[5] = 5;
x.length = 4;
assert.sameValue(x.length, 4, 'The value of x.length is expected to be 4');
assert.sameValue(x[5], undefined, 'The value of x[5] is expected to equal undefined');
assert.sameValue(x[3], 3, 'The value of x[3] is expected to be 3');

x.length = new Number(6);
assert.sameValue(x[5], undefined, 'The value of x[5] is expected to equal undefined');

x.length = 0;
assert.sameValue(x[0], undefined, 'The value of x[0] is expected to equal undefined');

x.length = 1;
assert.sameValue(x[1], undefined, 'The value of x[1] is expected to equal undefined');

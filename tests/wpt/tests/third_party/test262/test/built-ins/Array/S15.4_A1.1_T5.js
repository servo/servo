// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property name P (in the form of a string value) is an array index
    if and only if ToString(ToUint32(P)) is equal to P and ToUint32(P) is not equal to 2^32 - 1
es5id: 15.4_A1.1_T5
description: Checking for null and undefined
---*/

var x = [];
x[null] = 0;
assert.sameValue(x[0], undefined, 'The value of x[0] is expected to equal undefined');
assert.sameValue(x["null"], 0, 'The value of x["null"] is expected to be 0');

var y = [];
y[undefined] = 0;
assert.sameValue(y[0], undefined, 'The value of y[0] is expected to equal undefined');
assert.sameValue(y["undefined"], 0, 'The value of y["undefined"] is expected to be 0');

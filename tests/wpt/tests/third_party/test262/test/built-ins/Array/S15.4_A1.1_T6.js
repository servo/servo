// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property name P (in the form of a string value) is an array index
    if and only if ToString(ToUint32(P)) is equal to P and ToUint32(P) is not equal to 2^32 - 1
es5id: 15.4_A1.1_T6
description: Checking for Boolean object
---*/

var x = [];
x[new Boolean(true)] = 1;
assert.sameValue(x[1], undefined, 'The value of x[1] is expected to equal undefined');
assert.sameValue(x["true"], 1, 'The value of x["true"] is expected to be 1');

x[new Boolean(false)] = 0;
assert.sameValue(x[0], undefined, 'The value of x[0] is expected to equal undefined');
assert.sameValue(x["false"], 0, 'The value of x["false"] is expected to be 0');

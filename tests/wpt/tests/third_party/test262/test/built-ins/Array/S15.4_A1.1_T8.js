// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property name P (in the form of a string value) is an array index
    if and only if ToString(ToUint32(P)) is equal to P and ToUint32(P) is not equal to 2^32 - 1
es5id: 15.4_A1.1_T8
description: Checking for Number object
---*/

var x = [];
x[new String("0")] = 0;
assert.sameValue(x[0], 0, 'The value of x[0] is expected to be 0');

var y = [];
y[new String("1")] = 1;
assert.sameValue(y[1], 1, 'The value of y[1] is expected to be 1');

var z = [];
z[new String("1.1")] = 1;
assert.sameValue(z["1.1"], 1, 'The value of z["1.1"] is expected to be 1');

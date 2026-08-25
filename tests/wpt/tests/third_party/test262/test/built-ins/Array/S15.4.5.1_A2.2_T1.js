// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If ToUint32(P) is less than the value of
    the length property of A, then return
es5id: 15.4.5.1_A2.2_T1
description: length === 100, P in [0, 98, 99]
---*/

var x = Array(100);
x[0] = 1;
assert.sameValue(x.length, 100, 'The value of x.length is expected to be 100');

x[98] = 1;
assert.sameValue(x.length, 100, 'The value of x.length is expected to be 100');

x[99] = 1;
assert.sameValue(x.length, 100, 'The value of x.length is expected to be 100');

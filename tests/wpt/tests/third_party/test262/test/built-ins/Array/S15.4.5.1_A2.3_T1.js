// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If ToUint32(P) is less than the value of
    the length property of A, change (or set) length to ToUint32(P)+1
es5id: 15.4.5.1_A2.3_T1
description: length = 100, P in [100, 199]
---*/

var x = Array(100);
x[100] = 1;
assert.sameValue(x.length, 101, 'The value of x.length is expected to be 101');

x[199] = 1;
assert.sameValue(x.length, 200, 'The value of x.length is expected to be 200');

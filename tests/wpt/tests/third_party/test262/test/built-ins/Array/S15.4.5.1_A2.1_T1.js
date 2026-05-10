// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If P is not an array index, return
    (Create a property with name P, set its value to V and give it empty attributes)
es5id: 15.4.5.1_A2.1_T1
description: P in [4294967295, -1, true]
---*/

var x = [];
x[4294967295] = 1;
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');
assert.sameValue(x[4294967295], 1, 'The value of x[4294967295] is expected to be 1');

x = [];
x[-1] = 1;
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');
assert.sameValue(x[-1], 1, 'The value of x[-1] is expected to be 1');

x = [];
x[true] = 1;
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');
assert.sameValue(x[true], 1, 'The value of x[true] is expected to be 1');

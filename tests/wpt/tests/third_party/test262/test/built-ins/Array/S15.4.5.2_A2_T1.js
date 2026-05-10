// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If a property is added whose name is an array index,
    the length property is changed
es5id: 15.4.5.2_A2_T1
description: Checking length property
---*/

var x = [];
assert.sameValue(x.length, 0, 'The value of x.length is expected to be 0');

x[0] = 1;
assert.sameValue(x.length, 1, 'The value of x.length is expected to be 1');

x[1] = 1;
assert.sameValue(x.length, 2, 'The value of x.length is expected to be 2');

x[9] = 1;
assert.sameValue(x.length, 10, 'The value of x.length is expected to be 10');

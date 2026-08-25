// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    This description of Array constructor applies if and only if
    the Array constructor is given no arguments or at least two arguments
es5id: 15.4.2.1_A1.3_T1
description: Checking case when Array constructor is given one argument
---*/

var x = new Array(2);

assert.notSameValue(x.length, 1, 'The value of x.length is not 1');
assert.notSameValue(x[0], 2, 'The value of x[0] is not 2');

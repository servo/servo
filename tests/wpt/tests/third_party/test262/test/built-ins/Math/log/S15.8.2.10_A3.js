// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is +0 or -0, Math.log(x) is -Infinity
es5id: 15.8.2.10_A3
description: Checking if Math.log(+0) and Math.log(-0) equals to -Infinity
---*/

// CHECK#1
var x = +0;
assert.sameValue(Math.log(x), -Infinity, 'Math.log(+0) must return -Infinity');

// CHECK#2
var x = -0;
assert.sameValue(Math.log(x), -Infinity, 'Math.log(-0) must return -Infinity');

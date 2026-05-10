// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is +0, Math.exp(x) is 1
es5id: 15.8.2.8_A2
description: Checking if Math.exp(+0) is 1
---*/

// CHECK#1
var x = +0;
assert.sameValue(Math.exp(x), 1, 'Math.exp(+0) must return 1');

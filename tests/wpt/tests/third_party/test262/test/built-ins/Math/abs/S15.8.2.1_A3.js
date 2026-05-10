// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is -Infinity, Math.abs(x) is +Infinity
es5id: 15.8.2.1_A3
description: Checking if Math.abs(-Infinity) equals to +Infinity
---*/

// CHECK#1
var x = -Infinity;
assert.sameValue(Math.abs(x), +Infinity, 'Math.abs(-Infinity) must return +Infinity');

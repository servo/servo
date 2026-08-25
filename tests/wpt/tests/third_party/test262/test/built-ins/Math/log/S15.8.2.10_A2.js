// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is less than 0, Math.log(x) is NaN
es5id: 15.8.2.10_A2
description: Checking if Math.log(x) is NaN, where x is less than 0
---*/

assert.sameValue(Math.log(-0.000000000000001), NaN, "-0.000000000000001");
assert.sameValue(Math.log(-1), NaN, "-1");
assert.sameValue(Math.log(-Infinity), NaN, "-Infinity");

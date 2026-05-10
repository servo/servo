// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is less than -1, Math.asin(x) is NaN
es5id: 15.8.2.3_A3
description: Checking if Math.asin(x) is NaN, where x is less than -1
---*/

assert.sameValue(Math.asin(-1.000000000000001), NaN, "-1.000000000000001");
assert.sameValue(Math.asin(-2), NaN, "-2");
assert.sameValue(Math.asin(-Infinity), NaN, "-Infinity");

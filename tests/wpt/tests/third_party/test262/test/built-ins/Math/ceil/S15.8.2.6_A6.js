// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If x is less than 0 but greater than -1, Math.ceil(x) is -0
es5id: 15.8.2.6_A6
description: >
    Checking if Math.ceil(x) is -0, where x is less than 0 but greater
    than -1
---*/

assert.sameValue(Math.ceil(-0.000000000000001), -0, "-0.000000000000001");
assert.sameValue(Math.ceil(-0.999999999999999), -0, "-0.999999999999999");
assert.sameValue(Math.ceil(-0.5), -0, "-0.5");

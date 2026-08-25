// Copyright (C) 2018 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-applying-the-exp-operator
description: >
    Using -(2**31) as exponent with Math.pow should behave as expected.
---*/

const INT32_MIN = -2147483648;

assert.sameValue(Math.pow(2, INT32_MIN), +0.0,
                 "Math.pow(2, -(gonzo huge exponent > 1074)) should be +0 " +
                 "because 2**-1074 is the smallest positive IEEE-754 number");

assert.sameValue(Math.pow(1, INT32_MIN), 1,
                 "1**-(gonzo huge exponent > 1074) should be 1");

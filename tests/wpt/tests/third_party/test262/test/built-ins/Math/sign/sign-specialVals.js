// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Returns the sign of the x, indicating whether x is positive, negative or zero.
es6id: 20.2.2.29
---*/

assert.sameValue(Math.sign(NaN), NaN, "NaN");
assert.sameValue(Math.sign(-0), -0, "-0");
assert.sameValue(Math.sign(0), 0, "0");

assert.sameValue(Math.sign(-0.000001), -1, "-0.000001");
assert.sameValue(Math.sign(-1), -1, "-1");
assert.sameValue(Math.sign(-Infinity), -1, "-Infinity");

assert.sameValue(Math.sign(0.000001), 1, "0.000001");
assert.sameValue(Math.sign(1), 1, "1");
assert.sameValue(Math.sign(Infinity), 1, "Infinity");

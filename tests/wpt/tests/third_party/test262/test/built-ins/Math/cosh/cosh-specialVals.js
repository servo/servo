// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.cosh with special values
es6id: 20.2.2.13
---*/

assert.sameValue(Math.cosh(NaN), Number.NaN,
  "Math.cosh produces incorrect output for NaN");
assert.sameValue(Math.cosh(0), 1, "Math.cosh should produce 1 for input = 0");
assert.sameValue(Math.cosh(-0), 1, "Math.cosh should produce 1 for input = -0");
assert.sameValue(Math.cosh(Number.NEGATIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.cosh should produce Number.POSITIVE_INFINITY for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.cosh(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.cosh should produce Number.POSITIVE_INFINITY for Number.POSITIVE_INFINITY");

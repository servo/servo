// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.cbrt with special values
es6id: 20.2.2.9
---*/

assert.sameValue(Math.cbrt(NaN), Number.NaN,
  "Math.cbrt produces incorrect output for NaN");
assert.sameValue(Math.cbrt(Number.NEGATIVE_INFINITY), Number.NEGATIVE_INFINITY,
  "Math.cbrt should produce negative infinity for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.cbrt(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.cbrt should produce positive infinity for Number.POSITIVE_INFINITY");
assert.sameValue(1 / Math.cbrt(-0), Number.NEGATIVE_INFINITY,
  "Math.cbrt should produce -0 for -0");
assert.sameValue(1 / Math.cbrt(0), Number.POSITIVE_INFINITY,
  "Math.cbrt should produce +0 for +0");

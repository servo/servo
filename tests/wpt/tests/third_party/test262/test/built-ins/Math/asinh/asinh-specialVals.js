// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.asinh with special values
es6id: 20.2.2.5
---*/

assert.sameValue(Math.asinh(NaN), Number.NaN,
  "Math.asinh produces incorrect output for NaN");
assert.sameValue(Math.asinh(Number.NEGATIVE_INFINITY), Number.NEGATIVE_INFINITY,
  "Math.asinh should produce negative infinity for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.asinh(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.asinh should produce positive infinity for Number.POSITIVE_INFINITY");
assert.sameValue(1 / Math.asinh(-0), Number.NEGATIVE_INFINITY,
  "Math.asinh should produce -0 for -0");
assert.sameValue(1 / Math.asinh(0), Number.POSITIVE_INFINITY,
  "Math.asinh should produce +0 for +0");

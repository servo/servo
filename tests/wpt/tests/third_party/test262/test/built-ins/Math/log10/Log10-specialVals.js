// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.Log10 with sample values.
es6id: 20.2.2.20
---*/

assert.sameValue(Math.log10(-0), Number.NEGATIVE_INFINITY,
  "Math.log10 produces incorrect output for -0");
assert.sameValue(Math.log10(+0), Number.NEGATIVE_INFINITY,
  "Math.log10 produces incorrect output for +0");
assert.sameValue(Math.log10(-0.9), Number.NaN,
  "Math.log10 produces incorrect output for -0.9");
assert.sameValue(Math.log10(NaN), Number.NaN,
  "Math.log10 produces incorrect output for NaN");
assert.sameValue(Math.log10(-10), Number.NaN,
  "Math.log10 produces incorrect output for -10");
assert.sameValue(Math.log10(null), Number.NEGATIVE_INFINITY,
  "Math.log10 produces incorrect output for null");
assert.sameValue(Math.log10(undefined), Number.NaN,
  "Math.log10 produces incorrect output for undefined");
assert.sameValue(Math.log10(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.log10 produces incorrect output for Number.POSITIVE_INFINITY");
assert.sameValue(Math.log10(1), 0,
  "Math.log10 produces incorrect output for 1");
assert.sameValue(Math.log10(10.00), 1,
  "Math.log10 produces incorrect output for 10.00");
assert.sameValue(Math.log10(100.00), 2,
  "Math.log10 produces incorrect output for 100.00");
assert.sameValue(Math.log10(1000.00), 3,
  "Math.log10 produces incorrect output for 1000.00");

// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.Log2 with sample values.
es6id: 20.2.2.23
---*/

assert.sameValue(Math.log2(-0), Number.NEGATIVE_INFINITY,
  "Math.log2 produces incorrect output for -0");
assert.sameValue(Math.log2(+0), Number.NEGATIVE_INFINITY,
  "Math.log2 produces incorrect output for +0");
assert.sameValue(Math.log2(-0.9), NaN,
  "Math.log2 produces incorrect output for -0.9");
assert.sameValue(Math.log2(NaN), NaN,
  "Math.log2 produces incorrect output for NaN");
assert.sameValue(Math.log2(-10), NaN,
  "Math.log2 produces incorrect output for -10");
assert.sameValue(Math.log2(-Infinity), NaN,
  "Math.log2 produces incorrect output for -Infinity");
assert.sameValue(Math.log2(null), Number.NEGATIVE_INFINITY,
  "Math.log2 produces incorrect output for null");
assert.sameValue(Math.log2(undefined), NaN,
  "Math.log2 produces incorrect output for undefined");
assert.sameValue(Math.log2(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.log2 produces incorrect output for Number.POSITIVE_INFINITY");
assert.sameValue(Math.log2(1), 0,
  "Math.log2 produces incorrect output for 1");
assert.sameValue(Math.log2(2.00), 1,
  "Math.log2 produces incorrect output for 2.00");
assert.sameValue(Math.log2(4.00), 2,
  "Math.log2 produces incorrect output for 4.00");
assert.sameValue(Math.log2(8.00), 3,
  "Math.log2 produces incorrect output for 8.00");

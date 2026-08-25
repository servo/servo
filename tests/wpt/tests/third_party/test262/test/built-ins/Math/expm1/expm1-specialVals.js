// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.expm1 with sample values.
es6id: 20.2.2.15
---*/

assert.sameValue(Math.expm1(NaN), Number.NaN,
  "Math.expm1 produces incorrect output for NaN");
assert.sameValue(Math.expm1(Number.NEGATIVE_INFINITY), -1,
  "Math.expm1 should produce -1 for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.expm1(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.expm1 should produce POSITIVE infinity for Number.POSITIVE_INFINITY");
assert.sameValue(1 / Math.expm1(-0), Number.NEGATIVE_INFINITY,
  "Math.expm1 should produce -0 for -0");
assert.sameValue(1 / Math.expm1(0), Number.POSITIVE_INFINITY,
  "Math.expm1 should produce +0 for +0");

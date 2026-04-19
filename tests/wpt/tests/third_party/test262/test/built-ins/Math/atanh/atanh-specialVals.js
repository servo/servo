// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.atanh with special values
es6id: 20.2.2.7
---*/

assert.sameValue(Math.atanh(-1.9), Number.NaN,
  "Math.atanh produces incorrect output for -1.9");
assert.sameValue(Math.atanh(NaN), Number.NaN,
  "Math.atanh produces incorrect output for NaN");
assert.sameValue(Math.atanh(-10), Number.NaN,
  "Math.atanh produces incorrect output for -10");
assert.sameValue(Math.atanh(-Infinity), Number.NaN,
  "Math.atanh produces incorrect output for -Infinity");
assert.sameValue(Math.atanh(1.9), Number.NaN,
  "Math.atanh produces incorrect output for 1.9");
assert.sameValue(Math.atanh(10), Number.NaN,
  "Math.atanh produces incorrect output for 10");
assert.sameValue(Math.atanh(Number.POSITIVE_INFINITY), Number.NaN,
  "Math.atanh produces incorrect output for Number.POSITIVE_INFINITY");

assert.sameValue(Math.atanh(-1), Number.NEGATIVE_INFINITY,
  "Math.atanh should produce negative infinity for -1");
assert.sameValue(Math.atanh(+1), Number.POSITIVE_INFINITY,
  "Math.atanh should produce positive infinity for +1");
assert.sameValue(1 / Math.atanh(-0), Number.NEGATIVE_INFINITY,
  "Math.atanh should produce -0 for -0");
assert.sameValue(1 / Math.atanh(0), Number.POSITIVE_INFINITY,
  "Math.atanh should produce +0 for +0");

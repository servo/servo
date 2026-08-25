// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.tanh with special values
es6id: 20.2.2.34
---*/

assert.sameValue(Math.tanh(NaN), Number.NaN,
  "Math.tanh produces incorrect output for NaN");
assert.sameValue(Math.tanh(Number.NEGATIVE_INFINITY), -1,
  "Math.tanh should produce -1 for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.tanh(Number.POSITIVE_INFINITY), 1,
  "Math.tanh should produce 1 for Number.POSITIVE_INFINITY");
assert.sameValue(1 / Math.tanh(-0), Number.NEGATIVE_INFINITY,
  "Math.tanh should produce -0 for -0");
assert.sameValue(1 / Math.tanh(0), Number.POSITIVE_INFINITY,
  "Math.tanh should produce +0 for +0");

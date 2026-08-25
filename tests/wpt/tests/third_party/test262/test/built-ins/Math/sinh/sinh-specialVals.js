// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.sinh with special values
es6id: 20.2.2.31
---*/

assert.sameValue(Math.sinh(NaN), Number.NaN,
  "Math.sinh produces incorrect output for NaN");
assert.sameValue(Math.sinh(Number.NEGATIVE_INFINITY), Number.NEGATIVE_INFINITY,
  "Math.sinh should produce negative infinity for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.sinh(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.sinh should produce positive infinity for Number.POSITIVE_INFINITY");
assert.sameValue(1 / Math.sinh(-0), Number.NEGATIVE_INFINITY,
  "Math.sinh should produce -0 for -0");
assert.sameValue(1 / Math.sinh(0), Number.POSITIVE_INFINITY,
  "Math.sinh should produce +0 for +0");

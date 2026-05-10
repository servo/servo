// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.trunc with sample values.
es6id: 20.2.2.35
---*/

assert.sameValue(Math.trunc(Number.NEGATIVE_INFINITY), Number.NEGATIVE_INFINITY,
  "Math.trunc should produce negative infinity for Number.NEGATIVE_INFINITY");
assert.sameValue(Math.trunc(Number.POSITIVE_INFINITY), Number.POSITIVE_INFINITY,
  "Math.trunc should produce positive infinity for Number.POSITIVE_INFINITY");
assert.sameValue(1 / Math.trunc(-0), Number.NEGATIVE_INFINITY,
  "Math.trunc should produce -0 for -0");
assert.sameValue(1 / Math.trunc(0), Number.POSITIVE_INFINITY,
  "Math.trunc should produce +0 for +0");

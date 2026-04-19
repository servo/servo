// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Math.acosh with special values
es6id: 20.2.2.3
info: |
  Math.acosh ( x )

  - If x is NaN, the result is NaN.
  - If x is less than 1, the result is NaN.
---*/

assert.sameValue(Math.acosh(NaN), NaN, "NaN");
assert.sameValue(Math.acosh(0.999999), NaN, "0.999999");
assert.sameValue(Math.acosh(0), NaN, "0");
assert.sameValue(Math.acosh(-1), NaN, "-1");
assert.sameValue(Math.acosh(-Infinity), NaN, "-Infinity");

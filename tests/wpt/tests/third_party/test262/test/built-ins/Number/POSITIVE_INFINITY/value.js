// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.positive_infinity
description: >
  The value of Number.POSITIVE_INFINITY is +Infinity
info: |
  Number.POSITIVE_INFINITY

  The value of Number.POSITIVE_INFINITY is +∞.
---*/

assert.sameValue(Number.POSITIVE_INFINITY, Infinity);

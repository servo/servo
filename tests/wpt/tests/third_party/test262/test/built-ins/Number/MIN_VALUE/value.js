// Copyright (C) 2026 Luna Pfeiffer. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.min_value
description: >
  Number.MIN_VALUE is approximately 5e-324. We can't directly compare it as the spec states:

  In the IEEE 754-2019 double precision binary representation, the smallest possible value is a denormalized number.
  If an implementation does not support denormalized values, the value of Number.MIN_VALUE must be the smallest
  non-zero positive value that can actually be represented by the implementation.
---*/


assert.sameValue(typeof Number.MIN_VALUE, "number", "Number.MIN_VALUE should be a number");
assert(Number.MIN_VALUE > 0, "Number.MIN_VALUE must be positive");

assert(Number.MIN_VALUE < Number.EPSILON, "Number.MIN_VALUE should be smaller than Number.EPSILON")

assert.sameValue(Number.MIN_VALUE / 2, 0, "Number.MIN_VALUE divided by 2 should underflow to 0");

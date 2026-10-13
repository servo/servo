// Copyright (C) 2026 Luna Pfeiffer. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.max_value
description: >
  Number.MAX_VALUE is approximately 1.7976931348623157 × 10³⁰⁸

---*/


assert.sameValue(typeof Number.MAX_VALUE, "number", "Number.MAX_VALUE should be a number");
assert(Number.MAX_VALUE > 0, "Number.MAX_VALUE must be positive");

assert(Number.MAX_VALUE > Number.MAX_SAFE_INTEGER, "Number.MAX_VALUE should be larger than Number.MAX_SAFE_INTEGER");
assert(Number.MAX_VALUE < Infinity, "Number.MAX_VALUE should be less than Infinity");

assert.sameValue(Number.MAX_VALUE * 2, Infinity, "Number.MAX_VALUE multiplied by 2 should be Infinity");
assert.sameValue(Number.MAX_VALUE, 1.7976931348623157e+308, "Number.MAX_VALUE should be approximately 1.7976931348623157e+308");

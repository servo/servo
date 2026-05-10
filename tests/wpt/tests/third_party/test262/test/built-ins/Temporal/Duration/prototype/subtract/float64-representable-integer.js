// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.subtract
description: Internal representation uses float64-representable integers
features: [Temporal]
---*/

const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, /* µs = */ Number.MAX_SAFE_INTEGER, 0);
const result = d.subtract({ microseconds: Number.MIN_SAFE_INTEGER + 1 });

// ℝ(𝔽(18014398509481981)) = 18014398509481980
assert.sameValue(result.microseconds, 18014398509481980,
  "microseconds result should have FP precision loss");
assert.sameValue(result.toString(), "PT18014398509.48198S",
  "toString() should not use more precise internal representation than the spec prescribes");
assert.sameValue(Temporal.Duration.compare(result.subtract({ microseconds: 1 }), result), 0,
  "subsequent subtract() should not use more precise internal representation than the spec prescribes");

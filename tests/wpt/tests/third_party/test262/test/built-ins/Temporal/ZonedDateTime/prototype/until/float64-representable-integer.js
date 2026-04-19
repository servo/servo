// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Internal representation of Duration uses float64-representable integers
features: [Temporal]
---*/

const z1 = new Temporal.ZonedDateTime(0n, "UTC");
const z2 = new Temporal.ZonedDateTime(18446744073_709_551_616n, "UTC");
const result = z1.until(z2, { largestUnit: "microseconds" });

// ℝ(𝔽(18446744073709551)) = 18446744073709552
assert.sameValue(result.microseconds, 18446744073709552,
  "microseconds result should have FP precision loss");
assert.sameValue(result.toString(), "PT18446744073.709552616S",
  "Duration.p.toString() should not use more precise internal representation than the spec prescribes");
assert.sameValue(Temporal.Duration.compare(result.add({ microseconds: 1 }), result), 0,
  "subsequent ops on duration should not use more precise internal representation than the spec prescribes");

// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Internal representation uses float64-representable integers
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, /* ms = */ 18014398509481, /* µs = */ 981, 0);
const result = d.round({ largestUnit: "microseconds" });

// ℝ(𝔽(18014398509481981)) = 18014398509481980
assert.sameValue(result.microseconds, 18014398509481980,
  "microseconds result should have FP precision loss");
assert.sameValue(result.toString(), "PT18014398509.48198S",
  "toString() should not use more precise internal representation than the spec prescribes");
// Rounding bounds of 8 µs are ...976 and ...984. halfTrunc will round down if
// the µs component is ...980 and up if it is ...981
TemporalHelpers.assertDuration(
  result.round({ largestUnit: "seconds", smallestUnit: "microseconds", roundingMode: "halfTrunc", roundingIncrement: 8 }),
  0, 0, 0, 0, 0, 0, 18014398509, 481, 976, 0,
  "subsequent round() should not use more precise internal representation than the spec prescribes");

// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  Rounding with a large increment pushes internal epoch nanosecond arithmetic
  past the limits of the instant range, without affecting the result
info: |
  NudgeToDayOrTime ( _duration_, _destEpochNanoseconds_, _largestUnit_,
                     _increment_, _smallestUnit_, _roundingMode_ ):
    9. Let _nudgedEpochNanoseconds_ be _destEpochNanoseconds_ + _diffTime_.

  This test exercises inputs that make _nudgedEpochNanoseconds_ a very large
  value (up to ±11 × MaxEpochNanoseconds), which must still be safe for any
  plausible representation of epoch nanoseconds that an implementation might
  choose: 96-bit int, 128-bit float, or 64-bit float seconds + subseconds.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earliest = new Temporal.PlainDate(-271821, 4, 20);
const latest = new Temporal.PlainDate(275760, 9, 13);
const opts = { smallestUnit: "days", roundingIncrement: 1e9 };

const durationPositive = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, 1);
const resultPositive = durationPositive.round({ ...opts, relativeTo: latest, roundingMode: "ceil" });
TemporalHelpers.assertDuration(resultPositive, 0, 0, 0, 1e9, 0, 0, 0, 0, 0, 0, "result of rounding 1 ns up to 10^9 days");

const durationNegative = new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, -1);
const resultNegative = durationNegative.round({ ...opts, relativeTo: earliest, roundingMode: "floor" });
TemporalHelpers.assertDuration(resultNegative, 0, 0, 0, -1e9, 0, 0, 0, 0, 0, 0, "result of rounding -1 ns down to -10^9 days");

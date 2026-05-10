// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Rounding for roundingIncrement option
info: |
    sec-temporal-totemporalroundingincrement:
      3. Let _integerIncrement_ be truncate(ℝ(_increment_)).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(0, 0, 0, 1);
const options = {
  smallestUnit: "days",
  roundingMode: "expand",
  relativeTo: new Temporal.PlainDate(2000, 1, 1),
};
const result = instance.round({ ...options, roundingIncrement: 2.5 });
TemporalHelpers.assertDuration(result, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, "roundingIncrement 2.5 truncates to 2");
const result2 = instance.round({ ...options, roundingIncrement: 1e9 + 0.5 });
TemporalHelpers.assertDuration(result2, 0, 0, 0, 1e9, 0, 0, 0, 0, 0, 0, "roundingIncrement 1e9 + 0.5 truncates to 1e9");

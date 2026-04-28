// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Correctly handle special case where rounding value is at upper bound
info: |
  sec-temporal-nudgetocalendarunit:
    1. If _progress_ = 1, then
      1. Let _roundedUnit_ be abs(_r2_).
    1. Else,
      1. Let _roundedUnit_ be ApplyUnsignedRoundingMode(abs(_total_), abs(_r1_),
         abs(_r2_), _unsignedRoundingMode_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(0, 11);
const relativeTo = new Temporal.PlainDate(2023, 5, 31);
const result = instance.round({ relativeTo, smallestUnit: "months", roundingMode: "ceil" });
TemporalHelpers.assertDuration(result, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0);

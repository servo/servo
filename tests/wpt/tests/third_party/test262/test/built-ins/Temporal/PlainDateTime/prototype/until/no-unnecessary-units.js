// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Do not return Durations with unnecessary units
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const lastFeb21 = new Temporal.PlainDateTime(2021, 2, 28);
const lastFeb22 = new Temporal.PlainDateTime(2022, 2, 28);

TemporalHelpers.assertDuration(
  lastFeb21.until(lastFeb22, { largestUnit: "months" }),
  0, 12, 0, 0, 0, 0, 0, 0, 0, 0,
  "does not include higher units than necessary (largest unit = months)"
);

TemporalHelpers.assertDuration(
  lastFeb21.until(lastFeb22, { largestUnit: "years" }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "does not include higher units than necessary (largest unit = years)"
);

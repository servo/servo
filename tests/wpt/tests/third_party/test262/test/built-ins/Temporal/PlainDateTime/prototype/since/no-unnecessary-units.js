// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Do not return Durations with unnecessary units
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb2 = new Temporal.PlainDateTime(2020, 2, 2, 0, 0);
const feb28 = new Temporal.PlainDateTime(2021, 2, 28, 0, 0);

TemporalHelpers.assertDuration(
  feb28.since(feb2),
  0, 0, 0, 392, 0, 0, 0, 0, 0, 0,
  "does not include higher units than necessary (no largest unit)"
);

TemporalHelpers.assertDuration(
  feb28.since(feb2, { largestUnit: "months" }),
  0, 12, 0, 26, 0, 0, 0, 0, 0, 0,
  "does not include higher units than necessary (largest unit = months)"
);

TemporalHelpers.assertDuration(
  feb28.since(feb2, { largestUnit: "years" }),
  1, 0, 0, 26, 0, 0, 0, 0, 0, 0,
  "does not include higher units than necessary (largest unit = years)"
);

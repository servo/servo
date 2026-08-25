// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Days are the default level of specificity
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb_1_2020 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb_1_2021 = new Temporal.PlainDateTime(2021, 2, 1, 0, 0);

TemporalHelpers.assertDuration(
  feb_1_2021.since(feb_1_2020),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "defaults to returning days (no options)"
);

TemporalHelpers.assertDuration(
  feb_1_2021.since(feb_1_2020, { largestUnit: "auto" }),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "defaults to returning days (largest unit = auto)"
);

TemporalHelpers.assertDuration(
  feb_1_2021.since(feb_1_2020, { largestUnit: "days" }),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "defaults to returning days (largest unit = days)"
);

const dt = new Temporal.PlainDateTime(2020, 2, 1, 0, 0, 0, 0, 0, 1);

TemporalHelpers.assertDuration(
  dt.since(feb_1_2020),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
  "defaults to returning days (nanosecond)"
);

TemporalHelpers.assertDuration(
  feb_1_2021.since(dt),
  0, 0, 0, 365, 23, 59, 59, 999, 999, 999,
  "defaults to returning days (nanosecond)"
);

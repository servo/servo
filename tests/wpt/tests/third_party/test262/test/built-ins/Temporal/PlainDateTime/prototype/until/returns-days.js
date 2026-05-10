// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Return days by default
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2021, 2, 1, 0, 0);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "auto" }),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "defaults to returning days (largest unit = auto)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "days" }),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 0,
  "defaults to returning days (largest unit = days)"
);

TemporalHelpers.assertDuration(
  feb20.until(new Temporal.PlainDateTime(2021, 2, 1, 0, 0, 0, 0, 0, 1)),
  0, 0, 0, 366, 0, 0, 0, 0, 0, 1,
  "returns nanoseconds if argument is PDT with non-zero nanoseconds"
);

const dt = new Temporal.PlainDateTime(2020, 2, 1, 0, 0, 0, 0, 0, 1);

TemporalHelpers.assertDuration(
  dt.until(feb21),
  0, 0, 0, 365, 23, 59, 59, 999, 999, 999,
  "one nanosecond away from one year away"
);

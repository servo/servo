// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Largest unit is respected
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2021, 2, 1, 0, 0);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "years" }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "can return lower or higher units (years)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "months" }),
  0, 12, 0, 0, 0, 0, 0, 0, 0, 0,
  "can return lower or higher units (months)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "weeks" }),
  0, 0, 52, 2, 0, 0, 0, 0, 0, 0,
  "can return lower or higher units (weeks)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "hours" }),
  0, 0, 0, 0, 8784, 0, 0, 0, 0, 0,
  "can return lower or higher units (hours)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "minutes" }),
  0, 0, 0, 0, 0, 527040, 0, 0, 0, 0,
  "can return lower or higher units (minutes)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "seconds" }),
  0, 0, 0, 0, 0, 0, 31622400, 0, 0, 0,
  "can return lower or higher units (seconds)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "milliseconds" }),
  0, 0, 0, 0, 0, 0, 0, 31622400000, 0, 0,
  "can return lower or higher units (milliseconds)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "microseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 31622400000000, 0,
  "can return lower or higher units (microseconds)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "nanoseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 31622400000000000,
  "can return lower or higher units (nanoseconds)"
);

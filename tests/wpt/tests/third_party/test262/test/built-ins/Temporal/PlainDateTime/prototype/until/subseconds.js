// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Returned granularity may be finer than seconds
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2020, 2, 2, 0, 0, 0, 250, 250, 250);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "milliseconds" }),
  0, 0, 0, 0, 0, 0, 0, 86400250, 250, 250,
  "can return subseconds (millisecond precision)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "microseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 86400250250, 250,
  "can return subseconds (microsecond precision)"
);

TemporalHelpers.assertDuration(
  feb20.until(feb21, { largestUnit: "nanoseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 86400250250250,
  "can return subseconds (nanosecond precision)"
);

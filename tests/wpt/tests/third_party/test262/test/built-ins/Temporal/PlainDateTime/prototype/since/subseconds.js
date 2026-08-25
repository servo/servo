// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Returned granularity may be finer than seconds
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2020, 2, 2, 0, 0, 0, 250, 250, 250);

TemporalHelpers.assertDuration(
  feb21.since(feb20, { largestUnit: "milliseconds" }),
  0, 0, 0, 0, 0, 0, 0, 86400250, 250, 250,
  "can return subseconds (milliseconds)"
);

TemporalHelpers.assertDuration(
  feb21.since(feb20, { largestUnit: "microseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 86400250250, 250,
  "can return subseconds (microseconds)"
);

TemporalHelpers.assertDuration(
  feb21.since(feb20, { largestUnit: "nanoseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 0, 86400250250250,
  "can return subseconds (nanoseconds)"
);

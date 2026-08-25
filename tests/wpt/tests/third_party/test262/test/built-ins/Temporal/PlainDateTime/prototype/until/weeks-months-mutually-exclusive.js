// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Weeks and months are mutually exclusive
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);
const laterDateTime = dt.add({ days: 42, hours: 3 });

TemporalHelpers.assertDuration(
  dt.until(laterDateTime, { largestUnit: "weeks" }),
  0, 0, 6, 0, 3, 0, 0, 0, 0, 0,
  "weeks and months mutually exclusive (prefer weeks)"
);

TemporalHelpers.assertDuration(
  dt.until(laterDateTime, { largestUnit: "months" }),
  0, 1, 0, 12, 3, 0, 0, 0, 0, 0,
  "weeks and months mutually exclusive (prefer months)"
);

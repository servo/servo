// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Leap second is constrained in both an ISO string and a property bag
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24);

let relativeTo = "2016-12-31T23:59:60";
const result1 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result1,
  1, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for PlainDate relativeTo"
);

relativeTo = "2016-12-31T23:59:60+00:00[UTC]";
const result2 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result2,
  1, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for ZonedDateTime relativeTo"
);

relativeTo = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result3 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result3,
  1, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "second: 60 is valid in a property bag for PlainDate relativeTo"
);

relativeTo = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60, timeZone: "UTC" };
const result4 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result4,
  1, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "second: 60 is valid in a property bag for ZonedDateTime relativeTo"
);

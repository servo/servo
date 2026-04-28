// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Maximum and minimum dates can be used as relativeTo parameter
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(0);

let relativeTo = '-271821-04-19';
const result1 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result1,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "minimum date is a valid ISO string for PlainDate relativeTo"
);

relativeTo = "-271821-04-20T00:00+00:00[UTC]";
const result2 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result2,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "minimum date is a valid ISO string for ZonedDateTime relativeTo"
);

relativeTo = "+275760-09-13";
const result3 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result3,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "maximum date is a valid ISO string for PlainDateTime relativeTo"
);

relativeTo = "+275760-09-12T23:59:60+00:00[UTC]";
const result4 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result4,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "maximum date is a valid ISO string for ZonedDateTime relativeTo"
);

relativeTo = { year: -271821, month: 4, day: 19 };
const result5 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result5,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "maximum date is valid in a property bag for PlainDateTime relativeTo"
);

relativeTo = { year: 275760, month: 9, day: 13 };
const result6 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result6,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "maximum date is valid in a property bag for PlainDateTime relativeTo"
);

relativeTo = { year: -271821, month: 4, day: 20, hour: 0, minute: 0, second: 0 };
const result7 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result7,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "minimum date is valid in a property bag for ZonedDateTime relativeTo"
);

relativeTo = { year: 275760, month: 9, day: 12, hour: 23, minute: 59, second: 60, timeZone: "UTC" };
const result8 = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDuration(
  result8,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "maximum date is valid in a property bag for ZonedDateTime relativeTo"
);

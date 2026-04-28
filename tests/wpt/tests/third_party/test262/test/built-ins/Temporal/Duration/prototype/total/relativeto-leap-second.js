// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Leap second is constrained in both an ISO string and a property bag
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24);

let relativeTo = "2016-12-31T23:59:60";
const result1 = instance.total({ unit: "days", relativeTo });
assert.sameValue(
  result1,
  366,
  "leap second is a valid ISO string for PlainDate relativeTo"
);

relativeTo = "2016-12-31T23:59:60+00:00[UTC]";
const result2 = instance.total({ unit: "days", relativeTo });
assert.sameValue(
  result2,
  366,
  "leap second is a valid ISO string for ZonedDateTime relativeTo"
);

relativeTo = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result3 = instance.total({ unit: "days", relativeTo });
assert.sameValue(
  result3,
  366,
  "second: 60 is valid in a property bag for PlainDate relativeTo"
);

relativeTo = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60, timeZone: "UTC" };
const result4 = instance.total({ unit: "days", relativeTo });
assert.sameValue(
  result4,
  366,
  "second: 60 is valid in a property bag for ZonedDateTime relativeTo"
);

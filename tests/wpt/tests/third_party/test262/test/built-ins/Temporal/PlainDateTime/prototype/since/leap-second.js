// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: Leap second is a valid ISO string for PlainDateTime
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2016, 12, 31, 23, 59, 59);

let arg = "2016-12-31T23:59:60";
const result1 = instance.since(arg);
TemporalHelpers.assertDuration(
  result1,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for PlainDateTime"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result2 = instance.since(arg);
TemporalHelpers.assertDuration(
  result2,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "second: 60 is ignored in property bag for PlainDateTime"
);

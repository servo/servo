// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Leap second is a valid ISO string for PlainDateTime
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";

const result1 = Temporal.PlainDateTime.from(arg);
TemporalHelpers.assertPlainDateTime(
  result1,
  2016, 12, "M12", 31, 23, 59, 59, 0, 0, 0,
  "leap second is a valid ISO string for PlainDateTime"
);

const result2 = Temporal.PlainDateTime.from(arg);
TemporalHelpers.assertPlainDateTime(
  result2,
  2016, 12, "M12", 31, 23, 59, 59, 0, 0, 0,
  "leap second is a valid ISO string for PlainDateTime even with overflow: reject"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };

const result3 = Temporal.PlainDateTime.from(arg);
TemporalHelpers.assertPlainDateTime(
  result3,
  2016, 12, "M12", 31, 23, 59, 59, 0, 0, 0,
  "second: 60 is constrained in property bag for PlainDateTime"
);

assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from(arg, { overflow: "reject" }),
  "second: 60 is rejected in property bag for PlainDateTime with overflow: reject"
);

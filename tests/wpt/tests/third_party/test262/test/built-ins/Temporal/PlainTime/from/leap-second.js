// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Leap second is a valid ISO string for PlainTime
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";

const result1 = Temporal.PlainTime.from(arg);
TemporalHelpers.assertPlainTime(
  result1,
  23, 59, 59, 0, 0, 0,
  "leap second is a valid ISO string for PlainTime"
);

const result2 = Temporal.PlainTime.from(arg, { overflow: "reject" });
TemporalHelpers.assertPlainTime(
  result2,
  23, 59, 59, 0, 0, 0,
  "leap second is a valid ISO string for PlainTime"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };

const result3 = Temporal.PlainTime.from(arg);
TemporalHelpers.assertPlainTime(
  result3,
  23, 59, 59, 0, 0, 0,
  "second: 60 is ignored in property bag for PlainTime"
);

assert.throws(
  RangeError,
  () => Temporal.PlainTime.from(arg, { overflow: "reject" }),
  "second: 60 is rejected in property bag for PlainTime with overflow: reject"
);

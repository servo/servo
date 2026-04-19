// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Leap second is a valid ISO string for PlainYearMonth
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2016, 12);

let arg = "2016-12-31T23:59:60";
const result1 = instance.since(arg);
TemporalHelpers.assertDuration(
  result1,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for PlainYearMonth"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };
const result2 = instance.since(arg);
TemporalHelpers.assertDuration(
  result2,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "second: 60 is ignored in property bag for PlainYearMonth"
);

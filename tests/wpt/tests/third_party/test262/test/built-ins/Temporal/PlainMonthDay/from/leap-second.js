// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Leap second is a valid ISO string for PlainMonthDay
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";

const result1 = Temporal.PlainMonthDay.from(arg);
TemporalHelpers.assertPlainMonthDay(
  result1,
  "M12", 31,
  "leap second is a valid ISO string for PlainMonthDay"
);

const result2 = Temporal.PlainMonthDay.from(arg, { overflow: "reject" });
TemporalHelpers.assertPlainMonthDay(
  result2,
  "M12", 31,
  "leap second is a valid ISO string for PlainMonthDay"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };

const result3 = Temporal.PlainMonthDay.from(arg);
TemporalHelpers.assertPlainMonthDay(
  result3,
  "M12", 31,
  "second: 60 is ignored in property bag for PlainMonthDay"
);

const result4 = Temporal.PlainMonthDay.from(arg, { overflow: "reject" });
TemporalHelpers.assertPlainMonthDay(
  result4,
  "M12", 31,
  "second: 60 is ignored in property bag for PlainMonthDay even with overflow: reject"
);

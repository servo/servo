// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Leap second is a valid ISO string for PlainYearMonth
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let arg = "2016-12-31T23:59:60";

const result1 = Temporal.PlainYearMonth.from(arg);
TemporalHelpers.assertPlainYearMonth(
  result1,
  2016, 12, "M12",
  "leap second is a valid ISO string for PlainYearMonth"
);

const result2 = Temporal.PlainYearMonth.from(arg, { overflow: "reject" });
TemporalHelpers.assertPlainYearMonth(
  result2,
  2016, 12, "M12",
  "leap second is a valid ISO string for PlainYearMonth"
);

arg = { year: 2016, month: 12, day: 31, hour: 23, minute: 59, second: 60 };

const result3 = Temporal.PlainYearMonth.from(arg);
TemporalHelpers.assertPlainYearMonth(
  result3,
  2016, 12, "M12",
  "second: 60 is ignored in property bag for PlainYearMonth"
);

const result4 = Temporal.PlainYearMonth.from(arg, { overflow: "reject" });
TemporalHelpers.assertPlainYearMonth(
  result4,
  2016, 12, "M12",
  "second: 60 is ignored in property bag for PlainYearMonth even with overflow: reject"
);

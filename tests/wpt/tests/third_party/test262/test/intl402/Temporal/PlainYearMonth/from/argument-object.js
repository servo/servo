// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: An object argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const monthDayItem = { calendar: "gregory", era: "ce", eraYear: 2019, month: 11, get day() { throw new Test262Error("should not read the day property") } };
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from(monthDayItem),
  2019, 11, "M11", "month with day", "ce", 2019);

const monthCodeDayItem = { calendar: "gregory", era: "ce", eraYear: 2019, monthCode: "M11", get day() { throw new Test262Error("should not read the day property") } };
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from(monthCodeDayItem),
  2019, 11, "M11", "monthCode with day", "ce", 2019);

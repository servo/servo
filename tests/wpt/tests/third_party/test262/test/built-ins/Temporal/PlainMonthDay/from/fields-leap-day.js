// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Basic tests for PlainMonthDay.from(string).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

// Leap years
["reject", "constrain"].forEach((overflow) => {
  const string = Temporal.PlainMonthDay.from("02-29", { overflow });
  TemporalHelpers.assertPlainMonthDay(string, "M02", 29, `from ${overflow} string`);

  const monthCode = Temporal.PlainMonthDay.from({ monthCode: "M02", day: 29 }, { overflow });
  TemporalHelpers.assertPlainMonthDay(monthCode, "M02", 29, `from ${overflow} with monthCode`);

  const implicit = Temporal.PlainMonthDay.from({ month: 2, day: 29 }, { overflow });
  TemporalHelpers.assertPlainMonthDay(implicit, "M02", 29, `from ${overflow} without year`);

  const explicit = Temporal.PlainMonthDay.from({ month: 2, day: 29, year: 1996 }, { overflow });
  TemporalHelpers.assertPlainMonthDay(explicit, "M02", 29, `from ${overflow} with leap year`);
});

// Non-leap years
assert.throws(RangeError,
  () => Temporal.PlainMonthDay.from({ month: 2, day: 29, year: 2001 }, { overflow: "reject" }),
  "from reject with non-leap year");

const nonLeap = Temporal.PlainMonthDay.from({ month: 2, day: 29, year: 2001 }, { overflow: "constrain" });
TemporalHelpers.assertPlainMonthDay(nonLeap, "M02", 28, "from constrain with non-leap year");

assert.throws(RangeError,
  () => Temporal.PlainMonthDay.from({ month: 2, day: 29, year: 2001, calendar: "iso8601" }, { overflow: "reject" }),
  "from reject with non-leap year and explicit calendar");

const nonLeapCalendar = Temporal.PlainMonthDay.from({ month: 2, day: 29, year: 2001, calendar: "iso8601" }, { overflow: "constrain" });
TemporalHelpers.assertPlainMonthDay(nonLeapCalendar, "M02", 28, "from constrain with non-leap year and explicit calendar");

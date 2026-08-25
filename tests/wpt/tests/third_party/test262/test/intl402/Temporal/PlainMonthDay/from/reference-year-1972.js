// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Deterministic choosing of the reference year
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const result1 = Temporal.PlainMonthDay.from({ year: 2021, monthCode: "M02", day: 29, calendar: "gregory" });
TemporalHelpers.assertPlainMonthDay(
  result1, "M02", 28,
  "year and monthCode determine if calendar date exists, but reference year should be 1972",
  1972
);

const result2 = Temporal.PlainMonthDay.from({ year: 2021, month: 2, day: 29, calendar: "gregory" }, { overflow: "constrain" });
TemporalHelpers.assertPlainMonthDay(
  result2, "M02", 28,
  "year and month determine if calendar date exists, but reference year should be 1972",
  1972
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ year: 2021, monthCode: "M02", day: 29, calendar: "gregory" }, { overflow: "reject" }),
  "RangeError thrown if monthCode and day does not exist in given year and overflow is reject"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ year: 2021, month: 2, day: 29, calendar: "gregory" }, { overflow: "reject" }),
  "RangeError thrown if month and day does not exist in given year and overflow is reject"
);

const result3 = Temporal.PlainMonthDay.from({ monthCode: "M01", day: 1, calendar: "hebrew" });
TemporalHelpers.assertPlainMonthDay(
  result3, "M01", 1,
  "reference year should be 1972 if date exists in 1972",
  1972
);

const result4 = Temporal.PlainMonthDay.from({ monthCode: "M05L", day: 1, calendar: "hebrew" });
TemporalHelpers.assertPlainMonthDay(
  result4, "M05L", 1,
  "reference year should be the latest ISO year before 1972 if date does not exist in 1972",
  1970
);

const result5 = Temporal.PlainMonthDay.from({ year: 5781, monthCode: "M02", day: 30, calendar: "hebrew" });
TemporalHelpers.assertPlainMonthDay(
  result5, "M02", 29,
  "year and monthCode determine if calendar date exists, and reference year must agree (Cheshvan 5781 has 29 days)",
  1972
);

const result6 = Temporal.PlainMonthDay.from({ year: 5781, month: 2, day: 30, calendar: "hebrew" }, { overflow: "constrain" });
TemporalHelpers.assertPlainMonthDay(
  result6, "M02", 29,
  "year and month determine if calendar date exists, and reference year must agree (Cheshvan 5781 has 29 days)",
  1972
);

const result7 = Temporal.PlainMonthDay.from({ monthCode: "M02", day: 30, calendar: "hebrew" });
TemporalHelpers.assertPlainMonthDay(
  result7, "M02", 30,
  "reference year must be the latest ISO year at or before 1972 that includes monthCode and day (Cheshvan 5781 has 29 days)",
  1971
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ year: 5781, monthCode: "M02", day: 30, calendar: "hebrew" }, { overflow: "reject" }),
  "RangeError thrown if monthCode and day does not exist in given year and overflow is reject"
);

assert.throws(
  RangeError,
  () => Temporal.PlainMonthDay.from({ year: 5781, month: 2, day: 30, calendar: "hebrew" }, { overflow: "reject" }),
  "RangeError thrown if month and day does not exist in given year and overflow is reject"
);

const result8 = Temporal.PlainMonthDay.from({ monthCode: "M04", day: 26, calendar: "hebrew" });
TemporalHelpers.assertPlainMonthDay(
  result8, "M04", 26,
  "reference date should be the later one, if two options exist in ISO year 1972",
  1972
);
assert.sameValue(result8.toString(), "1972-12-31[u-ca=hebrew]", "reference date");

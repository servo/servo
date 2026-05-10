// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.calendar.prototype.yearmonthfromfields
description: Reference ISO day is chosen to be the first of the calendar month
info: |
  6.d. Perform ! CreateDataPropertyOrThrow(_fields_, *"day"*, *1*<sub>𝔽</sub>).
    e. Let _result_ be ? CalendarDateToISO(_calendar_.[[Identifier]], _fields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const result4 = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M04", day: 20, calendar: "hebrew" });
TemporalHelpers.assertPlainYearMonth(
  result4,
  5782, 4, "M04",
  "reference day is the first of the calendar month even if day is given",
  "am", /* era year = */ 5782, /* reference day = */ 5
);
const isoYearMonth = result4.toString().slice(0, 7);
assert.sameValue(isoYearMonth, "2021-12", "Tevet 5782 begins in ISO 2021-12");

const result5 = Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M05L", calendar: "hebrew" }, { overflow: "constrain" });
TemporalHelpers.assertPlainYearMonth(
  result5,
  5783, 6, "M06",
  "month code M05L does not exist in year 5783 (overflow constrain); Hebrew calendar constrains Adar I to Adar",
  "am", /* era year = */ 5783, /* reference day = */ 22
);

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 5783, monthCode: "M05L", calendar: "hebrew" }, { overflow: "reject" }),
  "month code M05L does not exist in year 5783 (overflow reject)",
);

const result6 = Temporal.PlainYearMonth.from({ year: 5783, month: 13, calendar: "hebrew" }, { overflow: "constrain" });
TemporalHelpers.assertPlainYearMonth(
  result6,
  5783, 12, "M12",
  "month 13 does not exist in year 5783 (overflow constrain)",
  "am", /* era year = */ 5783, /* reference day = */ 18
);

assert.throws(
  RangeError,
  () => Temporal.PlainYearMonth.from({ year: 5783, month: 13, calendar: "hebrew" }, { overflow: "reject" }),
  "month 13 does not exist in year 5783 (overflow reject)",
);

const result7 = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M04", day: 50, calendar: "hebrew" }, { overflow: "constrain" });
TemporalHelpers.assertPlainYearMonth(
  result7,
  5782, 4, "M04",
  "reference day is set correctly even if day is out of range (overflow constrain)",
  "am", /* era year = */ 5782, /* reference day = */ 5
);

const result8 = Temporal.PlainYearMonth.from({ year: 5782, monthCode: "M04", day: 50, calendar: "hebrew" }, { overflow: "reject" });
TemporalHelpers.assertPlainYearMonth(
  result8,
  5782, 4, "M04",
  "reference day is set correctly even if day is out of range (overflow reject)",
  "am", /* era year = */ 5782, /* reference day = */ 5
);

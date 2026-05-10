// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: An object argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M11" }),
  2019, 11, "M11", "Only monthCode");
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from({ year: 2019, month: 11 }),
  2019, 11, "M11", "Only month");
assert.throws(RangeError,
  () => Temporal.PlainYearMonth.from({ year: 2019, month: 11, monthCode: "M12" }),
  "Mismatch between month and monthCode");

const monthDayItem = { year: 2019, month: 11, get day() { throw new Test262Error("should not read the day property") } };
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from(monthDayItem),
  2019, 11, "M11", "month with day");

const monthCodeDayItem = { year: 2019, monthCode: "M11", get day() { throw new Test262Error("should not read the day property") } };
TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from(monthCodeDayItem),
  2019, 11, "M11", "monthCode with day");

assert.throws(TypeError,
  () => Temporal.PlainYearMonth.from({}),
  "No properties");
assert.throws(TypeError,
  () => Temporal.PlainYearMonth.from({ year: 2019 }),
  "Only year");
assert.throws(TypeError,
  () => Temporal.PlainYearMonth.from({ year: 2019, months: 6 }),
  "Year and plural 'months'");
assert.throws(TypeError,
  () => Temporal.PlainYearMonth.from({ month: 6 }),
  "Only month");
assert.throws(TypeError,
  () => Temporal.PlainYearMonth.from({ monthCode: "M06" }),
  "Only monthCode");
assert.throws(TypeError,
  () => Temporal.PlainYearMonth.from({ year: undefined, month: 6 }),
  "year explicit undefined");

TemporalHelpers.assertPlainYearMonth(Temporal.PlainYearMonth.from({ year: 1976, month: 11, months: 12 }),
  1976, 11, "M11", "Plural property ignored");

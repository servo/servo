// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: Basic tests for with
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const ym = Temporal.PlainYearMonth.from("2019-10");

TemporalHelpers.assertPlainYearMonth(ym.with({ year: 2020 }), 2020, 10, "M10", "year");
TemporalHelpers.assertPlainYearMonth(ym.with({ month: 9 }), 2019, 9, "M09", "month");
TemporalHelpers.assertPlainYearMonth(ym.with({ monthCode: "M09" }), 2019, 9, "M09", "monthCode");

assert.throws(RangeError, () => ym.with({ month: 9, monthCode: "M10" }), "month/monthCode mismatch");
TemporalHelpers.assertPlainYearMonth(ym.with({ month: 1, years: 2020 }), 2019, 1, "M01", "plural 'years'");

const withDay = ym.with({ year: 2019, get day() { throw new Test262Error("should not read the day property") } });
TemporalHelpers.assertPlainYearMonth(withDay, 2019, 10, "M10", "day property");
const isoDay = Number(withDay.toString({ calendarName: "always" }).split("-")[2].slice(0, 2));
assert.sameValue(isoDay, 1);

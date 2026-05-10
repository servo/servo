// Copyright 2025 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Basic tests for with
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const cases = {
  year2000: Temporal.ZonedDateTime.from({ year: 2000, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC" }),
  year1976: Temporal.ZonedDateTime.from({ year: 1976, monthCode: "M11", day: 18, hour: 12, minute: 34, timeZone: "UTC" }),
  year1: Temporal.ZonedDateTime.from({ year: 1, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC" }),
};

for (const [name, inCal] of Object.entries(cases)) {

  var afterWithDay = inCal.with({ day: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithDay.toPlainDateTime(),
    inCal.year, inCal.month, inCal.monthCode, 1,  12, 34, 0, 0, 0, 0, `${name} after setting day to 1`);

  var afterWithMonth = afterWithDay.with({ month: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithMonth.toPlainDateTime(),
    inCal.year, 1, "M01", 1,  12, 34, 0, 0, 0, 0, `${name} after setting month to 1`);

  var afterWithYear = afterWithMonth.with({ year: 2220 });
  TemporalHelpers.assertPlainDateTime(afterWithYear.toPlainDateTime(),
    2220, 1, "M01", 1,  12, 34, 0, 0, 0, 0, `${name} after setting year to 2220`);

  afterWithYear = inCal.with({ year: 2019 });
  TemporalHelpers.assertPlainDateTime(afterWithYear.toPlainDateTime(),
    2019, inCal.month, inCal.monthCode, inCal.day,  12, 34, 0, 0, 0, 0, `${name} after setting year to 2019`);

  afterWithMonth = afterWithYear.with({ month: 5 });
  TemporalHelpers.assertPlainDateTime(afterWithMonth.toPlainDateTime(),
    2019, 5, "M05", inCal.day,  12, 34, 0, 0, 0, 0, `${name} after setting month to 5`);

  afterWithDay = afterWithMonth.with({ day: 17 });
  TemporalHelpers.assertPlainDateTime(afterWithDay.toPlainDateTime(),
    2019, 5, "M05", 17,  12, 34, 0, 0, 0, 0, `${name} after setting day to 17`);
}

// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: with should work for Dangi calendar dates
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "dangi";

const cases = {
  year2000: {
    year: 1999,
    month: 11,
    monthCode: "M11",
    day: 25,
    calendar
  },
  year1900: {
    year: 1899,
    month: 12,
    monthCode: "M12",
    day: 1,
    calendar
  },
  year2050: {
    year: 2049,
    month: 11,
    day: 21,
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainDate.from(result);

  var afterWithDay = inCal.with({ day: 1 });
  TemporalHelpers.assertPlainDate(afterWithDay, inCal.year, inCal.month, inCal.monthCode, 1, `${name} (after setting day)`);

  var afterWithMonth = afterWithDay.with({ month: 1 });
  TemporalHelpers.assertPlainDate(afterWithMonth, inCal.year, 1, "M01", 1, `${name} (after setting month)`);

  var afterWithYear = afterWithMonth.with({ year: 2025 });
  TemporalHelpers.assertPlainDate(afterWithYear, 2025, 1, "M01", 1, `${name} (after setting year)`);
}

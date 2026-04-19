// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Test `with` with Chinese calendar
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const calendar = "chinese";

const cases = {
  year2000: {
    year: 1999,
    month: 11,
    monthCode: "M11",
    day: 25,
    timeZone: "UTC",
    calendar
  },
  year1900: {
    year: 1899,
    month: 12,
    monthCode: "M12",
    day: 1,
    timeZone: "UTC",
    calendar
  },
  year2100: {
    year: 2099,
    month: 11,
    day: 21,
    timeZone: "UTC",
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.ZonedDateTime.from(result);

  const afterWithDay = inCal.with({ day: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithDay.toPlainDateTime(), inCal.year, inCal.month, inCal.monthCode, 1, 0, 0, 0, 0, 0, 0, `${name} (after setting day)`);

  const afterWithMonth = afterWithDay.with({ month: 1 });
  TemporalHelpers.assertPlainDateTime(afterWithMonth.toPlainDateTime(), inCal.year, 1, "M01", 1, 0, 0, 0, 0, 0, 0, `${name} (after setting month)`);

  const afterWithYear = afterWithMonth.with({ year: 2025 });
  TemporalHelpers.assertPlainDateTime(afterWithYear.toPlainDateTime(), 2025, 1, "M01", 1,  0, 0, 0, 0, 0, 0, `${name} (after setting year)`);
}


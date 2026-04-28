// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
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
    calendar
  },
  year1900: {
    year: 1899,
    month: 12,
    monthCode: "M12",
    calendar
  },
  year2100: {
    year: 2099,
    month: 11,
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainYearMonth.from(result);

  const afterWithMonth = inCal.with({ month: 1 });
  TemporalHelpers.assertPlainYearMonth(afterWithMonth, inCal.year, 1, "M01", `${name} (after setting month)`,
    undefined, undefined, null);

  const afterWithYear = afterWithMonth.with({ year: 2025 });
  TemporalHelpers.assertPlainYearMonth(afterWithYear, 2025, 1, "M01", `${name} (after setting year)`,
    undefined, undefined, null);
}

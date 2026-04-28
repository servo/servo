// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: with should work for Persian calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "persian";

const cases = {
  year2000: {
    era: "ap",
    year: 1378,
    eraYear: 1378,
    month: 10,
    monthCode: "M10",
    calendar
  },
  year1: {
    era: "ap",
    year: -621,
    eraYear: -621,
    month: 10,
    monthCode: "M10",
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainYearMonth.from(result);

  var afterWithMonth = inCal.with({ month: 1 });
  TemporalHelpers.assertPlainYearMonth(afterWithMonth,
    inCal.year, 1, "M01", `${name} after setting month`, inCal.era, inCal.eraYear, null);

  var afterWithYear = afterWithMonth.with({ year: 1420 });
  TemporalHelpers.assertPlainYearMonth(afterWithYear,
    1420, 1, "M01", `${name} after setting year`, inCal.era, 1420, null);
}

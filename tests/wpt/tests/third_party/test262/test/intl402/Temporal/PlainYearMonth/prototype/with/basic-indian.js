// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: with should work for Indian calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "indian";

const cases = {
  year2000: {
    year: 1921,
    eraYear: 1921,
    era: "shaka",
    month: 10,
    monthCode: "M10",
    calendar
  },
  year1: {
    year: -78,
    eraYear: -78,
    era: "shaka",
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

  var afterWithYear = afterWithMonth.with({ year: 5860 });
  TemporalHelpers.assertPlainYearMonth(afterWithYear,
    5860, 1, "M01", `${name} after setting year`, inCal.era, 5860, null);
}

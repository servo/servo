// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.with
description: with should work for Coptic calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "coptic";

const cases = {
  year2000: {
    era: "am",
    eraYear: 1716,
    year: 1716,
    month: 4,
    monthCode: "M04",
    calendar
  },
  year1: {
    era: "am",
    eraYear: -283,
    year: -283,
    month: 5,
    monthCode: "M05",
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainYearMonth.from(result);

  var afterWithMonth = inCal.with({ month: 1 });
  TemporalHelpers.assertPlainYearMonth(afterWithMonth,
    inCal.year, 1, "M01", `${name} after setting month`, inCal.era, inCal.eraYear, null);

  var afterWithYear = afterWithMonth.with({ year: 1917 });
  TemporalHelpers.assertPlainYearMonth(afterWithYear,
    1917, 1, "M01", `${name} after setting year`, inCal.era, 1917, null);
}

// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
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
    day: 11,
    calendar
  },
  year1: {
    year: -78,
    eraYear: -78,
    era: "shaka",
    month: 10,
    monthCode: "M10",
    day: 11,
    calendar
  }
};
for (var [name, result] of Object.entries(cases)) {
  const inCal = Temporal.PlainDate.from(result);

  var afterWithDay = inCal.with({ day: 1 });
  TemporalHelpers.assertPlainDate(afterWithDay,
    inCal.year, inCal.month, inCal.monthCode, 1,  `${name} after setting day`, inCal.era, inCal.eraYear);

  var afterWithMonth = afterWithDay.with({ month: 1 });
  TemporalHelpers.assertPlainDate(afterWithMonth,
    inCal.year, 1, "M01", 1,  `${name} after setting month`, inCal.era, inCal.eraYear);

  var afterWithYear = afterWithMonth.with({ year: 5860 });
  TemporalHelpers.assertPlainDate(afterWithYear,
    5860, 1, "M01", 1,  `${name} after setting year`, inCal.era, 5860);
}

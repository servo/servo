// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: with should work for islamic-civil calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "islamic-civil";

const cases = {
  year2000: {
    year: 1420,
    eraYear: 1420,
    era: "ah",
    month: 9,
    monthCode: "M09",
    day: 24,
    calendar
  },
  year1: {
    year: -640,
    eraYear: 641,
    era: "bh",
    month: 5,
    monthCode: "M05",
    day: 18,
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

  var afterWithYear = afterWithMonth.with({ year: 1700 });
  TemporalHelpers.assertPlainDate(afterWithYear,
    1700, 1, "M01", 1,  `${name} after setting year`, "ah", 1700);
}

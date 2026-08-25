// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (roc calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";

// Years

const date04902 = Temporal.PlainYearMonth.from({ year: 49, monthCode: "M02", calendar });
const date04903 = Temporal.PlainYearMonth.from({ year: 49, monthCode: "M03", calendar });
const date05807 = Temporal.PlainYearMonth.from({ year: 58, monthCode: "M07", calendar });
const date08607 = Temporal.PlainYearMonth.from({ year: 86, monthCode: "M07", calendar });
const date08612 = Temporal.PlainYearMonth.from({ year: 86, monthCode: "M12", calendar });
const date09006 = Temporal.PlainYearMonth.from({ year: 90, monthCode: "M06", calendar });
const date10807 = Temporal.PlainYearMonth.from({ year: 108, monthCode: "M07", calendar });
const date10812 = Temporal.PlainYearMonth.from({ year: 108, monthCode: "M12", calendar });
const date10903 = Temporal.PlainYearMonth.from({ year: 109, monthCode: "M03", calendar });
const date10912 = Temporal.PlainYearMonth.from({ year: 109, monthCode: "M12", calendar });
const date11001 = Temporal.PlainYearMonth.from({ year: 110, monthCode: "M01", calendar });
const date11003 = Temporal.PlainYearMonth.from({ year: 110, monthCode: "M03", calendar });
const date11007 = Temporal.PlainYearMonth.from({ year: 110, monthCode: "M07", calendar });
const date11008 = Temporal.PlainYearMonth.from({ year: 110, monthCode: "M08", calendar });
const date11009 = Temporal.PlainYearMonth.from({ year: 110, monthCode: "M09", calendar });
const date11107 = Temporal.PlainYearMonth.from({ year: 111, monthCode: "M07", calendar });
const date11109 = Temporal.PlainYearMonth.from({ year: 111, monthCode: "M09", calendar });
const date12007 = Temporal.PlainYearMonth.from({ year: 120, monthCode: "M07", calendar });
const date12012 = Temporal.PlainYearMonth.from({ year: 120, monthCode: "M12", calendar });

const tests = [
  [
    date11007, date11007, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date11007, date11008, "1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date10912, date11001, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date11007, date11009, "2 months which both have 31 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date11007, date11107, "1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date11007, date12007, "10 years",
    ["years", -10, 0],
    ["months", 0, -120],
  ],
  [
    date11007, date11109, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date11007, date12012, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date08612, date11007, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date08607, date11007, "24 years",
    ["years", -24, 0],
  ],
  [
    date04902, date10903, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date11003, date11007, "4 months",
    ["years", 0, -4],
  ],
  [
    date10903, date11007, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date04903, date11007, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date10812, date11007, "1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date08612, date09006, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date10912, date11007, "7 months",
    ["years", 0, -7],
  ],
  [
    date08612, date11007, "23 years, 7 months",
    ["years", -23, -7],
  ],
  [
    date10812, date11003, "1 year, 3 months",
    ["years", -1, -3],
  ],
  [
    date05807, date10807, "crossing epoch",
    ["years", -50, 0],
  ],
  [
    date11008, date11007, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date11001, date10912, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date11009, date11007, "negative 2 months which both have 31 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date11107, date11007, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date12007, date11007, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 120],
  ],
  [
    date11109, date11007, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date12012, date11007, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date11007, date08612, "negative 23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date11007, date08607, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date10903, date04902, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date11007, date11003, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date11007, date10903, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date11007, date04903, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date11007, date10812, "negative 1 year, 7 months",
    ["years", 1, 7],
  ],
  [
    date11007, date10912, "negative 7 months",
    ["years", 0, 7],
  ],
  [
    date11007, date08612, "negative 23 years, 7 months",
    ["years", 23, 7],
  ],
  [
    date11003, date10812, "negative 1 year, 3 months",
    ["years", 1, 3],
  ],
  [
    date10807, date05807, "crossing epoch",
    ["years", 50, 0],
  ],
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}

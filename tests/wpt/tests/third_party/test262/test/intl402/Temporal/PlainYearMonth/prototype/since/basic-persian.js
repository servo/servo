// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (Persian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";

// Years

const date137512 = Temporal.PlainYearMonth.from({ year: 1375, monthCode: "M12", calendar });
const date137906 = Temporal.PlainYearMonth.from({ year: 1379, monthCode: "M06", calendar });
const date134202 = Temporal.PlainYearMonth.from({ year: 1342, monthCode: "M02", calendar });
const date134302 = Temporal.PlainYearMonth.from({ year: 1343, monthCode: "M02", calendar });
const date134303 = Temporal.PlainYearMonth.from({ year: 1343, monthCode: "M03", calendar });
const date138007 = Temporal.PlainYearMonth.from({ year: 1380, monthCode: "M07", calendar });
const date138012 = Temporal.PlainYearMonth.from({ year: 1380, monthCode: "M12", calendar });
const date140203 = Temporal.PlainYearMonth.from({ year: 1402, monthCode: "M03", calendar });
const date140212 = Temporal.PlainYearMonth.from({ year: 1402, monthCode: "M12", calendar });
const date140303 = Temporal.PlainYearMonth.from({ year: 1403, monthCode: "M03", calendar });
const date140312 = Temporal.PlainYearMonth.from({ year: 1403, monthCode: "M12", calendar });
const date140401 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M01", calendar });
const date140403 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M03", calendar });
const date140404 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M04", calendar });
const date140406 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M06", calendar });
const date140407 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M07", calendar });
const date140408 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M08", calendar });
const date140409 = Temporal.PlainYearMonth.from({ year: 1404, monthCode: "M09", calendar });
const date140507 = Temporal.PlainYearMonth.from({ year: 1405, monthCode: "M07", calendar });
const date140509 = Temporal.PlainYearMonth.from({ year: 1405, monthCode: "M09", calendar });
const date141407 = Temporal.PlainYearMonth.from({ year: 1414, monthCode: "M07", calendar });
const date141412 = Temporal.PlainYearMonth.from({ year: 1414, monthCode: "M12", calendar });

const tests = [
  [
    date140407, date140407, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date140407, date140408, "1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date140312, date140401, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date140407, date140409, "2 months which both have 30 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date140404, date140406, "2 months which both have 31 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date140407, date140507, "1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date140407, date141407, "10 years",
    ["years", -10, 0],
    ["months", 0, -120],
  ],
  [
    date140407, date140509, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date137512, date137906, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date140407, date141412, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date138012, date140407, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date138007, date140407, "24 years",
    ["years", -24, 0],
  ],
  [
    date134302, date140303, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date140403, date140407, "4 months",
    ["years", 0, -4],
  ],
  [
    date140303, date140407, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date134303, date140407, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date140212, date140407, "1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date140312, date140407, "7 months",
    ["years", 0, -7],
  ],
  [
    date138012, date140407, "23 years, 7 months",
    ["years", -23, -7],
  ],
  [
    date140212, date140403, "1 year, 3 months",
    ["years", -1, -3],
  ],
  [
    date140408, date140407, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date140401, date140312, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date140409, date140407, "negative 2 months which both have 30 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date140507, date140407, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date141407, date140407, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 120],
  ],
  [
    date140509, date140407, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date141412, date140407, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date140407, date138012, "negative 23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date140407, date138007, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date140203, date134202, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date140407, date140403, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date140407, date140303, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date140407, date134303, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date140407, date140212, "negative 1 year, 7 months",
    ["years", 1, 7],
  ],
  [
    date140407, date140312, "negative 7 months",
    ["years", 0, 7],
  ],
  [
    date140407, date138012, "negative 23 years, 7 months",
    ["years", 23, 7],
  ],
  [
    date140403, date140212, "negative 1 year, 3 months",
    ["years", 1, 3],
  ]
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

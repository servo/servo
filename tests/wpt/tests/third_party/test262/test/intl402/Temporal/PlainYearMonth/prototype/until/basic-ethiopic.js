// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
  (ethiopic calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethiopic";

// Years

const date196002 = Temporal.PlainYearMonth.from({ year: 1960, monthCode: "M02", calendar });
const date196003 = Temporal.PlainYearMonth.from({ year: 1960, monthCode: "M03", calendar });
const date196103 = Temporal.PlainYearMonth.from({ year: 1961, monthCode: "M03", calendar });
const date199707 = Temporal.PlainYearMonth.from({ year: 1997, monthCode: "M07", calendar });
const date199712 = Temporal.PlainYearMonth.from({ year: 1997, monthCode: "M12", calendar });
const date199713 = Temporal.PlainYearMonth.from({ year: 1997, monthCode: "M13", calendar });
const date200001 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M01", calendar });
const date200010 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M10", calendar });
const date200105 = Temporal.PlainYearMonth.from({ year: 2001, monthCode: "M05", calendar });
const date201912 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M12", calendar });
const date201913 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M13", calendar });
const date202003 = Temporal.PlainYearMonth.from({ year: 2020, monthCode: "M03", calendar });
const date202012 = Temporal.PlainYearMonth.from({ year: 2020, monthCode: "M12", calendar });
const date202013 = Temporal.PlainYearMonth.from({ year: 2020, monthCode: "M13", calendar });
const date202101 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01", calendar });
const date202102 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M02", calendar });
const date202103 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M03", calendar });;
const date202104 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M04", calendar });
const date202106 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M06", calendar });
const date202107 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M07", calendar });
const date202108 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M08", calendar });
const date202112 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M12", calendar });
const date202207 = Temporal.PlainYearMonth.from({ year: 2022, monthCode: "M07", calendar });
const date202209 = Temporal.PlainYearMonth.from({ year: 2022, monthCode: "M09", calendar });
const date202307 = Temporal.PlainYearMonth.from({ year: 2023, monthCode: "M07", calendar });
const date203107 = Temporal.PlainYearMonth.from({ year: 2031, monthCode: "M07", calendar });
const date203112 = Temporal.PlainYearMonth.from({ year: 2031, monthCode: "M12", calendar });

const tests = [
  [
    date202107, date202107, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date202107, date202108, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date202013, date202101, "1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date202101, date202102, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date202207, date202307, "1 year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    date202107, date203107, "10 years",
    ["years", 10, 0],
    ["months", 0, 130],
  ],
  [
    date202107, date202209, "1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date202107, date203112, "10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date199712, date202106, "23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date199707, date202107, "24 years",
    ["years", 24, 0],
  ],
  [
    date199707, date202106, "23 years, 12 months",
    ["years", 23, 12],
  ],
  [
    date196002, date202003, "60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date202103, date202107, "4 months",
    ["years", 0, 4],
  ],
  [
    date202003, date202107, "1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date199712, date200105, "3 years, 6 months",
    ["years", 3, 6],
  ],
  [
    date196003, date202107, "61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date201912, date202107, "1 year, 8 months",
    ["years", 1, 8],
  ],
  [
    date202013, date202106, "6 months",
    ["years", 0, 6],
  ],
  [
    date200001, date200010, "40 weeks",
  ],
  [
    date199713, date202106, "23 years, 6 months",
    ["years", 23, 6],
  ],
  [
    date201913, date202102, "1 year, 2 months",
    ["years", 1, 2],
  ],
  [
    date202108, date202107, "negative 1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date202101, date202013, "negative 1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date202104, date202102, "negative 2 months which both have 30 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date202307, date202207, "negative 1 year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    date203107, date202107, "negative 10 years",
    ["years", -10, 0],
    ["months", 0, -130],
  ],
  [
    date202209, date202107, "negative 1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date203112, date202107, "negative 10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date202107, date199712, "negative 23 years and 8 months",
    ["years", -23, -8],
  ],
  [
    date202107, date199707, "negative 24 years",
    ["years", -24, 0],
  ],
  [
    date202003, date196002, "negative 60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date202107, date202103, "negative 4 months",
    ["years", 0, -4],
  ],
  [
    date202207, date202103, "negative 1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date202207, date196103, "negative 61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date202307, date202112, "negative 1 year, 8 months",
    ["years", -1, -8],
  ],
  [
    date202107, date202012, "negative 8 months",
    ["years", 0, -8],
  ],
  [
    date202107, date199712, "negative 23 years, 8 months",
    ["years", -23, -8],
  ],
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      `${descr} (largestUnit = ${largestUnit})`
    );
  }
}

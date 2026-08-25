// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (japanese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";

// Years

const date196002 = Temporal.PlainYearMonth.from({ year: 1960, monthCode: "M02", calendar });
const date196003 = Temporal.PlainYearMonth.from({ year: 1960, monthCode: "M03", calendar });
const date196907 = Temporal.PlainYearMonth.from({ year: 1969, monthCode: "M07", calendar });
const date199707 = Temporal.PlainYearMonth.from({ year: 1997, monthCode: "M07", calendar });
const date199712 = Temporal.PlainYearMonth.from({ year: 1997, monthCode: "M12", calendar });
const date200001 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M01", calendar });
const date200010 = Temporal.PlainYearMonth.from({ year: 2000, monthCode: "M10", calendar });
const date200106 = Temporal.PlainYearMonth.from({ year: 2001, monthCode: "M06", calendar });
const date201907 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M07", calendar });
const date201912 = Temporal.PlainYearMonth.from({ year: 2019, monthCode: "M12", calendar });
const date202003 = Temporal.PlainYearMonth.from({ year: 2020, monthCode: "M03", calendar });
const date202012 = Temporal.PlainYearMonth.from({ year: 2020, monthCode: "M12", calendar });
const date202101 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M01", calendar });
const date202103 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M03", calendar });
const date202107 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M07", calendar });
const date202108 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M08", calendar });
const date202109 = Temporal.PlainYearMonth.from({ year: 2021, monthCode: "M09", calendar });
const date202207 = Temporal.PlainYearMonth.from({ year: 2022, monthCode: "M07", calendar });
const date202209 = Temporal.PlainYearMonth.from({ year: 2022, monthCode: "M09", calendar });
const date203107 = Temporal.PlainYearMonth.from({ year: 2031, monthCode: "M07", calendar });
const date203112 = Temporal.PlainYearMonth.from({ year: 2031, monthCode: "M12", calendar });

const tests = [
  [
    date202107, date202107, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date202107, date202108, "1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date202012, date202101, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date202107, date202109, "2 months which both have 31 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date202107, date202207, "1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date202107, date203107, "10 years",
    ["years", -10, 0],
    ["months", 0, -120],
  ],
  [
    date202107, date202209, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date202107, date203112, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date199712, date202107, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date199707, date202107, "24 years",
    ["years", -24, 0],
  ],
  [
    date196002, date202003, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date202103, date202107, "4 months",
    ["years", 0, -4],
  ],
  [
    date202003, date202107, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date199712, date200106, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date196003, date202107, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date201912, date202107, "1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date202012, date202107, "7 months",
    ["years", 0, -7],
  ],
  [
    date200001, date200010, "40 weeks",
  ],
  [
    date199712, date202107, "23 years, 7 months",
    ["years", -23, -7],
  ],
  [
    date201912, date202103, "1 year, 3 months",
    ["years", -1, -3],
  ],
  [
    date196907, date201907, "crossing epoch",
    ["years", -50, 0],
  ],
  [
    date202108, date202107, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date202101, date202012, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date202109, date202107, "negative 2 months which both have 31 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date202207, date202107, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date203107, date202107, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 120],
  ],
  [
    date202209, date202107, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date203112, date202107, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date202107, date199712, "negative 23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date202107, date199707, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date202003, date196002, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date202107, date202103, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date202107, date202003, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date202107, date196003, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date202107, date201912, "negative 1 year, 7 months",
    ["years", 1, 7],
  ],
  [
    date202107, date202012, "negative 7 months",
    ["years", 0, 7],
  ],
  [
    date202107, date199712, "negative 23 years, 7 months",
    ["years", 23, 7],
  ],
  [
    date202103, date201912, "negative 1 year, 3 months",
    ["years", 1, 3],
  ],
  [
    date201907, date196907, "crossing epoch",
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

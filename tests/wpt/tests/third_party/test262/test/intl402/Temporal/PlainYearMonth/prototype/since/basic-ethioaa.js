// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (ethioaa calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethioaa";

// Years

const date745302 = Temporal.PlainYearMonth.from({ year: 7453, monthCode: "M02", calendar });
const date745303 = Temporal.PlainYearMonth.from({ year: 7453, monthCode: "M03", calendar });
const date745403 = Temporal.PlainYearMonth.from({ year: 7454, monthCode: "M03", calendar });
const date748912 = Temporal.PlainYearMonth.from({ year: 7489, monthCode: "M12", calendar });
const date749007 = Temporal.PlainYearMonth.from({ year: 7490, monthCode: "M07", calendar });
const date749012 = Temporal.PlainYearMonth.from({ year: 7490, monthCode: "M12", calendar });
const date749013 = Temporal.PlainYearMonth.from({ year: 7490, monthCode: "M13", calendar });
const date749305 = Temporal.PlainYearMonth.from({ year: 7493, monthCode: "M05", calendar });
const date751212 = Temporal.PlainYearMonth.from({ year: 7512, monthCode: "M12", calendar });
const date751213 = Temporal.PlainYearMonth.from({ year: 7512, monthCode: "M13", calendar });
const date751303 = Temporal.PlainYearMonth.from({ year: 7513, monthCode: "M03", calendar });
const date751313 = Temporal.PlainYearMonth.from({ year: 7513, monthCode: "M13", calendar });
const date751401 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M01", calendar });
const date751402 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M02", calendar });
const date751403 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M03", calendar });
const date751404 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M04", calendar });
const date751406 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M06", calendar });
const date751407 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M07", calendar });
const date751408 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M08", calendar });
const date751412 = Temporal.PlainYearMonth.from({ year: 7514, monthCode: "M12", calendar });
const date751507 = Temporal.PlainYearMonth.from({ year: 7515, monthCode: "M07", calendar });
const date751509 = Temporal.PlainYearMonth.from({ year: 7515, monthCode: "M09", calendar });
const date751607 = Temporal.PlainYearMonth.from({ year: 7516, monthCode: "M07", calendar });
const date752407 = Temporal.PlainYearMonth.from({ year: 7524, monthCode: "M07", calendar });
const date752412 = Temporal.PlainYearMonth.from({ year: 7524, monthCode: "M12", calendar });

const tests = [
  [
    date751407, date751407, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date751407, date751408, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date751313, date751401, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date751401, date751402, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date751407, date751507, "1 year",
    ["years", -1, 0],
    ["months", 0, -13],
  ],
  [
    date751407, date752407, "10 years",
    ["years", -10, 0],
    ["months", 0, -130],
  ],
  [
    date751407, date751509, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date751407, date752412, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date749012, date751406, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date749007, date751407, "24 years",
    ["years", -24, 0],
  ],
  [
    date749007, date751406, "23 years, 12 months",
    ["years", -23, -12],
  ],
  [
    date745302, date751303, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date751403, date751407, "4 months",
    ["years", 0, -4],
  ],
  [
    date751303, date751407, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date748912, date749305, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date745303, date751407, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date751212, date751407, "1 year, 8 months",
    ["years", -1, -8],
  ],
  [
    date751313, date751406, "6 months",
    ["years", 0, -6],
  ],
  [
    date749013, date751406, "23 years, 6 months",
    ["years", -23, -6],
  ],
  [
    date751213, date751402, "1 year, 2 months",
    ["years", -1, -2],
  ],
  [
    date751408, date751407, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date751401, date751313, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date751404, date751402, "negative 2 months which both have 30 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date751507, date751407, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 13],
  ],
  [
    date752407, date751407, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 130],
  ],
  [
    date751509, date751407, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date752412, date751407, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date751407, date749012, "negative 23 years and 8 months",
    ["years", 23, 8],
  ],
  [
    date751407, date749007, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date751303, date745302, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date751407, date751403, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date751507, date751403, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date751507, date745403, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date751607, date751412, "negative 1 year, 8 months",
    ["years", 1, 8],
  ],
  [
    date751507, date751412, "negative 8 months",
    ["years", 0, 8],
  ],
  [
    date751407, date749012, "negative 23 years, 8 months",
    ["years", 23, 8],
  ],
  [
    date751403, date751212, "negative 1 year, 4 months",
    ["years", 1, 4],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, 0, 0, 0, 0, 0, 0, 0, 0,
      `${descr} (largestUnit = ${largestUnit})`
    );
  }
}

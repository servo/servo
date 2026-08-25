// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (indian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "indian";

// Years

const date188102 = Temporal.PlainYearMonth.from({ year: 1881, monthCode: "M02", calendar });
const date188202 = Temporal.PlainYearMonth.from({ year: 1882, monthCode: "M02", calendar });
const date188203 = Temporal.PlainYearMonth.from({ year: 1882, monthCode: "M03", calendar });
const date191907 = Temporal.PlainYearMonth.from({ year: 1919, monthCode: "M07", calendar });
const date191912 = Temporal.PlainYearMonth.from({ year: 1919, monthCode: "M12", calendar });
const date192306 = Temporal.PlainYearMonth.from({ year: 1923, monthCode: "M06", calendar });
const date194103 = Temporal.PlainYearMonth.from({ year: 1941, monthCode: "M03", calendar });
const date194112 = Temporal.PlainYearMonth.from({ year: 1941, monthCode: "M12", calendar });
const date194203 = Temporal.PlainYearMonth.from({ year: 1942, monthCode: "M03", calendar });
const date194212 = Temporal.PlainYearMonth.from({ year: 1942, monthCode: "M12", calendar });
const date194301 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M01", calendar });
const date194302 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M02", calendar });
const date194303 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M03", calendar });;
const date194304 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M04", calendar });
const date194306 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M06", calendar });
const date194307 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M07", calendar });
const date194308 = Temporal.PlainYearMonth.from({ year: 1943, monthCode: "M08", calendar });
const date194407 = Temporal.PlainYearMonth.from({ year: 1944, monthCode: "M07", calendar });
const date194409 = Temporal.PlainYearMonth.from({ year: 1944, monthCode: "M09", calendar });
const date195307 = Temporal.PlainYearMonth.from({ year: 1953, monthCode: "M07", calendar });
const date195312 = Temporal.PlainYearMonth.from({ year: 1953, monthCode: "M12", calendar });

const tests = [
  [
    date194307, date194307, "same day",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date194307, date194308, "1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date194212, date194301, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date194302, date194304, "2 months which both have 31 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date194307, date194407, "1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date194307, date195307, "10 years",
    ["years", -10, 0],
    ["months", 0, -120],
  ],
  [
    date194307, date194409, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date194307, date195312, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date191912, date194307, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date191907, date194307, "24 years",
    ["years", -24, 0],
  ],
  [
    date188202, date194203, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date194303, date194307, "4 months",
    ["years", 0, -4],
  ],
  [
    date194203, date194307, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date191912, date192306, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date188203, date194307, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date194112, date194307, "1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date194212, date194307, "7 months",
    ["years", 0, -7],
  ],
  [
    date191912, date194307, "23 years, 7 months",
    ["years", -23, -7],
  ],
  [
    date194112, date194303, "1 year, 3 months",
    ["years", -1, -3],
  ],
  [
    date194308, date194307, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date194301, date194212, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date194306, date194304, "negative 2 months which both have 31 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date194407, date194307, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date195307, date194307, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 120],
  ],
  [
    date194409, date194307, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date195312, date194307, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date194307, date191912, "negative 23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date194307, date191907, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date194103, date188102, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date194307, date194303, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date194307, date194203, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date194307, date188203, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date194307, date194112, "negative 1 year, 7 months",
    ["years", 1, 7],
  ],
  [
    date194307, date194212, "negative 7 months",
    ["years", 0, 7],
  ],
  [
    date194307, date191912, "negative 23 years, 7 months",
    ["years", 23, 7],
  ],
  [
    date194303, date194112, "negative 1 year, 3 months",
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

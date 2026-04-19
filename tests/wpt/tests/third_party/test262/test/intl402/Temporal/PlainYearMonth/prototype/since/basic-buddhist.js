// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (buddhist calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "buddhist";

const date250302 = Temporal.PlainYearMonth.from({ year: 2503, monthCode: "M02", calendar });
const date250303 = Temporal.PlainYearMonth.from({ year: 2503, monthCode: "M03", calendar });
const date251207 = Temporal.PlainYearMonth.from({ year: 2512, monthCode: "M07", calendar });
const date254007 = Temporal.PlainYearMonth.from({ year: 2540, monthCode: "M07", calendar });
const date254012 = Temporal.PlainYearMonth.from({ year: 2540, monthCode: "M12", calendar });
const date255512 = Temporal.PlainYearMonth.from({ year: 2555, monthCode: "M12", calendar });
const date255606 = Temporal.PlainYearMonth.from({ year: 2556, monthCode: "M06", calendar });
const date255906 = Temporal.PlainYearMonth.from({ year: 2559, monthCode: "M06", calendar });
const date256207 = Temporal.PlainYearMonth.from({ year: 2562, monthCode: "M07", calendar });
const date256212 = Temporal.PlainYearMonth.from({ year: 2562, monthCode: "M12", calendar });
const date256303 = Temporal.PlainYearMonth.from({ year: 2563, monthCode: "M03", calendar });
const date256312 = Temporal.PlainYearMonth.from({ year: 2563, monthCode: "M12", calendar });
const date256401 = Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M01", calendar });
const date256403 = Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M03", calendar });
const date256407 = Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M07", calendar });
const date256408 = Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M08", calendar });
const date256409 = Temporal.PlainYearMonth.from({ year: 2564, monthCode: "M09", calendar });
const date256507 = Temporal.PlainYearMonth.from({ year: 2565, monthCode: "M07", calendar });
const date256509 = Temporal.PlainYearMonth.from({ year: 2565, monthCode: "M09", calendar });
const date257407 = Temporal.PlainYearMonth.from({ year: 2574, monthCode: "M07", calendar });
const date257412 = Temporal.PlainYearMonth.from({ year: 2574, monthCode: "M12", calendar });

const tests = [
  [
    date256407, date256407, "same month",
    ["years", 0, 0],
    ["months", 0, 0],
  ],
  [
    date256407, date256408, "1 month in same year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date256312, date256401, "1 month in different year",
    ["years", 0, -1],
    ["months", 0, -1],
  ],
  [
    date256407, date256409, "2 months which both have 31 days",
    ["years", 0, -2],
    ["months", 0, -2],
  ],
  [
    date256407, date256507, "1 year",
    ["years", -1, 0],
    ["months", 0, -12],
  ],
  [
    date256407, date257407, "10 years",
    ["years", -10, 0],
    ["months", 0, -120],
  ],
  [
    date256407, date256509, "1 year 2 months",
    ["years", -1, -2],
  ],
  [
    date256407, date257412, "10 years and 5 months",
    ["years", -10, -5],
  ],
  [
    date254012, date256407, "23 years and 7 months",
    ["years", -23, -7],
  ],
  [
    date254007, date256407, "24 years",
    ["years", -24, 0],
  ],
  [
    date250302, date256303, "60 years, 1 month",
    ["years", -60, -1],
  ],
  [
    date256403, date256407, "4 months",
    ["years", 0, -4],
  ],
  [
    date256303, date256407, "1 year, 4 months",
    ["years", -1, -4],
  ],
  [
    date250303, date256407, "61 years, 4 months",
    ["years", -61, -4],
  ],
  [
    date256212, date256407, "1 year, 7 months",
    ["years", -1, -7],
  ],
  [
    date255512, date255606, "6 months",
    ["months", 0, -6],
  ],
  [
    date256212, date256403, "1 year, 3 months",
    ["years", -1, -3],
  ],
  [
    date255512, date255906, "3 years, 6 months",
    ["years", -3, -6],
  ],
  [
    date251207, date256207, "crossing epoch",
    ["years", -50, 0],
  ],
  [
    date256408, date256407, "negative 1 month in same year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date256401, date256312, "negative 1 month in different year",
    ["years", 0, 1],
    ["months", 0, 1],
  ],
  [
    date256409, date256407, "negative 2 months which both have 31 days",
    ["years", 0, 2],
    ["months", 0, 2],
  ],
  [
    date256507, date256407, "negative 1 year",
    ["years", 1, 0],
    ["months", 0, 12],
  ],
  [
    date257407, date256407, "negative 10 years",
    ["years", 10, 0],
    ["months", 0, 120],
  ],
  [
    date256509, date256407, "negative 1 year 2 months",
    ["years", 1, 2],
  ],
  [
    date257412, date256407, "negative 10 years and 5 months",
    ["years", 10, 5],
  ],
  [
    date256407, date254012, "negative 23 years and 7 months",
    ["years", 23, 7],
  ],
  [
    date256407, date254007, "negative 24 years",
    ["years", 24, 0],
  ],
  [
    date256303, date250302, "negative 60 years, 1 month",
    ["years", 60, 1],
  ],
  [
    date256407, date256403, "negative 4 months",
    ["years", 0, 4],
  ],
  [
    date256407, date256303, "negative 1 year, 4 months",
    ["years", 1, 4],
  ],
  [
    date256407, date250303, "negative 61 years, 4 months",
    ["years", 61, 4],
  ],
  [
    date256407, date256212, "negative 1 year, 7 months",
    ["years", 1, 7],
  ],
  [
    date256407, date256312, "negative 7 months",
    ["years", 0, 7],
  ],
  [
    date256407, date254012, "negative 23 years, 7 months",
    ["years", 23, 7],
  ],
  [
    date256207, date251207, "crossing epoch",
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

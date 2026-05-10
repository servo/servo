// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
  (indian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "indian";

// Years

const date18800216 = Temporal.PlainDate.from({ year: 1880, monthCode: "M02", day: 16, calendar });
const date18810216 = Temporal.PlainDate.from({ year: 1881, monthCode: "M02", day: 16, calendar });
const date18820216 = Temporal.PlainDate.from({ year: 1882, monthCode: "M02", day: 16, calendar });
const date18820330 = Temporal.PlainDate.from({ year: 1882, monthCode: "M03", day: 30, calendar });
const date18910724 = Temporal.PlainDate.from({ year: 1891, monthCode: "M07", day: 24, calendar });
const date19190616 = Temporal.PlainDate.from({ year: 1919, monthCode: "M06", day: 16, calendar });
const date19190617 = Temporal.PlainDate.from({ year: 1919, monthCode: "M06", day: 17, calendar });
const date19190716 = Temporal.PlainDate.from({ year: 1919, monthCode: "M07", day: 16, calendar });
const date19191201 = Temporal.PlainDate.from({ year: 1919, monthCode: "M12", day: 1, calendar });
const date19191216 = Temporal.PlainDate.from({ year: 1919, monthCode: "M12", day: 16, calendar });
const date19191230 = Temporal.PlainDate.from({ year: 1919, monthCode: "M12", day: 30, calendar });
const date19220101 = Temporal.PlainDate.from({ year: 1922, monthCode: "M01", day: 1, calendar });
const date19221005 = Temporal.PlainDate.from({ year: 1922, monthCode: "M10", day: 5, calendar });
const date19221201 = Temporal.PlainDate.from({ year: 1922, monthCode: "M12", day: 1, calendar });
const date19230601 = Temporal.PlainDate.from({ year: 1923, monthCode: "M06", day: 1, calendar });
const date19230618 = Temporal.PlainDate.from({ year: 1923, monthCode: "M06", day: 18, calendar });
const date19410101 = Temporal.PlainDate.from({ year: 1941, monthCode: "M01", day: 1, calendar });
const date19410201 = Temporal.PlainDate.from({ year: 1941, monthCode: "M02", day: 1, calendar });
const date19410316 = Temporal.PlainDate.from({ year: 1941, monthCode: "M03", day: 16, calendar });
const date19410724 = Temporal.PlainDate.from({ year: 1941, monthCode: "M07", day: 24, calendar });
const date19411230 = Temporal.PlainDate.from({ year: 1941, monthCode: "M12", day: 30, calendar });
const date19420201 = Temporal.PlainDate.from({ year: 1942, monthCode: "M02", day: 1, calendar });
const date19420316 = Temporal.PlainDate.from({ year: 1942, monthCode: "M03", day: 16, calendar });
const date19420330 = Temporal.PlainDate.from({ year: 1942, monthCode: "M03", day: 30, calendar });
const date19421216 = Temporal.PlainDate.from({ year: 1942, monthCode: "M12", day: 16, calendar });
const date19421230 = Temporal.PlainDate.from({ year: 1942, monthCode: "M12", day: 30, calendar });
const date19430105 = Temporal.PlainDate.from({ year: 1943, monthCode: "M01", day: 5, calendar });
const date19430107 = Temporal.PlainDate.from({ year: 1943, monthCode: "M01", day: 7, calendar });
const date19430116 = Temporal.PlainDate.from({ year: 1943, monthCode: "M01", day: 16, calendar });
const date19430201 = Temporal.PlainDate.from({ year: 1943, monthCode: "M02", day: 1, calendar });
const date19430205 = Temporal.PlainDate.from({ year: 1943, monthCode: "M02", day: 5, calendar });
const date19430216 = Temporal.PlainDate.from({ year: 1943, monthCode: "M02", day: 16, calendar });
const date19430228 = Temporal.PlainDate.from({ year: 1943, monthCode: "M02", day: 28, calendar })
const date19430304 = Temporal.PlainDate.from({ year: 1943, monthCode: "M03", day: 4, calendar });;
const date19430305 = Temporal.PlainDate.from({ year: 1943, monthCode: "M03", day: 5, calendar });
const date19430307 = Temporal.PlainDate.from({ year: 1943, monthCode: "M03", day: 7, calendar });
const date19430330 = Temporal.PlainDate.from({ year: 1943, monthCode: "M03", day: 30, calendar });
const date19430416 = Temporal.PlainDate.from({ year: 1943, monthCode: "M04", day: 16, calendar });
const date19430516 = Temporal.PlainDate.from({ year: 1943, monthCode: "M05", day: 16, calendar });
const date19430613 = Temporal.PlainDate.from({ year: 1943, monthCode: "M06", day: 13, calendar });
const date19430615 = Temporal.PlainDate.from({ year: 1943, monthCode: "M06", day: 15, calendar });
const date19430616 = Temporal.PlainDate.from({ year: 1943, monthCode: "M06", day: 16, calendar });
const date19430617 = Temporal.PlainDate.from({ year: 1943, monthCode: "M06", day: 17, calendar });
const date19430713 = Temporal.PlainDate.from({ year: 1943, monthCode: "M07", day: 13, calendar });
const date19430714 = Temporal.PlainDate.from({ year: 1943, monthCode: "M07", day: 14, calendar });
const date19430715 = Temporal.PlainDate.from({ year: 1943, monthCode: "M07", day: 15, calendar });
const date19430716 = Temporal.PlainDate.from({ year: 1943, monthCode: "M07", day: 16, calendar });
const date19430717 = Temporal.PlainDate.from({ year: 1943, monthCode: "M07", day: 17, calendar });
const date19430723 = Temporal.PlainDate.from({ year: 1943, monthCode: "M07", day: 23, calendar });
const date19430813 = Temporal.PlainDate.from({ year: 1943, monthCode: "M08", day: 13, calendar });
const date19430816 = Temporal.PlainDate.from({ year: 1943, monthCode: "M08", day: 16, calendar });
const date19430817 = Temporal.PlainDate.from({ year: 1943, monthCode: "M08", day: 17, calendar });
const date19430916 = Temporal.PlainDate.from({ year: 1943, monthCode: "M09", day: 16, calendar });
const date19440228 = Temporal.PlainDate.from({ year: 1944, monthCode: "M02", day: 28, calendar });
const date19440716 = Temporal.PlainDate.from({ year: 1944, monthCode: "M07", day: 16, calendar });
const date19440719 = Temporal.PlainDate.from({ year: 1944, monthCode: "M07", day: 19, calendar });
const date19440919 = Temporal.PlainDate.from({ year: 1944, monthCode: "M09", day: 19, calendar });
const date19530716 = Temporal.PlainDate.from({ year: 1953, monthCode: "M07", day: 16, calendar });
const date19531216 = Temporal.PlainDate.from({ year: 1953, monthCode: "M12", day: 16, calendar });

const tests = [
  [
    date19430716, date19430716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date19430716, date19430717, "one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date19430716, date19430723, "7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date19430716, date19430816, "1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date19421216, date19430116, "1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date19430105, date19430205, "1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date19430516, date19430617, "1 month and 1 day in a month with 31 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 32],
  ],
  [
    date19430616, date19430713, "28 days across a month which has 31 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date19430216, date19430416, "2 months which both have 31 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 6],
    ["days", 0, 0, 0, 62],
  ],
  [
    date19430716, date19440716, "1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date19420201, date19430201, "start of Vaisakha",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
  ],
  [
    date19430228, date19440228, "end of Vaisakha",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
  ],
  [
    date19410101, date19410201, "length of Chaitra 1941",
    ["days", 0, 0, 0, 30],
  ],
  [
    date19430716, date19530716, "10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 120, 0, 0],
    ["weeks", 0, 0, 521, 5],
    ["days", 0, 0, 0, 3652],
  ],
  [
    date19430716, date19440719, "1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date19430716, date19440919, "1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date19430716, date19531216, "10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date19191216, date19430716, "23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date19190716, date19430716, "24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date19190716, date19430714, "23 years, 11 months and 29 days",
    ["years", 23, 11, 0, 29],
  ],
  [
    date19190616, date19430615, "23 years, 11 months and 30 days",
    ["years", 23, 11, 0, 30],
  ],
  [
    date18820216, date19420316, "60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date19430330, date19430715, "3 months and 16 days",
    ["years", 0, 3, 0, 16],
  ],
  [
    date19420330, date19430715, "1 year, 3 months and 16 days",
    ["years", 1, 3, 0, 16],
  ],
  [
    date19191201, date19230618, "3 years, 6 months and 17 days",
    ["years", 3, 6, 0, 17],
  ],
  [
    date18820330, date19430715, "61 years, 3 months and 16 days",
    ["years", 61, 3, 0, 16],
  ],
  [
    date19411230, date19430715, "1 year, 6 months and 16 days",
    ["years", 1, 6, 0, 16],
  ],
  [
    date19421230, date19430715, "6 months and 16 days",
    ["years", 0, 6, 0, 16],
  ],
  [
    date19221201, date19230601, "6 months",
    ["months", 0, 6, 0, 0],
  ],
  [
    date19220101, date19221005, "40 weeks",
    ["weeks", 0, 0, 40, 0],
    ["days", 0, 0, 0, 280],
  ],
  [
    date19191230, date19430715, "23 years, 6 months and 16 days",
    ["years", 23, 6, 0, 16],
  ],
  [
    date19411230, date19430304, "1 year, 2 months and 5 days",
    ["years", 1, 2, 0, 5],
  ],
  [
    date19430717, date19430716, "negative one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date19430723, date19430716, "negative 7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date19430816, date19430716, "negative 1 month in same year (1)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date19430116, date19421216, "negative 1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date19430205, date19430105, "negative 1 month in same year (2)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date19430617, date19430516, "negative 1 month and 1 day in a month with 31 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -32],
  ],
  [
    date19430613, date19430516, "negative 28 days across a month which has 31 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date19430616, date19430416, "negative 2 months which both have 31 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -6],
    ["days", 0, 0, 0, -62],
  ],
  [
    date19440716, date19430716, "negative 1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date19530716, date19430716, "negative 10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -120, 0, 0],
    ["weeks", 0, 0, -521, -5],
    ["days", 0, 0, 0, -3652],
  ],
  [
    date19440719, date19430716, "negative 1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date19440919, date19430716, "negative 1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date19531216, date19430716, "negative 10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date19430716, date19191216, "negative 23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date19430716, date19190716, "negative 24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date19430615, date19190616, "negative 23 years, 11 months and 30 days",
    ["years", -23, -11, 0, -30],
  ],
  [
    date19430615, date19190617, "negative 23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date19410316, date18810216, "negative 60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date19430716, date19430330, "negative 3 months and 17 days",
    ["years", 0, -3, 0, -17],
  ],
  [
    date19430716, date19420330, "negative 1 year, 3 months and 17 days",
    ["years", -1, -3, 0, -17],
  ],
  [
    date19430716, date18820330, "negative 61 years, 3 months and 17 days",
    ["years", -61, -3, 0, -17],
  ],
  [
    date19430716, date19411230, "negative 1 year, 6 months and 16 days",
    ["years", -1, -6, 0, -16],
  ],
  [
    date19430716, date19421230, "negative 6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date19430716, date19191230, "negative 23 years, 6 months and 16 days",
    ["years", -23, -6, 0, -16],
  ],
  [
    date19430305, date19411230, "negative 1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}

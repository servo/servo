// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (coptic calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "coptic";

// Years

const date17100216 = Temporal.PlainDate.from({ year: 1710, monthCode: "M02", day: 16, calendar });
const date17100330 = Temporal.PlainDate.from({ year: 1710, monthCode: "M03", day: 30, calendar });
const date17110329 = Temporal.PlainDate.from({ year: 1711, monthCode: "M03", day: 29, calendar });
const date17131201 = Temporal.PlainDate.from({ year: 1713, monthCode: "M12", day: 1, calendar });
const date17160101 = Temporal.PlainDate.from({ year: 1716, monthCode: "M01", day: 1, calendar });
const date17161011 = Temporal.PlainDate.from({ year: 1716, monthCode: "M10", day: 11, calendar });
const date17161201 = Temporal.PlainDate.from({ year: 1716, monthCode: "M12", day: 1, calendar });
const date17170501 = Temporal.PlainDate.from({ year: 1717, monthCode: "M05", day: 1, calendar });
const date17170518 = Temporal.PlainDate.from({ year: 1717, monthCode: "M05", day: 18, calendar });
const date17190724 = Temporal.PlainDate.from({ year: 1719, monthCode: "M07", day: 24, calendar });
const date17460516 = Temporal.PlainDate.from({ year: 1746, monthCode: "M05", day: 16, calendar });
const date17460616 = Temporal.PlainDate.from({ year: 1746, monthCode: "M06", day: 16, calendar });
const date17460617 = Temporal.PlainDate.from({ year: 1746, monthCode: "M06", day: 17, calendar });
const date17460622 = Temporal.PlainDate.from({ year: 1746, monthCode: "M06", day: 22, calendar });
const date17460716 = Temporal.PlainDate.from({ year: 1746, monthCode: "M07", day: 16, calendar });
const date17470616 = Temporal.PlainDate.from({ year: 1747, monthCode: "M06", day: 16, calendar });
const date17470716 = Temporal.PlainDate.from({ year: 1747, monthCode: "M07", day: 16, calendar });
const date17471216 = Temporal.PlainDate.from({ year: 1747, monthCode: "M12", day: 16, calendar });
const date17471228 = Temporal.PlainDate.from({ year: 1747, monthCode: "M12", day: 28, calendar });
const date17471230 = Temporal.PlainDate.from({ year: 1747, monthCode: "M12", day: 30, calendar });
const date17471305 = Temporal.PlainDate.from({ year: 1747, monthCode: "M13", day: 5, calendar });
const date17690101 = Temporal.PlainDate.from({ year: 1769, monthCode: "M01", day: 1, calendar });
const date17690201 = Temporal.PlainDate.from({ year: 1769, monthCode: "M02", day: 1, calendar });
const date17690724 = Temporal.PlainDate.from({ year: 1769, monthCode: "M07", day: 24, calendar });
const date17691229 = Temporal.PlainDate.from({ year: 1769, monthCode: "M12", day: 29, calendar });
const date17691230 = Temporal.PlainDate.from({ year: 1769, monthCode: "M12", day: 30, calendar });
const date17691305 = Temporal.PlainDate.from({ year: 1769, monthCode: "M13", day: 5, calendar });
const date17700201 = Temporal.PlainDate.from({ year: 1770, monthCode: "M02", day: 1, calendar });
const date17700316 = Temporal.PlainDate.from({ year: 1770, monthCode: "M03", day: 16, calendar });
const date17700330 = Temporal.PlainDate.from({ year: 1770, monthCode: "M03", day: 30, calendar });
const date17701216 = Temporal.PlainDate.from({ year: 1770, monthCode: "M12", day: 16, calendar });
const date17701229 = Temporal.PlainDate.from({ year: 1770, monthCode: "M12", day: 29, calendar });
const date17701230 = Temporal.PlainDate.from({ year: 1770, monthCode: "M12", day: 30, calendar });
const date17701305 = Temporal.PlainDate.from({ year: 1770, monthCode: "M13", day: 5, calendar });
const date17710105 = Temporal.PlainDate.from({ year: 1771, monthCode: "M01", day: 5, calendar });
const date17710107 = Temporal.PlainDate.from({ year: 1771, monthCode: "M01", day: 7, calendar });
const date17710116 = Temporal.PlainDate.from({ year: 1771, monthCode: "M01", day: 16, calendar });
const date17710201 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 1, calendar });
const date17710205 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 5, calendar });
const date17710208 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 8, calendar });
const date17710209 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 9, calendar });
const date17710210 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 10, calendar });
const date17710216 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 16, calendar });
const date17710228 = Temporal.PlainDate.from({ year: 1771, monthCode: "M02", day: 28, calendar })
const date17710303 = Temporal.PlainDate.from({ year: 1771, monthCode: "M03", day: 3, calendar });;
const date17710305 = Temporal.PlainDate.from({ year: 1771, monthCode: "M03", day: 5, calendar });
const date17710306 = Temporal.PlainDate.from({ year: 1771, monthCode: "M03", day: 6, calendar });
const date17710307 = Temporal.PlainDate.from({ year: 1771, monthCode: "M03", day: 7, calendar });
const date17710329 = Temporal.PlainDate.from({ year: 1771, monthCode: "M03", day: 29, calendar });
const date17710330 = Temporal.PlainDate.from({ year: 1771, monthCode: "M03", day: 30, calendar });
const date17710416 = Temporal.PlainDate.from({ year: 1771, monthCode: "M04", day: 16, calendar });
const date17710515 = Temporal.PlainDate.from({ year: 1771, monthCode: "M05", day: 15, calendar });
const date17710614 = Temporal.PlainDate.from({ year: 1771, monthCode: "M06", day: 14, calendar });
const date17710615 = Temporal.PlainDate.from({ year: 1771, monthCode: "M06", day: 15, calendar });
const date17710616 = Temporal.PlainDate.from({ year: 1771, monthCode: "M06", day: 16, calendar });
const date17710621 = Temporal.PlainDate.from({ year: 1771, monthCode: "M06", day: 21, calendar });
const date17710715 = Temporal.PlainDate.from({ year: 1771, monthCode: "M07", day: 15, calendar });
const date17710716 = Temporal.PlainDate.from({ year: 1771, monthCode: "M07", day: 16, calendar });
const date17710717 = Temporal.PlainDate.from({ year: 1771, monthCode: "M07", day: 17, calendar });
const date17710721 = Temporal.PlainDate.from({ year: 1771, monthCode: "M07", day: 21, calendar });
const date17710723 = Temporal.PlainDate.from({ year: 1771, monthCode: "M07", day: 23, calendar });
const date17710813 = Temporal.PlainDate.from({ year: 1771, monthCode: "M08", day: 13, calendar });
const date17710816 = Temporal.PlainDate.from({ year: 1771, monthCode: "M08", day: 16, calendar });
const date17710817 = Temporal.PlainDate.from({ year: 1771, monthCode: "M08", day: 17, calendar });
const date17710916 = Temporal.PlainDate.from({ year: 1771, monthCode: "M09", day: 16, calendar });
const date17711228 = Temporal.PlainDate.from({ year: 1771, monthCode: "M12", day: 28, calendar });
const date17720228 = Temporal.PlainDate.from({ year: 1772, monthCode: "M02", day: 28, calendar });
const date17720716 = Temporal.PlainDate.from({ year: 1772, monthCode: "M07", day: 16, calendar });
const date17720719 = Temporal.PlainDate.from({ year: 1772, monthCode: "M07", day: 19, calendar });
const date17720919 = Temporal.PlainDate.from({ year: 1772, monthCode: "M09", day: 19, calendar });
const date17730716 = Temporal.PlainDate.from({ year: 1773, monthCode: "M07", day: 16, calendar });
const date17810716 = Temporal.PlainDate.from({ year: 1781, monthCode: "M07", day: 16, calendar });
const date17811216 = Temporal.PlainDate.from({ year: 1781, monthCode: "M12", day: 16, calendar });

const tests = [
  [
    date17710716, date17710716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date17710716, date17710717, "one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date17710716, date17710723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date17710716, date17710816, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date17701305, date17710105, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date17710105, date17710205, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date17710205, date17710306, "1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date17161201, date17170501, "6 months in different year",
    ["months", 0, -6, 0, 0],
  ],
  [
    date17710205, date17710303, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date17720716, date17730716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -13, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date17710716, date17810716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -130, 0, 0],
    ["weeks", 0, 0, -521, -6],
    ["days", 0, 0, 0, -3653],
  ],
  [
    date17710716, date17720719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date17710716, date17720919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date17131201, date17170518, "3 years, 6 months and 17 days",
    ["years", -3, -6, 0, -17],
  ],
  [
    date17710716, date17811216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date17471216, date17710616, "23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date17470716, date17710716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date17470716, date17710614, "23 years, 11 months and 28 days",
    ["years", -23, -11, 0, -28],
  ],
  [
    date17470716, date17710615, "23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date17100216, date17700316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date17710330, date17710716, "3 months and 16 days",
    ["years", 0, -3, 0, -16],
  ],
  [
    date17700330, date17710716, "1 year, 3 months and 16 days",
    ["years", -1, -3, 0, -16],
  ],
  [
    date17100330, date17710716, "61 years, 3 months and 16 days",
    ["years", -61, -3, 0, -16],
  ],
  [
    date17691230, date17710716, "1 year, 7 months and 16 days",
    ["years", -1, -7, 0, -16],
  ],
  [
    date17701305, date17710621, "6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date17471305, date17710621, "23 years, 6 months and 16 days",
    ["years", -23, -6, 0, -16],
  ],
  [
    date17691305, date17710210, "1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ],
  [
    date17160101, date17161011, "40 weeks",
    ["weeks", 0, 0, -40, 0],
    ["days", 0, 0, 0, -280],
  ],
  [
    date17710717, date17710716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date17710723, date17710716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date17710816, date17710716, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date17710105, date17701305, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date17710205, date17710105, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date17710329, date17710228, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date17710307, date17710209, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date17710416, date17710216, "negative 2 months which both have 30 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 4],
    ["days", 0, 0, 0, 60],
  ],
  [
    date17730716, date17720716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 13, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date17810716, date17710716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 130, 0, 0],
    ["weeks", 0, 0, 521, 6],
    ["days", 0, 0, 0, 3653],
  ],
  [
    date17720719, date17710716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date17720919, date17710716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date17811216, date17710716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date17710716, date17471216, "negative 23 years and 8 months",
    ["years", 23, 8, 0, 0],
  ],
  [
    date17710716, date17470716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date17710615, date17460617, "negative 24 years, 12 months and 28 days",
    ["years", 24, 12, 0, 28],
  ],
  [
    date17710515, date17460516, "negative 24 years, 12 months and 29 days",
    ["years", 24, 12, 0, 29],
  ],
  [
    date17700316, date17100216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date17710716, date17710329, "negative 3 months and 17 days",
    ["years", 0, 3, 0, 17],
  ],
  [
    date17720716, date17710329, "negative 1 year, 3 months and 17 days",
    ["years", 1, 3, 0, 17],
  ],
  [
    date17720716, date17110329, "negative 61 years, 3 months and 17 days",
    ["years", 61, 3, 0, 17],
  ],
  [
    date17730716, date17711228, "negative 1 year, 7 months and 8 days",
    ["years", 1, 7, 0, 8],
  ],
  [
    date17710716, date17701229, "negative 7 months and 6 days",
    ["years", 0, 7, 0, 6],
  ],
  [
    date17710716, date17471228, "negative 23 years, 7 months and 8 days",
    ["years", 23, 7, 0, 8],
  ],
  [
    date17710305, date17691229, "negative 1 year, 3 months and 6 days",
    ["years", 1, 3, 0, 6],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      `${descr} (largestUnit = ${largestUnit})`
    );
  }
}

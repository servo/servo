// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
  (ethioaa calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethioaa";

// Years

const date74530216 = Temporal.PlainDate.from({ year: 7453, monthCode: "M02", day: 16, calendar });
const date74530330 = Temporal.PlainDate.from({ year: 7453, monthCode: "M03", day: 30, calendar });
const date74540329 = Temporal.PlainDate.from({ year: 7454, monthCode: "M03", day: 29, calendar });
const date74610724 = Temporal.PlainDate.from({ year: 7461, monthCode: "M07", day: 24, calendar });
const date74890516 = Temporal.PlainDate.from({ year: 7489, monthCode: "M05", day: 16, calendar });
const date74890616 = Temporal.PlainDate.from({ year: 7489, monthCode: "M06", day: 16, calendar });
const date74890617 = Temporal.PlainDate.from({ year: 7489, monthCode: "M06", day: 17, calendar });
const date74890622 = Temporal.PlainDate.from({ year: 7489, monthCode: "M06", day: 22, calendar });
const date74890716 = Temporal.PlainDate.from({ year: 7489, monthCode: "M07", day: 16, calendar });
const date74891201 = Temporal.PlainDate.from({ year: 7489, monthCode: "M12", day: 1, calendar });
const date74900616 = Temporal.PlainDate.from({ year: 7490, monthCode: "M06", day: 16, calendar });
const date74900716 = Temporal.PlainDate.from({ year: 7490, monthCode: "M07", day: 16, calendar });
const date74901216 = Temporal.PlainDate.from({ year: 7490, monthCode: "M12", day: 16, calendar });
const date74901228 = Temporal.PlainDate.from({ year: 7490, monthCode: "M12", day: 28, calendar });
const date74901230 = Temporal.PlainDate.from({ year: 7490, monthCode: "M12", day: 30, calendar });
const date74901305 = Temporal.PlainDate.from({ year: 7490, monthCode: "M13", day: 5, calendar });
const date74920101 = Temporal.PlainDate.from({ year: 7492, monthCode: "M01", day: 1, calendar });
const date74921011 = Temporal.PlainDate.from({ year: 7492, monthCode: "M10", day: 11, calendar });
const date74921201 = Temporal.PlainDate.from({ year: 7492, monthCode: "M12", day: 1, calendar });
const date74930501 = Temporal.PlainDate.from({ year: 7493, monthCode: "M05", day: 1, calendar });
const date74930518 = Temporal.PlainDate.from({ year: 7493, monthCode: "M05", day: 18, calendar });
const date75120101 = Temporal.PlainDate.from({ year: 7512, monthCode: "M01", day: 1, calendar });
const date75120201 = Temporal.PlainDate.from({ year: 7512, monthCode: "M02", day: 1, calendar });
const date75120724 = Temporal.PlainDate.from({ year: 7512, monthCode: "M07", day: 24, calendar });
const date75121229 = Temporal.PlainDate.from({ year: 7512, monthCode: "M12", day: 29, calendar });
const date75121230 = Temporal.PlainDate.from({ year: 7512, monthCode: "M12", day: 30, calendar });
const date75121305 = Temporal.PlainDate.from({ year: 7512, monthCode: "M13", day: 5, calendar });
const date75130201 = Temporal.PlainDate.from({ year: 7513, monthCode: "M02", day: 1, calendar });
const date75130316 = Temporal.PlainDate.from({ year: 7513, monthCode: "M03", day: 16, calendar });
const date75130330 = Temporal.PlainDate.from({ year: 7513, monthCode: "M03", day: 30, calendar });
const date75131216 = Temporal.PlainDate.from({ year: 7513, monthCode: "M12", day: 16, calendar });
const date75131229 = Temporal.PlainDate.from({ year: 7513, monthCode: "M12", day: 29, calendar });
const date75131230 = Temporal.PlainDate.from({ year: 7513, monthCode: "M12", day: 30, calendar });
const date75131305 = Temporal.PlainDate.from({ year: 7513, monthCode: "M13", day: 5, calendar });
const date75140105 = Temporal.PlainDate.from({ year: 7514, monthCode: "M01", day: 5, calendar });
const date75140107 = Temporal.PlainDate.from({ year: 7514, monthCode: "M01", day: 7, calendar });
const date75140116 = Temporal.PlainDate.from({ year: 7514, monthCode: "M01", day: 16, calendar });
const date75140201 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 1, calendar });
const date75140205 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 5, calendar });
const date75140208 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 8, calendar });
const date75140209 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 9, calendar });
const date75140210 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 10, calendar });
const date75140216 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 16, calendar });
const date75140228 = Temporal.PlainDate.from({ year: 7514, monthCode: "M02", day: 28, calendar })
const date75140303 = Temporal.PlainDate.from({ year: 7514, monthCode: "M03", day: 3, calendar });;
const date75140305 = Temporal.PlainDate.from({ year: 7514, monthCode: "M03", day: 5, calendar });
const date75140306 = Temporal.PlainDate.from({ year: 7514, monthCode: "M03", day: 6, calendar });
const date75140307 = Temporal.PlainDate.from({ year: 7514, monthCode: "M03", day: 7, calendar });
const date75140329 = Temporal.PlainDate.from({ year: 7514, monthCode: "M03", day: 29, calendar });
const date75140330 = Temporal.PlainDate.from({ year: 7514, monthCode: "M03", day: 30, calendar });
const date75140416 = Temporal.PlainDate.from({ year: 7514, monthCode: "M04", day: 16, calendar });
const date75140515 = Temporal.PlainDate.from({ year: 7514, monthCode: "M05", day: 15, calendar });
const date75140614 = Temporal.PlainDate.from({ year: 7514, monthCode: "M06", day: 14, calendar });
const date75140615 = Temporal.PlainDate.from({ year: 7514, monthCode: "M06", day: 15, calendar });
const date75140616 = Temporal.PlainDate.from({ year: 7514, monthCode: "M06", day: 16, calendar });
const date75140621 = Temporal.PlainDate.from({ year: 7514, monthCode: "M06", day: 21, calendar });
const date75140715 = Temporal.PlainDate.from({ year: 7514, monthCode: "M07", day: 15, calendar });
const date75140716 = Temporal.PlainDate.from({ year: 7514, monthCode: "M07", day: 16, calendar });
const date75140717 = Temporal.PlainDate.from({ year: 7514, monthCode: "M07", day: 17, calendar });
const date75140721 = Temporal.PlainDate.from({ year: 7514, monthCode: "M07", day: 21, calendar });
const date75140723 = Temporal.PlainDate.from({ year: 7514, monthCode: "M07", day: 23, calendar });
const date75140813 = Temporal.PlainDate.from({ year: 7514, monthCode: "M08", day: 13, calendar });
const date75140816 = Temporal.PlainDate.from({ year: 7514, monthCode: "M08", day: 16, calendar });
const date75140817 = Temporal.PlainDate.from({ year: 7514, monthCode: "M08", day: 17, calendar });
const date75140916 = Temporal.PlainDate.from({ year: 7514, monthCode: "M09", day: 16, calendar });
const date75141228 = Temporal.PlainDate.from({ year: 7514, monthCode: "M12", day: 28, calendar });
const date75141230 = Temporal.PlainDate.from({ year: 7514, monthCode: "M12", day: 30, calendar });
const date75150228 = Temporal.PlainDate.from({ year: 7515, monthCode: "M02", day: 28, calendar });
const date75150716 = Temporal.PlainDate.from({ year: 7515, monthCode: "M07", day: 16, calendar });
const date75150719 = Temporal.PlainDate.from({ year: 7515, monthCode: "M07", day: 19, calendar });
const date75150919 = Temporal.PlainDate.from({ year: 7515, monthCode: "M09", day: 19, calendar });
const date75160716 = Temporal.PlainDate.from({ year: 7516, monthCode: "M07", day: 16, calendar });
const date75240716 = Temporal.PlainDate.from({ year: 7524, monthCode: "M07", day: 16, calendar });
const date75241216 = Temporal.PlainDate.from({ year: 7524, monthCode: "M12", day: 16, calendar });

const tests = [
  [
    date75140716, date75140716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date75140716, date75140717, "one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date75140716, date75140723, "7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date75140716, date75140816, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date75131305, date75140105, "1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date75140105, date75140205, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date75140205, date75140306, "1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date75140205, date75140303, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date75140716, date75150716, "1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 13, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date75140716, date75240716, "10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 130, 0, 0],
    ["weeks", 0, 0, 521, 6],
    ["days", 0, 0, 0, 3653],
  ],
  [
    date75140716, date75150719, "1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date75140716, date75150919, "1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date75140716, date75241216, "10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date74901216, date75140616, "23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date74900716, date75140716, "24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date74900716, date75140614, "23 years, 11 months and 28 days",
    ["years", 23, 11, 0, 28],
  ],
  [
    date74900716, date75140615, "23 years, 11 months and 29 days",
    ["years", 23, 11, 0, 29],
  ],
  [
    date74530216, date75130316, "60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date75140330, date75140716, "3 months and 16 days",
    ["years", 0, 3, 0, 16],
  ],
  [
    date75130330, date75140716, "1 year, 3 months and 16 days",
    ["years", 1, 3, 0, 16],
  ],
  [
    date74891201, date74930518, "3 years, 6 months and 17 days",
    ["years", 3, 6, 0, 17],
  ],
  [
    date74530330, date75140716, "61 years, 3 months and 16 days",
    ["years", 61, 3, 0, 16],
  ],
  [
    date75121230, date75140716, "1 year, 7 months and 16 days",
    ["years", 1, 7, 0, 16],
  ],
  [
    date75131305, date75140621, "6 months and 16 days",
    ["years", 0, 6, 0, 16],
  ],
  [
    date74921201, date74930501, "6 months",
    ["months", 0, 6, 0, 0],
  ],
  [
    date74920101, date74921011, "40 weeks",
    ["weeks", 0, 0, 40, 0],
    ["days", 0, 0, 0, 280],
  ],
  [
    date74901305, date75140621, "23 years, 6 months and 16 days",
    ["years", 23, 6, 0, 16],
  ],
  [
    date75121305, date75140210, "1 year, 2 months and 5 days",
    ["years", 1, 2, 0, 5],
  ],
  [
    date75140717, date75140716, "negative one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date75140723, date75140716, "negative 7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date75140816, date75140716, "negative 1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date75140105, date75131305, "negative 1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date75140205, date75140105, "negative 1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date75140329, date75140228, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date75140307, date75140209, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date75140416, date75140216, "negative 2 months which both have 30 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -4],
    ["days", 0, 0, 0, -60],
  ],
  [
    date75150716, date75140716, "negative 1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -13, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date75240716, date75140716, "negative 10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -130, 0, 0],
    ["weeks", 0, 0, -521, -6],
    ["days", 0, 0, 0, -3653],
  ],
  [
    date75150719, date75140716, "negative 1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date75150919, date75140716, "negative 1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date75241216, date75140716, "negative 10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date75140716, date74901216, "negative 23 years and 8 months",
    ["years", -23, -8, 0, 0],
  ],
  [
    date75140716, date74900716, "negative 24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date75140615, date74890617, "negative 24 years, 12 months and 28 days",
    ["years", -24, -12, 0, -28],
  ],
  [
    date75140515, date74890516, "negative 24 years, 12 months and 29 days",
    ["years", -24, -12, 0, -29],
  ],
  [
    date75130316, date74530216, "negative 60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date75140716, date75140329, "negative 3 months and 17 days",
    ["years", 0, -3, 0, -17],
  ],
  [
    date75150716, date75140329, "negative 1 year, 3 months and 17 days",
    ["years", -1, -3, 0, -17],
  ],
  [
    date75150716, date74540329, "negative 61 years, 3 months and 17 days",
    ["years", -61, -3, 0, -17],
  ],
  [
    date75160716, date75141228, "negative 1 year, 7 months and 7 days",
    ["years", -1, -7, 0, -7],
  ],
  [
    date75150716, date75141230, "negative 7 months and 5 days",
    ["years", 0, -7, 0, -5],
  ],
  [
    date75140716, date74901228, "negative 23 years, 7 months and 7 days",
    ["years", -23, -7, 0, -7],
  ],
  [
    date75140305, date75121229, "negative 1 year, 3 months and 6 days",
    ["years", -1, -3, 0, -6],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.until(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      `${descr} (largestUnit = ${largestUnit})`
    );
  }
}

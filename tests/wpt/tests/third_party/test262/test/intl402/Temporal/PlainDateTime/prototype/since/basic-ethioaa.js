// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (ethioaa calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethioaa";

// Years

const date74530216 = Temporal.PlainDateTime.from({ year: 7453, monthCode: "M02", day: 16, hour: 12, minute: 34, calendar });
const date74530330 = Temporal.PlainDateTime.from({ year: 7453, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date74540329 = Temporal.PlainDateTime.from({ year: 7454, monthCode: "M03", day: 29, hour: 12, minute: 34, calendar });
const date74610724 = Temporal.PlainDateTime.from({ year: 7461, monthCode: "M07", day: 24, hour: 12, minute: 34, calendar });
const date74890516 = Temporal.PlainDateTime.from({ year: 7489, monthCode: "M05", day: 16, hour: 12, minute: 34, calendar });
const date74890616 = Temporal.PlainDateTime.from({ year: 7489, monthCode: "M06", day: 16, hour: 12, minute: 34, calendar });
const date74890617 = Temporal.PlainDateTime.from({ year: 7489, monthCode: "M06", day: 17, hour: 12, minute: 34, calendar });
const date74890622 = Temporal.PlainDateTime.from({ year: 7489, monthCode: "M06", day: 22, hour: 12, minute: 34, calendar });
const date74890716 = Temporal.PlainDateTime.from({ year: 7489, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date74891201 = Temporal.PlainDateTime.from({ year: 7489, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });
const date74900616 = Temporal.PlainDateTime.from({ year: 7490, monthCode: "M06", day: 16, hour: 12, minute: 34, calendar });
const date74900716 = Temporal.PlainDateTime.from({ year: 7490, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date74901216 = Temporal.PlainDateTime.from({ year: 7490, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });
const date74901228 = Temporal.PlainDateTime.from({ year: 7490, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar });
const date74901230 = Temporal.PlainDateTime.from({ year: 7490, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date74901305 = Temporal.PlainDateTime.from({ year: 7490, monthCode: "M13", day: 5, hour: 12, minute: 34, calendar });
const date74920101 = Temporal.PlainDateTime.from({ year: 7492, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date74921011 = Temporal.PlainDateTime.from({ year: 7492, monthCode: "M10", day: 11, hour: 12, minute: 34, calendar });
const date74921201 = Temporal.PlainDateTime.from({ year: 7492, monthCode: "M12", day: 1, hour: 12, minute: 34, calendar });
const date74930501 = Temporal.PlainDateTime.from({ year: 7493, monthCode: "M05", day: 1, hour: 12, minute: 34, calendar });
const date74930518 = Temporal.PlainDateTime.from({ year: 7493, monthCode: "M05", day: 18, hour: 12, minute: 34, calendar });
const date75120101 = Temporal.PlainDateTime.from({ year: 7512, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date75120201 = Temporal.PlainDateTime.from({ year: 7512, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date75120724 = Temporal.PlainDateTime.from({ year: 7512, monthCode: "M07", day: 24, hour: 12, minute: 34, calendar });
const date75121229 = Temporal.PlainDateTime.from({ year: 7512, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar });
const date75121230 = Temporal.PlainDateTime.from({ year: 7512, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date75121305 = Temporal.PlainDateTime.from({ year: 7512, monthCode: "M13", day: 5, hour: 12, minute: 34, calendar });
const date75130201 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date75130316 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M03", day: 16, hour: 12, minute: 34, calendar });
const date75130330 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date75131216 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });
const date75131229 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar });
const date75131230 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date75131305 = Temporal.PlainDateTime.from({ year: 7513, monthCode: "M13", day: 5, hour: 12, minute: 34, calendar });
const date75140105 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M01", day: 5, hour: 12, minute: 34, calendar });
const date75140107 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M01", day: 7, hour: 12, minute: 34, calendar });
const date75140116 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M01", day: 16, hour: 12, minute: 34, calendar });
const date75140201 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date75140205 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 5, hour: 12, minute: 34, calendar });
const date75140208 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 8, hour: 12, minute: 34, calendar });
const date75140209 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 9, hour: 12, minute: 34, calendar });
const date75140210 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 10, hour: 12, minute: 34, calendar });
const date75140216 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 16, hour: 12, minute: 34, calendar });
const date75140228 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar })
const date75140303 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M03", day: 3, hour: 12, minute: 34, calendar });;
const date75140305 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M03", day: 5, hour: 12, minute: 34, calendar });
const date75140306 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M03", day: 6, hour: 12, minute: 34, calendar });
const date75140307 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M03", day: 7, hour: 12, minute: 34, calendar });
const date75140329 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M03", day: 29, hour: 12, minute: 34, calendar });
const date75140330 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date75140416 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M04", day: 16, hour: 12, minute: 34, calendar });
const date75140515 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M05", day: 15, hour: 12, minute: 34, calendar });
const date75140614 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M06", day: 14, hour: 12, minute: 34, calendar });
const date75140615 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar });
const date75140616 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M06", day: 16, hour: 12, minute: 34, calendar });
const date75140621 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M06", day: 21, hour: 12, minute: 34, calendar });
const date75140715 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M07", day: 15, hour: 12, minute: 34, calendar });
const date75140716 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date75140717 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M07", day: 17, hour: 12, minute: 34, calendar });
const date75140721 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M07", day: 21, hour: 12, minute: 34, calendar });
const date75140723 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M07", day: 23, hour: 12, minute: 34, calendar });
const date75140813 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M08", day: 13, hour: 12, minute: 34, calendar });
const date75140816 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M08", day: 16, hour: 12, minute: 34, calendar });
const date75140817 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M08", day: 17, hour: 12, minute: 34, calendar });
const date75140916 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M09", day: 16, hour: 12, minute: 34, calendar });
const date75141228 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar });
const date75141230 = Temporal.PlainDateTime.from({ year: 7514, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date75150228 = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar });
const date75150716 = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date75150719 = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M07", day: 19, hour: 12, minute: 34, calendar });
const date75150919 = Temporal.PlainDateTime.from({ year: 7515, monthCode: "M09", day: 19, hour: 12, minute: 34, calendar });
const date75160716 = Temporal.PlainDateTime.from({ year: 7516, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date75240716 = Temporal.PlainDateTime.from({ year: 7524, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date75241216 = Temporal.PlainDateTime.from({ year: 7524, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });

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
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date75140716, date75140723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date75140716, date75140816, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date75131305, date75140105, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date75140105, date75140205, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date75140205, date75140306, "1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date75140205, date75140303, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date75140716, date75150716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -13, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date75140716, date75240716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -130, 0, 0],
    ["weeks", 0, 0, -521, -6],
    ["days", 0, 0, 0, -3653],
  ],
  [
    date75140716, date75150719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date75140716, date75150919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date75140716, date75241216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date74901216, date75140616, "23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date74900716, date75140716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date74900716, date75140614, "23 years, 11 months and 28 days",
    ["years", -23, -11, 0, -28],
  ],
  [
    date74900716, date75140615, "23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date74530216, date75130316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date75140330, date75140716, "3 months and 16 days",
    ["years", 0, -3, 0, -16],
  ],
  [
    date75130330, date75140716, "1 year, 3 months and 16 days",
    ["years", -1, -3, 0, -16],
  ],
  [
    date74891201, date74930518, "3 years, 6 months and 17 days",
    ["years", -3, -6, 0, -17],
  ],
  [
    date74530330, date75140716, "61 years, 3 months and 16 days",
    ["years", -61, -3, 0, -16],
  ],
  [
    date75121230, date75140716, "1 year, 7 months and 16 days",
    ["years", -1, -7, 0, -16],
  ],
  [
    date75131305, date75140621, "6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date74921201, date74930501, "6 months",
    ["months", 0, -6, 0, 0],
  ],
  [
    date74920101, date74921011, "40 weeks",
    ["weeks", 0, 0, -40, 0],
    ["days", 0, 0, 0, -280],
  ],
  [
    date74901305, date75140621, "23 years, 6 months and 16 days",
    ["years", -23, -6, 0, -16],
  ],
  [
    date75121305, date75140210, "1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ],
  [
    date75140717, date75140716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date75140723, date75140716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date75140816, date75140716, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date75140105, date75131305, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date75140205, date75140105, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date75140329, date75140228, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date75140307, date75140209, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date75140416, date75140216, "negative 2 months which both have 30 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 4],
    ["days", 0, 0, 0, 60],
  ],
  [
    date75150716, date75140716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 13, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date75240716, date75140716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 130, 0, 0],
    ["weeks", 0, 0, 521, 6],
    ["days", 0, 0, 0, 3653],
  ],
  [
    date75150719, date75140716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date75150919, date75140716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date75241216, date75140716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date75140716, date74901216, "negative 23 years and 8 months",
    ["years", 23, 8, 0, 0],
  ],
  [
    date75140716, date74900716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date75140615, date74890617, "negative 24 years, 12 months and 28 days",
    ["years", 24, 12, 0, 28],
  ],
  [
    date75140515, date74890516, "negative 24 years, 12 months and 29 days",
    ["years", 24, 12, 0, 29],
  ],
  [
    date75130316, date74530216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date75140716, date75140329, "negative 3 months and 17 days",
    ["years", 0, 3, 0, 17],
  ],
  [
    date75150716, date75140329, "negative 1 year, 3 months and 17 days",
    ["years", 1, 3, 0, 17],
  ],
  [
    date75150716, date74540329, "negative 61 years, 3 months and 17 days",
    ["years", 61, 3, 0, 17],
  ],
  [
    date75160716, date75141228, "negative 1 year, 7 months and 7 days",
    ["years", 1, 7, 0, 7],
  ],
  [
    date75150716, date75141230, "negative 7 months and 5 days",
    ["years", 0, 7, 0, 5],
  ],
  [
    date75140716, date74901228, "negative 23 years, 7 months and 7 days",
    ["years", 23, 7, 0, 7],
  ],
  [
    date75140305, date75121229, "negative 1 year, 3 months and 6 days",
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

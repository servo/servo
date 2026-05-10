// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (Persian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";

// Years

const date13751201 = Temporal.PlainDate.from({ year: 1375, monthCode: "M12", day: 1, calendar });
const date13780101 = Temporal.PlainDate.from({ year: 1378, monthCode: "M01", day: 1, calendar });
const date13781005 = Temporal.PlainDate.from({ year: 1378, monthCode: "M10", day: 5, calendar });
const date13781201 = Temporal.PlainDate.from({ year: 1378, monthCode: "M12", day: 1, calendar });
const date13790601 = Temporal.PlainDate.from({ year: 1379, monthCode: "M06", day: 1, calendar });
const date13790618 = Temporal.PlainDate.from({ year: 1379, monthCode: "M06", day: 18, calendar });
const date13810216 = Temporal.PlainDate.from({ year: 1381, monthCode: "M02", day: 16, calendar });
const date13420216 = Temporal.PlainDate.from({ year: 1342, monthCode: "M02", day: 16, calendar });
const date13430216 = Temporal.PlainDate.from({ year: 1343, monthCode: "M02", day: 16, calendar });
const date13430330 = Temporal.PlainDate.from({ year: 1343, monthCode: "M03", day: 30, calendar });
const date13520724 = Temporal.PlainDate.from({ year: 1352, monthCode: "M07", day: 24, calendar });
const date13781202 = Temporal.PlainDate.from({ year: 1378, monthCode: "M12", day: 2, calendar });
const date13800504 = Temporal.PlainDate.from({ year: 1380, monthCode: "M05", day: 4, calendar });
const date13800614 = Temporal.PlainDate.from({ year: 1380, monthCode: "M06", day: 14, calendar });
const date13800615 = Temporal.PlainDate.from({ year: 1380, monthCode: "M06", day: 15, calendar });
const date13800616 = Temporal.PlainDate.from({ year: 1380, monthCode: "M06", day: 16, calendar });
const date13800617 = Temporal.PlainDate.from({ year: 1380, monthCode: "M06", day: 17, calendar });
const date13800618 = Temporal.PlainDate.from({ year: 1380, monthCode: "M06", day: 18, calendar });
const date13800716 = Temporal.PlainDate.from({ year: 1380, monthCode: "M07", day: 16, calendar });
const date13801202 = Temporal.PlainDate.from({ year: 1380, monthCode: "M12", day: 2, calendar });
const date13801216 = Temporal.PlainDate.from({ year: 1380, monthCode: "M12", day: 16, calendar })
const date13801228 = Temporal.PlainDate.from({ year: 1380, monthCode: "M12", day: 28, calendar });
const date13801230 = Temporal.PlainDate.from({ year: 1380, monthCode: "M12", day: 30, calendar })
const date13810104 = Temporal.PlainDate.from({ year: 1381, monthCode: "M01", day: 4, calendar });;
const date13810504 = Temporal.PlainDate.from({ year: 1381, monthCode: "M05", day: 4, calendar });
const date14020101 = Temporal.PlainDate.from({ year: 1402, monthCode: "M01", day: 1, calendar });
const date14020201 = Temporal.PlainDate.from({ year: 1402, monthCode: "M02", day: 1, calendar });
const date14020316 = Temporal.PlainDate.from({ year: 1402, monthCode: "M03", day: 16, calendar });
const date14020724 = Temporal.PlainDate.from({ year: 1402, monthCode: "M07", day: 24, calendar });
const date14021228 = Temporal.PlainDate.from({ year: 1402, monthCode: "M12", day: 28, calendar });
const date14021229 = Temporal.PlainDate.from({ year: 1402, monthCode: "M12", day: 29, calendar });
const date14021230 = Temporal.PlainDate.from({ year: 1402, monthCode: "M12", day: 30, calendar })
const date14030101 = Temporal.PlainDate.from({ year: 1403, monthCode: "M01", day: 1, calendar });;
const date14030201 = Temporal.PlainDate.from({ year: 1403, monthCode: "M02", day: 1, calendar });
const date14030316 = Temporal.PlainDate.from({ year: 1403, monthCode: "M03", day: 16, calendar });
const date14030330 = Temporal.PlainDate.from({ year: 1403, monthCode: "M03", day: 30, calendar });
const date14031216 = Temporal.PlainDate.from({ year: 1403, monthCode: "M12", day: 16, calendar });
const date14031229 = Temporal.PlainDate.from({ year: 1403, monthCode: "M12", day: 29, calendar });
const date14031230 = Temporal.PlainDate.from({ year: 1403, monthCode: "M12", day: 30, calendar });
const date14040105 = Temporal.PlainDate.from({ year: 1404, monthCode: "M01", day: 5, calendar });
const date14040107 = Temporal.PlainDate.from({ year: 1404, monthCode: "M01", day: 7, calendar });
const date14040116 = Temporal.PlainDate.from({ year: 1404, monthCode: "M01", day: 16, calendar });
const date14040201 = Temporal.PlainDate.from({ year: 1404, monthCode: "M02", day: 1, calendar });
const date14040205 = Temporal.PlainDate.from({ year: 1404, monthCode: "M02", day: 5, calendar });
const date14040216 = Temporal.PlainDate.from({ year: 1404, monthCode: "M02", day: 16, calendar });
const date14040228 = Temporal.PlainDate.from({ year: 1404, monthCode: "M02", day: 28, calendar })
const date14040303 = Temporal.PlainDate.from({ year: 1404, monthCode: "M03", day: 3, calendar });;
const date14040304 = Temporal.PlainDate.from({ year: 1404, monthCode: "M03", day: 4, calendar });;
const date14040305 = Temporal.PlainDate.from({ year: 1404, monthCode: "M03", day: 5, calendar });
const date14040307 = Temporal.PlainDate.from({ year: 1404, monthCode: "M03", day: 7, calendar });
const date14040316 = Temporal.PlainDate.from({ year: 1404, monthCode: "M03", day: 16, calendar });
const date14040330 = Temporal.PlainDate.from({ year: 1404, monthCode: "M03", day: 30, calendar });
const date14040416 = Temporal.PlainDate.from({ year: 1404, monthCode: "M04", day: 16, calendar });
const date14040513 = Temporal.PlainDate.from({ year: 1404, monthCode: "M05", day: 13, calendar });
const date14040515 = Temporal.PlainDate.from({ year: 1404, monthCode: "M05", day: 15, calendar });
const date14040516 = Temporal.PlainDate.from({ year: 1404, monthCode: "M05", day: 16, calendar });
const date14040517 = Temporal.PlainDate.from({ year: 1404, monthCode: "M05", day: 17, calendar });
const date14040613 = Temporal.PlainDate.from({ year: 1404, monthCode: "M06", day: 13, calendar });
const date14040614 = Temporal.PlainDate.from({ year: 1404, monthCode: "M06", day: 14, calendar });
const date14040615 = Temporal.PlainDate.from({ year: 1404, monthCode: "M06", day: 15, calendar });
const date14040616 = Temporal.PlainDate.from({ year: 1404, monthCode: "M06", day: 16, calendar });
const date14040617 = Temporal.PlainDate.from({ year: 1404, monthCode: "M06", day: 17, calendar });
const date14040713 = Temporal.PlainDate.from({ year: 1404, monthCode: "M07", day: 13, calendar });
const date14040714 = Temporal.PlainDate.from({ year: 1404, monthCode: "M07", day: 14, calendar });
const date14040715 = Temporal.PlainDate.from({ year: 1404, monthCode: "M07", day: 15, calendar });
const date14040716 = Temporal.PlainDate.from({ year: 1404, monthCode: "M07", day: 16, calendar });
const date14040717 = Temporal.PlainDate.from({ year: 1404, monthCode: "M07", day: 17, calendar });
const date14040723 = Temporal.PlainDate.from({ year: 1404, monthCode: "M07", day: 23, calendar });
const date14040813 = Temporal.PlainDate.from({ year: 1404, monthCode: "M08", day: 13, calendar });
const date14040814 = Temporal.PlainDate.from({ year: 1404, monthCode: "M08", day: 14, calendar });
const date14040815 = Temporal.PlainDate.from({ year: 1404, monthCode: "M08", day: 15, calendar });
const date14040816 = Temporal.PlainDate.from({ year: 1404, monthCode: "M08", day: 16, calendar });
const date14040817 = Temporal.PlainDate.from({ year: 1404, monthCode: "M08", day: 17, calendar });
const date14040916 = Temporal.PlainDate.from({ year: 1404, monthCode: "M09", day: 16, calendar });
const date14050228 = Temporal.PlainDate.from({ year: 1405, monthCode: "M02", day: 28, calendar });
const date14050716 = Temporal.PlainDate.from({ year: 1405, monthCode: "M07", day: 16, calendar });
const date14050719 = Temporal.PlainDate.from({ year: 1405, monthCode: "M07", day: 19, calendar });
const date14050919 = Temporal.PlainDate.from({ year: 1405, monthCode: "M09", day: 19, calendar });
const date14140716 = Temporal.PlainDate.from({ year: 1414, monthCode: "M07", day: 16, calendar });
const date14141216 = Temporal.PlainDate.from({ year: 1414, monthCode: "M12", day: 16, calendar });

const tests = [
  [
    date14040716, date14040716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date14040716, date14040717, "one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date14040716, date14040723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date14040716, date14040816, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date14031216, date14040116, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date14040105, date14040205, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date14040716, date14040817, "1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date14040516, date14040617, "1 month and 1 day in a month with 31 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -32],
  ],
  [
    date14040716, date14040814, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date14040716, date14040916, "2 months which both have 30 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -4],
    ["days", 0, 0, 0, -60],
  ],
  [
    date14040416, date14040616, "2 months which both have 31 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -6],
    ["days", 0, 0, 0, -62],
  ],
  [
    date14040716, date14050716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date14030201, date14040201, "start of Ordibehesht",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date14040228, date14050228, "end of Ordibehesht",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date14020101, date14020201, "length of Farvardin 1402",
    ["days", 0, 0, 0, -31],
  ],
  [
    date14040716, date14140716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -120, 0, 0],
    ["weeks", 0, 0, -521, -5],
    ["days", 0, 0, 0, -3652],
  ],
  [
    date14040716, date14050719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date14040716, date14050919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date13751201, date13790618, "3 years, 6 months and 17 days",
    ["years", -3, -6, 0, -17],
  ],
  [
    date14040716, date14141216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date13801216, date14040716, "23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date13800716, date14040716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date13800716, date14040713, "23 years, 11 months and 28 days",
    ["years", -23, -11, 0, -28],
  ],
  [
    date13800616, date14040614, "23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date13430216, date14030316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date14040330, date14040716, "3 months and 17 days",
    ["years", 0, -3, 0, -17],
  ],
  [
    date14030330, date14040716, "1 year, 3 months and 17 days",
    ["years", -1, -3, 0, -17],
  ],
  [
    date13430330, date14040716, "61 years, 3 months and 17 days",
    ["years", -61, -3, 0, -17],
  ],
  [
    date14021230, date14040715, "1 year, 6 months and 17 days",
    ["years", -1, -6, 0, -17],
  ],
  [
    date14031230, date14040716, "6 months and 17 days",
    ["years", 0, -6, 0, -17],
  ],
  [
    date13781201, date13790601, "6 months",
    ["months", 0, -6, 0, 0],
  ],
  [
    date13780101, date13781005, "40 weeks",
    ["weeks", 0, 0, -40, 0],
    ["days", 0, 0, 0, -280],
  ],
  [
    date13801230, date14040715, "23 years, 6 months and 17 days",
    ["years", -23, -6, 0, -17],
  ],
  [
    date14021230, date14040303, "1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ],
  [
    date14040717, date14040716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date14040723, date14040716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date14040816, date14040716, "negative 1 month in same year (1)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date14040116, date14031216, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date14040205, date14040105, "negative 1 month in same year (2)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date14040817, date14040716, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date14040815, date14040717, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date14040916, date14040716, "negative 2 months which both have 30 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 4],
    ["days", 0, 0, 0, 60],
  ],
  [
    date14050716, date14040716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date14140716, date14040716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 120, 0, 0],
    ["weeks", 0, 0, 521, 5],
    ["days", 0, 0, 0, 3652],
  ],
  [
    date14050719, date14040716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date14050919, date14040716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date14141216, date14040716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date14040716, date13801216, "negative 23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date14040716, date13800716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date14040615, date13800617, "negative 23 years, 11 months and 29 days",
    ["years", 23, 11, 0, 29],
  ],
  [
    date14040615, date13800616, "negative 23 years, 11 months and 30 days",
    ["years", 23, 11, 0, 30],
  ],
  [
    date14020316, date13420216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date14040716, date14040330, "negative 3 months and 17 days",
    ["years", 0, 3, 0, 17],
  ],
  [
    date14040716, date14030330, "negative 1 year, 3 months and 17 days",
    ["years", 1, 3, 0, 17],
  ],
  [
    date14040716, date13430330, "negative 61 years, 3 months and 17 days",
    ["years", 61, 3, 0, 17],
  ],
  [
    date14040716, date14021228, "negative 1 year, 6 months and 17 days",
    ["years", 1, 6, 0, 17],
  ],
  [
    date14040716, date14031229, "negative 6 months and 17 days",
    ["years", 0, 6, 0, 17],
  ],
  [
    date14040716, date13801228, "negative 23 years, 6 months and 17 days",
    ["years", 23, 6, 0, 17],
  ],
  [
    date14040305, date14021230, "negative 1 year, 2 months and 5 days",
    ["years", 1, 2, 0, 5],
  ]
];

for (const [one, two, descr, ...units] of tests) {
  for (const [largestUnit, years, months, weeks, days] of units) {
    TemporalHelpers.assertDuration(
      one.since(two, { largestUnit }),
      years, months, weeks, days, 0, 0, 0, 0, 0, 0,
      descr
    );
  }
}

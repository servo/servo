// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (dangi calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "dangi";

// Years

const date19600216 = Temporal.PlainDateTime.from({ year: 1960, monthCode: "M02", day: 16, hour: 12, minute: 34, calendar });
const date19600330 = Temporal.PlainDateTime.from({ year: 1960, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date19610329 = Temporal.PlainDateTime.from({ year: 1961, monthCode: "M03", day: 29, hour: 12, minute: 34, calendar });
const date19690724 = Temporal.PlainDateTime.from({ year: 1969, monthCode: "M07", day: 24, hour: 12, minute: 34, calendar });
const date19960516 = Temporal.PlainDateTime.from({ year: 1996, monthCode: "M05", day: 16, hour: 12, minute: 34, calendar });
const date19960616 = Temporal.PlainDateTime.from({ year: 1996, monthCode: "M06", day: 16, hour: 12, minute: 34, calendar });
const date19960622 = Temporal.PlainDateTime.from({ year: 1996, monthCode: "M06", day: 22, hour: 12, minute: 34, calendar });
const date19960716 = Temporal.PlainDateTime.from({ year: 1996, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date19970616 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M06", day: 16, hour: 12, minute: 34, calendar });
const date19970716 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date19971216 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });
const date19971228 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar });
const date19971230 = Temporal.PlainDateTime.from({ year: 1997, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date20190101 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M01", day: 1, hour: 12, minute: 34, calendar });
const date20190201 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date20190724 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M07", day: 24, hour: 12, minute: 34, calendar });
const date20191229 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar });
const date20191230 = Temporal.PlainDateTime.from({ year: 2019, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date20200201 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date20200316 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M03", day: 16, hour: 12, minute: 34, calendar });
const date20200330 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date20201216 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });
const date20201229 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M12", day: 29, hour: 12, minute: 34, calendar });
const date20201230 = Temporal.PlainDateTime.from({ year: 2020, monthCode: "M12", day: 30, hour: 12, minute: 34, calendar });
const date20210105 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 5, hour: 12, minute: 34, calendar });
const date20210107 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 7, hour: 12, minute: 34, calendar });
const date20210116 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M01", day: 16, hour: 12, minute: 34, calendar });
const date20210201 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar });
const date20210205 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 5, hour: 12, minute: 34, calendar });
const date20210208 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 8, hour: 12, minute: 34, calendar });
const date20210209 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 9, hour: 12, minute: 34, calendar });
const date20210210 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 10, hour: 12, minute: 34, calendar });
const date20210216 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 16, hour: 12, minute: 34, calendar });
const date20210228 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar })
const date20210303 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 3, hour: 12, minute: 34, calendar });;
const date20210305 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 5, hour: 12, minute: 34, calendar });
const date20210306 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 6, hour: 12, minute: 34, calendar });
const date20210307 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 7, hour: 12, minute: 34, calendar });
const date20210329 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 29, hour: 12, minute: 34, calendar });
const date20210330 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M03", day: 30, hour: 12, minute: 34, calendar });
const date20210416 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M04", day: 16, hour: 12, minute: 34, calendar });
const date20210515 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M05", day: 15, hour: 12, minute: 34, calendar });
const date20210615 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar });
const date20210715 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 15, hour: 12, minute: 34, calendar });
const date20210716 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date20210717 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 17, hour: 12, minute: 34, calendar });
const date20210723 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M07", day: 23, hour: 12, minute: 34, calendar });
const date20210813 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M08", day: 13, hour: 12, minute: 34, calendar });
const date20210816 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M08", day: 16, hour: 12, minute: 34, calendar });
const date20210817 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M08", day: 17, hour: 12, minute: 34, calendar });
const date20210916 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M09", day: 16, hour: 12, minute: 34, calendar });
const date20211228 = Temporal.PlainDateTime.from({ year: 2021, monthCode: "M12", day: 28, hour: 12, minute: 34, calendar });
const date20220228 = Temporal.PlainDateTime.from({ year: 2022, monthCode: "M02", day: 28, hour: 12, minute: 34, calendar });
const date20220716 = Temporal.PlainDateTime.from({ year: 2022, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date20220719 = Temporal.PlainDateTime.from({ year: 2022, monthCode: "M07", day: 19, hour: 12, minute: 34, calendar });
const date20220919 = Temporal.PlainDateTime.from({ year: 2022, monthCode: "M09", day: 19, hour: 12, minute: 34, calendar });
const date20230716 = Temporal.PlainDateTime.from({ year: 2023, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date20310716 = Temporal.PlainDateTime.from({ year: 2031, monthCode: "M07", day: 16, hour: 12, minute: 34, calendar });
const date20311216 = Temporal.PlainDateTime.from({ year: 2031, monthCode: "M12", day: 16, hour: 12, minute: 34, calendar });

const tests = [
  [
    date20210716, date20210716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date20210716, date20210717, "one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date20210716, date20210723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date20210716, date20210816, "1 month in same year (30-day month to 29-day month)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date20201216, date20210116, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date20210105, date20210205, "1 month in same year (29-day month to 30-day month)",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date20210205, date20210306, "1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date20210205, date20210303, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date20210716, date20210916, "2 months which both have 30 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -3],
    ["days", 0, 0, 0, -59],
  ],
  [
    date20210716, date20220716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -50, -5],
    ["days", 0, 0, 0, -355],
  ],
  [
    date20190101, date20190201, "length of Il 2019",
    ["days", 0, 0, 0, -30],
  ],
  [
    date20210716, date20310716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -124, 0, 0],
    ["weeks", 0, 0, -523, -1],
    ["days", 0, 0, 0, -3662],
  ],
  [
    date20210716, date20220719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date20210716, date20220919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date20210716, date20311216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date19971216, date20210716, "23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date19970716, date20210716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date19970716, date20210715, "23 years, 11 months and 28 days",
    ["years", -23, -11, 0, -28],
  ],
  [
    date19970616, date20210615, "23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date19600216, date20200316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date20210330, date20210716, "3 months and 16 days",
    ["years", 0, -3, 0, -16],
  ],
  [
    date20200330, date20210716, "1 year, 3 months and 16 days",
    ["years", -1, -3, 0, -16],
  ],
  [
    date19600330, date20210716, "61 years, 3 months and 16 days",
    ["years", -61, -3, 0, -16],
  ],
  [
    date20191230, date20210716, "1 year, 6 months and 16 days",
    ["years", -1, -6, 0, -16],
  ],
  [
    date20201230, date20210716, "6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date19971230, date20210716, "23 years, 6 months and 16 days",
    ["years", -23, -6, 0, -16],
  ],
  [
    date20191230, date20210305, "1 year, 2 months and 5 days",
    ["years", -1, -2, 0, -5],
  ],
  [
    date20210717, date20210716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date20210723, date20210716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date20210816, date20210716, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date20210116, date20201216, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date20210205, date20210105, "negative 1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date20210329, date20210228, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date20210307, date20210209, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date20210416, date20210216, "negative 2 months which both have 30 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 4],
    ["days", 0, 0, 0, 60],
  ],
  [
    date20220716, date20210716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 50, 5],
    ["days", 0, 0, 0, 355],
  ],
  [
    date20310716, date20210716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 124, 0, 0],
    ["weeks", 0, 0, 523, 1],
    ["days", 0, 0, 0, 3662],
  ],
  [
    date20220719, date20210716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date20220919, date20210716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date20311216, date20210716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date20210716, date19971216, "negative 23 years and 8 months",
    ["years", 23, 8, 0, 0],
  ],
  [
    date20210716, date19970716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date20210615, date19960616, "negative 24 years, 11 months and 28 days",
    ["years", 24, 11, 0, 28],
  ],
  [
    date20210515, date19960516, "negative 24 years, 11 months and 29 days",
    ["years", 24, 11, 0, 29],
  ],
  [
    date20200316, date19600216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date20210716, date20210329, "negative 3 months and 17 days",
    ["years", 0, 3, 0, 17],
  ],
  [
    date20220716, date20210329, "negative 1 year, 3 months and 17 days",
    ["years", 1, 3, 0, 17],
  ],
  [
    date20220716, date19610329, "negative 61 years, 3 months and 17 days",
    ["years", 61, 3, 0, 17],
  ],
  [
    date20230716, date20211228, "negative 1 year, 6 months and 17 days",
    ["years", 1, 6, 0, 17],
  ],
  [
    date20210716, date20201229, "negative 6 months and 17 days",
    ["years", 0, 6, 0, 17],
  ],
  [
    date20210716, date19971228, "negative 23 years, 7 months and 17 days",
    ["years", 23, 7, 0, 17],
  ],
  [
    date20210305, date20191229, "negative 1 year, 2 months and 6 days",
    ["years", 1, 2, 0, 6],
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

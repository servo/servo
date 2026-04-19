// Copyright (C) 2021 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Years

const date00011225 = Temporal.PlainDate.from("0001-12-25");
const date19600216 = Temporal.PlainDate.from("1960-02-16");
const date19600330 = Temporal.PlainDate.from("1960-03-30");
const date19690724 = new Temporal.PlainDate(1969, 7, 24);
const date19970616 = Temporal.PlainDate.from("1997-06-16");
const date19970716 = Temporal.PlainDate.from("1997-07-16");
const date19971216 = Temporal.PlainDate.from("1997-12-16");
const date19971230 = Temporal.PlainDate.from("1997-12-30");
const date20110716 = Temporal.PlainDate.from("2011-07-16");
const date20190101 = Temporal.PlainDate.from("2019-01-01");
const date20190201 = Temporal.PlainDate.from("2019-02-01");
const date20190724 = Temporal.PlainDate.from({ year: 2019, month: 7, day: 24 });
const date20191230 = Temporal.PlainDate.from("2019-12-30");
const date20200201 = Temporal.PlainDate.from("2020-02-01");
const date20200315 = Temporal.PlainDate.from("2020-03-15");
const date20200316 = Temporal.PlainDate.from("2020-03-16");
const date20200330 = Temporal.PlainDate.from("2020-03-30");
const date20201216 = Temporal.PlainDate.from("2020-12-16");
const date20201230 = Temporal.PlainDate.from("2020-12-30");
const date20210105 = Temporal.PlainDate.from("2021-01-05");
const date20210107 = Temporal.PlainDate.from("2021-01-07");
const date20210116 = Temporal.PlainDate.from("2021-01-16");
const date20210201 = Temporal.PlainDate.from("2021-02-01");
const date20210205 = Temporal.PlainDate.from("2021-02-05");
const date20210228 = new Temporal.PlainDate(2021, 2, 28);
const date20210305 = Temporal.PlainDate.from("2021-03-05");
const date20210307 = Temporal.PlainDate.from("2021-03-07");
const date20210315 = Temporal.PlainDate.from("2021-03-15");
const date20210330 = Temporal.PlainDate.from("2021-03-30");
const date20210615 = Temporal.PlainDate.from("2021-06-15");
const date20210713 = Temporal.PlainDate.from("2021-07-13");
const date20210715 = Temporal.PlainDate.from("2021-07-15");
const date20210716 = Temporal.PlainDate.from("2021-07-16");
const date20210717 = Temporal.PlainDate.from("2021-07-17");
const date20210723 = Temporal.PlainDate.from("2021-07-23");
const date20210813 = Temporal.PlainDate.from("2021-08-13");
const date20210816 = Temporal.PlainDate.from("2021-08-16");
const date20210817 = Temporal.PlainDate.from("2021-08-17");
const date20210916 = Temporal.PlainDate.from("2021-09-16");
const date20210921 = Temporal.PlainDate.from("2021-09-21");
const date20211017 = Temporal.PlainDate.from("2021-10-17");
const date20220228 = new Temporal.PlainDate(2022, 2, 28);
const date20220716 = Temporal.PlainDate.from("2022-07-16");
const date20220719 = Temporal.PlainDate.from("2022-07-19");
const date20220919 = Temporal.PlainDate.from("2022-09-19");
const date20221017 = Temporal.PlainDate.from("2022-10-17");
const date20310716 = Temporal.PlainDate.from("2031-07-16");
const date20311216 = Temporal.PlainDate.from("2031-12-16");

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
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date20210716, date20210723, "7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date20210716, date20210817, "32 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["weeks", 0, 0, 4, 4],
    ["days", 0, 0, 0, 32],
  ],
  [
    date20210716, date20210916, "62 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 6],
    ["days", 0, 0, 0, 62],
  ],
  [
    date20210716, date20210813, "4 weeks",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
    ["days", 0, 0, 0, 28],
  ],
  [
    date20210716, date20210816, "1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 3],
  ],
  [
    date20210713, date20210816, "1 month and 3 days",
    ["years", 0, 1, 0, 3],
    ["months", 0, 1, 0, 3],
    ["weeks", 0, 0, 4, 6],
  ],
  [
    date20201216, date20210116, "1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date20210105, date20210205, "1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date20210107, date20210307, "2 months in same year",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
  ],
  [
    date20210716, date20210921, "2 months and 5 days",
    ["years", 0, 2, 0, 5],
    ["months", 0, 2, 0, 5],
  ],
  [
    date20210716, date20210817, "1 month and 1 day in a month with 31 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 32],
  ],
  [
    date20210716, date20210813, "28 days across a month which has 31 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date20210716, date20210916, "2 months which both have 31 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 6],
    ["days", 0, 0, 0, 62],
  ],
  [
    date20210716, date20220716, "1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date20210716, date20220719, "1 year, 3 days",
    ["years", 1, 0, 0, 3],
    ["months", 0, 12, 0, 3],
    ["weeks", 0, 0, 52, 4],
    ["days", 0, 0, 0, 368],
  ],
  [
    date20210716, date20220919, "1 year, 2 months, 3 days",
    ["years", 1, 2, 0, 3],
    ["months", 0, 14, 0, 3],
  ],
  [
    date20210716, date20221017, "1 year, 3 months, 1 day",
    ["years", 1, 3, 0, 1],
    ["months", 0, 15, 0, 1],
  ],
  [
    date20200201, date20210201, "start of February",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
  ],
  [
    date20210228, date20220228, "end of February",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
  ],
  [
    date20190101, date20190201, "length of January 2019",
    ["days", 0, 0, 0, 31],
  ],
  [
    date20210716, date20310716, "10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 120, 0, 0],
    ["weeks", 0, 0, 521, 5],
    ["days", 0, 0, 0, 3652],
  ],
  [
    date20210716, date20311216, "10 years and 5 months",
    ["years", 10, 5, 0, 0],
    ["months", 0, 125, 0, 0],
  ],
  [
    date20210716, date20220719, "1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date20210716, date20220919, "1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date20210716, date20311216, "10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date19971216, date20110716, "13 years and 7 months",
    ["years", 13, 7, 0, 0],
  ],
  [
    date19971216, date20210716, "23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date19970716, date20210716, "24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date19970716, date20210715, "23 years, 11 months and 29 days",
    ["years", 23, 11, 0, 29],
  ],
  [
    date19970616, date20210615, "23 years, 11 months and 30 days",
    ["years", 23, 11, 0, 30],
  ],
  [
    date19600216, date20200315, "60 years, 28 days",
    ["years", 60, 0, 0, 28],
  ],
  [
    date19600216, date20200316, "60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date20210330, date20210716, "3 months and 16 days",
    ["years", 0, 3, 0, 16],
  ],
  [
    date20200330, date20210716, "1 year, 3 months and 16 days",
    ["years", 1, 3, 0, 16],
  ],
  [
    date19600216, date20210315, "61 years, 27 days",
    ["years", 61, 0, 0, 27],
  ],
  [
    date19600330, date20210716, "61 years, 3 months and 16 days",
    ["years", 61, 3, 0, 16],
  ],
  [
    date20191230, date20210716, "1 year, 6 months and 16 days",
    ["years", 1, 6, 0, 16],
  ],
  [
    date20201230, date20210716, "6 months and 16 days",
    ["years", 0, 6, 0, 16],
  ],
  [
    date19971230, date20210716, "23 years, 6 months and 16 days",
    ["years", 23, 6, 0, 16],
  ],
  [
    date00011225, date20210716, "2019 years, 6 months and 21 days",
    ["years", 2019, 6, 0, 21],
  ],
  [
    date20191230, date20210305, "1 year, 2 months and 5 days",
    ["years", 1, 2, 0, 5],
  ],
  [
    date19690724, date20190724, "crossing epoch",
    ["years", 50, 0, 0, 0],
  ],
  [
    date20210717, date20210716, "negative one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date20210723, date20210716, "negative 7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date20210816, date20210716, "negative 1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -3],
  ],
  [
    date20210116, date20201216, "negative 1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date20210205, date20210105, "negative 1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date20210817, date20210716, "negative 1 month and 1 day in a month with 31 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -32],
  ],
  [
    date20210816, date20210713, "negative 1 month and 3 days",
    ["years", 0, -1, 0, -3],
    ["months", 0, -1, 0, -3],
    ["weeks", 0, 0, -4, -6],
  ],
  [
    date20210813, date20210716, "negative 28 days across a month which has 31 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date20210916, date20210716, "negative 2 months which both have 31 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -6],
    ["days", 0, 0, 0, -62],
  ],
  [
    date20210921, date20210716, "negative 2 months and 5 days",
    ["years", 0, -2, 0, -5],
    ["months", 0, -2, 0, -5],
  ],
  [
    date20210817, date20210716, "negative 32 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["weeks", 0, 0, -4, -4],
    ["days", 0, 0, 0, -32],
  ],
  [
    date20210916, date20210716, "negative 62 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -6],
    ["days", 0, 0, 0, -62],
  ],
  [
    date20220716, date20210716, "negative 1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date20310716, date20210716, "negative 10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -120, 0, 0],
    ["weeks", 0, 0, -521, -5],
    ["days", 0, 0, 0, -3652],
  ],
  [
    date20220719, date20210716, "negative 1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date20220919, date20210716, "negative 1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date20221017, date20210716, "negative 1 year, 3 months, and 1 day",
    ["years", -1, -3, 0, -1],
    ["months", 0, -15, 0, -1],
  ],
  [
    date20311216, date20210716, "negative 10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date20110716, date19971216, "negative 13 years and 7 months",
    ["years", -13, -7, 0, 0],
  ],
  [
    date20210716, date19971216, "negative 23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date20210716, date19970716, "negative 24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date20210715, date19970716, "negative 23 years, 11 months and 30 days",
    ["years", -23, -11, 0, -30],
  ],
  [
    date20210615, date19970616, "negative 23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date20200315, date19600216, "negative 60 years, 28 days",
    ["years", -60, 0, 0, -28],
  ],
  [
    date20200316, date19600216, "negative 60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date20210716, date20210330, "negative 3 months and 17 days",
    ["years", 0, -3, 0, -17],
  ],
  [
    date20210716, date20200330, "negative 1 year, 3 months and 17 days",
    ["years", -1, -3, 0, -17],
  ],
  [
    date20210716, date19600330, "negative 61 years, 3 months and 17 days",
    ["years", -61, -3, 0, -17],
  ],
  [
    date20210716, date20191230, "negative 1 year, 6 months and 17 days",
    ["years", -1, -6, 0, -17],
  ],
  [
    date20210716, date20201230, "negative 6 months and 17 days",
    ["years", 0, -6, 0, -17],
  ],
  [
    date20210716, date19971230, "negative 23 years, 6 months and 17 days",
    ["years", -23, -6, 0, -17],
  ],
  [
    date20210716, date00011225, "negative 2019 years, 6 months and 22 days",
    ["years", -2019, -6, 0, -22],
  ],
  [
    date20210305, date20191230, "negative 1 year, 2 months and 6 days",
    ["years", -1, -2, 0, -6],
  ],
  [
    date20190724, date19690724, "crossing epoch",
    ["years", -50, 0, 0, 0],
  ],
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

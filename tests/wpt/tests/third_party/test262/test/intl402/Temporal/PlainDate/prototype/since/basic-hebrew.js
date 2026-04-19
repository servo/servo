// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: >
  Check various basic calculations not involving leap years or constraining
  (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";

// Years

const date57220216 = Temporal.PlainDate.from({ year: 5722, monthCode: "M02", day: 16, calendar });
const date57220329 = Temporal.PlainDate.from({ year: 5722, monthCode: "M03", day: 29, calendar });
const date57230216 = Temporal.PlainDate.from({ year: 5723, monthCode: "M02", day: 16, calendar });
const date57230329 = Temporal.PlainDate.from({ year: 5723, monthCode: "M03", day: 29, calendar });
const date57320724 = Temporal.PlainDate.from({ year: 5732, monthCode: "M07", day: 24, calendar });
const date57590101 = Temporal.PlainDate.from({ year: 5759, monthCode: "M01", day: 1, calendar });
const date57591014 = Temporal.PlainDate.from({ year: 5759, monthCode: "M10", day: 14, calendar });
const date57591201 = Temporal.PlainDate.from({ year: 5759, monthCode: "M12", day: 1, calendar });
const date57591229 = Temporal.PlainDate.from({ year: 5759, monthCode: "M12", day: 29, calendar });
const date57600117 = Temporal.PlainDate.from({ year: 5760, monthCode: "M01", day: 17, calendar });
const date57600616 = Temporal.PlainDate.from({ year: 5760, monthCode: "M06", day: 16, calendar });
const date57600716 = Temporal.PlainDate.from({ year: 5760, monthCode: "M07", day: 16, calendar });
const date57601216 = Temporal.PlainDate.from({ year: 5760, monthCode: "M12", day: 16, calendar });
const date57601229 = Temporal.PlainDate.from({ year: 5760, monthCode: "M12", day: 29, calendar });
const date57610716 = Temporal.PlainDate.from({ year: 5761, monthCode: "M07", day: 16, calendar });
const date57611201 = Temporal.PlainDate.from({ year: 5761, monthCode: "M12", day: 1, calendar });
const date57620601 = Temporal.PlainDate.from({ year: 5762, monthCode: "M06", day: 1, calendar });
const date57620618 = Temporal.PlainDate.from({ year: 5762, monthCode: "M06", day: 18, calendar })
const date57810101 = Temporal.PlainDate.from({ year: 5781, monthCode: "M01", day: 1, calendar });
const date57810201 = Temporal.PlainDate.from({ year: 5781, monthCode: "M02", day: 1, calendar });
const date57810724 = Temporal.PlainDate.from({ year: 5781, monthCode: "M07", day: 24, calendar });
const date57811229 = Temporal.PlainDate.from({ year: 5781, monthCode: "M12", day: 29, calendar });
const date57820201 = Temporal.PlainDate.from({ year: 5782, monthCode: "M02", day: 1, calendar });
const date57820316 = Temporal.PlainDate.from({ year: 5782, monthCode: "M03", day: 16, calendar });
const date57820329 = Temporal.PlainDate.from({ year: 5782, monthCode: "M03", day: 29, calendar });
const date57820716 = Temporal.PlainDate.from({ year: 5782, monthCode: "M07", day: 16, calendar });
const date57821216 = Temporal.PlainDate.from({ year: 5782, monthCode: "M12", day: 16, calendar });
const date57821229 = Temporal.PlainDate.from({ year: 5782, monthCode: "M12", day: 29, calendar });
const date57830105 = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 5, calendar });
const date57830107 = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 7, calendar });
const date57830116 = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 16, calendar });
const date57830122 = Temporal.PlainDate.from({ year: 5783, monthCode: "M01", day: 22, calendar });
const date57830201 = Temporal.PlainDate.from({ year: 5783, monthCode: "M02", day: 1, calendar });
const date57830205 = Temporal.PlainDate.from({ year: 5783, monthCode: "M02", day: 5, calendar });
const date57830228 = Temporal.PlainDate.from({ year: 5783, monthCode: "M02", day: 28, calendar });
const date57830305 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 5, calendar });
const date57830307 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 7, calendar });
const date57830316 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 16, calendar });
const date57830329 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 29, calendar })
const date57830401 = Temporal.PlainDate.from({ year: 5783, monthCode: "M04", day: 1, calendar });
const date57830417 = Temporal.PlainDate.from({ year: 5783, monthCode: "M04", day: 17, calendar });
const date57830615 = Temporal.PlainDate.from({ year: 5783, monthCode: "M06", day: 15, calendar });
const date57830704 = Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 4, calendar });
const date57830715 = Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 15, calendar });
const date57830716 = Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 16, calendar });
const date57830717 = Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 17, calendar });
const date57830723 = Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 23, calendar });
const date57830812 = Temporal.PlainDate.from({ year: 5783, monthCode: "M08", day: 12, calendar });
const date57830813 = Temporal.PlainDate.from({ year: 5783, monthCode: "M08", day: 13, calendar });
const date57830814 = Temporal.PlainDate.from({ year: 5783, monthCode: "M08", day: 14, calendar });
const date57830816 = Temporal.PlainDate.from({ year: 5783, monthCode: "M08", day: 16, calendar });
const date57830817 = Temporal.PlainDate.from({ year: 5783, monthCode: "M08", day: 17, calendar });
const date57830916 = Temporal.PlainDate.from({ year: 5783, monthCode: "M09", day: 16, calendar });
const date57840201 = Temporal.PlainDate.from({ year: 5784, monthCode: "M02", day: 1, calendar });
const date57840216 = Temporal.PlainDate.from({ year: 5784, monthCode: "M02", day: 16, calendar });
const date57840228 = Temporal.PlainDate.from({ year: 5784, monthCode: "M02", day: 28, calendar });
const date57840615 = Temporal.PlainDate.from({ year: 5784, monthCode: "M07", day: 6, calendar });
const date57840715 = Temporal.PlainDate.from({ year: 5784, monthCode: "M07", day: 15, calendar });
const date57840716 = Temporal.PlainDate.from({ year: 5784, monthCode: "M07", day: 16, calendar });
const date57840719 = Temporal.PlainDate.from({ year: 5784, monthCode: "M07", day: 19, calendar });
const date57840919 = Temporal.PlainDate.from({ year: 5784, monthCode: "M09", day: 19, calendar });
const date57850714 = Temporal.PlainDate.from({ year: 5785, monthCode: "M07", day: 14, calendar });
const date57850715 = Temporal.PlainDate.from({ year: 5785, monthCode: "M07", day: 15, calendar });
const date57930716 = Temporal.PlainDate.from({ year: 5793, monthCode: "M07", day: 16, calendar });
const date57931216 = Temporal.PlainDate.from({ year: 5793, monthCode: "M12", day: 16, calendar });

const tests = [
  [
    date57830716, date57830716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date57830716, date57830717, "one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date57830716, date57830723, "7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date57830716, date57830816, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -2],
  ],
  [
    date57821216, date57830116, "1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date57830105, date57830205, "1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date57830316, date57830417, "1 month and 1 day in a month with 30 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -31],
  ],
  [
    date57830716, date57830814, "28 days across a month which has 30 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date57830716, date57830916, "2 months which both have 30 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -3],
    ["days", 0, 0, 0, -59],
  ],
  [
    date57820716, date57830716, "1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -50, -5],
    ["days", 0, 0, 0, -355],
  ],
  [
    date57830201, date57840201, "start of Cheshvan",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date57830228, date57840228, "end of Cheshvan",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
  ],
  [
    date57810101, date57810201, "length of Tishrei 5781",
    ["days", 0, 0, 0, -30],
  ],
  [
    date57830716, date57930716, "10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -124, 0, 0],
    ["weeks", 0, 0, -523, 0],
    ["days", 0, 0, 0, -3661],
  ],
  [
    date57830716, date57840719, "1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date57830716, date57840919, "1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date57591201, date57620618, "2 years 6 months and 17 days",
    ["years", -2, -6, 0, -17],
  ],
  [
    date57830716, date57931216, "10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date57601216, date57840716, "23 years and 8 months",
    ["years", -23, -8, 0, 0],
  ],
  [
    date57600716, date57840716, "24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date57610716, date57850715, "23 years, 11 months and 28 days",
    ["years", -23, -11, 0, -28],
  ],
  [
    date57230216, date57830316, "60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date57830329, date57830716, "3 months and 16 days",
    ["years", 0, -3, 0, -16],
  ],
  [
    date57820329, date57830716, "1 year, 3 months and 16 days",
    ["years", -1, -3, 0, -16],
  ],
  [
    date57230329, date57840716, "61 years, 4 months and 16 days",
    ["years", -61, -4, 0, -16],
  ],
  [
    date57811229, date57830716, "1 year, 6 months and 16 days",
    ["years", -1, -6, 0, -16],
  ],
  [
    date57821229, date57830716, "6 months and 16 days",
    ["years", 0, -6, 0, -16],
  ],
  [
    date57611201, date57620601, "6 months",
    ["months", 0, -6, 0, 0],
  ],
  [
    date57590101, date57591014, "40 weeks",
    ["weeks", 0, 0, -40, 0],
    ["days", 0, 0, 0, -280],
  ],
  [
    date57600117, date57830704, "23 years, 5 months and 16 days",
    ["years", -23, -5, 0, -16],
  ],
  [
    date57811229, date57830305, "1 year, 2 months and 6 days",
    ["years", -1, -2, 0, -6],
  ],
  [
    date57830717, date57830716, "negative one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date57830723, date57830716, "negative 7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date57830816, date57830716, "negative 1 month in same year (1)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 2],
  ],
  [
    date57830116, date57821216, "negative 1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date57830205, date57830105, "negative 1 month in same year (2)",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date57830817, date57830716, "negative 1 month and 1 day in a month with 30 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 31],
  ],
  [
    date57830813, date57830715, "negative 28 days across a month which has 30 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date57830916, date57830716, "negative 2 months which both have 30 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 3],
    ["days", 0, 0, 0, 59],
  ],
  [
    date57830716, date57820716, "negative 1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 50, 5],
    ["days", 0, 0, 0, 355],
  ],
  [
    date57930716, date57830716, "negative 10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 124, 0, 0],
    ["weeks", 0, 0, 523, 0],
    ["days", 0, 0, 0, 3661],
  ],
  [
    date57840719, date57830716, "negative 1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date57840919, date57830716, "negative 1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date57931216, date57830716, "negative 10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date57840716, date57601216, "negative 23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date57840716, date57600716, "negative 24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date57840615, date57600616, "negative 24 years and 19 days",
    ["years", 24, 0, 0, 19],
  ],
  [
    date57820316, date57220216, "negative 60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date57830716, date57830329, "negative 3 months and 17 days",
    ["years", 0, 3, 0, 17],
  ],
  [
    date57830716, date57820329, "negative 1 year, 4 months and 17 days",
    ["years", 1, 4, 0, 17],
  ],
  [
    date57830716, date57220329, "negative 61 years, 4 months and 16 days",
    ["years", 61, 4, 0, 16],
  ],
  [
    date57830716, date57811229, "negative 1 year, 7 months and 16 days",
    ["years", 1, 7, 0, 16],
  ],
  [
    date57830716, date57821229, "negative 6 months and 16 days",
    ["years", 0, 6, 0, 16],
  ],
  [
    date57830716, date57591229, "negative 23 years, 7 months and 16 days",
    ["years", 23, 7, 0, 16],
  ],
  [
    date57830305, date57811229, "negative 1 year, 2 months and 5 days",
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

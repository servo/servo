// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
  Check various basic calculations not involving leap years or constraining
  (roc calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "roc";

// Years

const date0490216 = Temporal.ZonedDateTime.from({ year: 49, monthCode: "M02", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0490330 = Temporal.ZonedDateTime.from({ year: 49, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0580724 = Temporal.ZonedDateTime.from({ year: 58, monthCode: "M07", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0860616 = Temporal.ZonedDateTime.from({ year: 86, monthCode: "M06", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0860716 = Temporal.ZonedDateTime.from({ year: 86, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0861201 = Temporal.ZonedDateTime.from({ year: 86, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0861216 = Temporal.ZonedDateTime.from({ year: 86, monthCode: "M12", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0861230 = Temporal.ZonedDateTime.from({ year: 86, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0900101 = Temporal.ZonedDateTime.from({ year: 90, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0900618 = Temporal.ZonedDateTime.from({ year: 90, monthCode: "M06", day: 18, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0901008 = Temporal.ZonedDateTime.from({ year: 90, monthCode: "M10", day: 8, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0901201 = Temporal.ZonedDateTime.from({ year: 90, monthCode: "M12", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date0910601 = Temporal.ZonedDateTime.from({ year: 91, monthCode: "M06", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1080101 = Temporal.ZonedDateTime.from({ year: 108, monthCode: "M01", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1080201 = Temporal.ZonedDateTime.from({ year: 108, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1080724 = Temporal.ZonedDateTime.from({ year: 108, monthCode: "M07", day: 24, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1081230 = Temporal.ZonedDateTime.from({ year: 108, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1090201 = Temporal.ZonedDateTime.from({ year: 109, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1090316 = Temporal.ZonedDateTime.from({ year: 109, monthCode: "M03", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1090330 = Temporal.ZonedDateTime.from({ year: 109, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1091216 = Temporal.ZonedDateTime.from({ year: 109, monthCode: "M12", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1091230 = Temporal.ZonedDateTime.from({ year: 109, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100105 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M01", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100107 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M01", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100116 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M01", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100201 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M02", day: 1, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100205 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M02", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100228 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100305 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M03", day: 5, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100307 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M03", day: 7, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100330 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M03", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100615 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M06", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100715 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M07", day: 15, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100716 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100717 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M07", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100723 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M07", day: 23, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100813 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M08", day: 13, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100816 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M08", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100817 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M08", day: 17, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1100916 = Temporal.ZonedDateTime.from({ year: 110, monthCode: "M09", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110228 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110716 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110719 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M07", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1110919 = Temporal.ZonedDateTime.from({ year: 111, monthCode: "M09", day: 19, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1200716 = Temporal.ZonedDateTime.from({ year: 120, monthCode: "M07", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });
const date1201216 = Temporal.ZonedDateTime.from({ year: 120, monthCode: "M12", day: 16, hour: 12, minute: 34, timeZone: "UTC", calendar });

const tests = [
  [
    date1100716, date1100716, "same day",
    ["years", 0, 0, 0, 0],
    ["months", 0, 0, 0, 0],
    ["weeks", 0, 0, 0, 0],
    ["days", 0, 0, 0, 0],
  ],
  [
    date1100716, date1100717, "one day",
    ["years", 0, 0, 0, 1],
    ["months", 0, 0, 0, 1],
    ["weeks", 0, 0, 0, 1],
    ["days", 0, 0, 0, 1],
  ],
  [
    date1100716, date1100723, "7 days",
    ["years", 0, 0, 0, 7],
    ["months", 0, 0, 0, 7],
    ["weeks", 0, 0, 1, 0],
  ],
  [
    date1100716, date1100816, "1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
    ["weeks", 0, 0, 4, 3],
  ],
  [
    date1091216, date1100116, "1 month in different year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date1100105, date1100205, "1 month in same year",
    ["years", 0, 1, 0, 0],
    ["months", 0, 1, 0, 0],
  ],
  [
    date1100716, date1100817, "1 month and 1 day in a month with 31 days",
    ["years", 0, 1, 0, 1],
    ["months", 0, 1, 0, 1],
    ["days", 0, 0, 0, 32],
  ],
  [
    date1100716, date1100813, "28 days across a month which has 31 days",
    ["years", 0, 0, 0, 28],
    ["months", 0, 0, 0, 28],
    ["weeks", 0, 0, 4, 0],
  ],
  [
    date1100716, date1100916, "2 months which both have 31 days",
    ["years", 0, 2, 0, 0],
    ["months", 0, 2, 0, 0],
    ["weeks", 0, 0, 8, 6],
    ["days", 0, 0, 0, 62],
  ],
  [
    date1100716, date1110716, "1 year",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
    ["weeks", 0, 0, 52, 1],
    ["days", 0, 0, 0, 365],
  ],
  [
    date1090201, date1100201, "start of February",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
  ],
  [
    date1100228, date1110228, "end of February",
    ["years", 1, 0, 0, 0],
    ["months", 0, 12, 0, 0],
  ],
  [
    date1080101, date1080201, "length of January 108",
    ["days", 0, 0, 0, 31],
  ],
  [
    date1100716, date1200716, "10 years",
    ["years", 10, 0, 0, 0],
    ["months", 0, 120, 0, 0],
    ["weeks", 0, 0, 521, 5],
    ["days", 0, 0, 0, 3652],
  ],
  [
    date1100716, date1110719, "1 year and 3 days",
    ["years", 1, 0, 0, 3],
  ],
  [
    date1100716, date1110919, "1 year 2 months and 3 days",
    ["years", 1, 2, 0, 3],
  ],
  [
    date1100716, date1201216, "10 years and 5 months",
    ["years", 10, 5, 0, 0],
  ],
  [
    date0861216, date1100716, "23 years and 7 months",
    ["years", 23, 7, 0, 0],
  ],
  [
    date0860716, date1100716, "24 years",
    ["years", 24, 0, 0, 0],
  ],
  [
    date0860716, date1100715, "23 years, 11 months and 29 days",
    ["years", 23, 11, 0, 29],
  ],
  [
    date0860616, date1100615, "23 years, 11 months and 30 days",
    ["years", 23, 11, 0, 30],
  ],
  [
    date0490216, date1090316, "60 years, 1 month",
    ["years", 60, 1, 0, 0],
  ],
  [
    date1100330, date1100716, "3 months and 16 days",
    ["years", 0, 3, 0, 16],
  ],
  [
    date1090330, date1100716, "1 year, 3 months and 16 days",
    ["years", 1, 3, 0, 16],
  ],
  [
    date0490330, date1100716, "61 years, 3 months and 16 days",
    ["years", 61, 3, 0, 16],
  ],
  [
    date1081230, date1100716, "1 year, 6 months and 16 days",
    ["years", 1, 6, 0, 16],
  ],
  [
    date0861201, date0900618, "3 years, 6 months and 17 days",
    ["years", 3, 6, 0, 17],
  ],
  [
    date1091230, date1100716, "6 months and 16 days",
    ["years", 0, 6, 0, 16],
  ],
  [
    date0901201, date0910601, "6 months",
    ["months", 0, 6, 0, 0],
  ],
  [
    date0900101, date0901008, "40 weeks",
    ["weeks", 0, 0, 40, 0],
    ["days", 0, 0, 0, 280],
  ],
  [
    date0861230, date1100716, "23 years, 6 months and 16 days",
    ["years", 23, 6, 0, 16],
  ],
  [
    date1081230, date1100305, "1 year, 2 months and 5 days",
    ["years", 1, 2, 0, 5],
  ],
  [
    date0580724, date1080724, "crossing epoch",
    ["years", 50, 0, 0, 0],
  ],
  [
    date1100717, date1100716, "negative one day",
    ["years", 0, 0, 0, -1],
    ["months", 0, 0, 0, -1],
    ["weeks", 0, 0, 0, -1],
    ["days", 0, 0, 0, -1],
  ],
  [
    date1100723, date1100716, "negative 7 days",
    ["years", 0, 0, 0, -7],
    ["months", 0, 0, 0, -7],
    ["weeks", 0, 0, -1, 0],
  ],
  [
    date1100816, date1100716, "negative 1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
    ["weeks", 0, 0, -4, -3],
  ],
  [
    date1100116, date1091216, "negative 1 month in different year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date1100205, date1100105, "negative 1 month in same year",
    ["years", 0, -1, 0, 0],
    ["months", 0, -1, 0, 0],
  ],
  [
    date1100817, date1100716, "negative 1 month and 1 day in a month with 31 days",
    ["years", 0, -1, 0, -1],
    ["months", 0, -1, 0, -1],
    ["days", 0, 0, 0, -32],
  ],
  [
    date1100813, date1100716, "negative 28 days across a month which has 31 days",
    ["years", 0, 0, 0, -28],
    ["months", 0, 0, 0, -28],
    ["weeks", 0, 0, -4, 0],
  ],
  [
    date1100916, date1100716, "negative 2 months which both have 31 days",
    ["years", 0, -2, 0, 0],
    ["months", 0, -2, 0, 0],
    ["weeks", 0, 0, -8, -6],
    ["days", 0, 0, 0, -62],
  ],
  [
    date1110716, date1100716, "negative 1 year",
    ["years", -1, 0, 0, 0],
    ["months", 0, -12, 0, 0],
    ["weeks", 0, 0, -52, -1],
    ["days", 0, 0, 0, -365],
  ],
  [
    date1200716, date1100716, "negative 10 years",
    ["years", -10, 0, 0, 0],
    ["months", 0, -120, 0, 0],
    ["weeks", 0, 0, -521, -5],
    ["days", 0, 0, 0, -3652],
  ],
  [
    date1110719, date1100716, "negative 1 year and 3 days",
    ["years", -1, 0, 0, -3],
  ],
  [
    date1110919, date1100716, "negative 1 year 2 months and 3 days",
    ["years", -1, -2, 0, -3],
  ],
  [
    date1201216, date1100716, "negative 10 years and 5 months",
    ["years", -10, -5, 0, 0],
  ],
  [
    date1100716, date0861216, "negative 23 years and 7 months",
    ["years", -23, -7, 0, 0],
  ],
  [
    date1100716, date0860716, "negative 24 years",
    ["years", -24, 0, 0, 0],
  ],
  [
    date1100715, date0860716, "negative 23 years, 11 months and 30 days",
    ["years", -23, -11, 0, -30],
  ],
  [
    date1100615, date0860616, "negative 23 years, 11 months and 29 days",
    ["years", -23, -11, 0, -29],
  ],
  [
    date1090316, date0490216, "negative 60 years, 1 month",
    ["years", -60, -1, 0, 0],
  ],
  [
    date1100716, date1100330, "negative 3 months and 17 days",
    ["years", 0, -3, 0, -17],
  ],
  [
    date1100716, date1090330, "negative 1 year, 3 months and 17 days",
    ["years", -1, -3, 0, -17],
  ],
  [
    date1100716, date0490330, "negative 61 years, 3 months and 17 days",
    ["years", -61, -3, 0, -17],
  ],
  [
    date1100716, date1081230, "negative 1 year, 6 months and 17 days",
    ["years", -1, -6, 0, -17],
  ],
  [
    date1100716, date1091230, "negative 6 months and 17 days",
    ["years", 0, -6, 0, -17],
  ],
  [
    date1100716, date0861230, "negative 23 years, 6 months and 17 days",
    ["years", -23, -6, 0, -17],
  ],
  [
    date1100305, date1081230, "negative 1 year, 2 months and 6 days",
    ["years", -1, -2, 0, -6],
  ],
  [
    date1080724, date0580724, "crossing epoch",
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

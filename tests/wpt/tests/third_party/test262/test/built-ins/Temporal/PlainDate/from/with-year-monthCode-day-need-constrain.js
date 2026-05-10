// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plaindate.from
description: With year, monthCode and day and need constrain
info: |
  1. Let calendar be the this value.
  2. Perform ? RequireInternalSlot(calendar, [[InitializedTemporalCalendar]]).
  3. Assert: calendar.[[Identifier]] is "iso8601".
  4. If Type(fields) is not Object, throw a TypeError exception.
  5. Set options to ? GetOptionsObject(options).
  6. Let result be ? ISODateFromFields(fields, options).
  7. Return ? CreateTemporalDate(result.[[Year]], result.[[Month]], result.[[Day]], calendar).
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const year = 2021;

const testData = [
  ["M01", 133, "Jan", 1, "M01", 31],
  ["M02", 133, "Feb", 2, "M02", 28],
  ["M03", 133, "March", 3, "M03", 31],
  ["M04", 133, "April", 4, "M04", 30],
  ["M05", 133, "May", 5, "M05", 31],
  ["M06", 133, "Jun", 6, "M06", 30],
  ["M07", 133, "Jul", 7, "M07", 31],
  ["M08", 133, "Aug", 8, "M08", 31],
  ["M09", 133, "Sept", 9, "M09", 30],
  ["M10", 133, "Oct", 10, "M10", 31],
  ["M11", 133, "Nov", 11, "M11", 30],
  ["M12", 133, "Dec", 12, "M12", 31],
  ["M03", 9033, "out-of-range day 9033", 3, "M03", 31],
  ["M04", 50, "out-of-range day 50", 4, "M04", 30],
  ["M05", 77, "out-of-range day 77", 5, "M05", 31],
  ["M06", 33, "out-of-range date 06-33", 6, "M06", 30],
  ["M07", 33, "out-of-range day 07-33", 7, "M07", 31],
  ["M08", 300, "out-of-range day 300", 8, "M08", 31],
  ["M09", 400, "out-of-range date 09-400", 9, "M09", 30],
  ["M10", 400, "out-of-range date 10-400", 10, "M10", 31],
  ["M11", 400, "out-of-range date 11-400", 11, "M11", 30],
  ["M12", 500, "out-of-range day 500", 12, "M12", 31],
];

for (const [monthCode, day, descr, expectedMonth, expectedMonthCode, expectedDay] of testData) {
  TemporalHelpers.assertPlainDate(
    Temporal.PlainDate.from({year, monthCode, day}),
    year, expectedMonth, expectedMonthCode, expectedDay,
    `year/month code/day need to be constrained in ${descr}`);
}

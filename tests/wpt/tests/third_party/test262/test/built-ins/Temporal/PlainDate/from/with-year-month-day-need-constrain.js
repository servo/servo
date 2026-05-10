// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plaindate.from
description: Property bag with year/month/day and need constrain
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
  [1, 133, "Jan", 1, "M01", 31],
  [2, 133, "Feb", 2, "M02", 28],
  [3, 133, "March", 3, "M03", 31],
  [4, 133, "April", 4, "M04", 30],
  [5, 133, "May", 5, "M05", 31],
  [6, 133, "Jun", 6, "M06", 30],
  [7, 133, "Jul", 7, "M07", 31],
  [8, 133, "Aug", 8, "M08", 31],
  [9, 133, "Sept", 9, "M09", 30],
  [10, 133, "Oct", 10, "M10", 31],
  [11, 133, "Nov", 11, "M11", 30],
  [12, 133, "Dec", 12, "M12", 31],
  [13, 500, "out-of-range month 13", 12, "M12", 31],
  [999999, 500, "out-of-range month 999999", 12, "M12", 31],
  [3, 9033, "out-of-range day 9033", 3, "M03", 31],
  [4, 50, "out-of-range day 50", 4, "M04", 30],
  [5, 77, "out-of-range day 77", 5, "M05", 31],
  [6, 33, "out-of-range date 06-33", 6, "M06", 30],
  [7, 33, "out-of-range day 07-33", 7, "M07", 31],
  [8, 300, "out-of-range day 300", 8, "M08", 31],
  [9, 400, "out-of-range date 09-400", 9, "M09", 30],
  [10, 400, "out-of-range date 10-400", 10, "M10", 31],
  [11, 400, "out-of-range date 11-400", 11, "M11", 30],
  [12, 500, "out-of-range day 500", 12, "M12", 31],
];

for (const [month, day, descr, expectedMonth, expectedMonthCode, expectedDay] of testData) {
  TemporalHelpers.assertPlainDate(
    Temporal.PlainDate.from({year, month, day}),
    year, expectedMonth, expectedMonthCode, expectedDay,
    `year/month/day need to be constrained in ${descr}`);
}

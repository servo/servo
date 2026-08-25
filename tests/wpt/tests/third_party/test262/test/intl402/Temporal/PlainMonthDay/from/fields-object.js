// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Basic tests for PlainMonthDay.from(object) with non-ISO calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const okTests = [
  [{ monthCode: "M08", day: 1, calendar: "gregory" }, "gregory", "monthCode and non-ISO Gregorian string calendar"],
  [{ monthCode: "M08", day: 1, calendar: "hebrew" }, "hebrew", "monthCode and non-ISO non-Gregorian string calendar"],
];

for (const [argument, expectedCalendar, description] of okTests) {
  const plainMonthDay = Temporal.PlainMonthDay.from(argument);
  TemporalHelpers.assertPlainMonthDay(plainMonthDay, "M08", 1, description);
  assert.sameValue(plainMonthDay.calendarId, expectedCalendar, `resulting calendar is ${expectedCalendar}`);
}

const notOkTests = [
  [{ month: 8, day: 1, calendar: "gregory" }, "month and non-ISO string calendar"],
  [{ month: 8, day: 1, calendar: "hebrew" }, "month and non-ISO non-Gregorian string calendar"],
];

for (const [argument, description] of notOkTests) {
  assert.throws(TypeError, () => Temporal.PlainMonthDay.from(argument), description);
}

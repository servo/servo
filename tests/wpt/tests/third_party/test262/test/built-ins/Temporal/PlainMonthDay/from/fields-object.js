// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Basic tests for PlainMonthDay.from(object).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const tests = [
  [{ monthCode: "M10", day: 1 }, "option bag with monthCode"],
  [{ monthCode: "M10", day: 1, year: 2015 }, "option bag with year, monthCode"],
  [{ month: 10, day: 1 }, "option bag with year, month"],
  [{ month: 10, day: 1, year: 2015 }, "option bag with year, month"],
  [{ month: 10, day: 1, days: 31 }, "option bag with plural 'days'"],
  [new Temporal.PlainMonthDay(10, 1), "PlainMonthDay object"],
  [Temporal.PlainDate.from("2019-10-01"), "PlainDate object"],
  [{ monthCode: "M10", day: 1, calendar: "iso8601" }, "option bag with monthCode and explicit ISO calendar"],
  [{ month: 10, day: 1, calendar: "iso8601" }, "option bag with month and explicit ISO calendar"],
];

for (const [argument, description = argument] of tests) {
  const plainMonthDay = Temporal.PlainMonthDay.from(argument);
  assert.notSameValue(plainMonthDay, argument, `from ${description} converts`);
  TemporalHelpers.assertPlainMonthDay(plainMonthDay, "M10", 1, `from ${description}`);
  assert.sameValue(plainMonthDay.calendarId, "iso8601", `from ${description} calendar`);
}

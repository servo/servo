// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Basic tests for PlainMonthDay.from(string).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

for (const argument of TemporalHelpers.ISO.plainMonthDayStringsValid()) {
  const plainMonthDay = Temporal.PlainMonthDay.from(argument);
  assert.notSameValue(plainMonthDay, argument, `from ${argument} converts`);
  TemporalHelpers.assertPlainMonthDay(plainMonthDay, "M10", 1, `from ${argument}`);
  assert.sameValue(plainMonthDay.calendarId, "iso8601", `from ${argument} calendar`);
}

for (const arg of TemporalHelpers.ISO.plainMonthDayStringsInvalid()) {
  assert.throws(RangeError, () => Temporal.PlainMonthDay.from(arg), `"${arg}" not a valid PlainMonthDay string`);
}

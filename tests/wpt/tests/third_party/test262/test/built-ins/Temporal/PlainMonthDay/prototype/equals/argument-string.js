// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: A string argument is parsed into a PlainMonthDay
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(10, 1);
for (const arg of TemporalHelpers.ISO.plainMonthDayStringsValid()) {
  const result = instance.equals(arg);
  assert.sameValue(result, true, `"${arg}" is a valid PlainMonthDay string`);
}

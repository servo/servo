// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.equals
description: An invalid ISO string is never supported
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(11, 18);

for (const arg of TemporalHelpers.ISO.plainMonthDayStringsInvalid()) {
  assert.throws(RangeError, () => instance.equals(arg), `"${arg}" is not a valid PlainMonthDay string`);
}

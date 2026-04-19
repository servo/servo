// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: An invalid ISO string is never supported
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(1976, 11);

for (const arg of TemporalHelpers.ISO.plainYearMonthStringsInvalid()) {
  assert.throws(RangeError, () => instance.equals(arg), `"${arg}" is not a valid PlainYearMonth string`);
}

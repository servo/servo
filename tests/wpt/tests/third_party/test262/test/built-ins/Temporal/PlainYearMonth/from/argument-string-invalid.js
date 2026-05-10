// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: An invalid ISO string is never supported
includes: [temporalHelpers.js]
features: [Temporal]
---*/

for (const input of TemporalHelpers.ISO.plainYearMonthStringsInvalid()) {
  assert.throws(RangeError, () => Temporal.PlainYearMonth.from(input, { overflow: "reject" }));
  assert.throws(RangeError, () => Temporal.PlainYearMonth.from(input, { overflow: "constrain" }));
}

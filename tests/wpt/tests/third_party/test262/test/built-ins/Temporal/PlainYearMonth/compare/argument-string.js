// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: A string is parsed into the correct object when passed as the argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const validStrings = TemporalHelpers.ISO.plainYearMonthStringsValid().concat(TemporalHelpers.ISO.plainYearMonthStringsValidNegativeYear());

for (const arg of validStrings) {
  assert.sameValue(
    Temporal.PlainYearMonth.compare(arg, arg),
    0,
    `"${arg}" is a valid PlainYearMonth string`
  );
}

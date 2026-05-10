// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: A string is parsed into the correct object when passed as the argument
includes: [temporalHelpers.js]
features: [Temporal]
---*/

for (const input of TemporalHelpers.ISO.plainYearMonthStringsValid()) {
  const plainYearMonth = Temporal.PlainYearMonth.from(input);
  TemporalHelpers.assertPlainYearMonth(plainYearMonth, 1976, 11, "M11");
  assert.sameValue(plainYearMonth.calendarId, "iso8601", "calendar string should be iso8601");
  assert.sameValue(plainYearMonth.toString({ calendarName: "always" }), "1976-11-01[u-ca=iso8601]", "iso reference date");
}

for (const input of TemporalHelpers.ISO.plainYearMonthStringsValidNegativeYear()) {
  const plainYearMonth = Temporal.PlainYearMonth.from(input);
  TemporalHelpers.assertPlainYearMonth(plainYearMonth, -9999, 11, "M11");
  assert.sameValue(plainYearMonth.calendarId, "iso8601", "calendar string should be iso8601");
  assert.sameValue(plainYearMonth.toString({ calendarName: "always" }), "-009999-11-01[u-ca=iso8601]", "iso reference date");
}

// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: An ISO 8601 string can be converted to a calendar ID in Calendar
features: [Temporal]
---*/

for (const calendar of [
  "2020-01-01",
  "2020-01-01[u-ca=iso8601]",
  "2020-01-01T00:00:00.000000000",
  "2020-01-01T00:00:00.000000000[u-ca=iso8601]",
  "01-01",
  "01-01[u-ca=iso8601]",
  "2020-01",
  "2020-01[u-ca=iso8601]",
]) {
  const arg = { year: 2019, monthCode: "M06", calendar };
  const result1 = Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6));
  assert.sameValue(result1, 0, `Calendar created from string "${calendar}" (first argument)`);
  const result2 = Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg);
  assert.sameValue(result2, 0, `Calendar created from string "${calendar}" (second argument)`);
}

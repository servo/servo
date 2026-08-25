// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: Each date and time component can be formatted individually
locale: [en-u-ca-iso8601]
features: [Temporal]
---*/

const epochMs = 1735213600_321;  // 2024-12-26T11:46:40.321Z
const legacyDate = new Date(epochMs);
const plainYearMonth = new Temporal.PlainYearMonth(2024, 12);

for (const options of [
  { "year": "numeric" },
  { "month": "long" },
  // "era" is skipped; it implies year, month, and day in "en" locale
]) {
  const plainYearMonthResult = plainYearMonth.toLocaleString("en-u-ca-iso8601", options);
  const legacyDateResult = legacyDate.toLocaleString("en-u-ca-iso8601", options);
  assert.sameValue(plainYearMonthResult, legacyDateResult, `Instant.toLocaleString should format lone option ${JSON.stringify(options)}`);
}

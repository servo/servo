// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tolocalestring
description: Each date and time component can be formatted individually
locale: [en]
features: [Temporal]
---*/

const epochMs = 1735213600_321;  // 2024-12-26T11:46:40.321Z
const legacyDateLocal = new Date(epochMs);
const legacyDate = new Date(epochMs + legacyDateLocal.getTimezoneOffset() * 60 * 1000);
const plainDateTime = new Temporal.PlainDateTime(2024, 12, 26, 11, 46, 40, 321);

for (const options of [
  { "year": "numeric" },
  { "month": "long" },
  { "day": "numeric" },
  { "weekday": "long" },
  { "hour": "numeric" },
  { "minute": "numeric" },
  { "second": "numeric" },
  { "fractionalSecondDigits": 3 },
  { "dayPeriod": "short" },
  // "era" is skipped; it implies year, month, and day in "en" locale
]) {
  const plainDateTimeResult = plainDateTime.toLocaleString("en", options);
  const legacyDateResult = legacyDate.toLocaleString("en", options);
  assert.sameValue(plainDateTimeResult, legacyDateResult, `PlainDateTime.toLocaleString should format lone option ${JSON.stringify(options)}`);
}

// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: Each date and time component can be formatted individually
locale: [en, en-u-tz-utc]
features: [Temporal]
---*/

const epochMs = 1735213600_321;  // 2024-12-26T11:46:40.321Z
const epochNs = BigInt(epochMs) * 1_000_000n;
const legacyDate = new Date(epochMs);
const zonedDateTime = new Temporal.ZonedDateTime(epochNs, "UTC");

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
  { "timeZoneName": "short" },
  // "era" is skipped; it implies year, month, and day in "en" locale
]) {
  const zonedDateTimeResult = zonedDateTime.toLocaleString("en", options);
  const legacyDateResult = legacyDate.toLocaleString("en", { ...options, timeZone: "UTC" });
  assert.sameValue(zonedDateTimeResult, legacyDateResult, `ZonedDateTime.toLocaleString should format lone option ${JSON.stringify(options)}`);
}

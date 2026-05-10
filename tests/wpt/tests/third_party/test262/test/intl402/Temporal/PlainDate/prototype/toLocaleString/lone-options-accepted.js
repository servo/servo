// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: Each date component can be formatted individually
locale: [en]
features: [Temporal]
---*/

const epochMs = 1735213600_321;  // 2024-12-26T11:46:40.321Z
const legacyDate = new Date(epochMs);
const plainDate = new Temporal.PlainDate(2024, 12, 26);

for (const options of [
  { "year": "numeric" },
  { "month": "long" },
  { "day": "numeric" },
  { "weekday": "long" },
  // "era" is skipped; it implies year, month, and day in "en" locale
]) {
  const plainDateResult = plainDate.toLocaleString("en", options);
  const legacyDateResult = legacyDate.toLocaleString("en", options);
  assert.sameValue(plainDateResult, legacyDateResult, `Instant.toLocaleString should format lone option ${JSON.stringify(options)}`);
}

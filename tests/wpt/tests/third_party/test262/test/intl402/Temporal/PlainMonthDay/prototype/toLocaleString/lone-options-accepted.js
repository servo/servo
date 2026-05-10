// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tolocalestring
description: Each date component can be formatted individually
locale: [en-u-ca-iso8601]
features: [Temporal]
---*/

const epochMs = 1735213600_321;  // 2024-12-26T11:46:40.321Z
const legacyDate = new Date(epochMs);
const plainMonthDay = new Temporal.PlainMonthDay(12, 26);

for (const options of [
  { "month": "long" },
  { "day": "numeric" },
]) {
  const plainMonthDayResult = plainMonthDay.toLocaleString("en-u-ca-iso8601", options);
  const legacyDateResult = legacyDate.toLocaleString("en-u-ca-iso8601", options);
  assert.sameValue(plainMonthDayResult, legacyDateResult, `PlainMonthDay.toLocaleString should format lone option ${JSON.stringify(options)}`);
}

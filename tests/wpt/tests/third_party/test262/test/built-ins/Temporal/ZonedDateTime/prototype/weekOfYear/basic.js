// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.weekofyear
description: Checking week of year for a "normal" case, as well as for dates near the turn of the year.
features: [Temporal]
---*/

const zdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
assert.sameValue(zdt.weekOfYear, 47, "check week of year information");

for (let i = 29; i <= 31; ++i) {
  const zdt = new Temporal.PlainDateTime(1975, 12, i, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
  assert.sameValue(zdt.weekOfYear, 1, `${zdt} should be in week 1`);
}
for (let i = 1; i <= 4; ++i) {
  const zdt = new Temporal.PlainDateTime(1976, 1, i, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
  assert.sameValue(zdt.weekOfYear, 1, `${zdt} should be in week 1`);
}
for (let i = 5; i <= 11; ++i) {
  const zdt = new Temporal.PlainDateTime(1976, 1, i, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
  assert.sameValue(zdt.weekOfYear, 2, `${zdt} should be in week 2`);
}
for (let i = 20; i <= 26; ++i) {
  const zdt = new Temporal.PlainDateTime(1976, 12, i, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
  assert.sameValue(zdt.weekOfYear, 52, `${zdt} should be in week 52`);
}
for (let i = 27; i <= 31; ++i) {
  const zdt = new Temporal.PlainDateTime(1976, 12, i, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
  assert.sameValue(zdt.weekOfYear, 53, `${zdt} should be in week 53`);
}
for (let i = 1; i <= 2; ++i) {
  const zdt = new Temporal.PlainDateTime(1977, 1, i, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");
  assert.sameValue(zdt.weekOfYear, 53, `${zdt} should be in week 53`);
}


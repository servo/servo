// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.weekofyear
description: Basic tests for weekOfYear().
features: [Temporal]
---*/

for (let i = 29; i <= 31; ++i) {
  const plainDate = new Temporal.PlainDate(1975, 12, i);
  assert.sameValue(plainDate.weekOfYear, 1, `${plainDate} should be in week 1`);
}
for (let i = 1; i <= 4; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 1, i);
  assert.sameValue(plainDate.weekOfYear, 1, `${plainDate} should be in week 1`);
}
for (let i = 5; i <= 11; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 1, i);
  assert.sameValue(plainDate.weekOfYear, 2, `${plainDate} should be in week 2`);
}
for (let i = 20; i <= 26; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 12, i);
  assert.sameValue(plainDate.weekOfYear, 52, `${plainDate} should be in week 52`);
}
for (let i = 27; i <= 31; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 12, i);
  assert.sameValue(plainDate.weekOfYear, 53, `${plainDate} should be in week 53`);
}
for (let i = 1; i <= 2; ++i) {
  const plainDate = new Temporal.PlainDate(1977, 1, i);
  assert.sameValue(plainDate.weekOfYear, 53, `${plainDate} should be in week 53`);
}


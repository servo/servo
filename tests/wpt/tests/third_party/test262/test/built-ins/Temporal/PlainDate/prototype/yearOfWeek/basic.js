// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.yearofweek
description: Basic tests for yearOfWeek().
features: [Temporal]
---*/

for (let i = 29; i <= 31; ++i) {
  const plainDate = new Temporal.PlainDate(1975, 12, i);
  assert.sameValue(plainDate.yearOfWeek, 1976, `${plainDate} should be in yearOfWeek 1976`);
}
for (let i = 1; i <= 11; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 1, i);
  assert.sameValue(plainDate.yearOfWeek, 1976, `${plainDate} should be in yearOfWeek 1976`);
}
for (let i = 20; i <= 31; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 12, i);
  assert.sameValue(plainDate.yearOfWeek, 1976, `${plainDate} should be in yearOfWeek 1976`);
}
for (let i = 1; i <= 2; ++i) {
  const plainDate = new Temporal.PlainDate(1977, 1, i);
  assert.sameValue(plainDate.yearOfWeek, 1976, `${plainDate} should be in yearOfWeek 1976`);
}


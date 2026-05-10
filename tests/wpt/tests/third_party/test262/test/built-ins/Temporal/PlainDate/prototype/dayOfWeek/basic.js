// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.dayofweek
description: Basic tests for dayOfWeek().
features: [Temporal]
---*/

for (let i = 1; i <= 7; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 11, 14 + i);
  assert.sameValue(plainDate.dayOfWeek, i, `${plainDate} should be on day ${i}`);
}

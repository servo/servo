// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plaindate.prototype.dayofyear
description: Basic tests for dayOfYear().
features: [Temporal]
---*/

for (let i = 1; i <= 7; ++i) {
  const plainDate = new Temporal.PlainDate(1976, 11, 14 + i);
  assert.sameValue(plainDate.dayOfYear, 319 + i, `${plainDate} should be on day ${319 + i}`);
}

assert.sameValue((new Temporal.PlainDate(1970, 1, 1)).dayOfYear, 1);
assert.sameValue((new Temporal.PlainDate(2000, 1, 1)).dayOfYear, 1);
assert.sameValue((new Temporal.PlainDate(2021, 1, 15)).dayOfYear, 15);
assert.sameValue((new Temporal.PlainDate(2020, 2, 15)).dayOfYear, 46);
assert.sameValue((new Temporal.PlainDate(2000, 2, 15)).dayOfYear, 46);
assert.sameValue((new Temporal.PlainDate(2020, 3, 15)).dayOfYear, 75);
assert.sameValue((new Temporal.PlainDate(2000, 3, 15)).dayOfYear, 75);
assert.sameValue((new Temporal.PlainDate(2001, 3, 15)).dayOfYear, 74);
assert.sameValue((new Temporal.PlainDate(2000, 12, 31)).dayOfYear, 366);
assert.sameValue((new Temporal.PlainDate(2001, 12, 31)).dayOfYear, 365);
assert.sameValue(Temporal.PlainDate.from('2019-01-18').dayOfYear, 18);
assert.sameValue(Temporal.PlainDate.from('2020-02-18').dayOfYear, 49);
assert.sameValue(Temporal.PlainDate.from('2019-12-31').dayOfYear, 365);
assert.sameValue(Temporal.PlainDate.from('2000-12-31').dayOfYear, 366);

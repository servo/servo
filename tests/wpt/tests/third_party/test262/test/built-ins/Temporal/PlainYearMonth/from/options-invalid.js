// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: TypeError thrown when a primitive is passed as the options argument
features: [Temporal]
---*/

const items = [
  { year: 2000, month: 11 },
  "2000-11",
  new Temporal.PlainYearMonth(2000, 11),
];
const values = [null, true, "hello", Symbol("foo"), 1, 1n];

for (const item of items) {
  for (const badOptions of values) {
    assert.throws(TypeError, () => Temporal.PlainYearMonth.from(item, badOptions));
  }
}

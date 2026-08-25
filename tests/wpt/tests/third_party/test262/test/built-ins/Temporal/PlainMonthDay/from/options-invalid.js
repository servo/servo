// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: TypeError thrown when options argument is a primitive
features: [BigInt, Symbol, Temporal]
---*/

const fields = { month: 2, day: 31 };

const values = [null, true, "hello", Symbol("foo"), 1, 1n];
for (const badOptions of values) {
  assert.throws(TypeError, () => Temporal.PlainMonthDay.from(fields, badOptions));
}

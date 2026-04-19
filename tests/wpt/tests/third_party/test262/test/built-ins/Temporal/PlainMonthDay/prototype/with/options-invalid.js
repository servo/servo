// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: TypeError thrown when options argument is a primitive
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainMonthDay(2, 2);
[null, true, "hello", Symbol("foo"), 1, 1n].forEach((badOptions) =>
  assert.throws(TypeError, () => instance.with({ day: 17 }, badOptions))
);

// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.toplaindate
description: Throws a TypeError if the argument is not an Object
features: [BigInt, Symbol, Temporal]
---*/

[null, undefined, true, 3.1416, "a string", Symbol("symbol"), 7n].forEach((primitive) => {
  const plainMonthDay = new Temporal.PlainMonthDay(5, 2);
  assert.throws(TypeError, () => plainMonthDay.toPlainDate(primitive));
});

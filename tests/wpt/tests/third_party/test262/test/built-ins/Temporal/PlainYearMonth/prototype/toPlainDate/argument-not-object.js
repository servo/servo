// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.toplaindate
description: Throws a TypeError if the argument is not an Object
features: [BigInt, Symbol, Temporal]
---*/

[null, undefined, true, 3.1416, "a string", Symbol("symbol"), 7n].forEach((primitive) => {
  const plainYearMonth = new Temporal.PlainYearMonth(2000, 5);
  assert.throws(TypeError, () => plainYearMonth.toPlainDate(primitive));
});

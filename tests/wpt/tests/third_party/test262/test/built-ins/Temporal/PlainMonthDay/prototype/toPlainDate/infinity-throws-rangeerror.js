// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws a RangeError if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plainmonthday.prototype.toplaindate
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(5, 2);

[Infinity, -Infinity].forEach((inf) => {
  assert.throws(RangeError, () => instance.toPlainDate({ year: inf }), `year property cannot be ${inf}`);

  const calls = [];
  const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, "year");
  assert.throws(RangeError, () => instance.toPlainDate({ year: obj }));
  assert.compareArray(calls, ["get year.valueOf", "call year.valueOf"], "it fails after fetching the primitive value");
});

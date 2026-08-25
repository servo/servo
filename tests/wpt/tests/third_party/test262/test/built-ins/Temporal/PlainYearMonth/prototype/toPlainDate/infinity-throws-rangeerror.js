// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plainyearmonth.prototype.toplaindate
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 5);

[Infinity, -Infinity].forEach((inf) => {
  assert.throws(RangeError, () => instance.toPlainDate({ day: inf }), `day property cannot be ${inf}`);

  const calls = [];
  const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, "day");
  assert.throws(RangeError, () => instance.toPlainDate({ day: obj }));
  assert.compareArray(calls, ["get day.valueOf", "call day.valueOf"], "it fails after fetching the primitive value");
});

// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws a RangeError if eraYear in the property bag is Infinity or -Infinity
esid: sec-temporal.plainmonthday.prototype.toplaindate
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(5, 2, "gregory");

[Infinity, -Infinity].forEach((inf) => {
  assert.throws(RangeError, () => instance.toPlainDate({ era: "ad", eraYear: inf }), `eraYear property cannot be ${inf}`);

  const calls = [];
  const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, "eraYear");
  assert.throws(RangeError, () => instance.toPlainDate({ era: "ad", eraYear: obj }));
  assert.compareArray(calls, ["get eraYear.valueOf", "call eraYear.valueOf"], "it fails after fetching the primitive value");
});

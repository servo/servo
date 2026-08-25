// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if eraYear in the property bag is Infinity or -Infinity
esid: sec-temporal.plainyearmonth.prototype.until
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 5, "gregory");
const base = { era: "ad", month: 5, calendar: "gregory" };

[Infinity, -Infinity].forEach((inf) => {
  assert.throws(RangeError, () => instance.until({ ...base, eraYear: inf }), `eraYear property cannot be ${inf}`);

  const calls = [];
  const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, "eraYear");
  assert.throws(RangeError, () => instance.until({ ...base, eraYear: obj }));
  assert.compareArray(calls, ["get eraYear.valueOf", "call eraYear.valueOf"], "it fails after fetching the primitive value");
});

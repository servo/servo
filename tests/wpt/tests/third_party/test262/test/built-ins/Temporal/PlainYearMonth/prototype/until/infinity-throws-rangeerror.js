// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plainyearmonth.prototype.until
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 5);
const base = { year: 2000, month: 5 };

[Infinity, -Infinity].forEach((inf) => {
  ["year", "month"].forEach((prop) => {
    assert.throws(RangeError, () => instance.until({ ...base, [prop]: inf }), `${prop} property cannot be ${inf}`);

    const calls = [];
    const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, prop);
    assert.throws(RangeError, () => instance.until({ ...base, [prop]: obj }));
    assert.compareArray(calls, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");
  });
});

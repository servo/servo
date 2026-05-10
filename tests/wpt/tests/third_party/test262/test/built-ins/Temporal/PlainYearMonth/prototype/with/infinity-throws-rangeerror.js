// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plainyearmonth.prototype.with
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 5);

[Infinity, -Infinity].forEach((inf) => {
  ["year", "month"].forEach((prop) => {
    ["constrain", "reject"].forEach((overflow) => {
      assert.throws(RangeError, () => instance.with({ [prop]: inf }, { overflow }), `${prop} property cannot be ${inf} (overflow ${overflow}`);

      const calls = [];
      const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, prop);
      assert.throws(RangeError, () => instance.with({ [prop]: obj }, { overflow }));
      assert.compareArray(calls, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");
    });
  });
});

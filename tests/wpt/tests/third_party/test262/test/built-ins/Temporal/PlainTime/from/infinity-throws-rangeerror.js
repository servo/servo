// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plaintime.from
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const base = { hour: 15, minute: 30, second: 45, millisecond: 987, microsecond: 654, nanosecond: 321 };

[Infinity, -Infinity].forEach((inf) => {
  ["hour", "minute", "second", "millisecond", "microsecond", "nanosecond"].forEach((prop) => {
    ["constrain", "reject"].forEach((overflow) => {
      assert.throws(RangeError, () => Temporal.PlainTime.from({ ...base, [prop]: inf }, { overflow }), `${prop} property cannot be ${inf} (overflow ${overflow}`);

      const calls = [];
      const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, prop);
      assert.throws(RangeError, () => Temporal.PlainTime.from({ ...base, [prop]: obj }, { overflow }));
      assert.compareArray(calls, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");
    });
  });
});

// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.duration.prototype.total
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 2, 3, 4, 5, 6, 7, 987, 654, 321);
const base = { year: 2000, month: 5, day: 2, hour: 15, minute: 30, second: 45, millisecond: 987, microsecond: 654, nanosecond: 321 };

[Infinity, -Infinity].forEach((inf) => {
  ["year", "month", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"].forEach((prop) => {
    assert.throws(RangeError, () => instance.total({ unit: "seconds", relativeTo: { ...base, [prop]: inf } }), `${prop} property cannot be ${inf} in relativeTo`);

    const calls = [];
    const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, prop);
    assert.throws(RangeError, () => instance.total({ unit: "seconds", relativeTo: { ...base, [prop]: obj } }));
    assert.compareArray(calls, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");
  });
});

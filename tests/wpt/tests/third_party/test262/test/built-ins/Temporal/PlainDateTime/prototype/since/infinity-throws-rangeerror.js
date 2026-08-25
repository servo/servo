// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plaindatetime.prototype.since
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 15);
const base = { year: 2000, month: 5, day: 2, hour: 15, minute: 30, second: 45, millisecond: 987, microsecond: 654, nanosecond: 321 };

[Infinity, -Infinity].forEach((inf) => {
  ["year", "month", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"].forEach((prop) => {
    assert.throws(RangeError, () => instance.since({ ...base, [prop]: inf }), `${prop} property cannot be ${inf}`);

    const calls = [];
    const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, prop);
    assert.throws(RangeError, () => instance.since({ ...base, [prop]: obj }));
    assert.compareArray(calls, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");
  });
});

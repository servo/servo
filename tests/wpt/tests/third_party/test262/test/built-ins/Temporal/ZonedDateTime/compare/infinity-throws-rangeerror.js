// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in a property bag for either argument is Infinity or -Infinity
esid: sec-temporal.zoneddatetime.compare
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const other = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");
const base = { year: 2000, month: 5, day: 2, hour: 15, minute: 30, second: 45, millisecond: 987, microsecond: 654, nanosecond: 321, timeZone: "UTC" };

[Infinity, -Infinity].forEach((inf) => {
  ["year", "month", "day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"].forEach((prop) => {
    assert.throws(RangeError, () => Temporal.ZonedDateTime.compare({ ...base, [prop]: inf }, other), `${prop} property cannot be ${inf}`);

    assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(other, { ...base, [prop]: inf }), `${prop} property cannot be ${inf}`);

    const calls1 = [];
    const obj1 = TemporalHelpers.toPrimitiveObserver(calls1, inf, prop);
    assert.throws(RangeError, () => Temporal.ZonedDateTime.compare({ ...base, [prop]: obj1 }, other));
    assert.compareArray(calls1, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");

    const calls2 = [];
    const obj2 = TemporalHelpers.toPrimitiveObserver(calls2, inf, prop);
    assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(other, { ...base, [prop]: obj2 }));
    assert.compareArray(calls2, [`get ${prop}.valueOf`, `call ${prop}.valueOf`], "it fails after fetching the primitive value");
  });
});

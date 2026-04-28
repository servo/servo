// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if any value in the property bag is Infinity or -Infinity
esid: sec-temporal.plainmonthday.prototype.with
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainMonthDay(5, 2);

[Infinity, -Infinity].forEach((inf) => {
  ["constrain", "reject"].forEach((overflow) => {
    assert.throws(RangeError, () => instance.with({ day: inf }, { overflow }), `day property cannot be ${inf} (overflow ${overflow}`);

    const calls = [];
    const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, "day");
    assert.throws(RangeError, () => instance.with({ day: obj }, { overflow }));
    assert.compareArray(calls, ["get day.valueOf", "call day.valueOf"], "it fails after fetching the primitive value");
  });
});

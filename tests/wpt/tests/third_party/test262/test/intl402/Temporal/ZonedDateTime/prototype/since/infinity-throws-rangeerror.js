// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if eraYear in the property bag is Infinity or -Infinity
esid: sec-temporal.zoneddatetime.prototype.since
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC", "gregory");
const base = { era: "ad", month: 5, day: 2, hour: 15, timeZone: "UTC", calendar: "gregory" };

[Infinity, -Infinity].forEach((inf) => {
  assert.throws(RangeError, () => instance.since({ ...base, eraYear: inf }), `eraYear property cannot be ${inf}`);

  const calls = [];
  const obj = TemporalHelpers.toPrimitiveObserver(calls, inf, "eraYear");
  assert.throws(RangeError, () => instance.since({ ...base, eraYear: obj }));
  assert.compareArray(calls, ["get eraYear.valueOf", "call eraYear.valueOf"], "it fails after fetching the primitive value");
});

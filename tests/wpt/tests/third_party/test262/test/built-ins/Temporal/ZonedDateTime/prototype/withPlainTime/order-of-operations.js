// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.withplaintime
description: User code calls happen in the correct order
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const actual = [];
const expected = [
  // ToTemporalTime
  "get plainTimeLike.hour",
  "get plainTimeLike.hour.valueOf",
  "call plainTimeLike.hour.valueOf",
  "get plainTimeLike.microsecond",
  "get plainTimeLike.microsecond.valueOf",
  "call plainTimeLike.microsecond.valueOf",
  "get plainTimeLike.millisecond",
  "get plainTimeLike.millisecond.valueOf",
  "call plainTimeLike.millisecond.valueOf",
  "get plainTimeLike.minute",
  "get plainTimeLike.minute.valueOf",
  "call plainTimeLike.minute.valueOf",
  "get plainTimeLike.nanosecond",
  "get plainTimeLike.nanosecond.valueOf",
  "call plainTimeLike.nanosecond.valueOf",
  "get plainTimeLike.second",
  "get plainTimeLike.second.valueOf",
  "call plainTimeLike.second.valueOf",
];

const instance = new Temporal.ZonedDateTime(946713600_000_000_000n /* 2000-01-01T00:00-08:00 */, "UTC");

const plainTimeLike = TemporalHelpers.propertyBagObserver(actual, {
  hour: 2,
  minute: 30,
  second: 0,
  millisecond: 0,
  microsecond: 0,
  nanosecond: 0,
}, "plainTimeLike");

instance.withPlainTime(plainTimeLike);
assert.compareArray(actual, expected, "order of operations");
actual.splice(0); // clear

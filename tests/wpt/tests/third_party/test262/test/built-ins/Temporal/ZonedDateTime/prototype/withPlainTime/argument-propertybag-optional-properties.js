// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "UTC");

const minimumProperties = {
  hour: 0,
};
const allProperties = {
  hour: 0,
  minute: 0,
  second: 0,
  millisecond: 0,
  microsecond: 0,
  nanosecond: 0,
};
const resultWithout = instance.withPlainTime(minimumProperties);
const resultWith = instance.withPlainTime(allProperties);
assert(resultWithout.equals(resultWith), "results should be the same with and without optional properties");

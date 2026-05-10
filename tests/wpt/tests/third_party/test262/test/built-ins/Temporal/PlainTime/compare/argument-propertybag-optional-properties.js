// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

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
const resultWithout = Temporal.PlainTime.compare(minimumProperties, minimumProperties);
const resultWith = Temporal.PlainTime.compare(allProperties, allProperties);
assert.sameValue(resultWithout, resultWith, "results should be the same with and without optional properties");

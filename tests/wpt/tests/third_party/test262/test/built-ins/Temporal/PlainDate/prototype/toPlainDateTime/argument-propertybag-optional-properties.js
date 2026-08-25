// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);

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
const resultWithout = instance.toPlainDateTime(minimumProperties);
const resultWith = instance.toPlainDateTime(allProperties);
assert(resultWithout.equals(resultWith), "results should be the same with and without optional properties");

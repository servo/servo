// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Invalid object arguments.
features: [Temporal]
---*/

const tests = [
  { years: 0.5 },
  { months: 0.5 },
  { weeks: 0.5 },
  { days: 0.5 },
  { hours: 0.5, minutes: 20 },
  { hours: 0.5, seconds: 15 },
  { minutes: 10.7, nanoseconds: 400 },
  { hours: 1, minutes: -30 },
];

for (const input of tests) {
  assert.throws(RangeError, () => Temporal.Duration.from(input));
}

assert.throws(TypeError, () => Temporal.Duration.from({}));
assert.throws(TypeError, () => Temporal.Duration.from({ month: 12 }));

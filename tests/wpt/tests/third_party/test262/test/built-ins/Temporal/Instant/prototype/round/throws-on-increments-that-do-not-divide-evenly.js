// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: round() throws on increments that do not divide evenly into solar days.
features: [Temporal]
---*/

const inst = new Temporal.Instant(0n);

assert.throws(RangeError, () => inst.round({
  smallestUnit: "hour",
  roundingIncrement: 7
}));
assert.throws(RangeError, () => inst.round({
  smallestUnit: "minute",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => inst.round({
  smallestUnit: "second",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => inst.round({
  smallestUnit: "millisecond",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => inst.round({
  smallestUnit: "microsecond",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => inst.round({
  smallestUnit: "nanosecond",
  roundingIncrement: 29
}));

// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.round
description: Throws on invalid increments.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");

// throws on increments that do not divide evenly into the next highest
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "day",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "hour",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "minute",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "second",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "millisecond",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "microsecond",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "nanosecond",
  roundingIncrement: 29
}));

// throws on increments that are equal to the next highest
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "hour",
  roundingIncrement: 24
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "minute",
  roundingIncrement: 60
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "second",
  roundingIncrement: 60
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "millisecond",
  roundingIncrement: 1000
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "microsecond",
  roundingIncrement: 1000
}));
assert.throws(RangeError, () => zdt.round({
  smallestUnit: "nanosecond",
  roundingIncrement: 1000
}));

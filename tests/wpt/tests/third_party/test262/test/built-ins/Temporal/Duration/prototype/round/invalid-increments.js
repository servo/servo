// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test various invalid rounding increments.
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const relativeTo = Temporal.PlainDate.from("2020-01-01");

// throws on increments that do not divide evenly into the next highest
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "hours",
  roundingIncrement: 11
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "minutes",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "seconds",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "milliseconds",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "microseconds",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "nanoseconds",
  roundingIncrement: 29
}));

// throws on increments that are equal to the next highest
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "hours",
  roundingIncrement: 24
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "minutes",
  roundingIncrement: 60
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "seconds",
  roundingIncrement: 60
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "milliseconds",
  roundingIncrement: 1000
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "microseconds",
  roundingIncrement: 1000
}));
assert.throws(RangeError, () => d.round({
  relativeTo,
  smallestUnit: "nanoseconds",
  roundingIncrement: 1000
}));

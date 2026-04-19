// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Rounds to various increments.
features: [Temporal]
---*/

/*
const earlier = Temporal.ZonedDateTime.from('2019-01-08T09:22:36.123456789+01:00[+01:00]');
const later = Temporal.ZonedDateTime.from('2021-09-07T13:39:40.987654321+01:00[+01:00]');
*/
const earlier = new Temporal.ZonedDateTime(1546935756123456789n, "+01:00");
const later = new Temporal.ZonedDateTime(1631018380987654321n, "+01:00");

// throws on increments that do not divide evenly into the next highest
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "hours",
  roundingIncrement: 11
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "minutes",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "seconds",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "milliseconds",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "microseconds",
  roundingIncrement: 29
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "nanoseconds",
  roundingIncrement: 29
}));

// throws on increments that are equal to the next highest
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "hours",
  roundingIncrement: 24
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "minutes",
  roundingIncrement: 60
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "seconds",
  roundingIncrement: 60
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "milliseconds",
  roundingIncrement: 1000
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "microseconds",
  roundingIncrement: 1000
}));
assert.throws(RangeError, () => later.since(earlier, {
  smallestUnit: "nanoseconds",
  roundingIncrement: 1000
}));

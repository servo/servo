// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Test various rounding increments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1969-07-24T16:50:35.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");
const largestUnit = "hours";

// rounds to an increment of hours
TemporalHelpers.assertDuration(earlier.until(later, {
  largestUnit,
  smallestUnit: "hours",
  roundingIncrement: 4,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440608, 0, 0, 0, 0, 0);

// rounds to an increment of minutes
TemporalHelpers.assertDuration(earlier.until(later, {
  largestUnit,
  smallestUnit: "minutes",
  roundingIncrement: 30,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440610, 0, 0, 0, 0, 0);

// rounds to an increment of seconds
TemporalHelpers.assertDuration(earlier.until(later, {
  largestUnit,
  smallestUnit: "seconds",
  roundingIncrement: 15,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440609, 56, 0, 0, 0, 0);

// rounds to an increment of milliseconds
TemporalHelpers.assertDuration(earlier.until(later, {
  largestUnit,
  smallestUnit: "milliseconds",
  roundingIncrement: 10,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440609, 56, 3, 150, 0, 0);

// rounds to an increment of microseconds
TemporalHelpers.assertDuration(earlier.until(later, {
  largestUnit,
  smallestUnit: "microseconds",
  roundingIncrement: 10,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440609, 56, 3, 148, 530, 0);

// rounds to an increment of nanoseconds
TemporalHelpers.assertDuration(earlier.until(later, {
  largestUnit,
  smallestUnit: "nanoseconds",
  roundingIncrement: 10,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 440609, 56, 3, 148, 529, 310);


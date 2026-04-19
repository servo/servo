// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test that round() accepts singular units.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const relativeTo = new Temporal.PlainDate(2020, 1, 1);

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "year",
  relativeTo
}), d.round({
  largestUnit: "years",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "year",
  relativeTo
}), d.round({
  smallestUnit: "years",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "month",
  relativeTo
}), d.round({
  largestUnit: "months",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "month",
  relativeTo
}), d.round({
  smallestUnit: "months",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "day",
  relativeTo
}), d.round({
  largestUnit: "days",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "day",
  relativeTo
}), d.round({
  smallestUnit: "days",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "hour",
  relativeTo
}), d.round({
  largestUnit: "hours",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "hour",
  relativeTo
}), d.round({
  smallestUnit: "hours",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "minute",
  relativeTo
}), d.round({
  largestUnit: "minutes",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "minute",
  relativeTo
}), d.round({
  smallestUnit: "minutes",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "second",
  relativeTo
}), d.round({
  largestUnit: "seconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "second",
  relativeTo
}), d.round({
  smallestUnit: "seconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "millisecond",
  relativeTo
}), d.round({
  largestUnit: "milliseconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "millisecond",
  relativeTo
}), d.round({
  smallestUnit: "milliseconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "microsecond",
  relativeTo
}), d.round({
  largestUnit: "microseconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "microsecond",
  relativeTo
}), d.round({
  smallestUnit: "microseconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  largestUnit: "nanosecond",
  relativeTo
}), d.round({
  largestUnit: "nanoseconds",
  relativeTo
}));

TemporalHelpers.assertDurationsEqual(d.round({
  smallestUnit: "nanosecond",
  relativeTo
}), d.round({
  smallestUnit: "nanoseconds",
  relativeTo
}));

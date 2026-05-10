// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Fallback value for roundingMode option
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = Temporal.PlainTime.from("08:22:36.123456789");
const later = Temporal.PlainTime.from("12:39:40.987654321");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours", roundingMode: undefined }),
  0, 0, 0, 0, 4, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "hours" }),
  0, 0, 0, 0, 4, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "hours", roundingMode: undefined }),
  0, 0, 0, 0, -4, 0, 0, 0, 0, 0, "hours");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "hours" }),
  0, 0, 0, 0, -4, 0, 0, 0, 0, 0, "hours");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes", roundingMode: undefined }),
  0, 0, 0, 0, 4, 17, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "minutes" }),
  0, 0, 0, 0, 4, 17, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "minutes", roundingMode: undefined }),
  0, 0, 0, 0, -4, -17, 0, 0, 0, 0, "minutes");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "minutes" }),
  0, 0, 0, 0, -4, -17, 0, 0, 0, 0, "minutes");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "seconds", roundingMode: undefined }),
  0, 0, 0, 0, 4, 17, 4, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "seconds" }),
  0, 0, 0, 0, 4, 17, 4, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "seconds", roundingMode: undefined }),
  0, 0, 0, 0, -4, -17, -4, 0, 0, 0, "seconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "seconds" }),
  0, 0, 0, 0, -4, -17, -4, 0, 0, 0, "seconds");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds", roundingMode: undefined }),
  0, 0, 0, 0, 4, 17, 4, 864, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "milliseconds" }),
  0, 0, 0, 0, 4, 17, 4, 864, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "milliseconds", roundingMode: undefined }),
  0, 0, 0, 0, -4, -17, -4, -864, 0, 0, "milliseconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "milliseconds" }),
  0, 0, 0, 0, -4, -17, -4, -864, 0, 0, "milliseconds");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "microseconds", roundingMode: undefined }),
  0, 0, 0, 0, 4, 17, 4, 864, 197, 0, "microseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "microseconds" }),
  0, 0, 0, 0, 4, 17, 4, 864, 197, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "microseconds", roundingMode: undefined }),
  0, 0, 0, 0, -4, -17, -4, -864, -197, 0, "microseconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "microseconds" }),
  0, 0, 0, 0, -4, -17, -4, -864, -197, 0, "microseconds");

TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds", roundingMode: undefined }),
  0, 0, 0, 0, 4, 17, 4, 864, 197, 532, "nanoseconds");
TemporalHelpers.assertDuration(
  later.since(earlier, { smallestUnit: "nanoseconds" }),
  0, 0, 0, 0, 4, 17, 4, 864, 197, 532, "nanoseconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "nanoseconds", roundingMode: undefined }),
  0, 0, 0, 0, -4, -17, -4, -864, -197, -532, "nanoseconds");
TemporalHelpers.assertDuration(
  earlier.since(later, { smallestUnit: "nanoseconds" }),
  0, 0, 0, 0, -4, -17, -4, -864, -197, -532, "nanoseconds");

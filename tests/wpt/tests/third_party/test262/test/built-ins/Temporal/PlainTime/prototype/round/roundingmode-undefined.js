// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Tests calculations with roundingMode undefined.
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const plainTime = Temporal.PlainTime.from("13:46:23.123456789");

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hour", roundingMode: undefined }),
  14, 0, 0, 0, 0, 0, "hour");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "hour" }),
  14, 0, 0, 0, 0, 0, "hour");

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minute", roundingMode: undefined }),
  13, 46, 0, 0, 0, 0, "minute");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "minute" }),
  13, 46, 0, 0, 0, 0, "minute");

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "second", roundingMode: undefined }),
  13, 46, 23, 0, 0, 0, "second");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "second" }),
  13, 46, 23, 0, 0, 0, "second");

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "millisecond", roundingMode: undefined }),
  13, 46, 23, 123, 0, 0, "millisecond");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "millisecond" }),
  13, 46, 23, 123, 0, 0, "millisecond");

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microsecond", roundingMode: undefined }),
  13, 46, 23, 123, 457, 0, "microsecond");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "microsecond" }),
  13, 46, 23, 123, 457, 0, "microsecond");

TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanosecond", roundingMode: undefined }),
  13, 46, 23, 123, 456, 789, "nanosecond");
TemporalHelpers.assertPlainTime(
  plainTime.round({ smallestUnit: "nanosecond" }),
  13, 46, 23, 123, 456, 789, "nanosecond");

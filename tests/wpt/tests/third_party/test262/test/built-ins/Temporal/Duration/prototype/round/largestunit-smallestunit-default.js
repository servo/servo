// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: assumes a different default for largestUnit if smallestUnit is larger than the default
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const relativeTo = Temporal.PlainDate.from("2020-01-01");
const almostYear = Temporal.Duration.from({ days: 364 });
TemporalHelpers.assertDuration(almostYear.round({ smallestUnit: "years", relativeTo }),
  1, 0, 0, 0, 0, 0, 0, 0, 0, 0, "years");
const almostMonth = Temporal.Duration.from({ days: 27 });
TemporalHelpers.assertDuration(almostMonth.round({ smallestUnit: "months", relativeTo }),
  0, 1, 0, 0, 0, 0, 0, 0, 0, 0, "months");
const almostWeek = Temporal.Duration.from({ days: 6 });
TemporalHelpers.assertDuration(almostWeek.round({ smallestUnit: "weeks", relativeTo }),
  0, 0, 1, 0, 0, 0, 0, 0, 0, 0, "weeks");
const almostDay = Temporal.Duration.from({ seconds: 86399 });
TemporalHelpers.assertDuration(almostDay.round({ smallestUnit: "days" }),
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0, "days");
const almostHour = Temporal.Duration.from({ seconds: 3599 });
TemporalHelpers.assertDuration(almostHour.round({ smallestUnit: "hours" }),
  0, 0, 0, 0, 1, 0, 0, 0, 0, 0, "hours");
const almostMinute = Temporal.Duration.from({ seconds: 59 });
TemporalHelpers.assertDuration(almostMinute.round({ smallestUnit: "minutes" }),
  0, 0, 0, 0, 0, 1, 0, 0, 0, 0, "minutes");
const almostSecond = Temporal.Duration.from({ nanoseconds: 999999999 });
TemporalHelpers.assertDuration(almostSecond.round({ smallestUnit: "seconds" }),
  0, 0, 0, 0, 0, 0, 1, 0, 0, 0, "seconds");
const almostMillisecond = Temporal.Duration.from({ nanoseconds: 999999 });
TemporalHelpers.assertDuration(almostMillisecond.round({ smallestUnit: "milliseconds" }),
  0, 0, 0, 0, 0, 0, 0, 1, 0, 0, "milliseconds");
const almostMicrosecond = Temporal.Duration.from({ nanoseconds: 999 });
TemporalHelpers.assertDuration(almostMicrosecond.round({ smallestUnit: "microseconds" }),
  0, 0, 0, 0, 0, 0, 0, 0, 1, 0, "microseconds");

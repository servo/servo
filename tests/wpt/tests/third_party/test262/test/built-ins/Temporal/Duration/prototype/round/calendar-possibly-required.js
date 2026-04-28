// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: relativeTo argument required if days = 0 but years/months/weeks non-zero
features: [Temporal]
includes: [temporalHelpers.js]
---*/
const duration1 = new Temporal.Duration(1);
const duration2 = new Temporal.Duration(0, 12);
const duration3 = new Temporal.Duration(0, 0, 5);
const duration4 = new Temporal.Duration(0, 0, 0, 42);
const pd = new Temporal.PlainDate(2021, 12, 15);
const relativeToYears = { relativeTo: pd, largestUnit: "years" };
const relativeToMonths = { relativeTo: pd, largestUnit: "months" };
const relativeToWeeks = { relativeTo: pd, largestUnit: "weeks" };
const relativeToDays = { relativeTo: pd, largestUnit: "days" };

assert.throws(
  RangeError,
  () => { duration1.round({ relativeTo: pd }); },
  "rounding a year Duration fails without largest/smallest unit"
);

TemporalHelpers.assertDuration(duration1.round(relativeToYears), 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, "round year duration to years");
TemporalHelpers.assertDuration(duration1.round(relativeToMonths), 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, "round year duration to months");
TemporalHelpers.assertDuration(duration1.round(relativeToWeeks), 0, 0, 52, 1, 0, 0, 0, 0, 0, 0, "round year duration to weeks");
TemporalHelpers.assertDuration(duration1.round(relativeToDays), 0, 0, 0, 365, 0, 0, 0, 0, 0, 0, "round year duration to days");

assert.throws(
  RangeError,
  () => { duration2.round({ relativeTo: pd }); },
  "rounding a month Duration fails without largest/smallest unit"
);

TemporalHelpers.assertDuration(duration2.round(relativeToYears), 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, "round month duration to years");
TemporalHelpers.assertDuration(duration2.round(relativeToMonths), 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, "round month duration to months");
TemporalHelpers.assertDuration(duration2.round(relativeToWeeks), 0, 0, 52, 1, 0, 0, 0, 0, 0, 0, "round month duration to weeks");
TemporalHelpers.assertDuration(duration2.round(relativeToDays), 0, 0, 0, 365, 0, 0, 0, 0, 0, 0, "round month duration to days");

assert.throws(
  RangeError,
  () => { duration3.round({ relativeTo: pd }); },
  "rounding a week Duration fails without largest/smallest unit"
);

TemporalHelpers.assertDuration(duration3.round(relativeToYears), 0, 1, 0, 4, 0, 0, 0, 0, 0, 0, "round week duration to years");
TemporalHelpers.assertDuration(duration3.round(relativeToMonths), 0, 1, 0, 4, 0, 0, 0, 0, 0, 0, "round week duration to months");
TemporalHelpers.assertDuration(duration3.round(relativeToWeeks), 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, "round week duration to weeks");
TemporalHelpers.assertDuration(duration3.round(relativeToDays), 0, 0, 0, 35, 0, 0, 0, 0, 0, 0, "round week duration to days");

assert.throws(
  RangeError,
  () => { duration4.round({ relativeTo: pd }); },
  "rounding a day Duration fails without largest/smallest unit"
);

TemporalHelpers.assertDuration(duration4.round(relativeToYears), 0, 1, 0, 11, 0, 0, 0, 0, 0, 0, "round day duration to years");
TemporalHelpers.assertDuration(duration4.round(relativeToMonths), 0, 1, 0, 11, 0, 0, 0, 0, 0, 0, "round day duration to months");
TemporalHelpers.assertDuration(duration4.round(relativeToWeeks), 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, "round day duration to weeks");
TemporalHelpers.assertDuration(duration4.round(relativeToDays), 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, "round day duration to days");

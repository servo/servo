// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.duration.prototype.total
description: >
    ZonedDateTime relativeTo affects day length when the duration encompasses a
    DST change
features: [Temporal]
---*/

const oneDay = new Temporal.Duration(0, 0, 0, 1);
const oneDayNeg = new Temporal.Duration(0, 0, 0, -1);
const hours12 = new Temporal.Duration(0, 0, 0, 0, 12);
const hours12Neg = new Temporal.Duration(0, 0, 0, 0, -12);
const hours25 = new Temporal.Duration(0, 0, 0, 0, 25);
const hours25Neg = new Temporal.Duration(0, 0, 0, 0, -25);
const hours48 = new Temporal.Duration(0, 0, 0, 0, 48);

const skippedHourDay = new Temporal.ZonedDateTime(
  954662400_000_000_000n /* = 2000-04-02T08Z */,
  "America/Vancouver"); /* = 2000-04-02T00-08 in local time */
const repeatedHourDay = new Temporal.ZonedDateTime(
  972802800_000_000_000n /* = 2000-10-29T07Z */,
  "America/Vancouver"); /* = 2000-10-29T00-07 in local time */
const inRepeatedHour = new Temporal.ZonedDateTime(
  972806400_000_000_000n /* = 2000-10-29T08Z */,
  "America/Vancouver"); /* = 2000-10-29T01-07 in local time */
const oneDayAfterRepeatedHour = new Temporal.ZonedDateTime(
  972896400_000_000_000n /* = 2000-10-30T09Z */,
  "America/Vancouver"); /* = 2000-10-30T01-08 in local time */
const beforeSkippedHour = new Temporal.ZonedDateTime(
  954585000_000_000_000n /* = 2000-04-01T10:30Z */,
  "America/Vancouver"); /* = 2000-04-01T02:30-08 in local time */
const dayAfterSkippedHour = new Temporal.ZonedDateTime(
  954745200_000_000_000n /* = 2000-04-03T07Z */,
  "America/Vancouver"); /* = 2000-04-03T00-07 in local time */
const afterSkippedHour = new Temporal.ZonedDateTime(
  954702000_000_000_000n /* = 2000-04-02T19Z */,
  "America/Vancouver"); /* = 2000-04-02T12-07 in local time */
const afterRepeatedHour = new Temporal.ZonedDateTime(
  972892800_000_000_000n /* = 2000-10-30T08Z */,
  "America/Vancouver"); /* = 2000-10-30T00-08 in local time */
const afterRepeatedHourSameDay = new Temporal.ZonedDateTime(
  972849600_000_000_000n /* = 2000-10-29T20Z */,
  "America/Vancouver"); /* = 2000-10-29T12-08 in local time */
const beforeRepeatedHour = new Temporal.ZonedDateTime(
  972716400_000_000_000n /* = 2000-10-28T07Z */,
  "America/Vancouver"); /* = 2000-10-28T00-07 in local time */

assert.sameValue(hours25.total({
  unit: "days",
  relativeTo: inRepeatedHour
}), 1, "start inside repeated hour, end after: 25 hours = 1 day");

assert.sameValue(oneDay.total({
  unit: "hours",
  relativeTo: inRepeatedHour
}), 25, "start inside repeated hour, end after: 1 day = 25 hours");

assert.sameValue(hours25Neg.total({
  unit: "days",
  relativeTo: oneDayAfterRepeatedHour
}), -1, "start after repeated hour, end inside: -25 hours = 1 day");

assert.sameValue(oneDayNeg.total({
  unit: "hours",
  relativeTo: oneDayAfterRepeatedHour
}), -25, "start after repeated hour, end inside: -1 day = -25 hours");

assert.sameValue(hours25.total({
  unit: "days",
  relativeTo: beforeSkippedHour
}), 24 / 23, "start in normal hour, end in skipped hour: 25 hours = 1 1/23 day");

assert.sameValue(oneDay.total({
  unit: "hours",
  relativeTo: beforeSkippedHour
}), 24, "start in normal hour, end in skipped hour: 1 day = 24 hours");

assert.sameValue(hours25.total({
  unit: "days",
  relativeTo: skippedHourDay
}), 13 / 12, "start before skipped hour, end >1 day after: 25 hours = 1 2/24 day");

assert.sameValue(oneDay.total({
  unit: "hours",
  relativeTo: skippedHourDay
}), 23, "start before skipped hour, end >1 day after: 1 day = 23 hours");

assert.sameValue(hours25Neg.total({
  unit: "days",
  relativeTo: dayAfterSkippedHour
}), -13 / 12, "start after skipped hour, end >1 day before: -25 hours = -1 2/24 day");

assert.sameValue(oneDayNeg.total({
  unit: "hours",
  relativeTo: dayAfterSkippedHour
}), -23, "start after skipped hour, end >1 day before: -1 day = -23 hours");

assert.sameValue(hours12.total({
  unit: "days",
  relativeTo: skippedHourDay
}), 12 / 23, "start before skipped hour, end <1 day after: 12 hours = 12/23 days");

assert.sameValue(hours12Neg.total({
  unit: "days",
  relativeTo: afterSkippedHour
}), -12 / 23, "start after skipped hour, end <1 day before: -12 hours = -12/23 days");

assert.sameValue(hours25.total({
  unit: "days",
  relativeTo: repeatedHourDay
}), 1, "start before repeated hour, end >1 day after: 25 hours = 1 day");

assert.sameValue(oneDay.total({
  unit: "hours",
  relativeTo: repeatedHourDay
}), 25, "start before repeated hour, end >1 day after: 1 day = 25 hours");

assert.sameValue(hours25Neg.total({
  unit: "days",
  relativeTo: afterRepeatedHour
}), -1, "start after repeated hour, end >1 day before: -25 hours = -1 day");

assert.sameValue(oneDayNeg.total({
  unit: "hours",
  relativeTo: afterRepeatedHour
}), -25, "start after repeated hour, end >1 day before: -1 day = -25 hours");

assert.sameValue(hours12.total({
  unit: "days",
  relativeTo: repeatedHourDay
}), 12 / 25, "start before repeated hour, end <1 day after: 12 hours = 12/25 days");

assert.sameValue(hours12Neg.total({
  unit: "days",
  relativeTo: afterRepeatedHourSameDay
}), -12 / 25, "start after repeated hour, end <1 day before: -12 hours = -12/25 days");

assert.sameValue(hours48.total({
  unit: "days",
  relativeTo: beforeRepeatedHour
}), 49 / 25, "start before repeated hour, end after: 48 hours = 1 24/25 days");

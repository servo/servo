// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
    Balancing the resulting duration takes the time zone's UTC offset shifts
    into account
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// Vancouver spring time line change: 2:00AM 2 Apr 2000 -> 3:00AM 2 Apr 2000.
const beforeDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-02T01:00:00-08:00[America/Vancouver]");
const afterDstStart03_00 = Temporal.ZonedDateTime.from("2000-04-02T03:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart01_00.until(afterDstStart03_00, { largestUnit: "hours" }),
  0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
  "1:00 day DST start ->  3:00 day DST start = PT1H");

const afterDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-02T03:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart01_00.until(afterDstStart03_30, { largestUnit: "hours" }),
  0, 0, 0, 0, 1, 30, 0, 0, 0, 0,
  "1:00 day DST start ->  3:30 day DST start = PT1H30M");

const dayAfterDstStart02_00 = Temporal.ZonedDateTime.from("2000-04-03T02:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart01_00.until(dayAfterDstStart02_00, { largestUnit: "days" }),
  0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
  "1:00AM day DST starts -> 2:00AM day after DST starts = P1DT1H.");

const beforeDstStart00_00 = Temporal.ZonedDateTime.from("2000-04-02T00:00:00-08:00[America/Vancouver]");
const dayAfterDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-03T01:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart00_00.until(dayAfterDstStart01_00, { largestUnit: "days" }),
  0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
  "12:00AM day DST starts -> 1:00AM day after DST starts = P1DT1H.");

const beforeDstStart01_30 = Temporal.ZonedDateTime.from("2000-04-02T01:30:00-08:00[America/Vancouver]");
const afterDstStart04_30 = Temporal.ZonedDateTime.from("2000-04-02T04:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart01_30.until(afterDstStart04_30, { largestUnit: "days" }),
  0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
  "1:30 day DST starts -> 4:30 day DST starts = PT2H.");

const beforeDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-01T03:30:00-08:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart03_30.until(afterDstStart03_30, { largestUnit: "days" }),
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "3:30 day before DST start -> 3:30 day of DST start = P1D.");

const beforeDstStart02_30 = Temporal.ZonedDateTime.from("2000-04-01T02:30:00-08:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart02_30.until(afterDstStart03_30, { largestUnit: "days" }),
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "2:30 day before DST start -> 3:30 day of DST start = P1D.");

const beforeDstStart02_00 = Temporal.ZonedDateTime.from("2000-04-01T02:00:00-08:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  beforeDstStart02_00.until(afterDstStart03_00, { largestUnit: "days" }),
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "2:00 day before DST starts -> 3:00 day DST starts = P1D.");

// Based on a test case by Adam Shaw
const start = new Temporal.ZonedDateTime(
    949132800_000_000_000n /* = 2000-01-29T08Z */,
    "America/Vancouver"); /* = 2000-01-29T00-08 in local time */
const end = new Temporal.ZonedDateTime(
    972889200_000_000_000n /* = 2000-10-30T07Z */,
    "America/Vancouver"); /* = 2000-10-29T23-08 in local time */

const duration = start.until(end, { largestUnit: "years" });
TemporalHelpers.assertDuration(duration, 0, 9, 0, 0, 24, 0, 0, 0, 0, 0,
    "24 hours does not balance to 1 day in 25-hour day");

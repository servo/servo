// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Test behaviour around DST boundaries
features: [Temporal]
includes: [temporalHelpers.js]
---*/

// Vancouver spring time line change: 2:00AM 2 Apr 2000 -> 3:00AM 2 Apr 2000.
const beforeDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-02T01:00:00-08:00[America/Vancouver]");
const afterDstStart03_00 = Temporal.ZonedDateTime.from("2000-04-02T03:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  afterDstStart03_00.since(beforeDstStart01_00, { largestUnit: "hours" }),
  0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
  "1:00 day DST start ->  3:00 day DST start = PT1H");

const afterDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-02T03:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  afterDstStart03_30.since(beforeDstStart01_00, { largestUnit: "hours" }),
  0, 0, 0, 0, 1, 30, 0, 0, 0, 0,
  "1:00 day DST start ->  3:30 day DST start = PT1H30M");

const dayAfterDstStart02_00 = Temporal.ZonedDateTime.from("2000-04-03T02:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  dayAfterDstStart02_00.since(beforeDstStart01_00, { largestUnit: "days" }),
  0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
  "1:00AM day DST starts -> 2:00AM day after DST starts = P1DT1H.");

const beforeDstStart00_00 = Temporal.ZonedDateTime.from("2000-04-02T00:00:00-08:00[America/Vancouver]");
const dayAfterDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-03T01:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  dayAfterDstStart01_00.since(beforeDstStart00_00, { largestUnit: "days" }),
  0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
  "12:00AM day DST starts -> 1:00AM day after DST starts = P1DT1H.");

const beforeDstStart01_30 = Temporal.ZonedDateTime.from("2000-04-02T01:30:00-08:00[America/Vancouver]");
const afterDstStart04_30 = Temporal.ZonedDateTime.from("2000-04-02T04:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  afterDstStart04_30.since(beforeDstStart01_30, { largestUnit: "days" }),
  0, 0, 0, 0, 2, 0, 0, 0, 0, 0,
  "1:30 day DST starts -> 4:30 day DST starts = PT2H.");

const beforeDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-01T03:30:00-08:00[America/Vancouver]");
TemporalHelpers.assertDuration(
  afterDstStart03_30.since(beforeDstStart03_30, { largestUnit: "days" }),
  0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
  "3:30 day before DST start -> 3:30 day of DST start = P1D.");

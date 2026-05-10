// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Test behaviour around DST boundaries
features: [Temporal]
includes: [temporalHelpers.js]
---*/

// Samoa date line change (add): 22:00 29 Dec 2011 -> 23:00 31 Dec 2011.
const SamoaDateChangeStart = new Temporal.PlainDateTime(2011, 12, 29, 22).toZonedDateTime("Pacific/Apia");
const SamoaDateChangeEnd = new Temporal.PlainDateTime(2011, 12, 31, 23).toZonedDateTime("Pacific/Apia");
let result = SamoaDateChangeEnd.subtract({
  days: 1,
  hours: 1
});
assert.sameValue(result.day, 31,
  "Day result: Samoa date line change, subtract 1 day and 1 hour.");
assert.sameValue(result.hour, 22,
  "Hour result: Samoa date line change, subtract 1 day and 1 hour.");
assert.sameValue(result.minute, 0,
  "Minute result: Samoa date line change, subtract 1 day and 1 hour.");

result = SamoaDateChangeEnd.subtract({
  days: 2,
  hours: 1
});
assert.sameValue(result.day, 29,
  "Day result: Samoa date line change, subtract 2 days and 1 hour.");
assert.sameValue(result.hour, 22,
  "Hour result: Samoa date line change, subtract 2 days and 1 hour.");
assert.sameValue(result.minute, 0,
  "Minute result: Samoa date line change, subtract 2 days and 1 hour.");

// Vancouver spring time line change: 2:00AM 2 Apr 2000 -> 3:00AM 2 Apr 2000.
const beforeDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-02T01:00:00-08:00[America/Vancouver]");
const afterDstStart03_00 = Temporal.ZonedDateTime.from("2000-04-02T03:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_00,
  afterDstStart03_00.subtract({ hours: 1 }),
  "1:00 day DST start  ->  3:00 day DST start - 1 hour");

const afterDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-02T03:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_00,
  afterDstStart03_30.subtract({
    hours: 1,
    minutes: 30
  }),
  "1:00 day DST start -> 3:30 day of DST start - 1.5 hours.");

const dayAfterDstStart02_00 = Temporal.ZonedDateTime.from("2000-04-03T02:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_00,
  dayAfterDstStart02_00.subtract({ hours: 24 }),
  "1:00AM day DST starts -> 2:00AM day after DST starts - 24 hours.");

const beforeDstStart00_00 = Temporal.ZonedDateTime.from("2000-04-02T00:00:00-08:00[America/Vancouver]");
const dayAfterDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-03T01:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart00_00,
  dayAfterDstStart01_00.subtract({ hours: 24 }),
  "12:00AM day DST starts -> 1:00AM day after DST starts - 24 hours.");

const beforeDstStart01_30 = Temporal.ZonedDateTime.from("2000-04-02T01:30:00-08:00[America/Vancouver]");
const afterDstStart04_30 = Temporal.ZonedDateTime.from("2000-04-02T04:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_30,
  afterDstStart04_30.subtract({ hours: 2 }),
  "1:30 day DST starts -> 4:30 day DST starts - 2 hours.");

const beforeDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-01T03:30:00-08:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart03_30,
  afterDstStart03_30.subtract({ days: 1 }),
  "3:30 day before DST start -> 3:30 day of DST start - 1 day.");

// Vancouver fall time line change: 2:00AM 29 Oct 2000 -> 1:00AM 29 Oct 2000.

const twoDaysBeforeDstEnd00_45 = Temporal.ZonedDateTime.from("2000-10-27T00:45:00-07:00[America/Vancouver]");
const dayAfterDstEnd01_15 = Temporal.ZonedDateTime.from("2000-10-30T01:15:00-08:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  twoDaysBeforeDstEnd00_45,
  dayAfterDstEnd01_15.subtract({
    days: 2,
    hours: 24,
    minutes: 30
  }),
  "0:45 two days before DST end + 2 days 24 hours 30 minutes -> 1:15 day after DST end.");

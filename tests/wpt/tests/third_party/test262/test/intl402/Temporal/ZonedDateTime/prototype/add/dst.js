// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Test behaviour around DST boundaries
features: [Temporal]
includes: [temporalHelpers.js]
---*/

// Samoa date line change (add): 22:00 29 Dec 2011 -> 23:00 31 Dec 2011.
const SamoaDateChangeStart = new Temporal.PlainDateTime(2011, 12, 29, 22).toZonedDateTime("Pacific/Apia");
let result = SamoaDateChangeStart.add({
  days: 1,
  hours: 1
});
assert.sameValue(result.day, 31,
  "Day result: Samoa date line change, add 1 day and 1 hour.");
assert.sameValue(result.hour, 23,
  "Hour result: Samoa date line change, add 1 day and 1 hour.");
assert.sameValue(result.minute, 0,
  "Minute result: Samoa date line change, add 1 day and 1 hour.");

// Vancouver spring time line change: 2:00AM 2 Apr 2000 -> 3:00AM 2 Apr 2000.
const beforeDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-02T01:00:00-08:00[America/Vancouver]");
const afterDstStart03_00 = Temporal.ZonedDateTime.from("2000-04-02T03:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_00.add({ hours: 1 }),
  afterDstStart03_00,
  "1:00 day DST start + 1 hour ->  3:00 day DST start");

const afterDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-02T03:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_00.add({
    hours: 1,
    minutes: 30
  }),
  afterDstStart03_30,
  "1:00 day DST start + 1.5 hours -> 3:30 day of DST start.");

const dayAfterDstStart02_00 = Temporal.ZonedDateTime.from("2000-04-03T02:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_00.add({ hours: 24 }),
  dayAfterDstStart02_00,
  "1:00AM day DST starts -> (add 24 hours) -> 2:00AM day after DST starts.");

const beforeDstStart00_00 = Temporal.ZonedDateTime.from("2000-04-02T00:00:00-08:00[America/Vancouver]");
const dayAfterDstStart01_00 = Temporal.ZonedDateTime.from("2000-04-03T01:00:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart00_00.add({ hours: 24 }),
  dayAfterDstStart01_00,
  "12:00AM day DST starts + 24 hours -> 1:00AM day after DST starts.");

const beforeDstStart01_30 = Temporal.ZonedDateTime.from("2000-04-02T01:30:00-08:00[America/Vancouver]");
const afterDstStart04_30 = Temporal.ZonedDateTime.from("2000-04-02T04:30:00-07:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart01_30.add({ hours: 2 }),
  afterDstStart04_30,
  "1:30 day DST starts + 2 hours -> 4:30 day DST starts.");

const beforeDstStart03_30 = Temporal.ZonedDateTime.from("2000-04-01T03:30:00-08:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart03_30.add({ days: 1 }),
  afterDstStart03_30,
  "3:30 day before DST start + 1 day -> 3:30 day of DST start.");

const beforeDstStart02_30 = Temporal.ZonedDateTime.from("2000-04-01T02:30:00-08:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart02_30.add({ days: 1 }),
  afterDstStart03_30,
  "2:30 day before DST start + 1 day -> 3:30 day of DST start.");

const beforeDstStart02_00 = Temporal.ZonedDateTime.from("2000-04-01T02:00:00-08:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  beforeDstStart02_00.add({ days: 1 }),
  afterDstStart03_00,
  "2:00 day before DST starts + 1 day -> 3:00 day DST starts.");

// Vancouver fall time line change: 2:00AM 29 Oct 2000 -> 1:00AM 29 Oct 2000.

const twoDaysBeforeDstEnd01_45 = Temporal.ZonedDateTime.from("2000-10-27T01:45:00-07:00[America/Vancouver]");
const dayAfterDstEnd01_15 = Temporal.ZonedDateTime.from("2000-10-30T01:15:00-08:00[America/Vancouver]");
TemporalHelpers.assertZonedDateTimesEqual(
  twoDaysBeforeDstEnd01_45.add({
    days: 2,
    hours: 24,
    minutes: 30
  }),
  dayAfterDstEnd01_15,
  "1:45 two days before DST end + 2 days 24 hours 30 minutes -> 1:15 day after DST end.");

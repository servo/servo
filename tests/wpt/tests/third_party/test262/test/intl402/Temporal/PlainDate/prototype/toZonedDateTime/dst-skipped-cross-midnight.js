// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Test TZDB edge case where start of day is not 00:00 nor 01:00
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// DST spring-forward hour skipped from 1919-03-30T23:30 to 1919-03-31T00:30, so
// day starts at 00:30
const instance = new Temporal.PlainDate(1919, 3, 31);
const startOfDay = instance.toZonedDateTime("America/Toronto");
const midnightDisambiguated = instance.toZonedDateTime({
  timeZone: "America/Toronto",
  plainTime: new Temporal.PlainTime(),
});
TemporalHelpers.assertDuration(
  startOfDay.until(midnightDisambiguated),
  0, 0, 0, 0, 0, /* minutes = */ 30, 0, 0, 0, 0,
  "start of day is 30 minutes earlier than following the disambiguation strategy for midnight"
);

assert.sameValue(
  startOfDay.epochNanoseconds,
  instance.toZonedDateTime({ timeZone: "America/Toronto" }).epochNanoseconds,
  "omitted plainTime is the same result as using the string shorthand"
);
assert.sameValue(
  startOfDay.epochNanoseconds,
  instance.toZonedDateTime({ timeZone: "America/Toronto", plainTime: undefined }).epochNanoseconds,
  "explicitly undefined plainTime is the same result as using the string shorthand"
);

// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-temporal.zoneddatetime.prototype.hoursinday
description: Test TZDB edge case where start of day is not 00:00 nor 01:00
features: [Temporal]
---*/

// DST spring-forward hour skipped at 1919-03-30T23:30 (23.5 hour day)
const dayBefore = Temporal.ZonedDateTime.from({
  year: 1919,
  month: 3,
  day: 30,
  hour: 12,
  timeZone: "America/Toronto",
});
assert.sameValue(dayBefore.hoursInDay, 23.5, "1919-03-30 had 23.5 hours in America/Toronto");

// Following day was also 23.5 hours
const dayAfter = Temporal.ZonedDateTime.from({
  year: 1919,
  month: 3,
  day: 31,
  hour: 12,
  timeZone: "America/Toronto",
});
assert.sameValue(dayAfter.hoursInDay, 23.5, "1919-03-31 had 23.5 hours in America/Toronto");

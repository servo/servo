// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-temporal.zoneddatetime.prototype.startofday
description: Test TZDB edge case where start of day is not 00:00 nor 01:00
features: [Temporal]
---*/

// DST spring-forward hour skipped at 1919-03-30T23:30, so the following day
// started at 00:30
const instance = Temporal.ZonedDateTime.from({
  year: 1919,
  month: 3,
  day: 31,
  hour: 12,
  timeZone: "America/Toronto",
});
const result = instance.startOfDay();
assert.sameValue(result.hour, 0, "1919-03-31 started at hour 0");
assert.sameValue(result.minute, 30, "1919-03-31 started at minute 30");

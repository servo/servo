// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.hoursinday
description: Test hoursInDay for DST changes at midnight
features: [Temporal]
---*/

const fall = Temporal.ZonedDateTime.from({
  year: 2018,
  month: 2,
  day: 17,
  hour: 12,
  timeZone: "America/Sao_Paulo",
});
assert.sameValue(fall.hoursInDay, 25, "25-hour day with backward jump at midnight");

const spring = Temporal.ZonedDateTime.from({
  year: 2018,
  month: 11,
  day: 4,
  hour: 12,
  timeZone: "America/Sao_Paulo",
});
assert.sameValue(spring.hoursInDay, 23, "23-hour day with forward jump at midnight");

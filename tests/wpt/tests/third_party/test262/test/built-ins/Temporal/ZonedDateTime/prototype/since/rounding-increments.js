// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Rounds to various increments.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

/*
const earlier = Temporal.ZonedDateTime.from('2019-01-08T09:22:36.123456789+01:00[+01:00]');
const later = Temporal.ZonedDateTime.from('2021-09-07T13:39:40.987654321+01:00[+01:00]');
*/
const earlier = new Temporal.ZonedDateTime(1546935756123456789n, "+01:00");
const later = new Temporal.ZonedDateTime(1631018380987654321n, "+01:00");

// rounds to an increment of hours
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "hours",
  roundingIncrement: 3,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 23355, 0, 0, 0, 0, 0);

// rounds to an increment of minutes
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "minutes",
  roundingIncrement: 30,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 23356, 30, 0, 0, 0, 0);

// rounds to an increment of seconds
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "seconds",
  roundingIncrement: 15,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 23356, 17, 0, 0, 0, 0);

// rounds to an increment of milliseconds
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "milliseconds",
  roundingIncrement: 10,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 23356, 17, 4, 860, 0, 0);

// rounds to an increment of microseconds
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "microseconds",
  roundingIncrement: 10,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 23356, 17, 4, 864, 200, 0);

// rounds to an increment of nanoseconds
TemporalHelpers.assertDuration(later.since(earlier, {
  smallestUnit: "nanoseconds",
  roundingIncrement: 10,
  roundingMode: "halfExpand"
}), 0, 0, 0, 0, 23356, 17, 4, 864, 197, 530);

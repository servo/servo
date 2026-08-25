// Copyright (C) 2025 Adam Shaw. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Returns a simple nanosecond time-duration when ISO year-month-day is same-day
  and wall-clock delta direction is the reverse of the epoch-nanosecond direction
includes: [temporalHelpers.js]
features: [Temporal]
---*/

/*
Addresses https://github.com/tc39/proposal-temporal/issues/3141
*/

TemporalHelpers.assertDuration(
  Temporal.ZonedDateTime
    .from('2025-11-02T01:00:00-08:00[America/Vancouver]') // later
    .until('2025-11-02T01:01:00-07:00[America/Vancouver]', { // earlier
      largestUnit: 'year',
      smallestUnit: 'nanosecond',
    }),
  0, 0, 0, 0, 0, /* minutes = */ -59, 0, 0, 0, 0,
  'same-day, negative epoch-nanoseconds delta, positive wall-clock delta',
)

TemporalHelpers.assertDuration(
  Temporal.ZonedDateTime
    .from('2025-11-02T01:01:00-07:00[America/Vancouver]') // earlier
    .until('2025-11-02T01:00:00-08:00[America/Vancouver]', { // later
      largestUnit: 'year',
      smallestUnit: 'nanosecond',
    }),
  0, 0, 0, 0, 0, /* minutes = */ 59, 0, 0, 0, 0,
  'same-day, positive epoch-nanoseconds delta, negative wall-clock delta',
)

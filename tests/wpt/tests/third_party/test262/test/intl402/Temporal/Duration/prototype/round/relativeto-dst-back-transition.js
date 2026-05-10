// Copyright (C) 2025 Adam Shaw. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Ensures correct rounding results when relativeTo is within the second wallclock occurence of a
  DST fall-back transition.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

/*
Addresses https://github.com/tc39/proposal-temporal/issues/3149
(NudgeToCalendarUnit wrong span)
*/
{
  // the second 01:00 time in the DST transition. 01:00-07:00 happens just before this
  const origin = Temporal.ZonedDateTime.from('2025-11-02T01:00:00-08:00[America/Vancouver]');
  const dur = Temporal.Duration.from({ hours: 11, minutes: 30 });
  const roundedDur = dur.round({
    largestUnit: 'days',
    smallestUnit: 'days',
    relativeTo: origin,
    roundingMode: 'halfExpand',
  });
  TemporalHelpers.assertDuration(
    roundedDur,
    0, 0, 0, /* days = */ 0, 0, 0, 0, 0, 0, 0,
    'relativeTo in fall-back DST transition, second wallclock time, assumed 24 hour span when +1 day',
  );
}

/*
Addresses https://github.com/tc39/proposal-temporal/issues/3149
(NudgeToCalendarUnit wrong span)
*/
{
  // the second 01:00 time in the DST transition. 01:00-07:00 happens just before this
  const origin = Temporal.ZonedDateTime.from('2025-11-02T01:00:00-08:00[America/Vancouver]');
  const dur = Temporal.Duration.from({ hours: -12, minutes: -30 });
  const roundedDur = dur.round({
    largestUnit: 'days',
    smallestUnit: 'days',
    relativeTo: origin,
    roundingMode: 'halfExpand',
  });
  TemporalHelpers.assertDuration(
    roundedDur,
    0, 0, 0, /* days = */ -1, 0, 0, 0, 0, 0, 0,
    'relativeTo in fall-back DST transition, second wallclock time, assumed 25 hour span when -1 day',
  );
}

/*
Related to https://github.com/tc39/proposal-temporal/issues/3141
(DifferenceZonedDateTime assertion)
*/
TemporalHelpers.assertDuration(
  Temporal.Duration.from({ minutes: -59 }).round({
    smallestUnit: 'days',
    relativeTo: '2025-11-02T01:00:00-08:00[America/Vancouver]',
  }),
  0, 0, 0, /* days = */ 0, 0, 0, 0, 0, 0, 0,
  'negative delta from relativeTo, positive wallclock delta',
);

/*
Related to https://github.com/tc39/proposal-temporal/issues/3141
(DifferenceZonedDateTime assertion)
*/
TemporalHelpers.assertDuration(
  Temporal.Duration.from({ minutes: -59 }).round({
    smallestUnit: 'days',
    relativeTo: '2025-11-02T01:00:00-08:00[America/Vancouver]',
    roundingMode: 'expand',
  }),
  0, 0, 0, /* days = */ -1, 0, 0, 0, 0, 0, 0,
  'negative delta from relativeTo, positive wallclock delta, expanding',
);

/*
Related to https://github.com/tc39/proposal-temporal/issues/3141
(DifferenceZonedDateTime assertion)
*/
TemporalHelpers.assertDuration(
  Temporal.Duration.from({ minutes: 59 }).round({
    smallestUnit: 'days',
    relativeTo: '2025-11-02T01:01:00-07:00[America/Vancouver]',
  }),
  0, 0, 0, /* days = */ 0, 0, 0, 0, 0, 0, 0,
  'positive delta from relativeTo, negative wallclock delta',
);

/*
Related to https://github.com/tc39/proposal-temporal/issues/3141
(DifferenceZonedDateTime assertion)
*/
TemporalHelpers.assertDuration(
  Temporal.Duration.from({ minutes: 59 }).round({
    smallestUnit: 'days',
    relativeTo: '2025-11-02T01:01:00-07:00[America/Vancouver]',
    roundingMode: 'expand',
  }),
  0, 0, 0, /* days = */ 1, 0, 0, 0, 0, 0, 0,
  'positive delta from relativeTo, negative wallclock delta, expanding',
);

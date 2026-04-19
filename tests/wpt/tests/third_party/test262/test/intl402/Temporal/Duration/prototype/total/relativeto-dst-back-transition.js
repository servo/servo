// Copyright (C) 2025 Adam Shaw. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Ensures correct total results when relativeTo is within the second wallclock occurence of a
  DST fall-back transition.
features: [Temporal]
---*/

/*
Addresses https://github.com/tc39/proposal-temporal/issues/3148
(NudgeToCalendarUnit wrong span)
*/
{
  const origin = Temporal.ZonedDateTime.from('2025-11-02T01:00:00-08:00[America/Vancouver]');
  const dur = Temporal.Duration.from({ hours: 2 });
  const total = dur.total({ unit: 'days', relativeTo: origin });
  assert.sameValue(
    total,
    2 / 24,
    'relativeTo in fall-back DST transition, second wallclock time, assumed 24 hour span when +1 day',
  );
}

/*
Addresses https://github.com/tc39/proposal-temporal/issues/3148
(NudgeToCalendarUnit wrong span)
*/
{
  const origin = Temporal.ZonedDateTime.from('2025-11-02T01:00:00-08:00[America/Vancouver]');
  const dur = Temporal.Duration.from({ hours: -2 });
  const total = dur.total({ unit: 'days', relativeTo: origin });
  assert.sameValue(
    total,
    -2 / 25,
    'relativeTo in fall-back DST transition, second wallclock time, assumed 25 hour span when -1 day',
  );
}

/*
Related to https://github.com/tc39/proposal-temporal/issues/3141
(DifferenceZonedDateTime assertion)
*/
assert.sameValue(
  Temporal.Duration.from({ minutes: -59 }).total({
    unit: 'days',
    relativeTo: '2025-11-02T01:00:00-08:00[America/Vancouver]',
  }),
  -59 / (60 * 25), // 25 hour span when adding -1 day to relativeTo
  'negative delta from relativeTo, positive wallclock delta',
);

/*
Related to https://github.com/tc39/proposal-temporal/issues/3141
(DifferenceZonedDateTime assertion)
*/
assert.sameValue(
  Temporal.Duration.from({ minutes: 59 }).total({
    unit: 'days',
    relativeTo: '2025-11-02T01:01:00-07:00[America/Vancouver]',
  }),
  59 / (60 * 25), // 25 hour span when adding +1 day to relativeTo
  'positive delta from relativeTo, negative wallclock delta',
);

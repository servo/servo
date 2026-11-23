// Copyright (C) 2026 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Duration with an overly large time component total relative to a property bag
  with a time zone
info: |
  GetTemporalRelativeToOption ( _options_ )

  12. Return the Record { [[PlainRelativeTo]]: *undefined*,
    [[ZonedRelativeTo]]: _zonedRelativeTo_ }.

  Temporal.Duration.prototype.total ( _totalOf_ )

  12.e. Let _targetEpochNs_ be ? AddZonedDateTime(_relativeEpochNs_, _timeZone_,
    _calendar_, _internalDuration_, ~constrain~).
features: [Temporal]
---*/

// A property bag with a time zone yields a [[ZonedRelativeTo]] just as a
// Temporal.ZonedDateTime or a zoned ISO string does, so the epoch limits apply
// to it identically.
const relativeTo = { year: 2000, month: 1, day: 1, timeZone: "UTC" };

const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, Number.MAX_SAFE_INTEGER);

[
  "nanosecond",
  "second",
  "hour",
  "day",
  "week",
  "month",
  "year",
].forEach((unit) => {
  assert.throws(RangeError, () => d.total({ unit, relativeTo }), `unit ${unit}`);
});

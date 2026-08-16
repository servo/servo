// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Duration with an overly large time component total of a calendar unit relative
  to a ZonedDateTime
info: |
  12.e. Let _targetEpochNs_ be ? AddZonedDateTime(_relativeEpochNs_, _timeZone_,
    _calendar_, _internalDuration_, ~constrain~).
features: [Temporal]
---*/

const relativeTo = new Temporal.ZonedDateTime(0n, "UTC");

[Number.MIN_SAFE_INTEGER, Number.MAX_SAFE_INTEGER].forEach((seconds) => {
  const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, seconds);
  assert.throws(RangeError, () => d.total({ unit: "year", relativeTo }));
  assert.throws(RangeError, () => d.total({ unit: "month", relativeTo }));
  assert.throws(RangeError, () => d.total({ unit: "week", relativeTo }));
});

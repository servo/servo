// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Duration with an overly large time component total of a calendar unit relative
  to a PlainDate
info: |
  13.d. Let _dateDuration_ be ? AdjustDateDurationRecord(
    _internalDuration_.[[Date]], _targetTime_.[[Days]]).
features: [Temporal]
---*/

const relativeTo = new Temporal.PlainDate(2020, 1, 1);

[Number.MIN_SAFE_INTEGER, Number.MAX_SAFE_INTEGER].forEach((seconds) => {
  const d = new Temporal.Duration(0, 0, 0, 0, 0, 0, seconds);
  assert.throws(RangeError, () => d.total({ unit: "year", relativeTo }));
  assert.throws(RangeError, () => d.total({ unit: "month", relativeTo }));
  assert.throws(RangeError, () => d.total({ unit: "week", relativeTo }));
});

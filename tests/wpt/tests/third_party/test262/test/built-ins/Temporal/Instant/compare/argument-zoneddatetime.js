// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Fast path for converting Temporal.ZonedDateTime to Temporal.Instant
info: |
    sec-temporal.instant.compare steps 1–2:
      1. Set _one_ to ? ToTemporalInstant(_one_).
      2. Set _two_ to ? ToTemporalInstant(_two_).
    sec-temporal-totemporalinstant step 1.b:
      b. If _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        i. Return ! CreateTemporalInstant(_item_.[[Nanoseconds]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const instant = new Temporal.Instant(1_000_000_000_000_000_000n);

TemporalHelpers.checkToTemporalInstantFastPath((datetime) => {
  const result = Temporal.Instant.compare(datetime, instant);
  assert.sameValue(result, 1, "comparison result");
});

TemporalHelpers.checkToTemporalInstantFastPath((datetime) => {
  const result = Temporal.Instant.compare(instant, datetime);
  assert.sameValue(result, -1, "comparison result");
});

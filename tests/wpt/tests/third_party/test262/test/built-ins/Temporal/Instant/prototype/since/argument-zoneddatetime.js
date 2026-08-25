// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Fast path for converting Temporal.ZonedDateTime to Temporal.Instant
info: |
    sec-temporal.instant.prototype.since step 3:
      3. Set _other_ to ? ToTemporalInstant(_other_).
    sec-temporal-totemporalinstant step 1.b:
      b. If _item_ has an [[InitializedTemporalZonedDateTime]] internal slot, then
        i. Return ! CreateTemporalInstant(_item_.[[Nanoseconds]]).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkToTemporalInstantFastPath((datetime) => {
  const instant = new Temporal.Instant(1_000_000_000_000_000_000n);
  const result = instant.since(datetime);
  assert.sameValue(result.total({ unit: "nanoseconds" }), -987654321, "nanoseconds result");
});

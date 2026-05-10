// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.zoneddatetime.prototype.since step 13:
      13. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, _maximum_, *false*).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");
const later = new Temporal.ZonedDateTime(1_000_090_061_988_655_322n, "UTC");

const explicit = later.since(earlier, { roundingIncrement: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 0, 25, 1, 1, 1, 1, 1, "default roundingIncrement is 1");

const implicit = later.since(earlier, {});
TemporalHelpers.assertDuration(implicit, 0, 0, 0, 0, 25, 1, 1, 1, 1, 1, "default roundingIncrement is 1");

const lambda = later.since(earlier, () => {});
TemporalHelpers.assertDuration(lambda, 0, 0, 0, 0, 25, 1, 1, 1, 1, 1, "default roundingIncrement is 1");

// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.plainyearmonth.prototype.since step 13:
      13. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, *undefined*, *false*).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainYearMonth(2000, 5);
const later = new Temporal.PlainYearMonth(2001, 6);

const explicit = later.since(earlier, { roundingIncrement: undefined });
TemporalHelpers.assertDuration(explicit, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default roundingIncrement is 1");

const implicit = later.since(earlier, {});
TemporalHelpers.assertDuration(implicit, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default roundingIncrement is 1");

const lambda = later.since(earlier, () => {});
TemporalHelpers.assertDuration(lambda, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, "default roundingIncrement is 1");

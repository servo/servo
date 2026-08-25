// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal.plaindate.prototype.until step 11:
      11. Let _roundingIncrement_ be ? ToTemporalRoundingIncrement(_options_, *undefined*, *false*).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const earlier = new Temporal.PlainDate(2000, 5, 2);
const later = new Temporal.PlainDate(2000, 5, 7);

const explicit = earlier.until(later, { roundingIncrement: undefined });
TemporalHelpers.assertDuration(explicit, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, "default roundingIncrement is 1");

// See options-object.js for {} and () => {}

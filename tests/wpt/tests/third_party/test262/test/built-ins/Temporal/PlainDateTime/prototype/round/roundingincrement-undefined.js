// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Fallback value for roundingIncrement option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporalroundingincrement step 5:
      5. Let _increment_ be ? GetOption(_normalizedOptions_, *"roundingIncrement"*, « Number », *undefined*, 1).
    sec-temporal-totemporaldatetimeroundingincrement step 5:
      5. Return ? ToTemporalRoundingIncrement(_normalizedOptions_, _maximum_, *false*).
    sec-temporal.plaindatetime.prototype.round step 8:
      8. Let _roundingIncrement_ be ? ToTemporalDateTimeRoundingIncrement(_options_, _smallestUnit_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

const explicit = datetime.round({ smallestUnit: 'second', roundingIncrement: undefined });
TemporalHelpers.assertPlainDateTime(explicit, 2000, 5, "M05", 2, 12, 34, 57, 0, 0, 0, "default roundingIncrement is 1");

const implicit = datetime.round({ smallestUnit: 'second' });
TemporalHelpers.assertPlainDateTime(implicit, 2000, 5, "M05", 2, 12, 34, 57, 0, 0, 0, "default roundingIncrement is 1");

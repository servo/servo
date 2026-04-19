// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.plaintime.prototype.with step 11:
      11. Let _overflow_ be ? ToTemporalOverflow(_options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12);
const explicit = time.with({ minute: 67 }, { overflow: undefined });
TemporalHelpers.assertPlainTime(explicit, 12, 59, 0, 0, 0, 0, "default overflow is constrain");
const implicit = time.with({ minute: 67 }, {});
TemporalHelpers.assertPlainTime(implicit, 12, 59, 0, 0, 0, 0, "default overflow is constrain");
const lambda = time.with({ minute: 67 }, () => {});
TemporalHelpers.assertPlainTime(lambda, 12, 59, 0, 0, 0, 0, "default overflow is constrain");

// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isodatefromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plaindate.prototype.with step 16:
      16. Return ? DateFromFields(_calendar_, _fields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
const explicit = date.with({ month: 15 }, { overflow: undefined });
TemporalHelpers.assertPlainDate(explicit, 2000, 12, "M12", 2, "default overflow is constrain");
const implicit = date.with({ month: 15 }, {});
TemporalHelpers.assertPlainDate(implicit, 2000, 12, "M12", 2, "default overflow is constrain");
const fun = date.with({ month: 15 }, () => {});
TemporalHelpers.assertPlainDate(fun, 2000, 12, "M12", 2, "default overflow is constrain");

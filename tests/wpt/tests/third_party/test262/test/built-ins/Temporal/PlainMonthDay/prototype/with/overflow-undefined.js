// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isomonthdayfromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plainmonthday.prototype.with step 16:
      16. Return ? MonthDayFromFields(_calendar_, _fields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const monthday = new Temporal.PlainMonthDay(5, 2);
const explicit = monthday.with({ day: 33 }, { overflow: undefined });
TemporalHelpers.assertPlainMonthDay(explicit, "M05", 31, "default overflow is constrain");
const implicit = monthday.with({ day: 33 }, {});
TemporalHelpers.assertPlainMonthDay(implicit, "M05", 31, "default overflow is constrain");
const lambda = monthday.with({ day: 33 }, () => {});
TemporalHelpers.assertPlainMonthDay(lambda, "M05", 31, "default overflow is constrain");

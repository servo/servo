// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-interprettemporaldatetimefields steps 2–3:
      2. Let _temporalDate_ be ? DateFromFields(_calendar_, _fields_, _options_).
      3. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plaindatetime.prototype.with step 16:
      16. Let _result_ be ? InterpretTemporalDateTimeFields(_calendar_, _fields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12);
const explicit = datetime.with({ minute: 67 }, { overflow: undefined });
TemporalHelpers.assertPlainDateTime(explicit, 2000, 5, "M05", 2, 12, 59, 0, 0, 0, 0, "default overflow is constrain");
const implicit = datetime.with({ minute: 67 }, {});
TemporalHelpers.assertPlainDateTime(implicit, 2000, 5, "M05", 2, 12, 59, 0, 0, 0, 0, "default overflow is constrain");
const lambda = datetime.with({ minute: 67 }, () => {});
TemporalHelpers.assertPlainDateTime(lambda, 2000, 5, "M05", 2, 12, 59, 0, 0, 0, 0, "default overflow is constrain");

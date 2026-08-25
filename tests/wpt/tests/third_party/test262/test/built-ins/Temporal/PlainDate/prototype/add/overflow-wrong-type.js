// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Type conversions for overflow option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.calendar.prototype.dateadd step 7:
      7. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plaindate.prototype.add step 7:
      7. Return ? CalendarDateAdd(_temporalDate_.[[Calendar]], _temporalDate_, _balancedDuration_, _options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(2000, 5, 2);
const duration = new Temporal.Duration(3, 3, 0, 3);
TemporalHelpers.checkStringOptionWrongType("overflow", "constrain",
  (overflow) => date.add(duration, { overflow }),
  (result, descr) => TemporalHelpers.assertPlainDate(result, 2003, 8, "M08", 5, descr),
);

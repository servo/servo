// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isoyearmonthfromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plainyearmonth.prototype.subtract steps 13–15:
      13. Let _addedDate_ be ? CalendarDateAdd(_calendar_, _date_, _durationToAdd_, _options_).
      14. ...
      15. Return ? YearMonthFromFields(_calendar_, _addedDateFields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// In the ISO calendar, PlainYearMonth.prototype.subtract() actually ignores the
// overflow option. There is no subtraction in the ISO calendar that we could
// test which would actually show a difference between the 'constrain' and
// 'reject' values.
const yearmonth = new Temporal.PlainYearMonth(2000, 5);
const duration = new Temporal.Duration(1, 1);
const explicit = yearmonth.subtract(duration, { overflow: undefined });
TemporalHelpers.assertPlainYearMonth(explicit, 1999, 4, "M04", "default overflow is constrain");
const implicit = yearmonth.subtract(duration, {});
TemporalHelpers.assertPlainYearMonth(implicit, 1999, 4, "M04", "default overflow is constrain");
const lambda = yearmonth.subtract(duration, () => {});
TemporalHelpers.assertPlainYearMonth(lambda, 1999, 4, "M04", "default overflow is constrain");

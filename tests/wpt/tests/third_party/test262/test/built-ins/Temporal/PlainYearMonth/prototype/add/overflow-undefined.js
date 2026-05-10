// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal-isoyearmonthfromfields step 2:
      2. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal.plainyearmonth.prototype.add steps 13–15:
      13. Let _addedDate_ be ? CalendarDateAdd(_calendar_, _date_, _durationToAdd_, _options_).
      14. ...
      15. Return ? YearMonthFromFields(_calendar_, _addedDateFields_, _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// In the ISO calendar, PlainYearMonth.prototype.add() actually ignores the
// overflow option. There is no addition in the ISO calendar that we could test
// which would actually show a difference between the 'constrain' and 'reject'
// values.
const yearmonth = new Temporal.PlainYearMonth(2000, 5);
const duration = new Temporal.Duration(1, 1);
const explicit = yearmonth.add(duration, { overflow: undefined });
TemporalHelpers.assertPlainYearMonth(explicit, 2001, 6, "M06", "default overflow is constrain");
const implicit = yearmonth.add(duration, {});
TemporalHelpers.assertPlainYearMonth(implicit, 2001, 6, "M06", "default overflow is constrain");
const lambda = yearmonth.add(duration, () => {});
TemporalHelpers.assertPlainYearMonth(lambda, 2001, 6, "M06", "default overflow is constrain");

// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: RangeError thrown when adding negative duration and end of month is out of range
features: [Temporal]
info: |
  AddDurationToOrSubtractDurationFromPlainYearMonth:
  12. If _sign_ &lt; 0, then
    a. Let _oneMonthDuration_ be ! CreateTemporalDuration(0, 1, 0, 0, 0, 0, 0, 0, 0, 0).
    b. Let _nextMonth_ be ? CalendarDateAdd(_calendar_, _intermediateDate_, _oneMonthDuration_, *undefined*, _dateAdd_).
    c. Let _endOfMonthISO_ be ! AddISODate(_nextMonth_.[[ISOYear]], _nextMonth_.[[ISOMonth]], _nextMonth_.[[ISODay]], 0, 0, 0, -1, *"constrain"*).
    d. Let _endOfMonth_ be ? CreateTemporalDate(_endOfMonthISO_.[[Year]], _endOfMonthISO_.[[Month]], _endOfMonthISO_.[[Day]], _calendar_).
---*/

// Based on a test case by André Bargull <andre.bargull@gmail.com>

const duration = new Temporal.Duration(0, 0, 0, -1);

// Calendar addition result is out of range
assert.throws(RangeError, () => new Temporal.PlainYearMonth(275760, 9).add(duration), "Addition of 1 month to receiver out of range");

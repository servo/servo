// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: RangeError thrown when calendar part of duration added to relativeTo is out of range
features: [Temporal]
info: |
  RoundDuration:
  10.h. Let _isoResult_ be ! AddISODate(_plainRelativeTo_.[[ISOYear]]. _plainRelativeTo_.[[ISOMonth]], _plainRelativeTo_.[[ISODay]], 0, 0, 0, truncate(_fractionalDays_), *"constrain"*).
    i. Let _wholeDaysLater_ be ? CreateTemporalDate(_isoResult_.[[Year]], _isoResult_.[[Month]], _isoResult_.[[Day]], _calendar_).
  ...
  11.h. Let _isoResult_ be ! AddISODate(_plainRelativeTo_.[[ISOYear]]. _plainRelativeTo_.[[ISOMonth]], _plainRelativeTo_.[[ISODay]], 0, 0, 0, truncate(_fractionalDays_), *"constrain"*).
    i. Let _wholeDaysLater_ be ? CreateTemporalDate(_isoResult_.[[Year]], _isoResult_.[[Month]], _isoResult_.[[Day]], _calendar_).
  ...
  12.a. Let _isoResult_ be ! AddISODate(_plainRelativeTo_.[[ISOYear]]. _plainRelativeTo_.[[ISOMonth]], _plainRelativeTo_.[[ISODay]], 0, 0, 0, truncate(_fractionalDays_), *"constrain"*).
    b. Let _wholeDaysLater_ be ? CreateTemporalDate(_isoResult_.[[Year]], _isoResult_.[[Month]], _isoResult_.[[Day]], _calendar_).

  UnbalanceDateDurationRelative:
  11. Let _yearsMonthsWeeksDuration_ be ! CreateTemporalDuration(_years_, _months_, _weeks_, 0, 0, 0, 0, 0, 0, 0).
  12. Let _later_ be ? CalendarDateAdd(_calendaRec_, _plainRelativeTo_, _yearsMonthsWeeksDuration_).
  13. Let _yearsMonthsWeeksInDays_ be DaysUntil(_plainRelativeTo_, _later_).
  14. Return ? CreateDateDurationRecord(0, 0, 0, _days_ + _yearsMonthsWeeksInDays_).
---*/

// Based on a test case by André Bargull <andre.bargull@gmail.com>

const relativeTo = new Temporal.PlainDate(2000, 1, 1);

{
  const instance = new Temporal.Duration(0, 0, 0, /* days = */ 500_000_000);
  assert.throws(RangeError, () => instance.round({relativeTo, smallestUnit: "years"}), "days out of range, positive, smallestUnit years");
  assert.throws(RangeError, () => instance.round({relativeTo, smallestUnit: "months"}), "days out of range, positive, smallestUnit months");
  assert.throws(RangeError, () => instance.round({relativeTo, smallestUnit: "weeks"}), "days out of range, positive, smallestUnit weeks");

  const negInstance = new Temporal.Duration(0, 0, 0, /* days = */ -500_000_000);
  assert.throws(RangeError, () => negInstance.round({relativeTo, smallestUnit: "years"}), "days out of range, negative, smallestUnit years");
  assert.throws(RangeError, () => negInstance.round({relativeTo, smallestUnit: "months"}), "days out of range, negative, smallestUnit months");
  assert.throws(RangeError, () => negInstance.round({relativeTo, smallestUnit: "weeks"}), "days out of range, negative, smallestUnit weeks");
}

{
  const instance = new Temporal.Duration(0, 0, /* weeks = */ 1, /* days = */ Math.trunc((2 ** 53) / 86_400));
  assert.throws(RangeError, () => instance.round({relativeTo, largestUnit: "days"}), "weeks + days out of range, positive");

  const negInstance = new Temporal.Duration(0, 0, /* weeks = */ -1, /* days = */ -Math.trunc((2 ** 53) / 86_400));
  assert.throws(RangeError, () => instance.round({relativeTo, largestUnit: "days"}), "weeks + days out of range, negative");
}

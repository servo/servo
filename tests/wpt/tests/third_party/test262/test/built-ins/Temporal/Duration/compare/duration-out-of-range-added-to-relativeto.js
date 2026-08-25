// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: RangeError thrown when calendar part of duration added to relativeTo is out of range
features: [Temporal]
info: |
  UnbalanceDateDurationRelative:
  11. Let _yearsMonthsWeeksDuration_ be ! CreateTemporalDuration(_years_, _months_, _weeks_, 0, 0, 0, 0, 0, 0, 0).
  12. Let _later_ be ? CalendarDateAdd(_calendaRec_, _plainRelativeTo_, _yearsMonthsWeeksDuration_).
  13. Let _yearsMonthsWeeksInDays_ be DaysUntil(_plainRelativeTo_, _later_).
  14. Return ? CreateDateDurationRecord(0, 0, 0, _days_ + _yearsMonthsWeeksInDays_).
---*/

// Based on a test case by André Bargull <andre.bargull@gmail.com>

const relativeTo = new Temporal.PlainDate(2000, 1, 1);
const zero = new Temporal.Duration();

const instance = new Temporal.Duration(0, 0, /* weeks = */ 1, /* days = */ Math.trunc((2 ** 53) / 86_400));
assert.throws(RangeError, () => Temporal.Duration.compare(instance, zero, {relativeTo}), "weeks + days out of range, positive, first argument");
assert.throws(RangeError, () => Temporal.Duration.compare(zero, instance, {relativeTo}), "weeks + days out of range, positive, second argument");

const negInstance = new Temporal.Duration(0, 0, /* weeks = */ -1, /* days = */ -Math.trunc((2 ** 53) / 86_400));
assert.throws(RangeError, () => Temporal.Duration.compare(negInstance, zero, {relativeTo}), "weeks + days out of range, negative, first argument");
assert.throws(RangeError, () => Temporal.Duration.compare(zero, negInstance, {relativeTo}), "weeks + days out of range, negative, second argument");

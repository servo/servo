// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.with
description: >
  Basic tests for with(object) with missing properties for non-ISO calendars
features: [Temporal]
---*/

const calendarMonthDay = Temporal.PlainMonthDay.from({ year: 2021, month: 1, day: 15, calendar: "gregory" });
assert.throws(TypeError, () => calendarMonthDay.with({ month: 12 }), "nonIso8601MonthDay.with({month})");

// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.protoype.tostring
description: Type conversions for calendarName option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-toshowcalendaroption step 1:
      1. Return ? GetOption(_normalizedOptions_, *"calendarName"*, « *"string"* », « *"auto"*, *"always"*, *"never"*, *"critical"* », *"auto"*).
    sec-temporal.plainmonthday.protoype.tostring step 4:
      4. Let _showCalendar_ be ? ToShowCalendarOption(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const monthday = new Temporal.PlainMonthDay(5, 2, "iso8601");

TemporalHelpers.checkStringOptionWrongType("calendarName", "auto",
  (calendarName) => monthday.toString({ calendarName }),
  (result, descr) => assert.sameValue(result, "05-02", descr),
);

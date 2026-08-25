// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.protoype.tostring
description: Type conversions for calendarName option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-toshowcalendaroption step 1:
      1. Return ? GetOption(_normalizedOptions_, *"calendarName"*, « *"string"* », « *"auto"*, *"always"*, *"never"*, *"critical"* », *"auto"*).
    sec-temporal.plainyearmonth.protoype.tostring step 4:
      4. Let _showCalendar_ be ? ToShowCalendarOption(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const yearmonth = new Temporal.PlainYearMonth(2000, 5, "gregory");

TemporalHelpers.checkStringOptionWrongType("calendarName", "auto",
  (calendarName) => yearmonth.toString({ calendarName }),
  (result, descr) => assert.sameValue(result, "2000-05-01[u-ca=gregory]", descr),
);

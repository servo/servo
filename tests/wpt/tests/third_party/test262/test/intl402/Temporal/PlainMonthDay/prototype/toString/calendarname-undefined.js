// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.protoype.tostring
description: Fallback value for calendarName option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-toshowcalendaroption step 1:
      1. Return ? GetOption(_normalizedOptions_, *"calendarName"*, « *"string"* », « *"auto"*, *"always"*, *"never"*, *"critical"* », *"auto"*).
    sec-temporal.plainmonthday.protoype.tostring step 4:
      4. Let _showCalendar_ be ? ToShowCalendarOption(_options_).
features: [Temporal]
---*/

const tests = [
  [[], "05-02", "built-in ISO"],
  [["gregory"], "1972-05-02[u-ca=gregory]", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const monthday = new Temporal.PlainMonthDay(5, 2, ...args);
  const result = monthday.toString({ calendarName: undefined });
  assert.sameValue(result, expected, `default calendarName option is auto with ${description} calendar`);
  // See options-object.js for {} and options-undefined.js for absent options arg
}

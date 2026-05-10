// Copyright 2019 Googe Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks the order of getting options of 'dayPeriod' for the DateTimeFormat constructor.
info: |
  CreateDateTimeFormat ( newTarget, locales, options, required, defaults )
  ...
  36. For each row of Table 7, except the header row, in table order, do
    a. Let prop be the name given in the Property column of the row.
    b. If prop is "fractionalSecondDigits", then
      i. Let value be ? GetNumberOption(options, "fractionalSecondDigits", 1, 3, undefined).
    c. Else,
      i. Let values be a List whose elements are the strings given in the Values column of the row.
      ii. Let value be ? GetOption(options, prop, string, values, undefined).
    d. Set formatOptions.[[<prop>]] to value.
  ...
includes: [compareArray.js]
features: [Intl.DateTimeFormat-dayPeriod]

---*/

// Just need to ensure dayPeriod are get between day and hour.
const expected = [
  // CreateDateTimeFormat step 36.
  "day",
  "dayPeriod",
  "hour"
];

const actual = [];

const options = {
  get day() {
    actual.push("day");
    return "numeric";
  },
  get dayPeriod() {
    actual.push("dayPeriod");
    return "long";
  },
  get hour() {
    actual.push("hour");
    return "numeric";
  },
};

new Intl.DateTimeFormat("en", options);
assert.compareArray(actual, expected);

// Copyright 2019 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Checks error cases for the options argument to the DateTimeFormat constructor.
info: |
    CreateDateTimeFormat ( dateTimeFormat, locales, options, required, defaults )
    ...
    37. For each row of Table 7, except the header row, in table order, do
      a. Let prop be the name given in the Property column of the row.
      b. If prop is "fractionalSecondDigits", then
          i. Let value be ? GetNumberOption(options, "fractionalSecondDigits", 1, 3, undefined).
features: [Intl.DateTimeFormat-fractionalSecondDigits]
---*/


const invalidOptions = [
  "LONG",
  " long",
  "short ",
  "full",
  "numeric",
  -1,
  4,
  "4",
  "-1",
  -0.00001,
  3.000001,
];
for (const fractionalSecondDigits of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.DateTimeFormat("en", { fractionalSecondDigits });
  },
  `new Intl.DateTimeFormat("en", { fractionalSecondDigits: "${fractionalSecondDigits}" }) throws RangeError`);
}

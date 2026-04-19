// Copyright 2019 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Checks handling of the options argument to the DateTimeFormat constructor.
info: |
    CreateDateTimeFormat ( dateTimeFormat, locales, options, required, defaults )
    ...
    37. For each row of Table 7, except the header row, in table order, do
      a. Let prop be the name given in the Property column of the row.
      b. If prop is "fractionalSecondDigits", then
        i. Let value be ? GetNumberOption(options, "fractionalSecondDigits", 1, 3, undefined).
features: [Intl.DateTimeFormat-fractionalSecondDigits]
---*/


const validOptions = [
  [undefined, undefined],
  [1, 1],
  ["1", 1],
  [2, 2],
  ["2", 2],
  [3, 3],
  ["3", 3],
  [2.9, 2],
  ["2.9", 2],
  [1.00001, 1],
  [{ toString() { return "3"; } }, 3],
];
for (const [fractionalSecondDigits, expected] of validOptions) {
  const dtf = new Intl.DateTimeFormat("en", { fractionalSecondDigits });
  const options = dtf.resolvedOptions();
  assert.sameValue(options.fractionalSecondDigits, expected);
  const propdesc = Object.getOwnPropertyDescriptor(options, "fractionalSecondDigits");
  if (expected === undefined) {
    assert.sameValue(propdesc, undefined);
  } else {
    assert.sameValue(propdesc.value, expected);
  }
}

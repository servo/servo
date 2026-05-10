// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of invalid value for the type option to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
    7. Let type be GetOption(options, "type", "string", « "conjunction", "disjunction", "unit" », "conjunction").
features: [Intl.ListFormat]
---*/

const invalidOptions = [
  null,
  1,
  "",
  "Conjunction",
  "CONJUNCTION",
  "conjunction\0",
  "Disjunction",
  "DISJUNCTION",
  "disjunction\0",
  "Unit",
  "UNIT",
  "unit\0",
];

for (const invalidOption of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.ListFormat([], {"type": invalidOption});
  }, `${invalidOption} is an invalid type option value`);
}

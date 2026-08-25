// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of invalid value for the style option to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
    9. Let s be ? GetOption(options, "style", "string", «"long", "short", "narrow"», "long").
features: [Intl.ListFormat]
---*/

const invalidOptions = [
  null,
  1,
  "",
  "Long",
  "LONG",
  "long\0",
  "Short",
  "SHORT",
  "short\0",
  "Narrow",
  "NARROW",
  "narrow\0",
];

for (const invalidOption of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.ListFormat([], {"style": invalidOption});
  }, `${invalidOption} is an invalid style option value`);
}

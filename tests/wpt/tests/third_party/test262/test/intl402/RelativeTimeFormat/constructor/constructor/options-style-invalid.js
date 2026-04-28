// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks handling of invalid value for the style option to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    14. Let s be ? GetOption(options, "style", "string", «"long", "short", "narrow"», "long").
features: [Intl.RelativeTimeFormat]
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
    new Intl.RelativeTimeFormat([], {"style": invalidOption});
  }, `${invalidOption} is an invalid style option value`);
}

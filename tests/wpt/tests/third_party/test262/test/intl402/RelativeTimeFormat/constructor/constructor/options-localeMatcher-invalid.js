// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks handling of invalid value for the localeMatcher option to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    7. Let matcher be ? GetOption(options, "localeMatcher", "string", «"lookup", "best fit"», "best fit").
features: [Intl.RelativeTimeFormat]
---*/

const invalidOptions = [
  null,
  1,
  "",
  "Lookup",
  "LOOKUP",
  "lookup\0",
  "Best fit",
  "BEST FIT",
  "best\u00a0fit",
];

for (const invalidOption of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.RelativeTimeFormat([], {"localeMatcher": invalidOption});
  }, `${invalidOption} is an invalid localeMatcher option value`);
}

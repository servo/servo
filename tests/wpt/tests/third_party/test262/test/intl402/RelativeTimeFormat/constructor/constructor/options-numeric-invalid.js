// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks handling of invalid value for the numeric option to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    16. Let numeric be ? GetOption(options, "numeric", "string", «"always", "auto"», "always").
features: [Intl.RelativeTimeFormat]
---*/

assert.sameValue(typeof Intl.RelativeTimeFormat, "function");

const invalidOptions = [
  null,
  1,
  "",
  "Always",
  "ALWAYS",
  "always\0",
  "Auto",
  "AUTO",
  "auto\0",
];

for (const invalidOption of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.RelativeTimeFormat([], {"numeric": invalidOption});
  }, `${invalidOption} is an invalid numeric option value`);
}

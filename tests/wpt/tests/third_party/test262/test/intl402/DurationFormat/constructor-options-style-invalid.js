// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks handling of invalid value for the style option to the DurationFormat constructor.
info: |
    InitializeDurationFormat (DurationFormat, locales, options)
    (...)
    13. Let style be ? GetOption(options, "style", "string", « "long", "short", "narrow", "digital" », "long").
    14. Set durationFormat.[[Style]] to style.
features: [Intl.DurationFormat]
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
  "Digital",
  "DIGITAL",
  "digital\0",
];

for (const invalidOption of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.DurationFormat([], {"style": invalidOption});
  }, `${invalidOption} is an invalid style option value`);
}

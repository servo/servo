// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Tests that the option fractionalDigits is processed correctly.
info: |
    Intl.DurationFormat ( [ locales [ , options ] ] )
    (...)
    18. Set durationFormat.[[FractionalDigits]] to ? GetNumberOption(options, "fractionalDigits", 0, 9, undefined).
features: [Intl.DurationFormat]
---*/

const invalidOptions = [
  -10,
  10
];

for (const fractionalDigits of invalidOptions) {
  assert.throws(RangeError, function() {
    new Intl.DurationFormat("en", { fractionalDigits });
  }, `new Intl.DurationFormat("en", {fractionalDigits: "${fractionalDigits}"}) throws RangeError`);
}

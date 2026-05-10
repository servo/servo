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

const validOptions = [
  0,
  1,
  5,
  9,
  undefined
];

for (const fractionalDigits of validOptions) {
  const obj = new Intl.DurationFormat("en", {fractionalDigits});
  assert.sameValue(obj.resolvedOptions().fractionalDigits, fractionalDigits, `${fractionalDigits} is supported by DurationFormat`);
}

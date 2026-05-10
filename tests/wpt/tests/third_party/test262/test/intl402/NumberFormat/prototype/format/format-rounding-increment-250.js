// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.format
description: When set to `250`, roundingIncrement is correctly applied
features: [Intl.NumberFormat-v3]
includes: [testIntl.js]
---*/

var locales = [
  new Intl.NumberFormat().resolvedOptions().locale, 'ar', 'de', 'th', 'ja'
];
var numberingSystems = ['arab', 'latn', 'thai', 'hanidec'];

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 250, maximumFractionDigits: 3, minimumFractionDigits: 3},
  {
    '1.2500': '1.250',
    '1.3125': '1.250',
    '1.3750': '1.500',
    '1.4375': '1.500',
    '1.5000': '1.500',
  }
);

testNumberFormat(
  locales,
  numberingSystems,
  {roundingIncrement: 250, maximumFractionDigits: 4, minimumFractionDigits: 4},
  {
    '1.02500': '1.0250',
    '1.03125': '1.0250',
    '1.03750': '1.0500',
    '1.04375': '1.0500',
    '1.05000': '1.0500',
  }
);
